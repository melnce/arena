"""Export shards from `arena.matchup(..., export=dir)`."""

from __future__ import annotations

import importlib.util
from pathlib import Path

import pytest

np = pytest.importorskip("numpy")

_SAMPLES = Path(__file__).resolve().parents[1] / "samples.py"
_spec = importlib.util.spec_from_file_location("arena_samples", _SAMPLES)
assert _spec and _spec.loader
samples = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(samples)


def _forest(root: Path) -> dict[str, dict[str, int]]:
    import json

    path = root / "oracle" / "decks" / "basic-forest.json"
    return {"basic-forest": {str(k): int(v) for k, v in json.loads(path.read_text()).items()}}


def _sorted(data: dict) -> dict:
    aux = data["aux"]
    order = np.lexsort((aux[:, 2], aux[:, 1], aux[:, 0]))
    return {
        "features": data["features"][order],
        "ids": data["ids"][order],
        "labels": data["labels"][order],
        "aux": data["aux"][order],
        "meta": data["meta"],
        "aux_columns": data["aux_columns"],
    }


def test_export_determinism_and_byte_layout(db, root: Path, tmp_path: Path) -> None:
    import arena

    decks = _forest(root)
    kwargs = dict(
        db=db,
        decks=decks,
        games=2,
        seed=17,
        policy_a="h0-fast",
        policy_b="h0-fast",
        records=True,
        threads=1,
    )
    tmp1 = tmp_path / "e1"
    tmp2 = tmp_path / "e2"
    with_export_1 = arena.matchup(**kwargs, export=str(tmp1))
    with_export_2 = arena.matchup(**kwargs, export=str(tmp2))
    without = arena.matchup(**kwargs)
    assert with_export_1["records"] == without["records"]
    assert with_export_2["records"] == without["records"]
    assert "export" not in without

    n = int(with_export_1["export"]["samples"])
    assert n == sum(int(r["actions"]) for r in with_export_1["records"])
    assert (tmp1 / "features.f32le").stat().st_size == n * 545 * 4
    assert (tmp1 / "ids.u32le").stat().st_size == n * 220 * 4
    assert (tmp1 / "labels.f32le").stat().st_size == n * 4
    assert (tmp1 / "aux.f32le").stat().st_size == n * 10 * 4

    a = _sorted(samples.load(tmp1))
    b = _sorted(samples.load(tmp2))
    np.testing.assert_array_equal(a["features"], b["features"])
    np.testing.assert_array_equal(a["ids"], b["ids"])
    np.testing.assert_array_equal(a["labels"], b["labels"])
    np.testing.assert_array_equal(a["aux"], b["aux"])
    assert a["meta"]["samples"] == n
    assert set(np.unique(a["labels"])).issubset({-1.0, 0.0, 1.0})

    aux = a["aux"]
    labels = a["labels"]
    gi, side = aux[:, 0], aux[:, 2]
    for g in np.unique(gi):
        a_lab = labels[(gi == g) & (side == 0)]
        b_lab = labels[(gi == g) & (side == 1)]
        assert a_lab.size == 0 or np.all(a_lab == a_lab[0])
        assert b_lab.size == 0 or np.all(b_lab == b_lab[0])
        if a_lab.size and b_lab.size:
            assert a_lab[0] == -b_lab[0] or (a_lab[0] == 0.0 and b_lab[0] == 0.0)


def test_export_none_writes_nothing(db, root: Path, tmp_path: Path) -> None:
    import arena

    empty = tmp_path / "empty"
    empty.mkdir()
    result = arena.matchup(
        db,
        _forest(root),
        1,
        3,
        policy_a="h0-fast",
        policy_b="h0-fast",
        threads=1,
    )
    assert list(empty.iterdir()) == []
    assert "export" not in result


def test_export_epsilon_one_is_random(db, root: Path, tmp_path: Path) -> None:
    import arena

    decks = _forest(root)
    kwargs = dict(
        db=db,
        decks=decks,
        games=2,
        seed=17,
        policy_a="h0-fast",
        policy_b="h0-fast",
        records=True,
        threads=1,
    )
    zero = arena.matchup(**kwargs, export=str(tmp_path / "z"), export_epsilon=0.0)
    ones = arena.matchup(**kwargs, export=str(tmp_path / "o"), export_epsilon=1.0)
    data = samples.load(tmp_path / "o")
    random_col = data["aux_columns"].index("random")
    assert np.all(data["aux"][:, random_col] == 1.0)
    assert ones["records"] != zero["records"]
