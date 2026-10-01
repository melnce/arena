#!/usr/bin/env python3
"""Paired tkill cost table: h0 self-play positions on 16 meta mirrors.

Runs the ignored `tkill_paired_cost_report` integration test (same positions,
same seeds, interleaved specs). Build release first; full table is ~15–25 min
locally depending on position count.
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
        "tkill_paired_cost_report",
        "--",
        "--ignored",
        "--nocapture",
    ]
    print("Running paired bench via:", " ".join(cmd), file=sys.stderr)
    return subprocess.call(cmd, cwd=ROOT)


if __name__ == "__main__":
    raise SystemExit(main())
