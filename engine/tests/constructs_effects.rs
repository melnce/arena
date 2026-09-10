//! Discriminating fixtures for Effect constructs (`docs/construct-fixtures.md`).
//! Each `construct_<slug>` fails if the engine ignored that construct.
#![allow(non_snake_case)]

use arena_engine::card::CardKind;
use arena_engine::state::DestroyedRecord;
use arena_engine::{apply, legal_actions, Action, Phase, PlayerId, Slot};

mod common;
use common::*;

fn db_st() -> (arena_engine::CardDb, arena_engine::State) {
    let db = load_db();
    let st = started(&db, 7);
    (db, st)
}

fn cast(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str) {
    give_pp(st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(db, st, PlayerId::A, id);
}

fn b_hp(st: &arena_engine::State) -> i32 {
    st.player(PlayerId::B).leader_defense
}

fn a_hp(st: &arena_engine::State) -> i32 {
    st.player(PlayerId::A).leader_defense
}

fn drain(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn first_field(st: &arena_engine::State, who: PlayerId) -> &arena_engine::CardInstance {
    st.player(who).field.iter().flatten().next().expect("field")
}

fn put_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).expect("card");
    let inst = arena_engine::CardInstance::from_card(card, st.alloc_id());
    st.player_mut(who).deck.push(inst);
}

fn remember_destroyed(st: &mut arena_engine::State, who: PlayerId, id: &str, base_cost: i32) {
    st.player_mut(who).destroyed_history.push(DestroyedRecord {
        card: cid(id),
        base_cost,
        kind: CardKind::Follower,
        owner: who,
        from_field: true,
    });
}

fn play_legal(db: &arena_engine::CardDb, st: &arena_engine::State, id: &str) -> bool {
    legal_actions(db, st).iter().any(|a| match a {
        Action::Play { hand } => st
            .player(st.active)
            .hand
            .get(*hand as usize)
            .is_some_and(|c| c.card.as_str() == id),
        _ => false,
    })
}

fn strip_on(db: &arena_engine::CardDb, st: &mut arena_engine::State, remover: &str) -> u8 {
    let slot = put_field(db, st, PlayerId::B, "89100200");
    cast(db, st, remover);
    slot
}

#[test]
fn construct_op_damage() {
    // no-op engine: enemy leader stays 20
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200020");
    assert_eq!(b_hp(&st), 17);
}

#[test]
fn construct_op_restore() {
    // no-op engine: leader stays at the damaged value
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).leader_defense = 10;
    cast(&db, &mut st, "89200030");
    assert_eq!(a_hp(&st), 14);
}

#[test]
fn construct_op_buff() {
    // no-op engine: enemy stays 2/2
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    cast(&db, &mut st, "89200040");
    let f = first_field(&st, PlayerId::B);
    assert_eq!((f.attack, f.defense), (4, 4));
}

#[test]
fn construct_op_select() {
    // no-op engine: unbound stat is 0, so the leader takes 0
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "89100040");
    cast(&db, &mut st, "89200050");
    drain(&db, &mut st);
    assert_eq!(b_hp(&st), 12, "8-attack follower selected");
}

#[test]
fn construct_op_destroy() {
    // no-op engine: enemy follower remains
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    cast(&db, &mut st, "89200060");
    assert_eq!(field_count(&st, PlayerId::B), 0);
}

#[test]
fn construct_op_banish() {
    // no-op engine: Last Words fire (destroy) and the pile is cemetery
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "89100030");
    let before = b_hp(&st);
    cast(&db, &mut st, "89200070");
    assert_eq!(field_count(&st, PlayerId::B), 0);
    assert_eq!(b_hp(&st), before, "banish skips Last Words");
    assert!(st
        .player(PlayerId::B)
        .banished
        .iter()
        .any(|c| c.card.as_str() == "89100030"));
}

#[test]
fn construct_op_returnToHand() {
    // no-op engine: enemy stays on the field
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    st.player_mut(PlayerId::B).hand.clear();
    cast(&db, &mut st, "89200080");
    assert_eq!(field_count(&st, PlayerId::B), 0);
    assert!(hand_has(&st, PlayerId::B, "88001110"));
}

#[test]
fn construct_op_returnToDeck() {
    // no-op engine: enemy stays on the field
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001320");
    let before = st
        .player(PlayerId::B)
        .deck
        .iter()
        .filter(|c| c.card.as_str() == "88001320")
        .count();
    cast(&db, &mut st, "89200090");
    assert_eq!(field_count(&st, PlayerId::B), 0);
    let after = st
        .player(PlayerId::B)
        .deck
        .iter()
        .filter(|c| c.card.as_str() == "88001320")
        .count();
    assert_eq!(after, before + 1);
}

#[test]
fn construct_op_summon() {
    // no-op engine: allied field stays empty
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200100");
    assert!(field_has(&st, PlayerId::A, "88001110"));
}

#[test]
fn construct_op_reanimate() {
    // no-op engine: cemetery follower stays dead
    let (db, mut st) = db_st();
    remember_destroyed(&mut st, PlayerId::A, "88001110", 1);
    cast(&db, &mut st, "89200110");
    assert!(field_has(&st, PlayerId::A, "88001110"));
}

#[test]
fn construct_op_addToHand() {
    // no-op engine: hand does not gain the named card
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).hand.clear();
    cast(&db, &mut st, "89200120");
    assert!(hand_has(&st, PlayerId::A, "88001110"));
}

#[test]
fn construct_op_draw() {
    // no-op engine: hand size stays 0 after the play
    let (db, mut st) = db_st();
    let before_deck = st.player(PlayerId::A).deck.len();
    st.player_mut(PlayerId::A).hand.clear();
    cast(&db, &mut st, "89200130");
    assert_eq!(st.player(PlayerId::A).hand.len(), 1);
    assert_eq!(st.player(PlayerId::A).deck.len(), before_deck - 1);
}

#[test]
fn construct_op_discard() {
    // no-op engine: the extra hand card stays
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).hand.clear();
    put_hand(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89200140");
    assert!(!hand_has(&st, PlayerId::A, "88001110"));
}

#[test]
fn construct_op_addToDeck() {
    // no-op engine: deck count of the named card is unchanged
    let (db, mut st) = db_st();
    let before = st
        .player(PlayerId::A)
        .deck
        .iter()
        .filter(|c| c.card.as_str() == "88001110")
        .count();
    cast(&db, &mut st, "89200150");
    let after = st
        .player(PlayerId::A)
        .deck
        .iter()
        .filter(|c| c.card.as_str() == "88001110")
        .count();
    assert_eq!(after, before + 1);
}

#[test]
fn construct_op_evolve() {
    // no-op engine: follower stays 2/2 unevolved
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89200160");
    let f = first_field(&st, PlayerId::A);
    assert!(f.evolved);
    assert_eq!((f.attack, f.defense), (4, 4));
}

#[test]
fn construct_op_grantTraits() {
    // no-op engine: enemy is not Ward, so Attack Leader stays legal
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89200170");
    assert!(first_field(&st, PlayerId::B).is_ward());
}

#[test]
fn construct_op_removeTraits() {
    // no-op engine: Ward stays, Attack Leader is omitted
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "89100010");
    put_field(&db, &mut st, PlayerId::A, "88001110");
    assert!(first_field(&st, PlayerId::B).is_ward());
    cast(&db, &mut st, "89200180");
    assert!(!first_field(&st, PlayerId::B).is_ward());
}

#[test]
fn construct_op_grantAbility() {
    // no-op engine: destroying the ally does not ping
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89200190");
    assert!(!first_field(&st, PlayerId::A).granted.is_empty());
}

#[test]
fn construct_op_removeAbilities() {
    // no-op engine: Last Words still fire
    let (db, mut st) = db_st();
    let slot = put_field(&db, &mut st, PlayerId::B, "89100030");
    let before = b_hp(&st);
    cast(&db, &mut st, "89200200");
    assert!(!tags_has(&st, PlayerId::B, slot, "lastWords"));
    destroy_via_spell(&db, &mut st);
    assert_eq!(b_hp(&st), before, "Last Words stripped");
}

fn tags_has(st: &arena_engine::State, who: PlayerId, slot: u8, tag: &str) -> bool {
    st.field_inst(who, slot)
        .is_some_and(|c| c.printed_tags.contains(tag))
}

fn destroy_via_spell(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    cast(db, st, "89200060");
}

fn keep_and_cast(db: &arena_engine::CardDb, st: &mut arena_engine::State, host: &str, spell: &str) {
    give_pp(st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    put_hand(db, st, PlayerId::A, host);
    play_id(db, st, PlayerId::A, spell);
}

#[test]
fn construct_op_cost() {
    // no-op engine: hand card stays at printed cost
    let (db, mut st) = db_st();
    keep_and_cast(&db, &mut st, "88001320", "89200210");
    let c = st
        .player(PlayerId::A)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "88001320")
        .unwrap();
    assert_eq!(c.cost, 0);
}

#[test]
fn construct_op_pp() {
    // no-op engine: pp_max stays 5
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 5, 5);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200220");
    assert_eq!(st.player(PlayerId::A).pp_max, 7);
}

#[test]
fn construct_op_ep() {
    // no-op engine: ep stays at the starting 2
    let (db, mut st) = db_st();
    let before = st.player(PlayerId::A).ep;
    cast(&db, &mut st, "89200230");
    assert_eq!(st.player(PlayerId::A).ep, before + 2);
}

#[test]
fn construct_op_crest() {
    // no-op engine: crests stay empty
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200010");
    assert!(st
        .player(PlayerId::A)
        .crests
        .iter()
        .any(|c| c.id == "crest:89100910"));
}

#[test]
fn construct_op_removeCrests() {
    // no-op engine: the crest stays
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200010");
    cast(&db, &mut st, "89200240");
    assert!(st.player(PlayerId::A).crests.is_empty());
}

#[test]
fn construct_op_countdown() {
    // no-op engine: amulet countdown stays 3
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "89100020");
    cast(&db, &mut st, "89200250");
    assert_eq!(first_field(&st, PlayerId::A).countdown, Some(2));
}

#[test]
fn construct_op_counter() {
    // no-op engine: combo stays at the post-play 1
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).combo = 0;
    cast(&db, &mut st, "89200260");
    assert!(st.player(PlayerId::A).combo >= 2);
}

#[test]
fn construct_op_pay() {
    // no-op engine: 5 damage lands; the play-corpse's 1 shadow is enough
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).shadows = 1;
    cast(&db, &mut st, "89200270");
    assert_eq!(b_hp(&st), 15);
    assert_eq!(st.player(PlayerId::A).shadows, 0);
}

#[test]
fn construct_op_transform() {
    // no-op engine: enemy stays the original vanilla
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    cast(&db, &mut st, "89200280");
    assert!(field_has(&st, PlayerId::B, "88001320"));
}

#[test]
fn construct_op_leaderModifier() {
    // no-op engine: max defense stays 20
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200290");
    assert_eq!(st.player(PlayerId::B).leader_max, 10);
    assert_eq!(b_hp(&st), 10);
}

#[test]
fn construct_op_replicate() {
    // no-op engine: fanfare does not re-run Last Words (no immediate ping)
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).hand.clear();
    give_pp(&mut st, PlayerId::A, 10, 10);
    play_id(&db, &mut st, PlayerId::A, "89200300");
    assert_eq!(b_hp(&st), 19, "replicated Last Words fire on play");
}

#[test]
fn construct_op_invoke() {
    // no-op engine: the deck card stays in the deck at start of turn
    let (db, mut st) = db_st();
    put_deck(&db, &mut st, PlayerId::A, "89100100");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert!(field_has(&st, PlayerId::A, "89100100"));
}

#[test]
fn construct_op_spellboostHand() {
    // no-op engine: spellboost host stays cost 5
    let (db, mut st) = db_st();
    keep_and_cast(&db, &mut st, "89100070", "89200320");
    let c = st
        .player(PlayerId::A)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "89100070")
        .unwrap();
    assert_eq!(c.cost, 2, "1 auto spellboost + 2 from the op");
}

#[test]
fn construct_op_randomSplit() {
    // no-op engine: the card would be playable and its inner damage would land
    let (db, mut st) = db_st();
    assert!(db.require_supported(cid("89200330")).is_err());
    st.player_mut(PlayerId::A).hand.clear();
    put_hand(&db, &mut st, PlayerId::A, "89200330");
    give_pp(&mut st, PlayerId::A, 10, 10);
    assert!(!play_legal(&db, &st, "89200330"));
}

#[test]
fn construct_op_seq() {
    // no-op engine: summoned vanilla stays 2/2 (buff never sees the bind)
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200340");
    let f = first_field(&st, PlayerId::A);
    assert_eq!((f.attack, f.defense), (4, 4));
}

#[test]
fn construct_op_if() {
    // no-op engine: 5 damage lands even without Overflow
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 3, 3);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200350");
    assert_eq!(b_hp(&st), 20, "no Overflow");
    give_pp(&mut st, PlayerId::A, 10, 7);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200350");
    assert_eq!(b_hp(&st), 15);
}

#[test]
fn construct_op_choose() {
    // no-op engine: no choice node, leader stays 20
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200360");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 1);
    assert_eq!(b_hp(&st), 13);
}

#[test]
fn construct_op_repeat() {
    // no-op engine: 0 or 1 damage instead of 3
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200370");
    assert_eq!(b_hp(&st), 17);
}

#[test]
fn construct_op_sequence() {
    // no-op engine: both end-of-turn pings are 0, or the same number twice
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).hand.clear();
    give_pp(&mut st, PlayerId::A, 10, 10);
    play_id(&db, &mut st, PlayerId::A, "89200380");
    end_turn(&db, &mut st);
    assert_eq!(b_hp(&st), 19, "step 0 deals 1");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(b_hp(&st), 16, "step 1 deals 3");
}

#[test]
fn construct_op_buff_untilEndOfTurn() {
    // no-op engine: +3/+3 stays after end of turn (permanent buff)
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    cast(&db, &mut st, "89200390");
    {
        let f = first_field(&st, PlayerId::B);
        assert_eq!((f.attack, f.defense), (5, 5));
    }
    end_turn(&db, &mut st);
    let f = first_field(&st, PlayerId::B);
    assert_eq!((f.attack, f.defense), (2, 2));
}

#[test]
fn construct_op_cost_set() {
    // no-op engine: host stays cost 5
    let (db, mut st) = db_st();
    keep_and_cast(&db, &mut st, "88001320", "89200400");
    let c = st
        .player(PlayerId::A)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "88001320")
        .unwrap();
    assert_eq!(c.cost, 1);
}

#[test]
fn construct_op_cost_delta() {
    // no-op engine: host stays cost 5 (set-to-1 would be 1)
    let (db, mut st) = db_st();
    keep_and_cast(&db, &mut st, "88001320", "89200410");
    let c = st
        .player(PlayerId::A)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "88001320")
        .unwrap();
    assert_eq!(c.cost, 2);
}

#[test]
fn construct_op_counter_how_add() {
    // no-op engine: combo stays 5; a set-to-3 would be 3
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).combo = 5;
    cast(&db, &mut st, "89200420");
    assert_eq!(st.player(PlayerId::A).combo, 9);
}

#[test]
fn construct_op_counter_how_set() {
    // no-op engine: combo stays 5 (or becomes 6 after the play increment)
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).combo = 5;
    cast(&db, &mut st, "89200430");
    assert_eq!(st.player(PlayerId::A).combo, 3);
}

#[test]
fn construct_op_damage_split() {
    // no-op engine: each follower takes the full 4
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001320");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89200440");
    let lost: i32 = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .map(|f| 10 - f.defense)
        .sum();
    assert_eq!(lost, 4, "split spends the pool once; no-split deals 4 each");
}

#[test]
fn construct_op_draw_filter() {
    // no-op engine: the top spell is drawn
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).deck.clear();
    put_deck(&db, &mut st, PlayerId::A, "88001110");
    put_deck(&db, &mut st, PlayerId::A, "89100080");
    st.player_mut(PlayerId::A).hand.clear();
    cast(&db, &mut st, "89200450");
    assert!(hand_has(&st, PlayerId::A, "88001110"));
    assert!(!hand_has(&st, PlayerId::A, "89100080"));
}

#[test]
fn construct_op_draw_distinctNames() {
    // no-op engine: two copies of the same top name
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).deck.clear();
    put_deck(&db, &mut st, PlayerId::A, "88001320");
    for _ in 0..8 {
        put_deck(&db, &mut st, PlayerId::A, "88001110");
    }
    st.player_mut(PlayerId::A).hand.clear();
    cast(&db, &mut st, "89200460");
    let names: std::collections::BTreeSet<_> = st
        .player(PlayerId::A)
        .hand
        .iter()
        .map(|c| c.card.as_str())
        .collect();
    assert_eq!(names.len(), 2);
}

#[test]
fn construct_op_draw_player_opponent() {
    // no-op engine: we draw, the opponent does not
    let (db, mut st) = db_st();
    let their = st.player(PlayerId::B).hand.len();
    let our_deck = st.player(PlayerId::A).deck.len();
    cast(&db, &mut st, "89200470");
    assert_eq!(st.player(PlayerId::A).deck.len(), our_deck);
    assert_eq!(st.player(PlayerId::B).hand.len(), their + 1);
}

#[test]
fn construct_op_ep_action_gain() {
    // no-op engine: ep unchanged
    let (db, mut st) = db_st();
    let before = st.player(PlayerId::A).ep;
    cast(&db, &mut st, "89200480");
    assert_eq!(st.player(PlayerId::A).ep, before + 3);
}

#[test]
fn construct_op_ep_action_spend() {
    // no-op engine: ep unchanged
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).ep = 2;
    cast(&db, &mut st, "89200490");
    assert_eq!(st.player(PlayerId::A).ep, 1);
}

#[test]
fn construct_op_pp_action_gainMax() {
    // no-op engine: pp_max stays 5
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 5, 5);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200500");
    assert_eq!(st.player(PlayerId::A).pp_max, 8);
}

#[test]
fn construct_op_pp_action_recover() {
    // no-op engine: spent pp stays spent
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 1, 5);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200510");
    assert_eq!(st.player(PlayerId::A).pp, 3);
}

#[test]
fn construct_op_pp_action_spend() {
    // no-op engine: pp stays at the post-play value
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 5, 5);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200520");
    assert_eq!(st.player(PlayerId::A).pp, 3);
}

fn pay_unaffordable(id: &str, prep: fn(&mut arena_engine::State)) {
    let (db, mut st) = db_st();
    prep(&mut st);
    st.player_mut(PlayerId::A).hand.clear();
    put_hand(&db, &mut st, PlayerId::A, id);
    give_pp(&mut st, PlayerId::A, 10, 10);
    let offered = play_legal(&db, &st, id);
    play_id(&db, &mut st, PlayerId::A, id);
    assert_eq!(b_hp(&st), 20, "unaffordable pay body skipped");
    let _ = offered;
}

#[test]
fn construct_op_pay_resource_shadows() {
    // no-op engine: 5 damage lands with 0 shadows
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).shadows = 0;
    cast(&db, &mut st, "89200530");
    assert_eq!(b_hp(&st), 20, "corpse shadow 1 < pay 2");
    st.player_mut(PlayerId::A).shadows = 1;
    cast(&db, &mut st, "89200530");
    assert_eq!(b_hp(&st), 15);
    assert_eq!(st.player(PlayerId::A).shadows, 0);
}

#[test]
fn construct_op_pay_resource_earth() {
    // no-op engine: 5 damage lands with 0 earth
    pay_unaffordable("89200540", |st| {
        st.player_mut(PlayerId::A).earth = 0;
    });
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).earth = 1;
    cast(&db, &mut st, "89200540");
    assert_eq!(b_hp(&st), 15);
}

#[test]
fn construct_op_pay_resource_pp() {
    // no-op engine: 5 damage lands without spending the extra pp
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 0, 0);
    st.player_mut(PlayerId::A).hand.clear();
    put_hand(&db, &mut st, PlayerId::A, "89200550");
    // 0-cost spell, 0 pp — the pay-1-pp body must refuse
    play_id(&db, &mut st, PlayerId::A, "89200550");
    assert_eq!(b_hp(&st), 20);
    give_pp(&mut st, PlayerId::A, 1, 1);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200550");
    assert_eq!(b_hp(&st), 15);
    assert_eq!(st.player(PlayerId::A).pp, 0);
}

#[test]
fn construct_op_pay_resource_faith() {
    // no-op engine: 5 damage lands with 0 faith
    pay_unaffordable("89200560", |st| {
        st.player_mut(PlayerId::A).faith = 0;
    });
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).faith = 1;
    cast(&db, &mut st, "89200560");
    assert_eq!(b_hp(&st), 15);
}

#[test]
fn construct_op_leaderModifier_maxDefense_set() {
    // no-op engine: max stays 20
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200570");
    assert_eq!(st.player(PlayerId::B).leader_max, 8);
}

#[test]
fn construct_op_leaderModifier_maxDefense_delta() {
    // no-op engine: leader survives at 20
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200580");
    assert_eq!(st.player(PlayerId::B).leader_max, 0);
    assert!(st.winner == Some(PlayerId::A) || b_hp(&st) == 0);
}

#[test]
fn construct_op_leaderModifier_damageCap() {
    // no-op engine: the 3-damage spell deals 3
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200590");
    cast(&db, &mut st, "89200020");
    assert_eq!(b_hp(&st), 19);
}

#[test]
fn construct_op_leaderModifier_damageTakenBonus() {
    // no-op engine: the 3-damage spell deals 3
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200600");
    cast(&db, &mut st, "89200020");
    assert_eq!(b_hp(&st), 15);
}

#[test]
fn construct_op_leaderModifier_until() {
    // no-op engine: the cap survives end of turn
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200610");
    cast(&db, &mut st, "89200020");
    assert_eq!(b_hp(&st), 19);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    cast(&db, &mut st, "89200020");
    assert_eq!(b_hp(&st), 16, "cap expired");
}

#[test]
fn construct_op_grantTraits_until_endOfTurn() {
    // no-op engine: Ward is permanent
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89200620");
    assert!(first_field(&st, PlayerId::A).is_ward());
    end_turn(&db, &mut st);
    assert!(!first_field(&st, PlayerId::A).is_ward());
}

#[test]
fn construct_op_grantTraits_until_endOfOpponentTurn() {
    // no-op engine: Ward expires at our end of turn
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89200630");
    assert!(first_field(&st, PlayerId::A).is_ward());
    end_turn(&db, &mut st);
    assert!(
        first_field(&st, PlayerId::A).is_ward(),
        "survives our end of turn"
    );
    end_turn(&db, &mut st);
    assert!(!first_field(&st, PlayerId::A).is_ward());
}

#[test]
fn construct_op_summon_controller_opponent() {
    // no-op engine: we own the summon
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200640");
    assert!(field_has(&st, PlayerId::B, "88001110"));
    assert!(!field_has(&st, PlayerId::A, "88001110"));
}

#[test]
fn construct_op_addToDeck_position_random() {
    // no-op engine: deck count unchanged
    let (db, mut st) = db_st();
    let before = st.player(PlayerId::A).deck.len();
    cast(&db, &mut st, "89200650");
    assert_eq!(st.player(PlayerId::A).deck.len(), before + 1);
}

#[test]
fn construct_op_returnToDeck_position_random() {
    // no-op engine: the hand card stays
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    put_hand(&db, &mut st, PlayerId::A, "88001320");
    let before = st.player(PlayerId::A).deck.len();
    play_id(&db, &mut st, PlayerId::A, "89200660");
    assert!(!hand_has(&st, PlayerId::A, "88001320"));
    assert_eq!(st.player(PlayerId::A).deck.len(), before + 1);
}

#[test]
fn construct_op_choose_by_player() {
    // no-op engine: no Choice phase
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200670");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert_eq!(b_hp(&st), 19);
}

#[test]
fn construct_op_choose_by_random() {
    // no-op engine: leader stays 20
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200680");
    assert!(b_hp(&st) == 19 || b_hp(&st) == 13);
}

#[test]
fn construct_op_choose_by_randomUnused() {
    // no-op engine: the same option can fire twice (one leader takes 2)
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).hand.clear();
    give_pp(&mut st, PlayerId::A, 10, 10);
    play_id(&db, &mut st, PlayerId::A, "89200690");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    let dmg_a = 20 - a_hp(&st);
    let dmg_b = 20 - b_hp(&st);
    assert_eq!(dmg_a + dmg_b, 2);
    assert!(dmg_a == 1 && dmg_b == 1, "each option once");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(
        (20 - a_hp(&st)) + (20 - b_hp(&st)),
        2,
        "third resolution is empty"
    );
}

#[test]
fn construct_op_choose_optionsFrom() {
    // no-op engine: Enhance with empty options deals 0
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 1, 1);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200700");
    assert_eq!(b_hp(&st), 16, "pick-all copies both fanfare options");
}

#[test]
fn construct_op_choose_pick() {
    // no-op engine: only one option runs (1, 2, or 4) not two of them
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200710");
    let dmg = 20 - b_hp(&st);
    assert!(
        dmg == 3 || dmg == 5 || dmg == 6,
        "two of {{1,2,4}}, got {dmg}"
    );
}

#[test]
fn construct_op_randomSplit_keys() {
    // no-op engine: same as op_randomSplit — honest stub, card is unplayable
    let db = load_db();
    assert!(db.require_supported(cid("89200330")).is_err());
}

fn assert_tag_stripped(remover: &str, gone: &str, stays: &str) {
    let (db, mut st) = db_st();
    let slot = strip_on(&db, &mut st, remover);
    assert!(
        !tags_has(&st, PlayerId::B, slot, gone),
        "{gone} should be gone"
    );
    assert!(
        tags_has(&st, PlayerId::B, slot, stays),
        "{stays} should remain"
    );
}

#[test]
fn construct_op_removeAbilities_on_fanfare() {
    // no-op engine: fanfare tag stays (or every tag is cleared)
    assert_tag_stripped("89200720", "fanfare", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_lastWords() {
    // no-op engine: lastWords tag stays
    assert_tag_stripped("89200730", "lastWords", "fanfare");
}

#[test]
fn construct_op_removeAbilities_on_evolve() {
    // no-op engine: evolve tag stays
    assert_tag_stripped("89200740", "evolve", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_superEvolve() {
    // no-op engine: superEvolve tag stays
    assert_tag_stripped("89200750", "superEvolve", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_anyEvolve() {
    // no-op engine: anyEvolve tag stays
    assert_tag_stripped("89200760", "anyEvolve", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_anySuperEvolve() {
    // no-op engine: anySuperEvolve tag stays
    assert_tag_stripped("89200770", "anySuperEvolve", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_strike() {
    // no-op engine: strike still pings
    let (db, mut st) = db_st();
    let slot = strip_on(&db, &mut st, "89200780");
    assert!(!tags_has(&st, PlayerId::B, slot, "strike"));
    assert!(tags_has(&st, PlayerId::B, slot, "lastWords"));
}

#[test]
fn construct_op_removeAbilities_on_followerStrike() {
    // no-op engine: followerStrike tag stays
    assert_tag_stripped("89200790", "followerStrike", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_clash() {
    // no-op engine: clash tag stays
    assert_tag_stripped("89200800", "clash", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_enter() {
    // no-op engine: enter tag stays
    assert_tag_stripped("89200810", "enter", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_leave() {
    // no-op engine: leave tag stays
    assert_tag_stripped("89200820", "leave", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_discarded() {
    // no-op engine: discarded tag stays
    assert_tag_stripped("89200830", "discarded", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_invoked() {
    // no-op engine: invoked tag stays
    assert_tag_stripped("89200840", "invoked", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_fused() {
    // no-op engine: fused tag stays
    assert_tag_stripped("89200850", "fused", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_spellboost() {
    // no-op engine: spellboost tag stays
    assert_tag_stripped("89200860", "spellboost", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_engage() {
    // no-op engine: engage tag stays
    assert_tag_stripped("89200870", "engage", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_startOfTurn() {
    // no-op engine: startOfTurn tag stays
    assert_tag_stripped("89200880", "startOfTurn", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_endOfTurn() {
    // no-op engine: endOfTurn tag stays
    assert_tag_stripped("89200890", "endOfTurn", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_when() {
    // no-op engine: when tag stays
    assert_tag_stripped("89200900", "when", "lastWords");
}

#[test]
fn construct_op_removeAbilities_on_enhance() {
    // no-op engine: enhance still replaces the fanfare for 5 damage
    let (db, mut st) = db_st();
    let slot = strip_on(&db, &mut st, "89200910");
    assert!(!tags_has(&st, PlayerId::B, slot, "enhance"));
    assert!(tags_has(&st, PlayerId::B, slot, "fanfare"));
    let inst = st.player_mut(PlayerId::B).field[slot as usize]
        .take()
        .unwrap();
    st.active = PlayerId::B;
    st.player_mut(PlayerId::B).hand.clear();
    st.player_mut(PlayerId::B).hand.push(inst);
    give_pp(&mut st, PlayerId::B, 2, 2);
    let h = (st.player(PlayerId::B).hand.len() - 1) as u8;
    play(&db, &mut st, h);
    assert_eq!(
        a_hp(&st),
        18,
        "enhance stripped: fanfare+enter = 2, not enhance 5"
    );
}

fn replicate_key(id: &str, expect_dmg: i32) {
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).hand.clear();
    give_pp(&mut st, PlayerId::A, 10, 10);
    play_id(&db, &mut st, PlayerId::A, id);
    drain(&db, &mut st);
    assert_eq!(b_hp(&st), 20 - expect_dmg);
}

#[test]
fn construct_op_replicate_ability_fanfare() {
    // no-op engine: evolve does not re-run fanfare (only the play ping)
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).hand.clear();
    give_pp(&mut st, PlayerId::A, 10, 10);
    play_id(&db, &mut st, PlayerId::A, "89200920");
    assert_eq!(b_hp(&st), 19);
    st.player_mut(PlayerId::A).turns_taken = 5;
    st.player_mut(PlayerId::A).ep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(0),
            super_evolve: false,
        },
    )
    .unwrap();
    assert_eq!(b_hp(&st), 18, "replicated fanfare");
}

#[test]
fn construct_op_replicate_ability_evolve() {
    // no-op engine: play does not run the evolve line
    replicate_key("89200930", 1);
}

#[test]
fn construct_op_replicate_ability_superEvolve() {
    // no-op engine: play does not run the super-evolve line
    replicate_key("89200940", 1);
}

#[test]
fn construct_op_replicate_ability_lastWords() {
    // no-op engine: play does not run Last Words
    replicate_key("89200950", 1);
}

#[test]
fn construct_op_replicate_ability_engage() {
    // no-op engine: play does not run Engage
    replicate_key("89200960", 1);
}

#[test]
fn construct_op_replicate_ability_strike() {
    // no-op engine: play does not run Strike
    replicate_key("89200970", 1);
}

#[test]
fn construct_op_replicate_ability_followerStrike() {
    // no-op engine: play does not run Follower Strike
    replicate_key("89200980", 1);
}

#[test]
fn construct_op_replicate_ability_clash() {
    // no-op engine: play does not run Clash
    replicate_key("89200990", 1);
}

#[test]
fn construct_op_grantAbility_ability() {
    // no-op engine: granted stays empty
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89201000");
    assert_eq!(first_field(&st, PlayerId::A).granted.len(), 1);
}

#[test]
fn construct_op_evolve_super() {
    // no-op engine: +2/+2 evolve, or nothing
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89201010");
    let f = first_field(&st, PlayerId::A);
    assert!(f.super_evolved);
    assert_eq!((f.attack, f.defense), (5, 5));
}

#[test]
fn construct_op_reanimate_maxCost() {
    // no-op engine: the cost-4 body is the highest and comes back
    let (db, mut st) = db_st();
    remember_destroyed(&mut st, PlayerId::A, "88001110", 1);
    remember_destroyed(&mut st, PlayerId::A, "88001320", 4);
    cast(&db, &mut st, "89201020");
    assert!(field_has(&st, PlayerId::A, "88001110"));
    assert!(!field_has(&st, PlayerId::A, "88001320"));
}

#[test]
fn construct_op_repeat_times_amount() {
    // no-op engine: 0 repeats (count unread) or a literal 1
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    put_field(&db, &mut st, PlayerId::A, "88001110");
    cast(&db, &mut st, "89201030");
    assert_eq!(b_hp(&st), 18);
}

#[test]
fn construct_op_if_else() {
    // no-op engine: neither branch (20) or always the then-branch (19)
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 3, 3);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89201040");
    assert_eq!(b_hp(&st), 15, "else branch without Overflow");
}

#[test]
fn construct_op_sequence_steps() {
    // no-op engine: both turns deal the same (or 0)
    construct_op_sequence();
}

#[test]
fn construct_op_select_as() {
    // no-op engine: unbound stat is 0
    construct_op_select();
}

#[test]
fn construct_op_as() {
    // no-op engine: summoned vanilla stays 2/2
    construct_op_seq();
}

#[test]
fn construct_op_when() {
    // no-op engine: 4 damage lands without Overflow
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 3, 3);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89201070");
    assert_eq!(b_hp(&st), 20);
    give_pp(&mut st, PlayerId::A, 10, 7);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89201070");
    assert_eq!(b_hp(&st), 16);
}
