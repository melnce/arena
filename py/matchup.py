#!/usr/bin/env python3
"""Matchup win-rate matrix over oracle/decks (stdlib + arena)."""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path


def repo_root() -> Path:
    here = Path(__file__).resolve().parent
    for d in (Path.cwd(), here, *here.parents):
        if (d / "oracle" / "decks").is_dir() and (d / "cards").is_dir():
            return d
    return Path.cwd()


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
        out[name] = {str(k): int(v) for k, v in raw.items()}
    if wanted is not None:
        missing = sorted(wanted - set(out))
        if missing:
            raise SystemExit(f"unknown --decks: {', '.join(missing)}")
    if not out:
        raise SystemExit("no decks selected")
    return out


def cell(pair: dict) -> tuple[float, float, float]:
    g = pair["games"] or 1
    return (
        pair["a_wins"] / g,
        pair["first_player_wins"] / g,
        pair["mean_turns"],
    )


def aligned_tables(matrix: dict[str, dict[str, dict]], names: list[str]) -> str:
    col_w = max(14, max(len(n) for n in names) + 2)
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

    return "\n".join(win) + "\n\n" + "\n".join(fp)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--games", type=int, default=200)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--decks", nargs="*", default=None, help="restrict to these deck stems")
    parser.add_argument("--threads", type=int, default=None)
    parser.add_argument("--policy", default="random", choices=("random", "first-legal"))
    parser.add_argument("--out", default="matchup.json")
    parser.add_argument("--cards", default=None, help="cards/ or repo root (default: repo cards/)")
    args = parser.parse_args(argv)

    import arena

    root = repo_root()
    cards = args.cards or str(root / "cards")
    db = arena.load_cards(cards)
    decks = load_deck_files(root / "oracle" / "decks", args.decks)
    names = list(decks.keys())
    n_pairs = len(names) * len(names)
    total = n_pairs * args.games
    print(
        f"matchup: {len(names)} decks × {args.games} games = {total} games "
        f"(seed={args.seed}, policy={args.policy}, threads={args.threads})",
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
    )
    secs = time.perf_counter() - t0
    gps = total / secs if secs > 0 else 0.0
    result["seconds"] = secs
    result["games_per_second"] = gps
    result["decks"] = names
    text = aligned_tables(result["matrix"], names)
    print(text)
    print(f"\n{total} games in {secs:.3f}s ({gps:.1f} games/s)", file=sys.stderr)
    out = Path(args.out)
    out.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(f"wrote {out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
