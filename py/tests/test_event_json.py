"""Play-cues: `Game.apply` event JSON shapes (ids, sources, targets)."""

from __future__ import annotations

from typing import Any

import pytest

pytest.importorskip("arena")

PAD = "10001110"
FOLLOWER_B = "10011110"


def pad_deck(primary: str, n: int = 40) -> dict[str, int]:
    return {primary: n}


def mixed_deck(cards: dict[str, int]) -> dict[str, int]:
    total = sum(cards.values())
    out = dict(cards)
    if total < 40:
        out[PAD] = out.get(PAD, 0) + (40 - total)
    return out


def drain_mulligan(game) -> None:
    game.apply({"mulligan": {"player": "a", "swap": [False, False, False, False]}})
    game.apply({"mulligan": {"player": "b", "swap": [False, False, False, False]}})


def started(db, seed: int, deck: dict[str, int] | None = None, opening_hands: dict | None = None):
    import arena

    deck = deck or pad_deck(PAD)
    return arena.Game(
        db,
        seed,
        deck,
        deck,
        first="a",
        opening_hands=opening_hands,
    )


def slot_id(full: dict, player: str, slot: int) -> int:
    inst = full["players"][player]["field"][slot]
    assert inst is not None, f"empty field {player}[{slot}]"
    return inst["id"]


def hand_index(full: dict, player: str, card_id: str) -> int:
    for i, c in enumerate(full["players"][player]["hand"]):
        if c["card"] == card_id:
            return i
    raise AssertionError(f"{card_id} not in {player} hand")


def play_card(game, card_id: str) -> list[dict[str, Any]]:
    full = game.full()
    player = game.active
    pos = hand_index(full, player, card_id)
    return game.apply({"play": {"player": player, "hand_pos": pos, "card": card_id}})


def event_kind(ev: dict[str, Any]) -> str:
    assert len(ev) == 1, ev
    return next(iter(ev))


def slot_target_has_id(target: dict[str, Any]) -> None:
    assert "slot" in target
    assert "player" in target
    assert isinstance(target["id"], int) and target["id"] > 0


def check_event_contract(events: list[dict[str, Any]]) -> None:
    last_resolve: int | None = None
    for i, ev in enumerate(events):
        kind = event_kind(ev)
        if kind == "resolve":
            last_resolve = i
        if kind in ("damage", "restore"):
            target = ev[kind]["target"]
            if "slot" in target:
                assert last_resolve is not None, f"missing resolve before {kind} at {i}"
                slot_target_has_id(target)
        elif kind == "random_pick":
            assert last_resolve is not None, f"missing resolve before random_pick at {i}"
            target = ev[kind]["target"]
            if "slot" in target:
                slot_target_has_id(target)
        elif kind == "destroy":
            body = ev[kind]
            assert body["id"] > 0, f"destroy without id at {i}"
            assert body["player"] in ("a", "b")


def end_turn(game) -> None:
    game.apply({"end_turn": {"player": game.active}})


def wait_for(game, player: str) -> None:
    while game.active != player:
        end_turn(game)


def ensure_pp_for(game, player: str, need: int) -> None:
    while True:
        wait_for(game, player)
        st = game.full()["players"][player]
        if st["pp"] >= need:
            return
        end_turn(game)


def play_on(game, player: str, card_id: str) -> list[dict[str, Any]]:
    wait_for(game, player)
    return play_card(game, card_id)


def opponent_field_followers(game, *card_ids: str) -> None:
    for card_id in card_ids:
        ensure_pp_for(game, "b", 2)
        play_on(game, "b", card_id)
        end_turn(game)
    wait_for(game, "a")


def test_damage_slot_ids(db) -> None:
    deck = mixed_deck({"10753310": 1, PAD: 1, FOLLOWER_B: 1})
    game = started(
        db,
        43,
        deck,
        opening_hands={
            "a": ["10753310", PAD, PAD, PAD],
            "b": [PAD, FOLLOWER_B, PAD, PAD],
        },
    )
    drain_mulligan(game)
    opponent_field_followers(game, PAD, FOLLOWER_B)
    ensure_pp_for(game, "a", 3)
    full = game.full()
    id0 = slot_id(full, "b", 0)
    id1 = slot_id(full, "b", 1)
    events = play_on(game, "a", "10753310")
    check_event_contract(events)
    slot_damages = [
        (e["damage"]["target"]["player"], e["damage"]["target"]["slot"], e["damage"]["target"]["id"])
        for e in events
        if event_kind(e) == "damage" and "slot" in e["damage"]["target"]
    ]
    assert len(slot_damages) >= 2, slot_damages
    ids = {uid for _, _, uid in slot_damages}
    assert id0 in ids
    assert id1 in ids
    assert not any(
        event_kind(e) == "damage" and "leader" in e["damage"]["target"] for e in events
    )


def test_restore_slot_ids(db) -> None:
    deck = mixed_deck({"10411110": 1, PAD: 2})
    game = started(
        db,
        60,
        deck,
        opening_hands={
            "a": ["10411110", PAD, PAD, PAD],
            "b": [PAD, PAD, PAD, PAD],
        },
    )
    drain_mulligan(game)
    ensure_pp_for(game, "a", 2)
    play_on(game, "a", PAD)
    full = game.full()
    ally_id = slot_id(full, "a", 0)
    ensure_pp_for(game, "a", 7)
    play_on(game, "a", "10411110")
    full = game.full()
    kou_slot = next(
        s
        for s, inst in enumerate(full["players"]["a"]["field"])
        if inst and inst["card"] == "10411110"
    )
    end_turn(game)
    end_turn(game)
    wait_for(game, "a")
    events = game.apply(
        {"attack": {"player": "a", "attacker_slot": kou_slot, "target": "leader"}}
    )
    check_event_contract(events)
    restores = [
        e["restore"]["target"]
        for e in events
        if event_kind(e) == "restore" and "slot" in e["restore"]["target"]
    ]
    assert restores, events
    for target in restores:
        slot_target_has_id(target)
    assert any(t["id"] == ally_id for t in restores)


def test_destroy_player_and_id(db) -> None:
    deck = pad_deck("10011210")
    game = started(
        db,
        59,
        deck,
        opening_hands={
            "a": ["10011210", "10011210", "10011210", "10011210"],
            "b": ["10011210", "10011210", "10011210", "10011210"],
        },
    )
    drain_mulligan(game)
    ensure_pp_for(game, "b", 2)
    events = play_on(game, "b", "10011210")
    check_event_contract(events)
    full = game.full()
    uid = slot_id(full, "b", 0)
    end_turn(game)
    end_turn(game)
    end_turn(game)
    events = game.apply({"end_turn": {"player": "a"}})
    check_event_contract(events)
    destroy = next(e["destroy"] for e in events if event_kind(e) == "destroy")
    assert destroy["player"] == "b"
    assert destroy["id"] == uid


def test_random_pick_target_matches_damage(db) -> None:
    deck = mixed_deck({"90011110": 1, PAD: 2, FOLLOWER_B: 1, "10011210": 1})
    game = started(
        db,
        44,
        deck,
        opening_hands={
            "a": ["90011110", "10011210", PAD, PAD],
            "b": [PAD, FOLLOWER_B, PAD, PAD],
        },
    )
    drain_mulligan(game)
    opponent_field_followers(game, PAD, FOLLOWER_B)
    ensure_pp_for(game, "a", 3)
    play_on(game, "a", "10011210")
    events = play_on(game, "a", "90011110")
    check_event_contract(events)
    picks = [
        e["random_pick"]["target"]
        for e in events
        if event_kind(e) == "random_pick" and e["random_pick"]["what"] == "random_target"
    ]
    assert len(picks) == 1
    slot_target_has_id(picks[0])
    dmg_id = next(
        e["damage"]["target"]["id"]
        for e in events
        if event_kind(e) == "damage" and "slot" in e["damage"]["target"]
    )
    assert picks[0]["id"] == dmg_id


def test_resolve_spell_source_before_damage(db) -> None:
    deck = mixed_deck({"10753310": 1, PAD: 1})
    game = started(
        db,
        48,
        deck,
        opening_hands={
            "a": ["10753310", PAD, PAD, PAD],
            "b": [PAD, PAD, PAD, PAD],
        },
    )
    drain_mulligan(game)
    opponent_field_followers(game, PAD)
    ensure_pp_for(game, "a", 3)
    events = play_on(game, "a", "10753310")
    check_event_contract(events)
    dmg_idx = next(i for i, e in enumerate(events) if event_kind(e) == "damage")
    resolve_idx = next(
        i
        for i, e in enumerate(events)
        if event_kind(e) == "resolve"
        and "spell" in e["resolve"]["source"]
        and e["resolve"]["source"]["spell"]["card"] == "10753310"
    )
    assert resolve_idx < dmg_idx


def test_choice_offered_spell_source(db) -> None:
    deck = mixed_deck({"10671310": 1, PAD: 1})
    game = started(
        db,
        56,
        deck,
        opening_hands={
            "a": ["10671310", PAD, PAD, PAD],
            "b": [PAD, PAD, PAD, PAD],
        },
    )
    drain_mulligan(game)
    opponent_field_followers(game, PAD)
    ensure_pp_for(game, "a", 2)
    events = play_on(game, "a", "10671310")
    check_event_contract(events)
    offered = next(e["choice_offered"] for e in events if event_kind(e) == "choice_offered")
    source = offered["source"]
    assert "spell" in source
    assert source["spell"]["player"] == "a"
    assert source["spell"]["card"] == "10671310"
