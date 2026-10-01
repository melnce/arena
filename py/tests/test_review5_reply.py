"""Review5 real positions (origin/results @ e60c02a) for okill / omacro reply model."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
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

FIXTURES = Path(__file__).resolve().parent / "fixtures" / "review5"
GAME_1537 = "15374915787985923294-5ce21003.json"
GAME_1043 = "10436817046835498199-5ce21003.json"
GAME_6889 = "6889336734586405565-5ce21003.json"
GAME_2790 = "2790993798946146787-5ce21003.json"
GAME_4611 = "4611759015070762461-5ce21003.json"
# Pinned from origin/results:review5/games/ @ e60c02a.
FIXTURE_SHA256 = {
    GAME_1537: "01ad252256e377d0a22c9c0a15b177762610b419d326cc757392d693b3a1e991",
    GAME_1043: "a5d29d2ebc0798ee95bc6c37b2043dfd5e4073eb3b40c2c50a9dbf58944e5dd0",
    GAME_6889: "21ab70a06f9e4f3eabe9f2993c45f1d17ec371f3a3091cb85b91467d5ed31db2",
    GAME_2790: "8bf8f521f251efb6cea96d0ffdf00084a3917bf79d3da3fe17826717203d5058",
    GAME_4611: "edafdcc590f6fdf987098e07846d7efcf6066035995a59d993198a9fa1285da9",
}
BASE_SPEC = "h0:nodes=16000,horizon=3,info=all"


def _load_capture(name: str) -> dict[str, Any]:
    return json.loads((FIXTURES / name).read_text())


def _replay(db, cap: dict[str, Any], n: int):
    import arena

    game = arena.Game(
        db,
        cap["seed"],
        cap["deckA"],
        cap["deckB"],
        cap.get("first") or "coin",
    )
    for step in cap["actions"][:n]:
        audit.apply_step(game, step)
    return game


def _explain_seed(cap: dict[str, Any], ply: int) -> int:
    """Match review5/tools/botview3.py: ``rec['seed'] + action_index``."""
    return int(cap["seed"]) + ply


def _world_ends(worlds: list[dict[str, Any]]) -> dict[str, int]:
    out: dict[str, int] = {}
    for w in worlds:
        end = w.get("end") or "none"
        out[str(end)] = out.get(str(end), 0) + 1
    return out


def _end_turn_candidate(exp: dict[str, Any]) -> dict[str, Any]:
    for cand in exp["candidates"]:
        if "end_turn" in cand["action"]:
            return cand
    raise AssertionError("no EndTurn candidate in explain record")


def _opp_lethal_worlds(cand: dict[str, Any]) -> int:
    return _world_ends(cand["worlds"]).get("opp_lethal", 0)


def test_review5_fixtures_match_pinned_hashes() -> None:
    for name, expected in FIXTURE_SHA256.items():
        data = (FIXTURES / name).read_bytes()
        assert hashlib.sha256(data).hexdigest() == expected, name


def test_1537_75_okill_ward_break(db) -> None:
    """Ward-break prefix: Baal/Lt/Bat + evolve clears wards, then Garodeth."""
    cap = _load_capture(GAME_1537)
    game = _replay(db, cap, 75)
    seed = _explain_seed(cap, 75)

    off = game.bot_action_explain(BASE_SPEC, seed)
    on = game.bot_action_explain(f"{BASE_SPEC},okill=1", seed)
    cand_off = _end_turn_candidate(off)
    cand_on = _end_turn_candidate(on)

    assert _opp_lethal_worlds(cand_off) == 0
    assert _opp_lethal_worlds(cand_on) >= 3


def test_1043_109_okill_slot_free(db) -> None:
    """Slot-freeing: face chip, double trade, then Garodeth on a full board."""
    cap = _load_capture(GAME_1043)
    game = _replay(db, cap, 109)
    seed = _explain_seed(cap, 109)

    off = game.bot_action_explain(BASE_SPEC, seed)
    on = game.bot_action_explain(f"{BASE_SPEC},okill=2", seed)
    cand_off = _end_turn_candidate(off)
    cand_on = _end_turn_candidate(on)

    assert _opp_lethal_worlds(cand_off) == 0
    assert _opp_lethal_worlds(cand_on) >= 3


def test_6889_45_okill_play_through(db) -> None:
    """Play-through: super-evolve Adahime ping completes 12 vs 12."""
    cap = _load_capture(GAME_6889)
    game = _replay(db, cap, 45)
    seed = _explain_seed(cap, 45)

    off = game.bot_action_explain(BASE_SPEC, seed)
    on = game.bot_action_explain(f"{BASE_SPEC},okill=4", seed)
    cand_off = _end_turn_candidate(off)
    cand_on = _end_turn_candidate(on)

    assert _opp_lethal_worlds(cand_off) == 0
    assert _opp_lethal_worlds(cand_on) >= 3


@pytest.mark.skipif(
    os.environ.get("ARENA_REPORTS") != "1",
    reason="set ARENA_REPORTS=1 to run slow report-only review5 explain output",
)
def test_2790_83_okill_report(db) -> None:
    """Report: Hark random split — opp_lethal counts with okill=0 vs okill=7."""
    cap = _load_capture(GAME_2790)
    game = _replay(db, cap, 83)
    seed = _explain_seed(cap, 83)

    lines = ["review5 2790 before [83] — Ward break + Storm + Hark", ""]
    for okill in (0, 7):
        exp = game.bot_action_explain(
            BASE_SPEC if okill == 0 else f"{BASE_SPEC},okill={okill}", seed
        )
        cand_et = _end_turn_candidate(exp)
        lines.append(
            f"okill={okill} end-turn: ends={_world_ends(cand_et['worlds'])} "
            f"agg={cand_et['root_agg']:+.2f}"
        )
        mac = next(
            (c for c in exp["candidates"] if c["action"].get("play", {}).get("card") == "10754120"),
            None,
        )
        if mac is not None:
            lines.append(
                f"okill={okill} macmillan: ends={_world_ends(mac['worlds'])} "
                f"agg={mac['root_agg']:+.2f}"
            )
    print("\n".join(lines))


@pytest.mark.skipif(
    os.environ.get("ARENA_REPORTS") != "1",
    reason="set ARENA_REPORTS=1 to run slow report-only review5 explain output",
)
def test_4611_48_omacro_report(db) -> None:
    """Report: greedy reply should prefer Adahime + evolve with omacro=1."""
    cap = _load_capture(GAME_4611)
    game = _replay(db, cap, 48)
    seed = _explain_seed(cap, 48)

    lines = ["review5 4611 before [48] — Adahime + SE reply", ""]
    for omacro in (0, 1):
        spec = BASE_SPEC if omacro == 0 else f"{BASE_SPEC},omacro=1"
        exp = game.bot_action_explain(spec, seed)
        chosen = exp["candidates"][exp["chosen_index"]]
        lines.append(
            f"omacro={omacro}: pick idx={exp['chosen_index']} "
            f"value={exp['value']:+.2f} action={chosen['action']}"
        )
        for cand in exp["candidates"]:
            if "play" not in cand["action"]:
                continue
            lines.append(
                f"  cand{cand['legal_index']} agg={cand['root_agg']:+.2f} "
                f"ends={_world_ends(cand['worlds'])}"
            )
    print("\n".join(lines))


@pytest.mark.skipif(
    os.environ.get("ARENA_REPORTS") != "1",
    reason="set ARENA_REPORTS=1 to run slow report-only review5 explain output",
)
def test_6889_45_omacro_report(db) -> None:
    """Report: modelled reply on Adahime line with and without omacro."""
    cap = _load_capture(GAME_6889)
    game = _replay(db, cap, 45)
    seed = _explain_seed(cap, 45)

    lines = ["review5 6889 before [45] — omacro on Adahime line", ""]
    for omacro in (0, 1):
        spec = BASE_SPEC if omacro == 0 else f"{BASE_SPEC},omacro=1"
        exp = game.bot_action_explain(spec, seed)
        lines.append(
            f"omacro={omacro}: pick={exp['chosen_index']} value={exp['value']:+.2f}"
        )
        for cand in exp["candidates"]:
            lines.append(
                f"  cand{cand['legal_index']} {cand['action']} "
                f"agg={cand['root_agg']:+.2f} ends={_world_ends(cand['worlds'])}"
            )
    print("\n".join(lines))
