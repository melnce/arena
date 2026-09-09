#!/usr/bin/env python3
"""List rotation + reachable-token ids from official-meta.json."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--meta", required=True, type=Path)
    args = p.parse_args()
    meta = json.loads(args.meta.read_text())
    cards = {k: v for k, v in meta.items() if k != "_meta"}
    rot = {k: v for k, v in cards.items() if v.get("is_include_rotation") is True}
    tokens: set[str] = set()
    seen: set[str] = set()
    queue = list(rot)
    while queue:
        cid = queue.pop()
        if cid in seen:
            continue
        seen.add(cid)
        c = cards.get(cid)
        if not c:
            continue
        if c.get("is_token"):
            tokens.add(cid)
        for rid in c.get("related_card_ids") or []:
            rid = str(rid)
            if rid not in seen:
                queue.append(rid)
    print(f"rotation {len(rot)}")
    print(f"tokens {len(tokens)}")
    print(f"pool {len(rot) + len(tokens)}")
    for cid in sorted(rot):
        print(f"card {cid} {cards[cid]['name']}")
    for cid in sorted(tokens):
        print(f"token {cid} {cards[cid]['name']}")


if __name__ == "__main__":
    main()
