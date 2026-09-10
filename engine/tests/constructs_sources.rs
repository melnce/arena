//! Discriminating fixtures for CardSource constructs.
#![allow(non_snake_case)]

use arena_engine::{Phase, PlayerId};

mod common;
use common::*;

fn db_st() -> (arena_engine::CardDb, arena_engine::State) {
    let db = load_db();
    let st = started(&db, 11);
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

fn put_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).expect("card");
    let inst = arena_engine::CardInstance::from_card(card, st.alloc_id());
    st.player_mut(who).deck.push(inst);
}

#[test]
fn construct_source_named() {
    // no-op engine: field stays empty
    let (db, mut st) = db_st();
    cast(&db, &mut st, "89201080");
    assert!(field_has(&st, PlayerId::A, "88001110"));
}

#[test]
fn construct_source_copyOf() {
    // no-op engine: copy keeps the original's +2/+2
    let (db, mut st) = db_st();
    let slot = put_field(&db, &mut st, PlayerId::B, "88001110");
    if let Some(f) = st.field_inst_mut(PlayerId::B, slot) {
        f.attack = 9;
        f.defense = 9;
    }
    cast(&db, &mut st, "89201090");
    drain(&db, &mut st);
    let copy = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001110")
        .expect("copy");
    assert_eq!(
        (copy.attack, copy.defense),
        (2, 2),
        "non-exact copy is a fresh print"
    );
}

#[test]
fn construct_source_copyOf_exact() {
    // no-op engine: copy is a fresh 2/2
    let (db, mut st) = db_st();
    let slot = put_field(&db, &mut st, PlayerId::B, "88001110");
    if let Some(f) = st.field_inst_mut(PlayerId::B, slot) {
        f.attack = 9;
        f.defense = 9;
    }
    cast(&db, &mut st, "89201100");
    drain(&db, &mut st);
    let copy = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001110")
        .expect("copy");
    assert_eq!((copy.attack, copy.defense), (9, 9));
}

#[test]
fn construct_source_from() {
    // no-op engine: the hand follower stays in hand
    let (db, mut st) = db_st();
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    put_hand(&db, &mut st, PlayerId::A, "88001320");
    play_id(&db, &mut st, PlayerId::A, "89201110");
    drain(&db, &mut st);
    assert!(field_has(&st, PlayerId::A, "88001320"));
    assert!(!hand_has(&st, PlayerId::A, "88001320"));
}

#[test]
fn construct_source_randomFrom() {
    // no-op engine: field stays empty
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).deck.clear();
    put_deck(&db, &mut st, PlayerId::A, "88001320");
    put_deck(&db, &mut st, PlayerId::A, "89100080");
    cast(&db, &mut st, "89201120");
    assert!(field_has(&st, PlayerId::A, "88001320"));
}

#[test]
fn construct_source_randomFrom_countGt1() {
    // no-op engine: two copies of the same name, or a single summon
    let (db, mut st) = db_st();
    st.player_mut(PlayerId::A).deck.clear();
    put_deck(&db, &mut st, PlayerId::A, "88001110");
    put_deck(&db, &mut st, PlayerId::A, "88001110");
    put_deck(&db, &mut st, PlayerId::A, "88001320");
    cast(&db, &mut st, "89201130");
    let names: std::collections::BTreeSet<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .map(|c| c.card.as_str())
        .collect();
    assert_eq!(names.len(), 2, "count>1 picks differently named");
}
