#!/usr/bin/env python3
"""Build docs/research/sweep-ledger.md from origin/results sweep*/ folders."""

from __future__ import annotations

import json
import math
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

_REPO = Path(__file__).resolve().parents[2]
_OUT = _REPO / "docs" / "research" / "sweep-ledger.md"
_RESULTS_REF = "origin/results"


def git_show(path: str) -> bytes | None:
    r = subprocess.run(
        ["git", "-C", str(_REPO), "show", f"{_RESULTS_REF}:{path}"],
        capture_output=True,
    )
    if r.returncode != 0:
        return None
    return r.stdout


def list_sweeps() -> list[str]:
    r = subprocess.run(
        ["git", "-C", str(_REPO), "ls-tree", "--name-only", _RESULTS_REF],
        capture_output=True,
        text=True,
        check=True,
    )
    out: list[str] = []
    for line in r.stdout.splitlines():
        if re.fullmatch(r"sweep\d+b?", line):
            out.append(line)
    return sorted(out, key=_sweep_sort_key)


def _sweep_sort_key(name: str) -> tuple[int, int]:
    m = re.match(r"sweep(\d+)(b)?", name)
    if not m:
        return (0, 0)
    return (int(m.group(1)), 1 if m.group(2) else 0)


def parse_spec_keys(spec: str) -> dict[str, str]:
    if not spec.startswith("h0"):
        return {"_raw": spec}
    body = spec[2:]
    if body.startswith(":"):
        body = body[1:]
    elif body.startswith("-"):
        return {}
    keys: dict[str, str] = {}
    for token in body.split(","):
        token = token.strip()
        if not token:
            continue
        if "=" in token:
            k, v = token.split("=", 1)
            keys[k.strip()] = v.strip()
        else:
            keys[token] = ""
    return keys


def shorten_path(key: str, val: str) -> str:
    if key in ("net", "order", "oorder", "mull") and ("/" in val or "\\" in val):
        return val.replace("\\", "/").rsplit("/", 1)[-1]
    return val


def candidate_delta(baseline: str, candidate: str) -> str:
    b = parse_spec_keys(baseline)
    c = parse_spec_keys(candidate)
    if "_raw" in b or "_raw" in c:
        return candidate if candidate != baseline else "(same)"
    parts: list[str] = []
    for k in sorted(set(b) | set(c)):
        bv, cv = b.get(k), c.get(k)
        if bv == cv:
            continue
        if cv is None:
            parts.append(f"{k}=(removed)")
        elif bv is None:
            parts.append(f"{k}={shorten_path(k, cv)}")
        else:
            parts.append(f"{k}={shorten_path(k, cv)}")
    return ", ".join(parts) if parts else "(same)"


def elo_from_rate(p: float) -> str:
    if p <= 0.0 or p >= 1.0:
        return "—"
    return f"{400.0 * math.log10(p / (1.0 - p)):+.1f}"


def load_json(path: str) -> dict | list | None:
    raw = git_show(path)
    if raw is None:
        return None
    try:
        return json.loads(raw.decode("utf-8"))
    except json.JSONDecodeError:
        return None


def screen_counts(tag: str, idx: int) -> tuple[int, int, str]:
    stem = f"c{idx:02d}-screen.json"
    doc = load_json(f"{tag}/{stem}")
    if doc is None or not isinstance(doc, dict):
        return -1, -1, "missing screen json"
    summary = doc.get("summary")
    if not isinstance(summary, dict):
        return -1, -1, "no summary in screen"
    games = int(summary.get("games", summary.get("decisive", 0)))
    wins = int(round(float(summary.get("policy_a_win_rate", 0.5)) * games))
    return wins, games, ""


def stage_wins(tag: str, idx: int, stage: str) -> tuple[int, int, str]:
    stem = f"c{idx:02d}-{stage}.json"
    doc = load_json(f"{tag}/{stem}")
    if doc is None or not isinstance(doc, dict):
        return -1, -1, f"missing {stage}"
    summary = doc.get("summary")
    if not isinstance(summary, dict):
        return -1, -1, f"no summary in {stage}"
    n = int(summary.get("decisive", summary.get("games", 0)))
    w = int(round(float(summary.get("policy_a_win_rate", 0.5)) * n))
    return w, n, ""


def pooled_main_reverse(
    tag: str, idx: int
) -> tuple[float | None, tuple[float, float] | None, int, int, str]:
    mw, mn, err_m = stage_wins(tag, idx, "final")
    rw, rn, err_r = stage_wins(tag, idx, "reverse")
    if mw < 0 or rw < 0:
        return None, None, 0, 0, err_m or err_r
    # Reverse: candidate is B; wins = decisive - a_wins on reverse json
    rw_candidate = rn - rw if rn >= 0 else 0
    w = mw + rw_candidate
    n = mn + rn
    if n == 0:
        return None, None, 0, 0, "zero pooled games"
    p = w / n
    z = 1.959963984540054
    denom = 1 + z * z / n
    center = (p + z * z / (2 * n)) / denom
    margin = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / denom
    return p, (center - margin, center + margin), w, n, ""


def parse_summary_md(tag: str) -> dict[str, str]:
    raw = git_show(f"{tag}/SUMMARY.md")
    if raw is None:
        return {}
    text = raw.decode("utf-8", errors="replace")
    out: dict[str, str] = {}
    for line in text.splitlines():
        if line.startswith("- baseline:"):
            out["baseline"] = line.split(":", 1)[1].strip().strip("`")
        if line.startswith("- seed:"):
            out["seed"] = line.split(":", 1)[1].strip()
        if line.startswith("verdict:"):
            out["verdict"] = line.split(":", 1)[1].strip()
        if "early stop:" in line:
            out["early_stop"] = line.strip()
    m = re.search(r"^# Sweep `([^`]+)`", text, re.M)
    if m:
        out["tag"] = m.group(1)
    return out


def run_date(run: dict) -> str:
    stages = run.get("stages") or {}
    screen = stages.get("screen") or {}
    start = screen.get("start") or run.get("created") or ""
    if not start:
        return ""
    try:
        dt = datetime.fromisoformat(start.replace("Z", "+00:00"))
        return dt.strftime("%Y-%m-%d")
    except ValueError:
        return start[:10]


def candidates_from_run(run: dict) -> tuple[str, list[tuple[int, str]]]:
    argv = run.get("argv") or []
    baseline = ""
    cands: list[str] = []
    i = 0
    while i < len(argv):
        if argv[i] == "--baseline" and i + 1 < len(argv):
            baseline = argv[i + 1]
            i += 2
            continue
        if argv[i] == "--candidates":
            i += 1
            while i < len(argv) and not argv[i].startswith("-"):
                cands.append(argv[i])
                i += 1
            continue
        i += 1
    if not cands:
        doc = None
        tag = ""
        for a in argv:
            if a.startswith("sweep"):
                tag = a
        if tag:
            doc = load_json(f"{tag}/candidates.json")
        if isinstance(doc, list):
            out = [(int(c["index"]), str(c["spec"])) for c in doc if "spec" in c]
            return baseline, out
    indexed = list(enumerate(cands, start=1))
    return baseline, [(i, s) for i, s in indexed]


@dataclass
class Row:
    name: str
    date: str
    seed: str
    baseline: str
    cand_idx: int
    delta: str
    screen: str
    final_main: str
    final_rev: str
    pooled: str
    elo: str
    early: str
    verdict: str
    note: str


def build_row(tag: str) -> list[Row]:
    run_raw = git_show(f"{tag}/RUN.json")
    if run_raw is None:
        return [
            Row(
                tag,
                "",
                "",
                "",
                0,
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "layout: no RUN.json",
            )
        ]
    try:
        run = json.loads(run_raw.decode("utf-8"))
    except json.JSONDecodeError:
        return [
            Row(tag, "", "", "", 0, "", "", "", "", "", "", "", "", "layout: bad RUN.json")
        ]
    sm = parse_summary_md(tag)
    baseline, cands = candidates_from_run(run)
    if not baseline and sm.get("baseline"):
        baseline = sm["baseline"]
    if not cands:
        # legacy: candidates only in SUMMARY screen table — skip parsing
        return [
            Row(
                tag,
                run_date(run),
                sm.get("seed", ""),
                baseline or "?",
                0,
                "",
                "",
                "",
                "",
                "",
                "",
                sm.get("early_stop", ""),
                sm.get("verdict", ""),
                "layout: no machine-readable candidates",
            )
        ]
    rows: list[Row] = []
    for idx, spec in cands:
        sw, sg, serr = screen_counts(tag, idx)
        screen = f"{sw}/{sg}" if sw >= 0 else serr
        mw, mn, _ = stage_wins(tag, idx, "final")
        rw, rn, _ = stage_wins(tag, idx, "reverse")
        main_s = f"{mw}/{mn}" if mw >= 0 else "—"
        if rw >= 0 and rn >= 0:
            rev_s = f"{rn - rw}/{rn}"
        else:
            rev_s = "—"
        p, ci, pw, pn, perr = pooled_main_reverse(tag, idx)
        if p is not None and ci is not None:
            pooled = f"{pw}/{pn} ({p:.3f} [{ci[0]:.3f}, {ci[1]:.3f}])"
            elo = elo_from_rate(p)
        else:
            pooled = perr or "—"
            elo = "—"
        rows.append(
            Row(
                tag,
                run_date(run),
                sm.get("seed", str(run.get("seed", ""))),
                baseline,
                idx,
                candidate_delta(baseline, spec),
                screen,
                main_s,
                rev_s,
                pooled,
                elo,
                sm.get("early_stop", ""),
                sm.get("verdict", "") if idx == cands[0][0] else "",
                "",
            )
        )
    return rows


def main() -> int:
    sweeps = list_sweeps()
    all_rows: list[Row] = []
    for tag in sweeps:
        all_rows.extend(build_row(tag))

    lines = [
        "# Sweep ledger (`origin/results`)",
        "",
        "Auto-generated by `tools/research/sweep_ledger.py`. "
        f"{len(sweeps)} sweep folders, {len(all_rows)} candidate rows.",
        "",
        "| sweep | date | seed | cand | delta vs baseline | screen W/G | "
        "final main | final rev | pooled | Elo | early stop | verdict / note |",
        "|---|---|---:|---:|---|---|---|---|---|---:|---|---|",
    ]
    for r in all_rows:
        verdict_note = r.verdict or r.note
        if r.verdict and r.note:
            verdict_note = f"{r.verdict}; {r.note}"
        delta = r.delta.replace("|", "\\|")
        lines.append(
            f"| {r.name} | {r.date} | {r.seed} | {r.cand_idx} | `{delta}` | "
            f"{r.screen} | {r.final_main} | {r.final_rev} | {r.pooled} | {r.elo} | "
            f"{(r.early[:40] + '…') if len(r.early) > 40 else r.early} | "
            f"{verdict_note.replace('|', '/')} |"
        )

    _OUT.parent.mkdir(parents=True, exist_ok=True)
    _OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {_OUT} ({len(all_rows)} rows)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
