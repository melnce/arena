"""Review11 positions for opt-in h0 fuseguard."""

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


def _load_capture(name: str) -> dict[str, Any]:
    return json.loads((FIXTURES / name).read_text())


def _bot_side(cap: dict[str, Any]) -> str:
    return "b" if cap["humanSide"] == "a" else "a"


def _player_of_step(step: dict[str, Any]) -> str | None:
    body = audit.action_body(step)
    if not isinstance(body, dict) or not body:
        return None
    val = next(iter(body.values()))
    if isinstance(val, dict):
        return val.get("player")
    return None


def _bot_explain_seed(cap: dict[str, Any], n: int) -> int:
    bot = _bot_side(cap)
    count = sum(1 for step in cap["actions"][:n] if _player_of_step(step) == bot)
    return int(cap["seed"]) + count


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


def _has_fuse_candidate(exp: dict[str, Any]) -> bool:
    for cand in exp.get("candidates") or []:
        if "fuse" in cand.get("action", {}):
            return True
    return False


def _chosen_is_fuse(exp: dict[str, Any]) -> bool:
    return "fuse" in exp.get("chosen", {})


@pytest.mark.parametrize(
    ("capture", "step", "expect_fuse_off", "expect_fuse_on", "fuse_dropped_on"),
    (
        ("5935752092966957148-104cf827.json", 54, True, False, lambda n: n >= 1),
        ("7260600291112487607-104cf827.json", 2, True, False, lambda n: n >= 1),
        ("5935752092966957148-104cf827.json", 8, True, True, lambda n: n == 0),
        ("3505567890135518157-104cf827.json", 30, True, True, lambda n: n == 0),
    ),
)
def test_fuseguard_review11_positions(
    db,
    capture: str,
    step: int,
    expect_fuse_off: bool,
    expect_fuse_on: bool,
    fuse_dropped_on,
) -> None:
    cap = _load_capture(capture)
    game = _replay(db, cap, step)
    seed = _bot_explain_seed(cap, step)

    off = game.bot_action_explain(SERVED_SPEC, seed)
    on = game.bot_action_explain(f"{SERVED_SPEC},fuseguard=1", seed)

    assert off["fuse_dropped"] == 0
    assert _chosen_is_fuse(off) is expect_fuse_off
    assert _has_fuse_candidate(off) is expect_fuse_off or not expect_fuse_on
    assert _chosen_is_fuse(on) is expect_fuse_on
    assert _has_fuse_candidate(on) is expect_fuse_on
    assert fuse_dropped_on(on.get("fuse_dropped", 0))

    if capture == "5935752092966957148-104cf827.json" and step == 54:
        assert abs(float(off["value"]) - 32.5778) < 0.01


def test_explain_fields_wbase_and_lost_rerank(db) -> None:
    cap = _load_capture("5368404598479714909-104cf827.json")
    game = _replay(db, cap, 50)
    seed = _bot_explain_seed(cap, 50)

    base = game.bot_action_explain(SERVED_SPEC, seed)
    assert base.get("wbase") is None
    assert base.get("lost_rerank") is None

    wbase = game.bot_action_explain(f"{SERVED_SPEC},wbase=7", seed)
    assert wbase.get("wbase") == 7

    lost = game.bot_action_explain(f"{SERVED_SPEC},lostrank=2000", seed)
    assert lost.get("lost_rerank") is not None


@pytest.mark.parametrize(
    ("capture", "step"),
    (
        ("5935752092966957148-104cf827.json", 54),
        ("7260600291112487607-104cf827.json", 2),
        ("5935752092966957148-104cf827.json", 8),
        ("3505567890135518157-104cf827.json", 30),
    ),
)
def test_fuseguard_timing_unchanged(db, capture: str, step: int) -> None:
    cap = _load_capture(capture)
    game = _replay(db, cap, step)
    seed = _bot_explain_seed(cap, step)

    t0 = time.perf_counter()
    game.bot_action_explain(SERVED_SPEC, seed)
    off_ms = (time.perf_counter() - t0) * 1000.0

    t1 = time.perf_counter()
    game.bot_action_explain(f"{SERVED_SPEC},fuseguard=1", seed)
    on_ms = (time.perf_counter() - t1) * 1000.0

    # Same order of magnitude — fuseguard only filters candidates.
    assert on_ms < off_ms * 3 + 50
