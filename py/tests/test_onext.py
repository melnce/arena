"""Tablet swing positions: opponent next-turn lethal glance after reply (`onext`)."""

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

# baseline without onext (from test_oopen.py)
CASES = [
    ("15347574370000780434-97db9ba4-z29pmzb008.json", 25, 15347574370000780447, -5.25),
    ("16711146681088137021-97db9ba4-gh1qev38sg.json", 45, 16711146681088137041, -28.44),
    ("17120898228290884610-97db9ba4-jnbkdaf8om.json", 28, 17120898228290884623, -2.41),
    ("4483542872111018835-97db9ba4-w2kssgalv0.json", 32, 4483542872111018849, 13.49),
    ("5644357252607789766-97db9ba4-nk0gbw1ual.json", 45, 5644357252607789789, -6.90),
]

ONEXT40_VALUE: dict[str, float] = {
    "15347574370000780434-97db9ba4-z29pmzb008.json": -42.50,
    "16711146681088137021-97db9ba4-gh1qev38sg.json": -40.00,
    "17120898228290884610-97db9ba4-jnbkdaf8om.json": -34.94,
    "4483542872111018835-97db9ba4-w2kssgalv0.json": -9.37,
    "5644357252607789766-97db9ba4-nk0gbw1ual.json": -24.15,
}

ONEXT40_OOPEN1_VALUE: dict[str, float] = {
    "15347574370000780434-97db9ba4-z29pmzb008.json": -42.50,
    "16711146681088137021-97db9ba4-gh1qev38sg.json": -40.00,
    "17120898228290884610-97db9ba4-jnbkdaf8om.json": -34.97,
    "4483542872111018835-97db9ba4-w2kssgalv0.json": -17.01,
    "5644357252607789766-97db9ba4-nk0gbw1ual.json": -30.91,
}

# Exact bot moves with `,onext=40` (and without `onext` except 4483…).
MOVE_ONEXT40: dict[str, dict[str, Any]] = {
    "15347574370000780434-97db9ba4-z29pmzb008.json": {"end_turn": {"player": "b"}},
    "16711146681088137021-97db9ba4-gh1qev38sg.json": {
        "evolve": {"player": "b", "slot": 0, "super": False},
    },
    "17120898228290884610-97db9ba4-jnbkdaf8om.json": {"end_turn": {"player": "b"}},
    "4483542872111018835-97db9ba4-w2kssgalv0.json": {
        "attack": {"attacker_slot": 0, "player": "b", "target": "leader"},
    },
    "5644357252607789766-97db9ba4-nk0gbw1ual.json": {
        "attack": {"attacker_slot": 1, "player": "b", "target": {"slot": 0}},
    },
}

# Served spec without `onext`: same as MOVE_ONEXT40 except 4483… attacks follower slot 0.
MOVE_SERVED: dict[str, dict[str, Any]] = {
    **{k: v for k, v in MOVE_ONEXT40.items() if k != "4483542872111018835-97db9ba4-w2kssgalv0.json"},
    "4483542872111018835-97db9ba4-w2kssgalv0.json": {
        "attack": {"attacker_slot": 0, "player": "b", "target": {"slot": 0}},
    },
}


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


@pytest.mark.parametrize("fixture,action_idx,bot_seed,served", CASES)
def test_tablet_swing_onext(
    db,
    fixture: str,
    action_idx: int,
    bot_seed: int,
    served: float,
) -> None:
    cap = _load_capture(fixture)
    game, _bot_side = _replay(db, cap, action_idx)

    base = game.bot_action_value(SERVED_SPEC, bot_seed)
    assert abs(base["value"] - served) < 5e-3, f"served mismatch: {base['value']} vs {served}"

    act_served = game.bot_action(SERVED_SPEC, bot_seed)
    assert act_served == MOVE_SERVED[fixture]

    on40 = game.bot_action_value(f"{SERVED_SPEC},onext=40", bot_seed)
    o1n = game.bot_action_value(f"{SERVED_SPEC},oopen=1,onext=40", bot_seed)
    assert on40["value"] <= base["value"] + 1e-5

    act40 = game.bot_action(f"{SERVED_SPEC},onext=40", bot_seed)
    print(
        f"{fixture}@{action_idx}: served={base['value']:.4f} "
        f"onext=40={on40['value']:.4f} oopen+onext={o1n['value']:.4f}"
    )
    tol = 5e-3
    ref_val = ONEXT40_VALUE[fixture]
    ref_o1 = ONEXT40_OOPEN1_VALUE[fixture]
    if abs(on40["value"] - ref_val) >= tol:
        pytest.fail(f"onext=40 expected {ref_val}, got {on40['value']}")
    if abs(o1n["value"] - ref_o1) >= tol:
        pytest.fail(f"oopen=1,onext=40 expected {ref_o1}, got {o1n['value']}")
    assert act40 == MOVE_ONEXT40[fixture]

    exp0 = game.bot_action_explain(SERVED_SPEC, bot_seed)
    exp40 = game.bot_action_explain(f"{SERVED_SPEC},onext=40", bot_seed)
    assert _world_ends(exp0).get("opp_next", 0) == 0
    assert _world_ends(exp40).get("opp_next", 0) >= 1
