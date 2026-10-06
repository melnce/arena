#!/usr/bin/env python3
"""Build a trainer stacked v3 model on shards with ids outside base vocab."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[3]
TRAIN = ROOT / "py" / "train_value.py"
spec = importlib.util.spec_from_file_location("tv", TRAIN)
assert spec and spec.loader
tv = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tv)

data = Path(sys.argv[1])
out = Path(sys.argv[2])
n = 128
feat = np.zeros((n, 961), dtype="<f4")
ids = np.zeros((n, 230), dtype="<u4")
labels = np.full(n, 0.05, dtype="<f4")
aux = np.zeros((n, 11), dtype="<f4")
UNK = 88001140
for i in range(n):
    aux[i, 0] = i // 16
    aux[i, 2] = float(i % 2)
    if int(aux[i, 2]) == 0:
        ids[i, 0] = UNK
        ids[i, 110] = UNK
        feat[i, 353] = 2.0
        ids[i, 9] = UNK
data.mkdir(parents=True, exist_ok=True)
feat.tofile(data / "features.f32le")
ids.tofile(data / "ids.u32le")
labels.tofile(data / "labels.f32le")
aux.tofile(data / "aux.f32le")
meta = {
    "samples": n,
    "feature_len": 961,
    "ids_len": 230,
    "encoding": 3,
    "aux_columns": [
        "game_index",
        "decision_index",
        "side",
        "turn",
        "phase",
        "v0",
        "legal_len",
        "chosen",
        "random",
        "first_is_me",
        "search_v",
    ],
    "layout": [],
    "decks": ["a", "b"],
    "games": 8,
}
(data / "meta.json").write_text(json.dumps(meta) + "\n")
tv.train(
    type(
        "A",
        (),
        {
            "stack_on": str(ROOT / "engine/models/h0-linear-v3.json"),
            "data": [str(data)],
            "model": "linear",
            "out": str(out),
            "holdout": 0.25,
            "seed": 3,
            "max_samples": None,
            "no_deck_controls": True,
            "l2_grid": "1e-4,1",
            "target": "outcome",
            "race": False,
            "optimizer": "adam",
            "l2": 1e-4,
            "stack_search_rows": 300_000,
        },
    )()
)
