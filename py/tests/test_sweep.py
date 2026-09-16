"""One-command policy-spec sweep (`py/sweep.py`)."""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import time
from pathlib import Path

import pytest

pytest.importorskip("arena")

_SWEEP = Path(__file__).resolve().parents[1] / "sweep.py"
_spec = importlib.util.spec_from_file_location("arena_sweep", _SWEEP)
assert _spec and _spec.loader
sweep = importlib.util.module_from_spec(_spec)
sys.modules["arena_sweep"] = sweep
_spec.loader.exec_module(sweep)

_REPO = Path(__file__).resolve().parents[2]

FAST = "h0:depth=2,beam=2,k=1,nodes=80,value=v0,tt=0"
CAND_NODES = "h0:depth=2,beam=2,k=1,nodes=160,value=v0,tt=0"
CAND_DEPTH = "h0:depth=3,beam=2,k=1,nodes=80,value=v0,tt=0"


def _porcelain(repo: Path) -> str:
    r = subprocess.run(
        ["git", "-C", str(repo), "status", "--porcelain"],
        check=True,
        capture_output=True,
        text=True,
    )
    return r.stdout


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


def _cleanup_results_worktree(repo: Path, wt: Path, existed: bool) -> None:
    subprocess.run(
        ["git", "-C", str(repo), "worktree", "remove", "--force", str(wt)],
        capture_output=True,
        text=True,
    )
    subprocess.run(
        ["git", "-C", str(repo), "worktree", "prune"],
        capture_output=True,
        text=True,
    )
    if not existed:
        subprocess.run(
            ["git", "-C", str(repo), "branch", "-D", "results"],
            capture_output=True,
            text=True,
        )


@pytest.fixture(scope="module")
def smoke(tmp_path_factory: pytest.TempPathFactory):
    tmp = tmp_path_factory.mktemp("sweep")
    bare = tmp / "remote.git"
    subprocess.run(["git", "init", "--bare", str(bare)], check=True, capture_output=True)
    root = tmp / "results"
    wt = tmp / "wt"
    existed = (
        subprocess.run(
            ["git", "-C", str(_REPO), "show-ref", "--verify", "--quiet", "refs/heads/results"],
            capture_output=True,
        ).returncode
        == 0
    )
    before = _porcelain(_REPO)
    argv = [
        "--smoke",
        "--tag",
        "s1",
        "--baseline",
        FAST,
        "--candidates",
        CAND_NODES,
        CAND_DEPTH,
        "--root",
        str(root),
        "--publish",
        "--publish-remote",
        str(bare),
        "--publish-dir",
        str(wt),
    ]
    t0 = time.perf_counter()
    first = _run_sweep(argv)
    wall = time.perf_counter() - t0
    after = _porcelain(_REPO)
    ctx = {
        "tmp": tmp,
        "bare": bare,
        "root": root,
        "wt": wt,
        "tag": root / "s1",
        "argv": argv,
        "first": first,
        "wall": wall,
        "before": before,
        "after": after,
        "existed": existed,
    }
    try:
        yield ctx
    finally:
        _cleanup_results_worktree(_REPO, wt, existed)


def test_smoke_end_to_end(smoke) -> None:
    first = smoke["first"]
    tag: Path = smoke["tag"]
    print(f"sweep smoke wall: {smoke['wall']:.1f}s", flush=True)
    if first.returncode != 0:
        print(first.stdout)
        print(first.stderr, file=sys.stderr)
    assert first.returncode == 0, first.stderr or first.stdout
    assert smoke["before"] == smoke["after"]

    cands = json.loads((tag / "candidates.json").read_text())
    assert cands == [
        {"index": 1, "spec": CAND_NODES},
        {"index": 2, "spec": CAND_DEPTH},
    ]
    assert (tag / "c01-screen.json").is_file()
    assert (tag / "c02-screen.json").is_file()
    assert (tag / "tp-baseline.json").is_file()

    finals = sorted(tag.glob("c0*-final.json"))
    reverses = sorted(tag.glob("c0*-reverse.json"))
    assert len(finals) == 1, finals
    assert len(reverses) == 1, reverses
    assert finals[0].name.replace("-final.json", "") == reverses[0].name.replace(
        "-reverse.json", ""
    )

    summary = (tag / "SUMMARY.md").read_text()
    verdict_lines = [ln for ln in summary.splitlines() if ln.startswith("verdict:")]
    assert len(verdict_lines) == 1, summary
    best_lines = [ln for ln in summary.splitlines() if ln.startswith("best:")]
    assert len(best_lines) == 1, summary
    assert best_lines[0] != "best: none"

    run = json.loads((tag / "RUN.json").read_text())
    decisions = run["finalist_decisions"]
    assert len(decisions) == 2
    assert sum(1 for d in decisions if d["decision"] == "finalist") == 1

    branches = subprocess.run(
        ["git", "-C", str(smoke["bare"]), "branch"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    assert "results" in branches
    remote_summary = subprocess.run(
        ["git", "--git-dir", str(smoke["bare"]), "show", "results:s1/SUMMARY.md"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    assert remote_summary == summary


def test_resumable(smoke) -> None:
    tag: Path = smoke["tag"]
    tip = subprocess.run(
        ["git", "--git-dir", str(smoke["bare"]), "rev-parse", "results"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    second = _run_sweep(smoke["argv"])
    if second.returncode != 0:
        print(second.stdout)
        print(second.stderr, file=sys.stderr)
    assert second.returncode == 0, second.stderr or second.stdout
    out = second.stdout + second.stderr
    for stage in ("screen", "final", "summary", "publish"):
        assert f"skip: {stage}" in out, (stage, out)
    tip2 = subprocess.run(
        ["git", "--git-dir", str(smoke["bare"]), "rev-parse", "results"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    assert tip2 == tip

    summary = tag / "SUMMARY.md"
    others = {
        p: p.stat().st_mtime_ns
        for p in tag.iterdir()
        if p.is_file() and p.name != "SUMMARY.md" and p.name != "RUN.json"
    }
    mtime = summary.stat().st_mtime_ns
    time.sleep(0.05)
    forced = _run_sweep([*smoke["argv"], "--only", "summary", "--force"])
    assert forced.returncode == 0, forced.stderr or forced.stdout
    assert summary.stat().st_mtime_ns > mtime
    for p, old in others.items():
        assert p.stat().st_mtime_ns == old, p.name


def _row(index: int, spec: str, rate: float, lo: float, hi: float) -> sweep.ScreenRow:
    return sweep.ScreenRow(index, spec, rate, (lo, hi), games=1024, gps=1.0)


def test_finalist_rule() -> None:
    rows = [
        _row(1, "h0:nodes=4000", 0.56, 0.53, 0.59),
        _row(2, "h0:depth=4,beam=8", 0.52, 0.49, 0.55),
        _row(3, "h0:k=8", 0.47, 0.44, 0.50),
        _row(4, "h0:odepth=1", 0.45, 0.42, 0.48),
    ]
    two = sweep.decide_finalists(rows, 2)
    assert [d.index for d in two if d.is_finalist] == [1, 2]
    assert two[2].decision.startswith("skipped:")
    assert "0.50" in two[2].decision
    assert two[3].decision.startswith("skipped:")
    assert "0.48" in two[3].decision

    three = sweep.decide_finalists(rows, 3)
    assert [d.index for d in three if d.is_finalist] == [1, 2]
    assert three[2].decision.startswith("skipped:")
    assert "0.50" in three[2].decision

    losers = [
        _row(1, "a", 0.40, 0.37, 0.43),
        _row(2, "b", 0.41, 0.38, 0.44),
        _row(3, "c", 0.39, 0.36, 0.42),
    ]
    none = sweep.decide_finalists(losers, 2)
    assert [d.index for d in none if d.is_finalist] == []
    assert all(d.decision.startswith("skipped:") for d in none)
    assert sweep.format_best_line([]) == "best: none"


def test_bad_spec_stops_before_matchup(tmp_path: Path) -> None:
    root = tmp_path / "results"
    r = _run_sweep(
        [
            "--tag",
            "bad",
            "--candidates",
            "h0:depht=2",
            "--baseline",
            FAST,
            "--root",
            str(root),
        ]
    )
    text = r.stdout + r.stderr
    assert r.returncode != 0
    assert "depht" in text
    assert not (root / "bad" / "c01-screen.json").exists()
