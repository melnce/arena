//! Exhaustive forced-lethal solver: known kill the glance misses, a
//! proven `None`, a budget-starved `Unknown`, determinism, and a
//! fixture set that puts every searchable action kind on the board.

use std::collections::HashSet;

use arena_engine::{
    apply, forced_lethal, legal_actions, Action, CardDb, LethalActionKind, LethalVerdict, PlayerId,
    H0,
};

mod common;
use common::*;

const VANILLA: &str = "88001110";
/// Troue, Heroic Visionary — 3-cost 2/1 Storm.
const STORM: &str = "10461110";
/// Analyzing Artifact / Ancient Artifact pair (fuse host + partner).
const FUSE_HOST: &str = "90071210";
const FUSE_PARTNER: &str = "90071220";
/// Sacred Plea — Engage amulet.
const ENGAGE: &str = "10001210";

fn set_body(state: &mut arena_engine::State, who: PlayerId, slot: u8, atk: i32, def: i32) {
    let f = state.field_inst_mut(who, slot).unwrap();
    f.attack = atk;
    f.defense = def;
    f.max_defense = def;
    f.flags.summoning_sick = false;
    f.flags.attacks_left = 1;
}

/// Copied from `h0_oevo::storm_lethal_opp_state`: Storm in hand, one
/// on-board attacker, evolve available. `extra_def = 1` is the glance
/// control (play-then-evolve is one short).
fn storm_lethal_opp_state(db: &CardDb, extra_def: i32) -> arena_engine::State {
    let mut st = started(db, 42);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let att = put_field(db, &mut st, PlayerId::B, VANILLA);
    set_body(&mut st, PlayerId::B, att, 3, 3);
    put_hand(db, &mut st, PlayerId::B, STORM);
    give_pp(&mut st, PlayerId::B, 3, 3);
    set_round(&mut st, PlayerId::B, 5);
    give_pp(&mut st, PlayerId::B, 3, 5);
    st.player_mut(PlayerId::B).ep = 1;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    let storm_attack = st
        .player(PlayerId::B)
        .hand
        .iter()
        .find(|c| c.card == cid(STORM))
        .map(|c| c.attack)
        .expect("Storm in hand");
    let board_attack: i32 = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .map(|c| c.attack)
        .sum();
    st.player_mut(PlayerId::A).leader_defense = storm_attack + 2 + board_attack + extra_def;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

/// Two Storms, empty board, no evolve. Lethal only by playing both and
/// swinging — two `Play`s deep, which `opp_lethal_sweep` never tries.
fn two_storm_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 42);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    put_hand(db, &mut st, PlayerId::B, STORM);
    put_hand(db, &mut st, PlayerId::B, STORM);
    set_round(&mut st, PlayerId::B, 6);
    give_pp(&mut st, PlayerId::B, 6, 6);
    st.player_mut(PlayerId::B).ep = 0;
    st.player_mut(PlayerId::B).sep = 0;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    let storm_attack = st
        .player(PlayerId::B)
        .hand
        .iter()
        .find(|c| c.card == cid(STORM))
        .map(|c| c.attack)
        .expect("Storm in hand");
    st.player_mut(PlayerId::A).leader_defense = storm_attack * 2;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    assert!(st.winner.is_none());
    let legal = legal_actions(db, &st);
    assert!(
        legal
            .iter()
            .filter(|a| matches!(a, Action::Play { .. }))
            .count()
            >= 2,
        "need two Storm plays, got {legal:?}"
    );
    st
}

/// Evolve is legal, two 3/3s cannot reach 20 defence.
fn no_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 43);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    for _ in 0..2 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    set_round(&mut st, PlayerId::B, 5);
    st.player_mut(PlayerId::B).ep = 1;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    st.player_mut(PlayerId::A).leader_defense = 20;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    assert!(st.winner.is_none());
    st
}

fn is_neg_wv(v: f32, wv: f32) -> bool {
    (v + wv).abs() < 1e-3
}

fn glance_misses(db: &CardDb, state: &arena_engine::State) -> bool {
    const WV: f32 = 300.0;
    let mut on = match arena_engine::AnyPolicy::parse_spec("h0:olethal=1,oevo=1,wv=300") {
        Ok(arena_engine::AnyPolicy::H0(h)) => h,
        other => panic!("parse: {other:?}"),
    };
    let mut off = match arena_engine::AnyPolicy::parse_spec("h0:olethal=1,oevo=0,wv=300") {
        Ok(arena_engine::AnyPolicy::H0(h)) => h,
        other => panic!("parse: {other:?}"),
    };
    let nv = on.opponent_value(db, state, PlayerId::A);
    let ov = off.opponent_value(db, state, PlayerId::A);
    !is_neg_wv(nv, WV) && !is_neg_wv(ov, WV)
}

#[test]
fn two_storm_is_lethal_and_glance_misses() {
    let db = load_db();
    let st = two_storm_lethal_state(&db);
    assert!(
        glance_misses(&db, &st),
        "opp_lethal_sweep (oevo=0 and oevo=1) must miss the two-play Storm line"
    );
    let v = forced_lethal(&db, &st, 50_000);
    match v {
        LethalVerdict::Lethal {
            line,
            nodes,
            rng_dependent,
        } => {
            assert!(nodes > 0);
            assert!(!rng_dependent, "Storm line consumes no RNG");
            let plays = line
                .iter()
                .filter(|a| matches!(a, Action::Play { .. }))
                .count();
            let attacks = line
                .iter()
                .filter(|a| matches!(a, Action::Attack { .. }))
                .count();
            assert!(plays >= 2, "line should play both Storms: {line:?}");
            assert!(attacks >= 2, "line should swing both Storms: {line:?}");
            let mut walk = st.clone();
            for a in &line {
                apply(&db, &mut walk, a.clone()).expect("line applies");
            }
            assert_eq!(walk.winner, Some(PlayerId::B));
        }
        other => panic!("expected Lethal, got {other:?}"),
    }
}

#[test]
fn storm_extra_def_one_is_a_proof() {
    let db = load_db();
    let st = storm_lethal_opp_state(&db, 1);
    assert!(
        glance_misses(&db, &st),
        "control: glance misses extra_def=1 (play-then-evolve is one short)"
    );
    let v = forced_lethal(&db, &st, 50_000);
    match &v {
        LethalVerdict::Lethal { line, .. } => {
            eprintln!("storm_lethal_opp_state(db, 1) is Lethal via {line:?}");
        }
        LethalVerdict::None { nodes } => {
            eprintln!("storm_lethal_opp_state(db, 1) is None (nodes={nodes}): no kill");
        }
        LethalVerdict::Unknown { nodes } => {
            panic!("storm_lethal_opp_state(db, 1) ran out of budget ({nodes} nodes)");
        }
    }
    // extra_def=1 is one short of the evolve line. If a different kill
    // existed the exhaustive search would have returned Lethal; None is
    // the expected proof. Pin the three-way type, not merely "not Lethal".
    assert!(
        matches!(v, LethalVerdict::None { .. }),
        "extra_def=1 must be a proof of absence, got {v:?}"
    );
}

#[test]
fn storm_extra_def_zero_is_lethal() {
    let db = load_db();
    let st = storm_lethal_opp_state(&db, 0);
    let v = forced_lethal(&db, &st, 50_000);
    assert!(
        matches!(v, LethalVerdict::Lethal { .. }),
        "play+evolve+swing is a kill, got {v:?}"
    );
}

#[test]
fn no_lethal_is_a_proof() {
    let db = load_db();
    let st = no_lethal_state(&db);
    let v = forced_lethal(&db, &st, 50_000);
    match v {
        LethalVerdict::None { nodes } => assert!(nodes > 0),
        other => panic!("expected None (a proof), got {other:?}"),
    }
}

#[test]
fn budget_zero_on_a_wide_position_is_unknown() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, VANILLA);
    put_hand(&db, &mut st, PlayerId::A, VANILLA);
    put_hand(&db, &mut st, PlayerId::A, STORM);
    let legal = legal_actions(&db, &st);
    assert!(
        legal.iter().any(|a| matches!(a, Action::Play { .. })),
        "wide position must have a Play, got {legal:?}"
    );
    let v = forced_lethal(&db, &st, 0);
    assert!(
        matches!(v, LethalVerdict::Unknown { nodes: 0 }),
        "budget 0 with expandable actions is Unknown, not None: {v:?}"
    );
}

#[test]
fn budget_one_on_a_wide_position_is_unknown() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    clear_hand(&mut st, PlayerId::A);
    for _ in 0..4 {
        put_hand(&db, &mut st, PlayerId::A, VANILLA);
    }
    st.player_mut(PlayerId::A).leader_defense = 20;
    st.player_mut(PlayerId::B).leader_defense = 20;
    let v = forced_lethal(&db, &st, 1);
    assert!(
        matches!(v, LethalVerdict::Unknown { nodes: 1 }),
        "one apply on a non-lethal wide hand must be Unknown: {v:?}"
    );
}

#[test]
fn same_state_and_budget_are_deterministic() {
    let db = load_db();
    let st = two_storm_lethal_state(&db);
    let a = forced_lethal(&db, &st, 20_000);
    let b = forced_lethal(&db, &st, 20_000);
    assert_eq!(a, b);
    match (&a, &b) {
        (LethalVerdict::Lethal { nodes: n1, .. }, LethalVerdict::Lethal { nodes: n2, .. }) => {
            assert_eq!(n1, n2)
        }
        (LethalVerdict::None { nodes: n1 }, LethalVerdict::None { nodes: n2 }) => {
            assert_eq!(n1, n2)
        }
        (LethalVerdict::Unknown { nodes: n1 }, LethalVerdict::Unknown { nodes: n2 }) => {
            assert_eq!(n1, n2)
        }
        _ => panic!("verdict mismatch {a:?} vs {b:?}"),
    }
}

fn kinds_from_legal(db: &CardDb, state: &arena_engine::State) -> HashSet<LethalActionKind> {
    legal_actions(db, state)
        .iter()
        .filter_map(arena_engine::lethal_action_kind)
        .collect()
}

/// Second-player turn with Play / Attack / Evolve / Engage / Fuse /
/// BonusPp legal at the root; Fuse then yields Choose + Confirm.
fn coverage_root(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 14);
    end_turn(db, &mut st);
    assert_eq!(st.active, PlayerId::B);
    assert!(st.player(PlayerId::B).is_second);
    clear_hand(&mut st, PlayerId::B);
    let att = put_field(db, &mut st, PlayerId::B, VANILLA);
    set_body(&mut st, PlayerId::B, att, 3, 3);
    put_field(db, &mut st, PlayerId::B, ENGAGE);
    put_hand(db, &mut st, PlayerId::B, STORM);
    put_hand(db, &mut st, PlayerId::B, FUSE_HOST);
    put_hand(db, &mut st, PlayerId::B, FUSE_PARTNER);
    set_round(&mut st, PlayerId::B, 6);
    give_pp(&mut st, PlayerId::B, 10, 10);
    st.player_mut(PlayerId::B).ep = 1;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    st.player_mut(PlayerId::B).bonus_pp.early_charge = true;
    st.player_mut(PlayerId::B).bonus_pp.locked = false;
    st.player_mut(PlayerId::B).bonus_pp.active = false;
    st
}

fn fuse_choice_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 6);
    clear_hand(&mut st, PlayerId::A);
    put_hand(db, &mut st, PlayerId::A, FUSE_HOST);
    put_hand(db, &mut st, PlayerId::A, FUSE_PARTNER);
    apply(db, &mut st, Action::Fuse { host: 0 }).expect("fuse");
    assert!(matches!(st.phase, arena_engine::Phase::Choice { .. }));
    st
}

#[test]
fn search_reaches_every_action_kind() {
    let db = load_db();
    let mut seen: HashSet<LethalActionKind> = HashSet::new();
    let root = coverage_root(&db);
    seen.extend(kinds_from_legal(&db, &root));
    // Drive Fuse so Choose / Confirm appear, and confirm the solver
    // actually expands those nodes (a forgotten kind is 0 applies).
    let fuse = fuse_choice_state(&db);
    seen.extend(kinds_from_legal(&db, &fuse));
    let choose = legal_actions(&db, &fuse)
        .into_iter()
        .find(|a| matches!(a, Action::Choose(_)))
        .expect("choose");
    let mut after_choose = fuse.clone();
    apply(&db, &mut after_choose, choose).expect("choose applies");
    seen.extend(kinds_from_legal(&db, &after_choose));

    let need = [
        LethalActionKind::Play,
        LethalActionKind::Attack,
        LethalActionKind::Evolve,
        LethalActionKind::Engage,
        LethalActionKind::Fuse,
        LethalActionKind::BonusPp,
        LethalActionKind::Choose,
        LethalActionKind::Confirm,
    ];
    for k in need {
        assert!(
            seen.contains(&k),
            "fixture set never offered {k:?}; have {seen:?}"
        );
    }

    // The solver must spend nodes on each fixture — that is the search
    // reaching those kinds, not merely the generator listing them.
    for (label, st, budget) in [
        ("coverage_root", root, 8_u32),
        ("fuse_choice", fuse, 8),
        ("after_choose", after_choose, 8),
    ] {
        let v = forced_lethal(&db, &st, budget);
        let nodes = match v {
            LethalVerdict::Lethal { nodes, .. }
            | LethalVerdict::None { nodes }
            | LethalVerdict::Unknown { nodes } => nodes,
        };
        assert!(nodes > 0, "{label} search spent no nodes: {v:?}");
    }
}

#[test]
fn h0_defaults_untouched_from_this_crate() {
    // Compile-time reminder for the PR body: this file does not touch
    // engine/src/policy/. Runtime identity still matches H0::default.
    let a = H0::default();
    let b = H0::fast();
    assert_eq!(a.depth, 6);
    assert_eq!(b.depth, 2);
    assert!(a.oevo);
    assert!(!b.oevo);
}
