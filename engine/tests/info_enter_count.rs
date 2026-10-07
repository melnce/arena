//! Obsessed Test Subject enter-count gates (`engine::info`).

mod common;

use arena_engine::{apply, board_info, hand_info, Action, CardDb, PlayerId};
use common::{cid, give_pp, load_db, play, put_field, put_hand, started};

const TEST_SUBJECT: &str = "10931110";

const FOLLOWER_SUMMONERS: [&str; 3] = [
    "10932110", // Enamored Researcher
    "10933110", // Ecstatic Scholar
    "10934110", // Sephie, Maven Convict
];

const SPELL_SUMMONERS: [&str; 2] = [
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

fn first_hand_gates(
    db: &CardDb,
    st: &arena_engine::State,
    who: PlayerId,
) -> Vec<arena_engine::GateInfo> {
    hand_info(db, st, who)[0].gates.clone()
}

fn hand_yellow_decision(gates: &[arena_engine::GateInfo]) -> bool {
    gates.iter().any(|g| g.met && g.glow)
}

fn assert_follower_enter_count_gate(gate: &arena_engine::GateInfo, k: i32) {
    assert_eq!(gate.kind, "enterCount");
    assert_eq!(gate.label, "Obsessed Test Subjects entered");
    assert_eq!(gate.need, 5);
    assert_eq!(gate.have, k);
    assert_eq!(gate.met, k >= 5);
    assert!(!gate.glow);
}

#[test]
fn summoners_in_hand_emit_enter_count_gate_for_k() {
    let db = load_db();
    let me = PlayerId::A;
    for card in FOLLOWER_SUMMONERS {
        for k in [0, 4, 5, 7] {
            let mut st = started(&db, 1 + k as u64);
            set_enter_count(&mut st, me, k);
            set_opponent_enter_count(&mut st, 9);
            give_pp(&mut st, me, 10, 10);
            st.player_mut(me).hand.clear();
            let _ = put_hand(&db, &mut st, me, card);
            let gate = enter_count_gate(&first_hand_gates(&db, &st, me));
            assert_follower_enter_count_gate(&gate, k);
        }
    }
}

fn board_gates_for(
    db: &CardDb,
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
    for card in FOLLOWER_SUMMONERS {
        for k in [0, 4, 5, 7] {
            let mut st = started(&db, 10 + k as u64);
            set_enter_count(&mut st, me, k);
            set_opponent_enter_count(&mut st, 9);
            let slot = put_field(&db, &mut st, me, card);
            let gate = enter_count_gate(&board_gates_for(&db, &st, me, card));
            assert_follower_enter_count_gate(&gate, k);

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
            assert_follower_enter_count_gate(&gate, k);

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
            assert_follower_enter_count_gate(&gate, k);
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
    assert_follower_enter_count_gate(&gate, 6);
}

#[test]
fn spells_and_test_subject_have_no_enter_count_gates() {
    let db = load_db();
    let me = PlayerId::A;
    for card in [TEST_SUBJECT, SPELL_SUMMONERS[0], SPELL_SUMMONERS[1]] {
        let mut st = started(&db, 40);
        set_enter_count(&mut st, me, 5);
        set_opponent_enter_count(&mut st, 9);
        give_pp(&mut st, me, 10, 10);
        st.player_mut(me).hand.clear();
        let _ = put_hand(&db, &mut st, me, card);
        let gates = first_hand_gates(&db, &st, me);
        assert!(
            gates.iter().all(|g| g.kind != "enterCount"),
            "{card} must not show enterCount: {gates:?}"
        );
        assert!(
            gates.is_empty(),
            "{card} hand gates should match main (empty): {gates:?}"
        );
        assert!(
            !hand_yellow_decision(&gates),
            "{card} yellow decision should match main: {gates:?}"
        );
    }
}

fn summoned_test_subjects(st: &arena_engine::State, who: PlayerId) -> Vec<(i32, i32)> {
    st.player(who)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == TEST_SUBJECT)
        .map(|c| (c.attack, c.defense))
        .collect()
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
    assert_follower_enter_count_gate(&gate, 4);
    play(&db, &mut st, hand);
    let subjects = summoned_test_subjects(&st, me);
    assert_eq!(subjects.len(), 2);
    assert!(subjects.contains(&(2, 2)), "first summon is 2/2 at k=4");
    assert!(subjects.contains(&(5, 5)), "second summon is 5/5 at k=4");

    let mut st5 = started(&db, 61);
    set_enter_count(&mut st5, me, 5);
    give_pp(&mut st5, me, 7, 10);
    st5.player_mut(me).hand.clear();
    let hand5 = put_hand(&db, &mut st5, me, "10934110");
    let gate5 = enter_count_gate(&first_hand_gates(&db, &st5, me));
    assert_follower_enter_count_gate(&gate5, 5);
    play(&db, &mut st5, hand5);
    let subjects5 = summoned_test_subjects(&st5, me);
    assert_eq!(subjects5.len(), 2);
    assert_eq!(subjects5, vec![(5, 5), (5, 5)]);
}

#[test]
fn enamored_fanfare_summons_match_enter_count_line_at_five() {
    let db = load_db();
    let me = PlayerId::A;
    let mut st = started(&db, 62);
    set_enter_count(&mut st, me, 5);
    give_pp(&mut st, me, 4, 10);
    st.player_mut(me).hand.clear();
    let hand = put_hand(&db, &mut st, me, "10932110");
    let gate = enter_count_gate(&first_hand_gates(&db, &st, me));
    assert_follower_enter_count_gate(&gate, 5);
    play(&db, &mut st, hand);
    let subjects = summoned_test_subjects(&st, me);
    assert_eq!(subjects.len(), 2);
    assert_eq!(subjects, vec![(5, 5), (5, 5)]);
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
        gates
            .iter()
            .all(|g| g.kind != "enterCountAtLeast" && g.kind != "enterCount"),
        "field Test Subject must not show enter-count lines: {gates:?}"
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

fn hand_yellow_at(db: &CardDb, card: &str, k: i32) -> bool {
    let me = PlayerId::A;
    let mut st = started(db, 90);
    set_enter_count(&mut st, me, k);
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _ = put_hand(db, &mut st, me, card);
    hand_yellow_decision(&first_hand_gates(db, &st, me))
}

#[test]
fn whole_pool_only_three_cards_gain_enter_count_line() {
    let db = load_db();
    let me = PlayerId::A;
    let mut with_enter_count = Vec::new();

    for card_id in db.cards.keys() {
        let id = card_id.as_str();
        let mut st = started(&db, 90);
        set_enter_count(&mut st, me, 5);
        give_pp(&mut st, me, 10, 10);
        st.player_mut(me).hand.clear();
        let _ = put_hand(&db, &mut st, me, &id);
        let gates = first_hand_gates(&db, &st, me);
        if gates.iter().any(|g| g.kind == "enterCount") {
            with_enter_count.push(id.to_string());
        }
    }

    with_enter_count.sort();
    let expected = [
        "10932110".to_string(),
        "10933110".to_string(),
        "10934110".to_string(),
    ];
    assert_eq!(with_enter_count, expected, "enterCount line card set");
}

#[test]
fn enter_count_lines_do_not_change_yellow_decision() {
    let db = load_db();
    for card in [
        TEST_SUBJECT,
        FOLLOWER_SUMMONERS[0],
        FOLLOWER_SUMMONERS[1],
        FOLLOWER_SUMMONERS[2],
        SPELL_SUMMONERS[0],
        SPELL_SUMMONERS[1],
    ] {
        let at_zero = hand_yellow_at(&db, card, 0);
        let at_five = hand_yellow_at(&db, card, 5);
        assert_eq!(
            at_zero, at_five,
            "{card} yellow decision changed with enter_counts"
        );
    }
}
