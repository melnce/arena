#!/usr/bin/env python3
"""Build tiny residual order nets for tests (see test_order_residual.py)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import random

REPO = Path(__file__).resolve().parents[4]
V53 = REPO / "engine/models/h0-order-v53.json"


def build(hidden: int, emb_dim: int, seed: int, scale: float = 0.15) -> dict:
    base = json.loads(V53.read_text())
    rng = random.Random(seed)

    def gauss() -> float:
        return rng.gauss(0.0, scale)

    vocab_len = len(base["vocab"])
    n_zones = len(base["zones"])
    feat_len = base["feature_len"]
    emb = [
        [[gauss() for _ in range(emb_dim)] for _ in range(vocab_len)]
        for _ in range(n_zones)
    ]
    in_dim = feat_len + n_zones * emb_dim
    w1 = [[gauss() for _ in range(in_dim)] for _ in range(hidden)]
    b1 = [gauss() for _ in range(hidden)]
    w2 = [gauss() for _ in range(hidden)]
    b2 = gauss()
    out = dict(base)
    out["arch"] = "residual"
    out["mlp"] = {
        "emb": emb,
        "w1": w1,
        "b1": b1,
        "w2": w2,
        "b2": b2,
    }
    return out


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--out", type=Path, required=True)
    p.add_argument("--hidden", type=int, default=4)
    p.add_argument("--emb-dim", type=int, default=2)
    p.add_argument("--seed", type=int, default=42)
    p.add_argument("--zero-mlp", action="store_true")
    args = p.parse_args()
    model = build(args.hidden, args.emb_dim, args.seed)
    if args.zero_mlp:
        h = args.hidden
        model["mlp"]["w2"] = [0.0] * h
        model["mlp"]["b2"] = 0.0
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(model, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    sys.exit(main())
