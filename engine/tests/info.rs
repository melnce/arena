//! Presentation helpers (`engine::info`) — not on the apply / legal hot path.

mod common;

use arena_engine::{board_info, hand_info, legal_actions, Action, PlayerId};
use common::{give_pp, load_db, put_field, put_hand, started};

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
