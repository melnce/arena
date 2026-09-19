"""Shared helpers for one-command drivers (`iterate.py`, `sweep.py`)."""

from __future__ import annotations

import datetime as dt
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from stats import wilson  # noqa: E402


HALF = 0.5

TeeFn = Callable[..., None]


def repo_root() -> Path:
    here = Path(__file__).resolve().parent
    for d in (Path.cwd(), here, *here.parents):
        if (d / "oracle" / "decks").is_dir() and (d / "cards").is_dir():
            return d
    return Path.cwd()


def flag_given(argv: list[str], *names: str) -> bool:
    for a in argv:
        if a in names:
            return True
        for n in names:
            if a.startswith(n + "="):
                return True
    return False


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def git_head(repo: Path) -> str:
    r = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return r.stdout.strip() if r.returncode == 0 else "unknown"


def verdict(
    main_rate: float,
    main_interval: tuple[float, float],
    reverse_rate: float,
    reverse_interval: tuple[float, float],
) -> str:
    """Standing yardstick rule. Intervals are Wilson 95 % [lo, hi]."""
    del main_rate, reverse_rate
    mlo, mhi = main_interval
    rlo, rhi = reverse_interval
    if mhi < HALF:
        return "worse"
    main_clear = mlo > HALF
    rev_clear = rlo > HALF
    if main_clear and rev_clear:
        return "better"
    main_has = mlo <= HALF <= mhi
    rev_has = rlo <= HALF <= rhi
    if main_has and rev_has:
        return "coin flip"
    disagree: list[str] = []
    if not main_clear:
        disagree.append("main")
    if not rev_clear:
        disagree.append("reverse")
    return "unclear (" + ", ".join(disagree) + ")"


def format_verdict_line(
    model: str,
    main_rate: float,
    main_ci: tuple[float, float],
    rev_rate: float,
    rev_ci: tuple[float, float],
    pooled_rate: float | None = None,
    pooled_ci: tuple[float, float] | None = None,
) -> str:
    word = verdict(main_rate, main_ci, rev_rate, rev_ci)
    line = (
        f"verdict: {model} {word} — "
        f"main {main_rate:.3f} [{main_ci[0]:.3f}, {main_ci[1]:.3f}], "
        f"reverse {rev_rate:.3f} [{rev_ci[0]:.3f}, {rev_ci[1]:.3f}]"
    )
    if pooled_rate is not None and pooled_ci is not None:
        line += (
            f", pooled {pooled_rate:.3f} "
            f"[{pooled_ci[0]:.3f}, {pooled_ci[1]:.3f}] (not gated)"
        )
    return line


def reverse_candidate(summary: dict[str, Any]) -> tuple[float, tuple[float, float]]:
    """Candidate is seat B: flip A's decisive rate and Wilson interval."""
    a_rate = float(summary["policy_a_win_rate"])
    lo, hi = summary["wilson95"]
    return 1.0 - a_rate, (1.0 - float(hi), 1.0 - float(lo))


def _summary_and_matrix(
    doc: dict[str, Any],
) -> tuple[dict[str, Any], dict[str, Any] | None]:
    """Accept a full matchup JSON or a bare summary dict."""
    inner = doc.get("summary")
    if isinstance(inner, dict) and "policy_a_win_rate" in inner:
        matrix = doc.get("matrix")
        return inner, matrix if isinstance(matrix, dict) else None
    matrix = doc.get("matrix")
    return doc, matrix if isinstance(matrix, dict) else None


def _matrix_a_counts(matrix: dict[str, Any]) -> tuple[int, int] | None:
    """Sum per-cell a_wins and decisive games (games − draws)."""
    a_wins = 0
    decisive = 0
    cells = 0
    for row in matrix.values():
        if not isinstance(row, dict):
            continue
        for cell in row.values():
            if not isinstance(cell, dict) or "a_wins" not in cell:
                continue
            cells += 1
            a_wins += int(cell["a_wins"])
            games = int(cell["games"])
            draws = int(cell.get("draws", 0))
            decisive += games - draws
    if cells == 0:
        return None
    return a_wins, decisive


def _a_decisive_counts(doc: dict[str, Any], tag: str) -> tuple[int, int]:
    """A's decisive wins and decisive games, matrix preferred, rate as fallback."""
    summary, matrix = _summary_and_matrix(doc)
    n = int(summary["decisive"])
    w_rate = int(round(float(summary["policy_a_win_rate"]) * n))
    if matrix is None:
        return w_rate, n
    counted = _matrix_a_counts(matrix)
    if counted is None:
        return w_rate, n
    w_mat, n_mat = counted
    if abs(w_mat - w_rate) > 1 or abs(n_mat - n) > 1:
        name = tag or str(summary.get("policy_a") or "?")
        raise ValueError(
            f"{name}: matrix counts (a_wins={w_mat}, decisive={n_mat}) "
            f"disagree with rate-derived (a_wins={w_rate}, decisive={n})"
        )
    return w_mat, n_mat


def pooled_candidate(
    main: dict[str, Any],
    reverse: dict[str, Any],
    tag: str = "",
) -> tuple[float, tuple[float, float]]:
    """Candidate's pooled rate and Wilson interval across main and reverse.

    Pooling is a fixed-effects combination that assumes a common rate
    across seats, and is therefore exactly the thing that hides seat
    asymmetry — which is why it is reported and not gated on.

    Counts, not rates. Main: the candidate is seat A (A's decisive wins).
    Reverse: the candidate is seat B; the seat flip is the same convention
    as ``reverse_candidate`` (candidate wins = decisive − A's wins). Prefer
    exact integer counts summed from the matchup JSON's per-cell ``a_wins``
    / ``games`` / ``draws`` where available, and fall back to
    ``round(policy_a_win_rate * decisive)`` only when the matrix is absent.
    The two must agree to within one game or this raises, naming ``tag``.
    """
    w_main, n_main = _a_decisive_counts(main, tag)
    w_a_rev, n_rev = _a_decisive_counts(reverse, tag)
    # Integer form of reverse_candidate's 1 − A-rate flip.
    rev_summary, _ = _summary_and_matrix(reverse)
    rev_rate, _ = reverse_candidate(rev_summary)
    w_rev = n_rev - w_a_rev
    w_rev_rate = int(round(rev_rate * n_rev))
    if abs(w_rev - w_rev_rate) > 1:
        name = tag or str(rev_summary.get("policy_b") or "?")
        raise ValueError(
            f"{name}: reverse seat-flip counts disagree "
            f"(n-a_wins={w_rev}, reverse_candidate={w_rev_rate})"
        )
    w = w_main + w_rev
    n = n_main + n_rev
    rate = w / n if n else 0.0
    return rate, wilson(w, n)


def rate_ci(rate: float, ci: tuple[float, float]) -> str:
    return f"{rate:.3f} [{ci[0]:.3f}, {ci[1]:.3f}]"


def run_tee(argv: list[str], log_path: Path, append: bool = False) -> None:
    print(argv, flush=True)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    mode = "a" if append else "w"
    with log_path.open(mode, encoding="utf-8", errors="replace") as log:
        proc = subprocess.Popen(
            argv,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
            bufsize=1,
        )
        assert proc.stdout is not None
        for line in proc.stdout:
            sys.stdout.write(line)
            sys.stdout.flush()
            log.write(line)
        rc = proc.wait()
    if rc != 0:
        raise SystemExit(f"failed (exit {rc}); log: {log_path}")


def py_tool(py_dir: Path, name: str) -> list[str]:
    return [sys.executable, str(Path(py_dir) / name)]


def matchup_argv(
    py_dir: Path,
    extra: list[str],
    out_json: Path,
    seed: int | None = None,
    threads: int | None = None,
) -> list[str]:
    cmd = py_tool(py_dir, "matchup.py")
    cmd.extend(extra)
    cmd.extend(["--out", str(Path(out_json).resolve())])
    if seed is not None and "--seed" not in extra:
        cmd.extend(["--seed", str(seed)])
    if threads is not None:
        cmd.extend(["--threads", str(threads)])
    return cmd


def new_run(argv: list[str], repo: Path) -> dict[str, Any]:
    return {
        "argv": argv,
        "git": git_head(repo),
        "python": sys.version,
        "cpu_count": os.cpu_count(),
        "stages": {},
    }


def load_run(path: Path) -> dict[str, Any] | None:
    if path.is_file():
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            if isinstance(data, dict):
                data.setdefault("stages", {})
                return data
        except json.JSONDecodeError:
            pass
    return None


def save_run(path: Path, run: dict[str, Any], argv: list[str], repo: Path) -> None:
    run["argv"] = argv
    run["git"] = git_head(repo)
    run["python"] = sys.version
    run["cpu_count"] = os.cpu_count()
    path.write_text(json.dumps(run, indent=2) + "\n", encoding="utf-8")


def mark_start(run: dict[str, Any], path: Path, stage: str, argv: list[str], repo: Path) -> None:
    run.setdefault("stages", {})
    run["stages"].setdefault(stage, {})
    run["stages"][stage]["start"] = utc_now()
    save_run(path, run, argv, repo)


def mark_end(run: dict[str, Any], path: Path, stage: str, argv: list[str], repo: Path) -> None:
    run["stages"].setdefault(stage, {})
    run["stages"][stage]["end"] = utc_now()
    save_run(path, run, argv, repo)


def stage_seconds(run: dict[str, Any], stage: str) -> float | None:
    rec = run.get("stages", {}).get(stage) or {}
    start, end = rec.get("start"), rec.get("end")
    if not start or not end:
        return None
    try:
        a = dt.datetime.fromisoformat(start)
        b = dt.datetime.fromisoformat(end)
    except ValueError:
        return None
    return max(0.0, (b - a).total_seconds())


def is_git_dir(path: Path) -> bool:
    if not path.is_dir():
        return False
    git = path / ".git"
    return git.is_file() or git.is_dir()


def remote_has_branch(repo: Path, remote: str, branch: str) -> bool:
    r = subprocess.run(
        ["git", "-C", str(repo), "ls-remote", "--heads", remote, branch],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return r.returncode == 0 and bool(r.stdout.strip())


def local_has_branch(repo: Path, branch: str) -> bool:
    r = subprocess.run(
        ["git", "-C", str(repo), "show-ref", "--verify", "--quiet", f"refs/heads/{branch}"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return r.returncode == 0


def git_has_ident(cwd: Path) -> bool:
    """True when env or git config can form a commit identity (GHA often cannot)."""
    env_name = os.environ.get("GIT_AUTHOR_NAME") or os.environ.get("GIT_COMMITTER_NAME")
    env_email = os.environ.get("GIT_AUTHOR_EMAIL") or os.environ.get("GIT_COMMITTER_EMAIL")
    if env_name and env_email:
        return True
    name = subprocess.run(
        ["git", "-C", str(cwd), "config", "user.name"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    email = subprocess.run(
        ["git", "-C", str(cwd), "config", "user.email"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return bool(name.stdout.strip()) and bool(email.stdout.strip())


def git_commit_argv(cwd: Path, *commit_args: str) -> list[str]:
    cmd = ["git", "-C", str(cwd)]
    if not git_has_ident(cwd):
        cmd.extend(["-c", "user.name=arena", "-c", "user.email=arena@localhost"])
    cmd.append("commit")
    cmd.extend(commit_args)
    return cmd


def copy_tag_artifacts(tag_dir: Path, dest: Path) -> None:
    dest.mkdir(parents=True, exist_ok=True)
    for p in sorted(tag_dir.iterdir()):
        if not p.is_file():
            continue
        if p.name == "SUMMARY.md" or p.suffix in {".txt", ".json"}:
            shutil.copy2(p, dest / p.name)


def publish_output_exist(publish_dir: Path, tag: str, tag_dir: Path) -> bool:
    dest = publish_dir / tag / "SUMMARY.md"
    local = tag_dir / "SUMMARY.md"
    if not dest.is_file() or not local.is_file():
        return False
    return dest.read_text(encoding="utf-8") == local.read_text(encoding="utf-8")


def ensure_worktree(
    repo: Path,
    publish_dir: Path,
    remote: str,
    branch: str,
    log: Path,
    tee: TeeFn | None = None,
) -> None:
    do_tee = tee or run_tee
    dest = publish_dir
    if is_git_dir(dest):
        do_tee(["git", "-C", str(dest), "pull", "--ff-only"], log, True)
        return
    dest.parent.mkdir(parents=True, exist_ok=True)
    if remote_has_branch(repo, remote, branch):
        if local_has_branch(repo, branch):
            do_tee(["git", "-C", str(repo), "fetch", remote, branch], log, True)
            do_tee(["git", "-C", str(repo), "worktree", "add", str(dest), branch], log, True)
        else:
            do_tee(
                ["git", "-C", str(repo), "fetch", remote, f"{branch}:{branch}"],
                log,
                True,
            )
            do_tee(["git", "-C", str(repo), "worktree", "add", str(dest), branch], log, True)
        return
    if local_has_branch(repo, branch):
        do_tee(["git", "-C", str(repo), "worktree", "add", str(dest), branch], log, True)
        return
    do_tee(["git", "-C", str(repo), "worktree", "add", "--detach", str(dest)], log, True)
    do_tee(["git", "-C", str(dest), "checkout", "--orphan", branch], log, True)
    do_tee(["git", "-C", str(dest), "rm", "-rf", "-q", "."], log, True)
    do_tee(git_commit_argv(dest, "--allow-empty", "-m", f"init {branch}"), log, True)
    do_tee(["git", "-C", str(dest), "push", "-u", remote, branch], log, True)


def publish_tag(
    repo: Path,
    publish_dir: Path,
    remote: str,
    branch: str,
    tag_dir: Path,
    tag: str,
    log: Path,
    tee: TeeFn | None = None,
) -> None:
    """Copy tag artifacts into the results worktree and push. Never touches the main tree."""
    do_tee = tee or run_tee
    ensure_worktree(repo, publish_dir, remote, branch, log, tee=do_tee)
    dest = publish_dir / tag
    copy_tag_artifacts(tag_dir, dest)
    do_tee(["git", "-C", str(publish_dir), "add", "-A"], log, True)
    status = subprocess.run(
        ["git", "-C", str(publish_dir), "status", "--porcelain"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if status.returncode != 0:
        raise SystemExit(f"failed (exit {status.returncode}); log: {log}")
    if status.stdout.strip():
        day = dt.datetime.now(dt.timezone.utc).date().isoformat()
        msg = f"{tag} results {day}"
        do_tee(git_commit_argv(publish_dir, "-m", msg), log, True)
        do_tee(["git", "-C", str(publish_dir), "push"], log, True)
        show = subprocess.run(
            ["git", "-C", str(publish_dir), "log", "-1", "--oneline"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        print(show.stdout, end="", flush=True)
    else:
        print("publish: nothing new to commit", flush=True)
