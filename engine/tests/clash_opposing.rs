//! `pick: "opposing"` is the other combatant relative to the trigger source.
//! Defender Clash used to resolve against itself because `combat_opposing`
//! always stored the attack target.

use arena_engine::{apply, Action, AttackTarget, PlayerId, Slot};

mod common;
use common::*;

const EUSTACE: &str = "10472110";
const CERES: &str = "10854120";
const ARMES: &str = "10654110";
const HIGH_DEF: &str = "89500024";
const OKITA: &str = "10823110";
const ILLAMRITA: &str = "10704110";

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

#[test]
fn defender_only_clash_hits_attacker() {
    let db = load_db();
    let mut st = started(&db, 1);
    let att = put_field(&db, &mut st, PlayerId::A, HIGH_DEF);
    let def = put_field(&db, &mut st, PlayerId::B, CERES);
    attack_follower(&db, &mut st, att, def);
    assert_eq!(
        field_def(&st, PlayerId::A, HIGH_DEF),
        Some(7),
        "High Def 12 − 4 clash − 1 combat"
    );
    assert_eq!(
        field_def(&st, PlayerId::B, CERES),
        Some(3),
        "Ceres 4 − 1 combat"
    );
}

#[test]
fn attacker_only_clash_hits_defender() {
    let db = load_db();
    let mut st = started(&db, 2);
    let att = put_field(&db, &mut st, PlayerId::A, EUSTACE);
    let def = put_field(&db, &mut st, PlayerId::B, HIGH_DEF);
    attack_follower(&db, &mut st, att, def);
    assert_eq!(
        field_def(&st, PlayerId::B, HIGH_DEF),
        Some(4),
        "High Def 12 − 3 clash − 5 combat"
    );
    assert_eq!(field_def(&st, PlayerId::A, EUSTACE), Some(3));
}

#[test]
fn evolved_attacker_only_clash_hits_defender() {
    let db = load_db();
    let mut st = started(&db, 3);
    let att = put_field(&db, &mut st, PlayerId::A, EUSTACE);
    evolve_slot(&db, &mut st, att, false);
    assert_eq!(field_atk(&st, PlayerId::A, EUSTACE), Some(7));
    assert_eq!(field_def(&st, PlayerId::A, EUSTACE), Some(6));
    let def = put_field(&db, &mut st, PlayerId::B, HIGH_DEF);
    attack_follower(&db, &mut st, att, def);
    assert_eq!(
        field_def(&st, PlayerId::B, HIGH_DEF),
        Some(2),
        "High Def 12 − 3 clash − 7 combat"
    );
}

#[test]
fn both_clash_attacker_first_then_queued_defender() {
    let db = load_db();
    let mut st = started(&db, 4);
    let att = put_field(&db, &mut st, PlayerId::A, EUSTACE);
    let def = put_field(&db, &mut st, PlayerId::B, CERES);
    if let Some(f) = st.field_inst_mut(PlayerId::B, def) {
        f.defense = 3;
        f.max_defense = 3;
    }
    let eustace_atk = field_atk(&st, PlayerId::A, EUSTACE).unwrap();
    let ceres_atk = field_atk(&st, PlayerId::B, CERES).unwrap();
    assert_eq!(eustace_atk, 5);
    assert_eq!(ceres_atk, 1, "combat damage cannot kill Eustace (4 def)");
    attack_follower(&db, &mut st, att, def);
    assert!(
        !field_has(&st, PlayerId::B, CERES),
        "Eustace clash 3 destroys Ceres at 3 def"
    );
    assert!(
        !field_has(&st, PlayerId::A, EUSTACE),
        "queued Ceres clash 4 destroys Eustace; combat never reached him"
    );
}

#[test]
fn clash_never_on_a_leader() {
    let db = load_db();
    let mut st = started(&db, 5);
    let att = put_field(&db, &mut st, PlayerId::A, EUSTACE);
    put_field(&db, &mut st, PlayerId::B, HIGH_DEF);
    let a_hp = leader_def(&st, PlayerId::A);
    let b_hp = leader_def(&st, PlayerId::B);
    attack_leader(&db, &mut st, att);
    assert_eq!(leader_def(&st, PlayerId::B), b_hp - 5);
    assert_eq!(leader_def(&st, PlayerId::A), a_hp);
    assert_eq!(field_def(&st, PlayerId::A, EUSTACE), Some(4));
    assert_eq!(
        field_def(&st, PlayerId::B, HIGH_DEF),
        Some(12),
        "Clash does not fire when attacking a leader"
    );
}

#[test]
fn okita_follower_strike_opposing_still_hits_defender() {
    let db = load_db();
    let mut st = started(&db, 6);
    let att = put_field(&db, &mut st, PlayerId::A, OKITA);
    let def = put_field(&db, &mut st, PlayerId::B, HIGH_DEF);
    attack_follower(&db, &mut st, att, def);
    assert_eq!(
        field_def(&st, PlayerId::B, HIGH_DEF),
        Some(7),
        "High Def 12 − 3 strike − 2 combat"
    );
    assert!(
        !field_has(&st, PlayerId::A, OKITA),
        "Okita 1 def dies to High Def's 1 combat"
    );
}

#[test]
fn illamrita_follower_strike_opposing_still_hits_defender() {
    let db = load_db();
    let mut st = started(&db, 7);
    let att = put_field(&db, &mut st, PlayerId::A, ILLAMRITA);
    let def = put_field(&db, &mut st, PlayerId::B, HIGH_DEF);
    attack_follower(&db, &mut st, att, def);
    assert_eq!(
        field_def(&st, PlayerId::B, HIGH_DEF),
        Some(11),
        "High Def 12 − 1 combat; strike grants, does not damage"
    );
    assert_eq!(
        field_def(&st, PlayerId::A, ILLAMRITA),
        Some(4),
        "Barrier absorbs High Def's 1 combat"
    );
    let high = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid(HIGH_DEF))
        .expect("High Def");
    assert_eq!(high.traits.cant_attack_followers, Some(true));
    assert_eq!(high.traits.cant_attack_leader, Some(true));
    assert!(
        !high.granted.is_empty(),
        "end-of-turn banish granted on the defender"
    );
}

#[test]
fn super_evolve_knockback_when_clash_destroys_target() {
    let db = load_db();
    let mut st = started(&db, 8);
    let att = put_field(&db, &mut st, PlayerId::A, ARMES);
    if let Some(f) = st.field_inst_mut(PlayerId::A, att) {
        f.super_evolved = true;
        f.evolved = true;
    }
    let def = put_field(&db, &mut st, PlayerId::B, HIGH_DEF);
    let b_hp = leader_def(&st, PlayerId::B);
    attack_follower(&db, &mut st, att, def);
    assert!(
        !field_has(&st, PlayerId::B, HIGH_DEF),
        "Armes Clash destroys the opposing follower"
    );
    assert!(field_has(&st, PlayerId::A, ARMES));
    assert_eq!(
        leader_def(&st, PlayerId::B),
        b_hp - 1,
        "SE knockback still fires when Clash destroys the combat target"
    );
}
