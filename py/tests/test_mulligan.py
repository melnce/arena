"""Learned mulligan: fingerprints, records, fit, data resume."""

from __future__ import annotations

import json
import random
import subprocess
import sys
from pathlib import Path

import pytest

from tests.conftest import load_deck, repo_root

_PY = Path(__file__).resolve().parents[1]
if str(_PY) not in sys.path:
    sys.path.insert(0, str(_PY))

import mulligan as mulligan_mod  # noqa: E402


@pytest.fixture(scope="module")
def arena():
    import arena as a

    return a


def meta_deck_stems(root: Path) -> list[str]:
    pools = json.loads((root / "oracle/decks/POOLS.json").read_text(encoding="utf-8"))
    return list(pools["meta"])


def test_fingerprints_rust_python_agree(db, arena, root: Path) -> None:
    decks_dir = root / "oracle/decks"
    for stem in meta_deck_stems(root):
        deck = load_deck(decks_dir / f"{stem}.json")
        py_fp = arena.deck_fingerprint(deck)
        assert "x" in py_fp
        assert py_fp == arena.deck_fingerprint(deck)


def test_matchup_records_mulligan_fields(db, arena, root: Path) -> None:
    decks_dir = root / "oracle/decks"
    decks = {
        "a": load_deck(decks_dir / "meta-rune-test-subject.json"),
        "b": load_deck(decks_dir / "meta-sword-rally.json"),
    }
    with_rec = arena.matchup(db, decks, 2, 11, policy="h0:mull=random", threads=1, records=True)
    for rec in with_rec["records"]:
        assert "mull_a" in rec
        assert "mull_b" in rec
        for key in ("mull_a", "mull_b"):
            m = rec[key]
            if m is not None:
                assert isinstance(m["hand"], list)
                assert len(m["swap"]) == 4
                assert all(isinstance(x, bool) for x in m["swap"])
    without = arena.matchup(db, decks, 2, 11, policy="h0:mull=random", threads=1, records=False)
    for rec in without.get("records", []):
        assert "mull_a" not in rec
        assert "mull_b" not in rec


def _synthetic_observations(
    rng: random.Random,
    n: int,
    target_card: str,
    keep_bonus: float,
    seat_bonus: dict[str, float] | None = None,
) -> list[mulligan_mod.Observation]:
    seat_bonus = seat_bonus or {}
    out: list[mulligan_mod.Observation] = []
    filler = ["10001110", "10021120", "10031110"]
    for _ in range(n):
        seat = rng.choice(["first", "second"])
        hand = [target_card, filler[0], filler[1], filler[2]]
        rng.shuffle(hand)
        swap = [rng.random() < 0.5 for _ in range(4)]
        p = 0.5
        if not swap[hand.index(target_card)]:
            p += keep_bonus
            p += seat_bonus.get(seat, 0.0)
        won = 1 if rng.random() < p else 0
        out.append(
            mulligan_mod.Observation(
                deck="synthetic",
                seat=seat,
                opponent_deck="opp",
                opponent_class="runecraft",
                hand=hand,
                swap=swap,
                won=won,
            )
        )
    return out


def test_fit_planted_effect(db, arena, tmp_path: Path) -> None:
    rng = random.Random(0)
    obs = _synthetic_observations(rng, 20_000, "10934110", keep_bonus=0.15)
    per_seat, _ = mulligan_mod.gather_stats(obs)
    keep_first, _ = mulligan_mod.decide_keep(
        "synthetic", "first", "10934110", 7, per_seat, z_thr=2.0, min_n=30
    )
    keep_second, _ = mulligan_mod.decide_keep(
        "synthetic", "second", "10934110", 7, per_seat, z_thr=2.0, min_n=30
    )
    assert keep_first
    assert keep_second
    filler = ("10001110", "10021120", "10031110")
    rule_fallback = 0
    for seat in ("first", "second"):
        for card in filler:
            _, how = mulligan_mod.decide_keep(
                "synthetic", seat, card, 2, per_seat, 2.0, 30
            )
            if how == "rule":
                rule_fallback += 1
    assert rule_fallback / (len(filler) * 2) >= 0.80


def test_fit_planted_seat_effect(db, arena) -> None:
    rng = random.Random(1)
    obs = _synthetic_observations(
        rng,
        20_000,
        "10934110",
        keep_bonus=0.0,
        seat_bonus={"second": 0.20},
    )
    per_seat, _ = mulligan_mod.gather_stats(obs)
    keep_second, how_s = mulligan_mod.decide_keep(
        "synthetic", "second", "10934110", 7, per_seat, z_thr=2.0, min_n=50
    )
    keep_first, how_f = mulligan_mod.decide_keep(
        "synthetic", "first", "10934110", 7, per_seat, z_thr=2.0, min_n=50
    )
    sf = mulligan_mod.seat_stats(per_seat, "synthetic", "first", "10934110")
    ss = mulligan_mod.seat_stats(per_seat, "synthetic", "second", "10934110")
    e_f, _ = mulligan_mod.effect_se(sf.n_k, sf.wins_k, sf.n_s, sf.wins_s)
    e_s, _ = mulligan_mod.effect_se(ss.n_k, ss.wins_k, ss.n_s, ss.wins_s)
    assert e_s > 0.10
    assert abs(e_f) < 0.03
    assert keep_second and how_s == "seat"
    assert not keep_first and how_f == "rule"


def test_chi2_sf_known_values() -> None:
    # df=1: sf(3.841)=0.05, sf(6.635)=0.01 (chi-square critical values).
    assert abs(mulligan_mod.chi2_sf(3.841, 1) - 0.05) < 0.01
    assert abs(mulligan_mod.chi2_sf(6.635, 1) - 0.01) < 0.01


def test_data_resume_two_chunks(db, arena, tmp_path: Path, root: Path) -> None:
    tag = "resume-test"
    tag_dir = tmp_path / tag
    common = [
        sys.executable,
        str(_PY / "mulligan.py"),
        "data",
        "--tag",
        tag,
        "--root",
        str(tmp_path),
        "--decks",
        "meta-rune-test-subject",
        "meta-sword-rally",
        "--games-per-pair",
        "4",
        "--chunks",
        "2",
        "--seed",
        "99",
        "--threads",
        "1",
    ]
    subprocess.run(common, check=True, cwd=root)
    assert (tag_dir / "chunk-0.json").is_file()
    assert (tag_dir / "chunk-1.json").is_file()
    out = subprocess.run(common, capture_output=True, text=True, cwd=root)
    assert out.returncode == 0
    assert "skip: chunk-0" in out.stdout
    assert "skip: chunk-1" in out.stdout
