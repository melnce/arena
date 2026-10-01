#!/usr/bin/env python3
"""Play h0 self-play games and save tkill test fixtures (missed deterministic kills)."""
from __future__ import annotations

import json
import sys
from pathlib import Path

_HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(_HERE.parent / "py"))

from lethal_audit import audit_game, play_one, schedule  # noqa: E402
from matchup import DEFAULT_POOL, resolve_selected_decks  # noqa: E402
from runlib import repo_root  # noqa: E402

OUT = Path(__file__).resolve().parents[1] / "engine/tests/fixtures/tkill"
GAMES = 128
SEED = 1
MAX_FIXTURES = 5


def main() -> int:
    import arena

    root = repo_root()
    db = arena.load_cards(str(root / "cards"))
    _, decks = resolve_selected_decks(root / "oracle" / "decks", DEFAULT_POOL, None)
    names = list(decks.keys())
    OUT.mkdir(parents=True, exist_ok=True)
    saved = 0
    for n, (da, dbk, first) in enumerate(schedule(names, GAMES, "a")):
        if saved >= MAX_FIXTURES:
            break
        rec = play_one(db, SEED + n, decks[da], decks[dbk], first, "h0", "h0")
        row = audit_game(db, rec, budget=50_000, seats=("a", "b"))
        for dec in row.get("decisions") or []:
            if dec.get("kind") != "missed_lethal":
                continue
            if not dec.get("hit"):
                continue
            if dec.get("rng_dependent"):
                continue
            ply = int(dec["ply"])
            cap = {
                "seed": rec["seed"],
                "deckA": rec["deckA"],
                "deckB": rec["deckB"],
                "first": rec["first"],
                "actions": rec["actions"][:ply],
                "ply": ply,
                "game_id": rec.get("game_id"),
                "deck_a_name": da,
                "deck_b_name": dbk,
                "acting": dec.get("acting"),
            }
            name = f"{rec['seed']}-{ply:04d}.json"
            (OUT / name).write_text(json.dumps(cap, indent=2) + "\n", encoding="utf-8")
            print(f"saved {name} ({da} vs {dbk} acting={dec.get('acting')})")
            saved += 1
            if saved >= MAX_FIXTURES:
                break
    if saved < 3:
        raise SystemExit(f"only saved {saved} fixtures (need 3–5)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
