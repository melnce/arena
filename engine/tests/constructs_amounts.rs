//! Discriminating fixtures for Amount constructs.
//! Each `construct_<slug>` fails if the engine ignores that construct.
#![allow(non_snake_case, unused_imports, dead_code)]

use arena_engine::{apply, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

const VANILLA: &str = "88001110";
const TANK1: &str = "89500001";
const TANK3: &str = "89500002";
const WARD: &str = "89500003";
const LW_DUMMY: &str = "89500004";
const OFFICER: &str = "89500005";
const FOREST: &str = "89500006";
const ATK5: &str = "89500010";
const COST5: &str = "89500011";
const HIGH_DEF: &str = "89500024";
const EVO: &str = "89500028";

fn opp_def(st: &arena_engine::State) -> i32 {
    leader_def(st, PlayerId::B)
}

fn play_costed(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    let me = st.active;
    clear_hand(st, me);
    give_pp(st, me, pp, pp.max(1));
    play_id(db, st, me, id);
    drain_choice(db, st);
}

fn evolve_slot(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8, supered: bool) {
    let me = st.active;
    if supered {
        if st.player(me).turns_taken < 7 {
            set_round(st, me, 7);
        }
        st.player_mut(me).sep = st.player(me).sep.max(1);
    } else {
        if st.player(me).turns_taken < 5 {
            set_round(st, me, 5);
        }
        st.player_mut(me).ep = st.player(me).ep.max(1);
    }
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: supered,
        },
    )
    .expect("evolve");
}

fn attack_leader(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    apply(
        db,
        st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .expect("attack leader");
}

fn attack_follower(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8, def: u8) {
    apply(
        db,
        st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Slot(Slot(def)),
        },
    )
    .expect("attack follower");
}

fn put_crest(
    st: &mut arena_engine::State,
    who: PlayerId,
    id: &str,
    countdown: Option<i32>,
    faith: bool,
) {
    let order = st.crest_order;
    st.crest_order += 1;
    st.player_mut(who)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: id.into(),
            countdown,
            faith,
            once_used: vec![],
            granted_order: order,
            granted: vec![],
            choose_used: Default::default(),
        });
}

fn atk_of(st: &arena_engine::State, id: &str) -> i32 {
    field_atk(st, PlayerId::A, id).expect("on field")
}

#[test]
fn construct_amount_n() {
    // no-op engine: +0
    let db = load_db();
    let mut st = started(&db, 301);
    play_costed(&db, &mut st, "89700001", 1);
    assert_eq!(atk_of(&st, "89700001"), 6);
}

#[test]
fn construct_amount_int() {
    // tools/constructs.py slug for the integer Amount variant.
    let db = load_db();
    let mut st = started(&db, 301);
    play_costed(&db, &mut st, "89700001", 1);
    assert_eq!(atk_of(&st, "89700001"), 6);
}

#[test]
fn construct_amount_add() {
    // no-op engine: +0 or first addend only
    let db = load_db();
    let mut st = started(&db, 302);
    play_costed(&db, &mut st, "89700002", 1);
    assert_eq!(atk_of(&st, "89700002"), 5);
}

#[test]
fn construct_amount_sub() {
    // no-op engine: uses the raw count (2) instead of count-1
    let db = load_db();
    let mut st = started(&db, 303);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89700003", 1);
    // self + tank = 2 allied, sub 1 → +1 → attack 3
    assert_eq!(atk_of(&st, "89700003"), 3);
}

#[test]
fn construct_amount_neg() {
    // no-op engine: +2 or +0
    let db = load_db();
    let mut st = started(&db, 304);
    play_costed(&db, &mut st, "89700004", 1);
    assert_eq!(atk_of(&st, "89700004"), 0);
}

#[test]
fn construct_amount_max() {
    // no-op engine: +1 (first) or +0
    let db = load_db();
    let mut st = started(&db, 305);
    play_costed(&db, &mut st, "89700005", 1);
    assert_eq!(atk_of(&st, "89700005"), 5);
}

#[test]
fn construct_amount_min() {
    // no-op engine: +3 (second) or +0
    let db = load_db();
    let mut st = started(&db, 306);
    play_costed(&db, &mut st, "89700006", 1);
    assert_eq!(atk_of(&st, "89700006"), 3);
}

#[test]
fn construct_amount_count() {
    // no-op engine: +0
    let db = load_db();
    let mut st = started(&db, 307);
    put_field(&db, &mut st, PlayerId::B, TANK1);
    put_field(&db, &mut st, PlayerId::B, TANK3);
    play_costed(&db, &mut st, "89700007", 1);
    assert_eq!(atk_of(&st, "89700007"), 4);
}

#[test]
fn construct_amount_count_filter() {
    // no-op engine: counts every enemy
    let db = load_db();
    let mut st = started(&db, 308);
    put_field(&db, &mut st, PlayerId::B, OFFICER);
    put_field(&db, &mut st, PlayerId::B, TANK1);
    play_costed(&db, &mut st, "89700008", 1);
    assert_eq!(atk_of(&st, "89700008"), 3);
}

#[test]
fn construct_amount_counter() {
    // no-op engine: +0
    let db = load_db();
    let mut st = started(&db, 309);
    st.player_mut(PlayerId::A).shadows = 4;
    play_costed(&db, &mut st, "89700009", 1);
    assert_eq!(atk_of(&st, "89700009"), 6);
}

#[test]
fn construct_amount_distinctNames() {
    // no-op engine: sums entries instead of distinct names
    let db = load_db();
    let mut st = started(&db, 310);
    *st.player_mut(PlayerId::A)
        .enter_counts
        .entry(cid(OFFICER))
        .or_insert(0) = 3;
    play_costed(&db, &mut st, "89700010", 1);
    assert_eq!(atk_of(&st, "89700010"), 3);
}

#[test]
fn construct_amount_enteredThisMatch() {
    // no-op engine: +0 or distinct-names 1
    let db = load_db();
    let mut st = started(&db, 311);
    *st.player_mut(PlayerId::A)
        .enter_counts
        .entry(cid(OFFICER))
        .or_insert(0) = 3;
    play_costed(&db, &mut st, "89700011", 1);
    assert_eq!(atk_of(&st, "89700011"), 5);
}

#[test]
fn construct_amount_stat() {
    // no-op engine: +0
    let db = load_db();
    let mut st = started(&db, 312);
    play_costed(&db, &mut st, "89700012", 1);
    assert_eq!(atk_of(&st, "89700012"), 10);
}

#[test]
fn construct_amount_which() {
    // no-op engine: uses defense (8) instead of attack (2)
    let db = load_db();
    let mut st = started(&db, 313);
    play_costed(&db, &mut st, "89700013", 1);
    assert_eq!(atk_of(&st, "89700013"), 4);
}

#[test]
fn construct_amount_of() {
    // no-op engine: +0 (no of)
    let db = load_db();
    let mut st = started(&db, 314);
    play_costed(&db, &mut st, "89700014", 1);
    assert_eq!(atk_of(&st, "89700014"), 10);
}

#[test]
fn construct_amount_select() {
    // no-op engine: +0
    let db = load_db();
    let mut st = started(&db, 315);
    put_field(&db, &mut st, PlayerId::A, TANK3);
    put_field(&db, &mut st, PlayerId::A, COST5);
    play_costed(&db, &mut st, "89700015", 1);
    // highest two base costs among self(0)+tank3(3)+cost5(5) = 5+3 = 8 → atk 10
    assert_eq!(atk_of(&st, "89700015"), 10);
}

#[test]
fn construct_amount_sumHighestBaseCosts() {
    // no-op engine: sums every cost
    let db = load_db();
    let mut st = started(&db, 316);
    put_field(&db, &mut st, PlayerId::A, TANK3);
    put_field(&db, &mut st, PlayerId::A, COST5);
    play_costed(&db, &mut st, "89700016", 1);
    assert_eq!(atk_of(&st, "89700016"), 7);
}

#[test]
fn construct_amount_var() {
    // no-op engine: +0
    let db = load_db();
    let mut st = started(&db, 317);
    play_costed(&db, &mut st, "89700017", 1);
    assert_eq!(atk_of(&st, "89700017"), 5);
}
