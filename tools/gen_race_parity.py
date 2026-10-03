#!/usr/bin/env python3
"""Generate engine/tests/fixtures/race/parity.json for Rust/Python parity."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PY = ROOT / "py"
if str(PY) not in sys.path:
    sys.path.insert(0, str(PY))

import importlib.util

import numpy as np

_samples_spec = importlib.util.spec_from_file_location("arena_samples", PY / "samples.py")
assert _samples_spec and _samples_spec.loader
samples = importlib.util.module_from_spec(_samples_spec)
_samples_spec.loader.exec_module(samples)

_race_spec = importlib.util.spec_from_file_location("arena_race", PY / "race.py")
assert _race_spec and _race_spec.loader
race_mod = importlib.util.module_from_spec(_race_spec)
_race_spec.loader.exec_module(race_mod)

_train_spec = importlib.util.spec_from_file_location("arena_train_value", PY / "train_value.py")
assert _train_spec and _train_spec.loader
train_value = importlib.util.module_from_spec(_train_spec)
_train_spec.loader.exec_module(train_value)

OUT = ROOT / "engine" / "tests" / "fixtures" / "race" / "parity.json"


def _select_rows(features: np.ndarray, n: int = 60) -> np.ndarray:
    """Pick rows covering threat, near-threat, low HP, and amulets."""
    r = race_mod.race_features(features)
    me_hp = features[:, 41]
    opp_hp = features[:, 70]
    kind_opp = features[:, 253 + np.arange(5) * 20 + 19]
    has_amulet = np.any(kind_opp == 2.0, axis=1)
    picked: list[int] = []
    seen: set[int] = set()

    def add(mask: np.ndarray, limit: int) -> None:
        idx = np.flatnonzero(mask)
        for i in idx:
            ii = int(i)
            if ii in seen:
                continue
            picked.append(ii)
            seen.add(ii)
            if len(picked) >= n:
                return
            if sum(1 for p in picked if mask[p]) >= limit:
                break

    add(r[:, 2] == 1.0, 12)  # threat_me
    add(r[:, 3] == 1.0, 12)  # near_me
    add(me_hp <= 5.0, 10)
    add(has_amulet, 8)
    add(r[:, 8] == 1.0, 8)  # threat_opp
    for i in range(features.shape[0]):
        if len(picked) >= n:
            break
        if i not in seen:
            picked.append(i)
            seen.add(i)
    return np.asarray(picked[:n], dtype=np.int64)


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--export", type=Path, help="existing export dir (else generate)")
    args = p.parse_args()

    if args.export is not None:
        export_dir = args.export
    else:
        tmp = Path(tempfile.mkdtemp(prefix="race-parity-"))
        export_dir = tmp / "export"
        cmd = [
            sys.executable,
            str(PY / "matchup.py"),
            "--decks",
            "meta-abyss-midrange",
            "meta-sword-rally",
            "--games",
            "2",
            "--seed",
            "5",
            "--encoding",
            "2",
            "--export",
            str(export_dir),
        ]
        subprocess.run(cmd, cwd=ROOT, check=True)

    data = samples.load(export_dir)
    features = data["features"]
    ids = data["ids"]
    row_idx = _select_rows(features)
    sub_feat = features[row_idx]
    sub_ids = ids[row_idx]
    race_vals = race_mod.race_features(sub_feat).tolist()

    model_path = export_dir / "race_model.json"
    train_value.train(
        argparse.Namespace(
            data=[str(export_dir)],
            model="linear",
            out=str(model_path),
            holdout=0.0,
            epochs=1,
            seed=0,
            hidden=128,
            emb=16,
            l2=1e-4,
            max_samples=None,
            target="outcome",
            mix_weight=0.5,
            search_scale=60.0,
            eval=None,
            optimizer="adam",
            lbfgs_iters=500,
            std_floor=1e-3,
            race=True,
        )
    )
    spec = json.loads(model_path.read_text())
    pred = train_value.predict(spec, sub_feat, sub_ids).tolist()

    fixture = {
        "offsets": {
            "me_scalars": 41,
            "opp_scalars": 70,
            "own_board": 153,
            "opp_board": 253,
            "board_width": 20,
        },
        "model": spec,
        "rows": [
            {
                "features": sub_feat[i].tolist(),
                "ids": sub_ids[i].tolist(),
                "race": race_vals[i],
                "value": pred[i],
            }
            for i in range(len(row_idx))
        ],
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(fixture, indent=2) + "\n")
    print(f"wrote {OUT} ({len(row_idx)} rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
