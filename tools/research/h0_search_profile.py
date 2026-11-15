#!/usr/bin/env python3
"""Wall-time and explain aggregates for REVIEW_MOMENTS + meta self-play scan."""

from __future__ import annotations

import importlib.util
import json
import statistics
import sys
import time
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parents[2]
if str(_HERE / "py") not in sys.path:
    sys.path.insert(0, str(_HERE / "py"))

import arena  # noqa: E402

_PY = _HERE / "py"
_spec = importlib.util.spec_from_file_location("arena_lethal_audit", _PY / "lethal_audit.py")
assert _spec and _spec.loader
_mod = importlib.util.module_from_spec(_spec)
sys.modules["arena_lethal_audit"] = _mod
_spec.loader.exec_module(_mod)
audit = _mod

LEAF87Q = Path("/tmp/leaf87q.json")
SWEEP = (
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,"
    "fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,"
    "order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,"
    f"net={LEAF87Q},mull=leaf,mulls=256"
)
SERVED = (
    "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,"
    "fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,"
    "order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,"
    "net=h0-value-v87,mull=leaf,mulls=256"
)
QUARTER_K4 = SWEEP.replace("nodes=32000", "nodes=8000").replace("k=8", "k=4").replace(
    f"net={LEAF87Q}", "net=h0-value-v87"
)
QUARTER_K8 = SWEEP.replace("nodes=32000", "nodes=8000").replace(f"net={LEAF87Q}", "net=h0-value-v87")

REVIEW_MOMENTS = [
    ("review13", "4984932781931433298-b4973563.json", 27),
    ("review13", "4984932781931433298-b4973563.json", 45),
    ("review14", "14600367900189587136-d90f8eb2.json", 69),
    ("review14", "14600367900189587136-d90f8eb2.json", 40),
    ("review14", "6598261483642061665-d90f8eb2.json", 27),
    ("review14", "6598261483642061665-d90f8eb2.json", 52),
    ("review14", "6598261483642061665-d90f8eb2.json", 91),
    ("review14", "12644260486301391828-d90f8eb2.json", 15),
    ("review14", "12644260486301391828-d90f8eb2.json", 35),
    ("review14", "6598261483642061665-d90f8eb2.json", 60),
    ("review14", "14600367900189587136-d90f8eb2.json", 55),
    ("review13", "4984932781931433298-b4973563.json", 60),
]


def _load_capture(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def _bot_explain_seed(cap: dict[str, Any], ply: int) -> int:
    actions = cap.get("actions") or []
    if any(
        s.get("mulligan") is not None or s.get("bot_value") is not None
        for s in actions[:ply]
    ):
        base = int(cap["seed"])
        n = sum(
            1
            for s in actions[:ply]
            if s.get("mulligan") is not None or s.get("bot_value") is not None
        )
        return base + n
    return int(cap["seed"]) + ply


def _replay(db, cap: dict[str, Any], n: int) -> arena.Game:
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


def leaf_stats(explain: dict) -> dict[str, float]:
    stats = explain.get("stats") or {}
    pv_ends: dict[str, int] = {}
    world_spreads: list[float] = []
    margins: list[float] = []
    for cand in explain.get("candidates") or []:
        raw = [w.get("raw") for w in cand.get("worlds") or [] if not w.get("skipped")]
        if len(raw) >= 2:
            world_spreads.append(max(raw) - min(raw))
        for w in cand.get("worlds") or []:
            end = w.get("end")
            if end:
                pv_ends[end] = pv_ends.get(end, 0) + 1
    aggs = sorted(
        [c.get("root_agg") for c in explain.get("candidates") or [] if c.get("n")],
        reverse=True,
    )
    if len(aggs) >= 2:
        margins.append(aggs[0] - aggs[1])
    return {
        "nodes": float(explain.get("nodes") or 0),
        "k": float(explain.get("k") or 0),
        "candidates": float(len(explain.get("candidates") or [])),
        "spread_med": statistics.median(world_spreads) if world_spreads else 0.0,
        "margin_med": statistics.median(margins) if margins else 0.0,
        "opp_reply_ends": sum(pv_ends.get(k, 0) for k in ("opp_reply", "opp_open", "opp_next")),
        "own_search_ends": sum(pv_ends.get(k, 0) for k in ("depth", "cap", "depth_reply", "cap_reply")),
        "nodes_lethal": float(explain.get("nodes_lethal") or 0),
        "horizon_nodes": float(explain.get("horizon_nodes") or 0),
        "stats_nodes": float(stats.get("nodes", 0)),
        "stats_opp_lethal_nodes": float(stats.get("opp_lethal_nodes", 0)),
        "stats_opp_open_nodes": float(stats.get("opp_open_nodes", 0)),
        "stats_opp_next_nodes": float(stats.get("opp_next_nodes", 0)),
        "stats_horizon_nodes": float(stats.get("horizon_nodes", 0)),
    }


def profile_spec(spec: str, moments: list[tuple[Path, int]]) -> dict:
    db = arena.default_db()
    times: list[float] = []
    rows: list[dict[str, float]] = []
    for path, ply in moments:
        cap = _load_capture(path)
        game = _replay(db, cap, ply)
        seed = _bot_explain_seed(cap, ply)
        t0 = time.perf_counter()
        exp = game.bot_action_explain(spec, seed)
        dt = time.perf_counter() - t0
        times.append(dt)
        rows.append(leaf_stats(exp))
    agg = {k: statistics.mean(r[k] for r in rows) for k in rows[0]}
    return {
        "spec": spec,
        "wall_ms_per_decision": statistics.mean(times) * 1000,
        "n": len(moments),
        **agg,
    }


def main() -> int:
    fixtures = _HERE / "py/tests/fixtures"
    review_paths = [
        (fixtures / review / "games" / game, ply) for review, game, ply in REVIEW_MOMENTS
    ]
    if not LEAF87Q.is_file():
        print(f"missing leaf: {LEAF87Q}", file=sys.stderr)
        return 2
    out = {
        "sweep_scale": profile_spec(SWEEP, review_paths),
        "served": profile_spec(SERVED, review_paths),
        "quarter_k4": profile_spec(QUARTER_K4, review_paths),
        "quarter_k8": profile_spec(QUARTER_K8, review_paths),
    }
    art = Path("/opt/cursor/artifacts/h0_search_profile.json")
    art.parent.mkdir(parents=True, exist_ok=True)
    art.write_text(json.dumps(out, indent=2) + "\n")
    print(json.dumps(out, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())