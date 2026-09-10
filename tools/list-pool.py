#!/usr/bin/env python3
"""List rotation + reachable-token ids from cards/official/catalog.json."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "cards" / "official" / "catalog.json"


def load_catalog() -> dict:
    raw = json.loads(CATALOG.read_text())
    return {k: v for k, v in raw.items() if k != "_meta"}


def derive_pool(cards: dict) -> tuple[list[str], list[str], dict[str, int]]:
    rotation = [cid for cid, rec in cards.items() if rec.get("rotation") and not rec.get("token")]
    seen = set(rotation)
    queue = list(rotation)
    tokens: list[str] = []
    while queue:
        cid = queue.pop()
        rec = cards.get(cid)
        if not rec:
            continue
        for rid in rec.get("related_card_ids") or []:
            rid = str(rid)
            rel = cards.get(rid)
            if not rel or rid in seen or not rel.get("token"):
                continue
            seen.add(rid)
            tokens.append(rid)
            queue.append(rid)
    se = {"crest": 0, "faith": 0, "crystallize": 0, "accelerate": 0}
    for cid in [*rotation, *tokens]:
        for effect in cards[cid].get("specific_effects") or []:
            typ = effect.get("type")
            if typ in se:
                se[typ] += 1
    return sorted(rotation), sorted(tokens), se


def main() -> None:
    cards = load_catalog()
    rotation, tokens, se = derive_pool(cards)
    print(f"rotation {len(rotation)}")
    print(f"tokens {len(tokens)}")
    print(f"pool {len(rotation) + len(tokens)}")
    print(f"crest {se['crest']} faith {se['faith']} crystallize {se['crystallize']} accelerate {se['accelerate']}")
    for cid in rotation:
        print(f"card {cid} {cards[cid]['name']}")
    for cid in tokens:
        print(f"token {cid} {cards[cid]['name']}")


if __name__ == "__main__":
    main()
