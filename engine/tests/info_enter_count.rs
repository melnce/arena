//! Obsessed Test Subject enter-count gates (`engine::info`).

mod common;

use arena_engine::{apply, board_info, hand_info, Action, PlayerId};
use common::{
    choose, cid, end_turn, give_pp, load_db, play, play_id, put_field, put_hand, started,
};

const TEST_SUBJECT: &str = "10931110";

const SUMMONERS: [&str; 5] = [
    "10932110", // Enamored Researcher
    "10933110", // Ecstatic Scholar
    "10934110", // Sephie, Maven Convict
    "10932310", // Humane Love
    "10933310", // Obsidian Raven
];

const CONTROL_SUMMONER: &str = "10633110"; // Fanfare: Summon 2 copies of Crystalspawn

fn set_enter_count(st: &mut arena_engine::State, who: PlayerId, k: i32) {
    st.player_mut(who).enter_counts.clear();
    if k > 0 {
        st.player_mut(who).enter_counts.insert(cid(TEST_SUBJECT), k);
    }
}

fn set_opponent_enter_count(st: &mut arena_engine::State, k: i32) {
    set_enter_count(st, PlayerId::B, k);
}

fn enter_count_gate(gates: &[arena_engine::GateInfo]) -> arena_engine::GateInfo {
    let n = gates.iter().filter(|g| g.kind == "enterCount").count();
    assert_eq!(
        n, 1,
        "expected exactly one enterCount gate, got {n}: {gates:?}"
    );
    gates
        .iter()
        .find(|g| g.kind == "enterCount")
        .cloned()
        .unwrap()
}

fn other_copies_gate(gates: &[arena_engine::GateInfo]) -> arena_engine::GateInfo {
    let n = gates
        .iter()
        .filter(|g| g.kind == "enterCountAtLeast")
        .count();
    assert_eq!(
        n, 1,
        "expected exactly one enterCountAtLeast gate, got {n}: {gates:?}"
    );
    gates
        .iter()
        .find(|g| g.kind == "enterCountAtLeast")
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

fn assert_summoner_hand_gate(gate: &arena_engine::GateInfo, k: i32) {
    assert_eq!(gate.label, "Obsessed Test Subjects entered");
    assert_eq!(gate.need, 0);
    assert_eq!(gate.have, k);
    assert!(!gate.met);
    assert!(!gate.glow);
}

#[test]
fn summoners_in_hand_emit_enter_count_gate_for_k() {
    let db = load_db();
    let me = PlayerId::A;
    for card in SUMMONERS {
        for k in [0, 3, 5, 7] {
            let mut st = started(&db, 1 + k as u64);
            set_enter_count(&mut st, me, k);
            set_opponent_enter_count(&mut st, 9);
            give_pp(&mut st, me, 10, 10);
            st.player_mut(me).hand.clear();
            let _ = put_hand(&db, &mut st, me, card);
            let gate = enter_count_gate(&first_hand_gates(&db, &st, me));
            assert_summoner_hand_gate(&gate, k);
        }
    }
}

fn board_gates_for(
    db: &arena_engine::CardDb,
    st: &arena_engine::State,
    who: PlayerId,
    id: &str,
) -> Vec<arena_engine::GateInfo> {
    board_info(db, st, who)
        .iter()
        .find(|c| c.id == id)
        .expect("card on field")
        .gates
        .clone()
}

#[test]
fn summoners_on_field_emit_owner_enter_count() {
    let db = load_db();
    let me = PlayerId::A;
    for card in ["10932110", "10933110", "10934110"] {
        for k in [0, 3, 5, 7] {
            let mut st = started(&db, 10 + k as u64);
            set_enter_count(&mut st, me, k);
            set_opponent_enter_count(&mut st, 9);
            let slot = put_field(&db, &mut st, me, card);
            let gate = enter_count_gate(&board_gates_for(&db, &st, me, card));
            assert_summoner_hand_gate(&gate, k);

            st.player_mut(me).turns_taken = 5;
            st.player_mut(me).ep = 1;
            apply(
                &db,
                &mut st,
                Action::Evolve {
                    slot: arena_engine::Slot(slot),
                    super_evolve: false,
                },
            )
            .expect("evolve");
            let gate = enter_count_gate(&board_gates_for(&db, &st, me, card));
            assert_summoner_hand_gate(&gate, k);

            let mut st2 = started(&db, 20 + k as u64);
            set_enter_count(&mut st2, me, k);
            set_opponent_enter_count(&mut st2, 9);
            let slot2 = put_field(&db, &mut st2, me, card);
            st2.player_mut(me).turns_taken = 7;
            st2.player_mut(me).sep = 1;
            apply(
                &db,
                &mut st2,
                Action::Evolve {
                    slot: arena_engine::Slot(slot2),
                    super_evolve: true,
                },
            )
            .expect("super-evolve");
            let gate = enter_count_gate(&board_gates_for(&db, &st2, me, card));
            assert_summoner_hand_gate(&gate, k);
        }
    }
}

#[test]
fn enemy_sephie_shows_enemy_enter_count() {
    let db = load_db();
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let mut st = started(&db, 30);
    set_enter_count(&mut st, me, 2);
    set_enter_count(&mut st, opp, 6);
    put_field(&db, &mut st, opp, "10934110");
    let gate = enter_count_gate(&board_gates_for(&db, &st, opp, "10934110"));
    assert_summoner_hand_gate(&gate, 6);
}

#[test]
fn test_subject_in_hand_other_copies_gate() {
    let db = load_db();
    let me = PlayerId::A;
    for (k, want_have, want_met) in [(0, 0, false), (4, 4, false), (5, 5, true), (6, 6, true)] {
        let mut st = started(&db, 40 + k as u64);
        set_enter_count(&mut st, me, k);
        set_opponent_enter_count(&mut st, 9);
        give_pp(&mut st, me, 2, 10);
        st.player_mut(me).hand.clear();
        let _ = put_hand(&db, &mut st, me, TEST_SUBJECT);
        let gate = other_copies_gate(&first_hand_gates(&db, &st, me));
        assert_eq!(gate.label, "other copies entered");
        assert_eq!(gate.need, 5);
        assert_eq!(gate.have, want_have, "k={k}");
        assert_eq!(gate.met, want_met, "k={k}");
        assert_eq!(gate.glow, want_met, "k={k}");
    }
}

fn test_subject_stats(st: &arena_engine::State, who: PlayerId) -> (i32, i32) {
    let ts = st
        .player(who)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == TEST_SUBJECT)
        .expect("test subject on field");
    (ts.attack, ts.defense)
}

#[test]
fn test_subject_play_matches_gate() {
    let db = load_db();
    let me = PlayerId::A;
    for (k, expect_buffed) in [(4, false), (5, true)] {
        let mut st = started(&db, 50 + k as u64);
        set_enter_count(&mut st, me, k);
        give_pp(&mut st, me, 2, 10);
        st.player_mut(me).hand.clear();
        let hand = put_hand(&db, &mut st, me, TEST_SUBJECT);
        let gate = other_copies_gate(&first_hand_gates(&db, &st, me));
        assert_eq!(gate.met, expect_buffed);
        play(&db, &mut st, hand);
        let (atk, def) = test_subject_stats(&st, me);
        if expect_buffed {
            assert_eq!((atk, def), (5, 5));
        } else {
            assert_eq!((atk, def), (2, 2));
        }
    }
}

#[test]
fn sephie_fanfare_summons_match_enter_count_line() {
    let db = load_db();
    let me = PlayerId::A;
    let mut st = started(&db, 60);
    set_enter_count(&mut st, me, 4);
    give_pp(&mut st, me, 7, 10);
    st.player_mut(me).hand.clear();
    let hand = put_hand(&db, &mut st, me, "10934110");
    let gate = enter_count_gate(&first_hand_gates(&db, &st, me));
    assert_eq!(gate.have, 4);
    play(&db, &mut st, hand);
    let subjects: Vec<(i32, i32)> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == TEST_SUBJECT)
        .map(|c| (c.attack, c.defense))
        .collect();
    assert_eq!(subjects.len(), 2);
    assert!(subjects.contains(&(2, 2)), "first summon is 2/2 at k=4");
    assert!(subjects.contains(&(5, 5)), "second summon is 5/5 at k=4");
}

#[test]
fn test_subject_on_field_shows_no_gate() {
    let db = load_db();
    let me = PlayerId::A;
    let mut st = started(&db, 70);
    set_enter_count(&mut st, me, 6);
    put_field(&db, &mut st, me, TEST_SUBJECT);
    let gates = board_gates_for(&db, &st, me, TEST_SUBJECT);
    assert!(
        gates.iter().all(|g| g.kind != "enterCountAtLeast"),
        "field Test Subject must not show enterCountAtLeast: {gates:?}"
    );
}

#[test]
fn non_test_subject_summoner_has_no_enter_count_line() {
    let db = load_db();
    let me = PlayerId::A;
    let mut st = started(&db, 80);
    set_enter_count(&mut st, me, 5);
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(&db, &mut st, me, CONTROL_SUMMONER);
    let gates = first_hand_gates(&db, &st, me);
    assert!(
        gates.iter().all(|g| g.kind != "enterCount"),
        "{CONTROL_SUMMONER} must not show enterCount: {gates:?}"
    );
}
