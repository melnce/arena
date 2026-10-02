#!/usr/bin/env python3
"""holdback1 — does the bot hold back trades whose attacker dies anyway?

A "held-back moment" (HB) is a bot decision where the bot chose End Turn while it still had a **kill attack**: a legal
attack on an enemy follower after which (clone + apply) that follower is gone. For each HB moment the scan records,
from the recorded game:

- the primary trade X -> Y: among the kill attacks, prefer one whose attacker survives, then the target with the most
  attack;
- what the opponent's next turn did: did X die, was Y still alive at its end, how much leader damage Y dealt;
- whether the opponent could have killed X anyway: an exhaustive iterative-deepening search over the opponent's
  actions from the real start of its turn (its real hand and draw), up to --depth actions and --budget applies, goal
  "X is gone". A found line is replayed under 4 reseeds of the turn's randomness (`sure` = kills in all 4).

Subcommands:
  play   --out DIR [--games 256] [--seed 1] [--spec SPEC] [--pool meta] [--workers N]
         bot-vs-bot games, both seats SPEC, every ordered pair of the pool, first player alternating;
         one JSON per game in DIR/games (resumable: existing files are skipped).
  scan   --out FILE.jsonl GAME_JSON_OR_DIR ... [--budget 20000] [--depth 6] [--workers N]
         one line per HB moment plus one `{"game": ...}` line per game. Seats: both for self-play logs, the bot's
         seat for serve.py captures (`humanSide`).
  report FILE.jsonl ...     tables and the pre-registered verdict (claude/runbook-holdback1-local.md).

Run from the repo root (cards in ./cards, decks in ./oracle/decks, py/ importable), with the arena bindings of the
engine the games were played on.
"""
from __future__ import annotations

import argparse
import json
import os
import random
import statistics
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

SERVED = "h0:nodes=32000,horizon=3,k=8"
RESEEDS = (11, 22, 33, 44)
META = frozenset({"value", "bot_value"})

_DB = None


def _init(cards: str) -> None:
    global _DB
    import arena
    _DB = arena.load_cards(cards)


def strip(a: dict) -> dict:
    return {k: v for k, v in a.items() if k not in META}


def key(a: dict) -> str:
    return next(iter(strip(a)))


def player_of(a: dict):
    b = strip(a)[key(a)]
    return b.get("player") if isinstance(b, dict) else None


def acting(g) -> str | None:
    legal = g.legal()
    return player_of(legal[0]) if legal else None


def other(s: str) -> str:
    return "b" if s == "a" else "a"


# ---------------------------------------------------------------- play

def _play_one(job: dict) -> dict:
    import arena
    n, seed, da, dbn, first, deck_a, deck_b, spec, out = (job[k] for k in
        ("n", "seed", "da", "db", "first", "deck_a", "deck_b", "spec", "out"))
    path = Path(out) / f"play-{seed}.json"
    if path.exists():
        return {"n": n, "skipped": True}
    t0 = time.perf_counter()
    g = arena.Game(_DB, seed, deck_a, deck_b, first)
    actions = []
    i = 0
    while not g.terminal and i <= 400:
        act = g.bot_action(spec, seed + i)
        try:
            g.apply(act)
        except Exception:
            break
        actions.append(act)
        i += 1
    rec = {"v": 1, "game_id": f"play-{seed}", "seed": seed, "deckA": deck_a, "deckB": deck_b, "first": first,
           "deck_a_name": da, "deck_b_name": dbn, "winner": g.winner, "actions": actions,
           "policy_a": spec, "policy_b": spec, "turns": g.turn, "secs": round(time.perf_counter() - t0, 1)}
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(rec), encoding="utf-8")
    os.replace(tmp, path)
    return {"n": n, "skipped": False, "secs": rec["secs"], "actions": len(actions)}


def cmd_play(args) -> None:
    sys.path.insert(0, str(Path("py").resolve()))
    from matchup import resolve_selected_decks
    _, decks = resolve_selected_decks(Path("oracle") / "decks", args.pool, None)
    names = list(decks)
    pairs = [(a, b) for a in names for b in names]
    out = Path(args.out) / "games"
    out.mkdir(parents=True, exist_ok=True)
    jobs = []
    for n in range(args.games):
        da, dbn = pairs[n % len(pairs)]
        first = "a" if (n + n // len(pairs)) % 2 == 0 else "b"
        jobs.append({"n": n, "seed": args.seed * 100000 + n, "da": da, "db": dbn, "first": first,
                     "deck_a": decks[da], "deck_b": decks[dbn], "spec": args.spec, "out": str(out)})
    print(f"holdback play: {len(names)} decks, {len(pairs)} ordered pairs, {args.games} games, spec {args.spec}, "
          f"workers {args.workers}, out {out}", flush=True)
    t0 = time.perf_counter(); done = 0
    with ProcessPoolExecutor(args.workers, initializer=_init, initargs=(args.cards,)) as ex:
        for f in as_completed([ex.submit(_play_one, j) for j in jobs]):
            r = f.result(); done += 1
            if done % 16 == 0 or done == len(jobs):
                print(f"  {done}/{len(jobs)} games  {time.perf_counter() - t0:.0f}s", flush=True)


# ---------------------------------------------------------------- scan helpers
#
# Followers carry no instance id in the snapshot, and the field compacts when one leaves. Order is preserved and new
# followers enter at the end, so an instance is followed across an action by an order-preserving alignment of card
# ids (`follow`). Known blind spot: an instance that leaves while a copy of the same card enters right where it was
# matched counts as kept (rare).

def ids(g, seat: str) -> list:
    """The occupied field slots of a seat, in order (card dicts from the snapshot)."""
    return [c for c in g.snapshot()["players"][seat]["field"] if c]


_ATTRS = ("evolved", "super", "attack", "defense", "max_defense", "attacks_left", "can_attack")


def _cost(o: dict, n: dict) -> int:
    return sum(1 for k in _ATTRS if o.get(k) != n.get(k))


def follow(old: list, new: list, i):
    """Position of old[i] in `new`, or None if it left the field. Survivors keep their order at the front of `new`
    and newcomers are appended, so a matching is the set of survivors whose card ids equal new's prefix; among
    several (duplicates), the one whose attributes changed least wins."""
    if i is None:
        return None
    from itertools import combinations
    best = None
    for keep in range(min(len(old), len(new)), -1, -1):
        for S in combinations(range(len(old)), keep):
            if all(old[k]["card"] == new[t]["card"] for t, k in enumerate(S)):
                c = sum(_cost(old[k], new[t]) for t, k in enumerate(S))
                if best is None or c < best[0]:
                    best = (c, S)
        if best is not None:
            break
    if best is None:
        return None
    S = best[1]
    return S.index(i) if i in S else None


def card_at(g, seat: str, i) -> dict | None:
    if i is None:
        return None
    occ = [c for c in g.snapshot()["players"][seat]["field"] if c]
    return occ[i] if 0 <= i < len(occ) else None


def kill_attacks(g, me: str) -> list:
    """Legal attacks on enemy followers after which (clone + apply) the target has left the field."""
    opp = other(me)
    out = []
    mine0, theirs0 = ids(g, me), ids(g, opp)
    for a in g.legal():
        if key(a) != "attack":
            continue
        b = a["attack"]; t = b["target"]
        if t == "leader" or not isinstance(t, dict):
            continue
        xi, yi = b["attacker_slot"], t["slot"]
        x, y = card_at(g, me, xi), card_at(g, opp, yi)
        if not x or not y:
            continue
        h = g.clone()
        try:
            h.apply(a)
        except Exception:
            continue
        if follow(theirs0, ids(h, opp), yi) is None:
            out.append({"action": a, "xi": xi, "yi": yi, "x": x["card"], "y": y["card"],
                        "x_evolved": bool(x.get("evolved")), "x_attack": x["attack"], "x_defense": x["defense"],
                        "y_attack": y["attack"], "y_defense": y["defense"],
                        "x_survives": follow(mine0, ids(h, me), xi) is not None})
    return out


def can_kill(g0, bot: str, xi: int, budget: int, depth_max: int) -> dict:
    """Iterative deepening over the opponent's actions (End Turn excluded) for a line after which the bot's follower
    at position xi has left the field. Applies are counted across all depths. `none` = the whole turn tree was
    searched (nothing cut by the depth limit); `none_within_depth` = no line within depth_max actions."""
    opp = other(bot)
    nodes = 0
    for depth in range(1, depth_max + 1):
        seen = set()
        stack = [(g0, [], xi)]
        cut = False
        while stack:
            h, line, pos = stack.pop()
            before = ids(h, bot)
            for a in h.legal():
                if key(a) == "end_turn" or player_of(a) != opp:
                    continue
                if nodes >= budget:
                    return {"verdict": "unknown", "nodes": nodes, "depth": depth}
                h2 = h.clone()
                try:
                    h2.apply(a)
                except Exception:
                    continue
                nodes += 1
                if h2.terminal:
                    if h2.winner == opp:
                        return {"verdict": "lethal", "nodes": nodes, "line": line + [a], "depth": depth}
                    continue
                pos2 = follow(before, ids(h2, bot), pos)
                if pos2 is None:
                    return {"verdict": "kill", "nodes": nodes, "line": line + [a], "depth": depth}
                if acting(h2) != opp:
                    continue
                if len(line) + 1 >= depth:
                    cut = True
                    continue
                hk = h2.hash()
                if hk in seen:
                    continue
                seen.add(hk)
                stack.append((h2, line + [a], pos2))
        if not cut:
            return {"verdict": "none", "nodes": nodes, "depth": depth}
    return {"verdict": "none_within_depth", "nodes": nodes, "depth": depth_max}


def sure_count(g0, line: list, bot: str, xi: int) -> int:
    """Rerolls of the turn's randomness (RESEEDS) under which the line still applies and removes the follower."""
    ok = 0
    for r in RESEEDS:
        h = g0.clone(); h.reseed(r)
        pos = xi
        try:
            for a in line:
                before = ids(h, bot)
                h.apply(a)
                if h.terminal:
                    break
                pos = follow(before, ids(h, bot), pos)
        except Exception:
            continue
        if (h.terminal and h.winner == other(bot)) or pos is None:
            ok += 1
    return ok


def _scan_game(job: dict) -> list:
    import arena
    d, budget, depth = job["log"], job["budget"], job["depth"]
    g = arena.Game(_DB, int(d["seed"]), d["deckA"], d["deckB"], str(d.get("first") or "coin"))
    human = d.get("humanSide")
    seats = {other(human)} if human in ("a", "b") else {"a", "b"}
    acts = [a for a in d["actions"] if isinstance(a, dict)]
    rows = []
    n_dec = {"a": 0, "b": 0}; n_end = {"a": 0, "b": 0}
    for i, a in enumerate(acts):
        if g.terminal:
            break
        if "reseed" in a:
            g.reseed(int(a["reseed"])); continue
        who = player_of(a)
        if who in seats and key(a) != "mulligan":
            n_dec[who] += 1
            if key(a) == "end_turn":
                n_end[who] += 1
                kas = kill_attacks(g, who)
                if kas:
                    rows.append(_moment(g, d, acts, i, who, kas, budget, depth))
        g.apply(strip(a))
    rows.append({"game": d.get("game_id"), "seats": sorted(seats), "decisions": n_dec, "end_turns": n_end,
                 "winner": d.get("winner") or g.winner, "deck_a": d.get("deck_a_name"), "deck_b": d.get("deck_b_name"),
                 "policy": d.get("policy") or d.get("policy_a"), "human": human})
    return rows


def _moment(g, d, acts, i, bot, kas, budget, depth) -> dict:
    """One held-back moment: the primary trade, the recorded next turn, and the kill search."""
    opp = other(bot)
    ka = sorted(kas, key=lambda k: (not k["x_survives"], -k["y_attack"]))[0]
    s0 = g.snapshot()
    rec = {"game": d.get("game_id"), "ply": i, "bot": bot, "turn": g.turn,
           "def_bot": s0["players"][bot]["leader_defense"], "def_opp": s0["players"][opp]["leader_defense"],
           "bot_value": acts[i].get("bot_value"), "n_kill_attacks": len(kas),
           "primary": {k: ka[k] for k in ("x", "y", "x_evolved", "x_attack", "x_defense", "y_attack", "y_defense",
                                           "x_survives")},
           "free_kill_available": any(k["x_survives"] for k in kas)}
    # the recorded End Turn, then the opponent's recorded turn, following X and Y action by action
    h = g.clone()
    xi, yi = ka["xi"], ka["yi"]
    bb, ob = ids(h, bot), ids(h, opp)
    h.apply(strip(acts[i]))
    xi, yi = follow(bb, ids(h, bot), xi), follow(ob, ids(h, opp), yi)
    start, x_start = h.clone(), xi
    others = [k for k in range(len(ids(h, bot))) if k != xi]
    def0 = h.snapshot()["players"][bot]["leader_defense"]
    y_face = 0
    j = i + 1
    while j < len(acts) and not h.terminal:
        a = acts[j]
        if "reseed" in a:
            h.reseed(int(a["reseed"])); j += 1; continue
        if player_of(a) == bot and key(a) not in ("choose", "confirm"):
            break
        bb, ob = ids(h, bot), ids(h, opp)
        hit_leader = key(a) == "attack" and a["attack"].get("target") == "leader"
        y_attacks = hit_leader and yi is not None and a["attack"]["attacker_slot"] == yi
        before = h.snapshot()["players"][bot]["leader_defense"]
        h.apply(strip(a))
        if y_attacks:
            y_face += before - h.snapshot()["players"][bot]["leader_defense"]
        if not h.terminal:
            nb = ids(h, bot)
            xi, yi = follow(bb, nb, xi), follow(ob, ids(h, opp), yi)
            others = [follow(bb, nb, k) for k in others]
        j += 1
    over = bool(h.terminal)
    rec.update({
        "next_turn_actions": j - i - 1,
        "game_over_next": over,
        "x_died_next": None if over else (xi is None),
        "y_alive_end": None if over else (yi is not None),
        "y_face": y_face,
        "others_n": len(others), "others_died": None if over else sum(1 for k in others if k is None),
        "bot_took": def0 - h.snapshot()["players"][bot]["leader_defense"],
        "bot_won_game": (d.get("winner") or None) == bot,
    })
    # could the opponent have removed X anyway (real hand, real draw)?
    t0 = time.perf_counter()
    ck = can_kill(start, bot, x_start, budget, depth) if x_start is not None else {"verdict": "gone_at_end_turn"}
    ck["ms"] = int((time.perf_counter() - t0) * 1000)
    if ck.get("line"):
        ck["sure"] = sure_count(start, ck["line"], bot, x_start)
        ck["line_len"] = len(ck["line"])
        ck["line_kinds"] = [key(a) for a in ck["line"]]
        ck.pop("line")
    rec["can_kill_x"] = ck
    return rec


def iter_logs(paths):
    for p in paths:
        p = Path(p)
        files = sorted(p.glob("*.json")) if p.is_dir() else [p]
        for f in files:
            d = json.loads(f.read_text(encoding="utf-8"))
            if isinstance(d, dict) and d.get("actions"):
                d.setdefault("game_id", f.stem)
                yield d


def cmd_scan(args) -> None:
    logs = list(iter_logs(args.inputs))
    print(f"holdback scan: {len(logs)} games, budget {args.budget}, depth {args.depth}, workers {args.workers}",
          flush=True)
    t0 = time.perf_counter(); done = 0
    with open(args.out, "w", encoding="utf-8") as out, \
            ProcessPoolExecutor(args.workers, initializer=_init, initargs=(args.cards,)) as ex:
        futs = [ex.submit(_scan_game, {"log": d, "budget": args.budget, "depth": args.depth}) for d in logs]
        for f in as_completed(futs):
            for row in f.result():
                out.write(json.dumps(row) + "\n")
            done += 1
            if done % 16 == 0 or done == len(futs):
                print(f"  {done}/{len(futs)} games  {time.perf_counter() - t0:.0f}s", flush=True)


# ---------------------------------------------------------------- report

def wilson(k: int, n: int) -> tuple:
    if n == 0:
        return (None, None)
    z = 1.96; p = k / n
    den = 1 + z * z / n; c = p + z * z / (2 * n); r = z * ((p * (1 - p) / n + z * z / (4 * n * n)) ** 0.5)
    return ((c - r) / den, (c + r) / den)


def cluster_boot(rows, fn, B=2000, seed=7):
    """95 % bootstrap over games of a share fn(rows)."""
    by = {}
    for r in rows:
        by.setdefault(r["game"], []).append(r)
    games = list(by)
    rng = random.Random(seed)
    vals = []
    for _ in range(B):
        sample = [r for gid in (rng.choice(games) for _ in games) for r in by[gid]]
        v = fn(sample)
        if v is not None:
            vals.append(v)
    vals.sort()
    if not vals:
        return (None, None)
    return (vals[int(0.025 * len(vals))], vals[int(0.975 * len(vals)) - 1])


def share(rows, pred):
    rows = [r for r in rows if pred(r) is not None]
    return (sum(1 for r in rows if pred(r)) / len(rows)) if rows else None


def cmd_report(args) -> None:
    rows = []
    for f in args.inputs:
        rows += [json.loads(l) for l in open(f, encoding="utf-8") if l.strip()]
    games = [r for r in rows if "game" in r and "decisions" in r]
    hb = [r for r in rows if "primary" in r]
    n_games = len(games)
    seats = sum(len(g["seats"]) for g in games)
    ends = sum(sum(g["end_turns"].values()) for g in games)
    live = [r for r in hb if not r["game_over_next"]]
    wasted = lambda r: (r["x_died_next"] and r["y_alive_end"]) if not r["game_over_next"] else None
    died = lambda r: r["x_died_next"] if not r["game_over_next"] else None
    killable = lambda r: (r["can_kill_x"]["verdict"] == "kill" and r["can_kill_x"].get("sure", 0) == 4) \
        if r["can_kill_x"]["verdict"] not in ("unknown", "lethal") else None
    F = len(hb) / seats if seats else 0.0
    print(f"games {n_games} (audited seats {seats}), End Turns {ends}, HB moments {len(hb)} "
          f"= {F:.3f} per audited seat-game, {len(hb) / ends if ends else 0:.3f} per End Turn")
    for name, fn in (("W  wasted (X died next turn, Y alive at its end)", wasted),
                     ("D  X died next turn", died),
                     ("K  opponent could kill X (sure line)", killable)):
        s = share(hb, fn); lo, hi = cluster_boot(hb, lambda rr, fn=fn: share(rr, fn))
        n = len([r for r in hb if fn(r) is not None])
        print(f"  {name:<52} {s if s is None else round(s, 3)}  [{lo if lo is None else round(lo, 3)}, "
              f"{hi if hi is None else round(hi, 3)}]  n={n}")
    verdicts = {}
    for r in hb:
        v = r["can_kill_x"]["verdict"]
        if v == "kill":
            v = f"kill (sure {r['can_kill_x'].get('sure')}/4)"
        verdicts[v] = verdicts.get(v, 0) + 1
    print(f"  can_kill verdicts: {dict(sorted(verdicts.items()))}; game over on the next turn: {len(hb) - len(live)}")
    if live:
        print(f"  Y face damage on the next turn: mean {statistics.mean(r['y_face'] for r in live):.2f}; "
              f"bot took (all sources): mean {statistics.mean(r['bot_took'] for r in live):.2f}")
        fk = [r for r in hb if r["primary"]["x_survives"]]
        print(f"  primary trade is a free kill (X survives the attack): {len(fk)}/{len(hb)}")
        lens = [r["can_kill_x"].get("line_len") for r in hb if r["can_kill_x"].get("line_len")]
        if lens:
            print(f"  kill-line length: " + ", ".join(f"{k}:{lens.count(k)}" for k in sorted(set(lens))))
        won = share(hb, lambda r: r["bot_won_game"])
        print(f"  games the HB side went on to win: {won if won is None else round(won, 3)}")
    on = [r for r in live if r.get("others_n")]
    if on:
        tot = sum(r["others_n"] for r in on); dead = sum(r["others_died"] for r in on)
        print(f"  baseline: the HB side's other followers at those End Turns died next turn {dead}/{tot} = "
              f"{dead / tot:.3f}")
    sure = [r for r in hb if r["can_kill_x"]["verdict"] == "kill" and r["can_kill_x"].get("sure") == 4]
    cheap = [r for r in sure if r["can_kill_x"]["line_len"] <= 3 and r["can_kill_x"]["nodes"] <= 2000]
    c = len(cheap) / len(sure) if sure else None
    print(f"  sure kill lines within 3 actions and 2 000 applies: {len(cheap)}/{len(sure)}")
    # verdict (claude/runbook-holdback1-local.md)
    d = share(hb, died)
    print()
    if F >= 0.25 and d is not None and d >= 0.5 and c is not None and c >= 0.7:
        print("VERDICT: row 1 — brief: a cheap held-back check at the root (default off)")
    elif F >= 0.25 and d is not None and d >= 0.5:
        print("VERDICT: row 2 — no cheap check sees most kills; design note, no brief")
    else:
        print("VERDICT: row 3 — no brief; keep counting in owner reviews")


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--cards", default=str(Path("cards").resolve()))
    sub = p.add_subparsers(dest="cmd", required=True)
    pp = sub.add_parser("play")
    pp.add_argument("--out", required=True); pp.add_argument("--games", type=int, default=256)
    pp.add_argument("--seed", type=int, default=1); pp.add_argument("--spec", default=SERVED)
    pp.add_argument("--pool", default="meta"); pp.add_argument("--workers", type=int, default=os.cpu_count() or 2)
    ps = sub.add_parser("scan")
    ps.add_argument("inputs", nargs="+"); ps.add_argument("--out", required=True)
    ps.add_argument("--budget", type=int, default=20000); ps.add_argument("--depth", type=int, default=6)
    ps.add_argument("--workers", type=int, default=os.cpu_count() or 2)
    pr = sub.add_parser("report")
    pr.add_argument("inputs", nargs="+")
    args = p.parse_args()
    {"play": cmd_play, "scan": cmd_scan, "report": cmd_report}[args.cmd](args)


if __name__ == "__main__":
    main()
