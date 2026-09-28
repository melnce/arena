#!/usr/bin/env python3
"""Cost table for horizon: 16 meta decks, 2 games each, arena-bench --stats."""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BENCH = ROOT / "target/release/arena-bench"
POOLS = json.loads((ROOT / "oracle/decks/POOLS.json").read_text())["meta"]
BASES = ["h0", "h0:nodes=16000", "h0:nodes=16000,beam=2"]
HORIZONS = [0, 1, 2, 3]


def run_bench(spec: str) -> dict[str, float]:
    total = {
        "decisions": 0,
        "nodes": 0.0,
        "horizon_nodes": 0.0,
        "horizon_leaves": 0.0,
        "horizon_fallback": 0.0,
        "skipped_worlds": 0.0,
        "ms": 0.0,
    }
    for deck in POOLS:
        path = ROOT / f"oracle/decks/{deck}.json"
        out = subprocess.check_output(
            [
                str(BENCH),
                "--games",
                "2",
                "--seed",
                "20260927",
                "--deck-a",
                str(path),
                "--deck-b",
                str(path),
                "--policy",
                spec,
                "--vs",
                "random",
                "--stats",
            ],
            text=True,
            stderr=subprocess.DEVNULL,
            cwd=ROOT,
        ).strip().splitlines()
        data = json.loads(out[0])
        stats = next(
            (line for line in out if line.startswith("search-stats A ")),
            "",
        )
        m = re.search(r"decisions=(\d+)", stats)
        if not m:
            continue
        d = int(m.group(1))
        if d == 0:
            continue

        def per(name: str) -> float:
            mm = re.search(rf"{name}/decision=([0-9.]+)", stats)
            return float(mm.group(1)) if mm else 0.0

        total["decisions"] += d
        total["nodes"] += d * per("nodes")
        total["horizon_nodes"] += d * per("horizon_nodes")
        total["horizon_leaves"] += d * per("horizon_leaves")
        total["horizon_fallback"] += d * per("horizon_fallback")
        total["skipped_worlds"] += d * per("skipped_worlds")
        total["ms"] += data["seconds"] * 1000.0
    return total


def main() -> None:
    if not BENCH.is_file():
        subprocess.check_call(
            ["cargo", "build", "-p", "arena-engine", "--release", "--bin", "arena-bench"],
            cwd=ROOT,
        )
    print(
        f"{'spec':42} {'decisions':>10} {'nodes/dec':>14} {'horizon_nodes/dec':>16} "
        f"{'horizon_leaves/dec':>18} {'horizon_fallback/dec':>18} "
        f"{'skipped_worlds/dec':>18} {'ms/dec':>12}"
    )
    for base in BASES:
        for hz in HORIZONS:
            if hz == 0:
                spec = base
            elif ":" in base:
                spec = f"{base},horizon={hz}"
            else:
                spec = f"{base}:horizon={hz}"
            total = run_bench(spec)
            d = total["decisions"]
            if d == 0:
                print(
                    f"{spec:42} {'0':>10} {'—':>14} {'—':>16} {'—':>18} "
                    f"{'—':>18} {'—':>18} {'—':>12}"
                )
                continue
            print(
                f"{spec:42} {d:10d} {total['nodes']/d:14.2f} "
                f"{total['horizon_nodes']/d:16.2f} {total['horizon_leaves']/d:18.2f} "
                f"{total['horizon_fallback']/d:18.2f} "
                f"{total['skipped_worlds']/d:18.2f} {total['ms']/d:12.2f}"
            )


if __name__ == "__main__":
    main()
