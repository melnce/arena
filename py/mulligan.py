#!/usr/bin/env python3
"""Learned mulligan data collection and table fitting."""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import gzip
import json
import math
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from matchup import (  # noqa: E402
    DEFAULT_POOL,
    load_deck_files,
    load_extra_deck_files,
    resolve_selected_decks,
)
from runlib import (  # noqa: E402
    git_head,
    load_run,
    mark_end,
    mark_start,
    matchup_argv,
    new_run,
    publish_tag,
    repo_root,
    run_tee,
    save_run,
)

PUBLISH_FILES = frozenset(
    {"RUN.json", "FIT.md", "fit.json", "table.json", "observations.csv.gz"}
)


def load_card_index(repo: Path) -> dict[str, dict[str, Any]]:
    out: dict[str, dict[str, Any]] = {}
    cards_dir = repo / "cards"
    for path in cards_dir.rglob("*.json"):
        if "official" in path.parts:
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        cid = data.get("id")
        if isinstance(cid, str):
            out[cid] = data
    return out


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    raw = list(sys.argv[1:] if argv is None else argv)
    if not raw or raw[0] not in {"data", "fit"}:
        raise SystemExit("usage: mulligan.py data|fit ...")
    cmd = raw[0]
    rest = raw[1:]
    if cmd == "data":
        return _parse_data(rest)
    return _parse_fit(rest)


def _parse_data(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Collect mulligan matchup chunks.")
    p.add_argument("--tag", required=True)
    p.add_argument("--pool", default=None)
    p.add_argument("--decks", nargs="*", default=None)
    p.add_argument("--games-per-pair", type=int, required=True)
    p.add_argument("--chunks", type=int, required=True)
    p.add_argument("--seed", type=int, required=True)
    p.add_argument("--policy", default="h0:mull=random")
    p.add_argument("--threads", type=int, default=None)
    p.add_argument("--root", default=None)
    p.add_argument("--force", action="store_true")
    p.add_argument("--publish", action="store_true")
    p.add_argument("--publish-remote", default="origin")
    p.add_argument("--publish-branch", default="results")
    p.add_argument("--publish-dir", default=None)
    args = p.parse_args(argv)
    args.command = "data"
    args._argv = ["data", *argv]
    if args.pool is not None and args.decks is not None:
        raise SystemExit("use --pool or --decks, not both")
    return args


def _parse_fit(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Fit a mulligan keep table from chunks.")
    p.add_argument("--tag", required=True)
    p.add_argument("--z", type=float, default=1.5)
    p.add_argument("--min-n", type=int, default=30)
    p.add_argument("--root", default=None)
    p.add_argument("--publish", action="store_true")
    p.add_argument("--publish-remote", default="origin")
    p.add_argument("--publish-branch", default="results")
    p.add_argument("--publish-dir", default=None)
    args = p.parse_args(argv)
    args.command = "fit"
    args._argv = ["fit", *argv]
    return args


def tag_dir(root: Path, tag: str) -> Path:
    return root / tag


def chunk_path(tag_dir: Path, i: int) -> Path:
    return tag_dir / f"chunk-{i}.json"


def validate_sizing(games_per_pair: int, chunks: int) -> int:
    if games_per_pair % chunks != 0:
        raise SystemExit(f"games-per-pair {games_per_pair} not divisible by chunks {chunks}")
    per_chunk = games_per_pair // chunks
    if per_chunk % 2 != 0:
        raise SystemExit(
            f"games per chunk {per_chunk} must be even (first=alternate needs even games per pair)"
        )
    return per_chunk


def format_data_sizing(
    pool_name: str,
    names: list[str],
    games_per_pair: int,
    chunks: int,
) -> str:
    n = len(names)
    pairs = n * n
    per_chunk = games_per_pair // chunks
    total = pairs * games_per_pair
    decks_line = f"decks: {n} ({', '.join(names)})" if names else f"decks: {n}"
    return "\n".join(
        [
            f"pool: {pool_name}",
            decks_line,
            f"pairs: {pairs} ({n} x {n}, mirrors included)",
            f"games   {per_chunk:>3} games/pair x {pairs} = {pairs * per_chunk:>5} per chunk",
            f"chunks: {chunks}",
            f"total:  {total:>5} games",
        ]
    )


def chunk_valid(path: Path) -> bool:
    if not path.is_file():
        return False
    try:
        json.loads(path.read_text(encoding="utf-8"))
        return True
    except json.JSONDecodeError:
        return False


def matchup_extra(args: argparse.Namespace, pool_name: str, decks: dict[str, dict[str, int]]) -> list[str]:
    extra = ["--records", "--policy", args.policy]
    if args.decks is not None:
        extra.extend(["--decks", *args.decks])
    else:
        extra.extend(["--pool", pool_name if args.pool is not None else DEFAULT_POOL])
    if args.threads is not None:
        extra.extend(["--threads", str(args.threads)])
    return extra


def cmd_data(args: argparse.Namespace) -> None:
    repo = repo_root()
    root = Path(args.root) if args.root else repo / "results"
    td = tag_dir(root, args.tag)
    td.mkdir(parents=True, exist_ok=True)
    decks_dir = repo / "oracle" / "decks"
    pool_name, decks = resolve_selected_decks(decks_dir, args.pool, args.decks)
    decks.update(load_extra_deck_files([]))
    names = sorted(decks.keys())
    per_chunk = validate_sizing(args.games_per_pair, args.chunks)
    print(format_data_sizing(pool_name, names, args.games_per_pair, args.chunks), flush=True)

    run_path = td / "RUN.json"
    run = load_run(run_path) or new_run([sys.executable, str(Path(__file__).resolve()), *args._argv], repo)
    run.setdefault("chunks", {})
    extra_base = matchup_extra(args, pool_name, decks)

    for i in range(args.chunks):
        out = chunk_path(td, i)
        if not args.force and chunk_valid(out):
            print(f"skip: chunk-{i}", flush=True)
            continue
        stage = f"chunk-{i}"
        mark_start(run, run_path, stage, run["argv"], repo)
        cmd = matchup_argv(
            _HERE,
            [*extra_base, "--games", str(per_chunk)],
            out,
            seed=args.seed + i,
        )
        log = td / f"chunk-{i}.txt"
        run_tee(cmd, log, append=False)
        mark_end(run, run_path, stage, run["argv"], repo)
        save_run(run_path, run, run["argv"], repo)

    if args.publish:
        publish_mulligan(repo, args, td)


def load_chunks(td: Path) -> list[dict[str, Any]]:
    paths = sorted(td.glob("chunk-*.json"))
    if not paths:
        raise SystemExit(f"no chunk JSON in {td}")
    return [json.loads(p.read_text(encoding="utf-8")) for p in paths]


def decks_in_chunks(chunks: list[dict[str, Any]]) -> list[str]:
    """Deck stems present in chunk matchup JSON (matrix keys)."""
    names: set[str] = set()
    for doc in chunks:
        matrix = doc.get("matrix") or {}
        if isinstance(matrix, dict):
            names.update(matrix.keys())
    if not names:
        raise SystemExit("chunks contain no matrix deck names")
    return sorted(names)


def pool_meta_from_run(run: dict[str, Any] | None, deck_names: list[str]) -> str | list[str]:
    """Pool name or deck list for table meta — from RUN.json argv, not the matrix."""
    argv = (run or {}).get("argv") or []
    for i, arg in enumerate(argv):
        if arg == "--pool" and i + 1 < len(argv):
            return argv[i + 1]
        if arg == "--decks":
            decks: list[str] = []
            j = i + 1
            while j < len(argv) and not argv[j].startswith("-"):
                decks.append(argv[j])
                j += 1
            if decks:
                return decks
    return deck_names


@dataclass
class Observation:
    deck: str
    seat: str
    opponent_deck: str
    opponent_class: str
    hand: list[str]
    swap: list[bool]
    won: int


def deck_class(cards: dict[str, dict[str, Any]], deck: dict[str, int]) -> str:
    counts: dict[str, int] = defaultdict(int)
    for cid, n in deck.items():
        card = cards.get(str(cid), {})
        cls = card.get("class")
        if cls and cls != "neutral":
            counts[str(cls)] += int(n)
    if not counts:
        return "neutral"
    return max(counts, key=counts.get)


def observations_from_chunks(
    chunks: list[dict[str, Any]],
    decks: dict[str, dict[str, int]],
    cards: dict[str, dict[str, Any]],
) -> list[Observation]:
    out: list[Observation] = []
    for doc in chunks:
        for rec in doc.get("records") or []:
            first = rec["first"]
            winner = rec.get("winner")
            for side, deck_key, opp_key, mull_key in (
                ("a", "a", "b", "mull_a"),
                ("b", "b", "a", "mull_b"),
            ):
                mull = rec.get(mull_key)
                if not mull:
                    continue
                seat = "first" if first == side else "second"
                won = 1 if winner == side else 0
                opp = rec[opp_key]
                out.append(
                    Observation(
                        deck=rec[deck_key],
                        seat=seat,
                        opponent_deck=opp,
                        opponent_class=deck_class(cards, decks[opp]),
                        hand=list(mull["hand"]),
                        swap=[bool(x) for x in mull["swap"]],
                        won=won,
                    )
                )
    return out


def write_observations(path: Path, rows: list[Observation]) -> None:
    with gzip.open(path, "wt", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "deck",
                "seat",
                "opponent_deck",
                "opponent_class",
                "hand",
                "swap",
                "won",
            ]
        )
        for o in rows:
            w.writerow(
                [
                    o.deck,
                    o.seat,
                    o.opponent_deck,
                    o.opponent_class,
                    ";".join(o.hand),
                    ";".join("1" if s else "0" for s in o.swap),
                    o.won,
                ]
            )


@dataclass
class CardStats:
    n_k: int = 0
    wins_k: int = 0
    n_s: int = 0
    wins_s: int = 0
    excluded_split: int = 0

    def add(self, kept: bool, won: int) -> None:
        if kept:
            self.n_k += 1
            self.wins_k += won
        else:
            self.n_s += 1
            self.wins_s += won


def rate(wins: int, n: int) -> float:
    return wins / n if n else 0.0


def effect_se(n_k: int, wins_k: int, n_s: int, wins_s: int) -> tuple[float, float]:
    p_k = rate(wins_k, n_k)
    p_s = rate(wins_s, n_s)
    effect = p_k - p_s
    if n_k == 0 or n_s == 0:
        return effect, 0.0
    se = math.sqrt(p_k * (1 - p_k) / n_k + p_s * (1 - p_s) / n_s)
    return effect, se


def z_score(effect: float, se: float) -> float:
    return effect / se if se > 0 else 0.0


def rule_keep(cost: int) -> bool:
    return cost < 4


def card_contribution(obs: Observation, card: str) -> tuple[bool, int] | None:
    """Return (kept, won) for this card in obs, or None if split copies."""
    slots = [i for i, c in enumerate(obs.hand) if c == card]
    if not slots:
        return None
    kept_flags = [not obs.swap[i] for i in slots]
    if any(kept_flags) and not all(kept_flags):
        return None
    return kept_flags[0], obs.won


def gather_stats(
    observations: list[Observation],
) -> tuple[dict[tuple[str, str, str], CardStats], int]:
    """Key: (deck, seat, card). Returns stats and excluded-split count."""
    stats: dict[tuple[str, str, str], CardStats] = defaultdict(CardStats)
    excluded = 0
    for obs in observations:
        seen: set[str] = set()
        for card in obs.hand:
            if card in seen:
                continue
            seen.add(card)
            contrib = card_contribution(obs, card)
            if contrib is None:
                stats[(obs.deck, obs.seat, card)].excluded_split += 1
                excluded += 1
                continue
            kept, won = contrib
            stats[(obs.deck, obs.seat, card)].add(kept, won)
    return stats, excluded


def seat_stats(
    per_seat: dict[tuple[str, str, str], CardStats],
    deck: str,
    seat: str,
    card: str,
) -> CardStats:
    return per_seat.get((deck, seat, card), CardStats())


def pooled_stats(deck: str, card: str, per_seat: dict[tuple[str, str, str], CardStats]) -> CardStats:
    out = CardStats()
    for seat in ("first", "second"):
        s = seat_stats(per_seat, deck, seat, card)
        out.n_k += s.n_k
        out.wins_k += s.wins_k
        out.n_s += s.n_s
        out.wins_s += s.wins_s
        out.excluded_split += s.excluded_split
    return out


def decide_keep(
    deck: str,
    seat: str,
    card: str,
    cost: int,
    per_seat: dict[tuple[str, str, str], CardStats],
    z_thr: float,
    min_n: int,
) -> tuple[bool, str]:
    s = seat_stats(per_seat, deck, seat, card)
    eff, se = effect_se(s.n_k, s.wins_k, s.n_s, s.wins_s)
    z = z_score(eff, se)
    if s.n_k >= min_n and s.n_s >= min_n and abs(z) >= z_thr:
        return eff > 0, "seat"
    pool = pooled_stats(deck, card, per_seat)
    peff, pse = effect_se(pool.n_k, pool.wins_k, pool.n_s, pool.wins_s)
    pz = z_score(peff, pse)
    sf = seat_stats(per_seat, deck, "first", card)
    ss = seat_stats(per_seat, deck, "second", card)
    e_first, se_first = effect_se(sf.n_k, sf.wins_k, sf.n_s, sf.wins_s)
    e_second, se_second = effect_se(ss.n_k, ss.wins_k, ss.n_s, ss.wins_s)
    denom = math.sqrt(se_first**2 + se_second**2)
    seat_sim = denom > 0 and abs(e_first - e_second) / denom < z_thr
    if pool.n_k >= min_n and pool.n_s >= min_n and abs(pz) >= z_thr and seat_sim:
        return peff > 0, "pooled"
    return rule_keep(cost), "rule"


def chi2_sf(x: float, df: float) -> float:
    """Survival function for chi-square(df). df=1 uses the normal tail."""
    if x <= 0 or df <= 0:
        return 1.0
    if df == 1.0:
        return math.erfc(math.sqrt(x / 2.0))
    # Regularized incomplete gamma upper tail via Lentz continued fraction.
    a = df / 2.0
    z = x / 2.0
    return _gamma_q(a, z)


def _gamma_q(a: float, x: float) -> float:
    if x < a + 1.0:
        return 1.0 - _gamma_p(a, x)
    return _gamma_cf(a, x)


def _gamma_p(a: float, x: float) -> float:
    if x <= 0:
        return 0.0
    term = 1.0 / a
    total = term
    n = 1
    while n < 200:
        term *= x / (a + n)
        total += term
        if abs(term) < 1e-12 * abs(total):
            break
        n += 1
    return total * math.exp(-x + a * math.log(x) - math.lgamma(a))


def _gamma_cf(a: float, x: float) -> float:
    tiny = 1e-30
    fpmin = 1e-30
    b = x + 1.0 - a
    c = 1.0 / fpmin
    d = 1.0 / b
    h = d
    for i in range(1, 200):
        an = -i * (i - a)
        b += 2.0
        d = an * d + b
        if abs(d) < fpmin:
            d = fpmin
        c = b + an / c
        if abs(c) < fpmin:
            c = fpmin
        d = 1.0 / d
        delta = d * c
        h *= delta
        if abs(delta - 1.0) < 1e-12:
            break
    return math.exp(-x + a * math.log(x) - math.lgamma(a)) * h


def class_effects(
    observations: list[Observation],
    deck: str,
    card: str,
) -> dict[str, tuple[float, float, int, int]]:
    """Per opponent class: (effect, se, n_k, n_s)."""
    by_class: dict[str, list[tuple[bool, int]]] = defaultdict(list)
    for obs in observations:
        if obs.deck != deck:
            continue
        contrib = card_contribution(obs, card)
        if contrib is None:
            continue
        kept, won = contrib
        by_class[obs.opponent_class].append((kept, won))
    out: dict[str, tuple[float, float, int, int]] = {}
    for cls, rows in by_class.items():
        n_k = sum(1 for k, _ in rows if k)
        wins_k = sum(w for k, w in rows if k)
        n_s = sum(1 for k, _ in rows if not k)
        wins_s = sum(w for k, w in rows if not k)
        eff, se = effect_se(n_k, wins_k, n_s, wins_s)
        out[cls] = (eff, se, n_k, n_s)
    return out


def heterogeneity_classes(
    classes: dict[str, tuple[float, float, int, int]],
    min_n: int,
) -> list[str]:
    """Classes with enough kept and sent-back observations for the chi² test."""
    return [
        cls
        for cls, (_, _, n_k, n_s) in classes.items()
        if n_k >= min_n and n_s >= min_n
    ]


def cmd_fit(args: argparse.Namespace) -> None:
    import arena

    repo = repo_root()
    root = Path(args.root) if args.root else repo / "results"
    td = tag_dir(root, args.tag)
    chunks = load_chunks(td)
    deck_names = decks_in_chunks(chunks)
    decks_dir = repo / "oracle" / "decks"
    decks = load_deck_files(decks_dir, deck_names)
    run = load_run(td / "RUN.json")
    pool_meta = pool_meta_from_run(run, deck_names)
    cards = load_card_index(repo)

    observations = observations_from_chunks(chunks, decks, cards)
    write_observations(td / "observations.csv.gz", observations)
    per_seat, excluded_split = gather_stats(observations)

    card_costs: dict[str, int] = {}
    for deck_map in decks.values():
        for cid in deck_map:
            if cid not in card_costs:
                card_costs[cid] = int(cards.get(cid, {}).get("cost", 99))

    table_decks: dict[str, Any] = {}
    fit_cards: list[dict[str, Any]] = []
    rule_diffs: list[str] = []
    class_candidates: list[str] = []

    for deck_name in sorted(decks):
        fp = arena.deck_fingerprint(decks[deck_name])
        first_map: dict[str, bool] = {}
        second_map: dict[str, bool] = {}
        deck_rows: list[dict[str, Any]] = []
        for cid in sorted(decks[deck_name], key=lambda x: int(x)):
            cost = card_costs[cid]
            row: dict[str, Any] = {
                "card": cid,
                "copies": decks[deck_name][cid],
                "cost": cost,
                "rule_keep": rule_keep(cost),
            }
            seat_decisions: dict[str, bool] = {}
            for seat in ("first", "second"):
                s = seat_stats(per_seat, deck_name, seat, cid)
                eff, se = effect_se(s.n_k, s.wins_k, s.n_s, s.wins_s)
                keep, how = decide_keep(deck_name, seat, cid, cost, per_seat, args.z, args.min_n)
                seat_decisions[seat] = keep
                row[seat] = {
                    "n_k": s.n_k,
                    "n_s": s.n_s,
                    "effect": eff,
                    "se": se,
                    "z": z_score(eff, se),
                    "ci95": (eff - 1.96 * se, eff + 1.96 * se),
                    "keep": keep,
                    "how": how,
                }
            pool = pooled_stats(deck_name, cid, per_seat)
            peff, pse = effect_se(pool.n_k, pool.wins_k, pool.n_s, pool.wins_s)
            row["pooled"] = {
                "n_k": pool.n_k,
                "n_s": pool.n_s,
                "effect": peff,
                "se": pse,
                "z": z_score(peff, pse),
                "ci95": (peff - 1.96 * pse, peff + 1.96 * pse),
            }
            row["differs_from_rule"] = any(
                seat_decisions[s] != rule_keep(cost) for s in ("first", "second")
            )
            if row["differs_from_rule"]:
                rule_diffs.append(f"{deck_name} {cid}")
            first_map[cid] = seat_decisions["first"]
            second_map[cid] = seat_decisions["second"]
            deck_rows.append(row)
            fit_cards.append({"deck": deck_name, "card": cid, **row})

            if pool.n_k >= args.min_n and pool.n_s >= args.min_n:
                classes = class_effects(observations, deck_name, cid)
                used = heterogeneity_classes(classes, min_n=10)
                if len(used) >= 2:
                    chi2 = 0.0
                    for c in used:
                        e_c, se_c, _, _ = classes[c]
                        if se_c > 0:
                            chi2 += ((e_c - peff) / se_c) ** 2
                    pval = chi2_sf(chi2, len(used) - 1)
                    row["class_effects"] = {
                        c: {
                            "effect": classes[c][0],
                            "se": classes[c][1],
                            "n_k": classes[c][2],
                            "n_s": classes[c][3],
                        }
                        for c in sorted(classes)
                    }
                    row["class_chi2_p"] = pval
                    if pval < 0.01:
                        class_candidates.append(f"{deck_name} {cid} (p={pval:.4f})")

        deck_rows.sort(key=lambda r: abs(r["pooled"]["effect"]), reverse=True)
        table_decks[fp] = {
            "name": deck_name,
            "first": first_map,
            "second": second_map,
            "fit_rows": deck_rows,
        }

    policy = chunks[0].get("policy_a", "h0:mull=random")
    table = {
        "version": 1,
        "decks": {
            fp: {
                "name": entry["name"],
                "first": entry["first"],
                "second": entry["second"],
            }
            for fp, entry in table_decks.items()
        },
        "meta": {
            "pool": pool_meta,
            "policy": policy,
            "games": len(observations) // 2,
            "fit": {"z": args.z, "min_n": args.min_n},
            "created": dt.datetime.now(dt.timezone.utc).isoformat(),
        },
    }
    (td / "table.json").write_text(json.dumps(table, indent=2) + "\n", encoding="utf-8")
    fit_doc = {
        "observations": len(observations),
        "excluded_split": excluded_split,
        "z": args.z,
        "min_n": args.min_n,
        "cards": fit_cards,
        "rule_diffs": rule_diffs,
        "class_candidates": class_candidates,
    }
    (td / "fit.json").write_text(json.dumps(fit_doc, indent=2) + "\n", encoding="utf-8")
    write_fit_md(td / "FIT.md", len(observations), excluded_split, rule_diffs, class_candidates, table_decks)
    if args.publish:
        publish_mulligan(repo, args, td)


def write_fit_md(
    path: Path,
    n_obs: int,
    excluded: int,
    rule_diffs: list[str],
    class_candidates: list[str],
    table_decks: dict[str, Any],
) -> None:
    lines = [
        "# Mulligan fit",
        "",
        f"observations: {n_obs}",
        f"excluded (split copies): {excluded}",
        "",
        "## Decisions differing from rule",
        "",
    ]
    if rule_diffs:
        lines.extend(f"- {d}" for d in rule_diffs)
    else:
        lines.append("none")
    lines.extend(["", "## class-dependent candidates", ""])
    if class_candidates:
        lines.extend(f"- {c}" for c in class_candidates)
    else:
        lines.append("none")
    for fp, entry in sorted(table_decks.items(), key=lambda x: x[1]["name"]):
        lines.extend(["", f"## {entry['name']}", f"fingerprint: `{fp}`", ""])
        for row in entry["fit_rows"]:
            p = row["pooled"]
            lines.append(
                f"- `{row['card']}` cost={row['cost']} copies={row['copies']} "
                f"pooled effect={p['effect']:.3f} z={p['z']:.2f} "
                f"first={row['first']['keep']} ({row['first']['how']}) "
                f"second={row['second']['keep']} ({row['second']['how']})"
                + (" *rule*" if row["differs_from_rule"] else "")
            )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def publish_mulligan(repo: Path, args: argparse.Namespace, td: Path) -> None:
    publish_dir = (
        Path(args.publish_dir)
        if args.publish_dir
        else repo.parent / "arena-results-wt"
    )
    import shutil
    import tempfile

    staging = Path(tempfile.mkdtemp(prefix="mulligan-publish-"))
    dest_tag = staging / args.tag
    dest_tag.mkdir(parents=True)
    for name in PUBLISH_FILES:
        src = td / name
        if src.is_file():
            shutil.copy2(src, dest_tag / name)
    log = td / "publish.txt"
    publish_tag(
        repo,
        publish_dir,
        args.publish_remote,
        args.publish_branch,
        staging,
        args.tag,
        log,
    )


def main() -> None:
    args = parse_args()
    if args.command == "data":
        cmd_data(args)
    else:
        cmd_fit(args)


if __name__ == "__main__":
    main()
