//! H0 Bonus PP holding keys: bpp1, bpp2, bppv.

use arena_engine::{apply, legal_actions, policy_rng, AnyPolicy, CardDb, PlayerId, Policy};

mod common;
use common::*;

fn h0_pick_with(db: &CardDb, spec: &str, state: &arena_engine::State) -> arena_engine::Action {
    let legal = legal_actions(db, state);
    assert!(!legal.is_empty());
    let mut h0 = match AnyPolicy::parse_spec(spec).unwrap() {
        AnyPolicy::H0(h) => h,
        other => panic!("expected H0, got {other:?}"),
    };
    let mut rng = policy_rng(1);
    let i = h0.choose(db, state, &legal, &mut rng);
    legal[i.min(legal.len().saturating_sub(1))].clone()
}

fn second_turn2_state(db: &CardDb, seed: u64) -> arena_engine::State {
    let mut st = started(db, seed);
    st.active = PlayerId::B;
    st.turn = 2;
    st.player_mut(PlayerId::B).is_second = true;
    st.player_mut(PlayerId::B).turns_taken = 2;
    st.player_mut(PlayerId::B).bonus_pp.early_charge = true;
    st.player_mut(PlayerId::B).bonus_pp.late_charge = true;
    st.player_mut(PlayerId::B).bonus_pp.active = false;
    st.player_mut(PlayerId::B).bonus_pp.locked = false;
    give_pp(&mut st, PlayerId::B, 2, 2);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    st.player_mut(PlayerId::A).field = Default::default();
    st.player_mut(PlayerId::B).field = Default::default();
    put_hand(db, &mut st, PlayerId::B, "10002110");
    st
}

#[test]
fn bpp1_delays_early_charge_activation() {
    let db = load_db();
    let st = second_turn2_state(&db, 30);
    assert!(
        matches!(h0_pick_with(&db, "h0", &st), arena_engine::Action::BonusPp),
        "default activates early charge on turn 2"
    );
    let st2 = second_turn2_state(&db, 31);
    let a = h0_pick_with(&db, "h0:bpp1=4", &st2);
    assert!(
        !matches!(a, arena_engine::Action::BonusPp),
        "bpp1=4 must not activate early charge before turn 4, got {a:?}"
    );
}

#[test]
fn bpp2_delays_late_charge_activation() {
    let db = load_db();
    let mut st = started(&db, 32);
    let me = PlayerId::B;
    while st.player(me).turns_taken < 6 || st.active != me {
        end_turn(&db, &mut st);
    }
    assert_eq!(st.player(me).turns_taken, 6);
    give_pp(&mut st, me, 6, 6);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, "10002120");
    assert!(
        matches!(h0_pick_with(&db, "h0", &st), arena_engine::Action::BonusPp),
        "default activates late charge on turn 6 when a 7-cost needs the orb"
    );
    let mut st2 = started(&db, 33);
    while st2.player(me).turns_taken < 8 || st2.active != me {
        end_turn(&db, &mut st2);
    }
    give_pp(&mut st2, me, 6, 8);
    clear_hand(&mut st2, me);
    put_hand(&db, &mut st2, me, "10002120");
    let a = h0_pick_with(&db, "h0:bpp2=9", &st2);
    assert!(
        !matches!(a, arena_engine::Action::BonusPp),
        "bpp2=9 must not activate late charge before turn 9, got {a:?}"
    );
}

#[test]
fn bppv_adds_leaf_value_for_held_charges() {
    let db = load_db();
    let mut st = started(&db, 34);
    end_turn(&db, &mut st);
    let me = PlayerId::B;
    let bare = match AnyPolicy::parse_spec("h0:value=v0").unwrap() {
        AnyPolicy::H0(h) => h,
        _ => panic!("H0"),
    };
    let prized = match AnyPolicy::parse_spec("h0:value=v0,bppv=10").unwrap() {
        AnyPolicy::H0(h) => h,
        _ => panic!("H0"),
    };
    let v0 = bare.evaluate(&db, &st, me);
    let v1 = prized.evaluate(&db, &st, me);
    assert!(v1 > v0, "bppv should reward held charges");
    let mut spent = st.clone();
    spent.player_mut(me).bonus_pp.locked = true;
    let v_spent = prized.evaluate(&db, &spent, me);
    assert!(
        v1 - v_spent > 5.0,
        "spending the early charge should cost bppv value"
    );
}

#[test]
fn bppv_penalizes_opponent_charges_from_first_player_view() {
    let db = load_db();
    let mut st = started(&db, 36);
    end_turn(&db, &mut st);
    let first = PlayerId::A;
    let bare = match AnyPolicy::parse_spec("h0:value=v0").unwrap() {
        AnyPolicy::H0(h) => h,
        _ => panic!("H0"),
    };
    let prized = match AnyPolicy::parse_spec("h0:value=v0,bppv=10").unwrap() {
        AnyPolicy::H0(h) => h,
        _ => panic!("H0"),
    };
    let v0 = bare.evaluate(&db, &st, first);
    let v1 = prized.evaluate(&db, &st, first);
    assert!(v1 < v0, "bppv should penalize the opponent's held charges");
    assert!((v0 - v1 - 20.0).abs() < 1e-3, "two usable charges at bppv=10");
}

#[test]
fn bppv_changes_turn_1_decision() {
    let db = load_db();
    let mut st = started(&db, 35);
    end_turn(&db, &mut st);
    let me = PlayerId::B;
    give_pp(&mut st, me, 1, 1);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, "10002110");
    assert!(
        matches!(
            h0_pick_with(&db, "h0:value=v0", &st),
            arena_engine::Action::BonusPp
        ),
        "default v0 activates orb for a 2-cost on turn 1"
    );
    let prized = match AnyPolicy::parse_spec("h0:value=v0,bppv=50").unwrap() {
        AnyPolicy::H0(h) => h,
        _ => panic!("H0"),
    };
    let mut hold = st.clone();
    apply(&db, &mut hold, arena_engine::Action::EndTurn).unwrap();
    let mut spend = st.clone();
    give_pp(&mut spend, me, 0, 1);
    clear_hand(&mut spend, me);
    let h = put_hand(&db, &mut spend, me, "88001110");
    apply(&db, &mut spend, arena_engine::Action::BonusPp).unwrap();
    play(&db, &mut spend, h);
    let v_hold = prized.evaluate(&db, &hold, me);
    let v_spend = prized.evaluate(&db, &spend, me);
    assert!(
        v_hold > v_spend,
        "holding charges should beat spending for a 2-drop (hold={v_hold} spend={v_spend})"
    );
}

#[test]
fn bpp_spec_round_trips() {
    let spec = "h0:bpp1=4,bpp2=8,bppv=12.5";
    let p1 = AnyPolicy::parse_spec(spec).unwrap();
    let s = p1.spec();
    assert!(s.contains("bpp1=4"), "{s}");
    assert!(s.contains("bpp2=8"), "{s}");
    assert!(s.contains("bppv=12.5"), "{s}");
    let p2 = AnyPolicy::parse_spec(&s).unwrap();
    assert_eq!(p1, p2);
}
