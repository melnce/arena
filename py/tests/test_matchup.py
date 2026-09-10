"""Matchup determinism: same seed repeats; threads=1 equals threads=None."""

from __future__ import annotations

import json
from pathlib import Path


def _two_decks(root: Path) -> dict[str, dict[str, int]]:
    names = ("basic-forest", "basic-rune")
    out = {}
    for n in names:
        path = root / "oracle" / "decks" / f"{n}.json"
        out[n] = {str(k): int(v) for k, v in json.loads(path.read_text()).items()}
    return out


def test_matchup_same_seed_identical(db, root: Path) -> None:
    import arena

    decks = _two_decks(root)
    a = arena.matchup(db, decks, 2, 42, policy="random", threads=1)
    b = arena.matchup(db, decks, 2, 42, policy="random", threads=1)
    assert a["matrix"] == b["matrix"]
    assert a["seed"] == b["seed"] == 42


def test_matchup_threads_one_equals_default(db, root: Path) -> None:
    import arena

    decks = _two_decks(root)
    one = arena.matchup(db, decks, 2, 7, policy="random", threads=1)
    many = arena.matchup(db, decks, 2, 7, policy="random", threads=None)
    assert one["matrix"] == many["matrix"]


def test_play_random_reaches_terminal_or_cap(db, root: Path) -> None:
    import arena

    deck = json.loads(
        (root / "oracle" / "decks" / "basic-forest.json").read_text()
    )
    r = arena.play_random(db, 3, deck, deck, first="coin")
    assert r["first"] in ("a", "b")
    assert r["turns"] >= 0
    assert r["actions"] >= 0
    assert r["winner"] in ("a", "b", None)
