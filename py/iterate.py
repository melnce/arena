#!/usr/bin/env python3
"""One-command value-net iteration: data → train → yardstick → summary → publish."""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from runlib import (  # noqa: E402
    ensure_worktree as runlib_ensure_worktree,
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
    rate_ci as _rate_ci,
    repo_root,
    reverse_candidate,
    run_tee,
    save_run as runlib_save_run,
    stage_seconds as runlib_stage_seconds,
    verdict,
)


STAGES = ("data", "train", "yard", "summary", "publish")
DEFAULT_EVAL = Path("engine") / "models" / "h0-linear-v1.json"


def have_torch() -> bool:
    return importlib.util.find_spec("torch") is not None


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    raw = list(sys.argv[1:] if argv is None else argv)
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--tag", required=True, help="iteration name; everything lands in <root>/<tag>/")
    p.add_argument("--seed", type=int, required=True, help="data seed S; ε-run uses S+1")
    p.add_argument("--root", default=None, help="results root (default: <repo>/results)")
    p.add_argument("--bot", default="h0", help="self-play policy (default: h0)")
    p.add_argument("--games", type=int, default=24, help="games per pair for data export (default: 24)")
    p.add_argument("--epsilon", type=float, default=0.1, help="ε-greedy on the second export (default: 0.1)")
    p.add_argument("--decks", nargs="*", default=None, help="restrict matchup decks (passed through)")
    p.add_argument(
        "--models",
        nargs="+",
        default=["linear", "mlp"],
        help="train_value --model values (default: linear mlp)",
    )
    p.add_argument("--data", nargs="*", default=[], help="extra sample dirs after the new exports")
    p.add_argument("--target", default="outcome", choices=("outcome", "search", "mix"))
    p.add_argument("--mix-weight", type=float, default=0.5)
    p.add_argument("--search-scale", type=float, default=60.0)
    p.add_argument("--eval", nargs="+", default=None, help="baseline nets for train_value --eval")
    p.add_argument("--epochs", type=int, default=None, help="passed to train_value when set")
    p.add_argument("--max-samples", type=int, default=None, help="passed to train_value when set")
    p.add_argument("--baseline", default="h0", help="yardstick opponent (default: h0)")
    p.add_argument(
        "--yard-seed",
        type=int,
        default=1,
        help="matchup seed for every yardstick run (default: 1; not the data --seed)",
    )
    p.add_argument("--yard-games", type=int, default=16)
    p.add_argument("--reverse-games", type=int, default=8)
    p.add_argument("--sanity-games", type=int, default=100)
    p.add_argument("--tp-games", type=int, default=1)
    p.add_argument("--mirrors", nargs="+", default=["royal-nattui"])
    p.add_argument("--mirror-games", type=int, default=200)
    p.add_argument("--threads", type=int, default=None)
    p.add_argument("--force", action="store_true", help="rerun every requested stage")
    p.add_argument("--only", choices=STAGES, help="run a single stage")
    p.add_argument("--skip-data", action="store_true")
    p.add_argument("--skip-train", action="store_true")
    p.add_argument("--skip-yard", action="store_true")
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
        help="tiny run for tests (2 decks, 1 game, linear, 2 epochs); explicit flags still override",
    )
    args = p.parse_args(raw)
    args._argv = raw
    if args.smoke:
        if not flag_given(raw, "--decks"):
            args.decks = ["basic-forest", "basic-rune"]
        if not flag_given(raw, "--games"):
            args.games = 1
        if not flag_given(raw, "--yard-games"):
            args.yard_games = 1
        if not flag_given(raw, "--reverse-games"):
            args.reverse_games = 1
        if not flag_given(raw, "--sanity-games"):
            args.sanity_games = 4
        if not flag_given(raw, "--mirror-games"):
            args.mirror_games = 2
        if not flag_given(raw, "--tp-games"):
            args.tp_games = 1
        if not flag_given(raw, "--epochs"):
            args.epochs = 2
        if not flag_given(raw, "--models"):
            args.models = ["linear"]
    return args


def requested_stages(args: argparse.Namespace) -> list[str]:
    if args.only:
        wanted = [args.only]
    else:
        wanted = list(STAGES)
    skips = {
        "data": args.skip_data,
        "train": args.skip_train,
        "yard": args.skip_yard,
        "summary": args.skip_summary,
        "publish": args.skip_publish,
    }
    wanted = [s for s in wanted if not skips.get(s)]
    if "publish" in wanted and not args.publish and args.only != "publish":
        wanted.remove("publish")
    return wanted


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
        self.eval_paths = self._eval_paths()
        self.extra_data = [str(Path(d).resolve()) for d in args.data]
        self.run_path = self.tag_dir / "RUN.json"
        self.run: dict[str, Any] = self._load_run()

    def _eval_paths(self) -> list[str]:
        if self.args.eval is not None:
            return [str(Path(p).resolve()) for p in self.args.eval]
        builtin = self.repo / DEFAULT_EVAL
        if builtin.is_file():
            return [str(builtin.resolve())]
        return []

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

    def py_tool(self, name: str) -> list[str]:
        return [sys.executable, str(self.py_dir / name)]

    def add_threads(self, cmd: list[str]) -> None:
        if self.args.threads is not None:
            cmd.extend(["--threads", str(self.args.threads)])

    def add_decks(self, cmd: list[str]) -> None:
        if self.args.decks:
            cmd.extend(["--decks", *self.args.decks])

    def planned_models(self) -> list[str]:
        return [m for m in self.args.models if m != "mlp" or have_torch()]

    def models_to_train(self) -> list[str]:
        out: list[str] = []
        for m in self.args.models:
            if m == "mlp" and not have_torch():
                print("skip: mlp (torch is not importable)", flush=True)
                continue
            out.append(m)
        return out

    def trained_models(self) -> list[str]:
        found: list[str] = []
        for m in self.args.models:
            if (self.tag_dir / f"{m}.json").is_file():
                found.append(m)
        return found

    def data_outputs_exist(self) -> bool:
        for name in ("data-e0", "data-e10"):
            if not (self.tag_dir / name / "meta.json").is_file():
                return False
            if not (self.tag_dir / f"{name}.json").is_file():
                return False
        return True

    def train_outputs_exist(self) -> bool:
        planned = self.planned_models()
        if not planned:
            return False
        for m in planned:
            if not (self.tag_dir / f"{m}.json").is_file():
                return False
            if not (self.tag_dir / f"{m}.report.json").is_file():
                return False
        return True

    def yard_outputs_exist(self) -> bool:
        models = self.trained_models()
        if not models:
            return False
        if not (self.tag_dir / "tp-h0.json").is_file():
            return False
        for m in models:
            needed = [
                f"main-{m}.json",
                f"reverse-{m}.json",
                f"sanity-{m}.json",
                f"tp-{m}.json",
            ]
            for deck in self.args.mirrors:
                needed.append(f"mirror-{deck}-{m}.json")
            if any(not (self.tag_dir / n).is_file() for n in needed):
                return False
        return True

    def summary_output_exist(self) -> bool:
        return (self.tag_dir / "SUMMARY.md").is_file()

    def publish_output_exist(self) -> bool:
        return publish_output_exist(self.publish_dir, self.args.tag, self.tag_dir)

    def candidate_spec(self, model: str) -> str:
        net = (self.tag_dir / f"{model}.json").resolve()
        path = str(net)
        if "," in path:
            raise SystemExit(
                f"yard: model path contains a comma (policy spec splits on commas): {path}"
            )
        return f"h0:value=net,net={path}"

    def run_matchup(self, extra: list[str], out_stem: str, stage: str) -> None:
        out_json = self.tag_dir / f"{out_stem}.json"
        log = self.tag_dir / f"{out_stem}.txt"
        cmd = matchup_argv(
            self.py_dir,
            extra,
            out_json,
            seed=self.args.yard_seed,
            threads=self.args.threads,
        )
        self.tee(cmd, log, stage)

    def stage_data(self) -> None:
        if not self.args.force and self.data_outputs_exist():
            print("skip: data", flush=True)
            return
        self.mark_start("data")
        for name, seed, eps in (
            ("data-e0", self.args.seed, None),
            ("data-e10", self.args.seed + 1, self.args.epsilon),
        ):
            export = (self.tag_dir / name).resolve()
            cmd = self.py_tool("matchup.py")
            cmd.extend(["--policy", self.args.bot, "--games", str(self.args.games), "--seed", str(seed)])
            cmd.extend(["--export", str(export)])
            if eps is not None:
                cmd.extend(["--export-epsilon", str(eps)])
            cmd.extend(["--out", str((self.tag_dir / f"{name}.json").resolve())])
            self.add_threads(cmd)
            self.add_decks(cmd)
            self.tee(cmd, self.tag_dir / f"{name}.txt", "data")
        self.mark_end("data")

    def stage_train(self) -> None:
        if not self.args.force and self.train_outputs_exist():
            print("skip: train", flush=True)
            return
        self.mark_start("train")
        data_dirs = [
            str((self.tag_dir / "data-e0").resolve()),
            str((self.tag_dir / "data-e10").resolve()),
            *self.extra_data,
        ]
        for m in self.models_to_train():
            out = (self.tag_dir / f"{m}.json").resolve()
            cmd = self.py_tool("train_value.py")
            cmd.extend(["--data", *data_dirs, "--model", m, "--out", str(out)])
            cmd.extend(["--target", self.args.target])
            cmd.extend(["--mix-weight", str(self.args.mix_weight)])
            cmd.extend(["--search-scale", str(self.args.search_scale)])
            if self.eval_paths:
                cmd.extend(["--eval", *self.eval_paths])
            if self.args.epochs is not None:
                cmd.extend(["--epochs", str(self.args.epochs)])
            if self.args.max_samples is not None:
                cmd.extend(["--max-samples", str(self.args.max_samples)])
            self.tee(cmd, self.tag_dir / f"train-{m}.txt", "train")
        self.mark_end("train")

    def stage_yard(self) -> None:
        # Guard every candidate path before any matchup (spec parser splits on commas).
        for m in self.args.models:
            path = str((self.tag_dir / f"{m}.json").resolve())
            if "," in path:
                raise SystemExit(
                    f"yard: model path contains a comma (policy spec splits on commas): {path}"
                )
        if not self.args.force and self.yard_outputs_exist():
            print("skip: yard", flush=True)
            return
        models = self.trained_models()
        if not models:
            raise SystemExit("yard: no trained model JSON under the tag directory")
        self.mark_start("yard")
        baseline = self.args.baseline
        for m in models:
            cand = self.candidate_spec(m)
            main = ["--policy-a", cand, "--policy-b", baseline, "--games", str(self.args.yard_games)]
            self.add_decks(main)
            self.run_matchup(main, f"main-{m}", "yard")
            rev = ["--policy-a", baseline, "--policy-b", cand, "--games", str(self.args.reverse_games)]
            self.add_decks(rev)
            self.run_matchup(rev, f"reverse-{m}", "yard")
            sanity = [
                "--policy-a",
                cand,
                "--policy-b",
                "random",
                "--games",
                str(self.args.sanity_games),
                "--decks",
                "basic-forest",
            ]
            self.run_matchup(sanity, f"sanity-{m}", "yard")
            tp = ["--policy", cand, "--games", str(self.args.tp_games)]
            self.add_decks(tp)
            self.run_matchup(tp, f"tp-{m}", "yard")
            for deck in self.args.mirrors:
                mir = [
                    "--policy-a",
                    cand,
                    "--policy-b",
                    baseline,
                    "--games",
                    str(self.args.mirror_games),
                    "--decks",
                    deck,
                ]
                self.run_matchup(mir, f"mirror-{deck}-{m}", "yard")
        tp_base = ["--policy", baseline, "--games", str(self.args.tp_games)]
        self.add_decks(tp_base)
        self.run_matchup(tp_base, "tp-h0", "yard")
        self.mark_end("yard")

    def stage_summary(self) -> None:
        if not self.args.force and self.summary_output_exist():
            print("skip: summary", flush=True)
            return
        self.mark_start("summary")
        self.mark_end("summary")
        text = self.build_summary()
        (self.tag_dir / "SUMMARY.md").write_text(text, encoding="utf-8")
        print(text, flush=True)

    def load_json(self, name: str) -> dict[str, Any]:
        path = self.tag_dir / name
        return json.loads(path.read_text(encoding="utf-8"))

    def stage_seconds(self, stage: str) -> float | None:
        return runlib_stage_seconds(self.run, stage)

    def build_summary(self) -> str:
        models = self.trained_models()
        lines: list[str] = []
        sha = self.run.get("git") or git_head(self.repo)
        seeds = f"{self.args.seed}, {self.args.seed + 1}"
        times = []
        total = 0.0
        for s in STAGES:
            sec = self.stage_seconds(s)
            if sec is None:
                continue
            times.append(f"{s} {sec:.1f}s")
            total += sec
        lines.append(f"# Iteration `{self.args.tag}`")
        lines.append("")
        lines.append(f"- tag: `{self.args.tag}`")
        lines.append(f"- seeds: {seeds}")
        lines.append(f"- bot: `{self.args.bot}`")
        lines.append(f"- baseline: `{self.args.baseline}`")
        lines.append(f"- engine: `{sha}`")
        lines.append(f"- wall: {total:.1f}s")
        lines.append(f"- stages: {', '.join(times) if times else 'n/a'}")
        lines.append("")

        lines.append("## data")
        lines.append("")
        lines.append("| run | games | samples | samples/game | first-player | g/s |")
        lines.append("|---|---:|---:|---:|---:|---:|")
        for name in ("data-e0", "data-e10"):
            meta = json.loads((self.tag_dir / name / "meta.json").read_text(encoding="utf-8"))
            match = self.load_json(f"{name}.json")
            summary = match.get("summary", {})
            games = int(summary.get("export_games", summary.get("games", meta.get("games", 0))))
            samples = int(summary.get("export_samples", meta.get("samples", 0)))
            per = samples / games if games else 0.0
            fp = float(summary.get("first_player_win_rate", 0.0))
            gps = float(summary.get("games_per_second", 0.0))
            lines.append(
                f"| `{name}` | {games} | {samples} | {per:.1f} | {fp:.3f} | {gps:.2f} |"
            )
        lines.append("")

        lines.append("## holdout")
        lines.append("")
        for m in models:
            report = self.load_json(f"{m}.report.json")
            lines.append(f"### `{m}`")
            lines.append("")
            lines.append(
                "| source | sign_acc | auc | mse | "
                "≤3 sign/auc/mse | 4–6 sign/auc/mse | ≥7 sign/auc/mse |"
            )
            lines.append("|---|---:|---:|---:|---|---|---|")
            rows: list[tuple[str, dict[str, Any] | None]] = [
                ("net", report.get("net")),
                ("v0", report.get("v0")),
                ("search_v", report.get("search_v")),
            ]
            ev = report.get("eval") or {}
            if isinstance(ev, dict):
                for key in ev:
                    rows.append((f"eval {key}", ev[key]))
            for label, block in rows:
                lines.append(f"| {label} | {_holdout_cells(block)} |")
            lines.append("")

        lines.append("## yardstick")
        lines.append("")
        header = ["model", "main", "reverse", "pooled", "sanity", "g/s vs h0"]
        header.extend(self.args.mirrors)
        lines.append("| " + " | ".join(header) + " |")
        lines.append("|" + "|".join("---" if i == 0 else "---:" for i in range(len(header))) + "|")
        tp_h0 = self.load_json("tp-h0.json")
        base_gps = float(tp_h0.get("summary", {}).get("games_per_second", 0.0))
        verdicts: list[str] = []
        for m in models:
            main_doc = self.load_json(f"main-{m}.json")
            rev_doc = self.load_json(f"reverse-{m}.json")
            main_s = main_doc["summary"]
            rev_s = rev_doc["summary"]
            san_s = self.load_json(f"sanity-{m}.json")["summary"]
            tp_s = self.load_json(f"tp-{m}.json")["summary"]
            main_rate = float(main_s["policy_a_win_rate"])
            main_ci = (float(main_s["wilson95"][0]), float(main_s["wilson95"][1]))
            rev_rate, rev_ci = reverse_candidate(rev_s)
            pool_rate, pool_ci = pooled_candidate(main_doc, rev_doc, tag=m)
            san_rate = float(san_s["policy_a_win_rate"])
            san_ci = (float(san_s["wilson95"][0]), float(san_s["wilson95"][1]))
            gps = float(tp_s.get("games_per_second", 0.0))
            vs = f"{gps:.2f} / {base_gps:.2f}"
            cells = [
                m,
                _rate_ci(main_rate, main_ci),
                _rate_ci(rev_rate, rev_ci),
                _rate_ci(pool_rate, pool_ci),
                _rate_ci(san_rate, san_ci),
                vs,
            ]
            for deck in self.args.mirrors:
                ms = self.load_json(f"mirror-{deck}-{m}.json")["summary"]
                cells.append(
                    _rate_ci(
                        float(ms["policy_a_win_rate"]),
                        (float(ms["wilson95"][0]), float(ms["wilson95"][1])),
                    )
                )
            lines.append("| " + " | ".join(cells) + " |")
            verdicts.append(
                format_verdict_line(
                    m, main_rate, main_ci, rev_rate, rev_ci, pool_rate, pool_ci
                )
            )
        lines.append("")
        lines.append("## verdict")
        lines.append("")
        lines.extend(verdicts)
        lines.append("")
        return "\n".join(lines)

    def _tee_publish(self, argv: list[str], log_path: Path, append: bool = False) -> None:
        self.tee(argv, log_path, "publish", append=append)

    def stage_publish(self) -> None:
        if not self.args.force and self.publish_output_exist():
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

    def ensure_worktree(self, log: Path) -> None:
        runlib_ensure_worktree(
            self.repo,
            self.publish_dir,
            self.args.publish_remote,
            self.args.publish_branch,
            log,
            tee=self._tee_publish,
        )

    def go(self) -> int:
        self.save_run()
        wanted = requested_stages(self.args)
        dispatch = {
            "data": self.stage_data,
            "train": self.stage_train,
            "yard": self.stage_yard,
            "summary": self.stage_summary,
            "publish": self.stage_publish,
        }
        for stage in wanted:
            dispatch[stage]()
        return 0


def _fmt_num(x: Any) -> str:
    if x is None:
        return "—"
    try:
        f = float(x)
    except (TypeError, ValueError):
        return "—"
    if f != f:
        return "nan"
    return f"{f:.3f}"


def _band_triple(block: dict[str, Any], band: str) -> str:
    m = block.get(band) or {}
    return f"{_fmt_num(m.get('sign_acc'))}/{_fmt_num(m.get('auc'))}/{_fmt_num(m.get('mse'))}"


def _holdout_cells(block: dict[str, Any] | None) -> str:
    if not block:
        return "— | — | — | — | — | —"
    overall = block.get("overall") or {}
    return (
        f"{_fmt_num(overall.get('sign_acc'))} | {_fmt_num(overall.get('auc'))} | "
        f"{_fmt_num(overall.get('mse'))} | {_band_triple(block, '<=3')} | "
        f"{_band_triple(block, '4-6')} | {_band_triple(block, '>=7')}"
    )


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    return Runner(args).go()


if __name__ == "__main__":
    raise SystemExit(main())
