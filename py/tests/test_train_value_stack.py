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
UNKNOWN_CARD = 88001140  # in card db, not in h0-linear-v3 vocab


def _write_v3_shard(
    dir: Path,
    n: int,
    *,
    decks: list[str] | None = None,
    games: int = 8,
    rows_per_game: int = 16,
    crest_signal: bool = False,
    deck_signal: bool = False,
    unknown_base_ids: bool = False,
    noise_crests: bool = False,
    noise_labels_random: bool = False,
    multi_zone: bool = False,
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
        if unknown_base_ids and side == 0:
            ids[i, 0] = UNKNOWN_CARD
            ids[i, 110] = UNKNOWN_CARD
            feat[i, 353] = 2.0
            ids[i, 9] = UNKNOWN_CARD
        if noise_crests:
            rng = np.random.RandomState(i + 99)
            labels[i] = float(rng.uniform(-0.5, 0.5)) if noise_labels_random else 0.05
            for slot in range(5):
                ids[i, 220 + slot] = int(rng.choice([CREST_ID, 10714120, 10554110]))
                feat[i, 577 + slot] = float(rng.randint(1, 4))
                feat[i, 673 + slot] = float(rng.randint(1, 4))
                feat[i, 769 + slot] = float(rng.randint(1, 4))
                feat[i, 865 + slot] = float(rng.randint(1, 4))
        if multi_zone:
            ids[i, 220] = CREST_ID
            ids[i, 221] = 10714120
            ids[i, 222] = 10554110
            cards = [CARD_A, CARD_B, 90031110, 90031310]
            for j, card in enumerate(cards):
                feat[i, 577 + j] = 1.0
                feat[i, 673 + j] = 1.0
                ids[i, 9 + j] = card
                feat[i, 769 + j] = 2.0
                feat[i, 865 + j] = 1.0
            labels[i] = 0.2 if side == 0 else -0.1
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
    assert own_crest_row[idx] > 0.01
    assert own_crest_row[0] == 0.0

    # zero new rows -> BASE on shard rows
    zw[5:13, :] = 0.0
    spec["linear"]["zone_w"] = zw.tolist()
    features, ids, labels, _ = train_value._mmap_shard(data, json.loads((data / "meta.json").read_text()))
    base = json.loads(ENGINE_MODEL.read_text())
    pred = train_value.predict(spec, features, ids)
    base_pred = train_value.predict(base, features[:, :567], ids[:, :220])
    assert float(np.max(np.abs(pred - base_pred))) < 1e-4


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


def _stack_args(tmp_path: Path, data: Path, out: Path, **extra) -> object:
    base = {
        "stack_on": str(ENGINE_MODEL),
        "data": [str(data)],
        "model": "linear",
        "out": str(out),
        "holdout": 0.25,
        "seed": 11,
        "max_samples": None,
        "no_deck_controls": True,
        "l2_grid": "1e-4,1",
        "target": "outcome",
        "race": False,
        "optimizer": "adam",
        "l2": 1e-4,
        "stack_search_rows": 300_000,
    }
    base.update(extra)
    return type("Args", (), base)()


def test_stack_noise_penalty_selects_max(tmp_path: Path) -> None:
    data = tmp_path / "noise"
    _write_v3_shard(data, 512, games=32, rows_per_game=16, noise_crests=True)
    out = tmp_path / "noise.json"
    train_value.train(_stack_args(tmp_path, data, out))
    rep = json.loads(out.with_name(out.stem + ".report.json").read_text())
    grid_max = 1.0
    for group in train_value.STACK_ZONE_GROUPS:
        assert rep["l2_choices"][group] == grid_max


def test_stack_noise_penalty_leaky_search_fails(tmp_path: Path) -> None:
    import numpy as np

    data = tmp_path / "noise"
    _write_v3_shard(
        data, 512, games=32, rows_per_game=16, noise_crests=True, noise_labels_random=True
    )
    loaded = train_value.load_dirs([str(data)])
    features = loaded["features"]
    ids = loaded["ids"]
    labels = loaded["labels"]
    game_index = loaded["game_index"]
    train_idx, hold_idx = train_value.split_by_game(game_index, 0.25, 11)
    base = json.loads(ENGINE_MODEL.read_text())
    base_pre = train_value.compute_pre_activation(base, features, ids)
    vocab = train_value._stack_vocab([int(v) for v in base["vocab"]], ids, np.arange(features.shape[0]))
    zones = train_value.zones_for_encoding(3)
    z_lo, z_hi = 5, 13
    n_vocab = len(vocab)
    n_zone_w = (z_hi - z_lo) * (n_vocab - 1)
    grid = [1e-4, 1.0]
    search_tr = train_idx
    y_all = labels

    def _prepare(rows):
        return {
            "n": int(rows.size),
            "base_pre": base_pre[rows].astype(np.float64, copy=False),
            "y": y_all[rows],
            "own_deck": np.zeros(rows.size, dtype=np.int32),
            "opp_deck": np.zeros(rows.size, dtype=np.int32),
            "first_col": np.zeros(rows.size, dtype=np.float32),
            "zones": train_value._stack_sparse_zones(features, ids, rows, vocab, zones, z_lo, z_hi),
        }

    def unpack(theta):
        zw = theta[:n_zone_w].reshape(z_hi - z_lo, n_vocab - 1)
        full = np.zeros((z_hi - z_lo, n_vocab), dtype=np.float64)
        full[:, 1:] = zw
        return full, None, None, None

    def pre_from_fit(theta, rows):
        zw, _, _, _ = unpack(theta)
        prep = _prepare(rows)
        pre = prep["base_pre"].astype(np.float64, copy=True)
        pre += train_value._stack_zone_extra_sparse(zw, prep)
        return pre

    def fit_leaky(lams):
        leaky = np.concatenate([search_tr, hold_idx])
        prep = _prepare(leaky)
        theta0 = np.zeros(n_zone_w, dtype=np.float64)

        def objective(x, prep_fit, lams_fit, return_grad=False, return_loss=False):
            zw, _, _, _ = unpack(x)
            pre = prep_fit["base_pre"].astype(np.float64, copy=True)
            pre += train_value._stack_zone_extra_sparse(zw, prep_fit)
            y = prep_fit["y"]
            n = prep_fit["n"]
            pen = 0.0
            for name, (a, b) in train_value.STACK_ZONE_ROWS.items():
                lam = lams_fit[name]
                pen += 0.5 * lam * float(np.sum(zw[a - z_lo : b - z_lo, 1:] ** 2))
            loss = float(np.mean((np.tanh(pre) - y) ** 2)) + pen
            if not return_grad and not return_loss:
                return loss
            grad = np.zeros_like(x)
            resid = (2.0 / n) * (np.tanh(pre) - y) * (1.0 - np.tanh(pre) ** 2)
            off = 0
            for zi, (rows, cols, vals) in enumerate(prep_fit["zones"]):
                if rows.size:
                    np.add.at(grad[off : off + n_vocab - 1], cols - 1, resid[rows] * vals)
                off += n_vocab - 1
            for name, (a, b) in train_value.STACK_ZONE_ROWS.items():
                lam = lams_fit[name]
                for z in range(a, b):
                    g0 = (z - z_lo) * (n_vocab - 1)
                    g1 = g0 + (n_vocab - 1)
                    grad[g0:g1] += lam * zw[z - z_lo, 1:].reshape(-1)
            if return_loss:
                return loss, grad
            return grad

        return train_value._lbfgs_numpy(
            lambda x: objective(x, prep, lams, return_grad=True, return_loss=True),
            theta0,
        )

    lams_init = {g: grid[-1] for g in train_value.STACK_ZONE_GROUPS}
    lams_leaky, _ = train_value._stack_coordinate_search(
        grid,
        train_value.STACK_ZONE_GROUPS,
        fit_leaky,
        lambda theta: train_value._mse_tanh(pre_from_fit(theta, hold_idx), y_all[hold_idx]),
        lams_init,
    )
    assert any(lams_leaky[g] == grid[0] for g in train_value.STACK_ZONE_GROUPS)


def test_stack_unknown_base_ids_match_base_pre(tmp_path: Path) -> None:
    data = tmp_path / "unk"
    _write_v3_shard(data, 128, games=8, rows_per_game=16, unknown_base_ids=True)
    out = tmp_path / "unk.json"
    train_value.train(_stack_args(tmp_path, data, out))
    spec = json.loads(out.read_text())
    zw = np.asarray(spec["linear"]["zone_w"], dtype=np.float32)
    zw[5:13, :] = 0.0
    spec["linear"]["zone_w"] = zw.tolist()
    features, ids, _, _ = train_value._mmap_shard(
        data, json.loads((data / "meta.json").read_text())
    )
    base = json.loads(ENGINE_MODEL.read_text())
    pred = train_value.predict(spec, features, ids)
    base_pred = train_value.predict(base, features[:, :567], ids[:, :220])
    assert float(np.max(np.abs(pred - base_pred))) < 1e-4


def _stack_zone_extra_sparse_broken(zw, sparse):
    extra = np.zeros(sparse["n"], dtype=np.float64)
    for zi, (rows, cols, vals) in enumerate(sparse["zones"]):
        if rows.size:
            extra[rows] += vals * zw[zi, cols]
    return extra


def test_stack_zone_extra_sparse_repeated_rows() -> None:
    sparse = {
        "n": 1,
        "zones": [
            (
                np.array([0, 0], dtype=np.int64),
                np.array([3, 5], dtype=np.int64),
                np.array([1.0, 1.0], dtype=np.float64),
            )
        ]
        + [
            (
                np.zeros(0, dtype=np.int64),
                np.zeros(0, dtype=np.int64),
                np.zeros(0, dtype=np.float64),
            )
        ]
        * 7,
    }
    zw = np.zeros((8, 10), dtype=np.float64)
    zw[0, 3] = 0.3
    zw[0, 5] = 0.5
    got = train_value._stack_zone_extra_sparse(zw, sparse)
    assert float(got[0]) == pytest.approx(0.8)
    broken = _stack_zone_extra_sparse_broken(zw, sparse)
    assert float(broken[0]) == pytest.approx(0.5)


def _stack_holdout_mse_with_deck(
    spec: dict,
    report: dict,
    data: Path,
    holdout: float,
    seed: int,
) -> float:
    base = json.loads(ENGINE_MODEL.read_text())
    loaded = train_value.load_dirs([str(data)])
    features = loaded["features"]
    ids = loaded["ids"]
    labels = loaded["labels"]
    game_index = loaded["game_index"]
    aux = loaded["aux"]
    _, hold_idx = train_value.split_by_game(game_index, holdout, seed)
    base_pre = train_value.compute_pre_activation(base, features, ids)
    vocab = spec["vocab"]
    zones = train_value.zones_for_encoding(3)
    zw = np.asarray(spec["linear"]["zone_w"], dtype=np.float64)[5:13]
    prep = {
        "n": int(hold_idx.size),
        "zones": train_value._stack_sparse_zones(
            features, ids, hold_idx, vocab, zones, 5, 13
        ),
    }
    pre_ho = base_pre[hold_idx].astype(np.float64, copy=True)
    pre_ho += train_value._stack_zone_extra_sparse(zw, prep)
    if report.get("deck_controls"):
        meta = json.loads((data / "meta.json").read_text())
        decks, games_per_pair = train_value._shard_deck_layout(meta)
        cols = list(meta["aux_columns"])
        side = aux[:, cols.index("side")]
        first_is_me = aux[:, cols.index("first_is_me")]
        own_deck, opp_deck, first_col = train_value._row_deck_indices(
            game_index, side, first_is_me, decks, games_per_pair
        )
        deck = report["deck_theta"]
        own_w = np.asarray(deck["own"], dtype=np.float64)
        opp_w = np.asarray(deck["opp"], dtype=np.float64)
        first_w = float(deck["first"])
        pre_ho += own_w[own_deck[hold_idx]]
        pre_ho += opp_w[opp_deck[hold_idx]]
        pre_ho += first_w * first_col[hold_idx]
    return train_value._mse_tanh(pre_ho, labels[hold_idx])


def _multi_zone_holdout_checks(spec: dict, data: Path, holdout: float, seed: int) -> None:
    base = json.loads(ENGINE_MODEL.read_text())
    loaded = train_value.load_dirs([str(data)])
    features = loaded["features"]
    ids = loaded["ids"]
    game_index = loaded["game_index"]
    _, hold_idx = train_value.split_by_game(game_index, holdout, seed)
    assert hold_idx.size > 0
    base_pre = train_value.compute_pre_activation(base, features, ids)
    out_pre = train_value.compute_pre_activation(spec, features, ids)
    expected_new = (out_pre - base_pre)[hold_idx]
    vocab = spec["vocab"]
    zones = train_value.zones_for_encoding(3)
    zw = np.asarray(spec["linear"]["zone_w"], dtype=np.float64)[5:13]
    prep = {
        "n": int(hold_idx.size),
        "zones": train_value._stack_sparse_zones(
            features, ids, hold_idx, vocab, zones, 5, 13
        ),
    }
    sparse_new = train_value._stack_zone_extra_sparse(zw, prep)
    dense_new = train_value._new_zone_extra(
        zw, ids, features, hold_idx, vocab, zones, 5, 13
    )
    assert float(np.max(np.abs(sparse_new - dense_new))) < 1e-9
    assert float(np.max(np.abs(sparse_new - expected_new))) < 1e-6


def test_stack_multi_zone_holdout_parity(tmp_path: Path) -> None:
    data = tmp_path / "multi"
    _write_v3_shard(
        data,
        512,
        games=32,
        rows_per_game=16,
        multi_zone=True,
        decks=["deck-a", "deck-b"],
    )
    out = tmp_path / "multi.json"
    train_value.train(
        _stack_args(
            tmp_path,
            data,
            out,
            no_deck_controls=False,
            l2_grid="1e-4,1e-2,1",
            seed=17,
        )
    )
    spec = json.loads(out.read_text())
    report = json.loads(out.with_name(out.stem + ".report.json").read_text())
    _multi_zone_holdout_checks(spec, data, 0.25, 17)
    mse = _stack_holdout_mse_with_deck(spec, report, data, 0.25, 17)
    final_mse = report["holdout_mse_final"]
    assert abs(mse - final_mse) < 1e-5


def test_stack_two_folders_cli_deck_controls(tmp_path: Path) -> None:
    d1 = tmp_path / "d1"
    d2 = tmp_path / "d2"
    _write_v3_shard(d1, 128, games=4, rows_per_game=8, multi_zone=True)
    _write_v3_shard(d2, 128, games=4, rows_per_game=8, multi_zone=True)
    out = tmp_path / "two.json"
    r = _run_stack(
        [
            "--data",
            str(d1),
            str(d2),
            "--model",
            "linear",
            "--target",
            "outcome",
            "--stack-on",
            str(ENGINE_MODEL),
            "--seed",
            "1",
            "--out",
            str(out),
            "--holdout",
            "0.2",
        ]
    )
    assert r.returncode == 0, r.stderr + r.stdout
    assert out.is_file()
    report = json.loads(out.with_name(out.stem + ".report.json").read_text())
    assert report["deck_controls"] is True


def test_stack_deck_controls_helper_two_folders(tmp_path: Path) -> None:
    d1 = tmp_path / "d1"
    d2 = tmp_path / "d2"
    _write_v3_shard(d1, 128, games=4, rows_per_game=8, multi_zone=True)
    _write_v3_shard(d2, 128, games=4, rows_per_game=8, multi_zone=True)
    loaded = train_value.load_dirs([str(d1), str(d2)])
    meta = json.loads((d1 / "meta.json").read_text())
    cols = list(meta["aux_columns"])
    decks, games_per_pair = train_value._shard_deck_layout(meta)
    own, opp, first = train_value._stack_deck_controls(
        loaded["aux"], cols, decks, games_per_pair
    )
    for d in (d1, d2):
        one = train_value.load_dirs([str(d)])
        o, p, f = train_value._stack_deck_controls(one["aux"], cols, decks, games_per_pair)
        if d == d1:
            own1, opp1, first1 = o, p, f
        else:
            own2, opp2, first2 = o, p, f
    assert np.array_equal(own, np.concatenate([own1, own2]))
    assert np.array_equal(opp, np.concatenate([opp1, opp2]))
    assert np.array_equal(first, np.concatenate([first1, first2]))
    assert int(own.min()) >= 0 and int(own.max()) < 2
    assert int(opp.min()) >= 0 and int(opp.max()) < 2


def test_stack_refuse_mixed_aux_columns(tmp_path: Path) -> None:
    d1 = tmp_path / "d1"
    d2 = tmp_path / "d2"
    _write_v3_shard(d1, 64, games=4, rows_per_game=8)
    _write_v3_shard(d2, 64, games=4, rows_per_game=8)
    meta2 = json.loads((d2 / "meta.json").read_text())
    meta2["aux_columns"] = list(reversed(meta2["aux_columns"]))
    (d2 / "meta.json").write_text(json.dumps(meta2) + "\n")
    out = tmp_path / "mixed.json"
    with pytest.raises(SystemExit, match="identical aux_columns"):
        train_value.train_stack_on(
            type(
                "Args",
                (),
                {
                    "stack_on": str(ENGINE_MODEL),
                    "data": [str(d1), str(d2)],
                    "model": "linear",
                    "out": str(out),
                    "holdout": 0.2,
                    "seed": 1,
                    "max_samples": None,
                    "no_deck_controls": False,
                    "l2_grid": "1e-4,1",
                    "target": "outcome",
                    "race": False,
                    "optimizer": "adam",
                    "l2": 1e-4,
                    "stack_search_rows": 300_000,
                },
            )()
        )


def test_stack_deck_controls_bad_layout(tmp_path: Path) -> None:
    data = tmp_path / "bad"
    # 128 rows / 4 rows_per_game = 32 games, but 2 decks * 2 * 4 games = 16 max
    _write_v3_shard(data, 128, games=4, rows_per_game=4)
    loaded = train_value.load_dirs([str(data)])
    meta = json.loads((data / "meta.json").read_text())
    cols = list(meta["aux_columns"])
    decks, games_per_pair = train_value._shard_deck_layout(meta)
    with pytest.raises(SystemExit, match="game_index out of range"):
        train_value._stack_deck_controls(loaded["aux"], cols, decks, games_per_pair)


def test_stack_multi_zone_holdout_parity_broken_sparse_fails(tmp_path: Path) -> None:
    data = tmp_path / "multi"
    _write_v3_shard(
        data,
        256,
        games=16,
        rows_per_game=16,
        multi_zone=True,
        decks=["deck-a", "deck-b"],
    )
    out = tmp_path / "multi.json"
    train_value.train(
        _stack_args(
            tmp_path,
            data,
            out,
            no_deck_controls=False,
            l2_grid="1e-4,1e-2",
            seed=19,
        )
    )
    spec = json.loads(out.read_text())
    orig = train_value._stack_zone_extra_sparse
    train_value._stack_zone_extra_sparse = _stack_zone_extra_sparse_broken
    try:
        with pytest.raises(AssertionError):
            _multi_zone_holdout_checks(spec, data, 0.25, 19)
    finally:
        train_value._stack_zone_extra_sparse = orig
