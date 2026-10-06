"""sweep.py --early-stop: probabilities, schedule, smoke, operating characteristics."""

from __future__ import annotations

import importlib.util
import json
import math
import os
import random
import subprocess
import sys
from pathlib import Path

import pytest

_REPO = Path(__file__).resolve().parents[2]
_PY = _REPO / "py"
_SWEEP = _PY / "sweep.py"

_spec = importlib.util.spec_from_file_location("arena_sweep", _SWEEP)
assert _spec and _spec.loader
sweep = importlib.util.module_from_spec(_spec)
sys.modules["arena_sweep"] = sweep
_spec.loader.exec_module(sweep)

_runlib_spec = importlib.util.spec_from_file_location("arena_runlib", _PY / "runlib.py")
assert _runlib_spec and _runlib_spec.loader
runlib = importlib.util.module_from_spec(_runlib_spec)
sys.modules["arena_runlib"] = runlib
_runlib_spec.loader.exec_module(runlib)

FAST = sweep.FAST


def _brute_class_probs(
    w_main: int,
    n_main: int,
    r_main: int,
    w_rev: int,
    n_rev: int,
    r_rev: int,
    thresholds: list[float],
    combine_main: tuple[int, int] | None = None,
    combine_rev: tuple[int, int] | None = None,
) -> dict[tuple[bool, int], float]:
    if combine_main:
        w_main += combine_main[0]
        n_main += combine_main[1]
    if combine_rev:
        w_rev += combine_rev[0]
        n_rev += combine_rev[1]
    probs: dict[tuple[bool, int], float] = {}
    for km in range(r_main + 1):
        pm = math.exp(runlib.beta_binomial_log_pmf(km, r_main, w_main, n_main))
        wf_main = w_main + km
        nf_main = n_main + r_main
        for kr in range(r_rev + 1):
            pr = math.exp(runlib.beta_binomial_log_pmf(kr, r_rev, w_rev, n_rev))
            wf_rev = w_rev + kr
            nf_rev = n_rev + r_rev
            cls = runlib.decision_class(wf_main, nf_main, wf_rev, nf_rev, thresholds)
            probs[cls] = probs.get(cls, 0.0) + pm * pr
    return probs


def test_exact_probabilities_small_arms() -> None:
    thresholds = [0.51]
    cases = [
        (12, 20, 28, 8, 12, 8, None, None),
        (12, 20, 28, 8, 12, 8, (30, 50), (20, 40)),
    ]
    for w_m, n_m, r_m, w_r, n_r, r_r, c_m, c_r in cases:
        got = runlib.class_probabilities(
            w_m, n_m, r_m, w_r, n_r, r_r, thresholds, combine_main=c_m, combine_rev=c_r
        )
        want = _brute_class_probs(
            w_m, n_m, r_m, w_r, n_r, r_r, thresholds, combine_main=c_m, combine_rev=c_r
        )
        assert set(got) == set(want)
        for cls in want:
            assert got[cls] == pytest.approx(want[cls], abs=1e-12)


def test_exact_probabilities_standard_sizing_sums_to_one() -> None:
    pairs = 256
    main_games = 16 * pairs
    rev_games = 8 * pairs
    w_main, n_main, r_main = 800, 1200, main_games - 1200
    w_rev, n_rev, r_rev = 400, 600, rev_games - 600
    probs = runlib.class_probabilities(
        w_main, n_main, r_main, w_rev, n_rev, r_rev, [0.51]
    )
    assert sum(probs.values()) == pytest.approx(1.0, abs=1e-9)


def test_chunk_schedule() -> None:
    assert runlib.chunk_arm_schedule(8, 4, 2) == [
        ("main", 0),
        ("reverse", 0),
        ("main", 1),
        ("main", 2),
        ("reverse", 1),
        ("main", 3),
    ]
    assert runlib.chunk_arm_schedule(16, 8, 4) == runlib.chunk_arm_schedule(8, 4, 2)


def test_chunk_refusals() -> None:
    with pytest.raises(SystemExit):
        runlib.validate_stop_chunk(3, 8, 4)
    with pytest.raises(SystemExit):
        runlib.validate_stop_chunk(2, 9, 4)
    with pytest.raises(SystemExit):
        runlib.validate_stop_chunk(2, 8, 5)


def _simulate_final_decision(
    true_rate: float,
    *,
    seed: int,
    pairs: int = 256,
    final_games: int = 16,
    final_reverse: int = 8,
    chunk: int = 2,
    gamma: float = 0.02,
    threshold: float = 0.51,
    min_games: int = 1024,
) -> tuple[bool, float]:
    """Return (decision_differs_from_full, share_of_final_played)."""
    rng = random.Random(seed)
    planned_main = final_games * pairs
    planned_rev = final_reverse * pairs
    schedule = runlib.chunk_arm_schedule(final_games, final_reverse, chunk)

    def play_all() -> tuple[int, int, int, int]:
        wm = nm = wr = nr = 0
        for arm, _j in schedule:
            n_chunk = chunk * pairs
            wins = sum(1 for _ in range(n_chunk) if rng.random() < true_rate)
            if arm == "main":
                wm += wins
                nm += n_chunk
            else:
                wr += wins
                nr += n_chunk
        return wm, nm, wr, nr

    fw_m, fn_m, fw_r, fn_r = play_all()
    full_cls = runlib.decision_class(fw_m, fn_m, fw_r, fn_r, [threshold])

    rng = random.Random(seed)
    w_main = n_main = w_rev = n_rev = 0
    stopped = False
    games_played = 0
    gamma_cut = 1.0 - gamma
    settled_cls: tuple[bool, int] | None = None
    for arm, _j in schedule:
        n_chunk = chunk * pairs
        wins = sum(1 for _ in range(n_chunk) if rng.random() < true_rate)
        if arm == "main":
            w_main += wins
            n_main += n_chunk
        else:
            w_rev += wins
            n_rev += n_chunk
        games_played += n_chunk
        if stopped:
            continue
        r_main = planned_main - n_main
        r_rev = planned_rev - n_rev
        probs = runlib.class_probabilities(
            w_main, n_main, r_main, w_rev, n_rev, r_rev, [threshold]
        )
        best_cls = max(probs, key=lambda k: probs[k])
        best_p = probs[best_cls]
        if (
            games_played >= min_games
            and (n_rev > 0 or planned_rev == 0)
            and best_p >= gamma_cut
        ):
            stopped = True
            settled_cls = best_cls
            break
    if settled_cls is not None:
        early_cls = settled_cls
    else:
        early_cls = runlib.decision_class(w_main, n_main, w_rev, n_rev, [threshold])
    share = games_played / (planned_main + planned_rev)
    return early_cls != full_cls, share


def test_operating_characteristics(capsys: pytest.CaptureFixture[str]) -> None:
    rates = [0.46, 0.50, 0.52]
    runs = 400
    rows: list[tuple[float, float, float]] = []
    for rate in rates:
        diffs = 0
        shares: list[float] = []
        for i in range(runs):
            diff, share = _simulate_final_decision(rate, seed=10_000 + i)
            diffs += int(diff)
            shares.append(share)
        diff_rate = diffs / runs
        mean_share = sum(shares) / len(shares)
        rows.append((rate, mean_share, diff_rate))
        assert diff_rate <= 0.03, f"rate {rate}: diff {diff_rate:.3f}"
        if rate <= 0.46:
            assert mean_share <= 0.50, f"rate {rate}: share {mean_share:.3f}"
        if rate <= 0.50:
            assert mean_share <= 0.85, f"rate {rate}: share {mean_share:.3f}"
    print("\noperating characteristics (256 pairs, chunk 2, γ=0.02, threshold 0.51):")
    print("| true rate | mean share final | decision differs |")
    print("|---|---:|---:|")
    for rate, share, diff in rows:
        print(f"| {rate:.2f} | {share:.3f} | {100 * diff:.1f} % |")
    captured = capsys.readouterr()
    assert "operating characteristics" in captured.out


def _run_sweep(args: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.setdefault("GIT_AUTHOR_NAME", "arena-sweep-test")
    env.setdefault("GIT_AUTHOR_EMAIL", "arena-sweep-test@example.com")
    env.setdefault("GIT_COMMITTER_NAME", "arena-sweep-test")
    env.setdefault("GIT_COMMITTER_EMAIL", "arena-sweep-test@example.com")
    return subprocess.run(
        [sys.executable, str(_SWEEP), *args],
        cwd=cwd or _REPO,
        text=True,
        capture_output=True,
        env=env,
    )


@pytest.fixture(scope="module")
def es_smoke(tmp_path_factory: pytest.TempPathFactory):
    pytest.importorskip("arena")
    root = tmp_path_factory.mktemp("es")
    base_argv = [
        "--smoke",
        "--tag",
        "es1",
        "--baseline",
        FAST,
        "--candidates",
        "h0:depth=2,beam=2,k=1,nodes=160,value=v0,tt=0",
        "h0:depth=3,beam=2,k=1,nodes=80,value=v0,tt=0",
        "--root",
        str(root),
        "--final-games",
        "4",
        "--final-reverse",
        "2",
        "--skip-publish",
    ]
    first = _run_sweep(base_argv)
    assert first.returncode == 0, first.stderr or first.stdout
    yield {"root": root, "tag": root / "es1", "base_argv": base_argv}


def test_early_stop_min_games_never_stops(es_smoke) -> None:
    tag: Path = es_smoke["tag"]
    stem = "c01"
    plain = _run_sweep(
        [
            *es_smoke["base_argv"],
            "--force",
            "--only",
            "final",
        ]
    )
    assert plain.returncode == 0, plain.stderr or plain.stdout
    plain_main = json.loads((tag / f"{stem}-final.json").read_text())
    argv = [
        *es_smoke["base_argv"],
        "--early-stop",
        "--stop-min-games",
        "99999",
        "--force",
        "--only",
        "final",
    ]
    r = _run_sweep(argv)
    assert r.returncode == 0, r.stderr or r.stdout
    run = json.loads((tag / "RUN.json").read_text())
    assert run["early_stop"]["finalists"][stem]["stopped"] is False
    es_main = json.loads((tag / f"{stem}-final.json").read_text())
    assert plain_main["summary"]["decisive"] == es_main["summary"]["decisive"]
    assert plain_main["summary"]["policy_a_win_rate"] == pytest.approx(
        es_main["summary"]["policy_a_win_rate"]
    )


def test_early_stop_stops_and_resumes(es_smoke) -> None:
    tag: Path = es_smoke["tag"]
    stem = "c01"
    argv = [
        *es_smoke["base_argv"],
        "--early-stop",
        "--stop-gamma",
        "0.49",
        "--stop-min-games",
        "1",
        "--seed",
        "4242",
        "--force",
        "--only",
        "final",
    ]
    r = _run_sweep(argv)
    assert r.returncode == 0, r.stderr or r.stdout
    summary_r = _run_sweep(
        [
            *es_smoke["base_argv"],
            "--early-stop",
            "--stop-gamma",
            "0.49",
            "--stop-min-games",
            "1",
            "--seed",
            "4242",
            "--only",
            "summary",
            "--force",
        ]
    )
    assert summary_r.returncode == 0, summary_r.stderr or summary_r.stdout
    run = json.loads((tag / "RUN.json").read_text())
    assert run["early_stop"]["finalists"][stem]["stopped"] is True
    assert "early stop" in (tag / "SUMMARY.md").read_text()
    parts_before = sorted(tag.glob(f"{stem}-*.part*.json"))
    assert parts_before
    second = _run_sweep(argv)
    assert second.returncode == 0
    assert sorted(tag.glob(f"{stem}-*.part*.json")) == parts_before


def test_stop_combine_table(tmp_path: Path) -> None:
    pytest.importorskip("arena")
    results = tmp_path / "results"
    base = [
        "--smoke",
        "--tag",
        "first",
        "--baseline",
        FAST,
        "--candidates",
        "h0:depth=2,beam=2,k=1,nodes=160,value=v0,tt=0",
        "--root",
        str(results),
        "--final-games",
        "4",
        "--final-reverse",
        "2",
        "--seed",
        "7",
        "--skip-publish",
    ]
    assert _run_sweep(base).returncode == 0
    r2 = _run_sweep(
        [
            "--smoke",
            "--tag",
            "confirm",
            "--baseline",
            FAST,
            "--candidates",
            "h0:depth=2,beam=2,k=1,nodes=160,value=v0,tt=0",
            "--root",
            str(results),
            "--final-games",
            "4",
            "--final-reverse",
            "2",
            "--early-stop",
            "--stop-combine",
            "first",
            "--stop-min-games",
            "99999",
            "--seed",
            "8",
            "--skip-publish",
        ]
    )
    assert r2.returncode == 0, r2.stderr or r2.stdout
    confirm = results / "confirm"
    summary = (confirm / "SUMMARY.md").read_text()
    assert "## combined with first" in summary
    first_main = json.loads((results / "first" / "c01-final.json").read_text())
    first_rev = json.loads((results / "first" / "c01-reverse.json").read_text())
    conf_main = json.loads((confirm / "c01-final.json").read_text())
    conf_rev = json.loads((confirm / "c01-reverse.json").read_text())
    wm1, nm1 = runlib.arm_candidate_wins(first_main, reverse=False)
    wm2, nm2 = runlib.arm_candidate_wins(conf_main, reverse=False)
    wr1, nr1 = runlib.arm_candidate_wins(first_rev, reverse=True)
    wr2, nr2 = runlib.arm_candidate_wins(conf_rev, reverse=True)
    pool_w = wm1 + wm2 + wr1 + wr2
    pool_n = nm1 + nm2 + nr1 + nr2
    from stats import wilson

    pool_rate = pool_w / pool_n
    pool_ci = wilson(pool_w, pool_n)
    assert f"pooled {pool_rate:.3f} [{pool_ci[0]:.3f}, {pool_ci[1]:.3f}]" in summary


def test_defaults_without_early_stop_unchanged() -> None:
    args = sweep.parse_args(["--tag", "t", "--candidates", "h0"])
    assert args.early_stop is False
    with pytest.raises(SystemExit):
        sweep.parse_args(["--tag", "t", "--candidates", "h0", "--stop-chunk", "4"])
