"""Learned mulligan: fingerprints, records, fit, data resume."""

from __future__ import annotations

import argparse
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
    # df>=2: regularized-gamma / continued-fraction path.
    assert abs(mulligan_mod.chi2_sf(5.991, 2) - 0.05) < 0.001
    assert abs(mulligan_mod.chi2_sf(11.345, 3) - 0.01) < 0.001
    assert abs(mulligan_mod.chi2_sf(16.812, 6) - 0.01) < 0.001


def _class_obs(
    opponent_class: str,
    kept: bool,
    won: int,
    card: str = "10934110",
) -> mulligan_mod.Observation:
    filler = ["10001110", "10021120", "10031110"]
    hand = [card, filler[0], filler[1], filler[2]]
    swap = [not kept if c == card else False for c in hand]
    return mulligan_mod.Observation(
        deck="synthetic",
        seat="first",
        opponent_deck="opp",
        opponent_class=opponent_class,
        hand=hand,
        swap=swap,
        won=won,
    )


def test_class_heterogeneity_requires_both_groups() -> None:
    """9 kept / 1 sent back must not count toward the chi² class set."""
    card = "10934110"
    obs: list[mulligan_mod.Observation] = []
    for _ in range(9):
        obs.append(_class_obs("runecraft", kept=True, won=1, card=card))
    obs.append(_class_obs("runecraft", kept=False, won=0, card=card))
    for i in range(10):
        obs.append(_class_obs("swordcraft", kept=True, won=1, card=card))
    for i in range(10):
        obs.append(_class_obs("swordcraft", kept=False, won=0, card=card))
    classes = mulligan_mod.class_effects(obs, "synthetic", card)
    used = mulligan_mod.heterogeneity_classes(classes, min_n=10)
    assert "runecraft" not in used
    assert used == ["swordcraft"]


def test_pool_meta_from_run_decks_not_matrix() -> None:
    run = {
        "argv": [
            "mulligan.py",
            "data",
            "--decks",
            "meta-rune-test-subject",
            "meta-sword-rally",
            "--games-per-pair",
            "4",
        ]
    }
    pool = mulligan_mod.pool_meta_from_run(run, ["fallback"])
    assert pool == ["meta-rune-test-subject", "meta-sword-rally"]
    assert not isinstance(pool, dict)


def test_pool_meta_from_run_pool_name() -> None:
    run = {"argv": ["mulligan.py", "data", "--pool", "meta", "--games-per-pair", "4"]}
    assert mulligan_mod.pool_meta_from_run(run, []) == "meta"


def test_fit_uses_decks_from_chunks_only(db, arena, tmp_path: Path, root: Path) -> None:
    tag = "fit-decks-only"
    tag_dir = tmp_path / tag
    subprocess.run(
        [
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
            "2",
            "--chunks",
            "1",
            "--seed",
            "42",
            "--threads",
            "1",
        ],
        check=True,
        cwd=root,
    )
    subprocess.run(
        [
            sys.executable,
            str(_PY / "mulligan.py"),
            "fit",
            "--tag",
            tag,
            "--root",
            str(tmp_path),
            "--min-n",
            "1",
        ],
        check=True,
        cwd=root,
    )
    table = json.loads((tag_dir / "table.json").read_text(encoding="utf-8"))
    deck_names = {entry["name"] for entry in table["decks"].values()}
    assert deck_names == {"meta-rune-test-subject", "meta-sword-rally"}
    pool = table["meta"]["pool"]
    assert pool == ["meta-rune-test-subject", "meta-sword-rally"]
    assert not isinstance(pool, dict)


def test_fit_json_includes_class_effects(
    db, arena, tmp_path: Path, root: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    card = "10934110"
    obs: list[mulligan_mod.Observation] = []
    rng = random.Random(3)
    for cls, keep_rate in (("runecraft", 0.85), ("swordcraft", 0.15)):
        for _ in range(80):
            kept = rng.random() < keep_rate
            obs.append(_class_obs(cls, kept=kept, won=1 if kept else 0, card=card))
    tag = "fit-json-class"
    tag_dir = tmp_path / tag
    tag_dir.mkdir()
    (tag_dir / "chunk-0.json").write_text(
        json.dumps({"policy_a": "h0:mull=random", "matrix": {"synthetic": {}}}),
        encoding="utf-8",
    )
    monkeypatch.setattr(
        mulligan_mod,
        "load_deck_files",
        lambda _decks_dir, names: {n: {card: 3, "10001110": 37} for n in names},
    )
    monkeypatch.setattr(mulligan_mod, "observations_from_chunks", lambda *_a, **_k: obs)
    args = argparse.Namespace(
        tag=tag,
        z=1.5,
        min_n=30,
        root=str(tmp_path),
        publish=False,
        publish_remote="origin",
        publish_branch="results",
        publish_dir=None,
        _argv=["fit"],
    )
    mulligan_mod.cmd_fit(args)
    fit = json.loads((tag_dir / "fit.json").read_text(encoding="utf-8"))
    sample = next(c for c in fit["cards"] if c["card"] == card)
    assert "class_effects" in sample
    assert "class_chi2_p" in sample
    assert "runecraft" in sample["class_effects"]


def test_copy_tag_artifacts_explicit_names(tmp_path: Path) -> None:
    import gzip

    from runlib import copy_tag_artifacts

    tag_dir = tmp_path / "tag"
    tag_dir.mkdir()
    csv_path = tag_dir / "observations.csv.gz"
    payload = b"deck,seat\nx,first\n"
    with gzip.open(csv_path, "wb") as f:
        f.write(payload)
    for name in ("RUN.json", "FIT.md", "fit.json", "table.json", "extra.bin"):
        (tag_dir / name).write_text(f"{name}\n", encoding="utf-8")

    dest = tmp_path / "dest"
    copy_tag_artifacts(tag_dir, dest, names=mulligan_mod.PUBLISH_FILES)
    for name in sorted(mulligan_mod.PUBLISH_FILES):
        assert (dest / name).is_file(), f"missing {name}"
    assert not (dest / "extra.bin").exists()
    assert (dest / "observations.csv.gz").read_bytes() == csv_path.read_bytes()


def test_publish_fit_includes_all_files(tmp_path: Path, root: Path) -> None:
    import gzip
    import subprocess as sp

    from runlib import publish_tag

    tag = "pub-test"
    tag_dir = tmp_path / tag
    tag_dir.mkdir()
    csv_path = tag_dir / "observations.csv.gz"
    payload = b"deck,seat\nx,first\n"
    with gzip.open(csv_path, "wb") as f:
        f.write(payload)
    for name in ("RUN.json", "FIT.md", "fit.json", "table.json"):
        (tag_dir / name).write_text(f"{name}\n", encoding="utf-8")

    def git_ident(repo: Path) -> None:
        sp.run(["git", "config", "user.email", "test@example.com"], check=True, cwd=repo)
        sp.run(["git", "config", "user.name", "test"], check=True, cwd=repo)

    bare = tmp_path / "bare.git"
    sp.run(["git", "init", "--bare", str(bare)], check=True, cwd=tmp_path)
    mini = tmp_path / "mini"
    mini.mkdir()
    sp.run(["git", "init"], check=True, cwd=mini)
    git_ident(mini)
    sp.run(["git", "remote", "add", "origin", str(bare)], check=True, cwd=mini)
    sp.run(["git", "commit", "--allow-empty", "-m", "init"], check=True, cwd=mini)
    publish_dir = tmp_path / "results-wt"
    sp.run(
        ["git", "clone", str(bare), str(publish_dir)],
        check=True,
        cwd=tmp_path,
    )
    git_ident(publish_dir)
    sp.run(["git", "checkout", "--orphan", "results"], check=True, cwd=publish_dir)
    sp.run(["git", "commit", "--allow-empty", "-m", "init results"], check=True, cwd=publish_dir)
    sp.run(["git", "push", "-u", "origin", "results"], check=True, cwd=publish_dir)
    log = tag_dir / "publish.txt"
    publish_tag(
        mini,
        publish_dir,
        "origin",
        "results",
        tag_dir,
        tag,
        log,
        artifact_names=mulligan_mod.PUBLISH_FILES,
    )
    sp.run(
        ["git", "clone", "-b", "results", str(bare), str(tmp_path / "clone")],
        check=True,
        cwd=tmp_path,
    )
    published = tmp_path / "clone" / tag
    for name in sorted(mulligan_mod.PUBLISH_FILES):
        assert (published / name).is_file(), f"missing published file {name}"
    assert (published / "observations.csv.gz").read_bytes() == csv_path.read_bytes()


def _cards_for_shrink_deck(prefix: str) -> list[tuple[str, int]]:
    out: list[tuple[str, int]] = []
    for i in range(10):
        out.append((f"{prefix}{i:03d}110", 2))
    for i in range(10):
        out.append((f"{prefix}{i:03d}310", 5))
    return out


def _write_shrink_run(
    td: Path,
    tag_suffix: int,
    decks: dict[str, list[tuple[str, int]]],
) -> None:
    import csv
    import gzip

    cards_fit: list[dict] = []
    table_decks: dict[str, dict] = {}
    for deck, cards in decks.items():
        fp = f"fp_{deck}"
        first: dict[str, bool] = {}
        for cid, cost in cards:
            rk = cost < 4
            cards_fit.append(
                {"deck": deck, "card": cid, "copies": 3, "cost": cost, "rule_keep": rk}
            )
            first[cid] = rk
        table_decks[fp] = {"name": deck, "first": first, "second": dict(first)}
    td.mkdir(parents=True, exist_ok=True)
    (td / "fit.json").write_text(json.dumps({"cards": cards_fit}), encoding="utf-8")
    (td / "table.json").write_text(json.dumps({"version": 1, "decks": table_decks}), encoding="utf-8")
    rows: list[dict[str, str | int]] = []
    obs_id = 0
    for deck, cards in decks.items():
        deck_ids = [c for c, _ in cards]
        opp = "deck-b" if deck == "deck-a" else "deck-a"
        for seat in ("first", "second"):
            for cid, _cost in cards:
                fillers = [c for c in deck_ids if c != cid][:3]
                for kept, won_p in ((True, 0.72), (False, 0.38)):
                    for rep in range(8):
                        hand = [cid] + fillers
                        swap = ["0", "1", "1", "1"]
                        swap[0] = "0" if kept else "1"
                        won = 1 if (rep + obs_id + tag_suffix) % 10 < int(won_p * 10) else 0
                        rows.append(
                            {
                                "deck": deck,
                                "seat": seat,
                                "opponent_deck": opp,
                                "opponent_class": "neutral",
                                "hand": ";".join(hand),
                                "swap": ";".join(swap),
                                "won": won,
                            }
                        )
                        obs_id += 1
    with gzip.open(td / "observations.csv.gz", "wt", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)


SHRINK_EXPECTED_PRIOR = {
    "rule_keep": {
        "cells": 20,
        "mu": 0.327656,
        "tau_seat": 0.0,
        "tau_shared": 0.097584,
    },
    "rule_send_back": {
        "cells": 20,
        "mu": 0.4,
        "tau_seat": 0.0,
        "tau_shared": 0.030619,
    },
}


def test_shrink_two_runs(tmp_path: Path) -> None:
    decks = {
        "deck-a": _cards_for_shrink_deck("10"),
        "deck-b": _cards_for_shrink_deck("20"),
    }
    for i, tag in enumerate(("run0", "run1")):
        _write_shrink_run(tmp_path / tag, i, decks)
    out = tmp_path / "out.json"
    mulligan_mod.cmd_shrink(
        argparse.Namespace(
            tags=["run0", "run1"],
            root=str(tmp_path),
            min_n=5,
            out=str(out),
            command="shrink",
            _argv=[],
        )
    )
    table = json.loads(out.read_text(encoding="utf-8"))
    prior = table["meta"]["prior"]
    for key, expected in SHRINK_EXPECTED_PRIOR.items():
        for field, val in expected.items():
            assert abs(prior[key][field] - val) < 1e-9, (key, field, prior[key][field], val)
    keeps: list[tuple[str, str, str, bool]] = []
    for fp in sorted(table["decks"]):
        entry = table["decks"][fp]
        for seat in ("first", "second"):
            for cid in sorted(entry[seat], key=int):
                keeps.append((entry["name"], seat, cid, entry[seat][cid]))
    assert len(keeps) == 80
    assert all(k for *_, k in keeps)


def test_shrink_deck_list_mismatch(tmp_path: Path) -> None:
    decks_a = {"deck-a": _cards_for_shrink_deck("10")}
    decks_b = {"deck-a": _cards_for_shrink_deck("10"), "deck-b": _cards_for_shrink_deck("20")}
    _write_shrink_run(tmp_path / "run0", 0, decks_a)
    _write_shrink_run(tmp_path / "run1", 1, decks_b)
    out = tmp_path / "out.json"
    with pytest.raises(SystemExit, match="deck lists differ"):
        mulligan_mod.cmd_shrink(
            argparse.Namespace(
                tags=["run0", "run1"],
                root=str(tmp_path),
                min_n=5,
                out=str(out),
                command="shrink",
                _argv=[],
            )
        )


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
