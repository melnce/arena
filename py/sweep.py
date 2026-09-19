#!/usr/bin/env python3
"""One-command policy-spec sweep: screen → finalists → final → summary → publish."""

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

from matchup import load_deck_files, load_extra_deck_files  # noqa: E402
from runlib import (  # noqa: E402
    flag_given,
    format_verdict_line,
    git_head,
    load_run,
    mark_end as runlib_mark_end,
    mark_start as runlib_mark_start,
    matchup_argv,
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

# Standing yardstick pool: 16 on-disk oracle/decks, ordered pairs = n * n
# (mirrors included), matching matchup.py. Floors are half those totals.
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


def load_sweep_decks(repo: Path, restrict: list[str] | None) -> dict[str, dict[str, int]]:
    """Same composition as ``matchup.main``: oracle/decks, then extra files.

    Sweep has no ``--deck-file``, so the extra list is empty; the call is
    still the matchup path so pair counting cannot drift.
    """
    decks = load_deck_files(repo / "oracle" / "decks", restrict)
    decks.update(load_extra_deck_files([]))
    return decks


def _stage_total_target(target_games: int, default_per_pair: int) -> int:
    return math.ceil(target_games * default_per_pair / DEFAULT_FINAL_GAMES)


def _per_pair(total_target: int, pairs: int) -> int:
    return max(1, math.ceil(total_target / pairs))


def compute_sizing(
    args: argparse.Namespace,
    n_decks: int,
    names: list[str] | tuple[str, ...] | None = None,
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
    lines = [
        decks_line,
        f"pairs: {sizing.pairs} ({sizing.n_decks} x {sizing.n_decks}, mirrors included)",
    ]
    for name in GAME_STAGE_ORDER:
        if name not in shown:
            continue
        st = sizing.stage(name)
        lines.append(f"{name:<7} {st.games_per_pair:>3} games/pair x {st.pairs} = {st.total:>5}")
    return "\n".join(lines)


def sizing_metadata(sizing: SweepSizing, block: str) -> dict[str, Any]:
    return {
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
) -> SweepSizing:
    sizing = compute_sizing(args, n_decks, names)
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
    if args.smoke:
        if not flag_given(raw, "--decks"):
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
    return args


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
        return new_run(self._argv_list(), self.repo)

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

    def run_matchup(self, extra: list[str], out_stem: str, stage: str) -> None:
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
            self.run_matchup(main, f"{c.stem}-final", "final")
            rev = [
                "--policy-a",
                baseline,
                "--policy-b",
                c.spec,
                "--games",
                str(self.args.final_reverse),
            ]
            self.add_decks(rev)
            self.run_matchup(rev, f"{c.stem}-reverse", "final")
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
                self.run_matchup(mir, f"{c.stem}-mirror-{deck}", "final")
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
        header = ["index", "spec", "main", "reverse", "pooled"]
        header.extend(self.args.mirrors)
        lines.append("| " + " | ".join(header) + " |")
        lines.append("|" + "|".join("---" if i <= 1 else "---:" for i in range(len(header))) + "|")
        picks: list[BestPick] = []
        verdicts: list[str] = []
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
        lines.append("")
        lines.extend(verdicts)
        if verdicts:
            lines.append("")
        lines.append(format_best_line(picks))
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
        decks = load_sweep_decks(self.repo, self.args.decks)
        names = list(decks.keys())
        wanted = requested_stages(self.args)
        sizing = compute_sizing(self.args, len(names), names)
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
