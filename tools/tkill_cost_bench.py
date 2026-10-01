#!/usr/bin/env python3
"""Cost table for tkill: 16 meta decks, 2 games each, arena-bench --stats."""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BENCH = ROOT / "target/release/arena-bench"
POOLS = json.loads((ROOT / "oracle/decks/POOLS.json").read_text())["meta"]
_POOL_LIMIT = int(__import__("os").environ.get("TKILL_BENCH_DECKS", "0"))
if _POOL_LIMIT > 0:
    POOLS = POOLS[:_POOL_LIMIT]
BASES = ["h0", "h0:nodes=16000,horizon=3"]
TKILL = [0, 500, 2000, 10_000]


def p95(values: list[float]) -> float:
    if not values:
        return 0.0
    xs = sorted(values)
    i = min(len(xs) - 1, int(0.95 * len(xs) + 0.999999) - 1)
    return xs[max(0, i)]


def run_bench(spec: str) -> dict[str, float | list[float]]:
    total = {
        "decisions": 0,
        "ms": 0.0,
        "solver_calls": 0.0,
        "solver_found": 0.0,
        "solver_taken": 0.0,
        "solver_rejected": 0.0,
        "solver_unknown": 0.0,
        "solver_nodes": 0.0,
        "solver_nodes_max": 0.0,
        "ms_per_dec_samples": [],
        "solver_nodes_per_dec_samples": [],
    }
    for deck in POOLS:
        path = ROOT / f"oracle/decks/{deck}.json"
        out = subprocess.check_output(
            [
                str(BENCH),
                "--games",
                "2",
                "--seed",
                "20261001",
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

        def maxv(name: str) -> float:
            mm = re.search(rf"{name}=([0-9]+)", stats)
            return float(mm.group(1)) if mm else 0.0

        ms_per = data["seconds"] * 1000.0 / d
        nodes_per = per("own_solver_nodes")
        total["ms_per_dec_samples"].append(ms_per)
        total["solver_nodes_per_dec_samples"].append(nodes_per)
        total["decisions"] += d
        total["ms"] += data["seconds"] * 1000.0
        total["solver_calls"] += d * per("own_solver_calls")
        total["solver_found"] += d * per("own_solver_found")
        total["solver_taken"] += d * per("own_solver_taken")
        total["solver_rejected"] += d * per("own_solver_rejected")
        total["solver_unknown"] += d * per("own_solver_unknown")
        total["solver_nodes"] += d * nodes_per
        total["solver_nodes_max"] = max(total["solver_nodes_max"], maxv("own_solver_nodes_max"))
    return total


def main() -> None:
    if not BENCH.is_file():
        subprocess.check_call(
            ["cargo", "build", "-p", "arena-engine", "--release", "--bin", "arena-bench"],
            cwd=ROOT,
        )
    print(
        f"{'spec':52} {'decisions':>10} {'ms/dec':>10} {'p95_ms':>10} "
        f"{'solver_nodes/dec':>16} {'p95_nodes':>10} {'nodes_max':>10} "
        f"{'found/dec':>10} {'taken/dec':>10} {'rejected/dec':>12} {'unknown/dec':>12}"
    )
    for base in BASES:
        for tk in TKILL:
            if tk == 0 and ":" not in base:
                spec = base
            elif ":" in base:
                spec = f"{base},tkill={tk}"
            else:
                spec = f"{base}:tkill={tk}"
            total = run_bench(spec)
            d = int(total["decisions"])
            if d == 0:
                print(f"{spec:52} {'0':>10} {'—':>10} {'—':>10} {'—':>16} {'—':>10} {'—':>10} {'—':>10} {'—':>10} {'—':>12} {'—':>12}")
                continue
            ms_samples = total["ms_per_dec_samples"]
            node_samples = total["solver_nodes_per_dec_samples"]
            print(
                f"{spec:52} {d:10d} {total['ms']/d:10.2f} {p95(ms_samples):10.2f} "
                f"{total['solver_nodes']/d:16.2f} {p95(node_samples):10.2f} {total['solver_nodes_max']:10.0f} "
                f"{total['solver_found']/d:10.2f} {total['solver_taken']/d:10.2f} "
                f"{total['solver_rejected']/d:12.2f} {total['solver_unknown']/d:12.2f}"
            )


if __name__ == "__main__":
    main()
