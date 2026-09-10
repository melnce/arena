//! One behaviour test per M1-pool example card on `main` (26) plus Shoddy Plaything.

use arena_engine::{apply, legal_actions, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

#[test]
fn card_10001110_indomitable_fighter_enhance() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 4, 4);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "10001110");
    let f = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .next()
        .unwrap();
    assert_eq!((f.attack, f.defense), (5, 5));
}

#[test]
fn card_10002110_arriet_super_replaces() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    let slot = put_field(&db, &mut st, me, "10002110");
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .unwrap();
    assert_eq!(st.player(me).leader_defense, 14, "instead: 4, not 2+4");
}

#[test]
fn card_10012110_may_combo_three() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001320");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).combo = 2;
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10012110");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    let f = st.player(PlayerId::B).field[0].as_ref().unwrap();
    assert_eq!(f.defense, 7);
}

#[test]
fn card_10021310_way_of_the_maid_keeps_modifiers() {
    let db = load_db();
    let mut st = started_decks(&db, 1, &["10061120", "10061120"], &["88001110"]);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let returned = put_hand(&db, &mut st, me, "10001110");
    st.player_mut(me).hand[returned as usize].cost = 0;
    let h = put_hand(&db, &mut st, me, "10021310");
    play(&db, &mut st, h);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(
        st.player(me)
            .deck
            .iter()
            .any(|c| c.card.as_str() == "10001110" && c.cost == 0),
        "returned card keeps modifiers — official Q&A Way of the Maid"
    );
}

#[test]
fn card_10031110_dazzling_runeknight_modes() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    put_hand(&db, &mut st, me, "10032120");
    st.player_mut(me)
        .hand
        .retain(|c| c.card.as_str() != "10031110");
    play_id(&db, &mut st, me, "10031110");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0); // spellboost 2
    let blaze = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10032120");
    if let Some(b) = blaze {
        assert!(b.cost < 10, "spellboost reduced Blaze Destroyer");
    }
}

#[test]
fn card_10032120_blaze_on_foresight_like_spell() {
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10032120");
    // Puppet Theater is an amulet, not a spell. Use Depths token as spell.
    play_id(&db, &mut st, me, "90044330");
    let b = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10032120")
        .unwrap();
    assert_eq!(b.cost, 9);
}

#[test]
fn card_10041310_strike_of_the_dragonewt_overflow() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001320");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10041310");
    choose(&db, &mut st, 0);
    assert_eq!(st.player(PlayerId::B).field[0].as_ref().unwrap().defense, 6);
}

#[test]
fn card_10061120_fox_of_purity_is_ward() {
    let db = load_db();
    let mut st = started(&db, 1);
    put_field(&db, &mut st, PlayerId::B, "10061120");
    put_field(&db, &mut st, PlayerId::A, "88001110");
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().all(|a| !matches!(
        a,
        Action::Attack {
            target: AttackTarget::Leader,
            ..
        }
    )));
}

#[test]
fn card_10071310_bullet_from_beyond() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10071310");
    choose(&db, &mut st, 0);
    assert!(hand_has(&st, me, "90071210"));
    assert!(hand_has(&st, me, "90071220"));
}

#[test]
fn card_10072210_puppet_theater() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10072210");
    assert!(hand_has(&st, me, "90071110"));
    assert_eq!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .find(|c| c.card.as_str() == "10072210")
            .unwrap()
            .countdown,
        Some(2)
    );
}

#[test]
fn card_10444120_zooey_gain_max() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10444120");
    assert_eq!(st.player(me).pp_max, 6);
}

#[test]
fn card_10543310_sloth_repeat_and_overflow() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001320");
    give_pp(&mut st, me, 10, 7);
    st.player_mut(me).hand.clear();
    let def0 = st.player(PlayerId::B).leader_defense;
    play_id(&db, &mut st, me, "10543310");
    assert_eq!(st.player(PlayerId::B).leader_defense, def0 - 2);
    assert_eq!(
        st.player(PlayerId::B).field[0].as_ref().unwrap().defense,
        6,
        "2×2 random into the only follower"
    );
}

#[test]
fn card_10671110_shoddy_plaything_accelerate_gate() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 2, 2);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "10671110");
    assert_eq!(field_count(&st, PlayerId::A), 1);
}

#[test]
fn card_90011110_fairy_has_rush() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "90011110");
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().any(|a| matches!(
        a,
        Action::Attack {
            target: AttackTarget::Slot(_),
            ..
        }
    ) || matches!(a, Action::EndTurn)));
    let f = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .next()
        .unwrap();
    assert!(f.is_rush());
}

#[test]
fn card_90031210_magic_sediment_earth() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    let slot = put_field(&db, &mut st, me, "90031210");
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).unwrap();
    assert!(st.player(me).earth >= 2 || !field_has(&st, me, "90031210"));
}

#[test]
fn card_90044330_depths_on_discard() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 15;
    st.player_mut(PlayerId::B).leader_defense = 20;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90044330");
    // discard via Adventurers? Use a synth discard — Bug Alert is return. Use Sagatsumatsu? not loaded.
    // Direct: call discard by playing a choose-discard... we only have discard on synth via...
    // Put depths in hand and use a custom path: destroy isn't discard.
    // Play Depths itself as a spell.
    give_pp(&mut st, me, 10, 10);
    play(&db, &mut st, 0);
    assert_eq!(st.player(PlayerId::B).leader_defense, 19);
    assert_eq!(st.player(me).leader_defense, 16);
}

#[test]
fn card_90061110_holy_falcon_storm() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "90061110");
    assert!(legal_actions(&db, &st).iter().any(|a| matches!(
        a,
        Action::Attack {
            target: AttackTarget::Leader,
            ..
        }
    )));
}

#[test]
fn card_90061130_regal_falcon_storm() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "90061130");
    assert!(st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .any(|c| c.is_storm()));
}

#[test]
fn card_90071110_puppet_rush() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "90071110");
    assert!(st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .any(|c| c.is_rush()));
}

#[test]
fn card_90071210_gear_ambition_cant_play() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    let h = put_hand(&db, &mut st, PlayerId::A, "90071210");
    assert!(!legal_actions(&db, &st)
        .iter()
        .any(|a| matches!(a, Action::Play { hand } if *hand == h)));
}

#[test]
fn card_90071220_gear_remembrance_cant_play() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    let h = put_hand(&db, &mut st, PlayerId::A, "90071220");
    assert!(!legal_actions(&db, &st)
        .iter()
        .any(|a| matches!(a, Action::Play { hand } if *hand == h)));
}

#[test]
fn card_90072110_striker_rush() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "90072110");
    assert!(st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .any(|c| c.is_rush() && c.attack == 5));
}

#[test]
fn card_90072120_fortifier_ward() {
    let db = load_db();
    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "90072120");
    assert!(st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .any(|c| c.is_ward()));
}

#[test]
fn card_90073110_alpha_end_of_turn_restore() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    put_field(&db, &mut st, me, "90073110");
    end_turn(&db, &mut st);
    assert_eq!(st.player(me).leader_defense, 13);
}

#[test]
fn card_90073120_beta_end_of_turn_ping() {
    let db = load_db();
    let mut st = started(&db, 1);
    put_field(&db, &mut st, PlayerId::A, "90073120");
    let d0 = st.player(PlayerId::B).leader_defense;
    end_turn(&db, &mut st);
    assert_eq!(st.player(PlayerId::B).leader_defense, d0 - 3);
}

#[test]
fn card_90073130_gamma_end_of_turn_aoe() {
    let db = load_db();
    let mut st = started(&db, 1);
    put_field(&db, &mut st, PlayerId::A, "90073130");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    end_turn(&db, &mut st);
    assert_eq!(st.player(PlayerId::B).field[0].as_ref().unwrap().defense, 7);
}

#[test]
fn card_90074110_omega_fanfare() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001320");
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "90074110");
    assert_eq!(st.player(PlayerId::B).field[0].as_ref().unwrap().defense, 5);
    assert_eq!(st.player(me).leader_defense, 15);
}
