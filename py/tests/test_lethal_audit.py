"""Forced-lethal PyO3 export and the offline audit runner."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import pytest

pytest.importorskip("arena")

_PY = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("arena_lethal_audit", _PY / "lethal_audit.py")
assert _spec and _spec.loader
audit = importlib.util.module_from_spec(_spec)
sys.modules["arena_lethal_audit"] = audit
_spec.loader.exec_module(audit)


def _decks(root: Path) -> tuple[dict[str, int], dict[str, int]]:
    forest = json.loads((root / "oracle" / "decks" / "basic-forest.json").read_text())
    rune = json.loads((root / "oracle" / "decks" / "basic-rune.json").read_text())
    return {str(k): int(v) for k, v in forest.items()}, {str(k): int(v) for k, v in rune.items()}


def test_pool_and_decks_conflict() -> None:
    with pytest.raises(SystemExit) as exc:
        audit.parse_args(["--pool", "meta", "--decks", "basic-forest"])
    assert "--pool" in str(exc.value)
    assert "--decks" in str(exc.value)


def test_default_pool_is_meta() -> None:
    args = audit.parse_args([])
    assert args.pool == "meta"
    assert args.decks is None


def test_rate_block_never_divides_by_unknowns() -> None:
    bucket = audit.empty_bucket()
    bucket["lethal"] = 2
    bucket["none"] = 8
    bucket["unknown"] = 5
    bucket["hits"] = 2
    block = audit.rate_block(bucket)
    assert block["decisions_with_a_verdict"] == 10
    assert block["rate"] == pytest.approx(0.2)
    assert block["unknown"] == 5
    assert block["unknown_rate"] == pytest.approx(5 / 15)
    assert block["asked"] == 15
    # The headline rate is n / decided, not n / (decided + unknown).
    assert block["rate"] != pytest.approx(2 / 15)


def test_bot_converts_this_turn() -> None:
    bot_win = [
        {"play": {"player": "b", "hand_pos": 0, "card": "x"}},
        {"attack": {"player": "b", "attacker_slot": 0, "target": "leader"}},
    ]
    assert audit.bot_converts_this_turn(bot_win, 0, "b", "b") is True
    ended = [
        {"play": {"player": "b", "hand_pos": 0, "card": "x"}},
        {"end_turn": {"player": "b"}},
        {"play": {"player": "a", "hand_pos": 0, "card": "y"}},
    ]
    assert audit.bot_converts_this_turn(ended, 0, "b", "a") is False
    assert audit.bot_converts_this_turn(ended, 0, "b", "b") is False


def test_forced_lethal_export_shape(db, root: Path) -> None:
    import arena

    deck_a, deck_b = _decks(root)
    game = arena.Game(db, 1, deck_a, deck_b, "a")
    # Skip both mulligans so we are on a real turn.
    for _ in range(4):
        if game.phase != "mulligan":
            break
        game.apply(game.bot_action("first-legal", 1))
    v0 = arena.forced_lethal(game, 0)
    assert v0["verdict"] in {"lethal", "none", "unknown"}
    assert "nodes" in v0
    assert "is_lethal" not in v0
    # A live turn with expandable actions and budget 0 is Unknown, not None.
    if v0["verdict"] == "unknown":
        assert v0["nodes"] == 0
    v1 = arena.forced_lethal(game, 2_000)
    assert v1["verdict"] in {"lethal", "none", "unknown"}
    if v1["verdict"] == "lethal":
        assert isinstance(v1["line"], list)
        assert "rng_dependent" in v1
    else:
        assert "line" not in v1
        assert "rng_dependent" not in v1
    v2 = arena.forced_lethal(game, 2_000)
    assert v1 == v2


def test_audit_game_counts_over_verdicts(db, root: Path) -> None:
    deck_a, deck_b = _decks(root)
    rec = audit.play_one(db, 1, deck_a, deck_b, "a", "h0-fast", "h0-fast")
    assert rec["actions"], "expected a recorded prefix"
    row = audit.audit_game(db, rec, budget=80, seats=("b",))
    missed = audit.rate_block(row["missed"])
    handed = audit.rate_block(row["handed"])
    assert missed["asked"] == missed["decisions_with_a_verdict"] + missed["unknown"]
    assert handed["asked"] == handed["decisions_with_a_verdict"] + handed["unknown"]
    if missed["decisions_with_a_verdict"]:
        assert missed["rate"] == pytest.approx(
            missed["n"] / missed["decisions_with_a_verdict"]
        )
    # A boolean would have hidden Unknown. The runner must keep the split.
    assert "unknown_rate" in missed
    assert "unknown_rate" in handed
