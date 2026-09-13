"""Train-value fixtures: model file shapes and numpy/script forward agree."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

import pytest

np = pytest.importorskip("numpy")
pytest.importorskip("torch")

_TRAIN = Path(__file__).resolve().parents[1] / "train_value.py"
_spec = importlib.util.spec_from_file_location("arena_train_value", _TRAIN)
assert _spec and _spec.loader
train_value = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(train_value)


def _forest(root: Path) -> dict[str, dict[str, int]]:
    path = root / "oracle" / "decks" / "basic-forest.json"
    return {"basic-forest": {str(k): int(v) for k, v in json.loads(path.read_text()).items()}}


def _numpy_forward(spec: dict, features, ids):
    """Independent numpy forward (the Rust-side reference)."""
    mean = np.asarray(spec["feat_mean"], dtype=np.float32)
    std = np.asarray(spec["feat_std"], dtype=np.float32)
    x = (np.asarray(features, dtype=np.float32) - mean) / std
    vocab = np.asarray(spec["vocab"], dtype=np.int64)
    raw = np.asarray(ids, dtype=np.int64)
    pos = np.searchsorted(vocab, raw)
    pos = np.clip(pos, 0, len(vocab) - 1)
    idx = np.where(vocab[pos] == raw, pos, 0)
    scale = float(spec["scale"])
    extra = np.zeros(x.shape[0], dtype=np.float32)
    zone_vecs = []
    for z, zone in enumerate(spec["zones"]):
        sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
        zidx = idx[:, sl]
        if zone["hist_offset"] is None:
            count = np.ones(zidx.shape, dtype=np.float32)
        else:
            off = int(zone["hist_offset"])
            count = np.asarray(features[:, off : off + zidx.shape[1]], dtype=np.float32)
        if spec["arch"] == "linear":
            zw = np.asarray(spec["linear"]["zone_w"], dtype=np.float32)
            extra += (zw[z][zidx] * count).sum(axis=1)
        else:
            emb = np.asarray(spec["mlp"]["emb"], dtype=np.float32)
            zone_vecs.append((emb[zidx] * count[..., None]).sum(axis=1))
    if spec["arch"] == "linear":
        w = np.asarray(spec["linear"]["w"], dtype=np.float32)
        b = float(spec["linear"]["b"])
        pre = x @ w + extra + b
    else:
        w1 = np.asarray(spec["mlp"]["w1"], dtype=np.float32)
        b1 = np.asarray(spec["mlp"]["b1"], dtype=np.float32)
        w2 = np.asarray(spec["mlp"]["w2"], dtype=np.float32)
        b2 = float(spec["mlp"]["b2"])
        inp = np.concatenate([x, *zone_vecs], axis=1)
        h = np.maximum(inp @ w1.T + b1, 0.0)
        pre = h @ w2 + b2
    return (scale * np.tanh(pre)).astype(np.float32)


def _assert_shapes(spec: dict, arch: str) -> None:
    v = len(spec["vocab"])
    assert spec["arch"] == arch
    assert spec["feature_len"] == 545
    assert len(spec["feat_mean"]) == 545
    assert len(spec["feat_std"]) == 545
    assert spec["vocab"][0] == 0
    assert spec["vocab"] == sorted(spec["vocab"])
    assert len(spec["zones"]) == 5
    assert spec["scale"] == 60.0
    names = [z["name"] for z in spec["zones"]]
    assert names == ["own_hand", "own_deck", "opp_board", "own_board", "opp_pool"]
    assert spec["zones"][0]["id_offset"] == 0 and spec["zones"][0]["count"] == 9
    assert spec["zones"][1]["id_offset"] == 9 and spec["zones"][1]["hist_offset"] == 353
    assert spec["zones"][2]["id_offset"] == 105 and spec["zones"][2]["count"] == 5
    assert spec["zones"][3]["id_offset"] == 110 and spec["zones"][3]["count"] == 5
    assert spec["zones"][4]["id_offset"] == 115 and spec["zones"][4]["hist_offset"] == 449
    if arch == "linear":
        lin = spec["linear"]
        assert len(lin["w"]) == 545
        assert len(lin["zone_w"]) == 5
        assert all(len(row) == v for row in lin["zone_w"])
        assert isinstance(lin["b"], float)
    else:
        mlp = spec["mlp"]
        emb_w = len(mlp["emb"][0])
        hidden = len(mlp["b1"])
        assert len(mlp["emb"]) == v
        assert all(len(row) == emb_w for row in mlp["emb"])
        assert len(mlp["w1"]) == hidden
        assert all(len(row) == 545 + 5 * emb_w for row in mlp["w1"])
        assert len(mlp["w2"]) == hidden
        assert isinstance(mlp["b2"], float)


def test_train_linear_and_mlp_shapes_and_numpy_forward(db, root: Path, tmp_path: Path) -> None:
    import arena

    export = tmp_path / "tiny"
    arena.matchup(
        db,
        _forest(root),
        2,
        17,
        policy_a="h0-fast",
        policy_b="h0-fast",
        export=str(export),
        threads=1,
    )
    data = train_value.load_dirs([str(export)])
    n = int(data["features"].shape[0])
    assert n > 0
    take = min(100, n)

    for arch, extra in (("linear", []), ("mlp", ["--hidden", "8", "--emb", "4"])):
        out = tmp_path / f"{arch}.json"
        train_value.main(
            [
                "--data",
                str(export),
                "--model",
                arch,
                "--out",
                str(out),
                "--epochs",
                "2",
                "--seed",
                "3",
                *extra,
            ]
        )
        spec = json.loads(out.read_text())
        _assert_shapes(spec, arch)
        feats = data["features"][:take]
        ids = data["ids"][:take]
        script = train_value.predict(spec, feats, ids)
        indie = _numpy_forward(spec, feats, ids)
        np.testing.assert_allclose(indie, script, atol=1e-5, rtol=0.0)
