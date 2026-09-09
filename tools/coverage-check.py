#!/usr/bin/env python3
"""Assert coverage.md ops/conditions match a coarse verb walk of official texts."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from coverage_lib import cell_has, parse_coverage_table, required_conditions, required_ops

ROOT = Path(__file__).resolve().parents[1]
COVERAGE = ROOT / "docs" / "coverage.md"
CATALOG = ROOT / "cards" / "official" / "catalog.json"


def load_catalog() -> dict:
    return json.loads(CATALOG.read_text())


def official_record(catalog: dict, cid: str) -> tuple[str | None, str | None]:
    """Return (name, text) for a coverage id from the Cygames catalog."""
    if cid.startswith("crest:") or cid.startswith("faith:"):
        typ, nid = cid.split(":", 1)
        rec = catalog.get(nid)
        if not rec:
            return None, None
        for effect in rec.get("specific_effects") or []:
            if effect.get("type") == typ:
                return rec.get("name"), effect.get("text") or ""
        return rec.get("name"), None
    rec = catalog.get(cid)
    if not rec:
        return None, None
    return rec.get("name"), rec.get("text") or ""


def main() -> int:
    rows = parse_coverage_table(COVERAGE.read_text())
    catalog = load_catalog()
    errors: list[str] = []
    if len(rows) != 615:
        errors.append(f"expected 615 coverage rows, got {len(rows)}")
    for row in rows:
        cid = row["id"]
        name, text = official_record(catalog, cid)
        if name is None or text is None:
            errors.append(f"{cid}: missing official catalog text")
            continue
        for op in required_ops(text):
            if not cell_has(row["ops"], op):
                errors.append(f"{cid} {name}: text needs op {op}; ops column is {row['ops']!r}")
        for cond in required_conditions(text):
            if not cell_has(row["conditions"], cond):
                errors.append(
                    f"{cid} {name}: text needs condition {cond}; conditions column is {row['conditions']!r}"
                )
        if row["status"].startswith("needs:"):
            errors.append(f"{cid}: needs: row must be closed or named")
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        print(f"\n{len(errors)} coverage-check error(s)", file=sys.stderr)
        return 1
    print(f"ok: {len(rows)} coverage rows consistent with official catalog verb walk")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
