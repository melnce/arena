#!/usr/bin/env python3
"""Emit schema/cards.schema.json. Run from repo root."""

from __future__ import annotations

import json
from pathlib import Path


def closed(properties: dict, required=None, **extra):
    out = {
        "type": "object",
        "additionalProperties": False,
        "properties": properties,
    }
    if required:
        out["required"] = required
    out.update(extra)
    return out


def const_obj(name, value):
    return {"type": "object", "additionalProperties": False, "properties": {name: {"const": value}}, "required": [name]}


TRIBES = [
    "anathema",
    "artifact",
    "departed",
    "earth sigil",
    "encroacher",
    "golem",
    "loot",
    "marine",
    "mysteria",
    "officer",
    "pixie",
    "puppetry",
]

TRAIT_BOOLS = [
    "ward",
    "bane",
    "rush",
    "storm",
    "ambush",
    "aura",
    "intimidate",
    "drain",
    "barrier",
    "ignoresWard",
    "cantAttackFollowers",
    "cantAttackLeader",
    "cantBeDestroyedByAbilities",
    "cantBePlayed",
]

EVENTS = [
    "ally_follower_enter",
    "enemy_follower_enter",
    "ally_follower_destroyed",
    "ally_amulet_destroyed",
    "ally_card_played",
    "ally_spell_played",
    "ally_follower_attacks",
    "enemy_follower_attacks",
    "ally_evolve",
    "ally_super_evolve",
    "ally_draw",
    "ally_earth_rite",
    "ally_engage",
    "leader_restored",
    "self_buffed_up",
]

REF_PICKS = ["self", "entering", "attacker", "defender", "opposing", "selected"]
POOL_PICKS = ["all", "choose", "random", "randomDistinct", "leftmost", "highest", "lowest"]
VARS = ["X", "Y", "Z"]
TRAIT_KEYS = TRAIT_BOOLS + ["attacksPerTurn", "damageCap"]
TRIGGER_TAGS = [
    "fanfare",
    "lastWords",
    "evolve",
    "superEvolve",
    "anyEvolve",
    "anySuperEvolve",
    "strike",
    "followerStrike",
    "clash",
    "enter",
    "leave",
    "discarded",
    "invoked",
    "fused",
    "spellboost",
    "engage",
    "startOfTurn",
    "endOfTurn",
    "when",
    "enhance",
]
OPTIONS_FROM = [
    "fanfare",
    "lastWords",
    "evolve",
    "superEvolve",
    "anyEvolve",
    "anySuperEvolve",
    "strike",
    "followerStrike",
    "clash",
    "enter",
    "leave",
    "discarded",
    "invoked",
    "fused",
    "spellboost",
    "engage",
    "startOfTurn",
    "endOfTurn",
    "when",
]

REPLICATE_KEYS = [
    "fanfare",
    "evolve",
    "superEvolve",
    "lastWords",
    "engage",
    "strike",
    "followerStrike",
    "clash",
]


def traits_schema():
    props = {k: {"type": "boolean"} for k in TRAIT_BOOLS}
    props["attacksPerTurn"] = {"type": "integer", "minimum": 1}
    props["damageCap"] = {"type": "integer", "minimum": 0}
    return closed(props)


def filter_schema():
    return closed(
        {
            "all": {"type": "array", "items": {"$ref": "#/$defs/Filter"}, "minItems": 1},
            "any": {"type": "array", "items": {"$ref": "#/$defs/Filter"}, "minItems": 1},
            "not": {"$ref": "#/$defs/Filter"},
            "tribe": {
                "oneOf": [
                    {"$ref": "#/$defs/Tribe"},
                    {"type": "array", "items": {"$ref": "#/$defs/Tribe"}, "minItems": 1},
                ]
            },
            "card": {"$ref": "#/$defs/CardId"},
            "cards": {"type": "array", "items": {"$ref": "#/$defs/CardId"}, "minItems": 1},
            "notCard": {"$ref": "#/$defs/CardId"},
            "kind": {"enum": ["follower", "spell", "amulet", "card"]},
            "class": {"$ref": "#/$defs/Class"},
            "costEq": {"$ref": "#/$defs/Amount"},
            "costLte": {"$ref": "#/$defs/Amount"},
            "costGte": {"$ref": "#/$defs/Amount"},
            "costIn": {"type": "array", "items": {"type": "integer"}, "minItems": 1},
            "baseCostEq": {"$ref": "#/$defs/Amount"},
            "baseCostLte": {"$ref": "#/$defs/Amount"},
            "baseCostGte": {"$ref": "#/$defs/Amount"},
            "baseCostIn": {"type": "array", "items": {"type": "integer"}, "minItems": 1},
            "attackLte": {"$ref": "#/$defs/Amount"},
            "attackGte": {"$ref": "#/$defs/Amount"},
            "defenseLte": {"$ref": "#/$defs/Amount"},
            "defenseGte": {"$ref": "#/$defs/Amount"},
            "evolved": {"type": "boolean"},
            "unevolved": {"type": "boolean"},
            "damaged": {"type": "boolean"},
            "hasTrait": {"enum": TRAIT_KEYS},
            "enhanced": {"type": "boolean"},
            "sameCostGroup": {"type": "boolean"},
            "hasLastWords": {"type": "boolean"},
            "destroyedThisMatch": {"type": "boolean"},
        }
    )


def selector_schema():
    ref_plain = [
        closed({"pick": {"const": p}}, required=["pick"]) for p in REF_PICKS
    ]
    ref_bound = closed(
        {"pick": {"const": "bound"}, "ref": {"type": "string", "minLength": 1}},
        required=["pick", "ref"],
    )
    pool = closed(
        {
            "pick": {"enum": POOL_PICKS},
            "side": {"enum": ["ally", "enemy", "any"]},
            "zone": {"enum": ["field", "hand", "deck", "cemetery", "leader", "crests"]},
            "kind": {"enum": ["follower", "amulet", "card", "leader", "character", "faith"]},
            "filter": {"$ref": "#/$defs/Filter"},
            "count": {"$ref": "#/$defs/Amount"},
            "other": {"type": "boolean"},
            "includeLeader": {"type": "boolean"},
            "orderBy": {"enum": ["attack", "defense", "cost", "baseCost"]},
        },
        required=["pick", "side", "zone", "kind"],
    )
    return {"oneOf": [*ref_plain, ref_bound, pool]}


def amount_schema():
    return {
        "oneOf": [
            {"type": "integer"},
            closed({"count": {"$ref": "#/$defs/Selector"}}, required=["count"]),
            closed({"counter": {"$ref": "#/$defs/CounterKey"}}, required=["counter"]),
            closed(
                {
                    "stat": closed(
                        {
                            "of": {"$ref": "#/$defs/Selector"},
                            "which": {"enum": ["attack", "defense", "cost", "baseCost"]},
                        },
                        required=["of", "which"],
                    )
                },
                required=["stat"],
            ),
            closed({"var": {"enum": VARS}}, required=["var"]),
            closed(
                {
                    "add": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Amount"},
                        "minItems": 2,
                        "maxItems": 2,
                    }
                },
                required=["add"],
            ),
            closed(
                {
                    "sub": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Amount"},
                        "minItems": 2,
                        "maxItems": 2,
                    }
                },
                required=["sub"],
            ),
            closed(
                {
                    "max": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Amount"},
                        "minItems": 2,
                        "maxItems": 2,
                    }
                },
                required=["max"],
            ),
            closed(
                {
                    "min": {
                        "type": "array",
                        "items": {"$ref": "#/$defs/Amount"},
                        "minItems": 2,
                        "maxItems": 2,
                    }
                },
                required=["min"],
            ),
            closed({"neg": {"$ref": "#/$defs/Amount"}}, required=["neg"]),
            closed(
                {"distinctNames": {"$ref": "#/$defs/Selector"}},
                required=["distinctNames"],
            ),
            closed(
                {"enteredThisMatch": {"$ref": "#/$defs/Filter"}},
                required=["enteredThisMatch"],
            ),
        ]
    }


def condition_schema():
    return {
        "oneOf": [
            closed({"all": {"type": "array", "items": {"$ref": "#/$defs/Condition"}, "minItems": 1}}, required=["all"]),
            closed({"any": {"type": "array", "items": {"$ref": "#/$defs/Condition"}, "minItems": 1}}, required=["any"]),
            closed({"not": {"$ref": "#/$defs/Condition"}}, required=["not"]),
            closed(
                {
                    "countAtLeast": closed(
                        {"select": {"$ref": "#/$defs/Selector"}, "n": {"$ref": "#/$defs/Amount"}},
                        required=["select", "n"],
                    )
                },
                required=["countAtLeast"],
            ),
            closed(
                {
                    "counterAtLeast": closed(
                        {"key": {"$ref": "#/$defs/CounterKey"}, "n": {"$ref": "#/$defs/Amount"}},
                        required=["key", "n"],
                    )
                },
                required=["counterAtLeast"],
            ),
            closed({"evolved": {"type": "boolean"}}, required=["evolved"]),
            closed({"superEvolutionUnlocked": {"type": "boolean"}}, required=["superEvolutionUnlocked"]),
            closed({"combo": closed({"n": {"$ref": "#/$defs/Amount"}}, required=["n"])}, required=["combo"]),
            closed({"rally": closed({"n": {"$ref": "#/$defs/Amount"}}, required=["n"])}, required=["rally"]),
            closed({"overflow": {"type": "boolean"}}, required=["overflow"]),
            closed({"skyboundArt": closed({"n": {"$ref": "#/$defs/Amount"}}, required=["n"])}, required=["skyboundArt"]),
            closed(
                {"wasFused": {"oneOf": [{"type": "boolean"}, {"const": "both"}]}},
                required=["wasFused"],
            ),
            closed({"did": {"type": "string", "minLength": 1}}, required=["did"]),
            closed({"attackedLeaderLastTurn": {"type": "boolean"}}, required=["attackedLeaderLastTurn"]),
            closed({"turnOwner": {"enum": ["self", "opponent"]}}, required=["turnOwner"]),
            closed(
                {"evolvedCountAtLeast": closed({"n": {"$ref": "#/$defs/Amount"}}, required=["n"])},
                required=["evolvedCountAtLeast"],
            ),
            closed(
                {
                    "playedBaseCostsThisMatch": {
                        "type": "array",
                        "items": {"type": "integer", "minimum": 1},
                        "minItems": 1,
                    }
                },
                required=["playedBaseCostsThisMatch"],
            ),
            closed(
                {
                    "handHas": closed(
                        {"filter": {"$ref": "#/$defs/Filter"}, "n": {"$ref": "#/$defs/Amount"}},
                        required=["filter", "n"],
                    )
                },
                required=["handHas"],
            ),
            closed(
                {
                    "fieldHas": closed(
                        {
                            "filter": {"$ref": "#/$defs/Filter"},
                            "n": {"$ref": "#/$defs/Amount"},
                            "side": {"enum": ["ally", "enemy", "any"]},
                            "kind": {"enum": ["follower", "amulet", "card", "character"]},
                        },
                        required=["filter"],
                    )
                },
                required=["fieldHas"],
            ),
            closed({"leaderDefenseLte": {"$ref": "#/$defs/Amount"}}, required=["leaderDefenseLte"]),
            closed(
                {
                    "varAtLeast": closed(
                        {"key": {"enum": VARS}, "n": {"$ref": "#/$defs/Amount"}},
                        required=["key", "n"],
                    )
                },
                required=["varAtLeast"],
            ),
            closed(
                {
                    "enterCountAtLeast": closed(
                        {
                            "card": {"$ref": "#/$defs/CardId"},
                            "n": {"$ref": "#/$defs/Amount"},
                            "other": {"type": "boolean"},
                            "side": {"enum": ["ally", "enemy", "any"]},
                        },
                        required=["card", "n"],
                    )
                },
                required=["enterCountAtLeast"],
            ),
            closed(
                {"handSameCostAtLeast": closed({"n": {"$ref": "#/$defs/Amount"}}, required=["n"])},
                required=["handSameCostAtLeast"],
            ),
        ]
    }


def common_effect_fields():
    return {
        "printed": {"type": "string", "minLength": 1},
        "as": {"type": "string", "minLength": 1},
        "when": {"$ref": "#/$defs/Condition"},
    }


def leaf(op, extra_props, extra_req):
    props = {"op": {"const": op}, **common_effect_fields(), **extra_props}
    return closed(props, required=["op", *extra_req])


def card_source():
    return {
        "oneOf": [
            closed({"named": {"$ref": "#/$defs/CardId"}}, required=["named"]),
            closed(
                {"copyOf": {"$ref": "#/$defs/Selector"}, "exact": {"type": "boolean"}},
                required=["copyOf", "exact"],
            ),
            closed({"randomFrom": {"$ref": "#/$defs/Filter"}}, required=["randomFrom"]),
        ]
    }


def effect_schema():
    effects = [
        leaf("damage", {
            "select": {"$ref": "#/$defs/Selector"},
            "amount": {"$ref": "#/$defs/Amount"},
            "split": {"type": "boolean"},
        }, ["select", "amount"]),
        leaf("restore", {
            "select": {"$ref": "#/$defs/Selector"},
            "amount": {"$ref": "#/$defs/Amount"},
        }, ["select", "amount"]),
        leaf("buff", {
            "select": {"$ref": "#/$defs/Selector"},
            "attack": {"$ref": "#/$defs/Amount"},
            "defense": {"$ref": "#/$defs/Amount"},
            "untilEndOfTurn": {"type": "boolean"},
        }, ["select"]),
        leaf("destroy", {"select": {"$ref": "#/$defs/Selector"}}, ["select"]),
        leaf("banish", {"select": {"$ref": "#/$defs/Selector"}}, ["select"]),
        leaf("returnToHand", {"select": {"$ref": "#/$defs/Selector"}}, ["select"]),
        leaf("returnToDeck", {
            "select": {"$ref": "#/$defs/Selector"},
            "position": {"enum": ["random"]},
        }, ["select", "position"]),
        leaf("summon", {
            "card": {"$ref": "#/$defs/CardSource"},
            "count": {"$ref": "#/$defs/Amount"},
            "controller": {"enum": ["self", "opponent"]},
        }, ["card", "count"]),
        leaf("reanimate", {"maxCost": {"$ref": "#/$defs/Amount"}}, ["maxCost"]),
        leaf("addToHand", {
            "card": {"$ref": "#/$defs/CardSource"},
            "count": {"$ref": "#/$defs/Amount"},
        }, ["card", "count"]),
        leaf("draw", {
            "count": {"$ref": "#/$defs/Amount"},
            "filter": {"$ref": "#/$defs/Filter"},
        }, ["count"]),
        leaf("discard", {"select": {"$ref": "#/$defs/Selector"}}, ["select"]),
        leaf("addToDeck", {
            "card": {"$ref": "#/$defs/CardSource"},
            "count": {"$ref": "#/$defs/Amount"},
            "position": {"enum": ["random"]},
        }, ["card", "count", "position"]),
        leaf("evolve", {
            "select": {"$ref": "#/$defs/Selector"},
            "super": {"type": "boolean"},
        }, ["select", "super"]),
        leaf("grantTraits", {
            "select": {"$ref": "#/$defs/Selector"},
            "traits": {"$ref": "#/$defs/Traits"},
        }, ["select", "traits"]),
        leaf("removeTraits", {
            "select": {"$ref": "#/$defs/Selector"},
            "traits": {"$ref": "#/$defs/Traits"},
        }, ["select", "traits"]),
        leaf("grantAbility", {
            "select": {"$ref": "#/$defs/Selector"},
            "ability": {"$ref": "#/$defs/Ability"},
        }, ["select", "ability"]),
        leaf("removeAbilities", {
            "select": {"$ref": "#/$defs/Selector"},
            "on": {"type": "array", "items": {"enum": TRIGGER_TAGS}, "minItems": 1},
        }, ["select"]),
        leaf("cost", {
            "select": {"$ref": "#/$defs/Selector"},
            "delta": {"$ref": "#/$defs/Amount"},
            "set": {"$ref": "#/$defs/Amount"},
            "untilEndOfTurn": {"type": "boolean"},
        }, ["select"]),
        leaf("pp", {
            "action": {"enum": ["gainMax", "recover", "spend"]},
            "amount": {"$ref": "#/$defs/Amount"},
        }, ["action", "amount"]),
        leaf("ep", {
            "action": {"enum": ["gain", "spend"]},
            "super": {"type": "boolean"},
            "amount": {"$ref": "#/$defs/Amount"},
        }, ["action", "super", "amount"]),
        leaf("crest", {
            "gain": {"type": "string", "pattern": "^crest:[0-9]{8}$"},
            "player": {"enum": ["self", "opponent"]},
        }, ["gain", "player"]),
        leaf("removeCrests", {
            "select": {"$ref": "#/$defs/Selector"},
        }, ["select"]),
        leaf("countdown", {
            "select": {"$ref": "#/$defs/Selector"},
            "delta": {"$ref": "#/$defs/Amount"},
        }, ["select", "delta"]),
        leaf("counter", {
            "key": {"$ref": "#/$defs/CounterKey"},
            "how": {"enum": ["add", "set"]},
            "amount": {"$ref": "#/$defs/Amount"},
        }, ["key", "how", "amount"]),
        leaf("pay", {
            "resource": {"enum": ["shadows", "earth", "pp", "faith"]},
            "amount": {"$ref": "#/$defs/Amount"},
            "effects": {"type": "array", "items": {"$ref": "#/$defs/Effect"}, "minItems": 1},
        }, ["resource", "amount", "effects"]),
        leaf("transform", {
            "select": {"$ref": "#/$defs/Selector"},
            "into": {"$ref": "#/$defs/CardSource"},
        }, ["select", "into"]),
        leaf("leaderModifier", {
            "select": {"$ref": "#/$defs/Selector"},
            "maxDefense": {"$ref": "#/$defs/Amount"},
            "damageCap": {"$ref": "#/$defs/Amount"},
            "damageTakenBonus": {"$ref": "#/$defs/Amount"},
            "until": {"enum": ["endOfTurn", "endOfOpponentTurn"]},
        }, ["select"]),
        leaf("replicate", {
            "ability": {"enum": REPLICATE_KEYS},
        }, ["ability"]),
        leaf("invoke", {}, []),
        leaf("spellboostHand", {
            "times": {"$ref": "#/$defs/Amount"},
        }, ["times"]),
        leaf("randomSplit", {
            "keys": {"type": "array", "items": {"enum": VARS}, "minItems": 2},
            "times": {"$ref": "#/$defs/Amount"},
            "effects": {"type": "array", "items": {"$ref": "#/$defs/ClauseRoot"}, "minItems": 1},
        }, ["keys", "times", "effects"]),
        # combinators
        leaf("seq", {
            "effects": {"type": "array", "items": {"$ref": "#/$defs/Effect"}, "minItems": 1},
        }, ["effects"]),
        leaf("if", {
            "cond": {"$ref": "#/$defs/Condition"},
            "then": {"type": "array", "items": {"$ref": "#/$defs/Effect"}, "minItems": 1},
            "else": {"type": "array", "items": {"$ref": "#/$defs/Effect"}, "minItems": 1},
        }, ["cond", "then"]),
        leaf("choose", {
            "pick": {"oneOf": [{"type": "integer", "minimum": 1}, {"const": "all"}]},
            "by": {"enum": ["player", "randomUnused", "random"]},
            "options": {
                "type": "array",
                "minItems": 1,
                "items": closed(
                    {
                        "printed": {"type": "string", "minLength": 1},
                        "effects": {"type": "array", "items": {"$ref": "#/$defs/Effect"}, "minItems": 1},
                    },
                    required=["printed", "effects"],
                ),
            },
        }, ["pick", "by", "options"]),
        leaf("choose", {
            "pick": {"oneOf": [{"type": "integer", "minimum": 1}, {"const": "all"}]},
            "by": {"enum": ["player", "randomUnused", "random"]},
            "optionsFrom": {"enum": OPTIONS_FROM},
        }, ["pick", "by", "optionsFrom"]),
        leaf("repeat", {
            "times": {"$ref": "#/$defs/Amount"},
            "effects": {"type": "array", "items": {"$ref": "#/$defs/Effect"}, "minItems": 1},
        }, ["times", "effects"]),
        leaf("sequence", {
            "steps": {
                "type": "array",
                "minItems": 1,
                "items": closed(
                    {
                        "printed": {"type": "string", "minLength": 1},
                        "effects": {"type": "array", "items": {"$ref": "#/$defs/Effect"}, "minItems": 1},
                    },
                    required=["printed", "effects"],
                ),
            },
        }, ["steps"]),
    ]
    return {"oneOf": effects}


def clause_root():
    return {
        "allOf": [
            {"type": "object", "required": ["printed"], "properties": {"printed": {"type": "string", "minLength": 1}}},
            {"$ref": "#/$defs/Effect"},
        ]
    }


def trigger_extras():
    """Ability is a closed object with `on` discriminator."""
    base = {
        "printed": {"type": "string", "minLength": 1},
        "effects": {"type": "array", "items": {"$ref": "#/$defs/ClauseRoot"}, "minItems": 1},
        "zone": {"enum": ["field", "hand", "deck"]},
        "oncePerTurn": {"type": "boolean"},
        "when": {"$ref": "#/$defs/Condition"},
        "replaces": {"const": "evolve"},
    }
    simple = [
        "fanfare", "lastWords", "evolve", "superEvolve", "anyEvolve", "anySuperEvolve",
        "strike", "followerStrike", "clash", "enter", "leave", "discarded",
        "invoked", "fused", "spellboost",
    ]
    variants = []
    for on in simple:
        props = {"on": {"const": on}, **base}
        req = ["on", "printed", "effects"]
        variants.append(closed(props, required=req))
    variants.append(closed(
        {"on": {"const": "engage"}, "cost": {"type": "integer", "minimum": 0}, "sacrifice": {"type": "boolean"}, **base},
        required=["on", "cost", "sacrifice", "printed", "effects"],
    ))
    variants.append(closed(
        {"on": {"const": "startOfTurn"}, "whose": {"enum": ["own", "opponent"]}, **base},
        required=["on", "whose", "printed", "effects"],
    ))
    variants.append(closed(
        {"on": {"const": "endOfTurn"}, "whose": {"enum": ["own", "opponent"]}, **base},
        required=["on", "whose", "printed", "effects"],
    ))
    variants.append(closed(
        {
            "on": {"const": "when"},
            "event": {"enum": EVENTS},
            "filter": {"$ref": "#/$defs/Filter"},
            **base,
        },
        required=["on", "event", "printed", "effects"],
    ))
    variants.append(closed(
        {
            "on": {"const": "static"},
            "printed": {"type": "string", "minLength": 1},
            "modifier": closed(
                {
                    "suppress": {
                        "type": "array",
                        "items": {"enum": TRIGGER_TAGS},
                        "minItems": 1,
                    },
                    "select": {"$ref": "#/$defs/Selector"},
                },
                required=["suppress", "select"],
            ),
            "zone": {"enum": ["field", "hand", "deck"]},
            "when": {"$ref": "#/$defs/Condition"},
        },
        required=["on", "printed", "modifier"],
    ))
    return {"oneOf": variants}


def mode_schema():
    effects = {"type": "array", "items": {"$ref": "#/$defs/ClauseRoot"}, "minItems": 1}
    enhance = closed(
        {
            "kind": {"const": "enhance"},
            "cost": {"type": "integer", "minimum": 0},
            "printed": {"type": "string", "minLength": 1},
            "effects": effects,
            "replacesBase": {"type": "boolean"},
        },
        required=["kind", "cost", "printed", "effects"],
    )
    accelerate = closed(
        {
            "kind": {"const": "accelerate"},
            "cost": {"type": "integer", "minimum": 0},
            "printed": {"type": "string", "minLength": 1},
            "effects": effects,
        },
        required=["kind", "cost", "printed", "effects"],
    )
    crystallize = closed(
        {
            "kind": {"const": "crystallize"},
            "cost": {"type": "integer", "minimum": 0},
            "printed": {"type": "string", "minLength": 1},
            "countdown": {"type": "integer", "minimum": 0},
            "traits": {"$ref": "#/$defs/Traits"},
            "abilities": {"type": "array", "items": {"$ref": "#/$defs/Ability"}},
        },
        required=["kind", "cost", "printed"],
    )
    return {"oneOf": [enhance, accelerate, crystallize]}


def fuse_schema():
    result = {
        "oneOf": [
            closed({"transformInto": {"$ref": "#/$defs/CardId"}}, required=["transformInto"]),
            closed({"consume": {"const": True}}, required=["consume"]),
        ]
    }
    recipe = closed(
        {
            "partners": {"$ref": "#/$defs/Filter"},
            "costTotal": {"type": "integer", "minimum": 0},
            "costTotalGte": {"type": "integer", "minimum": 0},
            "requires": {"type": "array", "items": {"$ref": "#/$defs/CardId"}, "minItems": 1},
            "result": result,
        },
        required=["result"],
    )
    return closed(
        {
            "printed": {"type": "string", "minLength": 1},
            "partners": {"$ref": "#/$defs/Filter"},
            "recipes": {"type": "array", "items": recipe, "minItems": 1},
        },
        required=["printed", "partners"],
    )


def card_schema():
    common = {
        "id": {"$ref": "#/$defs/CardId"},
        "name": {"type": "string", "minLength": 1},
        "class": {"$ref": "#/$defs/Class"},
        "set": {"type": "integer"},
        "rarity": {"enum": ["bronze", "silver", "gold", "legendary"]},
        "token": {"type": "boolean"},
        "cost": {"type": "integer", "minimum": 0},
        "tribes": {"type": "array", "items": {"$ref": "#/$defs/Tribe"}},
        "text": {"type": "string"},
        "traits": {"$ref": "#/$defs/Traits"},
        "abilities": {"type": "array", "items": {"$ref": "#/$defs/Ability"}},
        "modes": {"type": "array", "items": {"$ref": "#/$defs/Mode"}},
        "fuse": {"$ref": "#/$defs/Fuse"},
        "vars": closed({"X": {"type": "integer"}}, required=["X"]),
    }
    follower = closed(
        {**common, "kind": {"const": "follower"}, "attack": {"type": "integer"}, "defense": {"type": "integer"}},
        required=["id", "name", "kind", "class", "set", "rarity", "token", "cost", "attack", "defense", "text"],
    )
    spell = closed(
        {**common, "kind": {"const": "spell"}},
        required=["id", "name", "kind", "class", "set", "rarity", "token", "cost", "text"],
    )
    amulet = closed(
        {**common, "kind": {"const": "amulet"}, "countdown": {"type": "integer", "minimum": 0}},
        required=["id", "name", "kind", "class", "set", "rarity", "token", "cost", "text"],
    )
    return {"oneOf": [follower, spell, amulet]}


def crest_schema():
    return closed(
        {
            "id": {"$ref": "#/$defs/CrestId"},
            "name": {"type": "string", "minLength": 1},
            "grantedBy": {"type": "array", "items": {"$ref": "#/$defs/CardId"}, "minItems": 1},
            "faith": {"type": "boolean"},
            "text": {"type": "string", "minLength": 1},
            "countdown": {"type": "integer", "minimum": 0},
            "abilities": {"type": "array", "items": {"$ref": "#/$defs/Ability"}},
        },
        required=["id", "name", "grantedBy", "faith", "text"],
    )


def build():
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://arena.invalid/schema/cards.schema.json",
        "title": "arena card or crest",
        "oneOf": [
            {"$ref": "#/$defs/Card"},
            {"$ref": "#/$defs/Crest"},
        ],
        "$defs": {
            "CardId": {"type": "string", "pattern": "^[0-9]{8}$"},
            "CrestId": {"type": "string", "pattern": "^(crest|faith):[0-9]{8}$"},
            "Class": {
                "enum": [
                    "neutral",
                    "forestcraft",
                    "swordcraft",
                    "runecraft",
                    "dragoncraft",
                    "abysscraft",
                    "havencraft",
                    "portalcraft",
                ]
            },
            "Tribe": {"enum": TRIBES},
            "Traits": traits_schema(),
            "CounterKey": {
                "oneOf": [
                    {"enum": ["combo", "earth", "faith", "shadows", "skyboundHand"]},
                    closed({"var": {"enum": VARS}}, required=["var"]),
                ]
            },
            "Filter": filter_schema(),
            "Selector": selector_schema(),
            "Amount": amount_schema(),
            "Condition": condition_schema(),
            "CardSource": card_source(),
            "Effect": effect_schema(),
            "ClauseRoot": clause_root(),
            "Ability": trigger_extras(),
            "Mode": mode_schema(),
            "Fuse": fuse_schema(),
            "Card": card_schema(),
            "Crest": crest_schema(),
        },
    }


def main():
    path = Path(__file__).resolve().parents[1] / "schema" / "cards.schema.json"
    path.write_text(json.dumps(build(), indent=2) + "\n")
    print("wrote", path)


if __name__ == "__main__":
    main()
