"""mull=leaf opening mulligan (default off)."""

import json
from pathlib import Path

import pytest

arena = pytest.importorskip("arena")

SERVED = (
    "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,"
    "fuseguard=1,okill=15,hbk=16,alloc=world,threads=8,lostrank=4000,"
    "order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,"
    "net=h0-value-v69"
)


def _load_deck_json(stem: str) -> dict:
    path = Path(__file__).resolve().parents[2] / "oracle/decks" / f"{stem}.json"
    return json.loads(path.read_text(encoding="utf-8"))


def _new_at_mulligan(db, seed: int):
    da = _load_deck_json("meta-rune-test-subject")
    dbk = _load_deck_json("meta-sword-rally")
    game = arena.Game(db, seed, da, dbk, first="a")
    assert game.phase == "mulligan"
    return game


def test_mull_leaf_bot_action(db):
    game = _new_at_mulligan(db, 42)
    spec = f"{SERVED},mull=leaf"
    out = game.bot_action(spec, seed=1001)
    assert "mulligan" in out
    again = game.bot_action(spec, seed=1001)
    assert out == again


def test_mull_leaf_explain_values(db):
    game = _new_at_mulligan(db, 7)
    spec = f"{SERVED},mull=leaf"
    exp = game.bot_action_explain(spec, seed=2002)
    mull = exp.get("mull")
    assert mull is not None
    assert mull["mode"] == "leaf"
    legal_len = len(game.legal())
    assert len(mull["values"]) == legal_len
