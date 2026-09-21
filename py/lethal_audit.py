#!/usr/bin/env python3
"""Replay recorded games through the exhaustive forced-lethal solver.

Two counts, both over decisions the solver actually decided — never over
all decisions, and never treating ``unknown`` as ``none``:

- ``missed_lethal`` — at a decision on the bot's own turn, ``forced_lethal``
  returns ``lethal`` and the recorded game did not end that turn with the
  bot winning.
- ``handed_lethal`` — at the bot's ``EndTurn``, ``forced_lethal`` from the
  opponent's resulting turn returns ``lethal``.

``unknown_rate`` is reported per count. A rising unknown rate silently
deflating ``missed_lethal`` is the failure mode this runner is built to
make visible.

RNG-dependent kills (a line that consumed ``state.rng``) are counted in
the headline and also broken out: they are real misses, but a weaker
indictment than a deterministic line.

Per-decision records are persisted (``--decisions``, default on) so a later
question is a re-read, not a re-run. Each record carries the snapshot-derived
board at the moment the solver was asked: both players' ``leader_defense``,
follower count, and board attack, plus the defense and board-attack deficits
(opponent minus auditee). The motivating result is the per-deck audit's
``handed_lethal`` vs win-rate correlation of r = −0.748 across the 16 meta
decks — the bottom decks hand over lethal 1.72× as often as the top ones.
That number is confounded by construction: handing over lethal is also what
losing looks like. Stratifying ``handed_lethal`` by the defense deficit at
EndTurn (``--by-deficit``) is the discriminating test. Equal rates at equal
deficit means the correlation was the confound (planning); a leftover gap
at equal deficit is tactics.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from collections import defaultdict
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from matchup import (  # noqa: E402
    DEFAULT_POOL,
    load_extra_deck_files,
    resolve_selected_decks,
)
from runlib import flag_given, repo_root  # noqa: E402


# Calibrated default — see the PR body. 50_000 nodes bought ~5% unknown
# on bot-turn decisions and ~17% on EndTurn (opponent-to-move) on a
# 4-game h0-fast forest/rune set. Raise to 200_000 to buy ~3% / ~9%;
# do not raise it just to make the rate look good.
DEFAULT_BUDGET = 50_000
DEFAULT_POLICY = "h0-fast"
ACTION_META_KEYS = frozenset({"value", "bot_value"})
MULLIGAN_KEY = "mulligan"
END_TURN_KEY = "end_turn"

# Defense-deficit buckets for ``--by-deficit``. Chosen to split the
# confound-relevant axis: 0 is even life; ±1..4 is a small swing (a few
# points of chip / one typical follower); ±5+ is a large life gap — the
# region where "behind on life" predicts handed_lethal regardless of
# tactics. The cuts partition every integer.
DEFICIT_BUCKETS = ("<= -5", "-4..-1", "0", "1..4", ">= 5")
DEFICIT_BUCKET_RULE = (
    "defense_deficit = opp.leader_defense - auditee.leader_defense at EndTurn. "
    "0 is even life; ±1..4 is a small swing; ±5+ is a large life gap "
    "(the confound region). Cuts taken from the brief's suggested partition "
    "because they split even / small / large without fitting the data."
)


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


def action_body(step: Any) -> Any:
    if not isinstance(step, dict):
        return step
    return {k: v for k, v in step.items() if k not in ACTION_META_KEYS}


def apply_step(game: Any, step: dict[str, Any]) -> None:
    if "reseed" in step:
        game.reseed(int(step["reseed"]))
        return
    game.apply(action_body(step))


def acting_of(game: Any) -> str:
    legal = game.legal()
    if legal:
        who = action_player(legal[0])
        if who:
            return who
    return str(game.active)


def is_mulligan(step: dict[str, Any]) -> bool:
    return MULLIGAN_KEY in step


def is_end_turn(step: dict[str, Any]) -> bool:
    return END_TURN_KEY in step


def field_cards(player: dict[str, Any]) -> list[dict[str, Any]]:
    field = player.get("field") or []
    return [c for c in field if isinstance(c, dict)]


def side_state(player: dict[str, Any]) -> dict[str, int]:
    """Snapshot-derived board for one seat.

    CanonicalState names leader life ``leader_defense`` — there is no player
    ``defense`` key. Field slots have no ``kind``; follower count is occupied
    slots with ``max_defense`` > 0, the snapshot-visible HP that amulets
    (Earth Sigil is 0/0) do not carry. Board attack sums ``attack`` on every
    occupied slot.
    """
    cards = field_cards(player)
    return {
        "leader_defense": int(player.get("leader_defense") or 0),
        "followers": sum(1 for c in cards if int(c.get("max_defense") or 0) > 0),
        "board_attack": sum(int(c.get("attack") or 0) for c in cards),
    }


def board_at_decision(snap: dict[str, Any], auditee: str) -> dict[str, Any]:
    opp = "b" if auditee == "a" else "a"
    players = snap.get("players") or {}
    mine = side_state(players.get(auditee) or {})
    theirs = side_state(players.get(opp) or {})
    return {
        "auditee": auditee,
        "auditee_state": mine,
        "opp_state": theirs,
        "defense_deficit": theirs["leader_defense"] - mine["leader_defense"],
        "board_attack_deficit": theirs["board_attack"] - mine["board_attack"],
    }


def deficit_bucket(defense_deficit: int) -> str:
    if defense_deficit <= -5:
        return "<= -5"
    if defense_deficit <= -1:
        return "-4..-1"
    if defense_deficit == 0:
        return "0"
    if defense_deficit <= 4:
        return "1..4"
    return ">= 5"


def deck_name_of(game: dict[str, Any], seat: str) -> str:
    key = "deck_a_name" if seat == "a" else "deck_b_name"
    name = game.get(key)
    if isinstance(name, str) and name:
        return name
    fallback = game.get("bot_deck")
    if isinstance(fallback, str) and fallback and "/" not in fallback:
        return fallback
    return "unknown"


def buckets_from_decisions(
    decisions: list[dict[str, Any]],
    seat: str | None = None,
) -> dict[str, dict[str, int]]:
    missed = empty_bucket()
    handed = empty_bucket()
    for rec in decisions:
        if seat is not None and rec.get("auditee") != seat:
            continue
        verdict = {
            "verdict": rec.get("verdict"),
            "nodes": rec.get("nodes"),
            "rng_dependent": rec.get("rng_dependent"),
        }
        hit = bool(rec.get("hit"))
        if rec.get("kind") == "missed_lethal":
            note_verdict(missed, verdict, hit=hit)
        elif rec.get("kind") == "handed_lethal":
            note_verdict(handed, verdict, hit=hit)
    return {"missed": missed, "handed": handed}


def by_deficit_report(games: list[dict[str, Any]]) -> dict[str, Any]:
    """``handed_lethal`` per deck, bucketed by defense deficit at EndTurn."""
    per_deck: dict[str, dict[str, dict[str, int]]] = defaultdict(
        lambda: {label: empty_bucket() for label in DEFICIT_BUCKETS}
    )
    shares_n: dict[str, dict[str, int]] = defaultdict(
        lambda: {label: 0 for label in DEFICIT_BUCKETS}
    )
    for g in games:
        for rec in g.get("decisions") or []:
            if rec.get("kind") != "handed_lethal":
                continue
            seat = str(rec.get("auditee") or "")
            deck = deck_name_of(g, seat) if seat in ("a", "b") else "unknown"
            label = deficit_bucket(int(rec.get("defense_deficit") or 0))
            verdict = {
                "verdict": rec.get("verdict"),
                "nodes": rec.get("nodes"),
                "rng_dependent": rec.get("rng_dependent"),
            }
            note_verdict(per_deck[deck][label], verdict, hit=bool(rec.get("hit")))
            shares_n[deck][label] += 1
    out: dict[str, Any] = {}
    for name in sorted(set(per_deck) | set(shares_n)):
        total = sum(shares_n[name].values())
        out[name] = {
            "handed_lethal": {
                label: rate_block(per_deck[name][label]) for label in DEFICIT_BUCKETS
            },
            "bucket_share": {
                label: (shares_n[name][label] / total) if total else None
                for label in DEFICIT_BUCKETS
            },
            "end_turn_decisions": total,
        }
    return {
        "kind": "handed_lethal",
        "on": "defense_deficit",
        "buckets": list(DEFICIT_BUCKETS),
        "bucket_rule": DEFICIT_BUCKET_RULE,
        "per_deck": out,
    }


def format_by_deficit(report: dict[str, Any]) -> str:
    lines = [
        "handed_lethal by defense deficit at EndTurn",
        report["bucket_rule"],
        "",
    ]
    header = f"{'deck':<22}" + "".join(f"{b:>16}" for b in DEFICIT_BUCKETS)
    lines.append(header)
    for name, row in report["per_deck"].items():
        cells = []
        for label in DEFICIT_BUCKETS:
            block = row["handed_lethal"][label]
            rate = block["rate"]
            rate_s = "—" if rate is None else f"{rate:.3f}"
            cells.append(f"{block['n']}/{block['decisions_with_a_verdict']} {rate_s}")
        lines.append(f"{name:<22}" + "".join(f"{c:>16}" for c in cells))
        share_cells = []
        for label in DEFICIT_BUCKETS:
            share = row["bucket_share"][label]
            share_s = "—" if share is None else f"{share:.0%}"
            share_cells.append(f"share {share_s}")
        lines.append(f"{'':<22}" + "".join(f"{c:>16}" for c in share_cells))
    return "\n".join(lines)


def bot_converts_this_turn(
    actions: list[dict[str, Any]],
    ply: int,
    bot: str,
    winner: str | None,
) -> bool:
    """True if the recorded game ends the current bot turn with ``bot`` winning.

    Walks forward from ``ply`` (the decision about to be played). A convert
    is a later recorded winner of ``bot`` before the opponent begins their
    next main-turn action. Used so a miss is "had a kill and did not
    convert", not "had a kill at ply N and also at ply N+1".
    """
    if winner != bot:
        # Fast path: the game as a whole did not end with the bot winning,
        # so this turn cannot have been a convert. (A bot can convert and
        # still lose only if the winner field is wrong; we trust the log.)
        # Still scan — a mid-game concede isn't a thing, but a win on this
        # turn would have set winner.
        pass
    for step in actions[ply:]:
        if not isinstance(step, dict):
            continue
        if is_end_turn(step) and action_player(step) == bot:
            return False
        # A recorded terminal step is not a dedicated event; the winner is
        # on the log. If the bot plays a killing blow, subsequent actions
        # stop. Treat "no further opponent action after this ply, and
        # winner == bot" as a convert at the caller.
    # If the bot never ended the turn after this ply and they won the
    # game, they converted on this turn.
    later_opp = False
    for step in actions[ply:]:
        if not isinstance(step, dict) or "reseed" in step:
            continue
        who = action_player(step)
        if who == bot and is_end_turn(step):
            return False
        if who is not None and who != bot and not is_mulligan(step):
            later_opp = True
            break
    return (not later_opp) and winner == bot


def empty_bucket() -> dict[str, int]:
    return {
        "lethal": 0,
        "none": 0,
        "unknown": 0,
        "hits": 0,
        "rng_dependent_hits": 0,
        "nodes": 0,
        "ms": 0,
    }


def note_verdict(bucket: dict[str, int], verdict: dict[str, Any], *, hit: bool) -> None:
    kind = str(verdict.get("verdict") or "")
    if kind == "lethal":
        bucket["lethal"] += 1
        if hit:
            bucket["hits"] += 1
            if verdict.get("rng_dependent"):
                bucket["rng_dependent_hits"] += 1
    elif kind == "none":
        bucket["none"] += 1
    elif kind == "unknown":
        bucket["unknown"] += 1
    bucket["nodes"] += int(verdict.get("nodes") or 0)


def rate_block(bucket: dict[str, int]) -> dict[str, Any]:
    decided = bucket["lethal"] + bucket["none"]
    asked = decided + bucket["unknown"]
    return {
        "n": bucket["hits"],
        "decisions_with_a_verdict": decided,
        "rate": (bucket["hits"] / decided) if decided else None,
        "unknown": bucket["unknown"],
        "unknown_rate": (bucket["unknown"] / asked) if asked else None,
        "asked": asked,
        "lethal": bucket["lethal"],
        "none": bucket["none"],
        "rng_dependent": bucket["rng_dependent_hits"],
        "nodes": bucket["nodes"],
    }


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    raw = list(sys.argv[1:] if argv is None else argv)
    p = argparse.ArgumentParser(
        prog="py/lethal_audit.py",
        description=(
            "Replay recorded (or freshly played) games through the exhaustive "
            "forced-lethal solver and emit missed_lethal / handed_lethal JSON."
        ),
    )
    p.add_argument("--games", type=int, default=2, help="games to play when not replaying files")
    p.add_argument(
        "--records",
        nargs="*",
        default=None,
        metavar="PATH",
        help="recorded game JSON files or directories (review.py shape)",
    )
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--decks", nargs="*", default=None, help="restrict to these deck stems")
    p.add_argument(
        "--pool",
        default=None,
        help=f"named pool from oracle/decks/POOLS.json (default: {DEFAULT_POOL})",
    )
    p.add_argument(
        "--deck-file",
        nargs="*",
        default=[],
        metavar="PATH",
        help="extra decks; name = file stem",
    )
    p.add_argument("--policy", default=DEFAULT_POLICY, help="spec for both seats unless overridden")
    p.add_argument("--policy-a", default=None)
    p.add_argument("--policy-b", default=None)
    p.add_argument(
        "--bot-seat",
        choices=("a", "b", "both"),
        default="b",
        help="which seat is the bot under audit (default: b)",
    )
    p.add_argument(
        "--budget",
        type=int,
        default=DEFAULT_BUDGET,
        help=f"solver node budget (default: {DEFAULT_BUDGET})",
    )
    p.add_argument(
        "--first",
        default="alternate",
        choices=("alternate", "coin", "a", "b"),
    )
    p.add_argument("--cards", default=None)
    p.add_argument("--out", default=None, help="write JSON here (also printed)")
    p.add_argument(
        "--decisions",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="include per-decision records in each game (default: on)",
    )
    p.add_argument(
        "--by-deficit",
        action="store_true",
        default=False,
        help=(
            "add handed_lethal stratified by defense deficit at EndTurn "
            "(also printed to stderr)"
        ),
    )
    args = p.parse_args(raw)
    args._argv = raw
    if flag_given(raw, "--pool") and flag_given(raw, "--decks"):
        raise SystemExit("--pool cannot be combined with --decks")
    if not flag_given(raw, "--pool") and not flag_given(raw, "--decks"):
        args.pool = DEFAULT_POOL
    return args


def load_record_files(paths: list[str]) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for raw in paths:
        path = Path(raw)
        files: list[Path]
        if path.is_dir():
            files = sorted(path.glob("*.json")) + sorted(path.glob("*.jsonl"))
        else:
            files = [path]
        for f in files:
            text = f.read_text(encoding="utf-8")
            if f.suffix == ".jsonl":
                for line in text.splitlines():
                    if line.strip():
                        rec = json.loads(line)
                        if isinstance(rec, dict) and rec.get("actions"):
                            rec.setdefault("game_id", f"{f.stem}:{len(out)}")
                            out.append(rec)
            else:
                rec = json.loads(text)
                if isinstance(rec, dict) and rec.get("actions"):
                    rec.setdefault("game_id", f.stem)
                    out.append(rec)
    if not out:
        raise SystemExit("no recorded games with an actions array")
    return out


def play_one(
    db: Any,
    seed: int,
    deck_a: dict[str, int],
    deck_b: dict[str, int],
    first: str,
    policy_a: str,
    policy_b: str,
) -> dict[str, Any]:
    import arena

    game = arena.Game(db, seed, deck_a, deck_b, first)
    actions: list[dict[str, Any]] = []
    n = 0
    while not game.terminal:
        acting = acting_of(game)
        spec = policy_a if acting == "a" else policy_b
        act = game.bot_action(spec, seed + n)
        if not isinstance(act, dict):
            raise RuntimeError(f"bot_action returned {act!r}")
        try:
            game.apply(act)
        except Exception:
            # A policy can walk into a resolution-ceiling or illegal; keep
            # the prefix so the audit still has a recorded game.
            break
        actions.append(act)
        n += 1
        if n > 400:
            break
    return {
        "v": 1,
        "game_id": f"play-{seed}",
        "seed": seed,
        "deckA": deck_a,
        "deckB": deck_b,
        "first": first,
        "winner": game.winner,
        "actions": actions,
        "policy_a": policy_a,
        "policy_b": policy_b,
        "turns": game.turn,
    }


def seats_for(bot_seat: str) -> tuple[str, ...]:
    if bot_seat == "both":
        return ("a", "b")
    return (bot_seat,)


def first_for(mode: str, game_index: int) -> str:
    if mode == "alternate":
        return "a" if game_index % 2 == 0 else "b"
    if mode == "coin":
        return "coin"
    return mode


def deck_label(log: dict[str, Any], seat: str, fallback: str) -> str:
    name = log.get("deck_a_name") if seat == "a" else log.get("deck_b_name")
    if isinstance(name, str) and name:
        return name
    return fallback


def audit_game(
    db: Any,
    log: dict[str, Any],
    *,
    budget: int,
    seats: tuple[str, ...],
) -> dict[str, Any]:
    import arena

    game = arena.Game(
        db,
        int(log["seed"]),
        log["deckA"],
        log["deckB"],
        str(log.get("first") or "coin"),
    )
    actions = [s for s in (log.get("actions") or []) if isinstance(s, dict)]
    winner = log.get("winner")
    if winner not in ("a", "b"):
        winner = game.winner
    missed = empty_bucket()
    handed = empty_bucket()
    decisions: list[dict[str, Any]] = []
    t_decided = 0.0
    n_timed = 0

    for ply, step in enumerate(actions):
        if game.terminal:
            break
        if "reseed" in step:
            apply_step(game, step)
            continue
        if is_mulligan(step):
            apply_step(game, step)
            continue
        acting = acting_of(game)
        if acting in seats:
            snap = game.snapshot()
            t0 = time.perf_counter()
            verdict = arena.forced_lethal(game, budget)
            dt = (time.perf_counter() - t0) * 1000.0
            t_decided += dt
            n_timed += 1
            missed["ms"] += int(dt)
            converted = bot_converts_this_turn(actions, ply, acting, winner)
            hit = str(verdict.get("verdict")) == "lethal" and not converted
            note_verdict(missed, verdict, hit=hit)
            rec = {
                "ply": ply,
                "turn": int(game.turn),
                "acting": acting,
                "kind": "missed_lethal",
                "verdict": verdict.get("verdict"),
                "nodes": verdict.get("nodes"),
                "rng_dependent": verdict.get("rng_dependent"),
                "converted": converted,
                "hit": hit,
                "ms": dt,
            }
            rec.update(board_at_decision(snap, acting))
            decisions.append(rec)
        apply_step(game, step)
        if is_end_turn(step) and acting in seats and not game.terminal:
            snap = game.snapshot()
            t0 = time.perf_counter()
            verdict = arena.forced_lethal(game, budget)
            dt = (time.perf_counter() - t0) * 1000.0
            t_decided += dt
            n_timed += 1
            handed["ms"] += int(dt)
            hit = str(verdict.get("verdict")) == "lethal"
            note_verdict(handed, verdict, hit=hit)
            rec = {
                "ply": ply,
                "turn": int(game.turn),
                "acting": acting_of(game),
                "kind": "handed_lethal",
                "verdict": verdict.get("verdict"),
                "nodes": verdict.get("nodes"),
                "rng_dependent": verdict.get("rng_dependent"),
                "hit": hit,
                "ms": dt,
            }
            rec.update(board_at_decision(snap, acting))
            decisions.append(rec)

    return {
        "game_id": log.get("game_id"),
        "winner": winner,
        "missed": missed,
        "handed": handed,
        "decisions": decisions,
        "ms_per_decision": (t_decided / n_timed) if n_timed else 0.0,
        "timed": n_timed,
    }


def merge_bucket(dst: dict[str, int], src: dict[str, int]) -> None:
    for k, v in src.items():
        dst[k] += v


def summarize(
    games: list[dict[str, Any]],
    *,
    budget: int,
    pool: str,
    policy_a: str,
    policy_b: str,
    bot_seat: str,
) -> dict[str, Any]:
    missed = empty_bucket()
    handed = empty_bucket()
    per_deck: dict[str, dict[str, dict[str, int]]] = defaultdict(
        lambda: {"missed": empty_bucket(), "handed": empty_bucket()}
    )
    ms = 0.0
    timed = 0
    for g in games:
        merge_bucket(missed, g["missed"])
        merge_bucket(handed, g["handed"])
        ms += float(g.get("ms_per_decision") or 0.0) * int(g.get("timed") or 0)
        timed += int(g.get("timed") or 0)
        if bot_seat == "both":
            for seat in ("a", "b"):
                name = deck_name_of(g, seat)
                split = buckets_from_decisions(g.get("decisions") or [], seat)
                merge_bucket(per_deck[name]["missed"], split["missed"])
                merge_bucket(per_deck[name]["handed"], split["handed"])
        else:
            deck = str(g.get("bot_deck") or "unknown")
            merge_bucket(per_deck[deck]["missed"], g["missed"])
            merge_bucket(per_deck[deck]["handed"], g["handed"])
    return {
        "budget": budget,
        "pool": pool,
        "policy_a": policy_a,
        "policy_b": policy_b,
        "bot_seat": bot_seat,
        "games": len(games),
        "missed_lethal": rate_block(missed),
        "handed_lethal": rate_block(handed),
        "ms_per_decision": (ms / timed) if timed else 0.0,
        "per_deck": {
            name: {
                "missed_lethal": rate_block(rows["missed"]),
                "handed_lethal": rate_block(rows["handed"]),
            }
            for name, rows in sorted(per_deck.items())
        },
    }


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    import arena

    root = repo_root()
    cards = args.cards or str(root / "cards")
    db = arena.load_cards(cards)
    policy_a = args.policy_a if args.policy_a is not None else args.policy
    policy_b = args.policy_b if args.policy_b is not None else args.policy
    seats = seats_for(args.bot_seat)

    logs: list[dict[str, Any]]
    pool_name = args.pool or "--decks"
    if args.records:
        logs = load_record_files(args.records)
        for rec in logs:
            rec.setdefault("deck_a_name", deck_label(rec, "a", "records"))
            rec.setdefault("deck_b_name", deck_label(rec, "b", "records"))
            if args.bot_seat == "a":
                rec.setdefault("bot_deck", rec["deck_a_name"])
            elif args.bot_seat == "b":
                rec.setdefault("bot_deck", rec["deck_b_name"])
            else:
                rec.setdefault(
                    "bot_deck", f"{rec['deck_a_name']}/{rec['deck_b_name']}"
                )
    else:
        _, decks = resolve_selected_decks(root / "oracle" / "decks", args.pool, args.decks)
        decks.update(load_extra_deck_files(args.deck_file))
        names = list(decks.keys())
        if not names:
            raise SystemExit("no decks selected")
        logs = []
        pairs = [(da, dbk) for da in names for dbk in names]
        for n in range(max(1, args.games)):
            da, dbk = pairs[n % len(pairs)]
            first = first_for(args.first, n)
            rec = play_one(
                db,
                args.seed + n,
                decks[da],
                decks[dbk],
                first,
                policy_a,
                policy_b,
            )
            rec["deck_a_name"] = da
            rec["deck_b_name"] = dbk
            if args.bot_seat == "a":
                rec["bot_deck"] = da
            elif args.bot_seat == "b":
                rec["bot_deck"] = dbk
            else:
                rec["bot_deck"] = f"{da}/{dbk}"
            logs.append(rec)

    audited = []
    for rec in logs:
        row = audit_game(db, rec, budget=args.budget, seats=seats)
        row["bot_deck"] = rec.get("bot_deck") or "unknown"
        row["deck_a_name"] = rec.get("deck_a_name")
        row["deck_b_name"] = rec.get("deck_b_name")
        audited.append(row)

    summary = summarize(
        audited,
        budget=args.budget,
        pool=pool_name,
        policy_a=policy_a,
        policy_b=policy_b,
        bot_seat=args.bot_seat,
    )
    if args.by_deficit:
        summary["by_deficit"] = by_deficit_report(audited)
    records = []
    for g in audited:
        rec = {
            "game_id": g.get("game_id"),
            "bot_deck": g.get("bot_deck"),
            "winner": g.get("winner"),
            "missed_lethal": rate_block(g["missed"]),
            "handed_lethal": rate_block(g["handed"]),
        }
        if args.decisions:
            rec["decisions"] = g.get("decisions") or []
        records.append(rec)
    summary["records"] = records
    text = json.dumps(summary, indent=2, sort_keys=True)
    print(text)
    if args.by_deficit:
        print(format_by_deficit(summary["by_deficit"]), file=sys.stderr)
    if args.out:
        Path(args.out).write_text(text + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
