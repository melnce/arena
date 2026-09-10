//! Discriminating fixtures for Selector constructs.
#![allow(non_snake_case)]

use arena_engine::{Phase, PlayerId};

mod common;
use common::*;

fn db_st() -> (arena_engine::CardDb, arena_engine::State) {
    let db = load_db();
    let st = started(&db, 13);
    (db, st)
}

fn cast(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str) {
    give_pp(st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(db, st, PlayerId::A, id);
}

fn drain(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn put_cem(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).expect("card");
    let inst = arena_engine::CardInstance::from_card(card, st.alloc_id());
    st.player_mut(who).cemetery.push(inst);
}

fn put_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).expect("card");
    let inst = arena_engine::CardInstance::from_card(card, st.alloc_id());
    st.player_mut(who).deck.push(inst);
}

fn b_hp(st: &arena_engine::State) -> i32 {
    st.player(PlayerId::B).leader_defense
}

fn ids(st: &arena_engine::State, who: PlayerId) -> Vec<String> {
    st.player(who)
        .field
        .iter()
        .flatten()
        .map(|c| c.card.as_str())
        .collect()
}

#[test]
fn construct_selector_pick_all() {
    // no-op engine: both enemies remain
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201140");
    assert_eq!(field_count(&st, PlayerId::B), 0);
}

#[test]
fn construct_selector_pick_choose() {
    // no-op engine: no Choice, or both die
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201150");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert_eq!(field_count(&st, PlayerId::B), 1);
}

#[test]
fn construct_selector_pick_random() {
    // no-op engine: both remain or both die
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201160");
    assert_eq!(field_count(&st, PlayerId::B), 1);
}

#[test]
fn construct_selector_pick_randomDistinct() {
    // no-op engine: 0 or 1 or 3 die
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    put_field(&db, &mut st, PlayerId::B, "89100040");
    cast(&db, &mut st, "89201170");
    assert_eq!(field_count(&st, PlayerId::B), 1);
}

#[test]
fn construct_selector_pick_leftmost() {
    // no-op engine: highest-attack (right) dies
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "89100050");
    put_field(&db, &mut st, PlayerId::B, "89100040");
    cast(&db, &mut st, "89201180");
    let left = ids(&st, PlayerId::B);
    assert!(!left.iter().any(|id| id == "89100050"), "leftmost gone");
    assert!(left.iter().any(|id| id == "89100040"), "right remains");
}

#[test]
fn construct_selector_pick_highest() {
    // no-op engine: leftmost (low attack) dies
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "89100050");
    put_field(&db, &mut st, PlayerId::B, "89100040");
    cast(&db, &mut st, "89201190");
    let left = ids(&st, PlayerId::B);
    assert!(left.iter().any(|id| id == "89100050"));
    assert!(!left.iter().any(|id| id == "89100040"));
}

#[test]
fn construct_selector_pick_lowest() {
    // no-op engine: only one of the tied 2-attack bodies dies
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "89100040");
    cast(&db, &mut st, "89201200");
    let left = ids(&st, PlayerId::B);
    assert_eq!(left, vec!["89100040".to_string()], "both ties die");
}

#[test]
fn construct_selector_zone_field() {
    // no-op engine: field follower remains
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_hand(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201210");
    assert_eq!(field_count(&st, PlayerId::B), 0);
    assert!(hand_has(&st, PlayerId::B, "88001320"));
}

#[test]
fn construct_selector_zone_hand() {
    // no-op engine: enemy hand is untouched
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::B).hand.clear();
    put_hand(&db, &mut st, PlayerId::B, "88001320");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    cast(&db, &mut st, "89201220");
    assert!(!hand_has(&st, PlayerId::B, "88001320"));
    assert_eq!(field_count(&st, PlayerId::B), 1);
}

#[test]
fn construct_selector_zone_deck() {
    // no-op engine: enemy deck still has the follower
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::B).deck.clear();
    put_deck(&db, &mut st, PlayerId::B, "88001320");
    put_deck(&db, &mut st, PlayerId::B, "89100080");
    cast(&db, &mut st, "89201230");
    assert!(!st
        .player(PlayerId::B)
        .deck
        .iter()
        .any(|c| c.card.as_str() == "88001320"));
    assert!(st
        .player(PlayerId::B)
        .deck
        .iter()
        .any(|c| c.card.as_str() == "89100080"));
}

#[test]
fn construct_selector_zone_cemetery() {
    // no-op engine: cemetery is an empty pool, so nothing is added
    let (db, mut st) = db_st();
    put_cem(&db, &mut st, PlayerId::A, "88001320");
    st.player_mut(PlayerId::A).hand.clear();
    cast(&db, &mut st, "89201240");
    assert!(hand_has(&st, PlayerId::A, "88001320"));
}

#[test]
fn construct_selector_zone_leader() {
    // no-op engine: leader stays 20 (field would be the target)
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201250");
    assert_eq!(b_hp(&st), 17);
    assert_eq!(
        st.player(PlayerId::B).field[0].as_ref().unwrap().defense,
        10
    );
}

#[test]
fn construct_selector_zone_crests() {
    // no-op engine: crest stays
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89200010");
    assert!(!st.player(PlayerId::A).crests.is_empty());
    cast(&db, &mut st, "89201260");
    assert!(st.player(PlayerId::A).crests.is_empty());
}

#[test]
fn construct_selector_kind_follower() {
    // no-op engine: the amulet also dies (kind unread)
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "89100020");
    cast(&db, &mut st, "89201270");
    assert!(!field_has(&st, PlayerId::B, "88001110"));
    assert!(field_has(&st, PlayerId::B, "89100020"));
}

#[test]
fn construct_selector_kind_amulet() {
    // no-op engine: the follower also dies
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "89100020");
    cast(&db, &mut st, "89201280");
    assert!(field_has(&st, PlayerId::B, "88001110"));
    assert!(!field_has(&st, PlayerId::B, "89100020"));
}

#[test]
fn construct_selector_kind_card() {
    // no-op engine: only followers die
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "89100020");
    cast(&db, &mut st, "89201290");
    assert_eq!(field_count(&st, PlayerId::B), 0);
}

#[test]
fn construct_selector_kind_leader() {
    // no-op engine: the follower is hit instead
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201300");
    assert_eq!(b_hp(&st), 18);
    assert_eq!(
        st.player(PlayerId::B).field[0].as_ref().unwrap().defense,
        10
    );
}

#[test]
fn construct_selector_kind_character() {
    // no-op engine: leader is not in the pool
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201310");
    assert_eq!(b_hp(&st), 18);
    assert_eq!(st.player(PlayerId::B).field[0].as_ref().unwrap().defense, 8);
}

#[test]
fn construct_selector_kind_faith() {
    // no-op engine: the grant lands on every crest, or on none
    let db = load_db();
    let mut st = started_decks(&db, 13, &["89100920"], &["88001110"]);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89200010");
    play_id(&db, &mut st, PlayerId::A, "89201320");
    let faith = st
        .player(PlayerId::A)
        .crests
        .iter()
        .find(|c| c.faith)
        .expect("faith");
    let plain = st
        .player(PlayerId::A)
        .crests
        .iter()
        .find(|c| !c.faith)
        .expect("plain crest");
    assert!(!faith.granted.is_empty());
    assert!(plain.granted.is_empty());
}

#[test]
fn construct_selector_side_ally() {
    // no-op engine: the enemy is buffed too
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    cast(&db, &mut st, "89201330");
    assert_eq!(st.player(PlayerId::A).field[0].as_ref().unwrap().attack, 4);
    assert_eq!(st.player(PlayerId::B).field[0].as_ref().unwrap().attack, 2);
}

#[test]
fn construct_selector_side_enemy() {
    // no-op engine: the ally is also hit
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001320");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201340");
    assert_eq!(st.player(PlayerId::B).field[0].as_ref().unwrap().defense, 8);
    assert_eq!(
        st.player(PlayerId::A).field[0].as_ref().unwrap().defense,
        10
    );
}

#[test]
fn construct_selector_side_any() {
    // no-op engine: only one side is buffed
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    cast(&db, &mut st, "89201350");
    assert_eq!(st.player(PlayerId::A).field[0].as_ref().unwrap().attack, 4);
    assert_eq!(st.player(PlayerId::B).field[0].as_ref().unwrap().attack, 4);
}

#[test]
fn construct_selector_orderBy_attack() {
    // no-op engine: defense-highest (the 8-def body) dies
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "89100050");
    put_field(&db, &mut st, PlayerId::B, "89100040");
    cast(&db, &mut st, "89201360");
    assert!(field_has(&st, PlayerId::B, "89100050"));
    assert!(!field_has(&st, PlayerId::B, "89100040"));
}

#[test]
fn construct_selector_orderBy_defense() {
    // no-op engine: attack-highest dies
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "89100050");
    put_field(&db, &mut st, PlayerId::B, "89100040");
    cast(&db, &mut st, "89201370");
    assert!(!field_has(&st, PlayerId::B, "89100050"));
    assert!(field_has(&st, PlayerId::B, "89100040"));
}

#[test]
fn construct_selector_orderBy_cost() {
    // no-op engine: the modified 0-cost card is skipped
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::B).hand.clear();
    let hi = put_hand(&db, &mut st, PlayerId::B, "89100070");
    put_hand(&db, &mut st, PlayerId::B, "88001110");
    st.player_mut(PlayerId::B).hand[hi as usize].cost = 9;
    cast(&db, &mut st, "89201380");
    assert!(!hand_has(&st, PlayerId::B, "89100070"));
    assert!(hand_has(&st, PlayerId::B, "88001110"));
}

#[test]
fn construct_selector_orderBy_baseCost() {
    // no-op engine: current-cost highest (the modified 9) is discarded
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::B).hand.clear();
    let cheap = put_hand(&db, &mut st, PlayerId::B, "89100070");
    put_hand(&db, &mut st, PlayerId::B, "88001110");
    st.player_mut(PlayerId::B).hand[cheap as usize].cost = 0;
    cast(&db, &mut st, "89201390");
    assert!(!hand_has(&st, PlayerId::B, "89100070"));
    assert!(hand_has(&st, PlayerId::B, "88001110"));
}

#[test]
fn construct_selector_other() {
    // no-op engine: this card also dies
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::A, "88001110");
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "89201400");
    assert!(field_has(&st, PlayerId::A, "89201400"));
    assert!(!field_has(&st, PlayerId::A, "88001110"));
}

#[test]
fn construct_selector_includeLeader() {
    // no-op engine: leader stays 20
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201410");
    assert_eq!(b_hp(&st), 18);
    assert_eq!(st.player(PlayerId::B).field[0].as_ref().unwrap().defense, 8);
}

#[test]
fn construct_selector_count() {
    // no-op engine: only the leftmost dies (count defaults to 1)
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    put_field(&db, &mut st, PlayerId::B, "89100040");
    cast(&db, &mut st, "89201420");
    assert_eq!(field_count(&st, PlayerId::B), 1);
    assert!(field_has(&st, PlayerId::B, "89100040"));
}

#[test]
fn construct_selector_filter() {
    // no-op engine: both enemies die, or the non-Ward dies
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "89100010");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    cast(&db, &mut st, "89201430");
    assert!(!field_has(&st, PlayerId::B, "89100010"));
    assert!(field_has(&st, PlayerId::B, "88001110"));
}

#[test]
fn construct_selector_ref() {
    // no-op engine: destroy has an empty bind and both stay
    let (db, mut st) = db_st();
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    cast(&db, &mut st, "89201440");
    drain(&db, &mut st);
    assert_eq!(field_count(&st, PlayerId::B), 1);
}
