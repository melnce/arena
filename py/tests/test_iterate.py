"""One-command iteration driver (`py/iterate.py`)."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import time
from pathlib import Path

import pytest

np = pytest.importorskip("numpy")
pytest.importorskip("arena")

_ITER = Path(__file__).resolve().parents[1] / "iterate.py"
_spec = importlib.util.spec_from_file_location("arena_iterate", _ITER)
assert _spec and _spec.loader
iterate = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(iterate)

_REPO = Path(__file__).resolve().parents[2]


def _forest(root: Path) -> dict[str, dict[str, int]]:
    path = root / "oracle" / "decks" / "basic-forest.json"
    return {"basic-forest": {str(k): int(v) for k, v in json.loads(path.read_text()).items()}}


def _porcelain(repo: Path) -> str:
    r = subprocess.run(
        ["git", "-C", str(repo), "status", "--porcelain"],
        check=True,
        capture_output=True,
        text=True,
    )
    return r.stdout


def _run_iterate(args: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(_ITER), *args],
        cwd=cwd or _REPO,
        text=True,
        capture_output=True,
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
    tmp = tmp_path_factory.mktemp("iterate")
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
        "t1",
        "--seed",
        "7",
        "--root",
        str(root),
        "--publish",
        "--publish-remote",
        str(bare),
        "--publish-dir",
        str(wt),
    ]
    t0 = time.perf_counter()
    first = _run_iterate(argv)
    wall = time.perf_counter() - t0
    after = _porcelain(_REPO)
    ctx = {
        "tmp": tmp,
        "bare": bare,
        "root": root,
        "wt": wt,
        "tag": root / "t1",
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


def test_smoke_end_to_end(smoke, db, root: Path) -> None:
    first = smoke["first"]
    tag: Path = smoke["tag"]
    print(f"iterate smoke wall: {smoke['wall']:.1f}s", flush=True)
    if first.returncode != 0:
        print(first.stdout)
        print(first.stderr, file=sys.stderr)
    assert first.returncode == 0, first.stderr or first.stdout
    assert smoke["before"] == smoke["after"]

    for name in ("data-e0", "data-e10"):
        meta = json.loads((tag / name / "meta.json").read_text())
        assert int(meta["samples"]) > 0
        assert meta["aux_columns"][-1] == "search_v"

    spec = json.loads((tag / "linear.json").read_text())
    assert spec["arch"] == "linear"
    assert spec["feature_len"] == 545

    import arena

    net = str((tag / "linear.json").resolve())
    arena.matchup(
        db,
        _forest(root),
        games=1,
        seed=1,
        policy_a=f"h0:value=net,net={net}",
        policy_b="h0-fast",
        threads=1,
    )

    for name in (
        "main-linear.json",
        "reverse-linear.json",
        "sanity-linear.json",
        "tp-linear.json",
        "mirror-royal-nattui-linear.json",
        "tp-h0.json",
    ):
        assert (tag / name).is_file(), name

    assert json.loads((tag / "main-linear.json").read_text())["seed"] == 1
    assert json.loads((tag / "data-e0.json").read_text())["seed"] == 7

    summary = (tag / "SUMMARY.md").read_text()
    verdict_lines = [ln for ln in summary.splitlines() if ln.startswith("verdict: linear")]
    assert verdict_lines, summary
    words = ("better", "worse", "coin flip", "unclear")
    assert any(w in verdict_lines[0] for w in words), verdict_lines[0]

    run = json.loads((tag / "RUN.json").read_text())
    for stage in ("data", "train", "yard", "summary", "publish"):
        rec = run["stages"][stage]
        assert rec.get("start"), stage
        assert rec.get("end"), stage

    branches = subprocess.run(
        ["git", "-C", str(smoke["bare"]), "branch"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    assert "results" in branches
    remote_summary = subprocess.run(
        ["git", "--git-dir", str(smoke["bare"]), "show", "results:t1/SUMMARY.md"],
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
    second = _run_iterate(smoke["argv"])
    if second.returncode != 0:
        print(second.stdout)
        print(second.stderr, file=sys.stderr)
    assert second.returncode == 0, second.stderr or second.stdout
    out = second.stdout + second.stderr
    for stage in ("data", "train", "yard", "summary", "publish"):
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
    forced = _run_iterate([*smoke["argv"], "--only", "summary", "--force"])
    assert forced.returncode == 0, forced.stderr or forced.stdout
    assert summary.stat().st_mtime_ns > mtime
    for p, old in others.items():
        assert p.stat().st_mtime_ns == old, p.name


def test_verdict_rule() -> None:
    assert (
        iterate.verdict(0.536, (0.521, 0.551), 0.529, (0.507, 0.550)) == "better"
    )
    unclear = iterate.verdict(0.523, (0.508, 0.538), 0.500, (0.478, 0.522))
    assert unclear.startswith("unclear")
    assert "reverse" in unclear
    assert iterate.verdict(0.496, (0.466, 0.527), 0.490, (0.46, 0.52)) == "coin flip"
    assert iterate.verdict(0.389, (0.347, 0.432), 0.500, (0.40, 0.60)) == "worse"


def test_yard_seed_in_matchup_argv(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    captured: list[list[str]] = []

    def fake_tee(self, argv, log_path, stage, append=False):
        captured.append([str(a) for a in argv])

    monkeypatch.setattr(iterate.Runner, "tee", fake_tee)
    root = tmp_path / "results"
    tag = root / "t1"
    tag.mkdir(parents=True)
    (tag / "linear.json").write_text("{}\n", encoding="utf-8")
    rc = iterate.main(
        [
            "--tag",
            "t1",
            "--seed",
            "99",
            "--yard-seed",
            "5",
            "--root",
            str(root),
            "--only",
            "yard",
            "--models",
            "linear",
            "--yard-games",
            "1",
            "--reverse-games",
            "1",
            "--sanity-games",
            "1",
            "--tp-games",
            "1",
            "--mirror-games",
            "1",
            "--mirrors",
            "basic-forest",
        ]
    )
    assert rc == 0
    matchups = [a for a in captured if any(str(x).endswith("matchup.py") for x in a)]
    assert matchups
    mains = [a for a in matchups if any("main-linear.json" in x for x in a)]
    revs = [a for a in matchups if any("reverse-linear.json" in x for x in a)]
    assert mains and revs
    for argv in (*mains, *revs):
        assert "--seed" in argv
        assert argv[argv.index("--seed") + 1] == "5"
        assert "99" not in argv[argv.index("--seed") :]


def test_comma_in_root_stops_yard(tmp_path: Path) -> None:
    root = tmp_path / "res,ults"
    tag = root / "t1"
    tag.mkdir(parents=True)
    (tag / "linear.json").write_text("{}\n", encoding="utf-8")
    r = _run_iterate(
        [
            "--tag",
            "t1",
            "--seed",
            "1",
            "--root",
            str(root),
            "--only",
            "yard",
            "--models",
            "linear",
        ]
    )
    text = r.stdout + r.stderr
    assert r.returncode != 0
    path = str((tag / "linear.json").resolve())
    assert "," in path
    assert path in text
    assert not (tag / "main-linear.json").exists()
    assert "matchup.py" not in text
