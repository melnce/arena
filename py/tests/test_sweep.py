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


# The seven real decks from results/sweep8/. 7 × 7 = 49 pairs.
SWEEP8_DECKS = [
    "abyss-p8rfn",
    "afnm-minatodao",
    "elf-neanisu2",
    "ramp-37772",
    "ramp-claywies",
    "royal-nattui",
    "rune-mach15",
]


def _parse(*extra: str):
    return sweep.parse_args(["--tag", "t", "--candidates", "h0", *extra])


@pytest.fixture(scope="module")
def smoke(tmp_path_factory: pytest.TempPathFactory):
    pytest.importorskip("arena")
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
    assert "pairs: 4 (2 x 2, mirrors included)" in first.stdout
    assert "1 games/pair x 4 =     4" in first.stdout

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
    assert "| index | spec | main | reverse | pooled |" in summary
    verdict_lines = [ln for ln in summary.splitlines() if ln.startswith("verdict:")]
    assert len(verdict_lines) == 1, summary
    assert "pooled" in verdict_lines[0] and "(not gated)" in verdict_lines[0], verdict_lines[0]
    best_lines = [ln for ln in summary.splitlines() if ln.startswith("best:")]
    assert len(best_lines) == 1, summary
    assert best_lines[0] != "best: none"

    run = json.loads((tag / "RUN.json").read_text())
    decisions = run["finalist_decisions"]
    assert len(decisions) == 2
    assert sum(1 for d in decisions if d["decision"] == "finalist") == 1
    sizing = run["sizing"]
    assert sizing["n_decks"] == 2
    assert sizing["pairs"] == 4
    assert sizing["games_per_pair"] == {
        "screen": 1,
        "final": 1,
        "reverse": 1,
        "tp": 1,
    }
    assert "pairs: 4 (2 x 2, mirrors included)" in sizing["block"]

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
    pytest.importorskip("arena")
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


def test_pair_count_matches_matchup() -> None:
    """Pair count is n * n (mirrors included), same as matchup.py."""
    decks = sweep.load_sweep_decks(_REPO, None)
    assert len(decks) == sweep.YARDSTICK_DECKS == 16
    # matchup.py: n_pairs = len(names) * len(names)
    assert len(decks) * len(decks) == sweep.YARDSTICK_PAIRS == 256
    seven = sweep.load_sweep_decks(_REPO, SWEEP8_DECKS)
    assert list(seven) == SWEEP8_DECKS
    assert len(seven) * len(seven) == 49


def test_defaults_16_decks_are_calibrated_totals() -> None:
    args = _parse()
    assert args.screen_games == 4
    assert args.final_games == 16
    assert args.final_reverse == 8
    assert args.tp_games == 1
    sizing = sweep.apply_sizing(args, 16)
    assert sizing.screen.total == 1024
    assert sizing.final.total == 4096
    assert sizing.reverse.total == 2048
    assert sizing.tp.total == 256
    # Explicit today's flags stay byte-identical (no mutation, no refuse).
    explicit = _parse(
        "--screen-games",
        "4",
        "--final-games",
        "16",
        "--final-reverse",
        "8",
        "--tp-games",
        "1",
    )
    after = sweep.apply_sizing(explicit, 16)
    assert explicit.screen_games == 4
    assert explicit.final_games == 16
    assert explicit.final_reverse == 8
    assert explicit.tp_games == 1
    assert after.screen.total == 1024
    assert after.final.total == 4096
    assert after.reverse.total == 2048


def test_sweep8_defaults_refused() -> None:
    """results/sweep8/ numbers: 7 decks, defaults → 196 / 784 / 392."""
    args = _parse()
    sizing = sweep.compute_sizing(args, 7, SWEEP8_DECKS)
    assert sizing.pairs == 49
    assert sizing.screen.total == 196
    assert sizing.final.total == 784
    assert sizing.reverse.total == 392
    with pytest.raises(SystemExit) as exc:
        sweep.apply_sizing(args, 7, SWEEP8_DECKS)
    msg = str(exc.value)
    assert "196" in msg
    assert "784" in msg
    assert "392" in msg
    assert "--target-games" in msg


def test_target_games_7_decks() -> None:
    args = _parse("--target-games", "4096")
    sizing = sweep.apply_sizing(args, 7, SWEEP8_DECKS)
    assert args.screen_games == 21
    assert args.final_games == 84
    assert args.final_reverse == 42
    assert sizing.screen.total >= 1029
    assert sizing.final.total >= 4116
    assert sizing.reverse.total >= 2058
    assert sizing.screen.total == 21 * 49
    assert sizing.final.total == 84 * 49
    assert sizing.reverse.total == 42 * 49


def test_target_games_conflicts_with_final_games() -> None:
    with pytest.raises(SystemExit) as exc:
        _parse("--target-games", "4096", "--final-games", "16")
    msg = str(exc.value)
    assert "--target-games" in msg
    assert "--final-games" in msg


def test_final_games_without_reverse_refused() -> None:
    args = _parse("--final-games", "84")
    assert args.final_reverse == 8
    with pytest.raises(SystemExit) as exc:
        sweep.apply_sizing(args, 7, SWEEP8_DECKS)
    msg = str(exc.value)
    assert "84" in msg
    assert "8" in msg
    assert "reverse" in msg.lower()


def test_allow_small_bypasses_floor() -> None:
    args = _parse("--allow-small")
    sizing = sweep.apply_sizing(args, 7, SWEEP8_DECKS)
    assert sizing.screen.total == 196
    assert sizing.final.total == 784
    assert sizing.reverse.total == 392


def test_allow_small_bypasses_reverse_arm() -> None:
    args = _parse("--final-games", "84", "--allow-small")
    sizing = sweep.apply_sizing(args, 7, SWEEP8_DECKS)
    assert sizing.final.games_per_pair == 84
    assert sizing.reverse.games_per_pair == 8


def test_smoke_bypasses_floor() -> None:
    args = _parse("--smoke")
    assert args.decks == ["basic-forest", "basic-rune"]
    assert args.screen_games == 1
    sizing = sweep.apply_sizing(args, 2, args.decks)
    assert sizing.pairs == 4
    assert sizing.screen.total == 4
    assert sizing.final.total == 4
    assert sizing.reverse.total == 4


def test_target_games_uneven_pool_ceils() -> None:
    args = _parse("--target-games", "4096")
    sizing = sweep.apply_sizing(args, 5)
    assert sizing.pairs == 25
    assert sizing.screen.games_per_pair == 41  # ceil(1024 / 25)
    assert sizing.final.games_per_pair == 164  # ceil(4096 / 25)
    assert sizing.reverse.games_per_pair == 82  # ceil(2048 / 25)
    assert sizing.screen.total >= 1024
    assert sizing.final.total >= 4096
    assert sizing.reverse.total >= 2048
    assert sizing.screen.total == 41 * 25
    assert sizing.final.total == 164 * 25
    assert sizing.reverse.total == 82 * 25


def test_sizing_block_lists_running_stages() -> None:
    args = _parse("--target-games", "4096")
    sizing = sweep.compute_sizing(args, 7, SWEEP8_DECKS)
    block = sweep.format_sizing_block(sizing)
    assert "decks: 7 (abyss-p8rfn, afnm-minatodao, elf-neanisu2, ramp-37772, ramp-claywies, royal-nattui, rune-mach15)" in block
    assert "pairs: 49 (7 x 7, mirrors included)" in block
    assert "21 games/pair x 49 =  1029" in block
    assert "84 games/pair x 49 =  4116" in block
    assert "42 games/pair x 49 =  2058" in block
    only_screen = sweep.format_sizing_block(sizing, sweep.running_game_stages(["screen"]))
    assert "screen" in only_screen
    assert "tp" in only_screen
    assert "final" not in only_screen
    assert "reverse" not in only_screen
