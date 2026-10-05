"""Hand-reading (`hread`) opponent-hand sampling on a review11 position."""

from __future__ import annotations

import importlib.util
import json
import sys
import time
from pathlib import Path
from typing import Any

import pytest

pytest.importorskip("arena")

_PY = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("arena_lethal_audit", _PY / "lethal_audit.py")
assert _spec and _spec.loader
_mod = importlib.util.module_from_spec(_spec)
sys.modules["arena_lethal_audit"] = _mod
_spec.loader.exec_module(_mod)
audit = _mod

FIXTURES = Path(__file__).resolve().parent / "fixtures" / "review11"
SERVED_SPEC = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000"
GARODETH = "10954120"


def _load_capture(name: str) -> dict[str, Any]:
    return json.loads((FIXTURES / name).read_text())


def _replay(db, cap: dict[str, Any], n: int):
    import arena

    game = arena.Game(
        db,
        int(cap["seed"]),
        cap["deckA"],
        cap["deckB"],
        cap.get("first") or "coin",
    )
    for step in cap["actions"][:n]:
        audit.apply_step(game, step)
    return game


def _garodeth_share(hands: list[list[str]]) -> float:
    n = len(hands)
    assert n > 0
    hits = sum(1 for h in hands if GARODETH in h)
    return hits / n


def test_hread_garodeth_share_review11(db) -> None:
    cap = _load_capture("5120746196322041219-5ce21003.json")
    game = _replay(db, cap, 60)
    assert game.active == "b"

    off_hands = game.sample_opponent_hands(SERVED_SPEC, seed=1, n=2000)
    on_hands = game.sample_opponent_hands(f"{SERVED_SPEC},hread=on", seed=1, n=2000)

    assert all(len(h) == 3 for h in off_hands)
    assert all(len(h) == 3 for h in on_hands)

    off_share = _garodeth_share(off_hands)
    on_share = _garodeth_share(on_hands)

    print(f"Garodeth share without hread: {off_share:.4f}")
    print(f"Garodeth share with hread=on: {on_share:.4f}")

    assert 0.22 <= off_share <= 0.31, off_share
    assert 0.37 <= on_share <= 0.50, on_share


@pytest.mark.parametrize("step", (40, 50, 55, 60, 65))
def test_hread_timing_midgame(db, step: int) -> None:
    cap = _load_capture("5120746196322041219-5ce21003.json")
    game = _replay(db, cap, step)

    t0 = time.perf_counter()
    game.bot_action_explain(SERVED_SPEC, seed=1)
    off_ms = (time.perf_counter() - t0) * 1000.0

    t1 = time.perf_counter()
    game.bot_action_explain(f"{SERVED_SPEC},hread=on", seed=1)
    on_ms = (time.perf_counter() - t1) * 1000.0

    print(f"step {step}: off={off_ms:.1f}ms on={on_ms:.1f}ms ratio={on_ms/off_ms:.2f}")

    assert on_ms < off_ms * 3 + 500
