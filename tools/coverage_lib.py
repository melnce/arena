"""Shared verb → op / condition mapping for coverage.md and coverage-check.py."""

from __future__ import annotations

import re

TRAIT_WORDS = (
    "Ward|Storm|Rush|Bane|Drain|Barrier|Ambush|Aura|Intimidate"
)

# (regex, op) — applied to printed text. Order matters for overlapping phrases.
OP_RULES: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"add (?:a |an |two |three |\d+ )?.{0,80}?to your hand", re.I), "addToHand"),
    (re.compile(r"add (?:a |an |two |three |\d+ )?.{0,80}?to (?:your |the )?deck", re.I), "addToDeck"),
    (re.compile(r"summon ", re.I), "summon"),
    (re.compile(r"\bdraw\b", re.I), "draw"),
    (re.compile(r"deal .{0,40}?damage", re.I), "damage"),
    (re.compile(r"restore .{0,20}?defense", re.I), "restore"),
    (re.compile(r"give .{0,80}?[+-]\d+/[+-]?\d+", re.I), "buff"),
    (re.compile(r"give this (?:follower|card) [+-]\d+", re.I), "buff"),
    (re.compile(rf"give .{{0,80}}?\b(?:{TRAIT_WORDS})\b", re.I), "grantTraits"),
    (re.compile(r'give .{0,80}?"(?:Can\'t attack|Can attack)', re.I), "grantTraits"),
    (re.compile(r"remove (?:all )?abilit", re.I), "removeAbilities"),
    (re.compile(r"\bInvoke\b", re.I), "invoke"),
    (re.compile(r"max defense|Takes \d+ more damage|damage cap", re.I), "leaderModifier"),
    (re.compile(r"\bdestroy\b", re.I), "destroy"),
    (re.compile(r"\bbanish\b", re.I), "banish"),
    (re.compile(r"return .{0,80}?to (?:your |the )?(?:hand)", re.I), "returnToHand"),
    (re.compile(r"return .{0,80}?to (?:your |the )?deck", re.I), "returnToDeck"),
    (re.compile(r"\bdiscard\b", re.I), "discard"),
    (re.compile(r"(?<!Super-)(?<!when this follower )evolve (?:it|this|an |a |them|all )", re.I), "evolve"),
    (re.compile(r"and evolve it", re.I), "evolve"),
    (re.compile(r"\btransform\b", re.I), "transform"),
    (re.compile(r"(?:reduce|increase|change) the cost|costs? \d+ less|costs? \d+ more|set (?:its |the |this card's )?cost", re.I), "cost"),
    (re.compile(r"(?:recover|gain|spend) \d+ play points?|recover \d+ PP", re.I), "pp"),
    (re.compile(r"(?:recover|gain|spend) \d+ evolution points?", re.I), "ep"),
    (re.compile(r"gain Crest|give (?:your opponent )?Crest", re.I), "crest"),
    (re.compile(r"increase your Combo|your Combo by|Combo by \d+", re.I), "counter"),
    (re.compile(r"earth sigil|faith(?:'s)? value|shadows", re.I), "counter"),
    (re.compile(r"\bNecromancy\b|\bEarth Rite\b", re.I), "pay"),
    (re.compile(r"Reduce your faith's value|spend \d+ play points", re.I), "pay"),
    (re.compile(r"Spellboost your hand", re.I), "spellboostHand"),
    (re.compile(r"\bReanimate\b", re.I), "reanimate"),
    (re.compile(r"\bReplicate\b", re.I), "replicate"),
    (re.compile(r"Select a Mode|activate (?:all of them|a random ability)|random abilities", re.I), "choose"),
    (re.compile(r"Do this \d+ times|this effect \d+ times|\d+ times\.", re.I), "repeat"),
    (re.compile(r"in sequence", re.I), "sequence"),
    (re.compile(r"Enhance \(\d+\)", re.I), "mode:enhance"),
    (re.compile(r"Accelerate \(\d+\)", re.I), "mode:accelerate"),
    (re.compile(r"Crystallize \(\d+\)", re.I), "mode:crystallize"),
]

COND_RULES: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"Combo \(\d+\)"), "combo"),
    (re.compile(r"Rally \(\d+\)"), "rally"),
    (re.compile(r"\bOverflow\b"), "overflow"),
    (re.compile(r"Skybound Art|Super Skybound"), "skyboundArt"),
    (re.compile(r"if this follower is evolved|if .{0,40}evolved", re.I), "evolved"),
    (re.compile(r"If you've Fused|if you have Fused|Fused both|you've Fused", re.I), "wasFused"),
    (re.compile(r"during your turn|during the opponent", re.I), "turnOwner"),
    (re.compile(r"super-evolved this match|evolved .{0,20}this match", re.I), "evolvedCountAtLeast"),
]


def required_ops(text: str) -> list[str]:
    seen: list[str] = []
    for rx, op in OP_RULES:
        if rx.search(text) and op not in seen:
            seen.append(op)
    # Combo/earth/faith increment without "pay" wording is counter, not both
    if "pay" in seen and re.search(r"\bNecromancy\b|\bEarth Rite\b|Reduce your faith", text, re.I):
        pass
    return seen


def required_conditions(text: str) -> list[str]:
    seen: list[str] = []
    for rx, cond in COND_RULES:
        if rx.search(text) and cond not in seen:
            seen.append(cond)
    return seen


def parse_coverage_table(md: str) -> list[dict]:
    rows = []
    for line in md.splitlines():
        if not line.startswith("| `"):
            continue
        parts = [p.strip() for p in line.strip("|").split("|")]
        if len(parts) < 6:
            continue
        cid = parts[0].strip("`")
        if cid in ("id", "---|---"):
            continue
        rows.append(
            {
                "id": cid,
                "name": parts[1],
                "triggers": parts[2],
                "ops": parts[3],
                "conditions": parts[4],
                "status": parts[5],
            }
        )
    return rows


def ops_cell(ops: list[str]) -> str:
    return ", ".join(ops) if ops else "—"


def cell_has(cell: str, token: str) -> bool:
    if cell == "—":
        return False
    parts = [p.strip() for p in cell.split(",")]
    return token in parts
