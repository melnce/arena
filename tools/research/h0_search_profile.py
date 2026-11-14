#!/usr/bin/env python3
"""Wall-time and explain aggregates for REVIEW_MOMENTS + meta self-play scan."""

from __future__ import annotations

import json
import statistics
import sys
import time
from pathlib import Path

_HERE = Path(__file__).resolve().parents[2]
if str(_HERE / "py") not in sys.path:
    sys.path.insert(0, str(_HERE / "py"))

import arena  # noqa: E402

SWEEP = (
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,"
    "fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,"
    "order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,"
    "net=/tmp/leaf87q.json,mull=leaf,mulls=256"
)
SERVED = (
    "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,"
    "fuseguard=1,okill=15,hbk=16,alloc=world,threads=8,lostrank=4000,"
    "order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,"
    "net=/tmp/leaf87q.json,mull=leaf,mulls=256"
)

REVIEW = [
    ("review13", "4984932781931433298-b4973563.json", 27),
    ("review14", "14600367900189587136-d90f8eb2.json", 69),
]


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


def profile_spec(spec: str, labels: list[tuple[str, int]]) -> dict:
    g = arena.Game()
    db = arena.default_db()
    times: list[float] = []
    rows: list[dict[str, float]] = []
    for path, ply in labels:
        cap = json.loads(Path(path).read_text())
        seed = cap["seed"] + ply
        t0 = time.perf_counter()
        exp = g.bot_action_explain(spec, seed)
        dt = time.perf_counter() - t0
        times.append(dt)
        rows.append(leaf_stats(exp))
    agg = {k: statistics.mean(r[k] for r in rows) for k in rows[0]}
    return {
        "spec": spec,
        "wall_ms_per_decision": statistics.mean(times) * 1000,
        "n": len(labels),
        **agg,
    }


def main() -> int:
    fixtures = _HERE / "py/tests/fixtures"
    review_paths = [
        (fixtures / rev / "games" / game, ply)
        for rev, game, ply in REVIEW
    ]
    # meta scan: reuse arena helper if available
    out = {
        "sweep_scale": profile_spec(SWEEP, [(str(p), ply) for p, ply in review_paths]),
        "served": profile_spec(SERVED, [(str(p), ply) for p, ply in review_paths]),
    }
    art = Path("/opt/cursor/artifacts/h0_search_profile.json")
    art.parent.mkdir(parents=True, exist_ok=True)
    art.write_text(json.dumps(out, indent=2) + "\n")
    print(json.dumps(out, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
