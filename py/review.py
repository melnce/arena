#!/usr/bin/env python3
"""Replay captured vs-bot games against a deep reference and rank blunders.

For actions that keep the turn, a blunder is a decision whose reference
value is worse than the reference's own choice: cost = max(0, delta)
where delta = v_best − v_played (signed; search noise goes both ways).

Turn-passing actions (end_turn, or anything that hands the seat to the
opponent) do not get a `cost`. Both searches flatter the side to move, so
v_best − v_played is not a mistake score — it is the bot's end-of-turn
valuation minus a real search from the opponent's seat. That quantity is
emitted as `optimism` and kept out of the blunder ranking and every cost
aggregate.

Hidden information is where a shallow read goes wrong, so the default
reference uses four times H0's usual determinizations (`k=16`).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import statistics
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from runlib import (  # noqa: E402
    git_commit_argv,
    git_head,
    load_run,
    mark_end as runlib_mark_end,
    mark_start as runlib_mark_start,
    new_run,
    publish_tag,
    repo_root,
    run_tee,
    save_run as runlib_save_run,
)


DEFAULT_REF = "h0:nodes=200000,k=16"
DEFAULT_WV = 80.0
DEFAULT_TAG = "review1"
HUMAN_SEAT = "a"
BOT_SEAT = "b"
TURN_BANDS = ("1-3", "4-6", "7-9", "10+")
KIND_ORDER = ("play", "attack", "evolve", "end-turn", "ability")
COST_KIND_ORDER = ("play", "attack", "evolve", "ability")
LETHAL_EPS = 1e-6
HUMAN_SIDE_WARN = "no humanSide in capture; assuming human = seat a"
ACTION_META_KEYS = frozenset({"value", "bot_value"})


def seed_for(game_id: str, ply: int, extra: int = 0) -> int:
    """Deterministic u64 from `(game_id, ply[, extra])` so a rerun matches."""
    blob = f"{game_id}:{ply}:{extra}".encode("utf-8")
    return int.from_bytes(hashlib.sha256(blob).digest()[:8], "little")


def parse_wv(spec: str) -> float:
    for token in spec.split(","):
        token = token.strip()
        if token.startswith("wv="):
            try:
                return float(token.split("=", 1)[1])
            except ValueError:
                return DEFAULT_WV
    return DEFAULT_WV


def action_player(step: dict[str, Any]) -> str | None:
    if "reseed" in step:
        return None
    for key in (
        "mulligan",
        "play",
        "attack",
        "evolve",
        "engage",
        "fuse",
        "bonus_pp",
        "choose",
        "confirm",
        "end_turn",
    ):
        inner = step.get(key)
        if isinstance(inner, dict) and inner.get("player") in ("a", "b"):
            return str(inner["player"])
    return None


def action_kind(step: dict[str, Any]) -> str:
    if "play" in step:
        return "play"
    if "attack" in step:
        return "attack"
    if "evolve" in step:
        return "evolve"
    if "end_turn" in step:
        return "end-turn"
    return "ability"


def acting_of(game: Any) -> str:
    legal = game.legal()
    if legal:
        who = action_player(legal[0])
        if who:
            return who
    return str(game.active)


def apply_step(game: Any, step: dict[str, Any]) -> None:
    if "reseed" in step:
        game.reseed(int(step["reseed"]))
        return
    if any(k in step for k in ACTION_META_KEYS):
        step = {k: v for k, v in step.items() if k not in ACTION_META_KEYS}
    game.apply(step)


def passes_turn(played: Any, *, seat_changed: bool | None = None) -> bool:
    """True for end_turn, or when the caller saw the acting seat change."""
    if seat_changed is True:
        return True
    return isinstance(played, dict) and "end_turn" in played


def action_body(step: Any) -> Any:
    if not isinstance(step, dict):
        return step
    return {k: v for k, v in step.items() if k not in ACTION_META_KEYS}


def actions_equal(a: Any, b: Any) -> bool:
    return json.dumps(action_body(a), sort_keys=True, separators=(",", ":")) == json.dumps(
        action_body(b), sort_keys=True, separators=(",", ":")
    )


def recorded_human_side(log: dict[str, Any]) -> str | None:
    human = log.get("humanSide") or log.get("human")
    return human if human in ("a", "b") else None


def decorate_decision(
    rec: dict[str, Any],
    *,
    turn_passing: bool | None = None,
) -> dict[str, Any]:
    """Attach signed `delta` / `optimism`; drop `cost` on turn-passing actions."""
    out = dict(rec)
    passing = passes_turn(out.get("played"), seat_changed=turn_passing)
    v_best = float(out["v_best"])
    v_played = float(out["v_played"])
    delta = v_best - v_played
    out.pop("clamped_negative", None)
    if passing:
        out["optimism"] = delta
        out.pop("cost", None)
        out.pop("delta", None)
    else:
        out["delta"] = delta
        out["cost"] = 0.0 if delta < 0 else delta
        out.pop("optimism", None)
    return out


def is_analysis(data: dict[str, Any]) -> bool:
    dec = data.get("decisions")
    if not isinstance(dec, list) or not dec:
        return False
    first = dec[0]
    return isinstance(first, dict) and "v_best" in first and "v_played" in first


def refresh_analysis(data: dict[str, Any], *, ref: str) -> dict[str, Any]:
    """Re-decorate a previously analysed game without re-searching."""
    out = dict(data)
    decisions = [
        decorate_decision(d) for d in (data.get("decisions") or []) if isinstance(d, dict)
    ]
    out["decisions"] = [
        {k: v for k, v in d.items() if k not in ("clamped_negative", "ms", "kind")}
        for d in decisions
    ]
    out["clamped_negative"] = sum(
        1 for d in decisions if isinstance(d.get("delta"), (int, float)) and float(d["delta"]) < 0
    )
    out.setdefault("ref", ref)
    return out


def decision_delta(d: dict[str, Any]) -> float:
    if "delta" in d:
        return float(d["delta"])
    if "optimism" in d:
        return float(d["optimism"])
    return float(d["v_best"]) - float(d["v_played"])


def clamped_cost(d: dict[str, Any]) -> float:
    if "cost" in d:
        return float(d["cost"])
    raw = decision_delta(d)
    return 0.0 if raw < 0 else raw


def review_stats(
    games: list[dict[str, Any]],
    *,
    wv: float,
    mean_ms: float,
    clamped_total: int,
) -> dict[str, Any]:
    decisions = [d for g in games for d in (g.get("decisions") or [])]
    end_turns = [d for d in decisions if passes_turn(d.get("played"))]
    lethal = [
        d for d in end_turns if float(d.get("v_played") or 0) <= -wv + LETHAL_EPS
    ]
    opts = [decision_delta(d) for d in end_turns]
    n_games = len(games)
    same = [
        d
        for d in decisions
        if actions_equal(d.get("played"), d.get("reference"))
        and not passes_turn(d.get("played"))
    ]
    noise = [clamped_cost(d) for d in same]
    return {
        "games": n_games,
        "analysed_end_turns": len(end_turns),
        "lethal_walks": len(lethal),
        "mean_optimism": mean(opts),
        "end_turns_per_game": (len(end_turns) / n_games) if n_games else 0.0,
        "lethal_per_game": (len(lethal) / n_games) if n_games else 0.0,
        "noise_n": len(same),
        "noise_mean": mean(noise),
        "noise_std": statistics.stdev(noise) if len(noise) > 1 else 0.0,
        "noise_max": max(noise) if noise else 0.0,
        "clamped_total": clamped_total,
        "mean_ms": mean_ms,
        "wv": wv,
    }


def rebuild_to(db: Any, log: dict[str, Any], ply: int) -> Any:
    import arena

    game = arena.Game(
        db,
        int(log["seed"]),
        log["deckA"],
        log["deckB"],
        str(log.get("first") or "coin"),
    )
    for step in (log.get("actions") or [])[:ply]:
        if not isinstance(step, dict):
            raise ValueError("action must be an object")
        apply_step(game, step)
    return game


def position_summary(full: dict[str, Any]) -> dict[str, Any]:
    players = full.get("players") or {}

    def side(pid: str) -> dict[str, Any]:
        p = players.get(pid) or {}
        field = p.get("field") or []
        hand = p.get("hand") or []
        return {
            "hp": int(p.get("leader_defense") or 0),
            "pp": int(p.get("pp") or 0),
            "hand": len(hand),
            "board": sum(1 for c in field if c),
            "ep": int(p.get("ep") or 0),
            "sep": int(p.get("sep") or 0),
        }

    return {"a": side("a"), "b": side("b")}


def load_card_names(cards_root: Path) -> dict[str, str]:
    names: dict[str, str] = {}
    if not cards_root.is_dir():
        return names
    for path in cards_root.rglob("*.json"):
        if "official" in path.parts:
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if not isinstance(data, dict):
            continue
        name = data.get("name")
        if not isinstance(name, str) or not name:
            continue
        cid = str(data.get("id") or path.stem)
        names[cid] = name
    return names


def seats_for(side: str, log: dict[str, Any]) -> set[str]:
    human = recorded_human_side(log) or HUMAN_SEAT
    bot = "b" if human == "a" else "a"
    if side == "bot":
        return {bot}
    if side == "human":
        return {human}
    return {"a", "b"}


def turn_band(turn: int) -> str:
    if turn <= 3:
        return "1-3"
    if turn <= 6:
        return "4-6"
    if turn <= 9:
        return "7-9"
    return "10+"


def dumps(obj: Any) -> str:
    return json.dumps(obj, indent=2, sort_keys=True) + "\n"


def collect_game_paths(items: list[Path]) -> list[Path]:
    out: list[Path] = []
    for item in items:
        if item.is_dir():
            out.extend(sorted(p for p in item.glob("*.json") if p.is_file()))
        elif item.is_file():
            out.append(item)
    return out


def load_input(path: Path) -> tuple[str, dict[str, Any]] | None:
    """Load a raw capture or a previously written per-game analysis."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if not isinstance(data, dict):
        return None
    if is_analysis(data):
        if not data.get("game_id"):
            data["game_id"] = path.stem
        return ("analysis", data)
    if "actions" not in data or "deckA" not in data or "deckB" not in data:
        return None
    if not data.get("game_id"):
        data["game_id"] = path.stem
    return ("capture", data)


def load_game_log(path: Path) -> dict[str, Any] | None:
    loaded = load_input(path)
    if loaded is None:
        return None
    kind, data = loaded
    return data if kind == "capture" else None


def analyse_decision(
    db: Any,
    log: dict[str, Any],
    ply: int,
    ref: str,
    wv: float,
) -> dict[str, Any] | None:
    actions = log.get("actions") or []
    step = actions[ply]
    if not isinstance(step, dict) or "reseed" in step:
        return None
    game_id = str(log["game_id"])
    game = rebuild_to(db, log, ply)
    if game.phase == "mulligan" or game.terminal:
        return None
    legal = game.legal()
    if len(legal) <= 1:
        return None
    acting = action_player(step) or acting_of(game)
    t0 = time.perf_counter()
    ref_out = game.bot_action_value(ref, seed_for(game_id, ply))
    best_ms = (time.perf_counter() - t0) * 1000.0
    best_action = ref_out["action"]
    v_best = ref_out["value"]
    clone = game.clone()
    clone.apply(action_body(step))
    played_ms = 0.0
    if clone.terminal:
        winner = clone.winner
        if winner == acting:
            v_played: float | None = wv
        elif winner is None:
            v_played = 0.0
        else:
            v_played = -wv
    else:
        t1 = time.perf_counter()
        after = clone.bot_action_value(ref, seed_for(game_id, ply, 1))
        played_ms = (time.perf_counter() - t1) * 1000.0
        v_after = after["value"]
        if v_after is None:
            v_played = None
        elif acting_of(clone) != acting:
            v_played = -float(v_after)
        else:
            v_played = float(v_after)
    if v_best is None or v_played is None:
        return None
    v_best_f = float(v_best)
    v_played_f = float(v_played)
    turn_passing = (not clone.terminal and acting_of(clone) != acting) or (
        "end_turn" in step
    )
    full = game.full()
    bot_value = None
    if isinstance(step.get("value"), (int, float)):
        bot_value = float(step["value"])
    elif isinstance(step.get("bot_value"), (int, float)):
        bot_value = float(step["bot_value"])
    rec = {
        "ply": ply,
        "turn": int(game.turn),
        "phase": str(game.phase),
        "acting": acting,
        "played": action_body(step),
        "reference": best_action,
        "v_played": v_played_f,
        "v_best": v_best_f,
        "bot_value": bot_value,
        "position": position_summary(full),
        "ms": best_ms + played_ms,
        "kind": action_kind(step),
    }
    return decorate_decision(rec, turn_passing=turn_passing)


def analyse_game(
    db: Any,
    log: dict[str, Any],
    *,
    ref: str,
    side: str,
    threads: int,
    names: dict[str, str] | None = None,
) -> dict[str, Any]:
    names = names if names is not None else {}
    seats = seats_for(side, log)
    wv = parse_wv(ref)
    actions = log.get("actions") or []
    jobs: list[int] = []
    for ply, step in enumerate(actions):
        if not isinstance(step, dict) or "reseed" in step:
            continue
        who = action_player(step)
        if who not in seats:
            continue
        jobs.append(ply)

    decisions: list[dict[str, Any]] = []
    workers = max(1, threads)
    if workers == 1 or len(jobs) <= 1:
        for ply in jobs:
            rec = analyse_decision(db, log, ply, ref, wv)
            if rec:
                decisions.append(rec)
    else:
        with ThreadPoolExecutor(max_workers=workers) as pool:
            futs = {
                pool.submit(analyse_decision, db, log, ply, ref, wv): ply
                for ply in jobs
            }
            for fut in as_completed(futs):
                rec = fut.result()
                if rec:
                    decisions.append(rec)
        decisions.sort(key=lambda r: int(r["ply"]))

    clamped = sum(
        1 for d in decisions if isinstance(d.get("delta"), (int, float)) and float(d["delta"]) < 0
    )
    times = [float(d["ms"]) for d in decisions if "ms" in d]
    winner = log.get("winner")
    if winner not in ("a", "b", None):
        winner = None
    turns = 0
    if decisions:
        turns = max(int(d["turn"]) for d in decisions)
    elif isinstance(log.get("turn"), int):
        turns = int(log["turn"])
    clean = []
    for d in decisions:
        row = {k: v for k, v in d.items() if k not in ("clamped_negative", "ms", "kind")}
        clean.append(row)
    return {
        "v": 1,
        "game_id": log.get("game_id"),
        "seed": log.get("seed"),
        "deckA": log.get("deckA"),
        "deckB": log.get("deckB"),
        "first": log.get("first"),
        "winner": winner,
        "policy": log.get("policy") or "",
        "strong": log.get("strong") or "",
        "ref": ref,
        "side": side,
        "turns": turns,
        "decisions": clean,
        "clamped_negative": clamped,
        "mean_ms": (sum(times) / len(times)) if times else 0.0,
        "decision_ms": times,
    }


def render_action(action: Any, names: dict[str, str]) -> str:
    if not isinstance(action, dict):
        return str(action)
    if "play" in action:
        card = str((action["play"] or {}).get("card") or "")
        return f"play {names.get(card, card)}"
    if "attack" in action:
        att = action["attack"] or {}
        slot = att.get("attacker_slot")
        target = att.get("target")
        if target == "leader" or (isinstance(target, dict) and "leader" in target):
            tgt = "leader"
        elif isinstance(target, dict) and "slot" in target:
            tgt = f"slot {target['slot']}"
        else:
            tgt = str(target)
        return f"attack slot {slot} → {tgt}"
    if "evolve" in action:
        ev = action["evolve"] or {}
        kind = "super-evolve" if ev.get("super") else "evolve"
        return f"{kind} slot {ev.get('slot')}"
    if "end_turn" in action:
        return "end-turn"
    if "mulligan" in action:
        swap = (action["mulligan"] or {}).get("swap")
        return f"mulligan {swap}"
    if "engage" in action:
        return f"engage slot {(action['engage'] or {}).get('slot')}"
    if "fuse" in action:
        return f"fuse host {(action['fuse'] or {}).get('host_pos')}"
    if "bonus_pp" in action:
        return "bonus-pp"
    if "choose" in action:
        return f"choose {(action['choose'] or {}).get('option')}"
    if "confirm" in action:
        return "confirm"
    if "reseed" in action:
        return f"reseed {action['reseed']}"
    return json.dumps(action, sort_keys=True, separators=(",", ":"))


def render_position(pos: dict[str, Any]) -> str:
    def fmt(pid: str) -> str:
        s = pos.get(pid) or {}
        return (
            f"{pid.upper()} HP {s.get('hp', '?')} PP {s.get('pp', '?')} "
            f"hand {s.get('hand', '?')} board {s.get('board', '?')} "
            f"EP {s.get('ep', '?')} SEP {s.get('sep', '?')}"
        )

    return f"{fmt('a')}; {fmt('b')}"


def value_trace(decisions: list[dict[str, Any]]) -> list[tuple[int, float]]:
    seen: set[int] = set()
    out: list[tuple[int, float]] = []
    for d in decisions:
        turn = int(d["turn"])
        if turn in seen:
            continue
        seen.add(turn)
        out.append((turn, float(d["v_best"])))
    return out


def format_deck(deck: Any, names: dict[str, str]) -> str:
    if not isinstance(deck, dict):
        return str(deck)
    parts = []
    for cid, n in sorted(deck.items(), key=lambda kv: str(kv[0])):
        label = names.get(str(cid), str(cid))
        parts.append(f"{n}× {label}")
    return ", ".join(parts) if parts else "(empty)"


def mean(xs: list[float]) -> float:
    return sum(xs) / len(xs) if xs else 0.0


def build_review_md(
    games: list[dict[str, Any]],
    *,
    ref: str,
    top: int,
    names: dict[str, str],
    name_source: str,
    clamped_total: int,
    mean_ms: float,
) -> str:
    wv = parse_wv(ref)
    if games:
        wv = parse_wv(str(games[0].get("ref") or ref))
    stats = review_stats(games, wv=wv, mean_ms=mean_ms, clamped_total=clamped_total)
    two_sigma = 2.0 * float(stats["noise_std"])
    lines: list[str] = []
    lines.append("# Game review")
    lines.append("")
    lines.append(f"- games: {len(games)}")
    lines.append(f"- reference: `{ref}`")
    lines.append(f"- card names: {name_source}")
    lines.append(
        f"- lethal-walk end-turns: {stats['lethal_walks']} / {stats['analysed_end_turns']} "
        f"({stats['lethal_per_game']:.2f} per game; "
        f"{stats['end_turns_per_game']:.2f} analysed end-turns/game)"
    )
    lines.append(
        f"- mean end-of-turn optimism: {stats['mean_optimism']:.1f} "
        f"(n={stats['analysed_end_turns']}; "
        f"{stats['end_turns_per_game']:.2f} analysed end-turns/game)"
    )
    lines.append(
        f"- noise floor (same-action, true cost 0): n={stats['noise_n']} "
        f"mean={stats['noise_mean']:.2f} σ={stats['noise_std']:.2f} "
        f"max={stats['noise_max']:.1f}"
    )
    lines.append(
        f"- any single cost below roughly {two_sigma:.1f} (2σ) is "
        "indistinguishable from measurement noise"
    )
    lines.append(f"- reference mean decision time: {mean_ms:.1f} ms")
    lines.append("")

    for g in games:
        gid = g.get("game_id")
        lines.append(f"## `{gid}`")
        lines.append("")
        lines.append(f"- decks A: {format_deck(g.get('deckA'), names)}")
        lines.append(f"- decks B: {format_deck(g.get('deckB'), names)}")
        lines.append(f"- winner: {g.get('winner')}")
        lines.append(f"- turns: {g.get('turns')}")
        lines.append(f"- bot spec: `{g.get('policy') or 'n/a'}`")
        lines.append(f"- reference: `{g.get('ref') or ref}`")
        lines.append("")
        lines.append("Value trace (reference value at the start of each analysed turn):")
        lines.append("")
        trace = value_trace(g.get("decisions") or [])
        if not trace:
            lines.append("- (no analysed decisions)")
        for turn, val in trace:
            lines.append(f"- turn {turn}: {val:.3f}")
        lines.append("")

    blunders: list[tuple[dict[str, Any], dict[str, Any]]] = []
    for g in games:
        for d in g.get("decisions") or []:
            if "cost" not in d:
                continue
            blunders.append((g, d))
    blunders.sort(key=lambda pair: (-float(pair[1].get("cost") or 0), int(pair[1].get("ply") or 0)))
    lines.append(f"## Top {top} blunders")
    lines.append("")
    lines.append(
        "Turn-passing actions are omitted (they have `optimism`, not `cost`). "
        f"Any single cost below roughly {two_sigma:.1f} (2σ of the same-action "
        "noise floor) is indistinguishable from measurement noise."
    )
    lines.append("")
    if not blunders:
        lines.append("None.")
        lines.append("")
    for i, (g, d) in enumerate(blunders[:top], 1):
        played = render_action(d.get("played"), names)
        ref_act = render_action(d.get("reference"), names)
        delta = float(d.get("delta") if d.get("delta") is not None else decision_delta(d))
        lines.append(
            f"{i}. `{g.get('game_id')}` turn {d.get('turn')} {d.get('phase')} "
            f"({d.get('acting')}): played {played}, reference says {ref_act}. "
            f"v_played={float(d.get('v_played') or 0):.3f} "
            f"v_best={float(d.get('v_best') or 0):.3f} "
            f"delta={delta:.3f} "
            f"cost={float(d.get('cost') or 0):.3f}. "
            f"{render_position(d.get('position') or {})}"
        )
    if blunders:
        lines.append("")

    lines.append("## Aggregate")
    lines.append("")
    lines.append(
        "Turn-passing actions are excluded. `delta` is signed (`v_best − v_played`); "
        "it is not clamped, so search noise in both directions cancels in the mean."
    )
    lines.append("")

    def costed_rows() -> list[dict[str, Any]]:
        return [
            d
            for g in games
            for d in (g.get("decisions") or [])
            if "delta" in d or "cost" in d
        ]

    def bucket_lines(title: str, keys: list[str], key_of) -> None:
        lines.append(f"### {title}")
        lines.append("")
        lines.append("| bucket | n | mean delta | total delta |")
        lines.append("|---|---:|---:|---:|")
        pool = costed_rows()
        for key in keys:
            rows = [d for d in pool if key_of(d) == key]
            deltas = [decision_delta(d) for d in rows]
            lines.append(
                f"| {key} | {len(rows)} | {mean(deltas):.3f} | {sum(deltas):.3f} |"
            )
        lines.append("")

    bucket_lines("by turn band", list(TURN_BANDS), lambda d: turn_band(int(d.get("turn") or 0)))
    bucket_lines("by action kind", list(COST_KIND_ORDER), lambda d: action_kind(d.get("played") or {}))
    bucket_lines("by side", ["a", "b"], lambda d: d.get("acting"))

    lines.append("### end-of-turn")
    lines.append("")
    lines.append(
        f"- turns ended with the opponent holding lethal "
        f"(v_played ≤ −{stats['wv']:g}): "
        f"{stats['lethal_walks']} / {stats['analysed_end_turns']} "
        f"({stats['lethal_per_game']:.2f} per game; "
        f"{stats['end_turns_per_game']:.2f} analysed end-turns/game)"
    )
    lines.append(
        f"- mean end-of-turn optimism: {stats['mean_optimism']:.1f} "
        f"(n={stats['analysed_end_turns']}; "
        f"{stats['end_turns_per_game']:.2f} analysed end-turns/game)"
    )
    lines.append(
        "- `optimism` is the bot's own end-of-turn valuation minus a real "
        "search from the opponent's seat — not a blunder score."
    )
    lines.append("")
    lines.append(
        f"noise floor (same-action): n={stats['noise_n']} "
        f"mean={stats['noise_mean']:.2f} σ={stats['noise_std']:.2f} "
        f"max={stats['noise_max']:.1f}"
    )
    lines.append(f"reference mean decision time: {mean_ms:.1f} ms")
    lines.append("")
    return "\n".join(lines)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    raw = list(sys.argv[1:] if argv is None else argv)
    p = argparse.ArgumentParser(
        prog="py/review.py",
        description=(
            "Replay captured vs-bot games against a deep reference search and "
            "rank the bot's blunders. A blunder is cost = max(0, delta) for "
            "actions that keep the turn (delta = v_best − v_played, signed). "
            "Turn-passing actions emit optimism instead of cost."
        ),
    )
    p.add_argument(
        "--games",
        nargs="+",
        default=None,
        metavar="PATH",
        help="game JSON files or directories (default: results/games)",
    )
    p.add_argument(
        "--ref",
        default=DEFAULT_REF,
        help=(
            f'reference policy spec (default: "{DEFAULT_REF}"). '
            "k=16 is four times H0's usual determinizations — hidden information "
            "is where a shallow read goes wrong."
        ),
    )
    p.add_argument(
        "--side",
        choices=("bot", "human", "both"),
        default="bot",
        help="which seat to analyse (default: bot = B unless the log sets humanSide)",
    )
    p.add_argument("--tag", default=DEFAULT_TAG, help=f"run name (default: {DEFAULT_TAG})")
    p.add_argument(
        "--threads",
        type=int,
        default=None,
        help="decision thread pool (default: all CPUs)",
    )
    p.add_argument("--top", type=int, default=25, help="blunders listed in REVIEW.md (default: 25)")
    p.add_argument(
        "--out",
        default=None,
        help="output directory (default: results/<tag>)",
    )
    p.add_argument("--publish", action="store_true", help="copy artifacts to the results worktree and push")
    p.add_argument("--publish-remote", default="origin")
    p.add_argument("--publish-branch", default="results")
    p.add_argument(
        "--publish-dir",
        default=None,
        help="git worktree of the orphan results branch (default: <repo>/../arena-results-wt)",
    )
    p.add_argument(
        "--cards",
        default=None,
        help="cards/ directory or repo root (default: repo cards/)",
    )
    p.add_argument("--only", default=None, help="analyse only this game_id")
    p.add_argument("--force", action="store_true", help="redo games whose output already exists")
    args = p.parse_args(raw)
    args._argv = raw
    return args


class Runner:
    def __init__(self, args: argparse.Namespace) -> None:
        self.args = args
        self.repo = repo_root()
        root = Path(args.out) if args.out else (self.repo / "results" / args.tag)
        self.tag_dir = root.resolve()
        self.tag_dir.mkdir(parents=True, exist_ok=True)
        pub = args.publish_dir
        self.publish_dir = (
            Path(pub).resolve() if pub else (self.repo / ".." / "arena-results-wt").resolve()
        )
        self.run_path = self.tag_dir / "RUN.json"
        self.run: dict[str, Any] = load_run(self.run_path) or new_run(self._argv_list(), self.repo)
        cards = Path(args.cards) if args.cards else (self.repo / "cards")
        self.cards = cards
        self.names = load_card_names(cards if cards.name == "cards" or (cards / "cards").is_dir() else cards)
        if (cards / "cards").is_dir() and not self.names:
            self.names = load_card_names(cards / "cards")
        self.name_source = (
            "card JSON `name` fields under cards/ (plus full() instance names)"
            if self.names
            else "ids (card db names unavailable); full() instance names when present"
        )
        self.threads = args.threads if args.threads and args.threads > 0 else (os.cpu_count() or 1)
        self.human_side_assumed = False

    def _argv_list(self) -> list[str]:
        return [sys.executable, str(Path(__file__).resolve()), *self.args._argv]

    def save_run(self) -> None:
        runlib_save_run(self.run_path, self.run, self._argv_list(), self.repo)

    def mark_start(self, stage: str) -> None:
        runlib_mark_start(self.run, self.run_path, stage, self._argv_list(), self.repo)

    def mark_end(self, stage: str) -> None:
        runlib_mark_end(self.run, self.run_path, stage, self._argv_list(), self.repo)

    def game_inputs(self) -> list[Path]:
        raw = self.args.games
        if raw:
            items = [Path(p) for p in raw]
        else:
            items = [self.repo / "results" / "games"]
        return collect_game_paths(items)

    def analyse(self) -> list[dict[str, Any]]:
        import arena

        db = None
        self.mark_start("analyse")
        outputs: list[dict[str, Any]] = []
        warned = False
        for path in self.game_inputs():
            loaded = load_input(path)
            if loaded is None:
                continue
            kind, data = loaded
            gid = str(data.get("game_id") or path.stem)
            if self.args.only and gid != self.args.only and path.stem != self.args.only:
                continue
            dest = self.tag_dir / f"{gid}.json"
            if kind == "capture" and recorded_human_side(data) is None:
                if not warned:
                    print(HUMAN_SIDE_WARN, flush=True)
                    warned = True
                self.human_side_assumed = True
            if dest.is_file() and not self.args.force and kind != "analysis":
                print(f"skip: {gid}", flush=True)
                try:
                    prev = json.loads(dest.read_text(encoding="utf-8"))
                    outputs.append(refresh_analysis(prev, ref=self.args.ref))
                except (OSError, json.JSONDecodeError):
                    pass
                continue
            if kind == "analysis":
                print(f"refresh {gid}", flush=True)
                result = refresh_analysis(data, ref=self.args.ref)
            else:
                if db is None:
                    db = arena.load_cards(str(self.cards))
                print(f"review {gid}", flush=True)
                result = analyse_game(
                    db,
                    data,
                    ref=self.args.ref,
                    side=self.args.side,
                    threads=self.threads,
                    names=self.names,
                )
            written = {k: v for k, v in result.items() if k not in ("mean_ms", "decision_ms")}
            dest.write_text(dumps(written), encoding="utf-8")
            outputs.append(result)
        self.mark_end("analyse")
        return outputs

    def write_review(self, games: list[dict[str, Any]]) -> None:
        self.mark_start("summary")
        clamped = 0
        times: list[float] = []
        for g in games:
            clamped += int(g.get("clamped_negative") or 0)
            times.extend(float(x) for x in (g.get("decision_ms") or []))
            if not g.get("decision_ms") and g.get("mean_ms"):
                n = len(g.get("decisions") or [])
                times.extend([float(g["mean_ms"])] * n)
        text = build_review_md(
            games,
            ref=self.args.ref,
            top=max(0, int(self.args.top)),
            names=self.names,
            name_source=self.name_source,
            clamped_total=clamped,
            mean_ms=mean(times),
        )
        (self.tag_dir / "REVIEW.md").write_text(text, encoding="utf-8")
        print(text, flush=True)
        wv = parse_wv(self.args.ref)
        if games:
            wv = parse_wv(str(games[0].get("ref") or self.args.ref))
        stats = review_stats(games, wv=wv, mean_ms=mean(times), clamped_total=clamped)
        self.run["human_side_assumed"] = bool(self.human_side_assumed)
        self.run["review"] = {
            "games": len(games),
            "clamped_negative": clamped,
            "mean_ms": mean(times),
            "ref": self.args.ref,
            "side": self.args.side,
            "engine": git_head(self.repo),
            "human_side_assumed": bool(self.human_side_assumed),
            "lethal_walks": stats["lethal_walks"],
            "analysed_end_turns": stats["analysed_end_turns"],
            "mean_optimism": stats["mean_optimism"],
            "noise_n": stats["noise_n"],
            "noise_mean": stats["noise_mean"],
            "noise_std": stats["noise_std"],
            "noise_max": stats["noise_max"],
        }
        self.save_run()
        self.mark_end("summary")

    def publish_ready(self) -> bool:
        local = self.tag_dir / "REVIEW.md"
        dest = self.publish_dir / self.args.tag / "REVIEW.md"
        if not dest.is_file() or not local.is_file():
            return False
        return dest.read_text(encoding="utf-8") == local.read_text(encoding="utf-8")

    def publish(self) -> None:
        if not self.args.publish:
            return
        if not self.args.force and self.publish_ready():
            print("skip: publish", flush=True)
            return
        self.mark_start("publish")
        log = self.tag_dir / "publish.txt"
        if log.is_file():
            log.unlink()
        def tee(argv: list[str], log_path: Path, append: bool = False) -> None:
            run_tee(argv, log_path, append=append)

        publish_tag(
            self.repo,
            self.publish_dir,
            self.args.publish_remote,
            self.args.publish_branch,
            self.tag_dir,
            self.args.tag,
            log,
            tee=tee,
        )
        review = self.tag_dir / "REVIEW.md"
        dest = self.publish_dir / self.args.tag
        if review.is_file():
            dest.mkdir(parents=True, exist_ok=True)
            target = dest / "REVIEW.md"
            if not target.is_file() or target.read_text(encoding="utf-8") != review.read_text(
                encoding="utf-8"
            ):
                import datetime as dt
                import subprocess

                shutil.copy2(review, target)
                tee(["git", "-C", str(self.publish_dir), "add", "-A"], log, True)
                status = subprocess.run(
                    ["git", "-C", str(self.publish_dir), "status", "--porcelain"],
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                    errors="replace",
                )
                if status.returncode == 0 and status.stdout.strip():
                    msg = (
                        f"{self.args.tag} results "
                        f"{dt.datetime.now(dt.timezone.utc).date().isoformat()}"
                    )
                    tee(git_commit_argv(self.publish_dir, "-m", msg), log, True)
                    tee(["git", "-C", str(self.publish_dir), "push"], log, True)
        self.mark_end("publish")

    def go(self) -> int:
        self.save_run()
        games = self.analyse()
        self.write_review(games)
        self.publish()
        return 0


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    return Runner(args).go()


if __name__ == "__main__":
    raise SystemExit(main())
