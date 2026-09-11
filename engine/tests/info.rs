//! Presentation helpers (`engine::info`) — not on the apply / legal hot path.

mod common;

use arena_engine::{
    apply, board_info, hand_info, legal_actions, player_info, Action, PlayerId, Slot,
};
use common::{end_turn, give_pp, load_db, play_id, put_field, put_hand, started};

#[test]
fn depths_of_the_eld_sword_enhance_at_8_pp() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 8, 8);
    put_field(&db, &mut st, PlayerId::B, "88001110");
    let _ = put_hand(&db, &mut st, me, "90024320");
    let info = hand_info(&db, &st, me);
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].id, "90024320");
    assert_eq!(info[0].cost, Some(1));
    assert_eq!(info[0].form.as_deref(), Some("enhance"));
    // Enhance/Accelerate/Crystallize are no longer gates — the paid cost +
    // `form` are enough (A4/A5). The client hides any leftover form lines.
    assert!(info[0].gates.iter().all(|g| g.kind != "enhance"));
}

#[test]
fn hark_necromancy_progress() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 3, 3);
    let _ = put_hand(&db, &mut st, me, "10753310");

    st.player_mut(me).shadows = 4;
    let short = hand_info(&db, &st, me);
    let necro = short[0]
        .gates
        .iter()
        .find(|g| g.kind == "necromancy")
        .expect("necromancy gate");
    assert_eq!(necro.need, 6);
    assert_eq!(necro.have, 4);
    assert!(!necro.met);

    st.player_mut(me).shadows = 6;
    let ready = hand_info(&db, &st, me);
    let necro = ready[0]
        .gates
        .iter()
        .find(|g| g.kind == "necromancy")
        .expect("necromancy gate");
    assert_eq!(necro.need, 6);
    assert_eq!(necro.have, 6);
    assert!(necro.met);
}

#[test]
fn rally_twenty_at_twelve() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).rally = 12;
    let _ = put_hand(&db, &mut st, me, "10824110");
    let info = hand_info(&db, &st, me);
    let rally = info[0]
        .gates
        .iter()
        .find(|g| g.kind == "rally")
        .expect("rally gate");
    assert_eq!(rally.need, 20);
    assert_eq!(rally.have, 12);
    assert!(!rally.met);
}

#[test]
fn board_info_occupied_slots_only() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001120");
    let info = board_info(&db, &st, me);
    assert_eq!(info.len(), 2);
    assert_eq!(info[0].slot, 0);
    assert_eq!(info[1].slot, 1);
    assert_eq!(info[0].id, "88001110");
}

#[test]
fn non_acting_player_playable_and_can_attack_are_false() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).hand.clear();
    st.player_mut(opp).hand.clear();
    give_pp(&mut st, me, 5, 5);
    give_pp(&mut st, opp, 5, 5);
    let _ = put_hand(&db, &mut st, me, "88001110");
    let _ = put_hand(&db, &mut st, opp, "88001110");
    let _ = put_hand(&db, &mut st, opp, "88001120");
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, me, "88001120");

    let legal = legal_actions(&db, &st);
    let info_a = hand_info(&db, &st, me);
    for (i, card) in info_a.iter().enumerate() {
        let expect = legal
            .iter()
            .any(|a| matches!(a, Action::Play { hand } if *hand == i as u8));
        assert_eq!(card.playable, expect, "hand_info(A)[{i}] vs legal_actions");
    }

    let info_b = hand_info(&db, &st, opp);
    assert!(!info_b.is_empty());
    assert!(
        info_b.iter().all(|c| !c.playable),
        "non-acting hand_info must have playable=false: {info_b:?}"
    );
    let board_b = board_info(&db, &st, opp);
    assert!(!board_b.is_empty());
    assert!(
        board_b.iter().all(|c| !c.can_attack),
        "non-acting board_info must have can_attack=false: {board_b:?}"
    );
}

#[test]
fn slice_of_domesticity_count_at_least_gate() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 2, 2);
    let _ = put_hand(&db, &mut st, me, "10823310");

    put_field(&db, &mut st, me, "88001110");
    let one = hand_info(&db, &st, me);
    let gate = one[0]
        .gates
        .iter()
        .find(|g| g.kind == "countAtLeast")
        .expect("countAtLeast gate");
    assert_eq!(gate.label, "allied cards on the field");
    assert_eq!(gate.need, 2);
    assert_eq!(gate.have, 1);
    assert!(!gate.met);
    assert!(
        one[0]
            .gates
            .iter()
            .filter(|g| g.kind == "countAtLeast")
            .count()
            == 1,
        "Not-branch must not emit an inverse gate: {:?}",
        one[0].gates
    );

    put_field(&db, &mut st, me, "88001120");
    let two = hand_info(&db, &st, me);
    let gate = two[0]
        .gates
        .iter()
        .find(|g| g.kind == "countAtLeast")
        .expect("countAtLeast gate");
    assert_eq!(gate.label, "allied cards on the field");
    assert_eq!(gate.need, 2);
    assert_eq!(gate.have, 2);
    assert!(gate.met);
    assert_eq!(
        two[0]
            .gates
            .iter()
            .filter(|g| g.kind == "countAtLeast")
            .count(),
        1
    );
}

#[test]
fn player_info_evolve_unlock_and_countdown() {
    let db = load_db();
    let mut st = started(&db, 1);
    assert!(!st.player(PlayerId::A).is_second, "A is first");
    assert!(st.player(PlayerId::B).is_second, "B is second");

    st.player_mut(PlayerId::A).turns_taken = 1;
    let a = player_info(&db, &st, PlayerId::A);
    assert!(!a.evolve_unlocked);
    assert!(!a.super_evolve_unlocked);
    assert_eq!(a.evolve_unlock_in, 4);
    assert_eq!(a.super_evolve_unlock_in, 6);

    st.player_mut(PlayerId::A).turns_taken = 4;
    let a = player_info(&db, &st, PlayerId::A);
    assert!(!a.evolve_unlocked);
    assert_eq!(a.evolve_unlock_in, 1);

    st.player_mut(PlayerId::A).turns_taken = 5;
    let a = player_info(&db, &st, PlayerId::A);
    assert!(a.evolve_unlocked);
    assert_eq!(a.evolve_unlock_in, 0);
    assert!(!a.super_evolve_unlocked);
    assert_eq!(a.super_evolve_unlock_in, 2);

    st.player_mut(PlayerId::A).turns_taken = 6;
    assert!(!player_info(&db, &st, PlayerId::A).super_evolve_unlocked);
    st.player_mut(PlayerId::A).turns_taken = 7;
    let a = player_info(&db, &st, PlayerId::A);
    assert!(a.super_evolve_unlocked);
    assert_eq!(a.super_evolve_unlock_in, 0);

    st.player_mut(PlayerId::B).turns_taken = 1;
    let b = player_info(&db, &st, PlayerId::B);
    assert!(!b.evolve_unlocked);
    assert!(!b.super_evolve_unlocked);
    assert_eq!(b.evolve_unlock_in, 3);
    assert_eq!(b.super_evolve_unlock_in, 5);

    st.player_mut(PlayerId::B).turns_taken = 3;
    assert!(!player_info(&db, &st, PlayerId::B).evolve_unlocked);
    assert_eq!(player_info(&db, &st, PlayerId::B).evolve_unlock_in, 1);

    st.player_mut(PlayerId::B).turns_taken = 4;
    let b = player_info(&db, &st, PlayerId::B);
    assert!(b.evolve_unlocked);
    assert_eq!(b.evolve_unlock_in, 0);
    assert!(!b.super_evolve_unlocked);

    st.player_mut(PlayerId::B).turns_taken = 5;
    assert!(!player_info(&db, &st, PlayerId::B).super_evolve_unlocked);
    st.player_mut(PlayerId::B).turns_taken = 6;
    let b = player_info(&db, &st, PlayerId::B);
    assert!(b.super_evolve_unlocked);
    assert_eq!(b.super_evolve_unlock_in, 0);
}

/// Evolved-this-turn (no Rush) can only hit followers on entry; next turn it can
/// hit the leader too, so the yellow flag clears.
#[test]
fn followers_only_this_turn_evolved_no_rush() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    st.player_mut(me).turns_taken = 5;
    st.player_mut(me).ep = 1;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10001110");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(0),
            super_evolve: false,
        },
    )
    .expect("evolve");
    let info = board_info(&db, &st, me);
    assert_eq!(info.len(), 1);
    assert!(info[0].can_attack);
    assert!(!info[0].can_attack_leader);
    assert!(info[0].followers_only_this_turn);
    assert!(info[0].rush_only);
    assert!(info[0].evolved);

    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    let next = board_info(&db, &st, me);
    assert_eq!(next.len(), 1);
    assert!(next[0].can_attack);
    assert!(next[0].can_attack_leader);
    assert!(!next[0].followers_only_this_turn);
    assert!(!next[0].rush_only);
}

#[test]
fn followers_only_this_turn_rush_on_entry() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10631110");
    let info = board_info(&db, &st, me);
    assert_eq!(info.len(), 1);
    assert!(info[0].can_attack);
    assert!(!info[0].can_attack_leader);
    assert!(info[0].followers_only_this_turn);

    end_turn(&db, &mut st);
    let sick = st.player(me).field[0]
        .as_ref()
        .expect("rush follower still on field")
        .flags
        .summoning_sick;
    assert!(sick, "summoning_sick stays true during the opponent's turn");
    let opp = board_info(&db, &st, me);
    assert_eq!(opp.len(), 1);
    assert!(!opp[0].can_attack);
    assert!(!opp[0].followers_only_this_turn);
    assert!(!opp[0].rush_only);
}

#[test]
fn followers_only_this_turn_false_on_opponent_turn_after_evolve() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    st.player_mut(me).turns_taken = 5;
    st.player_mut(me).ep = 1;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10001110");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(0),
            super_evolve: false,
        },
    )
    .expect("evolve");
    assert!(board_info(&db, &st, me)[0].followers_only_this_turn);

    end_turn(&db, &mut st);
    let sick = st.player(me).field[0]
        .as_ref()
        .expect("evolved follower still on field")
        .flags
        .summoning_sick;
    assert!(sick, "summoning_sick stays true during the opponent's turn");
    let opp = board_info(&db, &st, me);
    assert_eq!(opp.len(), 1);
    assert!(!opp[0].can_attack);
    assert!(!opp[0].followers_only_this_turn);
    assert!(!opp[0].rush_only);
}

#[test]
fn followers_only_this_turn_storm_on_entry_is_false() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10021110");
    let info = board_info(&db, &st, me);
    assert_eq!(info.len(), 1);
    assert!(info[0].can_attack);
    assert!(info[0].can_attack_leader);
    assert!(!info[0].followers_only_this_turn);
    assert!(!info[0].rush_only);
}

#[test]
fn hand_info_blocked_reason_pp_and_not_your_turn() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).hand.clear();
    st.player_mut(opp).hand.clear();
    give_pp(&mut st, me, 0, 0);
    give_pp(&mut st, opp, 5, 5);
    let _ = put_hand(&db, &mut st, me, "88001110");
    let _ = put_hand(&db, &mut st, opp, "88001110");
    let mine = hand_info(&db, &st, me);
    assert!(!mine[0].playable);
    assert_eq!(mine[0].blocked_reason.as_deref(), Some("Not enough PP."));
    let theirs = hand_info(&db, &st, opp);
    assert!(!theirs[0].playable);
    assert_eq!(theirs[0].blocked_reason.as_deref(), Some("Not your turn."));
}

#[test]
fn board_info_cannot_attack_reason_ignores_sickness() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001110");
    let inst = st.player_mut(me).field[slot as usize]
        .as_mut()
        .expect("field");
    inst.flags.summoning_sick = true;
    inst.traits.cant_attack_followers = Some(true);
    inst.traits.cant_attack_leader = Some(true);
    let info = board_info(&db, &st, me);
    assert_eq!(
        info[0].cannot_attack_reason.as_deref(),
        Some("Cannot attack.")
    );
}

#[test]
fn board_info_named_counter_is_first_xyz_var_when_no_countdown() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "10031210");
    let inst = st.player_mut(me).field[slot as usize]
        .as_mut()
        .expect("field");
    inst.countdown = None;
    inst.vars.insert(arena_engine::card::VarKey::Y, 4);
    inst.vars.insert(arena_engine::card::VarKey::X, 7);
    let info = board_info(&db, &st, me);
    assert_eq!(info[0].named_counter, Some(7), "X wins over Y");

    let inst = st.player_mut(me).field[slot as usize]
        .as_mut()
        .expect("field");
    inst.vars.remove(&arena_engine::card::VarKey::X);
    inst.countdown = Some(2);
    let info = board_info(&db, &st, me);
    assert_eq!(info[0].named_counter, None, "countdown hides named counter");
}

#[test]
fn player_info_has_leader_barrier_from_damage_cap() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    assert!(!player_info(&db, &st, me).has_leader_barrier);
    st.player_mut(me)
        .leader_mods
        .push(arena_engine::state::LeaderMod {
            max_defense: None,
            damage_cap: Some(1),
            damage_taken_bonus: 0,
            until: None,
        });
    assert!(player_info(&db, &st, me).has_leader_barrier);
}
