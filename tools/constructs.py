#!/usr/bin/env python3
"""Enumerate schema constructs as stable slugs; check / write fixture coverage.

Coverage is by test-name convention: construct ``X`` is covered when a test
function named ``construct_X`` exists under ``engine/tests/**``.

    python3 tools/constructs.py              # print slug list
    python3 tools/constructs.py --write      # regenerate docs/construct-fixtures.md
    python3 tools/constructs.py --check --families effect,source,selector
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "schema" / "cards.schema.json"
DOC = ROOT / "docs" / "construct-fixtures.md"
TESTS = ROOT / "engine" / "tests"

# Common Effect fields that do not get a per-op key slug. Generic ``as`` /
# ``when`` are emitted once (op_as, op_when). ``select_as`` is the select op's
# bind, listed separately in the brief.
_EFFECT_COMMON = frozenset({"op", "printed", "as", "when"})

# Keys that change behaviour; value-level slugs are hand-named for enums.
# Derived from the schema, then expanded here so the list stays stable.
_ENUM_KEYS = {
    "until": None,  # filled from schema enum
    "action": None,
    "how": None,
    "resource": None,
    "position": None,
    "controller": None,
    "player": None,
    "by": None,
}

# Trigger tags on removeAbilities.on (schema enum).
_REMOVE_ON: tuple[str, ...] = ()
# replicate.ability enum
_REPLICATE_ABILITY: tuple[str, ...] = ()


def load_schema() -> dict:
    return json.loads(SCHEMA.read_text())


def _op_const(variant: dict) -> str | None:
    op = variant.get("properties", {}).get("op")
    if not isinstance(op, dict):
        return None
    return op.get("const")


def _enum_values(node: dict) -> list[str]:
    if "enum" in node:
        return [str(x) for x in node["enum"]]
    if "const" in node:
        return [str(node["const"])]
    return []


def _items_enum(node: dict) -> list[str]:
    items = node.get("items")
    if isinstance(items, dict):
        return _enum_values(items)
    return []


def effect_ops(schema: dict) -> list[str]:
    seen: list[str] = []
    for variant in schema["$defs"]["Effect"]["oneOf"]:
        op = _op_const(variant)
        if op and op not in seen:
            seen.append(op)
    return seen


def _effect_variant(schema: dict, op: str) -> dict | None:
    for variant in schema["$defs"]["Effect"]["oneOf"]:
        if _op_const(variant) == op:
            return variant
    return None


def _prop(schema: dict, op: str, key: str) -> dict | None:
    variant = _effect_variant(schema, op)
    if not variant:
        return None
    return variant.get("properties", {}).get(key)


def enumerate_effect_slugs(schema: dict) -> list[str]:
    """Every Effect op plus behaviour-changing keys (brief Part A)."""
    slugs: list[str] = []
    ops = effect_ops(schema)
    for op in ops:
        slugs.append(f"op_{op}")

    # Op-specific keys that change behaviour. Names match the brief.
    slugs.append("op_buff_untilEndOfTurn")
    slugs.append("op_cost_set")
    slugs.append("op_cost_delta")
    for how in _enum_values(_prop(schema, "counter", "how") or {}):
        slugs.append(f"op_counter_how_{how}")
    slugs.append("op_damage_split")
    slugs.append("op_draw_filter")
    slugs.append("op_draw_distinctNames")
    slugs.append("op_draw_player_opponent")

    # Schema: ep.action is gain|spend; pp.action is gainMax|recover|spend.
    # The brief swapped the two names; we follow the schema.
    for action in _enum_values(_prop(schema, "ep", "action") or {}):
        slugs.append(f"op_ep_action_{action}")
    for action in _enum_values(_prop(schema, "pp", "action") or {}):
        slugs.append(f"op_pp_action_{action}")

    for res in _enum_values(_prop(schema, "pay", "resource") or {}):
        slugs.append(f"op_pay_resource_{res}")

    slugs.append("op_leaderModifier_maxDefense_set")
    slugs.append("op_leaderModifier_maxDefense_delta")
    slugs.append("op_leaderModifier_damageCap")
    slugs.append("op_leaderModifier_damageTakenBonus")
    slugs.append("op_leaderModifier_until")

    until = _enum_values(_prop(schema, "grantTraits", "until") or {})
    for u in until:
        slugs.append(f"op_grantTraits_until_{u}")

    slugs.append("op_summon_controller_opponent")
    slugs.append("op_addToDeck_position_random")
    slugs.append("op_returnToDeck_position_random")

    for by in _enum_values(_prop(schema, "choose", "by") or {}):
        slugs.append(f"op_choose_by_{by}")
    slugs.append("op_choose_optionsFrom")
    slugs.append("op_choose_pick")
    slugs.append("op_randomSplit_keys")

    on_items = _items_enum(_prop(schema, "removeAbilities", "on") or {})
    for tag in on_items:
        slugs.append(f"op_removeAbilities_on_{tag}")

    for ab in _enum_values(_prop(schema, "replicate", "ability") or {}):
        slugs.append(f"op_replicate_ability_{ab}")

    slugs.append("op_grantAbility_ability")
    slugs.append("op_evolve_super")
    slugs.append("op_reanimate_maxCost")
    slugs.append("op_repeat_times_amount")
    slugs.append("op_if_else")
    slugs.append("op_sequence_steps")
    slugs.append("op_select_as")
    slugs.append("op_as")
    slugs.append("op_when")
    return slugs


def enumerate_source_slugs(schema: dict) -> list[str]:
    _ = schema
    return [
        "source_named",
        "source_copyOf",
        "source_copyOf_exact",
        "source_from",
        "source_randomFrom",
        "source_randomFrom_countGt1",
    ]


def enumerate_selector_slugs(schema: dict) -> list[str]:
    pool = None
    for variant in schema["$defs"]["Selector"]["oneOf"]:
        props = variant.get("properties", {})
        pick = props.get("pick", {})
        if "enum" in pick:
            pool = props
            break
    assert pool is not None
    slugs: list[str] = []
    for p in pool["pick"]["enum"]:
        slugs.append(f"selector_pick_{p}")
    for z in pool["zone"]["enum"]:
        slugs.append(f"selector_zone_{z}")
    for k in pool["kind"]["enum"]:
        slugs.append(f"selector_kind_{k}")
    for s in pool["side"]["enum"]:
        slugs.append(f"selector_side_{s}")
    for o in pool["orderBy"]["enum"]:
        slugs.append(f"selector_orderBy_{o}")
    slugs.extend(
        [
            "selector_other",
            "selector_includeLeader",
            "selector_count",
            "selector_filter",
            "selector_ref",
        ]
    )
    return slugs


def enumerate_condition_slugs(schema: dict) -> list[str]:
    slugs: list[str] = []
    for variant in schema["$defs"]["Condition"]["oneOf"]:
        required = variant.get("required") or []
        if required:
            slugs.append(f"condition_{required[0]}")
    return slugs


def enumerate_filter_slugs(schema: dict) -> list[str]:
    props = schema["$defs"]["Filter"]["properties"]
    return [f"filter_{k}" for k in props]


def enumerate_amount_slugs(schema: dict) -> list[str]:
    slugs: list[str] = []
    for variant in schema["$defs"]["Amount"]["oneOf"]:
        if variant.get("type") == "integer":
            slugs.append("amount_int")
            continue
        required = variant.get("required") or []
        if required:
            slugs.append(f"amount_{required[0]}")
    return slugs


def enumerate_ability_slugs(schema: dict) -> list[str]:
    slugs: list[str] = []
    ons: list[str] = []
    events: list[str] = []
    suppress: list[str] = []
    for variant in schema["$defs"]["Ability"]["oneOf"]:
        props = variant.get("properties", {})
        on = props.get("on", {})
        if "const" in on and on["const"] not in ons:
            ons.append(on["const"])
        if "event" in props:
            events = _enum_values(props["event"])
        if on.get("const") == "static":
            mod = props.get("modifier", {}).get("properties", {})
            suppress = _items_enum(mod.get("suppress") or {})
    for on in ons:
        slugs.append(f"ability_on_{on}")
    for ev in events:
        slugs.append(f"ability_event_{ev}")
    slugs.append("ability_oncePerTurn")
    slugs.append("ability_when_turnOwner")
    slugs.append("ability_zone_hand")
    slugs.append("ability_zone_deck")
    slugs.append("ability_whose_own")
    slugs.append("ability_whose_opponent")
    for s in suppress:
        slugs.append(f"ability_modifier_suppress_{s}")
    slugs.append("ability_replaces")
    slugs.append("ability_sacrifice")
    slugs.append("ability_cost")
    return slugs


FAMILIES = (
    "effect",
    "source",
    "selector",
    "condition",
    "filter",
    "amount",
    "ability",
)


def all_constructs(schema: dict) -> list[tuple[str, str]]:
    """Return (slug, family) in stable family-then-slug order."""
    out: list[tuple[str, str]] = []
    out.extend((s, "effect") for s in enumerate_effect_slugs(schema))
    out.extend((s, "source") for s in enumerate_source_slugs(schema))
    out.extend((s, "selector") for s in enumerate_selector_slugs(schema))
    out.extend((s, "condition") for s in enumerate_condition_slugs(schema))
    out.extend((s, "filter") for s in enumerate_filter_slugs(schema))
    out.extend((s, "amount") for s in enumerate_amount_slugs(schema))
    out.extend((s, "ability") for s in enumerate_ability_slugs(schema))
    return out


_FN_RE = re.compile(r"(?m)^(?:pub\s+)?(?:async\s+)?fn\s+(construct_[A-Za-z0-9_]+)\s*\(")


def scan_tests() -> dict[str, str]:
    """Map construct slug -> relative path of the covering test file."""
    covered: dict[str, str] = {}
    for path in sorted(TESTS.rglob("*.rs")):
        text = path.read_text()
        rel = path.relative_to(ROOT).as_posix()
        for m in _FN_RE.finditer(text):
            name = m.group(1)
            slug = name[len("construct_") :]
            covered.setdefault(slug, rel)
    return covered


def render_doc(rows: list[tuple[str, str, str, str]]) -> str:
    lines = [
        "# Construct fixtures",
        "",
        "Generated by `python3 tools/constructs.py --write`. Do not edit by hand.",
        "",
        "A construct is covered when `engine/tests/**` contains `fn construct_<slug>`.",
        "Families `condition`, `filter`, `amount`, and `ability` are enforced by a",
        "sibling PR; this table still lists them.",
        "",
        "| slug | family | covering test | source |",
        "| --- | --- | --- | --- |",
    ]
    for slug, family, test, src in rows:
        lines.append(f"| `{slug}` | {family} | {test} | {src} |")
    lines.append("")
    return "\n".join(lines)


def coverage_rows(schema: dict) -> list[tuple[str, str, str, str]]:
    covered = scan_tests()
    rows = []
    for slug, family in all_constructs(schema):
        src = covered.get(slug)
        if src:
            rows.append((slug, family, f"`construct_{slug}`", f"`{src}`"))
        else:
            rows.append((slug, family, "**MISSING**", "—"))
    return rows


def print_slug_list(schema: dict) -> None:
    by: dict[str, list[str]] = {f: [] for f in FAMILIES}
    for slug, family in all_constructs(schema):
        by[family].append(slug)
    for family in FAMILIES:
        slugs = by[family]
        print(f"## {family} ({len(slugs)})")
        for s in slugs:
            print(s)
        print()
    print("counts:", {f: len(by[f]) for f in FAMILIES}, "total", sum(len(by[f]) for f in FAMILIES))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="regenerate docs/construct-fixtures.md")
    ap.add_argument("--check", action="store_true", help="exit 1 on MISSING (selected families) or stale doc")
    ap.add_argument(
        "--families",
        default="",
        help="comma-separated families to enforce under --check (default: none)",
    )
    args = ap.parse_args()
    schema = load_schema()
    rows = coverage_rows(schema)

    if not args.write and not args.check:
        print_slug_list(schema)
        return 0

    doc = render_doc(rows)
    if args.write:
        DOC.write_text(doc)
        print(f"wrote {DOC.relative_to(ROOT)} ({len(rows)} slugs)", file=sys.stderr)

    if args.check:
        errors: list[str] = []
        families = {f.strip() for f in args.families.split(",") if f.strip()}
        unknown = families - set(FAMILIES)
        if unknown:
            errors.append(f"unknown families: {sorted(unknown)}")
        for slug, family, test, _src in rows:
            if family in families and test == "**MISSING**":
                errors.append(f"MISSING {family} {slug}")
        if DOC.exists():
            if DOC.read_text() != doc:
                errors.append(f"{DOC.relative_to(ROOT)} is stale; run python3 tools/constructs.py --write")
        else:
            errors.append(f"{DOC.relative_to(ROOT)} is missing; run python3 tools/constructs.py --write")
        if errors:
            for e in errors:
                print(e, file=sys.stderr)
            print(f"\n{len(errors)} construct-check error(s)", file=sys.stderr)
            return 1
        enforced = ",".join(sorted(families)) if families else "(none)"
        print(f"ok: {len(rows)} slugs, families {enforced} complete, doc current")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
