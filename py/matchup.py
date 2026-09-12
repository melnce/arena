#!/usr/bin/env python3
"""Matchup win-rate matrix over oracle/decks (stdlib + arena)."""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

from stats import wilson


def repo_root() -> Path:
    here = Path(__file__).resolve().parent
    for d in (Path.cwd(), here, *here.parents):
        if (d / "oracle" / "decks").is_dir() and (d / "cards").is_dir():
            return d
    return Path.cwd()


def parse_deck_json(raw: object) -> dict[str, int]:
    """Client import shapes: `{id: count}` or `[ids]`."""
    if isinstance(raw, list):
        out: dict[str, int] = {}
        for item in raw:
            key = str(item)
            out[key] = out.get(key, 0) + 1
        return out
    if isinstance(raw, dict):
        out = {}
        for k, n in raw.items():
            try:
                count = int(n)
            except (TypeError, ValueError):
                continue
            if count <= 0:
                continue
            out[str(k)] = count
        return out
    raise SystemExit("deck JSON must be {id: count} or [ids]")


def load_deck_files(decks_dir: Path, restrict: list[str] | None) -> dict[str, dict[str, int]]:
    names = restrict
    out: dict[str, dict[str, int]] = {}
    files = sorted(decks_dir.glob("*.json"))
    if not files:
        raise SystemExit(f"no decks in {decks_dir}")
    wanted = set(names) if names else None
    for path in files:
        name = path.stem
        if wanted is not None and name not in wanted:
            continue
        with path.open() as f:
            raw = json.load(f)
        out[name] = parse_deck_json(raw)
    if wanted is not None:
        missing = sorted(wanted - set(out))
        if missing:
            raise SystemExit(f"unknown --decks: {', '.join(missing)}")
    if not out:
        raise SystemExit("no decks selected")
    return out


def load_extra_deck_files(paths: list[str]) -> dict[str, dict[str, int]]:
    out: dict[str, dict[str, int]] = {}
    for p in paths:
        path = Path(p)
        with path.open() as f:
            raw = json.load(f)
        out[path.stem] = parse_deck_json(raw)
    return out


def cell(pair: dict) -> tuple[float, float, float]:
    g = pair["games"] or 1
    return (
        pair["a_wins"] / g,
        pair["first_player_wins"] / g,
        pair["mean_turns"],
    )


def decisive_wr(pair: dict) -> tuple[float, float, float, int]:
    n = int(pair["a_wins"]) + int(pair["b_wins"])
    wr = pair["a_wins"] / n if n else 0.0
    lo, hi = wilson(int(pair["a_wins"]), n)
    return wr, lo, hi, n


def aligned_tables(matrix: dict[str, dict[str, dict]], names: list[str]) -> str:
    col_w = max(24, max(len(n) for n in names) + 2)
    name_w = max(len(n) for n in names)

    def hdr(title: str) -> list[str]:
        lines = [title, " " * name_w + "".join(n.rjust(col_w) for n in names)]
        return lines

    win = hdr("Win rate (row = A, col = B; cell = A's win rate)")
    for a in names:
        row = a.ljust(name_w)
        for b in names:
            wr, _, _ = cell(matrix[a][b])
            row += f"{wr:5.3f}".rjust(col_w)
        win.append(row)

    fp = hdr("First-player win rate / mean turns")
    for a in names:
        row = a.ljust(name_w)
        for b in names:
            _, fpr, turns = cell(matrix[a][b])
            row += f"{fpr:5.3f} / {turns:5.1f}".rjust(col_w)
        fp.append(row)

    ci = hdr("A win rate over decisive games, Wilson 95% [lo, hi]")
    for a in names:
        row = a.ljust(name_w)
        for b in names:
            wr, lo, hi, _ = decisive_wr(matrix[a][b])
            row += f"{wr:5.3f} [{lo:5.3f}, {hi:5.3f}]".rjust(col_w)
        ci.append(row)

    return "\n".join(win) + "\n\n" + "\n".join(fp) + "\n\n" + "\n".join(ci)


def summarize(
    result: dict,
    names: list[str],
    secs: float,
    total: int,
) -> dict:
    matrix = result["matrix"]
    a_wins = b_wins = fp_wins = 0
    turns = 0.0
    actions = 0.0
    games = 0
    end = {
        "lethal": 0,
        "deckout": 0,
        "turn_cap": 0,
        "action_cap": 0,
        "no_legal": 0,
        "illegal": 0,
    }
    for a in names:
        for b in names:
            p = matrix[a][b]
            g = int(p["games"])
            games += g
            a_wins += int(p["a_wins"])
            b_wins += int(p["b_wins"])
            fp_wins += int(p["first_player_wins"])
            turns += float(p["mean_turns"]) * g
            actions += float(p["mean_actions"]) * g
            for k in end:
                end[k] += int(p.get("end", {}).get(k, 0))
    decisive = a_wins + b_wins
    draws = games - a_wins - b_wins
    wr = a_wins / decisive if decisive else 0.0
    lo, hi = wilson(a_wins, decisive)
    fp_n = decisive
    fp_rate = fp_wins / fp_n if fp_n else 0.0
    per_deck = {}
    for d in names:
        row_aw = row_dec = 0
        col_bw = col_dec = 0
        for other in names:
            rp = matrix[d][other]
            row_aw += int(rp["a_wins"])
            row_dec += int(rp["a_wins"]) + int(rp["b_wins"])
            cp = matrix[other][d]
            col_bw += int(cp["b_wins"])
            col_dec += int(cp["a_wins"]) + int(cp["b_wins"])
        per_deck[d] = {
            "policy_a_as_row": row_aw / row_dec if row_dec else 0.0,
            "policy_b_as_col": col_bw / col_dec if col_dec else 0.0,
        }
    gps = total / secs if secs > 0 else 0.0
    return {
        "policy_a": result["policy_a"],
        "policy_b": result["policy_b"],
        "games": games,
        "decisive": decisive,
        "draws": draws,
        "policy_a_win_rate": wr,
        "wilson95": [lo, hi],
        "first_player_win_rate": fp_rate,
        "mean_turns": turns / games if games else 0.0,
        "mean_actions": actions / games if games else 0.0,
        "end": end,
        "seconds": secs,
        "games_per_second": gps,
        "cpu_count": os.cpu_count(),
        "per_deck": per_deck,
    }


def format_summary(s: dict) -> str:
    lo, hi = s["wilson95"]
    e = s["end"]
    end_s = " ".join(f"{k}={e[k]}" for k in e)
    lines = [
        "=== summary ===",
        f"{s['policy_a']} vs {s['policy_b']}",
        f"games: {s['games']}  decisive: {s['decisive']}  draws: {s['draws']}",
        f"policy_a win rate: {s['policy_a_win_rate']:.3f} [{lo:.3f}, {hi:.3f}]",
        f"first-player win rate: {s['first_player_win_rate']:.3f}",
        f"mean turns: {s['mean_turns']:.2f}  mean actions: {s['mean_actions']:.2f}",
        f"end: {end_s}",
        f"{s['seconds']:.3f}s  {s['games_per_second']:.2f} games/s  cpus={s['cpu_count']}",
        "per deck:",
    ]
    for name, d in s["per_deck"].items():
        lines.append(
            f"  {name}: policy_a (row) {d['policy_a_as_row']:.3f}  "
            f"policy_b (col) {d['policy_b_as_col']:.3f}"
        )
    return "\n".join(lines)


def add_wilson95(matrix: dict[str, dict[str, dict]], names: list[str]) -> None:
    for a in names:
        for b in names:
            wr, lo, hi, _ = decisive_wr(matrix[a][b])
            matrix[a][b]["wilson95"] = [lo, hi]
            matrix[a][b]["wr_decisive"] = wr


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--games", type=int, default=200)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--decks", nargs="*", default=None, help="restrict to these deck stems")
    parser.add_argument(
        "--deck-file",
        nargs="*",
        default=[],
        metavar="PATH",
        help="extra decks; name = file stem; {{id: count}} or [ids]",
    )
    parser.add_argument("--threads", type=int, default=None)
    parser.add_argument("--policy", default="random", help="spec for both seats unless overridden")
    parser.add_argument("--policy-a", default=None, help="seat A spec (default: --policy)")
    parser.add_argument("--policy-b", default=None, help="seat B spec (default: --policy)")
    parser.add_argument(
        "--first",
        default="alternate",
        choices=("alternate", "coin", "a", "b"),
        help="who goes first (default: alternate, seats mirrored)",
    )
    parser.add_argument("--records", action="store_true", help="include per-game records in JSON")
    parser.add_argument("--out", default="matchup.json")
    parser.add_argument("--cards", default=None, help="cards/ or repo root (default: repo cards/)")
    args = parser.parse_args(argv)

    import arena

    root = repo_root()
    cards = args.cards or str(root / "cards")
    db = arena.load_cards(cards)
    decks = load_deck_files(root / "oracle" / "decks", args.decks)
    decks.update(load_extra_deck_files(args.deck_file))
    names = list(decks.keys())
    n_pairs = len(names) * len(names)
    total = n_pairs * args.games
    policy_a = args.policy_a if args.policy_a is not None else args.policy
    policy_b = args.policy_b if args.policy_b is not None else args.policy
    print(
        f"matchup: {len(names)} decks × {args.games} games = {total} games "
        f"(seed={args.seed}, policy_a={policy_a}, policy_b={policy_b}, "
        f"first={args.first}, threads={args.threads})",
        file=sys.stderr,
    )
    t0 = time.perf_counter()
    result = arena.matchup(
        db,
        decks,
        args.games,
        args.seed,
        policy=args.policy,
        threads=args.threads,
        policy_a=policy_a,
        policy_b=policy_b,
        first=args.first,
        records=args.records,
    )
    secs = time.perf_counter() - t0
    gps = total / secs if secs > 0 else 0.0
    add_wilson95(result["matrix"], names)
    summary = summarize(result, names, secs, total)
    result["seconds"] = secs
    result["games_per_second"] = gps
    result["decks"] = names
    result["summary"] = summary
    text = aligned_tables(result["matrix"], names)
    print(text)
    print()
    print(format_summary(summary))
    print(f"\n{total} games in {secs:.3f}s ({gps:.1f} games/s)", file=sys.stderr)
    out = Path(args.out)
    out.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(f"wrote {out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
