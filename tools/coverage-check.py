#!/usr/bin/env python3
"""Assert coverage.md ops/conditions match a coarse verb walk of pool texts."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from coverage_lib import cell_has, parse_coverage_table, required_conditions, required_ops

ROOT = Path(__file__).resolve().parents[1]
COVERAGE = ROOT / "docs" / "coverage.md"
TEXTS = ROOT / "tools" / "pool-texts.json"


def main() -> int:
    rows = parse_coverage_table(COVERAGE.read_text())
    texts = json.loads(TEXTS.read_text())
    errors: list[str] = []
    if len(rows) != 615:
        errors.append(f"expected 615 coverage rows, got {len(rows)}")
    for row in rows:
        cid = row["id"]
        rec = texts.get(cid)
        if not rec:
            errors.append(f"{cid}: missing from pool-texts.json")
            continue
        text = rec.get("text") or ""
        for op in required_ops(text):
            if not cell_has(row["ops"], op):
                errors.append(f"{cid} {rec.get('name')}: text needs op {op}; ops column is {row['ops']!r}")
        for cond in required_conditions(text):
            if not cell_has(row["conditions"], cond):
                errors.append(
                    f"{cid} {rec.get('name')}: text needs condition {cond}; conditions column is {row['conditions']!r}"
                )
        if row["status"].startswith("needs:"):
            errors.append(f"{cid}: needs: row must be closed or named")
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        print(f"\n{len(errors)} coverage-check error(s)", file=sys.stderr)
        return 1
    print(f"ok: {len(rows)} coverage rows consistent with verb walk")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
