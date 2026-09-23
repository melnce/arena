"""H0 bot_action_explain: identity with bot_action / bot_action_value."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

SPECS = ("h0", "h0:value=v0", "h0:nodes=6000", "h0:k=1", "h0:info=all")


def _collect_states(db, n: int = 40):
    import arena

    root = Path(__file__).resolve().parents[2]
    decks_dir = root / "oracle" / "decks"
    stems = [
        "abyss-p8rfn",
        "abyss-pool",
        "afnm-minatodao",
        "basic-forest",
        "basic-portal",
        "basic-rune",
        "dragon-pool",
        "elf-neanisu2",
        "forest-pool",
        "portal-pool",
        "ramp-37772",
        "ramp-claywies",
        "royal-nattui",
        "rune-mach15",
        "rune-pool",
        "sword-pool",
    ]
    decks = []
    for stem in stems:
        path = decks_dir / f"{stem}.json"
        decks.append(json.loads(path.read_text()))
    out = []
    seed = 1
    while len(out) < n and seed < 50_000:
        da = decks[seed % len(decks)]
        dbk = decks[(seed // 3) % len(decks)]
        game = arena.Game(db, seed, da, dbk, first="a")
        while not game.terminal and len(out) < n:
            if game.turn >= 3 and game.phase == "main":
                out.append(game.clone())
            legal = game.legal()
            if not legal:
                break
            game.apply(legal[0])
        seed += 1
    return out


@pytest.mark.parametrize("spec", SPECS)
def test_explain_matches_bot_action(db, spec: str) -> None:
    import arena

    for i, game in enumerate(_collect_states(db, 24)):
        legal = game.legal()
        if not legal:
            continue
        seed = 20260923 + i
        act = game.bot_action(spec, seed)
        val = game.bot_action_value(spec, seed)
        exp = game.bot_action_explain(spec, seed)
        assert exp["chosen"] == act
        assert exp["value"] == val["value"]
        assert exp["chosen_index"] in exp["tie_set"]
        assert exp["chosen_index"] == min(exp["tie_set"])


def test_opaque_policy(db) -> None:
    import arena

    root = Path(__file__).resolve().parents[2]
    deck = json.loads(
        (root / "engine" / "tests" / "fixtures" / "decks" / "basic-neutral-forest.json").read_text()
    )
    game = arena.Game(db, 1, deck, deck, first="a")
    out = game.bot_action_explain("random", 7)
    assert out["path"] == "opaque"
    assert "chosen" in out
    assert "value" not in out
