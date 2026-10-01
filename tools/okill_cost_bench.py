#!/usr/bin/env python3
"""Cost table for okill/omacro: 4 meta decks, 2 games each, arena-bench --stats."""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BENCH = ROOT / "target/release/arena-bench"
DECKS = [
    "meta-abyss-midrange",
    "meta-haven-amulet",
    "meta-dragon-ramp",
    "meta-sword-rally",
]
SPECS = [
    "h0",
    "h0:okill=7",
    "h0:omacro=1",
    "h0:okill=7,omacro=1",
]
SEED = "42"
GAMES = "2"


def run_bench(spec: str, deck: str) -> dict[str, float]:
    path = ROOT / f"oracle/decks/{deck}.json"
    out = subprocess.check_output(
        [
            str(BENCH),
            "--games",
            GAMES,
            "--seed",
            SEED,
            "--deck-a",
            str(path),
            "--deck-b",
            str(path),
            "--policy",
            spec,
            "--vs",
            spec,
            "--stats",
        ],
        text=True,
        stderr=subprocess.DEVNULL,
        cwd=ROOT,
    ).strip().splitlines()
    data = json.loads(out[0])
    stats = next((line for line in out if line.startswith("search-stats A ")), "")
    m = re.search(r"decisions=(\d+)", stats)
    if not m:
        return {"decisions": 0, "ms": 0.0}
    d = int(m.group(1))

    def per(name: str) -> float:
        mm = re.search(rf"{name}/decision=([0-9.]+)", stats)
        return float(mm.group(1)) if mm else 0.0

    checks = per("opp_lethal_checks")
    opp_nodes = per("opp_lethal_nodes")
    applies_mean = opp_nodes / checks if checks > 0 else 0.0
    mm = re.search(r"opp_lethal_applies_max=(\d+)", stats)
    applies_max = int(mm.group(1)) if mm else 0
    return {
        "decisions": d,
        "nodes": d * per("nodes"),
        "ms": data["seconds"] * 1000.0,
        "cap_hit_rate": per("cap_hit_rate"),
        "opp_lethal_nodes": d * opp_nodes,
        "applies_mean": applies_mean,
        "applies_max": applies_max,
        "ward": per("opp_lethal_ward_found"),
        "slot": per("opp_lethal_slot_found"),
        "through": per("opp_lethal_through_found"),
    }


def main() -> None:
    if not BENCH.is_file():
        subprocess.check_call(
            ["cargo", "build", "-p", "arena-engine", "--release", "--bin", "arena-bench"],
            cwd=ROOT,
        )
    print(
        f"{'spec':28} {'dec/dec':>8} {'nodes/dec':>10} {'ms/dec':>8} "
        f"{'cap_hit':>8} {'ol_nodes':>9} {'appl μ':>7} {'appl max':>8} "
        f"{'ward':>6} {'slot':>6} {'thru':>6}"
    )
    for spec in SPECS:
        total = {
            "decisions": 0,
            "nodes": 0.0,
            "ms": 0.0,
            "cap_hit_rate": 0.0,
            "opp_lethal_nodes": 0.0,
            "applies_mean": 0.0,
            "applies_max": 0,
            "ward": 0.0,
            "slot": 0.0,
            "through": 0.0,
        }
        for deck in DECKS:
            row = run_bench(spec, deck)
            total["decisions"] += row["decisions"]
            total["nodes"] += row["nodes"]
            total["ms"] += row["ms"]
            total["cap_hit_rate"] += row["decisions"] * row["cap_hit_rate"]
            total["opp_lethal_nodes"] += row["opp_lethal_nodes"]
            total["applies_mean"] += row["decisions"] * row["applies_mean"]
            total["applies_max"] = max(total["applies_max"], row["applies_max"])
            total["ward"] += row["decisions"] * row["ward"]
            total["slot"] += row["decisions"] * row["slot"]
            total["through"] += row["decisions"] * row["through"]
        d = total["decisions"]
        if d == 0:
            print(f"{spec:28} {'—':>8}")
            continue
        print(
            f"{spec:28} {d:8d} {total['nodes']/d:10.1f} {total['ms']/d:8.1f} "
            f"{total['cap_hit_rate']/d:8.4f} {total['opp_lethal_nodes']/d:9.1f} "
            f"{total['applies_mean']/d:7.2f} {total['applies_max']:8d} "
            f"{total['ward']/d:6.3f} {total['slot']/d:6.3f} {total['through']/d:6.3f}"
        )


if __name__ == "__main__":
    main()
