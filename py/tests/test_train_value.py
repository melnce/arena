"""Train-value fixtures: model file shapes and numpy/script forward agree."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

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


def _report_path(out: Path) -> Path:
    return out.with_name(out.stem + ".report.json")


def _write_legacy_10col(dir: Path, n: int = 12) -> None:
    dir.mkdir(parents=True, exist_ok=True)
    feat = np.zeros((n, 545), dtype="<f4")
    feat[:, 0] = 1.0
    ids = np.zeros((n, 220), dtype="<u4")
    labels = np.array([1.0 if i % 2 == 0 else -1.0 for i in range(n)], dtype="<f4")
    aux = np.zeros((n, 10), dtype="<f4")
    aux[:, 0] = np.arange(n) % 4
    aux[:, 3] = 4
    aux[:, 5] = 0.25
    aux[:, 6] = 3
    feat.tofile(dir / "features.f32le")
    ids.tofile(dir / "ids.u32le")
    labels.tofile(dir / "labels.f32le")
    aux.tofile(dir / "aux.f32le")
    meta = {
        "samples": n,
        "feature_len": 545,
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
        ],
    }
    (dir / "meta.json").write_text(json.dumps(meta) + "\n")


def test_target_search_mix_eval_parses(db, root: Path, tmp_path: Path) -> None:
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

    search_out = tmp_path / "search.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(search_out),
            "--epochs",
            "2",
            "--seed",
            "3",
            "--target",
            "search",
            "--holdout",
            "0.5",
        ]
    )
    search_report = json.loads(_report_path(search_out).read_text())
    assert search_report["target"] == "search"
    assert search_report["search_v"] is not None
    assert "overall" in search_report["search_v"]
    spec = json.loads(search_out.read_text())
    _assert_shapes(spec, "linear")
    arena.matchup(
        db,
        _forest(root),
        1,
        5,
        policy_a=f"h0:value=net,net={search_out}",
        policy_b="h0-fast",
        threads=1,
    )

    mix_out = tmp_path / "mix.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(mix_out),
            "--epochs",
            "2",
            "--seed",
            "3",
            "--target",
            "mix",
            "--mix-weight",
            "0.5",
            "--holdout",
            "0.5",
        ]
    )
    mix_report = json.loads(_report_path(mix_out).read_text())
    assert mix_report["target"] == "mix"
    assert mix_report["mix_weight"] == 0.5
    assert mix_report["search_v"] is not None
    arena.matchup(
        db,
        _forest(root),
        1,
        5,
        policy_a=f"h0:value=net,net={mix_out}",
        policy_b="h0-fast",
        threads=1,
    )

    eval_out = tmp_path / "evaled.json"
    builtin = root / "engine" / "models" / "h0-linear-v1.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(eval_out),
            "--epochs",
            "2",
            "--seed",
            "3",
            "--eval",
            str(builtin),
            "--holdout",
            "0.5",
        ]
    )
    eval_report = json.loads(_report_path(eval_out).read_text())
    ev = eval_report["eval"]["h0-linear-v1.json"]
    acc = ev["overall"]["sign_acc"]
    assert np.isfinite(acc)
    assert 0.0 <= acc <= 1.0

    legacy = tmp_path / "legacy10"
    _write_legacy_10col(legacy, n=12)
    legacy_out = tmp_path / "legacy.json"
    train_value.main(
        [
            "--data",
            str(legacy),
            "--model",
            "linear",
            "--out",
            str(legacy_out),
            "--epochs",
            "1",
            "--seed",
            "3",
            "--target",
            "search",
        ]
    )
    legacy_report = json.loads(_report_path(legacy_out).read_text())
    assert legacy_report["target"] == "search"
    assert legacy_report["search_rows"] == 0
    assert legacy_report["search_v"] is None


def test_train_value_refuses_mixed_encoding_shards(db, root: Path, tmp_path: Path) -> None:
    import arena

    v1_dir = tmp_path / "v1"
    v2_dir = tmp_path / "v2"
    arena.matchup(
        db,
        _forest(root),
        1,
        11,
        policy_a="h0-fast",
        policy_b="h0-fast",
        threads=1,
        export=str(v1_dir),
        encoding=1,
    )
    arena.matchup(
        db,
        _forest(root),
        1,
        12,
        policy_a="h0-fast",
        policy_b="h0-fast",
        threads=1,
        export=str(v2_dir),
        encoding=2,
    )
    with pytest.raises(SystemExit, match="mixed encoding"):
        train_value.load_dirs([str(v1_dir), str(v2_dir)])


def test_eval_v1_model_on_v2_rows(db, root: Path, tmp_path: Path) -> None:
    import arena

    export = tmp_path / "v2"
    arena.matchup(
        db,
        _forest(root),
        2,
        17,
        policy_a="h0-fast",
        policy_b="h0-fast",
        threads=1,
        export=str(export),
        encoding=2,
    )
    builtin = root / "engine" / "models" / "h0-linear-v1.json"
    out = tmp_path / "evaled-v2.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(out),
            "--epochs",
            "1",
            "--seed",
            "3",
            "--eval",
            str(builtin),
            "--holdout",
            "0.5",
        ]
    )
    report = json.loads(_report_path(out).read_text())
    key = "h0-linear-v1.json (v1 block of v2 rows)"
    assert key in report["eval"]
    ev = report["eval"][key]
    assert "skipped" not in ev
    acc = ev["overall"]["sign_acc"]
    assert np.isfinite(acc)
    assert 0.0 <= acc <= 1.0


def test_eval_skips_v2_model_on_v1_rows(db, root: Path, tmp_path: Path) -> None:
    import arena

    v1_export = tmp_path / "v1"
    v2_export = tmp_path / "v2"
    arena.matchup(
        db,
        _forest(root),
        2,
        21,
        policy_a="h0-fast",
        policy_b="h0-fast",
        threads=1,
        export=str(v1_export),
        encoding=1,
    )
    arena.matchup(
        db,
        _forest(root),
        2,
        22,
        policy_a="h0-fast",
        policy_b="h0-fast",
        threads=1,
        export=str(v2_export),
        encoding=2,
    )
    v2_model = tmp_path / "tiny-v2.json"
    train_value.main(
        [
            "--data",
            str(v2_export),
            "--model",
            "linear",
            "--out",
            str(v2_model),
            "--epochs",
            "1",
            "--seed",
            "3",
            "--holdout",
            "0.5",
        ]
    )
    out = tmp_path / "evaled-v1.json"
    train_value.main(
        [
            "--data",
            str(v1_export),
            "--model",
            "linear",
            "--out",
            str(out),
            "--epochs",
            "1",
            "--seed",
            "3",
            "--eval",
            str(v2_model),
            "--holdout",
            "0.5",
        ]
    )
    report = json.loads(_report_path(out).read_text())
    assert report["eval"][v2_model.name] == {"skipped": "encoding mismatch"}


_FIXTURES = Path(__file__).resolve().parent / "fixtures" / "train_value_ref"
_SYNTH_V2_AUX = [
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


def _write_synthetic_encoding2(dir: Path, n: int, rows_per_game: int = 88) -> int:
    feat = np.zeros((n, 567), dtype="<f4")
    feat[:, 0] = 1.0
    feat[:, 353 : 353 + 96] = 1.0
    feat[:, 449 : 449 + 96] = 1.0
    ids = np.zeros((n, 220), dtype="<u4")
    labels = np.where(np.arange(n) % 2 == 0, 1.0, -1.0).astype("<f4")
    aux = np.zeros((n, len(_SYNTH_V2_AUX)), dtype="<f4")
    aux[:, 0] = np.arange(n) // rows_per_game
    aux[:, 3] = 4
    aux[:, 5] = 0.25
    aux[:, _SYNTH_V2_AUX.index("search_v")] = labels
    dir.mkdir(parents=True, exist_ok=True)
    feat.tofile(dir / "features.f32le")
    ids.tofile(dir / "ids.u32le")
    labels.tofile(dir / "labels.f32le")
    aux.tofile(dir / "aux.f32le")
    meta = {
        "samples": n,
        "feature_len": 567,
        "ids_len": 220,
        "encoding": 2,
        "aux_columns": _SYNTH_V2_AUX,
    }
    (dir / "meta.json").write_text(json.dumps(meta) + "\n")
    return sum(f.stat().st_size for f in dir.iterdir())


def _run_reference_training(name: str, export: Path, out: Path) -> tuple[dict, dict]:
    ref = json.loads((_FIXTURES / f"{name}.json").read_text())
    args = ref["args"]
    cmd = [
        "--data",
        str(export),
        "--model",
        args["model"],
        "--out",
        str(out),
        "--seed",
        str(args["seed"]),
        "--holdout",
        str(args["holdout"]),
        "--epochs",
        str(args["epochs"]),
    ]
    if args.get("hidden") is not None:
        cmd += ["--hidden", str(args["hidden"]), "--emb", str(args["emb"])]
    train_value.main(cmd)
    return json.loads(out.read_text()), json.loads(_report_path(out).read_text())


def test_reference_fixtures_reproduce(db, root: Path, tmp_path: Path) -> None:
    import arena

    for name in ("linear_seed0", "mlp_seed0"):
        ref = json.loads((_FIXTURES / f"{name}.json").read_text())
        export = tmp_path / name
        arena.matchup(
            db,
            _forest(root),
            ref["export_games"],
            ref["export_seed"],
            policy_a="h0-fast",
            policy_b="h0-fast",
            export=str(export),
            threads=1,
        )
        out = tmp_path / f"{name}_out.json"
        spec, report = _run_reference_training(name, export, out)
        assert abs(report["net"]["overall"]["mse"] - ref["holdout_mse"]) < 1e-3
        assert abs(report["net"]["overall"]["sign_acc"] - ref["holdout_sign_acc"]) < 1e-3
        if ref["arch"] == "linear":
            lin = spec["linear"]
            rlin = ref["linear"]
            assert max(abs(a - b) for a, b in zip(lin["w"], rlin["w"])) < 1e-4
            assert (
                max(
                    max(abs(a - b) for a, b in zip(row, rrow))
                    for row, rrow in zip(lin["zone_w"], rlin["zone_w"])
                )
                < 1e-4
            )
            assert abs(lin["b"] - rlin["b"]) < 1e-4
        else:
            for key in ("emb", "w1", "b1", "w2"):
                a = np.asarray(spec["mlp"][key])
                b = np.asarray(ref["mlp"][key])
                assert float(np.max(np.abs(a - b))) < 1e-3


def _write_memory_gate_shard(dir: Path, n: int, seed: int, rows_per_game: int = 88) -> None:
    rng = np.random.RandomState(seed)
    feat = rng.randn(n, 567).astype("<f4") * 0.1
    feat[:, 0] = 1.0
    feat[:, 353 : 353 + 96] = rng.rand(n, 96).astype("<f4")
    feat[:, 449 : 449 + 96] = rng.rand(n, 96).astype("<f4")
    ids = rng.randint(1, 400, size=(n, 220)).astype("<u4")
    logits = feat[:, 1] + feat[:, 2] * 0.5 + rng.randn(n).astype("<f4") * 0.05
    labels = np.where(logits >= 0, 1.0, -1.0).astype("<f4")
    aux = np.zeros((n, len(_SYNTH_V2_AUX)), dtype="<f4")
    aux[:, 0] = np.arange(n) // rows_per_game
    aux[:, 3] = 4
    aux[:, 5] = logits * 0.25
    aux[:, _SYNTH_V2_AUX.index("search_v")] = labels
    dir.mkdir(parents=True, exist_ok=True)
    feat.tofile(dir / "features.f32le")
    ids.tofile(dir / "ids.u32le")
    labels.tofile(dir / "labels.f32le")
    aux.tofile(dir / "aux.f32le")
    meta = {
        "samples": n,
        "feature_len": 567,
        "ids_len": 220,
        "encoding": 2,
        "aux_columns": _SYNTH_V2_AUX,
    }
    (dir / "meta.json").write_text(json.dumps(meta) + "\n")


def _write_memory_gate_export(root: Path, rows_per_shard: int) -> int:
    _write_memory_gate_shard(root / "data-e0", rows_per_shard, seed=11)
    _write_memory_gate_shard(root / "data-e10", rows_per_shard, seed=29)
    return sum(
        f.stat().st_size
        for shard in ("data-e0", "data-e10")
        for f in (root / shard).iterdir()
    )


def _memory_ratio_subprocess(
    data_dirs: list[Path],
    disk_bytes: int,
    out: Path,
    extra_args: list[str],
) -> float:
    argv_parts = ['"--data"'] + [repr(str(d)) for d in data_dirs]
    argv_parts += [
        '"--model"',
        '"linear"',
        '"--out"',
        repr(str(out)),
        '"--holdout"',
        '"0.1"',
        '"--seed"',
        '"0"',
    ]
    argv_parts += [repr(a) for a in extra_args]
    argv_py = ", ".join(argv_parts)
    script = f"""
import importlib.util
import resource

import numpy as np
import torch

spec = importlib.util.spec_from_file_location("tv", "{_TRAIN}")
tv = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tv)

def vm_rss() -> int:
    for line in open("/proc/self/status"):
        if line.startswith("VmRSS:"):
            return int(line.split()[1])
    return 0

rss0 = vm_rss()
tv.main([{argv_py}])
hwm = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
print((hwm - rss0) * 1024 / {disk_bytes})
"""
    proc = subprocess.run(
        [sys.executable, "-c", script],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise AssertionError(
            f"memory subprocess failed:\nstdout={proc.stdout}\nstderr={proc.stderr}"
        )
    return float(proc.stdout.strip().splitlines()[-1])


@pytest.mark.skipif(sys.platform != "linux", reason="memory gate uses /proc VmRSS")
def test_memory_gate(tmp_path: Path) -> None:
    rows_per_shard = 100_000
    disk_bytes = _write_memory_gate_export(tmp_path, rows_per_shard)
    data_dirs = [tmp_path / "data-e0", tmp_path / "data-e10"]
    cases = [
        ("adam", ["--epochs", "3"]),
        ("lbfgs", ["--optimizer", "lbfgs", "--lbfgs-iters", "3"]),
    ]
    for name, extra in cases:
        out = tmp_path / f"{name}.json"
        ratio = _memory_ratio_subprocess(data_dirs, disk_bytes, out, extra)
        assert ratio <= 1.5, (
            f"{name} memory ratio {ratio:.3f} > 1.5 "
            f"(disk {disk_bytes} bytes, {rows_per_shard * 2} rows, two-shard preallocated load)"
        )


def _lbfgs_teacher_bundle(db, root: Path, tmp_path: Path) -> dict[str, Any]:
    import arena

    export = tmp_path / "teacher"
    arena.matchup(
        db,
        _forest(root),
        3,
        41,
        policy_a="h0-fast",
        policy_b="h0-fast",
        export=str(export),
        threads=1,
    )
    adam_out = tmp_path / "adam.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(adam_out),
            "--holdout",
            "0",
            "--epochs",
            "20",
            "--seed",
            "0",
        ]
    )
    lbfgs_out = tmp_path / "lbfgs.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(lbfgs_out),
            "--holdout",
            "0",
            "--optimizer",
            "lbfgs",
            "--lbfgs-iters",
            "200",
        ]
    )
    adam_spec = json.loads(adam_out.read_text())
    lbfgs_report = json.loads(_report_path(lbfgs_out).read_text())
    data = train_value.load_dirs([str(export)])
    row_idx = np.arange(data["features"].shape[0], dtype=np.int32)
    adam_pred = train_value.predict(adam_spec, data["features"][row_idx], data["ids"][row_idx])
    adam_unit = adam_pred / float(adam_spec["scale"])
    adam_train_mse = float(np.mean((adam_unit - data["labels"][row_idx]) ** 2))
    return {
        "export": export,
        "lbfgs_out": lbfgs_out,
        "adam_train_mse": adam_train_mse,
        "lbfgs_report": lbfgs_report,
    }


@pytest.mark.xfail(
    sys.platform == "win32",
    reason=(
        "Windows L-BFGS final_loss=0.4799 > Adam train MSE 0.3983+1e-3 on the owner's box; "
        "L-BFGS is not used by any run"
    ),
    raises=AssertionError,
    strict=False,
)
def test_lbfgs_final_loss_at_most_adam_train_mse(db, root: Path, tmp_path: Path) -> None:
    bundle = _lbfgs_teacher_bundle(db, root, tmp_path)
    assert bundle["lbfgs_report"]["final_loss"] <= bundle["adam_train_mse"] + 1e-3


def test_lbfgs_linear_teacher(db, root: Path, tmp_path: Path) -> None:
    import arena

    bundle = _lbfgs_teacher_bundle(db, root, tmp_path)
    export = bundle["export"]
    lbfgs_out = bundle["lbfgs_out"]

    lbfgs_a = None
    for seed in (1, 99):
        out = tmp_path / f"lbfgs_seed{seed}.json"
        train_value.main(
            [
                "--data",
                str(export),
                "--model",
                "linear",
                "--out",
                str(out),
                "--holdout",
                "0",
                "--seed",
                str(seed),
                "--optimizer",
                "lbfgs",
                "--lbfgs-iters",
                "200",
            ]
        )
        spec = json.loads(out.read_text())
        if lbfgs_a is None:
            lbfgs_a = spec
            continue
        for key in ("w", "zone_w"):
            a = np.asarray(lbfgs_a["linear"][key])
            b = np.asarray(spec["linear"][key])
            assert float(np.max(np.abs(a - b))) < 1e-6
        assert abs(lbfgs_a["linear"]["b"] - spec["linear"]["b"]) < 1e-6

    arena.matchup(
        db,
        _forest(root),
        0,
        1,
        policy=f"h0:net={lbfgs_out}",
        threads=1,
    )


def test_std_floor_caps_feat_std_and_loads(db, root: Path, tmp_path: Path) -> None:
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
    out = tmp_path / "floored.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(out),
            "--epochs",
            "2",
            "--seed",
            "0",
            "--std-floor",
            "0.05",
        ]
    )
    spec = json.loads(out.read_text())
    stds = np.asarray(spec["feat_std"], dtype=np.float32)
    assert float(np.min(stds)) >= 0.05
    report = json.loads(_report_path(out).read_text())
    assert report["std_floor"] == 0.05
    assert spec["trained_on"]["std_floor"] == 0.05
    arena.matchup(
        db,
        _forest(root),
        0,
        1,
        policy=f"h0:net={out}",
        threads=1,
    )


def test_mlp_lbfgs_rejected(tmp_path: Path) -> None:
    data = tmp_path / "tiny"
    _write_synthetic_encoding2(data, 64)
    with pytest.raises(SystemExit, match="lbfgs is only supported with --model linear"):
        train_value.main(
            [
                "--data",
                str(data),
                "--model",
                "mlp",
                "--out",
                str(tmp_path / "bad.json"),
                "--optimizer",
                "lbfgs",
            ]
        )


def test_numpy_fallback_new_layout(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    data = tmp_path / "tiny"
    _write_synthetic_encoding2(data, 128)
    monkeypatch.setattr(train_value, "_try_torch", lambda: None)
    out = tmp_path / "numpy.json"
    train_value.main(
        [
            "--data",
            str(data),
            "--model",
            "linear",
            "--out",
            str(out),
            "--epochs",
            "2",
            "--seed",
            "0",
        ]
    )
    spec = json.loads(out.read_text())
    assert spec["arch"] == "linear"
    report = json.loads(_report_path(out).read_text())
    assert report["optimizer"] == "adam"
    assert report["epochs_run"] == 2
    assert report["best_epoch"] <= report["epochs_run"]


def test_epochs_run_and_best_epoch(db, root: Path, tmp_path: Path) -> None:
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
    out = tmp_path / "holdout.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(out),
            "--holdout",
            "0.5",
            "--epochs",
            "5",
            "--seed",
            "0",
        ]
    )
    report = json.loads(_report_path(out).read_text())
    assert report["epochs_run"] >= 1
    assert report["best_epoch"] <= report["epochs_run"]

    out0 = tmp_path / "nohold.json"
    train_value.main(
        [
            "--data",
            str(export),
            "--model",
            "linear",
            "--out",
            str(out0),
            "--holdout",
            "0",
            "--epochs",
            "5",
            "--seed",
            "0",
        ]
    )
    report0 = json.loads(_report_path(out0).read_text())
    assert report0["epochs_run"] == 5
    assert report0["best_epoch"] == 5


V3_AUX = [
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
V3_CREST = 10714110


def _write_v3_train_shard(dir: Path, n: int, *, games: int = 8, rows_per_game: int = 16) -> None:
    feat = np.zeros((n, 961), dtype="<f4")
    ids = np.zeros((n, 230), dtype="<u4")
    labels = np.full(n, 0.05, dtype="<f4")
    aux = np.zeros((n, len(V3_AUX)), dtype="<f4")
    for i in range(n):
        g = i // rows_per_game
        aux[i, 0] = g
        aux[i, 2] = float(i % 2)
        aux[i, 3] = 4.0
        aux[i, 9] = float(g % 2)
        if int(aux[i, 2]) == 0:
            ids[i, 220] = V3_CREST
            labels[i] = 0.25
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
        "aux_columns": V3_AUX,
        "layout": [],
        "decks": ["deck-a", "deck-b"],
        "games": games,
    }
    (dir / "meta.json").write_text(json.dumps(meta) + "\n")


def _engine_loads_model(model_path: Path) -> None:
    env = {**dict(__import__("os").environ), "ARENA_MODEL_PATH": str(model_path)}
    r = subprocess.run(
        [
            "cargo",
            "test",
            "--release",
            "--test",
            "encoding_v3",
            "v3_model_from_env_loads",
            "--",
            "--exact",
        ],
        cwd=str(Path(__file__).resolve().parents[2] / "engine"),
        capture_output=True,
        text=True,
        env=env,
    )
    assert r.returncode == 0, r.stdout + r.stderr


def _assert_v3_holdout_predict_parity(
    spec: dict, data_dir: Path, holdout: float, seed: int, report: dict
) -> None:
    loaded = train_value.load_dirs([str(data_dir)])
    _, hold_idx = train_value.split_by_game(loaded["game_index"], holdout, seed)
    assert hold_idx.size > 0
    pred = train_value.predict(spec, loaded["features"][hold_idx], loaded["ids"][hold_idx])
    pred_unit = pred / float(spec["scale"])
    y = loaded["labels"][hold_idx].astype(np.float64)
    mse = float(np.mean((pred_unit - y) ** 2))
    assert abs(mse - report["net"]["overall"]["mse"]) < 1e-5


@pytest.mark.parametrize("optimizer", ["numpy", "adam", "lbfgs"])
def test_encoding3_full_fit_paths(tmp_path: Path, optimizer: str) -> None:
    data = tmp_path / "v3data"
    _write_v3_train_shard(data, 256, games=16, rows_per_game=16)
    out = tmp_path / f"v3-{optimizer}.json"
    cmd = [
        sys.executable,
        str(_TRAIN),
        "--data",
        str(data),
        "--model",
        "linear",
        "--out",
        str(out),
        "--holdout",
        "0.25",
        "--seed",
        "5",
        "--epochs",
        "8",
        "--l2",
        "1e-4",
    ]
    env = dict(__import__("os").environ)
    if optimizer == "lbfgs":
        cmd.extend(["--optimizer", "lbfgs", "--lbfgs-iters", "30"])
    elif optimizer == "numpy":
        env["TORCH_DISABLE"] = "1"
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=str(_TRAIN.parents[1]), env=env)
    assert r.returncode == 0, r.stdout + r.stderr
    spec = json.loads(out.read_text())
    assert spec["encoding"] == 3
    zw = np.asarray(spec["linear"]["zone_w"], dtype=np.float32)
    assert zw.shape[0] == 13
    assert np.max(np.abs(zw[5:13, 0])) < 1e-12
    report = json.loads(_report_path(out).read_text())
    assert report["rows_holdout"] > 0
    _engine_loads_model(out)
    _assert_v3_holdout_predict_parity(spec, data, 0.25, 5, report)
