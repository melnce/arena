//! R4 differential: fuse filter/transform, choose {card}, bounce reset,
//! includeLeader, SE-vs-Bane, max_defense debuff.

use arena_engine::{
    apply, apply_neutral, legal_actions_neutral, Action, AttackTarget, ChoiceNode, NeutralAction,
    Phase, PlayerId, Slot,
};

mod common;
use common::*;

fn fuse_options(st: &arena_engine::State) -> Vec<u8> {
    match &st.phase {
        Phase::Choice {
            node: ChoiceNode::FusePartners { options, .. },
            ..
        } => options.clone(),
        _ => panic!("expected FusePartners, got {:?}", st.phase),
    }
}

fn complete_fuse(db: &arena_engine::CardDb, st: &mut arena_engine::State, host: u8, partner: u8) {
    apply_neutral(
        db,
        st,
        &NeutralAction::Fuse {
            player: "a".into(),
            host_pos: host,
            partner_pos: vec![partner],
        },
    )
    .expect("fuse");
}

// ----- E7 -----

#[test]
fn gear_partner_pool_excludes_striker_fortifier() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90072110");
    put_hand(&db, &mut st, me, "90072120");
    put_hand(&db, &mut st, me, "90071220");
    apply(&db, &mut st, Action::Fuse { host: 0 }).unwrap();
    let opts = fuse_options(&st);
    assert!(
        opts.iter().all(|&p| {
            let id = st.player(me).hand[p as usize].card.as_str();
            id != "90072110" && id != "90072120"
        }),
        "Gear partners must be Artifact amulets, not Striker/Fortifier: {opts:?}"
    );
    assert!(opts.contains(&3), "Gear of Remembrance is a legal partner");
}

#[test]
fn fortifier_partner_pool_includes_a_gear() {
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90072120");
    put_hand(&db, &mut st, me, "90071210");
    apply(&db, &mut st, Action::Fuse { host: 0 }).unwrap();
    let opts = fuse_options(&st);
    assert!(
        opts.contains(&1),
        "Fortifier fuses Artifact cards including Gears"
    );
}

// ----- E8 -----

#[test]
fn fuse_transform_can_fuse_again_same_turn_and_alpha_remembers_beta() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071220");
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90071210");
    complete_fuse(&db, &mut st, 0, 1);
    assert!(
        hand_has(&st, me, "90072120"),
        "Remembrance + Gear → Fortifier"
    );
    let host = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "90072120")
        .unwrap() as u8;
    let partner = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "90071210")
        .unwrap() as u8;
    complete_fuse(&db, &mut st, host, partner);
    assert!(
        hand_has(&st, me, "90073110"),
        "Fortifier + Gear (cost 1) → Ominous α in the same turn"
    );
    let alpha = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "90073110")
        .unwrap() as u8;
    put_hand(&db, &mut st, me, "90073120");
    let beta = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "90073120")
        .unwrap() as u8;
    complete_fuse(&db, &mut st, alpha, beta);
    assert!(hand_has(&st, me, "90073110"), "α stays after fusing β");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    let alpha = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "90073110")
        .unwrap() as u8;
    put_hand(&db, &mut st, me, "90073130");
    let gamma = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "90073130")
        .unwrap() as u8;
    complete_fuse(&db, &mut st, alpha, gamma);
    assert!(
        hand_has(&st, me, "90074110"),
        "α remembers β across a turn and transforms with γ"
    );
}

// ----- E9 -----

#[test]
fn hand_choose_emits_card_id_and_legal_dedupes_copies() {
    let db = load_db();
    let mut st = started(&db, 4);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "88001120");
    put_hand(&db, &mut st, me, "10844120");
    play_id_at_last(&db, &mut st, me);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    let neu = legal_actions_neutral(&db, &st);
    let cards: Vec<_> = neu
        .iter()
        .filter_map(|a| match a {
            NeutralAction::Choose {
                option: arena_engine::trace::ChooseOptionJson::Card { card },
                ..
            } => Some(card.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        cards.contains(&"88001110"),
        "hand choose is {{card}}, not {{mode}}: {neu:?}"
    );
    assert_eq!(
        cards.iter().filter(|c| **c == "88001110").count(),
        1,
        "copies of one id collapse to one option"
    );
    assert!(!neu.iter().any(|a| matches!(
        a,
        NeutralAction::Choose {
            option: arena_engine::trace::ChooseOptionJson::Mode { .. },
            ..
        }
    )));
}

fn play_id_at_last(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId) {
    let h = (st.player(who).hand.len() - 1) as u8;
    play(db, st, h);
}

// ----- E10 -----

#[test]
fn enhanced_fighter_returned_and_replayed_at_3_pp_is_printed() {
    let db = load_db();
    let mut st = started(&db, 5);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10001110");
    play(&db, &mut st, 0);
    let f = st.player(me).field.iter().flatten().next().unwrap();
    assert_eq!((f.attack, f.defense), (5, 5), "Enhance (4)");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    let h = put_hand(&db, &mut st, me, "10012310");
    play(&db, &mut st, h);
    choose(&db, &mut st, 0);
    let back = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10001110")
        .expect("bounced");
    assert_eq!(back.attack, 2);
    assert_eq!(back.defense, 2);
    assert!(!back.evolved);
    give_pp(&mut st, me, 3, 3);
    let pos = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == "10001110")
        .unwrap() as u8;
    play(&db, &mut st, pos);
    let f = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10001110")
        .unwrap();
    assert_eq!((f.attack, f.defense), (2, 2));
}

#[test]
fn evolved_follower_returned_to_hand_is_unevolved() {
    let db = load_db();
    let mut st = started(&db, 6);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001110");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.evolved = true;
        f.attack += 2;
        f.defense += 2;
        f.max_defense += 2;
    }
    give_pp(&mut st, me, 10, 10);
    put_field(&db, &mut st, PlayerId::B, "88001320");
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10012310");
    play(&db, &mut st, h);
    choose(&db, &mut st, 0);
    let back = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "88001110")
        .expect("bounced");
    assert!(!back.evolved);
    assert_eq!((back.attack, back.defense), (2, 2));
}

// ----- E11 -----

#[test]
fn lumiore_fanfare_hits_enemy_leader_and_followers() {
    let db = load_db();
    let mut st = started(&db, 7);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001320");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "88001120");
    put_hand(&db, &mut st, me, "10844120");
    play(&db, &mut st, 2);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert_eq!(st.player(PlayerId::B).leader_defense, 16);
    let tank = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001320")
        .expect("10/10 survives at 6");
    assert_eq!(tank.defense, 6);
    assert!(
        !field_has(&st, PlayerId::B, "88001110"),
        "2/2 is destroyed by the 4 damage"
    );
}

// ----- E12 -----

#[test]
fn super_evolve_own_turn_blocks_bane() {
    let db = load_db();
    let mut st = started(&db, 8);
    end_turn(&db, &mut st);
    let me = PlayerId::B;
    let opp = PlayerId::A;
    let att = put_field(&db, &mut st, me, "10741110");
    if let Some(f) = st.field_inst_mut(me, att) {
        f.super_evolved = true;
        f.evolved = true;
        f.attack = 5;
        f.defense = 4;
        f.max_defense = 4;
        f.flags.summoning_sick = false;
    }
    let def = put_field(&db, &mut st, opp, "10644120");
    if let Some(f) = st.field_inst_mut(opp, def) {
        f.evolved = true;
        f.attack = 2;
        f.defense = 4;
        f.max_defense = 4;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(att),
            target: AttackTarget::Slot(Slot(def)),
        },
    )
    .unwrap();
    assert!(
        field_has(&st, me, "10741110"),
        "SE own-turn protection blocks Bane"
    );
    assert!(!field_has(&st, opp, "10644120"));
    assert_eq!(st.player(opp).leader_defense, 19, "knockback 1");
}

#[test]
fn super_evolve_on_defenders_turn_bane_kills() {
    let db = load_db();
    let mut st = started(&db, 9);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let att = put_field(&db, &mut st, me, "10644120");
    if let Some(f) = st.field_inst_mut(me, att) {
        f.evolved = true;
        f.attack = 2;
        f.defense = 4;
        f.max_defense = 4;
        f.flags.summoning_sick = false;
    }
    let def = put_field(&db, &mut st, opp, "10741110");
    if let Some(f) = st.field_inst_mut(opp, def) {
        f.super_evolved = true;
        f.evolved = true;
        f.attack = 5;
        f.defense = 4;
        f.max_defense = 4;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(att),
            target: AttackTarget::Slot(Slot(def)),
        },
    )
    .unwrap();
    assert!(
        !field_has(&st, opp, "10741110"),
        "Bane kills an SE follower on the opponent's turn"
    );
}

// ----- E13 -----

#[test]
fn defense_debuff_lowers_max_defense_and_current() {
    let db = load_db();
    let mut st = started(&db, 10);
    let opp = PlayerId::B;
    let slot = put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, slot) {
        f.attack = 7;
        f.defense = 5;
        f.max_defense = 7;
    }
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "88001670");
    let f = st.field_inst(opp, slot).expect("still on field");
    assert_eq!(f.attack, 7);
    assert_eq!(f.defense, 1);
    assert_eq!(f.max_defense, 3);
}
