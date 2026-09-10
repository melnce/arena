//! Discriminating fixtures for Filter constructs.
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

fn two_enemy(db: &arena_engine::CardDb, st: &mut arena_engine::State, hit: &str, miss: &str) {
    put_field(db, st, PlayerId::B, hit);
    put_field(db, st, PlayerId::B, miss);
}

fn defs(st: &arena_engine::State, a: &str, b: &str) -> (i32, i32) {
    (
        field_def(st, PlayerId::B, a).unwrap(),
        field_def(st, PlayerId::B, b).unwrap(),
    )
}

#[test]
fn construct_filter_all() {
    // no-op engine: damages the officer without Ward too
    let db = load_db();
    let mut st = started(&db, 201);
    put_field(&db, &mut st, PlayerId::B, WARD);
    if let Some(f) = st.field_inst_mut(PlayerId::B, 0) {
        f.tribes = vec![arena_engine::card::Tribe::Officer];
    }
    put_field(&db, &mut st, PlayerId::B, OFFICER);
    play_costed(&db, &mut st, "89600001", 1);
    assert_eq!(field_def(&st, PlayerId::B, WARD), Some(9));
    assert_eq!(field_def(&st, PlayerId::B, OFFICER), Some(10));
}

#[test]
fn construct_filter_any() {
    // no-op engine: damages a cost-1 non-officer
    let db = load_db();
    let mut st = started(&db, 202);
    two_enemy(&db, &mut st, OFFICER, TANK1);
    play_costed(&db, &mut st, "89600002", 1);
    let (h, m) = defs(&st, OFFICER, TANK1);
    assert_eq!((h, m), (9, 10));
}

#[test]
fn construct_filter_not() {
    // no-op engine: damages the officer too
    let db = load_db();
    let mut st = started(&db, 203);
    two_enemy(&db, &mut st, TANK1, OFFICER);
    play_costed(&db, &mut st, "89600003", 1);
    let (h, m) = defs(&st, TANK1, OFFICER);
    assert_eq!((h, m), (9, 10));
}

#[test]
fn construct_filter_attackGte() {
    // no-op engine: damages the 2-attack body
    let db = load_db();
    let mut st = started(&db, 204);
    two_enemy(&db, &mut st, ATK5, TANK1);
    play_costed(&db, &mut st, "89600004", 1);
    assert_eq!(defs(&st, ATK5, TANK1), (9, 10));
}

#[test]
fn construct_filter_attackLte() {
    // no-op engine: damages the 5-attack body
    let db = load_db();
    let mut st = started(&db, 205);
    two_enemy(&db, &mut st, HIGH_DEF, ATK5);
    play_costed(&db, &mut st, "89600005", 1);
    assert_eq!(defs(&st, HIGH_DEF, ATK5), (11, 10));
}

#[test]
fn construct_filter_defenseGte() {
    // no-op engine: damages the 10-defense body
    let db = load_db();
    let mut st = started(&db, 206);
    two_enemy(&db, &mut st, HIGH_DEF, TANK1);
    play_costed(&db, &mut st, "89600006", 1);
    assert_eq!(defs(&st, HIGH_DEF, TANK1), (11, 10));
}

#[test]
fn construct_filter_defenseLte() {
    // no-op engine: damages the tank
    let db = load_db();
    let mut st = started(&db, 207);
    put_field(&db, &mut st, PlayerId::B, VANILLA);
    put_field(&db, &mut st, PlayerId::B, TANK1);
    play_costed(&db, &mut st, "89600007", 1);
    assert_eq!(field_def(&st, PlayerId::B, VANILLA), Some(1));
    assert_eq!(field_def(&st, PlayerId::B, TANK1), Some(10));
}

#[test]
fn construct_filter_baseCostEq() {
    // no-op engine: damages the 1-cost
    let db = load_db();
    let mut st = started(&db, 208);
    two_enemy(&db, &mut st, TANK3, TANK1);
    play_costed(&db, &mut st, "89600008", 1);
    assert_eq!(defs(&st, TANK3, TANK1), (9, 10));
}

#[test]
fn construct_filter_baseCostGte() {
    // no-op engine: damages the 1-cost
    let db = load_db();
    let mut st = started(&db, 209);
    two_enemy(&db, &mut st, COST5, TANK1);
    play_costed(&db, &mut st, "89600009", 1);
    assert_eq!(defs(&st, COST5, TANK1), (9, 10));
}

#[test]
fn construct_filter_baseCostLte() {
    // no-op engine: damages the 3-cost
    let db = load_db();
    let mut st = started(&db, 210);
    two_enemy(&db, &mut st, TANK1, TANK3);
    play_costed(&db, &mut st, "89600010", 1);
    assert_eq!(defs(&st, TANK1, TANK3), (9, 10));
}

#[test]
fn construct_filter_baseCostIn() {
    // no-op engine: damages the 1-cost
    let db = load_db();
    let mut st = started(&db, 211);
    two_enemy(&db, &mut st, TANK3, TANK1);
    play_costed(&db, &mut st, "89600011", 1);
    assert_eq!(defs(&st, TANK3, TANK1), (9, 10));
}

#[test]
fn construct_filter_costEq() {
    // no-op engine: damages the 1-cost
    let db = load_db();
    let mut st = started(&db, 212);
    two_enemy(&db, &mut st, TANK3, TANK1);
    play_costed(&db, &mut st, "89600012", 1);
    assert_eq!(defs(&st, TANK3, TANK1), (9, 10));
}

#[test]
fn construct_filter_costGte() {
    // no-op engine: damages the 1-cost
    let db = load_db();
    let mut st = started(&db, 213);
    two_enemy(&db, &mut st, COST5, TANK1);
    play_costed(&db, &mut st, "89600013", 1);
    assert_eq!(defs(&st, COST5, TANK1), (9, 10));
}

#[test]
fn construct_filter_costLte() {
    // no-op engine: damages the 3-cost
    let db = load_db();
    let mut st = started(&db, 214);
    two_enemy(&db, &mut st, TANK1, TANK3);
    play_costed(&db, &mut st, "89600014", 1);
    assert_eq!(defs(&st, TANK1, TANK3), (9, 10));
}

#[test]
fn construct_filter_costIn() {
    // no-op engine: damages the 1-cost
    let db = load_db();
    let mut st = started(&db, 215);
    two_enemy(&db, &mut st, TANK3, TANK1);
    play_costed(&db, &mut st, "89600015", 1);
    assert_eq!(defs(&st, TANK3, TANK1), (9, 10));
}

#[test]
fn construct_filter_card() {
    // no-op engine: damages every enemy
    let db = load_db();
    let mut st = started(&db, 216);
    two_enemy(&db, &mut st, OFFICER, TANK1);
    play_costed(&db, &mut st, "89600016", 1);
    assert_eq!(defs(&st, OFFICER, TANK1), (9, 10));
}

#[test]
fn construct_filter_cards() {
    // no-op engine: damages the 1-cost tank
    let db = load_db();
    let mut st = started(&db, 217);
    two_enemy(&db, &mut st, OFFICER, TANK1);
    play_costed(&db, &mut st, "89600017", 1);
    assert_eq!(defs(&st, OFFICER, TANK1), (9, 10));
}

#[test]
fn construct_filter_notCard() {
    // no-op engine: damages Officer Tank
    let db = load_db();
    let mut st = started(&db, 218);
    two_enemy(&db, &mut st, TANK1, OFFICER);
    play_costed(&db, &mut st, "89600018", 1);
    assert_eq!(defs(&st, TANK1, OFFICER), (9, 10));
}

#[test]
fn construct_filter_class() {
    // no-op engine: damages the neutral
    let db = load_db();
    let mut st = started(&db, 219);
    two_enemy(&db, &mut st, FOREST, TANK1);
    play_costed(&db, &mut st, "89600019", 1);
    assert_eq!(defs(&st, FOREST, TANK1), (9, 10));
}

#[test]
fn construct_filter_tribe() {
    // no-op engine: damages the non-officer
    let db = load_db();
    let mut st = started(&db, 220);
    two_enemy(&db, &mut st, OFFICER, TANK1);
    play_costed(&db, &mut st, "89600020", 1);
    assert_eq!(defs(&st, OFFICER, TANK1), (9, 10));
}

#[test]
fn construct_filter_kind() {
    // no-op engine: damages the follower
    let db = load_db();
    let mut st = started(&db, 221);
    put_field(&db, &mut st, PlayerId::B, "89500009");
    put_field(&db, &mut st, PlayerId::B, TANK1);
    play_costed(&db, &mut st, "89600021", 1);
    assert!(field_has(&st, PlayerId::B, TANK1));
}

#[test]
fn construct_filter_damaged() {
    // no-op engine: damages the undamaged body
    let db = load_db();
    let mut st = started(&db, 222);
    two_enemy(&db, &mut st, TANK1, TANK3);
    if let Some(f) = st.field_inst_mut(PlayerId::B, 0) {
        f.defense = 8;
    }
    play_costed(&db, &mut st, "89600022", 1);
    assert_eq!(defs(&st, TANK1, TANK3), (7, 10));
}

#[test]
fn construct_filter_destroyedThisMatch() {
    // no-op engine: counts the live field (empty after destroy) → 0
    let db = load_db();
    let mut st = started(&db, 223);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89500031", 1);
    play_costed(&db, &mut st, "89600023", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 223);
    play_costed(&db, &mut st, "89600023", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_filter_didNotAttackThisTurn() {
    // no-op engine: damages the attacker too
    let db = load_db();
    let mut st = started(&db, 224);
    let s = put_field(&db, &mut st, PlayerId::B, TANK1);
    put_field(&db, &mut st, PlayerId::B, TANK3);
    if let Some(f) = st.field_inst_mut(PlayerId::B, s) {
        f.flags.attacked_this_turn = true;
    }
    play_costed(&db, &mut st, "89600024", 1);
    assert_eq!(defs(&st, TANK1, TANK3), (10, 9));
}

#[test]
fn construct_filter_enhanced() {
    // no-op engine: damages the unenhanced body
    let db = load_db();
    let mut st = started(&db, 225);
    two_enemy(&db, &mut st, TANK1, TANK3);
    if let Some(f) = st.field_inst_mut(PlayerId::B, 0) {
        f.flags.enhanced = true;
    }
    play_costed(&db, &mut st, "89600025", 1);
    assert_eq!(defs(&st, TANK1, TANK3), (9, 10));
}

#[test]
fn construct_filter_evolved() {
    // no-op engine: damages the unevolved body
    let db = load_db();
    let mut st = started(&db, 226);
    two_enemy(&db, &mut st, TANK1, TANK3);
    if let Some(f) = st.field_inst_mut(PlayerId::B, 0) {
        f.evolved = true;
    }
    play_costed(&db, &mut st, "89600026", 1);
    assert_eq!(defs(&st, TANK1, TANK3), (9, 10));
}

#[test]
fn construct_filter_unevolved() {
    // no-op engine: damages the evolved body
    let db = load_db();
    let mut st = started(&db, 227);
    two_enemy(&db, &mut st, TANK1, TANK3);
    if let Some(f) = st.field_inst_mut(PlayerId::B, 1) {
        f.evolved = true;
    }
    play_costed(&db, &mut st, "89600027", 1);
    assert_eq!(defs(&st, TANK1, TANK3), (9, 10));
}

#[test]
fn construct_filter_superEvolved() {
    // no-op engine: damages the ordinary evolved body
    let db = load_db();
    let mut st = started(&db, 228);
    two_enemy(&db, &mut st, TANK1, TANK3);
    if let Some(f) = st.field_inst_mut(PlayerId::B, 0) {
        f.super_evolved = true;
        f.evolved = true;
    }
    play_costed(&db, &mut st, "89600028", 1);
    assert_eq!(defs(&st, TANK1, TANK3), (9, 10));
}

#[test]
fn construct_filter_hasLastWords() {
    // no-op engine: damages the dummy-less tank
    let db = load_db();
    let mut st = started(&db, 229);
    two_enemy(&db, &mut st, LW_DUMMY, TANK1);
    play_costed(&db, &mut st, "89600029", 1);
    assert_eq!(defs(&st, LW_DUMMY, TANK1), (9, 10));
}

#[test]
fn construct_filter_hasSpellboost() {
    // no-op engine: discards the vanilla too
    let db = load_db();
    let mut st = started(&db, 230);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "89500008");
    put_hand(&db, &mut st, me, TANK1);
    play_keeping_hand(&db, &mut st, me, "89600030", 1);
    assert!(!hand_has(&st, me, "89500008"));
    assert!(hand_has(&st, me, TANK1));
}

#[test]
fn construct_filter_hasTrait() {
    // no-op engine: damages the traitless tank
    let db = load_db();
    let mut st = started(&db, 231);
    two_enemy(&db, &mut st, WARD, TANK1);
    play_costed(&db, &mut st, "89600031", 1);
    assert_eq!(defs(&st, WARD, TANK1), (9, 10));
}

#[test]
fn construct_filter_notBound() {
    // no-op engine: damages the bound officer too
    let db = load_db();
    let mut st = started(&db, 232);
    put_field(&db, &mut st, PlayerId::A, OFFICER);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89600032", 1);
    assert_eq!(field_def(&st, PlayerId::A, OFFICER), Some(10));
    assert_eq!(field_def(&st, PlayerId::A, TANK1), Some(9));
}

#[test]
fn construct_filter_sameCostGroup() {
    // no-op engine: fires on a different-cost play
    let db = load_db();
    let mut st = started(&db, 233);
    put_field(&db, &mut st, PlayerId::A, "89600033");
    play_costed(&db, &mut st, TANK3, 3);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 233);
    put_field(&db, &mut st, PlayerId::A, "89600033");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 20);
}
