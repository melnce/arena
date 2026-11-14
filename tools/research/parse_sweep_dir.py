#!/usr/bin/env python3
"""Print pooled rates from a local sweep directory."""

from __future__ import annotations

import json
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
if str(_REPO / "py") not in sys.path:
    sys.path.insert(0, str(_REPO / "py"))

from runlib import pooled_candidate, rate_ci  # noqa: E402


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: parse_sweep_dir.py DIR", file=sys.stderr)
        return 2
    d = Path(sys.argv[1])
    cands = json.loads((d / "candidates.json").read_text())
    for c in cands:
        idx = c["index"]
        stem = f"c{idx:02d}"
        mf = d / f"{stem}-final.json"
        mr = d / f"{stem}-reverse.json"
        if not mf.is_file() or not mr.is_file():
            print(f"{stem}: incomplete")
            continue
        main = json.loads(mf.read_text())
        rev = json.loads(mr.read_text())
        p, ci = pooled_candidate(main, rev, tag=stem)
        wall = main["summary"].get("seconds", 0) + rev["summary"].get("seconds", 0)
        elo = 400 * __import__("math").log10(p / (1 - p)) if 0 < p < 1 else 0
        print(f"{stem} {c['spec'].split(',')[-1]}: pooled {rate_ci(p, ci)} Elo {elo:+.1f} wall {wall:.0f}s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
