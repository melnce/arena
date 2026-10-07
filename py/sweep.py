#!/usr/bin/env python3
"""One-command policy-spec sweep: screen → finalists → final → summary → publish.

With ``--early-stop``, the final stage plays main/reverse in chunks and stops
each finalist once a decision class reaches probability ≥ 1 − γ (default γ=0.02).
Chunk files ``<stem>-final.part<j>.json`` merge into the arm files unchanged for
downstream readers. See ``--stop-chunk``, ``--stop-thresholds``, ``--stop-gamma``,
``--stop-min-games``, and ``--stop-combine``.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from matchup import (  # noqa: E402
    DEFAULT_POOL,
    load_extra_deck_files,
    resolve_selected_decks,
)
from stats import wilson  # noqa: E402
from runlib import (  # noqa: E402
    arm_candidate_wins,
    chunk_arm_schedule,
    class_probabilities,
    flag_given,
    format_decision_class,
    format_verdict_line,
    git_head,
    load_run,
    mark_end as runlib_mark_end,
    mark_start as runlib_mark_start,
    matchup_argv,
    merge_matchup_arm,
    new_run,
    pooled_candidate,
    publish_output_exist,
    publish_tag,
    rate_ci,
    repo_root,
    reverse_candidate,
    run_tee,
    save_run as runlib_save_run,
    stage_seconds,
    validate_stop_chunk,
    verdict,
)


STAGES = ("screen", "final", "summary", "publish")
HALF = 0.5
FAST = "h0:depth=2,beam=2,k=1,nodes=80,value=v0,tt=0"

# Per-pair defaults. --target-games scales the 1024 / 4096 / 2048 / 256
# totals from these ratios (not a second copy of those totals).
DEFAULT_SCREEN_GAMES = 4
DEFAULT_FINAL_GAMES = 16
DEFAULT_FINAL_REVERSE = 8
DEFAULT_TP_GAMES = 1

# Historical 16-deck calibration (the on-disk set before named pools).
# Floors stay those totals; the live default pool is ``meta``.
YARDSTICK_DECKS = 16
YARDSTICK_PAIRS = YARDSTICK_DECKS * YARDSTICK_DECKS
FLOOR_SCREEN = DEFAULT_SCREEN_GAMES * YARDSTICK_PAIRS // 2  # 512
FLOOR_FINAL = DEFAULT_FINAL_GAMES * YARDSTICK_PAIRS // 2  # 2048
FLOOR_REVERSE = DEFAULT_FINAL_REVERSE * YARDSTICK_PAIRS // 2  # 1024

PER_PAIR_FLAGS = ("--screen-games", "--final-games", "--final-reverse", "--tp-games")
GAME_STAGE_ORDER = ("screen", "final", "reverse", "tp")


@dataclass(frozen=True)
class Candidate:
    index: int
    spec: str

    @property
    def stem(self) -> str:
        return f"c{self.index:02d}"


@dataclass(frozen=True)
class ScreenRow:
    index: int
    spec: str
    rate: float
    interval: tuple[float, float]
    games: int
    gps: float


@dataclass(frozen=True)
class FinalistDecision:
    index: int
    spec: str
    decision: str

    @property
    def is_finalist(self) -> bool:
        return self.decision == "finalist"


@dataclass(frozen=True)
class BestPick:
    index: int
    spec: str
    main_rate: float
    word: str


def decide_finalists(rows: list[ScreenRow], n_keep: int) -> list[FinalistDecision]:
    """Keep the top *n_keep* candidates whose screen interval high end is above 0.50."""
    eligible = [r for r in rows if r.interval[1] > HALF]
    eligible.sort(key=lambda r: (-r.rate, r.index))
    chosen = {r.index for r in eligible[: max(0, n_keep)]}
    out: list[FinalistDecision] = []
    for r in rows:
        hi = r.interval[1]
        if hi <= HALF:
            decision = f"skipped: interval high {hi:.2f} < 0.50"
        elif r.index in chosen:
            decision = "finalist"
        else:
            decision = f"not in top {n_keep}"
        out.append(FinalistDecision(r.index, r.spec, decision))
    return out


def format_best_line(picks: list[BestPick]) -> str:
    if not picks:
        return "best: none"
    winner = max(picks, key=lambda p: (p.main_rate, -p.index))
    return f"best: {winner.spec} ({winner.word})"


@dataclass(frozen=True)
class StageSize:
    name: str
    games_per_pair: int
    pairs: int

    @property
    def total(self) -> int:
        return self.games_per_pair * self.pairs


@dataclass(frozen=True)
class SweepSizing:
    names: tuple[str, ...]
    n_decks: int
    pairs: int
    screen: StageSize
    final: StageSize
    reverse: StageSize
    tp: StageSize
    target_games: int | None
    pool: str | None = None

    def stage(self, name: str) -> StageSize:
        return {
            "screen": self.screen,
            "final": self.final,
            "reverse": self.reverse,
            "tp": self.tp,
        }[name]


def calibrated_target_games() -> int:
    """Final-stage total the standing yardstick is calibrated on (16 × 16 × 16)."""
    return DEFAULT_FINAL_GAMES * YARDSTICK_PAIRS


def load_sweep_decks(
    repo: Path,
    restrict: list[str] | None,
    pool: str | None = None,
) -> tuple[str, dict[str, dict[str, int]]]:
    """Same composition as ``matchup.main``: named pool or --decks, then extras.

    Sweep has no ``--deck-file``, so the extra list is empty; the call is
    still the matchup path so pair counting cannot drift.
    """
    pool_name, decks = resolve_selected_decks(
        repo / "oracle" / "decks", pool, restrict
    )
    decks.update(load_extra_deck_files([]))
    return pool_name, decks


def _stage_total_target(target_games: int, default_per_pair: int) -> int:
    return math.ceil(target_games * default_per_pair / DEFAULT_FINAL_GAMES)


def _per_pair(total_target: int, pairs: int) -> int:
    return max(1, math.ceil(total_target / pairs))


def compute_sizing(
    args: argparse.Namespace,
    n_decks: int,
    names: list[str] | tuple[str, ...] | None = None,
    pool: str | None = None,
) -> SweepSizing:
    """Connect per-pair flags to the totals every runbook quotes.

    ``sweep.py`` takes games *per ordered deck pair*. Every runbook and
    every results summary quotes *totals*. Nothing used to join the two,
    so a smaller ``--decks`` pool silently inherited the 16-deck
    per-pair defaults and ran a fraction of the games the standing
    yardstick is calibrated on.

    That calibration is 16 decks → 16 × 16 = 256 ordered pairs (mirrors
    included, same ``n * n`` as ``matchup.py``) × the default 4 / 16 / 8
    / 1 per-pair flags → 1 024 / 4 096 / 2 048 / 256 games.
    ``results/sweep8/`` restricted the pool to the seven real decks,
    kept those defaults, and got 49 pairs → 196 / 784 / 392 — the same
    ~4 games per cell and 5.2× fewer games overall. Every candidate
    came back a coin flip at roughly ±0.07 and the run was unreadable.
    The reverse arm is the sharpest edge: ``--final-reverse`` is
    independent of ``--final-games`` and defaults to 8, so a raised
    main arm with a forgotten reverse flag makes every candidate
    ``unclear (reverse)``. That is a silent wrong answer, not a crash.

    ``--target-games N`` derives the per-pair counts from the resolved
    pair count so one flag reproduces that shape at any pool size. The
    ratios come from the current defaults (screen:final:reverse:tp =
    4:16:8:1), not a second hardcoded copy of 1024 / 4096 / 2048 / 256.
    Totals below half the calibrated figures refuse unless
    ``--allow-small`` or ``--smoke``.
    """
    if n_decks < 1:
        raise SystemExit("no decks selected")
    resolved = tuple(names) if names is not None else ()
    if resolved and len(resolved) != n_decks:
        raise SystemExit(f"deck count {n_decks} != names {len(resolved)}")
    pairs = n_decks * n_decks
    if getattr(args, "target_games", None) is not None:
        target = int(args.target_games)
        if target < 1:
            raise SystemExit("--target-games must be a positive integer")
        args.screen_games = _per_pair(_stage_total_target(target, DEFAULT_SCREEN_GAMES), pairs)
        args.final_games = _per_pair(target, pairs)
        args.final_reverse = _per_pair(_stage_total_target(target, DEFAULT_FINAL_REVERSE), pairs)
        args.tp_games = _per_pair(_stage_total_target(target, DEFAULT_TP_GAMES), pairs)
        target_games: int | None = target
    else:
        target_games = None
    return SweepSizing(
        names=resolved,
        n_decks=n_decks,
        pairs=pairs,
        screen=StageSize("screen", int(args.screen_games), pairs),
        final=StageSize("final", int(args.final_games), pairs),
        reverse=StageSize("reverse", int(args.final_reverse), pairs),
        tp=StageSize("tp", int(args.tp_games), pairs),
        target_games=target_games,
        pool=pool,
    )


def running_game_stages(wanted: list[str] | None) -> tuple[str, ...]:
    """Game-count stages implied by ``requested_stages`` (tp ⊂ screen, reverse ⊂ final)."""
    if wanted is None:
        return GAME_STAGE_ORDER
    out: list[str] = []
    if "screen" in wanted:
        out.extend(["screen", "tp"])
    if "final" in wanted:
        out.extend(["final", "reverse"])
    return tuple(out)


def format_sizing_block(sizing: SweepSizing, running: tuple[str, ...] | None = None) -> str:
    shown = GAME_STAGE_ORDER if running is None else running
    if sizing.names:
        decks_line = f"decks: {sizing.n_decks} ({', '.join(sizing.names)})"
    else:
        decks_line = f"decks: {sizing.n_decks}"
    lines = []
    if sizing.pool:
        lines.append(f"pool: {sizing.pool}")
    lines.extend(
        [
            decks_line,
            f"pairs: {sizing.pairs} ({sizing.n_decks} x {sizing.n_decks}, mirrors included)",
        ]
    )
    for name in GAME_STAGE_ORDER:
        if name not in shown:
            continue
        st = sizing.stage(name)
        lines.append(f"{name:<7} {st.games_per_pair:>3} games/pair x {st.pairs} = {st.total:>5}")
    return "\n".join(lines)


def sizing_metadata(sizing: SweepSizing, block: str) -> dict[str, Any]:
    return {
        "pool": sizing.pool,
        "decks": list(sizing.names),
        "n_decks": sizing.n_decks,
        "pairs": sizing.pairs,
        "games_per_pair": {
            "screen": sizing.screen.games_per_pair,
            "final": sizing.final.games_per_pair,
            "reverse": sizing.reverse.games_per_pair,
            "tp": sizing.tp.games_per_pair,
        },
        "totals": {
            "screen": sizing.screen.total,
            "final": sizing.final.total,
            "reverse": sizing.reverse.total,
            "tp": sizing.tp.total,
        },
        "target_games": sizing.target_games,
        "block": block,
    }


def check_sizing(
    args: argparse.Namespace,
    sizing: SweepSizing,
    wanted: list[str] | None = None,
) -> None:
    """Refuse an undersized run. ``wanted`` is ``requested_stages``; None checks all."""
    raw = list(getattr(args, "_argv", []))
    running = running_game_stages(wanted)
    allow_small = bool(getattr(args, "allow_small", False))
    smoke = bool(getattr(args, "smoke", False))

    # Independent of the floor: a raised --final-games with the default
    # reverse 8 is the sweep-8 near-miss (392 vs 4116). --smoke does not
    # bypass this; --allow-small does.
    if (
        "reverse" in running
        and not allow_small
        and flag_given(raw, "--final-games")
        and not flag_given(raw, "--final-reverse")
    ):
        raise SystemExit(
            f"reverse-arm: --final-games {args.final_games} was given but "
            f"--final-reverse was not; reverse would run at the default "
            f"{DEFAULT_FINAL_REVERSE} against a {args.final_games}-game main arm. "
            f"Pass --final-reverse or --allow-small."
        )

    if smoke or allow_small:
        return

    floors = {
        "screen": FLOOR_SCREEN,
        "final": FLOOR_FINAL,
        "reverse": FLOOR_REVERSE,
    }
    failed: list[str] = []
    for name, floor in floors.items():
        if name not in running:
            continue
        st = sizing.stage(name)
        if st.total < floor:
            failed.append(
                f"{name} total {st.total} < floor {floor} "
                f"({st.games_per_pair} games/pair x {st.pairs})"
            )
    if failed:
        hint = calibrated_target_games()
        raise SystemExit(
            "sizing: "
            + "; ".join(failed)
            + f". Use --target-games {hint} to keep the calibrated totals "
            f"at this pool size, or --allow-small to bypass."
        )


def apply_sizing(
    args: argparse.Namespace,
    n_decks: int,
    names: list[str] | tuple[str, ...] | None = None,
    *,
    wanted: list[str] | None = None,
    pool: str | None = None,
) -> SweepSizing:
    sizing = compute_sizing(args, n_decks, names, pool)
    check_sizing(args, sizing, wanted)
    return sizing


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    raw = list(sys.argv[1:] if argv is None else argv)
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--tag", required=True, help="sweep name; everything lands in <root>/<tag>/")
    p.add_argument(
        "--candidates",
        nargs="+",
        required=True,
        help="policy specs to measure (files are named by index, never by spec)",
    )
    p.add_argument("--baseline", default="h0", help="screen / final opponent (default: h0)")
    p.add_argument("--seed", type=int, default=1, help="matchup seed (default: 1)")
    p.add_argument("--root", default=None, help="results root (default: <repo>/results)")
    p.add_argument("--decks", nargs="*", default=None, help="restrict matchup decks (passed through)")
    p.add_argument(
        "--pool",
        default=None,
        help="named pool from oracle/decks/POOLS.json (default: meta)",
    )
    p.add_argument("--threads", type=int, default=None)
    p.add_argument(
        "--screen-games",
        type=int,
        default=DEFAULT_SCREEN_GAMES,
        help=f"games per pair on the screen (default: {DEFAULT_SCREEN_GAMES})",
    )
    p.add_argument(
        "--final-games",
        type=int,
        default=DEFAULT_FINAL_GAMES,
        help=f"games per pair on the final (default: {DEFAULT_FINAL_GAMES})",
    )
    p.add_argument(
        "--final-reverse",
        type=int,
        default=DEFAULT_FINAL_REVERSE,
        help=f"games per pair on the reverse seat (default: {DEFAULT_FINAL_REVERSE})",
    )
    p.add_argument(
        "--tp-games",
        type=int,
        default=DEFAULT_TP_GAMES,
        help=f"throughput games per pair (default: {DEFAULT_TP_GAMES})",
    )
    p.add_argument(
        "--target-games",
        type=int,
        default=None,
        help=(
            "derive per-pair counts from the resolved deck pool so the "
            "final-stage total matches this value (mutually exclusive with "
            "--screen-games / --final-games / --final-reverse / --tp-games)"
        ),
    )
    p.add_argument(
        "--allow-small",
        action="store_true",
        help="bypass the totals floor and the reverse-arm consistency check",
    )
    p.add_argument(
        "--finalists",
        type=int,
        default=2,
        help="how many screen survivors to take to the final (default: 2)",
    )
    p.add_argument("--mirrors", nargs="+", default=[], help="optional extra single-deck finals")
    p.add_argument("--mirror-games", type=int, default=200)
    p.add_argument("--force", action="store_true", help="rerun every requested stage / matchup")
    p.add_argument("--only", choices=STAGES, help="run a single stage")
    p.add_argument("--skip-screen", action="store_true")
    p.add_argument("--skip-final", action="store_true")
    p.add_argument("--skip-summary", action="store_true")
    p.add_argument("--skip-publish", action="store_true")
    p.add_argument("--publish", action="store_true", help="copy artifacts to the results worktree and push")
    p.add_argument("--publish-remote", default="origin")
    p.add_argument("--publish-branch", default="results")
    p.add_argument(
        "--publish-dir",
        default=None,
        help="git worktree of the orphan results branch (default: <repo>/../arena-results-wt)",
    )
    p.add_argument(
        "--smoke",
        action="store_true",
        help=(
            "tiny run for tests (2 decks, 1 game, 1 finalist, h0-fast baseline); "
            "explicit flags still override"
        ),
    )
    p.add_argument(
        "--early-stop",
        action="store_true",
        help="chunked finals with predictive early stopping (opt-in)",
    )
    p.add_argument(
        "--stop-chunk",
        type=int,
        default=2,
        help="games per pair per chunk with --early-stop (default: 2, must be even)",
    )
    p.add_argument(
        "--stop-thresholds",
        type=float,
        nargs="+",
        default=[0.51],
        help="pooled thresholds for the stop rule (default: 0.51)",
    )
    p.add_argument(
        "--stop-gamma",
        type=float,
        default=0.02,
        help="stop when a class reaches probability ≥ 1 − γ (default: 0.02)",
    )
    p.add_argument(
        "--stop-min-games",
        type=int,
        default=1024,
        help="no early stop before this many final games for the candidate (default: 1024)",
    )
    p.add_argument(
        "--stop-combine",
        default=None,
        metavar="TAG",
        help="confirm run: decide on counts combined with <root>/TAG",
    )
    args = p.parse_args(raw)
    args._argv = raw
    if flag_given(raw, "--target-games"):
        conflicted = [f for f in PER_PAIR_FLAGS if flag_given(raw, f)]
        if conflicted:
            raise SystemExit(
                f"--target-games cannot be combined with {', '.join(conflicted)}"
            )
        if args.target_games is not None and args.target_games < 1:
            raise SystemExit("--target-games must be a positive integer")
    if flag_given(raw, "--pool") and flag_given(raw, "--decks"):
        raise SystemExit("--pool cannot be combined with --decks")
    if args.smoke:
        if not flag_given(raw, "--decks") and not flag_given(raw, "--pool"):
            args.decks = ["basic-forest", "basic-rune"]
        # --target-games owns the per-pair counts when both are given.
        if not flag_given(raw, "--target-games"):
            if not flag_given(raw, "--screen-games"):
                args.screen_games = 1
            if not flag_given(raw, "--final-games"):
                args.final_games = 1
            if not flag_given(raw, "--final-reverse"):
                args.final_reverse = 1
            if not flag_given(raw, "--tp-games"):
                args.tp_games = 1
        if not flag_given(raw, "--finalists"):
            args.finalists = 1
        if not flag_given(raw, "--baseline"):
            args.baseline = FAST
    if not flag_given(raw, "--pool") and not flag_given(raw, "--decks") and not args.decks:
        args.pool = DEFAULT_POOL
    _validate_early_stop_args(args, raw)
    return args


def _validate_early_stop_args(args: argparse.Namespace, raw: list[str]) -> None:
    early_only = (
        "--stop-chunk",
        "--stop-thresholds",
        "--stop-gamma",
        "--stop-min-games",
        "--stop-combine",
    )
    if not args.early_stop:
        for name in early_only:
            if flag_given(raw, name):
                raise SystemExit(f"{name} requires --early-stop")
        return
    if args.mirrors:
        raise SystemExit("--mirrors is not supported with --early-stop")
    if args.stop_gamma <= 0 or args.stop_gamma >= 0.5:
        raise SystemExit(f"--stop-gamma must satisfy 0 < γ < 0.5 (got {args.stop_gamma})")
    if args.stop_min_games < 1:
        raise SystemExit("--stop-min-games must be a positive integer")
    for t in args.stop_thresholds:
        if t <= 0 or t >= 1:
            raise SystemExit(f"--stop-thresholds values must lie in (0, 1) (got {t})")
    validate_stop_chunk(args.stop_chunk, args.final_games, args.final_reverse)


def requested_stages(args: argparse.Namespace) -> list[str]:
    if args.only:
        wanted = [args.only]
    else:
        wanted = list(STAGES)
    skips = {
        "screen": args.skip_screen,
        "final": args.skip_final,
        "summary": args.skip_summary,
        "publish": args.skip_publish,
    }
    wanted = [s for s in wanted if not skips.get(s)]
    if "publish" in wanted and not args.publish and args.only != "publish":
        wanted.remove("publish")
    return wanted


def validate_policy_spec(spec: str, repo: Path) -> None:
    """0-game dry call: arena.matchup parses via AnyPolicy::parse_spec (same as by_name)."""
    import arena

    db = arena.load_cards(str(repo / "cards"))
    raw = json.loads((repo / "oracle" / "decks" / "basic-forest.json").read_text(encoding="utf-8"))
    decks = {"basic-forest": {str(k): int(v) for k, v in raw.items()}}
    try:
        arena.matchup(db, decks, 0, 1, policy=spec, threads=1)
    except Exception as e:
        raise SystemExit(f"invalid policy spec {spec!r}: {e}") from e


class Runner:
    def __init__(self, args: argparse.Namespace) -> None:
        self.args = args
        self.repo = repo_root()
        self.py_dir = Path(__file__).resolve().parent
        root = Path(args.root) if args.root else self.repo / "results"
        self.root = root.resolve()
        self.tag_dir = (self.root / args.tag).resolve()
        self.tag_dir.mkdir(parents=True, exist_ok=True)
        pub = args.publish_dir
        self.publish_dir = (
            Path(pub).resolve() if pub else (self.repo / ".." / "arena-results-wt").resolve()
        )
        self.candidates = [Candidate(i + 1, spec) for i, spec in enumerate(args.candidates)]
        self.run_path = self.tag_dir / "RUN.json"
        self.run: dict[str, Any] = self._load_run()

    def _argv_list(self) -> list[str]:
        return [sys.executable, str(Path(__file__).resolve()), *self.args._argv]

    def _load_run(self) -> dict[str, Any]:
        data = load_run(self.run_path)
        if data is not None:
            return data
        run = new_run(self._argv_list(), self.repo)
        run["final_game_offset"] = int(self.args.screen_games)
        return run

    def save_run(self) -> None:
        runlib_save_run(self.run_path, self.run, self._argv_list(), self.repo)

    def tee(self, argv: list[str], log_path: Path, stage: str, append: bool = False) -> None:
        try:
            run_tee(argv, log_path, append=append)
        except SystemExit as e:
            text = str(e)
            if text.startswith("failed "):
                raise SystemExit(f"{stage} {text}") from None
            raise

    def mark_start(self, stage: str) -> None:
        runlib_mark_start(self.run, self.run_path, stage, self._argv_list(), self.repo)

    def mark_end(self, stage: str) -> None:
        runlib_mark_end(self.run, self.run_path, stage, self._argv_list(), self.repo)

    def add_decks(self, cmd: list[str]) -> None:
        if self.args.decks:
            cmd.extend(["--decks", *self.args.decks])
        elif self.args.pool:
            cmd.extend(["--pool", self.args.pool])

    def write_candidates(self) -> None:
        payload = [{"index": c.index, "spec": c.spec} for c in self.candidates]
        text = json.dumps(payload, indent=2) + "\n"
        path = self.tag_dir / "candidates.json"
        if path.is_file() and path.read_text(encoding="utf-8") == text:
            return
        path.write_text(text, encoding="utf-8")

    def validate_all(self) -> None:
        specs = [self.args.baseline, *[c.spec for c in self.candidates]]
        seen: set[str] = set()
        for spec in specs:
            if spec in seen:
                continue
            seen.add(spec)
            validate_policy_spec(spec, self.repo)

    def matchup_exists(self, stem: str) -> bool:
        return (self.tag_dir / f"{stem}.json").is_file()

    def final_game_offset(self) -> int:
        """Per-pair game index where the final stage starts (after the screen)."""
        if "final_game_offset" in self.run:
            return int(self.run["final_game_offset"])
        return 0

    def run_matchup(
        self,
        extra: list[str],
        out_stem: str,
        stage: str,
        game_offset: int | None = None,
    ) -> None:
        out_json = self.tag_dir / f"{out_stem}.json"
        log = self.tag_dir / f"{out_stem}.txt"
        if not self.args.force and out_json.is_file():
            print(f"skip: {out_stem}", flush=True)
            return
        cmd = matchup_argv(
            self.py_dir,
            extra,
            out_json,
            seed=self.args.seed,
            threads=self.args.threads,
            game_offset=game_offset,
        )
        self.tee(cmd, log, stage)

    def screen_outputs_exist(self) -> bool:
        if not (self.tag_dir / "tp-baseline.json").is_file():
            return False
        for c in self.candidates:
            if not (self.tag_dir / f"{c.stem}-screen.json").is_file():
                return False
            if not (self.tag_dir / f"tp-{c.stem}.json").is_file():
                return False
        return True

    def load_json(self, name: str) -> dict[str, Any]:
        return json.loads((self.tag_dir / name).read_text(encoding="utf-8"))

    def screen_rows(self) -> list[ScreenRow]:
        rows: list[ScreenRow] = []
        for c in self.candidates:
            summary = self.load_json(f"{c.stem}-screen.json")["summary"]
            tp = self.load_json(f"tp-{c.stem}.json")["summary"]
            lo, hi = summary["wilson95"]
            rows.append(
                ScreenRow(
                    index=c.index,
                    spec=c.spec,
                    rate=float(summary["policy_a_win_rate"]),
                    interval=(float(lo), float(hi)),
                    games=int(summary.get("games", 0)),
                    gps=float(tp.get("games_per_second", 0.0)),
                )
            )
        return rows

    def record_finalists(self, decisions: list[FinalistDecision]) -> None:
        self.run["finalist_decisions"] = [
            {"index": d.index, "spec": d.spec, "decision": d.decision} for d in decisions
        ]
        self.save_run()

    def stored_decisions(self) -> list[FinalistDecision]:
        raw = self.run.get("finalist_decisions")
        if isinstance(raw, list) and raw:
            out: list[FinalistDecision] = []
            for item in raw:
                if not isinstance(item, dict):
                    continue
                out.append(
                    FinalistDecision(
                        int(item["index"]),
                        str(item["spec"]),
                        str(item["decision"]),
                    )
                )
            if out:
                return out
        return decide_finalists(self.screen_rows(), self.args.finalists)

    def finalists(self) -> list[Candidate]:
        chosen = {d.index for d in self.stored_decisions() if d.is_finalist}
        return [c for c in self.candidates if c.index in chosen]

    def final_outputs_exist(self) -> bool:
        finals = self.finalists()
        if not finals and self.screen_outputs_exist():
            return True
        if not finals:
            return False
        for c in finals:
            if not (self.tag_dir / f"{c.stem}-final.json").is_file():
                return False
            if not (self.tag_dir / f"{c.stem}-reverse.json").is_file():
                return False
            for deck in self.args.mirrors:
                if not (self.tag_dir / f"{c.stem}-mirror-{deck}.json").is_file():
                    return False
        return True

    def summary_output_exist(self) -> bool:
        return (self.tag_dir / "SUMMARY.md").is_file()

    def publish_ready(self) -> bool:
        return publish_output_exist(self.publish_dir, self.args.tag, self.tag_dir)

    def _deck_names(self) -> list[str]:
        pool_name, decks = load_sweep_decks(self.repo, self.args.decks, self.args.pool)
        del pool_name
        return list(decks.keys())

    def _record_early_stop_config(self) -> None:
        self.run["early_stop"] = {
            "chunk": self.args.stop_chunk,
            "thresholds": list(self.args.stop_thresholds),
            "gamma": self.args.stop_gamma,
            "min_games": self.args.stop_min_games,
            "combine": self.args.stop_combine,
            "finalists": self.run.get("early_stop", {}).get("finalists", {}),
        }
        self.save_run()

    def _load_combine_arm_counts(
        self, c: Candidate
    ) -> tuple[tuple[int, int], tuple[int, int]]:
        tag = self.args.stop_combine
        assert tag
        comb_dir = (self.root / tag).resolve()
        run_path = comb_dir / "RUN.json"
        cands_path = comb_dir / "candidates.json"
        if not run_path.is_file():
            raise SystemExit(f"--stop-combine {tag!r}: missing {run_path}")
        if not cands_path.is_file():
            raise SystemExit(f"--stop-combine {tag!r}: missing {cands_path}")
        comb_run = json.loads(run_path.read_text(encoding="utf-8"))
        comb_baseline = None
        argv = comb_run.get("argv") or []
        for i, tok in enumerate(argv):
            if tok == "--baseline" and i + 1 < len(argv):
                comb_baseline = argv[i + 1]
                break
        if comb_baseline != self.args.baseline:
            raise SystemExit(
                f"--stop-combine {tag!r}: baseline {comb_baseline!r} "
                f"≠ this run's {self.args.baseline!r}"
            )
        comb_cands = json.loads(cands_path.read_text(encoding="utf-8"))
        match = next((x for x in comb_cands if x.get("spec") == c.spec), None)
        if match is None:
            raise SystemExit(
                f"--stop-combine {tag!r}: no finalist with candidate spec {c.spec!r}"
            )
        stem = f"c{int(match['index']):02d}"
        main_path = comb_dir / f"{stem}-final.json"
        rev_path = comb_dir / f"{stem}-reverse.json"
        if not main_path.is_file() or not rev_path.is_file():
            raise SystemExit(f"--stop-combine {tag!r}: missing arm files for {stem}")
        main_doc = json.loads(main_path.read_text(encoding="utf-8"))
        rev_doc = json.loads(rev_path.read_text(encoding="utf-8"))
        if main_doc.get("policy_b") != self.args.baseline:
            raise SystemExit(
                f"--stop-combine {tag!r}: {stem}-final baseline "
                f"{main_doc.get('policy_b')!r} ≠ {self.args.baseline!r}"
            )
        if rev_doc.get("policy_a") != self.args.baseline:
            raise SystemExit(
                f"--stop-combine {tag!r}: {stem}-reverse baseline "
                f"{rev_doc.get('policy_a')!r} ≠ {self.args.baseline!r}"
            )
        return (
            arm_candidate_wins(main_doc, reverse=False),
            arm_candidate_wins(rev_doc, reverse=True),
        )

    def _chunk_part_stem(self, c: Candidate, arm: str, chunk_index: int) -> str:
        base = f"{c.stem}-final" if arm == "main" else f"{c.stem}-reverse"
        return f"{base}.part{chunk_index + 1}"

    @staticmethod
    def _prefix_part_numbers(
        schedule: list[tuple[str, int]],
        prefix_len: int,
    ) -> tuple[set[int], set[int]]:
        """Part numbers (1-based) played in the first *prefix_len* schedule steps."""
        main_parts: set[int] = set()
        rev_parts: set[int] = set()
        for arm, j in schedule[:prefix_len]:
            part = j + 1
            if arm == "main":
                main_parts.add(part)
            else:
                rev_parts.add(part)
        return main_parts, rev_parts

    def _load_arm_chunk_docs_for_parts(
        self,
        c: Candidate,
        arm: str,
        parts: set[int],
    ) -> list[dict[str, Any]]:
        if not parts:
            return []
        base = f"{c.stem}-final" if arm == "main" else f"{c.stem}-reverse"
        out: list[dict[str, Any]] = []
        for part in sorted(parts):
            path = self.tag_dir / f"{base}.part{part}.json"
            if not path.is_file():
                raise SystemExit(f"early stop: missing chunk {path.name}")
            out.append(json.loads(path.read_text(encoding="utf-8")))
        return out

    def _load_arm_chunk_docs(self, c: Candidate, arm: str) -> list[dict[str, Any]]:
        prefix = f"{c.stem}-final" if arm == "main" else f"{c.stem}-reverse"
        paths = sorted(
            self.tag_dir.glob(f"{prefix}.part*.json"),
            key=lambda p: int(p.stem.rsplit("part", 1)[1]),
        )
        return [json.loads(p.read_text(encoding="utf-8")) for p in paths]

    def _run_chunk_matchup(
        self,
        extra: list[str],
        part_stem: str,
        game_offset: int,
    ) -> None:
        out_json = self.tag_dir / f"{part_stem}.json"
        log = self.tag_dir / f"{part_stem}.txt"
        if not self.args.force and out_json.is_file():
            print(f"skip: {part_stem}", flush=True)
            return
        cmd = matchup_argv(
            self.py_dir,
            extra,
            out_json,
            seed=self.args.seed,
            threads=self.args.threads,
            game_offset=game_offset,
        )
        self.tee(cmd, log, "final")

    def _chunk_matchup_argv(
        self,
        c: Candidate,
        arm: str,
        games: int,
        offset: int,
    ) -> list[str]:
        baseline = self.args.baseline
        if arm == "main":
            extra = [
                "--policy-a",
                c.spec,
                "--policy-b",
                baseline,
                "--games",
                str(games),
            ]
        else:
            extra = [
                "--policy-a",
                baseline,
                "--policy-b",
                c.spec,
                "--games",
                str(games),
            ]
        self.add_decks(extra)
        return extra

    def _merge_and_write_arm_chunks(
        self,
        c: Candidate,
        arm: str,
        chunks: list[dict[str, Any]],
    ) -> dict[str, Any]:
        if not chunks:
            raise SystemExit(f"early stop: no chunks for {c.stem} {arm}")
        names = chunks[0].get("decks") or self._deck_names()
        merged = merge_matchup_arm(chunks, list(names))
        stem = f"{c.stem}-final" if arm == "main" else f"{c.stem}-reverse"
        out = self.tag_dir / f"{stem}.json"
        out.write_text(json.dumps(merged, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return merged

    def _merge_and_write_arm(self, c: Candidate, arm: str) -> dict[str, Any]:
        return self._merge_and_write_arm_chunks(c, arm, self._load_arm_chunk_docs(c, arm))

    def _early_stop_look(
        self,
        c: Candidate,
        combine: tuple[tuple[int, int], tuple[int, int]] | None,
        schedule: list[tuple[str, int]],
        prefix_len: int,
    ) -> dict[str, Any]:
        chunk = self.args.stop_chunk
        pairs = int(self.run.get("sizing", {}).get("pairs") or 0)
        if pairs <= 0:
            _, decks = load_sweep_decks(self.repo, self.args.decks, self.args.pool)
            pairs = len(decks) * len(decks)
        planned_main = self.args.final_games * pairs
        planned_rev = self.args.final_reverse * pairs
        main_parts, rev_parts = self._prefix_part_numbers(schedule, prefix_len)
        main_chunks = self._load_arm_chunk_docs_for_parts(c, "main", main_parts)
        rev_chunks = self._load_arm_chunk_docs_for_parts(c, "reverse", rev_parts)
        if main_chunks:
            w_main, n_main = arm_candidate_wins(
                merge_matchup_arm(
                    main_chunks,
                    list(main_chunks[0].get("decks") or self._deck_names()),
                ),
                reverse=False,
            )
        else:
            w_main, n_main = 0, 0
        if rev_chunks:
            w_rev, n_rev = arm_candidate_wins(
                merge_matchup_arm(
                    rev_chunks,
                    list(rev_chunks[0].get("decks") or self._deck_names()),
                ),
                reverse=True,
            )
        else:
            w_rev, n_rev = 0, 0
        r_main = planned_main - n_main
        r_rev = planned_rev - n_rev
        comb_main = combine[0] if combine else None
        comb_rev = combine[1] if combine else None
        probs = class_probabilities(
            w_main,
            n_main,
            r_main,
            w_rev,
            n_rev,
            r_rev,
            list(self.args.stop_thresholds),
            combine_main=comb_main,
            combine_rev=comb_rev,
        )
        best_cls = max(probs, key=lambda k: probs[k])
        best_p = probs[best_cls]
        games_so_far = n_main + n_rev
        return {
            "main_games": n_main,
            "main_planned": planned_main,
            "reverse_games": n_rev,
            "reverse_planned": planned_rev,
            "main_rate": w_main / n_main if n_main else 0.0,
            "reverse_rate": w_rev / n_rev if n_rev else 0.0,
            "class_probs": {
                format_decision_class(k, self.args.stop_thresholds): v for k, v in sorted(probs.items())
            },
            "best_class": format_decision_class(best_cls, self.args.stop_thresholds),
            "best_prob": best_p,
            "games_this_run": games_so_far,
        }

    def _log_early_stop_look(self, c: Candidate, look: dict[str, Any], action: str) -> None:
        thresh = ", ".join(f"{t:g}" for t in self.args.stop_thresholds)
        print(
            f"early stop {c.stem} look: "
            f"main {look['main_games']}/{look['main_planned']} "
            f"({look['main_rate']:.3f}) "
            f"reverse {look['reverse_games']}/{look['reverse_planned']} "
            f"({look['reverse_rate']:.3f}) "
            f"classes={look['class_probs']} "
            f"{action} "
            f"(γ={self.args.stop_gamma}, thresholds {thresh})",
            flush=True,
        )

    def _finalist_early_stop(
        self,
        c: Candidate,
        combine: tuple[tuple[int, int], tuple[int, int]] | None,
    ) -> None:
        schedule = chunk_arm_schedule(
            self.args.final_games,
            self.args.final_reverse,
            self.args.stop_chunk,
        )
        chunk = self.args.stop_chunk
        base_offset = self.final_game_offset()
        early = self.run.setdefault("early_stop", {})
        fin_rec = early.setdefault("finalists", {})
        rec = dict(fin_rec.get(c.stem) or {})
        looks: list[dict[str, Any]] = []
        stopped = False
        stopped_at: int | None = rec.get("stopped_at_look")
        gamma_cutoff = 1.0 - self.args.stop_gamma

        for step, (arm, j) in enumerate(schedule):
            look_num = step + 1
            if stopped_at is not None and look_num > stopped_at:
                break
            part_stem = self._chunk_part_stem(c, arm, j)
            if stopped_at is None or look_num <= stopped_at:
                game_off = base_offset + j * chunk
                extra = self._chunk_matchup_argv(c, arm, chunk, game_off)
                self._run_chunk_matchup(extra, part_stem, game_off)

            look = self._early_stop_look(c, combine, schedule, step + 1)
            looks.append(look)
            action = "continue"
            if (
                stopped_at is None
                and look["games_this_run"] >= self.args.stop_min_games
                and (
                    look["reverse_games"] > 0
                    or int(look.get("reverse_planned") or 0) == 0
                )
                and look["best_prob"] >= gamma_cutoff
            ):
                stopped = True
                stopped_at = look_num
                action = "stop"
            elif stopped_at is not None and look_num == stopped_at:
                action = "stop"
            self._log_early_stop_look(c, look, action)
            if stopped:
                break

        rec["looks"] = looks
        rec["stopped"] = stopped or stopped_at is not None
        if stopped_at is not None:
            rec["stopped_at_look"] = stopped_at
            last = looks[stopped_at - 1]
            rec["settled_class"] = last["best_class"]
            rec["settled_prob"] = last["best_prob"]
        elif looks:
            last = looks[-1]
            rec["settled_class"] = last["best_class"]
            rec["settled_prob"] = last["best_prob"]
        fin_rec[c.stem] = rec
        self.save_run()
        main_parts, rev_parts = self._prefix_part_numbers(schedule, len(looks))
        if main_parts:
            self._merge_and_write_arm_chunks(
                c,
                "main",
                self._load_arm_chunk_docs_for_parts(c, "main", main_parts),
            )
        if rev_parts:
            self._merge_and_write_arm_chunks(
                c,
                "reverse",
                self._load_arm_chunk_docs_for_parts(c, "reverse", rev_parts),
            )

    def _early_stop_summary_line(self, c: Candidate) -> str | None:
        early = self.run.get("early_stop") or {}
        rec = (early.get("finalists") or {}).get(c.stem)
        if not rec:
            return None
        looks = rec.get("looks") or []
        if not looks:
            return None
        last = looks[-1]
        gamma = early.get("gamma", self.args.stop_gamma)
        thresh = early.get("thresholds", self.args.stop_thresholds)
        thresh_s = ", ".join(f"{t:g}" for t in thresh)
        if rec.get("stopped"):
            stop_look = int(rec.get("stopped_at_look") or len(looks))
            stop_data = looks[stop_look - 1]
            return (
                f"early stop: stopped after {stop_data['main_games']}/"
                f"{stop_data['main_planned']} main and "
                f"{stop_data['reverse_games']}/{stop_data['reverse_planned']} reverse games — "
                f"{rec.get('settled_class', stop_data['best_class'])} settled at "
                f"P = {rec.get('settled_prob', stop_data['best_prob']):.3f} "
                f"(γ {gamma}, thresholds {thresh_s})"
            )
        return (
            f"early stop: ran to the end (largest class probability at the last look "
            f"{last.get('best_prob', 0.0):.3f})"
        )

    def _build_combined_section(self) -> list[str]:
        tag = self.args.stop_combine or (self.run.get("early_stop") or {}).get("combine")
        if not tag:
            return []
        combine = tag
        lines = ["", f"## combined with {combine}", ""]
        lines.append("| index | spec | main | reverse | pooled |")
        lines.append("|---|---:|---:|---:|---:|")
        verdicts: list[str] = []
        comb_dir = (self.root / tag).resolve()
        for c in self.finalists():
            combine_counts = self._load_combine_arm_counts(c)
            main_doc = self.load_json(f"{c.stem}-final.json")
            rev_doc = self.load_json(f"{c.stem}-reverse.json")
            comb_stem = None
            comb_cands = json.loads((comb_dir / "candidates.json").read_text(encoding="utf-8"))
            match = next((x for x in comb_cands if x.get("spec") == c.spec), None)
            if match:
                comb_stem = f"c{int(match['index']):02d}"
            if comb_stem:
                comb_main = json.loads((comb_dir / f"{comb_stem}-final.json").read_text())
                comb_rev = json.loads((comb_dir / f"{comb_stem}-reverse.json").read_text())
            else:
                comb_main = comb_rev = None
            w_main, n_main = arm_candidate_wins(main_doc, reverse=False)
            w_rev, n_rev = arm_candidate_wins(rev_doc, reverse=True)
            if comb_main and comb_rev:
                wc_m, nc_m = arm_candidate_wins(comb_main, reverse=False)
                wc_r, nc_r = arm_candidate_wins(comb_rev, reverse=True)
                w_main += wc_m
                n_main += nc_m
                w_rev += wc_r
                n_rev += nc_r
            del combine_counts
            main_rate = w_main / n_main if n_main else 0.0
            main_ci = wilson(w_main, n_main)
            rev_rate = w_rev / n_rev if n_rev else 0.0
            rev_ci = wilson(w_rev, n_rev)
            pool_w = w_main + w_rev
            pool_n = n_main + n_rev
            pool_rate = pool_w / pool_n if pool_n else 0.0
            pool_ci = wilson(pool_w, pool_n)
            lines.append(
                f"| {c.index} | `{c.spec}` | {rate_ci(main_rate, main_ci)} | "
                f"{rate_ci(rev_rate, rev_ci)} | {rate_ci(pool_rate, pool_ci)} |"
            )
            verdicts.append(
                format_verdict_line(
                    c.spec, main_rate, main_ci, rev_rate, rev_ci, pool_rate, pool_ci
                )
            )
        lines.append("")
        lines.extend(verdicts)
        return lines

    def stage_screen(self) -> None:
        if not self.args.force and self.screen_outputs_exist():
            print("skip: screen", flush=True)
            self.record_finalists(decide_finalists(self.screen_rows(), self.args.finalists))
            return
        self.mark_start("screen")
        baseline = self.args.baseline
        tp_base = ["--policy", baseline, "--games", str(self.args.tp_games)]
        self.add_decks(tp_base)
        self.run_matchup(tp_base, "tp-baseline", "screen")
        for c in self.candidates:
            screen = [
                "--policy-a",
                c.spec,
                "--policy-b",
                baseline,
                "--games",
                str(self.args.screen_games),
            ]
            self.add_decks(screen)
            self.run_matchup(screen, f"{c.stem}-screen", "screen")
            tp = ["--policy", c.spec, "--games", str(self.args.tp_games)]
            self.add_decks(tp)
            self.run_matchup(tp, f"tp-{c.stem}", "screen")
        self.record_finalists(decide_finalists(self.screen_rows(), self.args.finalists))
        self.mark_end("screen")

    def stage_final(self) -> None:
        if not self.args.force and self.final_outputs_exist():
            print("skip: final", flush=True)
            return
        finals = self.finalists()
        self.mark_start("final")
        offset = self.final_game_offset()
        if self.args.early_stop:
            self._record_early_stop_config()
            for c in finals:
                cand_combine = (
                    self._load_combine_arm_counts(c) if self.args.stop_combine else None
                )
                self._finalist_early_stop(c, cand_combine)
        else:
            baseline = self.args.baseline
            for c in finals:
                main = [
                    "--policy-a",
                    c.spec,
                    "--policy-b",
                    baseline,
                    "--games",
                    str(self.args.final_games),
                ]
                self.add_decks(main)
                self.run_matchup(main, f"{c.stem}-final", "final", game_offset=offset)
                rev = [
                    "--policy-a",
                    baseline,
                    "--policy-b",
                    c.spec,
                    "--games",
                    str(self.args.final_reverse),
                ]
                self.add_decks(rev)
                self.run_matchup(rev, f"{c.stem}-reverse", "final", game_offset=offset)
                for deck in self.args.mirrors:
                    mir = [
                        "--policy-a",
                        c.spec,
                        "--policy-b",
                        baseline,
                        "--games",
                        str(self.args.mirror_games),
                        "--decks",
                        deck,
                    ]
                    self.run_matchup(
                        mir, f"{c.stem}-mirror-{deck}", "final", game_offset=offset
                    )
        self.mark_end("final")

    def stage_summary(self) -> None:
        if not self.args.force and self.summary_output_exist():
            print("skip: summary", flush=True)
            return
        self.mark_start("summary")
        self.mark_end("summary")
        text = self.build_summary()
        (self.tag_dir / "SUMMARY.md").write_text(text, encoding="utf-8")
        print(text, flush=True)

    def build_summary(self) -> str:
        rows = self.screen_rows()
        decisions = self.stored_decisions()
        by_index = {d.index: d for d in decisions}
        sha = self.run.get("git") or git_head(self.repo)
        times: list[str] = []
        total = 0.0
        for s in STAGES:
            sec = stage_seconds(self.run, s)
            if sec is None:
                continue
            times.append(f"{s} {sec:.1f}s")
            total += sec
        lines: list[str] = []
        lines.append(f"# Sweep `{self.args.tag}`")
        lines.append("")
        lines.append(f"- tag: `{self.args.tag}`")
        lines.append(f"- baseline: `{self.args.baseline}`")
        lines.append(f"- seed: {self.args.seed}")
        lines.append(f"- engine: `{sha}`")
        lines.append(f"- wall: {total:.1f}s")
        lines.append(f"- stages: {', '.join(times) if times else 'n/a'}")
        lines.append("")
        lines.append("## screen")
        lines.append("")
        lines.append("| index | spec | rate | games | g/s vs baseline | decision |")
        lines.append("|---|---:|---|---:|---|---|")
        tp_base = self.load_json("tp-baseline.json")
        base_gps = float(tp_base.get("summary", {}).get("games_per_second", 0.0))
        for r in sorted(rows, key=lambda x: (-x.rate, x.index)):
            d = by_index.get(r.index)
            decision = d.decision if d else ""
            vs = f"{r.gps:.2f} / {base_gps:.2f}"
            lines.append(
                f"| {r.index} | `{r.spec}` | {rate_ci(r.rate, r.interval)} | "
                f"{r.games} | {vs} | {decision} |"
            )
        lines.append("")
        lines.append("## final")
        lines.append("")
        off = self.final_game_offset()
        fg = int(self.args.final_games)
        if fg > 0:
            lines.append(
                f"final games per pair: {off} … {off + fg - 1} "
                f"(after the screen's {off} game{'s' if off != 1 else ''} per pair)"
            )
            lines.append("")
        header = ["index", "spec", "main", "reverse", "pooled"]
        header.extend(self.args.mirrors)
        lines.append("| " + " | ".join(header) + " |")
        lines.append("|" + "|".join("---" if i <= 1 else "---:" for i in range(len(header))) + "|")
        picks: list[BestPick] = []
        verdicts: list[str] = []
        early_lines: list[str] = []
        for c in self.finalists():
            main_doc = self.load_json(f"{c.stem}-final.json")
            rev_doc = self.load_json(f"{c.stem}-reverse.json")
            main_s = main_doc["summary"]
            rev_s = rev_doc["summary"]
            main_rate = float(main_s["policy_a_win_rate"])
            main_ci = (float(main_s["wilson95"][0]), float(main_s["wilson95"][1]))
            rev_rate, rev_ci = reverse_candidate(rev_s)
            pool_rate, pool_ci = pooled_candidate(main_doc, rev_doc, tag=c.spec)
            cells = [
                str(c.index),
                f"`{c.spec}`",
                rate_ci(main_rate, main_ci),
                rate_ci(rev_rate, rev_ci),
                rate_ci(pool_rate, pool_ci),
            ]
            for deck in self.args.mirrors:
                ms = self.load_json(f"{c.stem}-mirror-{deck}.json")["summary"]
                cells.append(
                    rate_ci(
                        float(ms["policy_a_win_rate"]),
                        (float(ms["wilson95"][0]), float(ms["wilson95"][1])),
                    )
                )
            lines.append("| " + " | ".join(cells) + " |")
            word = verdict(main_rate, main_ci, rev_rate, rev_ci)
            picks.append(BestPick(c.index, c.spec, main_rate, word))
            verdicts.append(
                format_verdict_line(
                    c.spec, main_rate, main_ci, rev_rate, rev_ci, pool_rate, pool_ci
                )
            )
            es_line = self._early_stop_summary_line(c)
            if es_line:
                early_lines.append(es_line)
        lines.append("")
        lines.extend(verdicts)
        if early_lines:
            if verdicts:
                lines.append("")
            lines.extend(early_lines)
        if verdicts or early_lines:
            lines.append("")
        lines.append(format_best_line(picks))
        if self.args.stop_combine or (self.run.get("early_stop") or {}).get("combine"):
            lines.extend(self._build_combined_section())
        lines.append("")
        return "\n".join(lines)

    def _tee_publish(self, argv: list[str], log_path: Path, append: bool = False) -> None:
        self.tee(argv, log_path, "publish", append=append)

    def stage_publish(self) -> None:
        if not self.args.force and self.publish_ready():
            print("skip: publish", flush=True)
            return
        self.mark_start("publish")
        log = self.tag_dir / "publish.txt"
        if log.is_file():
            log.unlink()
        try:
            publish_tag(
                self.repo,
                self.publish_dir,
                self.args.publish_remote,
                self.args.publish_branch,
                self.tag_dir,
                self.args.tag,
                log,
                tee=self._tee_publish,
            )
        except SystemExit as e:
            text = str(e)
            if text.startswith("failed "):
                raise SystemExit(f"publish {text}") from None
            raise
        self.mark_end("publish")

    def apply_and_record_sizing(self) -> None:
        """Print and persist the pool arithmetic before any stage runs."""
        pool_name, decks = load_sweep_decks(
            self.repo, self.args.decks, self.args.pool
        )
        names = list(decks.keys())
        wanted = requested_stages(self.args)
        sizing = compute_sizing(self.args, len(names), names, pool_name)
        running = running_game_stages(wanted)
        block = format_sizing_block(sizing, running)
        print(block, flush=True)
        self.run["sizing"] = sizing_metadata(sizing, block)
        check_sizing(self.args, sizing, wanted)

    def go(self) -> int:
        self.apply_and_record_sizing()
        self.validate_all()
        self.write_candidates()
        self.save_run()
        wanted = requested_stages(self.args)
        dispatch = {
            "screen": self.stage_screen,
            "final": self.stage_final,
            "summary": self.stage_summary,
            "publish": self.stage_publish,
        }
        for stage in wanted:
            dispatch[stage]()
        return 0


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    return Runner(args).go()


if __name__ == "__main__":
    raise SystemExit(main())
