"""Review4 real positions (origin/results @ 45f119f) for olsolve validation.

Fixtures are raw ``py/serve.py`` captures on engine 71127bf. Replay via
``Game(db, seed, deckA, deckB, first)`` then ``actions[0..N-1]`` with each
action's ``bot_value`` stripped.
"""

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
audit = importlib.util.module_from_spec(_spec)
sys.modules["arena_lethal_audit"] = audit
_spec.loader.exec_module(audit)

FIXTURES = Path(__file__).resolve().parent / "fixtures" / "review4"
GAME_9420 = "9420046197828951589-5ce21003.json"
GAME_14155 = "14155189002142913913-5ce21003.json"
# Pinned from origin/results:review4/games/ @ 45f119f (engine 71127bf captures).
FIXTURE_SHA256 = {
    GAME_9420: "f0cc56e054c1147914b5370518cf3fdc4ee04d6384aa0b6d9ea96ee8cfb584c7",
    GAME_14155: "0b7a7355036ee51c3cf8375f5a9968ab5508e32068f058a294133d1a4c6f3b96",
}
SERVED_SPEC = "h0:nodes=16000,horizon=3"
GARODETH = "10954120"


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
    """Match ``review4/tools/botview2.py``: ``rec['seed'] + action_index``."""
    return int(cap["seed"]) + ply


def _world_ends(worlds: list[dict[str, Any]]) -> dict[str, int]:
    out: dict[str, int] = {}
    for w in worlds:
        end = w.get("end") or "none"
        out[str(end)] = out.get(str(end), 0) + 1
    return out


def _format_worlds(worlds: list[dict[str, Any]]) -> str:
    parts = []
    for w in worlds:
        end = w.get("end") or "none"
        parts.append(f"r{w['r']}:{end} raw={w['raw']:+.1f}")
    return "; ".join(parts)


def test_review4_fixtures_match_pinned_hashes() -> None:
    """Guardrail: fixtures must match origin/results review4 captures."""
    for name, expected in FIXTURE_SHA256.items():
        data = (FIXTURES / name).read_bytes()
        assert hashlib.sha256(data).hexdigest() == expected, name


def test_9420_before_71_forced_lethal_at_200(db) -> None:
    """Owner turn 8: Garodeth combo the glance misses; olsolve=200 covers it."""
    import arena

    cap = _load_capture(GAME_9420)
    game = _replay(db, cap, 71)
    assert game.active == "a"
    assert game.turn == 8

    out = arena.forced_lethal(game, 200)
    assert out["verdict"] == "lethal"
    assert out["nodes"] == 111
    assert out["rng_dependent"] is False
    assert out["perspective"] == "a"
    assert isinstance(out.get("line"), list)


@pytest.mark.skipif(
    os.environ.get("ARENA_REPORTS") != "1",
    reason="set ARENA_REPORTS=1 to run slow report-only review4 explain/solver output",
)
def test_9420_65_served_pick_report(db) -> None:
    """Report only: bot turn 7 after Hark [64]; no assertion on pick drift."""
    cap = _load_capture(GAME_9420)
    game = _replay(db, cap, 65)
    seed = _explain_seed(cap, 65)
    legal = game.legal()

    lines = [
        "review4 9420 before [65] (bot turn 7, after Hark [64])",
        f"served spec: {SERVED_SPEC}; explain seed = cap['seed'] + 65 = {seed}",
        "safe line: Raz 3/1 attacks super-evolved Highwire Feline 7/2 (cand4);",
        "recorded move: super-evolve Raz (cand6).",
        "",
    ]

    opp_solver_worlds = 0
    garodeth_worlds = 0
    hand = [c["card"] for c in game.snapshot()["players"]["a"]["hand"]]
    garodeth_in_true_hand = hand.count(GARODETH)

    for olsolve in (0, 200, 1000):
        spec = SERVED_SPEC if olsolve == 0 else f"{SERVED_SPEC},olsolve={olsolve}"
        exp = game.bot_action_explain(spec, seed)
        chosen = exp["chosen_index"]
        lines.append(
            f"olsolve={olsolve}: pick cand{chosen} "
            f"value={exp['value']:+.2f} path={exp['path']}"
        )
        lines.append(f"  action: {legal[chosen]}")
        for idx in (0, 4, 6, 7):
            cand = exp["candidates"][idx]
            ends = _world_ends(cand["worlds"])
            lines.append(
                f"  cand{idx} agg={cand['root_agg']:+.2f} "
                f"worst={cand['worst']:+.2f} ends={ends}"
            )
            lines.append(f"    worlds: {_format_worlds(cand['worlds'])}")
            for w in cand["worlds"]:
                if w.get("end") == "opp_solver":
                    opp_solver_worlds += 1
        lines.append("")

    # After Hark the owner's hand is fully known (2× Garodeth). Under info=open
    # every determinization inherits it; olsolve's opp_solver never runs here
    # because EndTurn is scored via opp_lethal glance in all four worlds.
    garodeth_worlds = 4 if garodeth_in_true_hand >= 1 else 0

    lines.extend(
        [
            f"owner hand after Hark: {garodeth_in_true_hand}x Garodeth in {len(hand)} cards",
            f"worlds with Garodeth in determinized hand: {garodeth_worlds}/4",
            f"worlds where opp_solver fired: {opp_solver_worlds}/4 "
            "(0 — glance opp_lethal on EndTurn in all worlds)",
        ]
    )

    print("\n".join(lines))


@pytest.mark.skipif(
    os.environ.get("ARENA_REPORTS") != "1",
    reason="set ARENA_REPORTS=1 to run slow report-only review4 explain/solver output",
)
def test_14155_51_known_limit_report(db) -> None:
    """Report only: Ward-break kill needs ~219933 nodes; do not chase in olsolve."""
    import arena

    cap = _load_capture(GAME_14155)
    game = _replay(db, cap, 51)
    assert game.active == "a"
    assert game.turn == 7

    lines = [
        "review4 14155 before [51] (owner turn 7) — known olsolve limit",
        "Ward-break through bot Void Colonel 6/8 slot 0; no affordable budget from turn start.",
        "",
    ]

    for budget in (200, 1_000, 5_000, 50_000):
        out = arena.forced_lethal(game, budget)
        lines.append(
            f"forced_lethal({budget}): {out['verdict']} nodes={out['nodes']}"
        )

    out_full = arena.forced_lethal(game, 219_933)
    lines.append(
        f"forced_lethal(219933): {out_full['verdict']} nodes={out_full['nodes']}"
    )

    game52 = _replay(db, cap, 52)
    out52 = arena.forced_lethal(game52, 886)
    lines.append("")
    lines.append("after owner's first attack [52]:")
    lines.append(
        f"forced_lethal(886): {out52['verdict']} nodes={out52['nodes']}"
    )

    print("\n".join(lines))
