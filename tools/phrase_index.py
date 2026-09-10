#!/usr/bin/env python3
"""Pool-wide phrase index: one construction per printed phrase.

Walk every authored file under cards/** (skip cards/official/). For every
node that carries `printed` — abilities, modes, and clause roots — emit
(normalized phrase, construction shape, file, JSON path).

  python3 tools/phrase_index.py --write   regenerate docs/phrase-index.md
  python3 tools/phrase_index.py --check   exit 1 on unlisted divergence or stale doc
"""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CARDS = ROOT / "cards"
CATALOG = CARDS / "official" / "catalog.json"
DOC = ROOT / "docs" / "phrase-index.md"
EXCEPTIONS = ROOT / "tools" / "phrase-index-exceptions.json"

# Leading trigger prefixes already encoded by ability `on` / mode `kind`.
# `At the start/end of your turn, ` stays — it is the phrase.
_TRIGGER_PREFIX = re.compile(
    r"^(?:"
    r"Fanfare:\s*"
    r"|Last Words:\s*"
    r"|Evolve:\s*"
    r"|Super-Evolve:\s*"
    r"|Strike:\s*"
    r"|Follower Strike:\s*"
    r"|Clash:\s*"
    r"|Enhance \(\d+\):\s*"
    r"|Combo \(\d+\):\s*"
    r"|Rally \(\d+\):\s*"
    r"|Engage:\s*"
    r"|Spellboost:\s*"
    r"|Countdown \(\d+\)\s*"
    r")"
)

_ID_RE = re.compile(r"^(?:(?:crest|faith):)?\d{8}$")
_PLUS_STAT = re.compile(r"\+\d+/\+\d+")
_DIGIT = re.compile(r"\d+")

# Same markup strip as fetch-official.mjs / docs/schema.md (coverage walk).
_HR = re.compile(r"<hr\s*/?>", re.I)
_BI = re.compile(r"</?(?:b|i)>", re.I)
_COLOR = re.compile(r"</?color(?:=[^>]*)?>", re.I)
_RIDX = re.compile(r"</?ridx(?:=[^>]*)?>", re.I)
_TAG = re.compile(r"<[^>]+>")


def strip_markup(s: str) -> str:
    t = _HR.sub("\n", s)
    t = _BI.sub("", t)
    t = _COLOR.sub("", t)
    t = _RIDX.sub("", t)
    t = _TAG.sub("", t)
    return t


def load_catalog_names() -> list[str]:
    raw = json.loads(CATALOG.read_text())
    names: set[str] = set()
    for key, rec in raw.items():
        if key == "_meta" or not isinstance(rec, dict):
            continue
        name = rec.get("name")
        if isinstance(name, str) and name.strip():
            names.add(name.strip())
        for effect in rec.get("specific_effects") or []:
            # Crest / Faith share the granting card's name in official text
            # (`Crest: <card name>`). Tokens are already in `names`.
            if effect.get("type") in ("crest", "faith") and isinstance(name, str):
                names.add(name.strip())
    return sorted(names, key=lambda n: (-len(n), n))


def _compile_name_replacer(names: list[str]):
    crest_pats: list[tuple[re.Pattern[str], str]] = []
    name_pats: list[tuple[re.Pattern[str], str]] = []
    for name in names:
        esc = re.escape(name)
        # Longest-first; don't eat a letter/digit on either side.
        crest_pats.append((re.compile(rf"(?<!\w)Crest:\s*{esc}(?!\w)"), "CREST"))
        name_pats.append((re.compile(rf"(?<!\w){esc}(?!\w)"), "NAME"))
    return crest_pats, name_pats


def normalize_phrase(printed: str, crest_pats, name_pats) -> str:
    t = strip_markup(printed)
    t = _TRIGGER_PREFIX.sub("", t, count=1)
    for rx, repl in crest_pats:
        t = rx.sub(repl, t)
    for rx, repl in name_pats:
        t = rx.sub(repl, t)
    t = _PLUS_STAT.sub("+N/+N", t)
    t = _DIGIT.sub("N", t)
    t = t.rstrip(".")
    t = re.sub(r"\s+", " ", t).strip()
    return t


# Trigger fields already encoded by the stripped prefix / remaining phrase.
# `event` stays — "whenever X" constructions differ by event.
_ABILITY_TRIGGER_KEYS = frozenset({"on", "whose"})
_MODE_TRIGGER_KEYS = frozenset({"kind", "cost"})


def _transform_node(node: object):
    if isinstance(node, dict):
        out = {}
        for k, v in node.items():
            if k == "printed":
                continue
            if k in ("as", "ref") and isinstance(v, str):
                out[k] = "REF"
                continue
            out[k] = _transform_node(v)
        return out
    if isinstance(node, list):
        return [_transform_node(i) for i in node]
    if isinstance(node, bool):
        return node
    if isinstance(node, (int, float)):
        return "N"
    if isinstance(node, str) and _ID_RE.match(node):
        return "ID"
    return node


def construction_shape(node: object, kind: str) -> str:
    data = _transform_node(node)
    if isinstance(data, dict):
        drop = set()
        if kind == "ability":
            drop = _ABILITY_TRIGGER_KEYS
        elif kind == "mode":
            drop = _MODE_TRIGGER_KEYS
        if drop:
            data = {k: v for k, v in data.items() if k not in drop}
    return json.dumps(data, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def walk_json_files(root: Path) -> list[Path]:
    out: list[Path] = []
    for p in sorted(root.rglob("*.json")):
        if "official" in p.parts:
            continue
        out.append(p)
    return out


def json_path_join(prefix: str, key: str | int) -> str:
    if isinstance(key, int):
        return f"{prefix}[{key}]"
    if prefix == "$":
        return key
    return f"{prefix}.{key}"


def node_kind(node: dict, path: str) -> str:
    if "on" in node:
        return "ability"
    kind = node.get("kind")
    if kind in ("enhance", "accelerate", "crystallize"):
        return "mode"
    if path == "fuse" or path.startswith("fuse."):
        return "fuse"
    return "clause"


def iter_printed(obj, path: str = "$"):
    if isinstance(obj, dict):
        if "printed" in obj and isinstance(obj["printed"], str):
            yield path, obj
        for k, v in obj.items():
            yield from iter_printed(v, json_path_join(path, k))
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            yield from iter_printed(v, json_path_join(path, i))


def file_id(data: dict, path: Path) -> str:
    cid = data.get("id")
    if isinstance(cid, str) and cid:
        return cid
    return path.stem


def collect_rows(crest_pats, name_pats) -> list[dict]:
    rows: list[dict] = []
    for path in walk_json_files(CARDS):
        data = json.loads(path.read_text())
        cid = file_id(data, path)
        rel = path.relative_to(ROOT).as_posix()
        for jpath, node in iter_printed(data):
            kind = node_kind(node, jpath)
            phrase = normalize_phrase(node["printed"], crest_pats, name_pats)
            rows.append(
                {
                    "phrase": phrase,
                    "shape": construction_shape(node, kind),
                    "file": rel,
                    "path": jpath,
                    "id": cid,
                    "kind": kind,
                }
            )
    return rows


def group_key(row: dict) -> str:
    """Same-kind phrases only — an ability is not compared to its inner clause."""
    return f"{row['kind']}|{row['phrase']}"


def group_by_phrase(rows: list[dict]) -> dict[str, dict[str, list[dict]]]:
    grouped: dict[str, dict[str, list[dict]]] = defaultdict(lambda: defaultdict(list))
    for row in rows:
        grouped[group_key(row)][row["shape"]].append(row)
    return grouped


def display_phrase(key: str) -> str:
    kind, _, phrase = key.partition("|")
    return f"({kind}) {phrase}"


def load_exceptions() -> list[dict]:
    if not EXCEPTIONS.exists():
        return []
    data = json.loads(EXCEPTIONS.read_text())
    if not isinstance(data, list):
        raise SystemExit(f"{EXCEPTIONS}: expected a JSON array")
    return data


def exception_map(entries: list[dict]) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for e in entries:
        phrase = e.get("phrase")
        if not isinstance(phrase, str) or not phrase:
            raise SystemExit("exception entry missing phrase")
        if phrase in out:
            raise SystemExit(f"duplicate exception phrase: {phrase}")
        ids = e.get("ids")
        if not isinstance(ids, list) or not ids or not all(isinstance(i, str) for i in ids):
            raise SystemExit(f"exception {phrase!r}: ids must be a non-empty string list")
        if not isinstance(e.get("reason"), str) or not e["reason"].strip():
            raise SystemExit(f"exception {phrase!r}: reason required")
        out[phrase] = e
    return out


def divergent_phrases(grouped) -> dict[str, dict[str, list[dict]]]:
    return {p: shapes for p, shapes in grouped.items() if len(shapes) > 1}


def md_cell(s: str) -> str:
    return s.replace("|", "\\|").replace("\n", " ")


def render_doc(rows: list[dict], grouped, exceptions: list[dict]) -> str:
    lines: list[str] = []
    n_rows = len(rows)
    n_div = sum(1 for shapes in grouped.values() if len(shapes) > 1)
    lines.append("# Phrase index")
    lines.append("")
    lines.append(
        "Standing rule: **one construction per printed phrase**, pool-wide. "
        "Two constructions for one phrase are two behaviours for one card text."
    )
    lines.append("")
    lines.append("Generated by `python3 tools/phrase_index.py --write`. Do not edit by hand.")
    lines.append("")
    lines.append("## Normalization")
    lines.append("")
    lines.append("Each authored node that carries `printed` (ability, mode, clause root) is one row.")
    lines.append("The phrase is normalized so rows compare across cards, in this order:")
    lines.append("")
    lines.append("1. Markup stripping — same walk as coverage (`<hr>` → newline; drop `<b>`/`<i>`/`<color>`/`<ridx>` and leftover tags).")
    lines.append(
        "2. Strip the leading trigger prefix already encoded by `on` / mode `kind`: "
        "`Fanfare:`, `Last Words:`, `Evolve:`, `Super-Evolve:`, `Strike:`, `Follower Strike:`, "
        "`Clash:`, `Enhance (N):`, `Combo (N):`, `Rally (N):`, "
        "`Engage:`, `Spellboost:`, `Countdown (N)`. "
        "`At the start/end of your turn,` and `Skybound Art` / `Super Skybound Art` stay — "
        "they are the phrase (a condition, not `on`)."
    )
    lines.append("3. `Crest: <catalog name>` → `CREST`; remaining catalog names (cards, tokens, crests; longest first) → `NAME`.")
    lines.append("4. `+N/+N` stat pairs stay `+N/+N`; other digits → `N`. `X is …` keeps `X`.")
    lines.append("5. Trailing period stripped; whitespace collapsed.")
    lines.append("")
    lines.append(
        "The construction shape is the node's JSON with every `printed` removed, "
        "numeric literals → `N`, card/crest ids → `ID`, binding names (`as`, `ref`) → `REF`, "
        "then canonical JSON (sorted keys). Ability `on`/`whose` and mode `kind`/`cost` are "
        "omitted from the shape — they are the stripped prefix. Two nodes share a construction "
        "when the shapes are byte-equal. Divergence is judged per node kind (ability / mode / clause) "
        "so a Fanfare wrapper is not compared to its inner clause."
    )
    lines.append("")
    n_raw_phrases = len({r["phrase"] for r in rows})
    n_raw_div_phrases = len({k.partition("|")[2] for k, s in grouped.items() if len(s) > 1})
    n_raw_one = n_raw_phrases - n_raw_div_phrases
    lines.append(
        f"Index: **{n_rows}** rows, **{n_raw_phrases}** distinct phrases, "
        f"**{n_raw_one}** with one shape (per kind), **{n_raw_div_phrases}** divergent "
        f"({n_div} kind×phrase groups)."
    )
    lines.append("")
    lines.append("## Phrases")
    lines.append("")
    lines.append("| phrase | shape | rows | examples |")
    lines.append("|---|---|---|---|")
    for key in sorted(grouped, key=lambda k: (k.partition("|")[2], k)):
        label = display_phrase(key)
        for shape in sorted(grouped[key]):
            occ = grouped[key][shape]
            ids = sorted({r["id"] for r in occ})
            examples = ", ".join(f"`{i}`" for i in ids[:3])
            if len(ids) > 3:
                examples += ", …"
            lines.append(
                f"| {md_cell(label)} | `{md_cell(shape)}` | {len(occ)} | {examples} |"
            )
    lines.append("")
    lines.append("## Divergent phrases")
    lines.append("")
    div = divergent_phrases(grouped)
    if not div:
        lines.append("None.")
        lines.append("")
    else:
        lines.append(
            "Every same-kind phrase with more than one construction shape. "
            "Unlisted divergences fail `python3 tools/phrase_index.py --check`."
        )
        lines.append("")
        for key in sorted(div, key=lambda k: (k.partition("|")[2], k)):
            lines.append(f"### `{md_cell(display_phrase(key))}`")
            lines.append("")
            for shape in sorted(div[key]):
                ids = sorted({r["id"] for r in div[key][shape]})
                id_cell = ", ".join(f"`{i}`" for i in ids)
                lines.append(f"- `{md_cell(shape)}` — {id_cell}")
            lines.append("")
    lines.append("## Exceptions")
    lines.append("")
    if not exceptions:
        lines.append("None. `tools/phrase-index-exceptions.json` is `[]`.")
        lines.append("")
    else:
        lines.append("From `tools/phrase-index-exceptions.json`. Each entry names ids and the sentence that forces the difference.")
        lines.append("")
        lines.append("| phrase | reason | ids |")
        lines.append("|---|---|---|")
        for e in exceptions:
            ids = ", ".join(f"`{i}`" for i in e["ids"])
            lines.append(f"| {md_cell(e['phrase'])} | {md_cell(e['reason'])} | {ids} |")
        lines.append("")
    return "\n".join(lines)


def _phrase_of_key(key: str) -> str:
    return key.partition("|")[2]


def check_exceptions(div: dict, exceptions: dict[str, dict]) -> list[str]:
    errors: list[str] = []
    div_by_phrase: dict[str, list[tuple[str, dict]]] = defaultdict(list)
    for key, shapes in div.items():
        div_by_phrase[_phrase_of_key(key)].append((key, shapes))
    for key, shapes in sorted(div.items()):
        phrase = _phrase_of_key(key)
        if phrase in exceptions:
            continue
        errors.append(f"unlisted divergence: {display_phrase(key)}")
        for shape, occ in sorted(shapes.items()):
            ids = sorted({r["id"] for r in occ})
            errors.append(f"  {shape}")
            errors.append(f"    ids: {', '.join(ids)}")
    for phrase, entry in sorted(exceptions.items()):
        groups = div_by_phrase.get(phrase) or []
        listed = set(entry["ids"])
        used_shapes: set[str] = set()
        found: set[str] = set()
        for _key, shapes in groups:
            for shape, occ in shapes.items():
                for r in occ:
                    if r["id"] in listed:
                        found.add(r["id"])
                        used_shapes.add(f"{_key}::{shape}")
        if len(used_shapes) < 2:
            errors.append(f"stale exception: {phrase} (ids no longer diverge)")
            continue
        missing = listed - found
        if missing:
            errors.append(
                f"stale exception: {phrase} (ids not on this phrase: {', '.join(sorted(missing))})"
            )
    return errors


def build() -> tuple[list[dict], dict, list[dict]]:
    names = load_catalog_names()
    crest_pats, name_pats = _compile_name_replacer(names)
    rows = collect_rows(crest_pats, name_pats)
    grouped = group_by_phrase(rows)
    exceptions = load_exceptions()
    return rows, grouped, exceptions


def cmd_write() -> int:
    rows, grouped, exceptions = build()
    DOC.write_text(render_doc(rows, grouped, exceptions))
    n_phrases = len({r["phrase"] for r in rows})
    n_div = len({k.partition("|")[2] for k, s in grouped.items() if len(s) > 1})
    print(
        f"wrote {DOC.relative_to(ROOT)}: {len(rows)} rows, "
        f"{n_phrases} phrases, {n_div} divergent"
    )
    return 0


def cmd_check(*, divergences: bool, doc: bool) -> int:
    rows, grouped, exceptions = build()
    errors: list[str] = []
    div = divergent_phrases(grouped)
    exmap = exception_map(exceptions)
    if divergences:
        errors.extend(check_exceptions(div, exmap))
    if doc:
        expected = render_doc(rows, grouped, exceptions)
        committed = DOC.read_text() if DOC.exists() else ""
        if committed != expected:
            errors.append(
                f"{DOC.relative_to(ROOT)} differs from --write; run python3 tools/phrase_index.py --write"
            )
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        print(f"\n{len(errors)} phrase-index error(s)", file=sys.stderr)
        return 1
    n_phrases = len({r["phrase"] for r in rows})
    n_div_phrases = len({k.partition("|")[2] for k, s in grouped.items() if len(s) > 1})
    print(
        f"ok: {len(rows)} rows, {n_phrases} phrases, "
        f"{n_phrases - n_div_phrases} one-shape, {n_div_phrases} divergent (excepted), doc current"
    )
    return 0


def main(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 0
    cmd = argv[0]
    if cmd == "--write":
        return cmd_write()
    if cmd == "--check":
        return cmd_check(divergences=True, doc=True)
    if cmd == "--check-divergences":
        return cmd_check(divergences=True, doc=False)
    if cmd == "--check-doc":
        return cmd_check(divergences=False, doc=True)
    print(f"unknown command: {cmd}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
