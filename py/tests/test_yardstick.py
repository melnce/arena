"""Yardstick: per-side policies, seat mirroring, golden coin matrix, Wilson."""

from __future__ import annotations

import importlib.util
import json
from collections import defaultdict
from pathlib import Path

import pytest

_STATS = Path(__file__).resolve().parents[1] / "stats.py"
_spec = importlib.util.spec_from_file_location("arena_stats", _STATS)
assert _spec and _spec.loader
_stats = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_stats)
wilson = _stats.wilson

# Captured on main @ fe63a91 with
# `arena.matchup(db, {basic-forest, basic-rune}, 4, 42, policy="random", threads=1)`
# (then First::Coin). Extra pair fields added by this PR are ignored here.
GOLDEN_COIN_MATRIX = {
    "basic-forest": {
        "basic-forest": {
            "a_wins": 4,
            "b_wins": 0,
            "first_player_wins": 3,
            "games": 4,
            "mean_actions": 91.5,
            "mean_turns": 10.75,
        },
        "basic-rune": {
            "a_wins": 1,
            "b_wins": 3,
            "first_player_wins": 1,
            "games": 4,
            "mean_actions": 79.25,
            "mean_turns": 9.25,
        },
    },
    "basic-rune": {
        "basic-forest": {
            "a_wins": 2,
            "b_wins": 2,
            "first_player_wins": 1,
            "games": 4,
            "mean_actions": 116.5,
            "mean_turns": 14.0,
        },
        "basic-rune": {
            "a_wins": 1,
            "b_wins": 3,
            "first_player_wins": 3,
            "games": 4,
            "mean_actions": 88.5,
            "mean_turns": 10.75,
        },
    },
}


def _two_decks(root: Path) -> dict[str, dict[str, int]]:
    names = ("basic-forest", "basic-rune")
    out = {}
    for n in names:
        path = root / "oracle" / "decks" / f"{n}.json"
        out[n] = {str(k): int(v) for k, v in json.loads(path.read_text()).items()}
    return out


def _forest(root: Path) -> dict[str, dict[str, int]]:
    path = root / "oracle" / "decks" / "basic-forest.json"
    return {"basic-forest": {str(k): int(v) for k, v in json.loads(path.read_text()).items()}}


def _by_pair(records: list[dict]) -> dict[tuple[str, str], list[dict]]:
    grouped: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for rec in records:
        grouped[(rec["a"], rec["b"])].append(rec)
    for recs in grouped.values():
        recs.sort(key=lambda r: r["g"])
    return grouped


def _recompute(recs: list[dict]) -> dict:
    first_player_wins = sum(1 for r in recs if r["winner"] is not None and r["winner"] == r["first"])
    a_games_as_first = sum(1 for r in recs if r["first"] == "a")
    a_wins_as_first = sum(1 for r in recs if r["first"] == "a" and r["winner"] == "a")
    draws = sum(1 for r in recs if r["winner"] is None)
    end = {
        "lethal": 0,
        "deckout": 0,
        "turn_cap": 0,
        "action_cap": 0,
        "no_legal": 0,
        "illegal": 0,
    }
    for r in recs:
        end[r["end"]] += 1
    return {
        "first_player_wins": first_player_wins,
        "a_games_as_first": a_games_as_first,
        "a_wins_as_first": a_wins_as_first,
        "draws": draws,
        "end": end,
    }


def test_seat_mirroring_and_record_aggregates(db, root: Path) -> None:
    import arena

    decks = _two_decks(root)
    alt = arena.matchup(
        db,
        decks,
        4,
        11,
        policy="random",
        threads=1,
        first="alternate",
        records=True,
    )
    grouped = _by_pair(alt["records"])
    assert set(grouped) == {
        ("basic-forest", "basic-forest"),
        ("basic-forest", "basic-rune"),
        ("basic-rune", "basic-forest"),
        ("basic-rune", "basic-rune"),
    }
    for recs in grouped.values():
        assert [r["first"] for r in recs] == ["a", "b", "a", "b"]
        pair = alt["matrix"][recs[0]["a"]][recs[0]["b"]]
        got = _recompute(recs)
        assert pair["first_player_wins"] == got["first_player_wins"]
        assert pair["a_games_as_first"] == got["a_games_as_first"]
        assert pair["a_wins_as_first"] == got["a_wins_as_first"]
        assert pair["draws"] == got["draws"]
        assert pair["end"] == got["end"]

    only_a = arena.matchup(
        db,
        decks,
        3,
        11,
        policy="random",
        threads=1,
        first="a",
        records=True,
    )
    assert all(r["first"] == "a" for r in only_a["records"])
    for recs in _by_pair(only_a["records"]).values():
        pair = only_a["matrix"][recs[0]["a"]][recs[0]["b"]]
        assert _recompute(recs)["end"] == pair["end"]
        assert pair["a_games_as_first"] == len(recs)


def test_per_side_policies_land_on_the_right_seat(db, root: Path) -> None:
    import arena

    decks = _forest(root)
    h0_a = arena.matchup(
        db,
        decks,
        40,
        20260910,
        policy_a="h0-fast",
        policy_b="random",
        first="alternate",
        threads=1,
    )
    cell = h0_a["matrix"]["basic-forest"]["basic-forest"]
    decisive = cell["a_wins"] + cell["b_wins"]
    assert decisive > 0
    rate = cell["a_wins"] / decisive
    assert rate >= 0.85, f"h0-fast as A: {cell['a_wins']}/{decisive} = {rate:.3f}"

    h0_b = arena.matchup(
        db,
        decks,
        40,
        20260910,
        policy_a="random",
        policy_b="h0-fast",
        first="alternate",
        threads=1,
    )
    cell = h0_b["matrix"]["basic-forest"]["basic-forest"]
    decisive = cell["a_wins"] + cell["b_wins"]
    assert decisive > 0
    rate = cell["b_wins"] / decisive
    assert rate >= 0.85, f"h0-fast as B: {cell['b_wins']}/{decisive} = {rate:.3f}"


def test_h0_fast_determinism(db, root: Path) -> None:
    import arena

    decks = _forest(root)
    kwargs = dict(
        db=db,
        decks=decks,
        games=2,
        seed=99,
        policy_a="h0-fast",
        policy_b="h0-fast",
        first="alternate",
        records=True,
    )
    a = arena.matchup(**kwargs, threads=1)
    b = arena.matchup(**kwargs, threads=1)
    assert a == b
    none = arena.matchup(**kwargs, threads=None)
    assert none["matrix"] == a["matrix"]
    assert none["records"] == a["records"]


def test_random_coin_matrix_unchanged(db, root: Path) -> None:
    import arena

    decks = _two_decks(root)
    got = arena.matchup(db, decks, 4, 42, policy="random", threads=1, first="coin")
    for a_name, row in GOLDEN_COIN_MATRIX.items():
        for b_name, cell in row.items():
            pair = got["matrix"][a_name][b_name]
            for key, value in cell.items():
                assert pair[key] == value, (a_name, b_name, key, pair[key], value)


def test_spec_errors_name_the_token(db, root: Path) -> None:
    import arena

    decks = _forest(root)
    with pytest.raises(ValueError, match="depht"):
        arena.matchup(db, decks, 1, 1, policy_a="h0:depht=2", policy_b="random")
    with pytest.raises(ValueError, match="hzero"):
        arena.matchup(db, decks, 1, 1, policy="hzero")


def test_wilson_values() -> None:
    def r3(pair: tuple[float, float]) -> tuple[float, float]:
        return (round(pair[0], 3), round(pair[1], 3))

    assert r3(wilson(50, 100)) == (0.404, 0.596)
    assert r3(wilson(20, 20)) == (0.839, 1.000)
    assert r3(wilson(0, 20)) == (0.000, 0.161)
    assert wilson(0, 0) == (0.0, 1.0)
