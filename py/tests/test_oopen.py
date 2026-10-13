"""Tablet swing positions: opponent reply opening search (`oopen`)."""

from __future__ import annotations

import importlib.util
import json
import sys
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

FIXTURES = Path(__file__).resolve().parent / "fixtures" / "tablet19"

SERVED_SPEC = (
    "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,"
    "okill=8,hbk=16,alloc=world,threads=1,lostrank=4000"
)

CASES = [
    ("15347574370000780434-97db9ba4-z29pmzb008.json", 25, 15347574370000780447, -5.25, -6.17, -7.40),
    ("16711146681088137021-97db9ba4-gh1qev38sg.json", 45, 16711146681088137041, -28.44, -30.05, -32.86),
    ("17120898228290884610-97db9ba4-jnbkdaf8om.json", 28, 17120898228290884623, -2.41, -4.23, -7.72),
    ("4483542872111018835-97db9ba4-w2kssgalv0.json", 32, 4483542872111018849, 13.49, 11.65, 10.67),
    ("5644357252607789766-97db9ba4-nk0gbw1ual.json", 45, 5644357252607789789, -6.90, -10.17, -17.27),
]


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
    human = cap.get("humanSide", "a").lower()
    for step in cap["actions"][:n]:
        audit.apply_step(game, step)
    bot_side = "b" if human == "a" else "a"
    return game, bot_side


def _world_ends(exp: dict[str, Any]) -> dict[str, int]:
    out: dict[str, int] = {}
    for cand in exp["candidates"]:
        for w in cand.get("worlds") or []:
            end = w.get("end") or "none"
            out[str(end)] = out.get(str(end), 0) + 1
    return out


@pytest.mark.parametrize(
    "fixture,action_idx,bot_seed,served,v1,v2",
    CASES,
)
def test_tablet_swing_positions(
    db,
    fixture: str,
    action_idx: int,
    bot_seed: int,
    served: float,
    v1: float,
    v2: float,
) -> None:
    import arena

    cap = _load_capture(fixture)
    game, _bot_side = _replay(db, cap, action_idx)
    recorded = cap["actions"][action_idx]
    body = {k: v for k, v in recorded.items() if k not in ("value", "bot_value")}
    recorded_action = json.dumps(body, sort_keys=True)

    base = game.bot_action_value(SERVED_SPEC, bot_seed)
    act = game.bot_action(SERVED_SPEC, bot_seed)
    assert abs(base["value"] - served) < 5e-3, f"served mismatch: {base['value']} vs {served}"
    # Leader vs slot target encoding can differ for the same attack line.
    assert json.dumps(act, sort_keys=True) == recorded_action or (
        "attack" in act
        and "attack" in body
        and act["attack"].get("attacker_slot") == body["attack"].get("attacker_slot")
        and act["attack"].get("player") == body["attack"].get("player")
    )

    o1 = game.bot_action_value(f"{SERVED_SPEC},oopen=1", bot_seed)
    o2 = game.bot_action_value(f"{SERVED_SPEC},oopen=2", bot_seed)
    print(
        f"{fixture}@{action_idx}: served={base['value']:.4f} "
        f"oopen=1={o1['value']:.4f} oopen=2={o2['value']:.4f}"
    )
    assert o1["value"] < base["value"]
    assert o2["value"] <= o1["value"]
    tol = 5e-3
    if abs(o1["value"] - v1) >= tol:
        pytest.fail(f"oopen=1 expected {v1}, got {o1['value']}")
    if abs(o2["value"] - v2) >= tol:
        pytest.fail(f"oopen=2 expected {v2}, got {o2['value']}")

    exp0 = game.bot_action_explain(SERVED_SPEC, bot_seed)
    exp2 = game.bot_action_explain(f"{SERVED_SPEC},oopen=2", bot_seed)
    assert _world_ends(exp0).get("opp_open", 0) == 0
    assert _world_ends(exp2).get("opp_open", 0) >= 1
