//! Artifact counter gates and combo-after-playing presentation (`engine::info`).

mod common;

use arena_engine::{apply, board_info, hand_info, Action, Phase, PlayerId};
use common::{
    choose, cid, end_turn, give_pp, load_db, play, play_id, put_deck, put_field, put_hand, started,
};

const ARTIFACT_FOLLOWERS: [&str; 5] = ["90071130", "90071140", "90071150", "90071160", "90073110"];

const THRESHOLD_CARDS: [&str; 4] = [
    "10771120", // Beat Breaker
    "10771310", // Freerunning
    "10772120", // Audacious Artist
    "10873310", // The Journey Ahead
];

const X_CARDS: [&str; 2] = ["10773310", "10774110"]; // Warp Slash, Scarlet

fn set_distinct_artifacts(st: &mut arena_engine::State, who: PlayerId, k: usize) {
    st.player_mut(who).enter_counts.clear();
    for id in ARTIFACT_FOLLOWERS
        .iter()
        .take(k.min(ARTIFACT_FOLLOWERS.len()))
    {
        st.player_mut(who).enter_counts.insert(cid(id), 1);
    }
}

fn set_artifact_ids(st: &mut arena_engine::State, who: PlayerId, ids: &[&str]) {
    st.player_mut(who).enter_counts.clear();
    for id in ids {
        st.player_mut(who).enter_counts.insert(cid(id), 1);
    }
}

fn other_copies_gate(gates: &[arena_engine::GateInfo]) -> arena_engine::GateInfo {
    gates
        .iter()
        .find(|g| g.label == "other copies entered")
        .cloned()
        .expect("other copies entered gate")
}

fn artifacts_gate(gates: &[arena_engine::GateInfo]) -> arena_engine::GateInfo {
    let n = gates.iter().filter(|g| g.kind == "artifacts").count();
    assert_eq!(
        n, 1,
        "expected exactly one artifacts gate, got {n}: {gates:?}"
    );
    gates
        .iter()
        .find(|g| g.kind == "artifacts")
        .cloned()
        .unwrap()
}

fn first_hand_gates(
    db: &arena_engine::CardDb,
    st: &arena_engine::State,
    who: PlayerId,
) -> Vec<arena_engine::GateInfo> {
    hand_info(db, st, who)[0].gates.clone()
}

#[test]
fn threshold_cards_emit_artifacts_gate_for_k() {
    let db = load_db();
    for card in THRESHOLD_CARDS {
        for k in [0usize, 2, 3, 5] {
            let mut st = started(&db, 1);
            let me = PlayerId::A;
            set_distinct_artifacts(&mut st, me, k);
            give_pp(&mut st, me, 10, 10);
            st.player_mut(me).hand.clear();
            let _ = put_hand(&db, &mut st, me, card);
            let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
            assert_eq!(gate.label, "artifacts");
            assert_eq!(gate.need, 3);
            assert_eq!(gate.have, k as i32);
            assert_eq!(gate.met, k >= 3);
            assert!(gate.glow);
        }
    }
}

#[test]
fn x_cards_emit_artifacts_count_without_threshold() {
    let db = load_db();
    for card in X_CARDS {
        for k in [0usize, 2, 3, 5] {
            let mut st = started(&db, 1);
            let me = PlayerId::A;
            set_distinct_artifacts(&mut st, me, k);
            give_pp(&mut st, me, 10, 10);
            st.player_mut(me).hand.clear();
            let _ = put_hand(&db, &mut st, me, card);
            let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
            assert_eq!(gate.need, 0);
            assert_eq!(gate.have, k as i32);
            assert!(!gate.met);
            assert!(!gate.glow);
        }
    }
}

#[test]
fn non_artifact_follower_does_not_count() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    set_distinct_artifacts(&mut st, me, 2);
    st.player_mut(me).enter_counts.insert(cid("88001110"), 3);
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10771120");
    let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
    assert_eq!(gate.have, 2);
}

fn assert_no_yellow_condition_gates(gates: &[arena_engine::GateInfo], card: &str) {
    assert!(
        !gates.iter().any(|g| g.met && g.glow),
        "{card} must not show met+glow from non-Fanfare hand abilities: {gates:?}"
    );
}

#[test]
fn garodeth_hand_does_not_leak_leader_defense_gate() {
    let db = load_db();
    let mut st = started(&db, 110);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10954120");
    assert_no_yellow_condition_gates(&first_hand_gates(&db, &st, me), "Garodeth vs. Zeth");
}

#[test]
fn frostbow_hand_does_not_leak_end_of_turn_combo_gate() {
    let db = load_db();
    let mut st = started(&db, 111);
    let me = PlayerId::A;
    st.player_mut(me).combo = 3;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10713110");
    assert_no_yellow_condition_gates(&first_hand_gates(&db, &st, me), "Frostbow Sniper");
}

#[test]
fn cutthroat_hand_does_not_leak_evolve_deck_gate() {
    let db = load_db();
    let mut st = started(&db, 112);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, "88001110");
    put_deck(&db, &mut st, me, "88001120");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10974110");
    assert_no_yellow_condition_gates(&first_hand_gates(&db, &st, me), "Cutthroat");
}

#[test]
fn galleon_hand_does_not_leak_super_evolution_gate() {
    let db = load_db();
    let mut st = started(&db, 113);
    let me = PlayerId::A;
    st.player_mut(me).turns_taken = 7;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10464110");
    assert_no_yellow_condition_gates(&first_hand_gates(&db, &st, me), "Galleon");
}

#[test]
fn duplicate_artifact_id_counts_once() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    st.player_mut(me).enter_counts.insert(cid("90071130"), 5);
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10773310");
    let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
    assert_eq!(gate.have, 1);
}

#[test]
fn warp_slash_damage_matches_artifact_count() {
    let db = load_db();
    for k in [0usize, 3] {
        let mut st = started(&db, 10 + k as u64);
        let me = PlayerId::A;
        let opp = PlayerId::B;
        set_distinct_artifacts(&mut st, me, k);
        put_field(&db, &mut st, opp, "88001110");
        put_field(&db, &mut st, opp, "88001120");
        for slot in 0..2 {
            if let Some(f) = st.field_inst_mut(opp, slot) {
                f.defense = 10;
                f.max_defense = 10;
            }
        }
        give_pp(&mut st, me, 3, 10);
        st.player_mut(me).hand.clear();
        let hand = put_hand(&db, &mut st, me, "10773310");
        let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
        play(&db, &mut st, hand);
        for slot in 0..2 {
            let def = st.field_inst(opp, slot).expect("follower").defense;
            assert_eq!(
                10 - def,
                gate.have,
                "slot {slot} took wrong damage at k={k}"
            );
        }
    }
}

#[test]
fn scarlet_damage_matches_artifact_count() {
    let db = load_db();
    for k in [0usize, 3] {
        let mut st = started(&db, 20 + k as u64);
        let me = PlayerId::A;
        let opp = PlayerId::B;
        set_distinct_artifacts(&mut st, me, k);
        put_field(&db, &mut st, opp, "88001110");
        put_field(&db, &mut st, opp, "88001120");
        for slot in 0..2 {
            if let Some(f) = st.field_inst_mut(opp, slot) {
                f.defense = 10;
                f.max_defense = 10;
            }
        }
        give_pp(&mut st, me, 8, 10);
        st.player_mut(me).hand.clear();
        let hand = put_hand(&db, &mut st, me, "10774110");
        let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
        play(&db, &mut st, hand);
        for slot in 0..2 {
            let def = st.field_inst(opp, slot).expect("follower").defense;
            assert_eq!(10 - def, gate.have, "slot {slot} at k={k}");
        }
    }
}

#[test]
fn beat_breaker_threshold_is_check_time_count() {
    let db = load_db();
    let mut st = started(&db, 85);
    let me = PlayerId::A;
    set_distinct_artifacts(&mut st, me, 2);
    give_pp(&mut st, me, 7, 10);
    st.player_mut(me).hand.clear();
    let hand = put_hand(&db, &mut st, me, "10771120");
    let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
    assert_eq!(gate.have, 2);
    assert_eq!(gate.need, 3);
    assert!(!gate.met);
    let before = common::field_count(&st, me);
    play(&db, &mut st, hand);
    let summoned = common::field_count(&st, me) - before - 1;
    assert_eq!(summoned, 1, "k=2 must summon one copy");
}

#[test]
fn beat_breaker_summons_two_when_met() {
    let db = load_db();
    for (k, expect) in [(2usize, 1), (3, 2)] {
        let mut st = started(&db, 30 + k as u64);
        let me = PlayerId::A;
        set_distinct_artifacts(&mut st, me, k);
        give_pp(&mut st, me, 7, 10);
        st.player_mut(me).hand.clear();
        let hand = put_hand(&db, &mut st, me, "10771120");
        let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
        assert_eq!(gate.met, k >= 3);
        let before = common::field_count(&st, me);
        play(&db, &mut st, hand);
        let summoned = common::field_count(&st, me) - before - 1;
        assert_eq!(summoned, expect, "k={k}");
    }
}

#[test]
fn journey_ahead_recovers_ep_when_met() {
    let db = load_db();
    for (k, expect_gain) in [(2usize, false), (3, true)] {
        let mut st = started(&db, 40 + k as u64);
        let me = PlayerId::A;
        let opp = PlayerId::B;
        set_distinct_artifacts(&mut st, me, k);
        put_field(&db, &mut st, opp, "88001110");
        give_pp(&mut st, me, 3, 10);
        st.player_mut(me).ep = 0;
        st.player_mut(me).hand.clear();
        let hand = put_hand(&db, &mut st, me, "10873310");
        let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
        assert_eq!(gate.met, k >= 3);
        play(&db, &mut st, hand);
        assert!(matches!(st.phase, Phase::Choice { .. }));
        let ep_before = st.player(me).ep;
        choose(&db, &mut st, 0);
        let ep_after = st.player(me).ep;
        if expect_gain {
            assert_eq!(ep_after, ep_before + 1);
        } else {
            assert_eq!(ep_after, ep_before);
        }
    }
}

#[test]
fn freerunning_activates_both_modes_when_met() {
    let db = load_db();
    let mut st = started(&db, 50);
    let me = PlayerId::A;
    set_distinct_artifacts(&mut st, me, 3);
    give_pp(&mut st, me, 1, 10);
    st.player_mut(me).hand.clear();
    let hand = put_hand(&db, &mut st, me, "10771310");
    let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
    assert!(gate.met);
    play(&db, &mut st, hand);
    assert!(!matches!(st.phase, Phase::Choice { .. }));
    assert!(st
        .player(me)
        .hand
        .iter()
        .any(|c| c.card.as_str() == "90071130"));
    assert!(st
        .player(me)
        .hand
        .iter()
        .any(|c| c.card.as_str() == "90071140"));
}

#[test]
fn artifact_count_is_card_owners_not_viewer() {
    let db = load_db();
    let mut st = started(&db, 60);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    set_distinct_artifacts(&mut st, me, 1);
    set_distinct_artifacts(&mut st, opp, 3);
    put_field(&db, &mut st, opp, "10774120");
    let info = board_info(&db, &st, opp);
    let myuu = info.iter().find(|c| c.id == "10774120").expect("myuu");
    let gate = artifacts_gate(&myuu.gates.clone());
    assert_eq!(gate.have, 3);
    assert_eq!(gate.label, "artifacts");
}

#[test]
fn myuu_hand_shows_artifacts_without_glow() {
    let db = load_db();
    let mut st = started(&db, 70);
    let me = PlayerId::A;
    set_distinct_artifacts(&mut st, me, 3);
    give_pp(&mut st, me, 4, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10774120");
    let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
    assert_eq!(gate.have, 3);
    assert_eq!(gate.need, 3);
    assert_eq!(gate.label, "artifacts");
    assert!(gate.met);
    assert!(!gate.glow);
}

#[test]
fn myuu_line_shows_current_count_without_ancient() {
    let db = load_db();
    let mut st = started(&db, 74);
    let me = PlayerId::A;
    set_artifact_ids(&mut st, me, &["90071130", "90071150"]);
    give_pp(&mut st, me, 4, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10774120");
    let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
    assert_eq!(gate.have, 2, "line shows current count, not evolve summon");
    assert_eq!(gate.need, 3);
    assert!(!gate.met);
}

#[test]
fn myuu_line_shows_current_count_with_ancient_already_entered() {
    let db = load_db();
    let mut st = started(&db, 75);
    let me = PlayerId::A;
    set_artifact_ids(&mut st, me, &["90071130", "90071140"]);
    give_pp(&mut st, me, 4, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10774120");
    let gate = artifacts_gate(&first_hand_gates(&db, &st, me));
    assert_eq!(gate.have, 2);
    assert_eq!(gate.need, 3);
    assert!(!gate.met);
}

#[test]
fn myuu_super_evolve_grants_storm_when_ancient_not_yet_entered() {
    let db = load_db();
    let mut st = started(&db, 76);
    let me = PlayerId::A;
    set_artifact_ids(&mut st, me, &["90071130", "90071150"]);
    let slot = put_field(&db, &mut st, me, "10774120");
    let gate = artifacts_gate(&board_info(&db, &st, me)[0].gates.clone());
    assert_eq!(gate.have, 2);
    assert!(!gate.met);
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve: true,
        },
    )
    .expect("super-evolve");
    let myuu = st.field_inst(me, slot).expect("myuu");
    assert_eq!(myuu.traits.storm, Some(true));
}

#[test]
fn myuu_super_evolve_no_storm_when_ancient_already_counted() {
    let db = load_db();
    let mut st = started(&db, 77);
    let me = PlayerId::A;
    set_artifact_ids(&mut st, me, &["90071130", "90071140"]);
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    let slot = put_field(&db, &mut st, me, "10774120");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve: true,
        },
    )
    .expect("super-evolve");
    let myuu = st.field_inst(me, slot).expect("myuu");
    assert_ne!(myuu.traits.storm, Some(true));
}

#[test]
fn myuu_on_field_unevolved_shows_artifacts() {
    let db = load_db();
    let mut st = started(&db, 71);
    let me = PlayerId::A;
    set_distinct_artifacts(&mut st, me, 3);
    put_field(&db, &mut st, me, "10774120");
    let info = board_info(&db, &st, me);
    let gate = artifacts_gate(&info[0].gates.clone());
    assert_eq!(gate.have, 3);
    assert_eq!(gate.label, "artifacts");
}

#[test]
fn myuu_super_evolve_grants_storm_at_three_distinct() {
    let db = load_db();
    let mut st = started(&db, 78);
    let me = PlayerId::A;
    set_distinct_artifacts(&mut st, me, 3);
    let slot = put_field(&db, &mut st, me, "10774120");
    let gate = artifacts_gate(&board_info(&db, &st, me)[0].gates.clone());
    assert_eq!(gate.have, 3);
    assert!(gate.met);
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve: true,
        },
    )
    .expect("super-evolve");
    let myuu = st.field_inst(me, slot).expect("myuu");
    assert_eq!(myuu.traits.storm, Some(true));
}

#[test]
fn myuu_hides_artifacts_after_super_evolve() {
    let db = load_db();
    let mut st = started(&db, 73);
    let me = PlayerId::A;
    set_distinct_artifacts(&mut st, me, 3);
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    let slot = put_field(&db, &mut st, me, "10774120");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve: true,
        },
    )
    .expect("super-evolve");
    let info = board_info(&db, &st, me);
    assert!(
        info[0].gates.iter().all(|g| g.kind != "artifacts"),
        "super-evolved Myuu must not show artifacts line"
    );
}

#[test]
fn myuu_hides_artifacts_after_evolve() {
    let db = load_db();
    let mut st = started(&db, 72);
    let me = PlayerId::A;
    set_distinct_artifacts(&mut st, me, 3);
    st.player_mut(me).turns_taken = 5;
    st.player_mut(me).ep = 1;
    let slot = put_field(&db, &mut st, me, "10774120");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve");
    let info = board_info(&db, &st, me);
    assert!(
        info[0].gates.iter().all(|g| g.kind != "artifacts"),
        "evolved Myuu must not show artifacts line"
    );
}

#[test]
fn fanfare_threshold_cards_on_field_show_no_artifacts() {
    let db = load_db();
    let me = PlayerId::A;
    for id in ["10771120", "10772120", "10774110"] {
        let mut st = started(&db, 80);
        set_distinct_artifacts(&mut st, me, 3);
        put_field(&db, &mut st, me, id);
        let info = board_info(&db, &st, me);
        assert!(
            info[0].gates.iter().all(|g| g.kind != "artifacts"),
            "{id} on field must not show artifacts"
        );
    }
}

#[test]
fn combo_hand_after_playing_label_and_counts() {
    let db = load_db();
    let mut st = started(&db, 90);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 4, 4);
    let _ = put_hand(&db, &mut st, me, "10011130");

    st.player_mut(me).combo = 2;
    let gates = first_hand_gates(&db, &st, me);
    let combo = gates.iter().find(|g| g.kind == "combo").expect("combo");
    assert_eq!(combo.label, "combo after playing");
    assert_eq!(combo.need, 3);
    assert_eq!(combo.have, 3);
    assert!(combo.met);

    st.player_mut(me).combo = 1;
    let gates = first_hand_gates(&db, &st, me);
    let combo = gates.iter().find(|g| g.kind == "combo").expect("combo");
    assert_eq!(combo.have, 2);
    assert!(!combo.met);

    end_turn(&db, &mut st);
    st.player_mut(me).combo = 2;
    let gates = first_hand_gates(&db, &st, me);
    let combo = gates.iter().find(|g| g.kind == "combo").expect("combo");
    assert_eq!(combo.have, 1, "stale combo on opponent's turn");
}

#[test]
fn combo_board_label_and_no_double_count() {
    let db = load_db();
    let mut st = started(&db, 91);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 10, 10);
    play_id(&db, &mut st, me, "10631110");
    play_id(&db, &mut st, me, "10631110");
    play_id(&db, &mut st, me, "10011130");
    assert_eq!(st.player(me).combo, 3);
    let board = board_info(&db, &st, me);
    let treant = board.iter().find(|i| i.id == "10011130").expect("treant");
    let combo = treant
        .gates
        .iter()
        .find(|g| g.kind == "combo")
        .expect("combo");
    assert_eq!(combo.label, "combo");
    assert_eq!(combo.have, 3);
    assert!(combo.met);
    assert!(!combo.glow);

    end_turn(&db, &mut st);
    let board = board_info(&db, &st, me);
    let opp = board.iter().find(|i| i.id == "10011130").expect("treant");
    if let Some(combo) = opp.gates.iter().find(|g| g.kind == "combo") {
        assert_eq!(combo.have, 0);
    }
}

#[test]
fn drache_aluzard_other_copies_entered_in_hand() {
    let db = load_db();
    let me = PlayerId::A;
    for (previous, want_have, want_met) in
        [(0, 0, false), (1, 1, false), (2, 2, true), (3, 3, true)]
    {
        let mut st = started(&db, 100 + previous as u64);
        if previous > 0 {
            st.player_mut(me)
                .enter_counts
                .insert(cid("10844110"), previous);
        }
        give_pp(&mut st, me, 4, 10);
        st.player_mut(me).hand.clear();
        let _ = put_hand(&db, &mut st, me, "10844110");
        let gate = other_copies_gate(&first_hand_gates(&db, &st, me));
        assert_eq!(gate.have, want_have, "previous={previous}");
        assert_eq!(gate.need, 2);
        assert_eq!(gate.met, want_met, "previous={previous}");
        assert_eq!(gate.glow, want_met);
    }
}

#[test]
fn drache_aluzard_play_matches_gate_and_evolve_threshold() {
    let db = load_db();
    let mut st = started(&db, 200);
    let me = PlayerId::A;
    st.player_mut(me).enter_counts.insert(cid("10844110"), 2);
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    let hand = put_hand(&db, &mut st, me, "10844110");
    let gate = other_copies_gate(&first_hand_gates(&db, &st, me));
    assert_eq!(gate.have, 2);
    assert!(gate.met);
    play(&db, &mut st, hand);
    let d = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10844110")
        .unwrap();
    assert_eq!(d.attack, 8);
    assert_eq!(d.defense, 8);
    assert!(d.evolved);
}

#[test]
fn limil_amount_at_least_evaluates_both_sides() {
    let db = load_db();
    let mut st = started(&db, 101);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).leader_defense = 15;
    st.player_mut(opp).leader_defense = 10;
    give_pp(&mut st, me, 2, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, "10851130");
    let info = hand_info(&db, &st, me);
    let gate = info[0]
        .gates
        .iter()
        .find(|g| g.kind == "amountAtLeast")
        .expect("amountAtLeast");
    assert_eq!(gate.have, 15);
    assert_eq!(gate.need, 11);
    assert!(gate.met);
}
