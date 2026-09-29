#!/usr/bin/env python3
"""Cost table for olsolve: 16 meta decks, 2 games each, arena-bench --stats."""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BENCH = ROOT / "target/release/arena-bench"
POOLS = json.loads((ROOT / "oracle/decks/POOLS.json").read_text())["meta"]
# Full meta pool for sweep sizing; override with OLSOLVE_BENCH_DECKS=N.
_POOL_LIMIT = int(__import__("os").environ.get("OLSOLVE_BENCH_DECKS", "0"))
if _POOL_LIMIT > 0:
    POOLS = POOLS[:_POOL_LIMIT]
BASES = ["h0", "h0:nodes=16000,horizon=3"]
OLSOLVE = [0, 200, 1000]


def run_bench(spec: str) -> dict[str, float]:
    total = {
        "decisions": 0,
        "nodes": 0.0,
        "solver_calls": 0.0,
        "solver_found": 0.0,
        "solver_unknown": 0.0,
        "solver_nodes": 0.0,
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
                "20260929",
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
        total["solver_calls"] += d * per("opp_solver_calls")
        total["solver_found"] += d * per("opp_solver_found")
        total["solver_unknown"] += d * per("opp_solver_unknown")
        total["solver_nodes"] += d * per("opp_solver_nodes")
        total["ms"] += data["seconds"] * 1000.0
    return total


def main() -> None:
    if not BENCH.is_file():
        subprocess.check_call(
            ["cargo", "build", "-p", "arena-engine", "--release", "--bin", "arena-bench"],
            cwd=ROOT,
        )
    print(
        f"{'spec':52} {'decisions':>10} {'ms/dec':>10} {'nodes/dec':>12} "
        f"{'solver_calls/dec':>16} {'solver_found/dec':>16} "
        f"{'solver_unknown/dec':>18} {'solver_nodes/dec':>16} "
        f"{'solver_share':>12}"
    )
    for base in BASES:
        allowed = OLSOLVE if "horizon=3" in base else [0, 200]
        for ol in allowed:
            if ol == 0 and ":" not in base:
                spec = base
            elif ":" in base:
                spec = f"{base},olsolve={ol}"
            else:
                spec = f"{base}:olsolve={ol}"
            total = run_bench(spec)
            d = total["decisions"]
            if d == 0:
                print(f"{spec:52} {'0':>10} {'—':>10} {'—':>12} {'—':>16} {'—':>16} {'—':>18} {'—':>16} {'—':>12}")
                continue
            nodes = total["nodes"] / d
            solver_nodes = total["solver_nodes"] / d
            share = 100.0 * solver_nodes / nodes if nodes > 0 else 0.0
            print(
                f"{spec:52} {d:10d} {total['ms']/d:10.2f} {nodes:12.2f} "
                f"{total['solver_calls']/d:16.2f} {total['solver_found']/d:16.2f} "
                f"{total['solver_unknown']/d:18.2f} {solver_nodes:16.2f} {share:11.1f}%"
            )


if __name__ == "__main__":
    main()
