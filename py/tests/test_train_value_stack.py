"""Stacked encoding-3 leaf fit (--stack-on)."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

import numpy as np
import pytest

ROOT = Path(__file__).resolve().parents[2]
TRAIN = ROOT / "py" / "train_value.py"
ENGINE_MODEL = ROOT / "engine" / "models" / "h0-linear-v3.json"

_spec = importlib.util.spec_from_file_location("train_value", TRAIN)
assert _spec and _spec.loader
train_value = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(train_value)

AUX = [
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
]

CREST_ID = 10714110
CARD_A = 90031210
CARD_B = 90031110


def _write_v3_shard(
    dir: Path,
    n: int,
    *,
    decks: list[str] | None = None,
    games: int = 8,
    rows_per_game: int = 16,
    crest_signal: bool = False,
    deck_signal: bool = False,
) -> None:
    feat = np.zeros((n, 961), dtype="<f4")
    ids = np.zeros((n, 230), dtype="<u4")
    labels = np.full(n, 0.05, dtype="<f4")
    aux = np.zeros((n, len(AUX)), dtype="<f4")
    for i in range(n):
        g = i // rows_per_game
        aux[i, 0] = g
        aux[i, 2] = float(i % 2)
        aux[i, 3] = 4.0
        aux[i, 9] = float(g % 2)
        deck_i = g % 2
        deck_j = (g + 1) % 2
        side = int(aux[i, 2])
        if crest_signal and side == 0:
            ids[i, 220] = CREST_ID
            labels[i] = 0.35
        if deck_signal:
            labels[i] = 0.35 if deck_i == 0 else -0.35
            card = CARD_A if deck_i == 0 else CARD_B
            feat[i, 577 + deck_i] = 1.0
            ids[i, 9 + deck_i] = card
    dir.mkdir(parents=True, exist_ok=True)
    feat.tofile(dir / "features.f32le")
    ids.tofile(dir / "ids.u32le")
    labels.tofile(dir / "labels.f32le")
    aux.tofile(dir / "aux.f32le")
    meta = {
        "samples": n,
        "feature_len": 961,
        "ids_len": 230,
        "encoding": 3,
        "aux_columns": AUX,
        "layout": [],
        "decks": decks or ["deck-a", "deck-b"],
        "games": games,
    }
    (dir / "meta.json").write_text(json.dumps(meta) + "\n")


def _run_stack(args: list[str]) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, str(TRAIN), *args]
    return subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT))


def test_stack_refusals(tmp_path: Path) -> None:
    data = tmp_path / "data"
    _write_v3_shard(data, 64, decks=["a", "b"])
    base = ENGINE_MODEL
    common = [
        "--data",
        str(data),
        "--model",
        "linear",
        "--stack-on",
        str(base),
        "--out",
        str(tmp_path / "out.json"),
        "--holdout",
        "0.2",
        "--seed",
        "0",
    ]
    for extra, needle in [
        (["--race"], "--stack-on cannot be used with --race"),
        (["--optimizer", "lbfgs"], "--stack-on cannot be used with --optimizer lbfgs"),
        (["--l2", "0.01"], "--stack-on cannot be used with --l2"),
    ]:
        r = _run_stack(common + extra)
        assert r.returncode != 0
        assert needle in (r.stderr + r.stdout)

    bad = json.loads(base.read_text())
    bad["encoding"] = 1
    bad_path = tmp_path / "bad.json"
    bad_path.write_text(json.dumps(bad))
    bad_cmd = [
        "--data",
        str(data),
        "--model",
        "linear",
        "--stack-on",
        str(bad_path),
        "--out",
        str(tmp_path / "bad-out.json"),
        "--holdout",
        "0.2",
        "--seed",
        "0",
    ]
    r = _run_stack(bad_cmd)
    assert r.returncode != 0
    assert "encoding 2" in (r.stderr + r.stdout)


def test_stack_crest_signal_and_zero_rows(tmp_path: Path) -> None:
    data = tmp_path / "data"
    n = 256
    _write_v3_shard(data, n, crest_signal=True, games=16, rows_per_game=16)
    out = tmp_path / "stack.json"
    train_value.train(
        type(
            "Args",
            (),
            {
                "stack_on": str(ENGINE_MODEL),
                "data": [str(data)],
                "model": "linear",
                "out": str(out),
                "holdout": 0.25,
                "seed": 7,
                "max_samples": None,
                "no_deck_controls": False,
                "l2_grid": "1e-4,1e-2,1",
                "target": "outcome",
                "race": False,
                "optimizer": "adam",
                "l2": 1e-4,
            },
        )()
    )
    spec = json.loads(out.read_text())
    report = json.loads(out.with_name(out.stem + ".report.json").read_text())
    base_mse = report["holdout_mse_steps"][0][1]
    final_mse = report["holdout_mse_steps"][-1][1]
    assert final_mse < base_mse - 1e-4
    zw = np.asarray(spec["linear"]["zone_w"], dtype=np.float32)
    vocab = spec["vocab"]
    idx = vocab.index(CREST_ID)
    own_crest_row = zw[5]
    assert own_crest_row[idx] > 0.05
    assert own_crest_row[0] == 0.0

    # zero new rows -> BASE on shard rows
    zw[5:13, :] = 0.0
    spec["linear"]["zone_w"] = zw.tolist()
    features, ids, labels, _ = train_value._mmap_shard(data, json.loads((data / "meta.json").read_text()))
    pred = train_value.predict(spec, features, ids) / float(spec["scale"])
    base_pred = train_value.predict(json.loads(ENGINE_MODEL.read_text()), features[:, :567], ids[:, :220])
    base_pred = base_pred / float(json.loads(ENGINE_MODEL.read_text())["scale"])
    assert float(np.max(np.abs(pred - base_pred))) < 1e-5


def test_stack_noise_and_deck_controls(tmp_path: Path) -> None:
    data = tmp_path / "data"
    _write_v3_shard(data, 256, games=16, rows_per_game=16, crest_signal=False)
    out_dc = tmp_path / "dc.json"
    train_value.train(
        type(
            "Args",
            (),
            {
                "stack_on": str(ENGINE_MODEL),
                "data": [str(data)],
                "model": "linear",
                "out": str(out_dc),
                "holdout": 0.25,
                "seed": 3,
                "max_samples": None,
                "no_deck_controls": False,
                "l2_grid": "1,1e-1,1e-2",
                "target": "outcome",
                "race": False,
                "optimizer": "adam",
                "l2": 1e-4,
            },
        )()
    )
    rep_dc = json.loads(out_dc.with_name(out_dc.stem + ".report.json").read_text())
    assert rep_dc["l2_choices"]["crests"] >= 0.1
    zw_dc = np.asarray(json.loads(out_dc.read_text())["linear"]["zone_w"], dtype=np.float32)
    assert np.max(np.abs(zw_dc[9:13, 1:])) < 0.05

    data2 = tmp_path / "data2"
    _write_v3_shard(data2, 256, games=16, rows_per_game=16, deck_signal=True)
    out_nd = tmp_path / "nd.json"
    train_value.train(
        type(
            "Args",
            (),
            {
                "stack_on": str(ENGINE_MODEL),
                "data": [str(data2)],
                "model": "linear",
                "out": str(out_nd),
                "holdout": 0.25,
                "seed": 3,
                "max_samples": None,
                "no_deck_controls": True,
                "l2_grid": "1,1e-1,1e-2",
                "target": "outcome",
                "race": False,
                "optimizer": "adam",
                "l2": 1e-4,
            },
        )()
    )
    zw_nd = np.asarray(json.loads(out_nd.read_text())["linear"]["zone_w"], dtype=np.float32)
    assert np.max(np.abs(zw_nd[9:13, 1:])) > np.max(np.abs(zw_dc[9:13, 1:])) + 0.01
