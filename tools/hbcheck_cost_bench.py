#!/usr/bin/env python3
"""Paired hbcheck cost table: holdback1 166-moment replay positions.

Runs the ignored `hbcheck_paired_cost_report` integration test (same
moments, same seeds, interleaved specs). Build release first; full table
is ~10–20 min locally depending on machine.
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    cmd = [
        "cargo",
        "test",
        "-p",
        "arena-engine",
        "--release",
        "hbcheck_paired_cost_report",
        "--",
        "--ignored",
        "--nocapture",
    ]
    print("Running paired hbcheck bench via:", " ".join(cmd), file=sys.stderr)
    return subprocess.call(cmd, cwd=ROOT)


if __name__ == "__main__":
    raise SystemExit(main())
