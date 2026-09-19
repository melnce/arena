#!/usr/bin/env python3
"""One-command policy-spec sweep: screen → finalists → final → summary → publish."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

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
    p.add_argument("--screen-games", type=int, default=4, help="games per pair on the screen (default: 4)")
    p.add_argument("--final-games", type=int, default=16, help="games per pair on the final (default: 16)")
    p.add_argument(
        "--final-reverse",
        type=int,
        default=8,
        help="games per pair on the reverse seat (default: 8)",
    )
    p.add_argument("--tp-games", type=int, default=1, help="throughput games per pair (default: 1)")
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
    if args.smoke:
        if not flag_given(raw, "--decks"):
            args.decks = ["basic-forest", "basic-rune"]
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

    def go(self) -> int:
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
