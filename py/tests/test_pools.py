"""Named deck pools: --pool resolution, POOLS.json integrity, meta decks."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import pytest

_PY = Path(__file__).resolve().parents[1]
_REPO = _PY.parent

_mspec = importlib.util.spec_from_file_location("arena_matchup", _PY / "matchup.py")
assert _mspec and _mspec.loader
matchup = importlib.util.module_from_spec(_mspec)
sys.modules["arena_matchup"] = matchup
_mspec.loader.exec_module(matchup)

_sspec = importlib.util.spec_from_file_location("arena_sweep", _PY / "sweep.py")
assert _sspec and _sspec.loader
sweep = importlib.util.module_from_spec(_sspec)
sys.modules.setdefault("arena_sweep", sweep)
_sspec.loader.exec_module(sweep)

SIDECARS = frozenset({"POOLS", "meta-pool"})
REAL = [
    "abyss-p8rfn",
    "afnm-minatodao",
    "elf-neanisu2",
    "ramp-37772",
    "ramp-claywies",
    "royal-nattui",
    "rune-mach15",
]
SYNTHETIC = [
    "abyss-pool",
    "basic-forest",
    "basic-portal",
    "basic-rune",
    "dragon-pool",
    "forest-pool",
    "portal-pool",
    "rune-pool",
    "sword-pool",
]


def _decks_dir() -> Path:
    return _REPO / "oracle" / "decks"


def _pools() -> dict:
    return json.loads((_decks_dir() / "POOLS.json").read_text(encoding="utf-8"))


def _on_disk_stems() -> list[str]:
    return [
        p.stem
        for p in sorted(_decks_dir().glob("*.json"))
        if p.stem not in SIDECARS
    ]


def test_pools_json_every_stem_has_a_file() -> None:
    pools = _pools()
    assert set(pools) >= {"meta", "real", "synthetic", "all"}
    decks_dir = _decks_dir()
    for name, stems in pools.items():
        assert stems, f"pool {name} is empty"
        for stem in stems:
            path = decks_dir / f"{stem}.json"
            assert path.is_file(), f"pool {name}: missing {path.name}"


def test_pools_json_named_sets() -> None:
    pools = _pools()
    assert pools["real"] == REAL
    assert pools["synthetic"] == SYNTHETIC
    assert set(pools["all"]) == set(pools["meta"]) | set(REAL) | set(SYNTHETIC)
    assert set(pools["all"]) == set(_on_disk_stems())
    assert all(s.startswith("meta-") for s in pools["meta"])


def test_pool_resolution_default_is_meta() -> None:
    args = matchup.parse_args([])
    assert args.pool == "meta"
    assert args.decks is None
    name, decks = matchup.resolve_selected_decks(_decks_dir(), args.pool, args.decks)
    assert name == "meta"
    assert list(decks) == _pools()["meta"]


def test_pool_resolution_named() -> None:
    name, decks = matchup.resolve_selected_decks(_decks_dir(), "real", None)
    assert name == "real"
    assert list(decks) == REAL
    name, decks = matchup.resolve_selected_decks(_decks_dir(), "synthetic", None)
    assert name == "synthetic"
    assert list(decks) == SYNTHETIC
    name, decks = matchup.resolve_selected_decks(_decks_dir(), "all", None)
    assert name == "all"
    assert set(decks) == set(_on_disk_stems())


def test_pool_unknown() -> None:
    with pytest.raises(SystemExit) as exc:
        matchup.resolve_deck_stems(_decks_dir(), "no-such-pool", None)
    assert "no-such-pool" in str(exc.value)


def test_pool_and_decks_conflict_matchup() -> None:
    with pytest.raises(SystemExit) as exc:
        matchup.parse_args(["--pool", "meta", "--decks", "basic-forest"])
    assert "--pool" in str(exc.value)
    assert "--decks" in str(exc.value)


def test_pool_and_decks_conflict_sweep() -> None:
    with pytest.raises(SystemExit) as exc:
        sweep.parse_args(
            [
                "--tag",
                "t",
                "--candidates",
                "h0",
                "--pool",
                "meta",
                "--decks",
                "basic-forest",
            ]
        )
    assert "--pool" in str(exc.value)
    assert "--decks" in str(exc.value)


def test_sweep_default_pool_is_meta() -> None:
    args = sweep.parse_args(["--tag", "t", "--candidates", "h0"])
    assert args.pool == "meta"
    assert args.decks is None
    pool_name, decks = sweep.load_sweep_decks(_REPO, args.decks, args.pool)
    assert pool_name == "meta"
    assert list(decks) == _pools()["meta"]


def test_sweep_explicit_decks_skips_default_pool() -> None:
    args = sweep.parse_args(
        ["--tag", "t", "--candidates", "h0", "--decks", "basic-forest", "basic-rune"]
    )
    assert args.pool is None
    pool_name, decks = sweep.load_sweep_decks(_REPO, args.decks, args.pool)
    assert pool_name == "--decks"
    assert list(decks) == ["basic-forest", "basic-rune"]


def test_sweep_smoke_still_two_basic_decks() -> None:
    args = sweep.parse_args(["--tag", "t", "--candidates", "h0", "--smoke"])
    assert args.decks == ["basic-forest", "basic-rune"]
    assert args.pool is None


def test_matchup_sizing_block() -> None:
    block = matchup.format_matchup_sizing("meta", ["a", "b"], 200)
    assert "pool: meta" in block
    assert "decks: 2 (a, b)" in block
    assert "pairs: 4 (2 x 2, mirrors included)" in block
    assert "200 games/pair x 4 =   800" in block


def test_sweep_sizing_prints_pool() -> None:
    args = sweep.parse_args(["--tag", "t", "--candidates", "h0", "--allow-small"])
    pool_name, decks = sweep.load_sweep_decks(_REPO, None, "real")
    sizing = sweep.compute_sizing(args, len(decks), list(decks), pool_name)
    block = sweep.format_sizing_block(sizing)
    assert "pool: real" in block
    assert sizing.n_decks == 7
    assert sizing.pairs == 49


def test_meta_decks_sum_40_and_resolve_under_cards() -> None:
    files = {
        p.stem
        for p in (_REPO / "cards").rglob("*.json")
        if "official" not in p.parts
    }
    meta = sorted(_decks_dir().glob("meta-*.json"))
    meta = [p for p in meta if p.stem != "meta-pool"]
    assert meta, "no meta-*.json decks"
    for path in meta:
        raw = json.loads(path.read_text(encoding="utf-8"))
        assert isinstance(raw, dict), path.name
        total = 0
        for cid, n in raw.items():
            assert cid in files, f"{path.name}: {cid} has no file under cards/"
            total += int(n)
        assert total == 40, f"{path.name} sums to {total}"


def test_existing_sixteen_still_on_disk() -> None:
    for stem in REAL + SYNTHETIC:
        assert (_decks_dir() / f"{stem}.json").is_file()
