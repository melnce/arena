"""Race inputs and --race training."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
import pytest

pytest.importorskip("arena")

_RACE = Path(__file__).resolve().parents[1] / "race.py"
_spec = importlib.util.spec_from_file_location("arena_race", _RACE)
assert _spec and _spec.loader
race = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(race)

_TRAIN = Path(__file__).resolve().parents[1] / "train_value.py"
_tspec = importlib.util.spec_from_file_location("arena_train_value", _TRAIN)
assert _tspec and _tspec.loader
train_value = importlib.util.module_from_spec(_tspec)
_tspec.loader.exec_module(train_value)


def _craft_features(
    me_hp: float,
    opp_hp: float,
    opp_board_atk: float = 0.0,
    own_board_atk: float = 0.0,
    opp_kind: float = 1.0,
    own_kind: float = 1.0,
) -> np.ndarray:
    f = np.zeros((1, 567), dtype=np.float32)
    f[0, 41] = me_hp
    f[0, 70] = opp_hp
    if opp_board_atk:
        f[0, 253] = opp_board_atk
        f[0, 253 + 19] = opp_kind
    if own_board_atk:
        f[0, 153] = own_board_atk
        f[0, 153 + 19] = own_kind
    return f


def test_threat_boundary() -> None:
    hp = 10.0
    at_threat = _craft_features(hp, 20.0, opp_board_atk=hp)
    below = _craft_features(hp, 20.0, opp_board_atk=hp - 0.01)
    r_at = race.race_features(at_threat)[0]
    r_below = race.race_features(below)[0]
    assert r_at[2] == 1.0
    assert r_below[2] == 0.0


def test_near_boundary() -> None:
    hp = 10.0
    at_near = _craft_features(hp, 20.0, opp_board_atk=hp - 3.0)
    below = _craft_features(hp, 20.0, opp_board_atk=hp - 4.0)
    r_at = race.race_features(at_near)[0]
    r_below = race.race_features(below)[0]
    assert r_at[3] == 1.0
    assert r_below[3] == 0.0


def test_margin_clamps() -> None:
    f_lo = _craft_features(5.0, 20.0, opp_board_atk=20.0)
    f_hi = _craft_features(20.0, 20.0, own_board_atk=5.0)
    assert race.race_features(f_lo)[0, 4] == -5.0
    assert race.race_features(f_hi)[0, 10] == 10.0


def test_amulet_attack_ignored() -> None:
    follower = _craft_features(10.0, 20.0, opp_board_atk=8.0, opp_kind=1.0)
    amulet = _craft_features(10.0, 20.0, opp_board_atk=8.0, opp_kind=2.0)
    assert race.race_features(follower)[0, 2] == 0.0
    assert race.race_features(amulet)[0, 2] == 0.0
    assert race.race_features(follower)[0, 5] > race.race_features(amulet)[0, 5]


def _synthetic_export(tmp: Path) -> Path:
    export = tmp / "export"
    export.mkdir()
    n = 64
    fl = 567
    features = np.random.RandomState(0).rand(n, fl).astype(np.float32)
    features[:, 41] = np.linspace(1, 20, n)
    features[:, 70] = np.linspace(20, 1, n)
    ids = np.zeros((n, 220), dtype=np.uint32)
    labels = np.sign(np.random.RandomState(1).randn(n)).astype(np.float32)
    aux = np.zeros((n, 11), dtype=np.float32)
    aux[:, 0] = np.arange(n) // 4
    meta = {
        "samples": n,
        "feature_len": fl,
        "ids_len": 220,
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
        "encoding": 2,
    }
    features.tofile(export / "features.f32le")
    ids.tofile(export / "ids.u32le")
    labels.tofile(export / "labels.f32le")
    aux.tofile(export / "aux.f32le")
    (export / "meta.json").write_text(json.dumps(meta))
    return export


def test_race_training_writes_block(tmp_path: Path) -> None:
    export = _synthetic_export(tmp_path)
    out = tmp_path / "race.json"
    train_value.train(
        type(
            "Args",
            (),
            {
                "data": [str(export)],
                "model": "linear",
                "out": str(out),
                "holdout": 0.2,
                "epochs": 2,
                "seed": 1,
                "hidden": 8,
                "emb": 4,
                "l2": 1e-3,
                "max_samples": None,
                "target": "outcome",
                "mix_weight": 0.5,
                "search_scale": 60.0,
                "eval": None,
                "optimizer": "adam",
                "lbfgs_iters": 500,
                "std_floor": 1e-3,
                "race": True,
            },
        )()
    )
    spec = json.loads(out.read_text())
    assert "race" in spec
    assert spec["trained_on"].get("race") is True
    assert spec["race"]["names"] == list(race.RACE_NAMES)
    features = np.fromfile(export / "features.f32le", dtype="<f4").reshape(-1, 567)
    id_arr = np.fromfile(export / "ids.u32le", dtype="<u4").reshape(-1, 220)
    pred = train_value.predict(spec, features, id_arr)
    no_race = dict(spec)
    no_race.pop("race")
    pred_no = train_value.predict(no_race, features, id_arr)
    assert np.any(np.asarray(spec["race"]["w"]) != 0.0)
    assert not np.allclose(pred, pred_no)
    assert np.all(np.isfinite(pred))


def test_without_race_no_block(tmp_path: Path) -> None:
    export = _synthetic_export(tmp_path)
    out = tmp_path / "plain.json"
    train_value.train(
        type(
            "Args",
            (),
            {
                "data": [str(export)],
                "model": "linear",
                "out": str(out),
                "holdout": 0.2,
                "epochs": 1,
                "seed": 1,
                "hidden": 8,
                "emb": 4,
                "l2": 1e-3,
                "max_samples": None,
                "target": "outcome",
                "mix_weight": 0.5,
                "search_scale": 60.0,
                "eval": None,
                "optimizer": "adam",
                "lbfgs_iters": 500,
                "std_floor": 1e-3,
                "race": False,
            },
        )()
    )
    spec = json.loads(out.read_text())
    assert "race" not in spec
    assert "race" not in spec.get("trained_on", {})


def test_mlp_race_exits() -> None:
    result = subprocess.run(
        [sys.executable, str(_TRAIN), "--data", "/tmp", "--model", "mlp", "--out", "/tmp/x.json", "--race"],
        capture_output=True,
        text=True,
    )
    assert result.returncode != 0
    assert "linear" in result.stderr.lower() or "linear" in result.stdout.lower()
