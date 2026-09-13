//! Recorder fixtures: ε = 0 identity, one sample per decision, labels, ε = 1.

use arena_engine::{
    acting_player, legal_actions, new_game, play_game, policy_rng, AnyPolicy, CardId, End, First,
    FirstLegal, GameConfig, Observation, Outcome, PlayerId, Policy, Recorder, MAX_TURNS,
};

mod common;
use common::*;

fn forest() -> Vec<CardId> {
    load_deck_file(repo_root().join("oracle/decks/basic-forest.json"))
}

fn play_plain(spec: &str, seed: u64) -> Outcome {
    let db = load_db();
    let deck = forest();
    let mut state = new_game(
        &db,
        GameConfig {
            seed,
            deck_a: deck.clone(),
            deck_b: deck,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut a = AnyPolicy::parse_spec(spec).expect("spec a");
    let mut b = AnyPolicy::parse_spec(spec).expect("spec b");
    let mut rng = policy_rng(seed);
    play_game(&db, &mut state, &mut a, &mut b, &mut rng)
}

fn play_recorded(spec: &str, seed: u64, epsilon: f32) -> (Outcome, Recorder, Recorder) {
    let db = load_db();
    let deck = forest();
    let mut state = new_game(
        &db,
        GameConfig {
            seed,
            deck_a: deck.clone(),
            deck_b: deck,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut a = Recorder::new(
        Box::new(AnyPolicy::parse_spec(spec).expect("spec a")),
        PlayerId::A,
    )
    .with_epsilon(epsilon);
    let mut b = Recorder::new(
        Box::new(AnyPolicy::parse_spec(spec).expect("spec b")),
        PlayerId::B,
    )
    .with_epsilon(epsilon);
    let mut rng = policy_rng(seed);
    let out = play_game(&db, &mut state, &mut a, &mut b, &mut rng);
    (out, a, b)
}

fn check_identity(spec: &str, seeds: impl IntoIterator<Item = u64>) {
    for seed in seeds {
        let plain = play_plain(spec, seed);
        let (rec, mut a, mut b) = play_recorded(spec, seed, 0.0);
        assert_eq!(plain, rec, "ε = 0 must match plain {spec} seed={seed}");
        assert_eq!(plain.winner, rec.winner);
        assert_eq!(plain.turns, rec.turns);
        assert_eq!(plain.actions, rec.actions);
        assert_eq!(plain.end, rec.end);
        a.label(&rec);
        b.label(&rec);
        check_one_sample_per_decision(&rec, &a, &b);
        check_labels(&rec, &a, &b);
    }
}

fn phase_one_hot_ok(features: &[f32], phase: u8) -> bool {
    let ones: Vec<usize> = (0..5).filter(|&i| features[i] == 1.0).collect();
    ones.len() == 1
        && ones[0] == phase as usize
        && (0..5).all(|i| features[i] == 0.0 || features[i] == 1.0)
}

fn check_one_sample_per_decision(out: &Outcome, a: &Recorder, b: &Recorder) {
    assert_eq!(
        a.len() + b.len(),
        out.actions as usize,
        "one sample per decision"
    );
    for rec in [a, b] {
        for (i, s) in rec.samples().iter().enumerate() {
            assert_eq!(s.features.len(), Observation::LEN);
            assert_eq!(s.ids.len(), Observation::IDS_LEN);
            assert_eq!(s.features[7], 0.0, "winner slot is 0 at decision time");
            assert!(
                phase_one_hot_ok(&s.features, s.phase),
                "phase one-hot {} vs sample.phase {}",
                s.features[0],
                s.phase
            );
            assert!(
                s.chosen < s.legal_len,
                "chosen {} < legal_len {}",
                s.chosen,
                s.legal_len
            );
            assert_eq!(s.decision_index, i as u32);
        }
    }
}

fn check_labels(out: &Outcome, a: &Recorder, b: &Recorder) {
    let expect = |me: PlayerId| match out.winner {
        Some(w) if w == me => 1.0,
        Some(_) => -1.0,
        None => 0.0,
    };
    for s in a.samples() {
        assert_eq!(s.label, expect(PlayerId::A));
    }
    for s in b.samples() {
        assert_eq!(s.label, expect(PlayerId::B));
    }
}

#[test]
fn identity_epsilon_zero_h0_fast() {
    check_identity("h0-fast", 1u64..=20);
}

#[test]
fn identity_epsilon_zero_h0_one() {
    check_identity("h0", [1u64]);
}

#[test]
#[ignore]
fn identity_epsilon_zero_h0_three() {
    check_identity("h0", 1u64..=3);
}

#[test]
fn one_sample_per_decision_and_labels() {
    let (out, mut a, mut b) = play_recorded("h0-fast", 7, 0.0);
    a.label(&out);
    b.label(&out);
    check_one_sample_per_decision(&out, &a, &b);
    check_labels(&out, &a, &b);
}

#[test]
fn draw_via_turn_cap_labels_zero() {
    // Drive a handful of decisions so both seats have samples, then trip the
    // turn cap (`state.turn > MAX_TURNS`) so `winner` stays `None`.
    let db = load_db();
    let deck = forest();
    let mut state = new_game(
        &db,
        GameConfig {
            seed: 3,
            deck_a: deck.clone(),
            deck_b: deck,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut a = Recorder::new(Box::new(FirstLegal), PlayerId::A);
    let mut b = Recorder::new(Box::new(FirstLegal), PlayerId::B);
    let mut rng = policy_rng(3);
    for _ in 0..12 {
        if state.winner.is_some() {
            break;
        }
        let legal = legal_actions(&db, &state);
        if legal.is_empty() {
            break;
        }
        let idx = match acting_player(&state) {
            PlayerId::A => a.choose(&db, &state, &legal, &mut rng),
            PlayerId::B => b.choose(&db, &state, &legal, &mut rng),
        };
        let idx = idx.min(legal.len().saturating_sub(1));
        arena_engine::apply(&db, &mut state, legal[idx].clone()).expect("apply");
    }
    assert!(
        a.len() + b.len() > 0,
        "need samples before forcing the turn cap"
    );
    state.turn = MAX_TURNS + 1;
    let out = play_game(&db, &mut state, &mut a, &mut b, &mut rng);
    assert_eq!(out.end, End::TurnCap);
    assert_eq!(out.winner, None);
    a.label(&out);
    b.label(&out);
    assert!(a.samples().iter().all(|s| s.label == 0.0));
    assert!(b.samples().iter().all(|s| s.label == 0.0));
}

#[test]
fn epsilon_one_is_random_and_changes_outcomes() {
    let mut zero = Vec::new();
    let mut ones = Vec::new();
    for seed in 1u64..=20 {
        let (out0, a0, b0) = play_recorded("h0-fast", seed, 0.0);
        let (out1, a1, b1) = play_recorded("h0-fast", seed, 1.0);
        assert!(
            a1.samples().iter().chain(b1.samples()).all(|s| s.random),
            "ε = 1: every sample is random"
        );
        assert!(
            a0.samples().iter().chain(b0.samples()).all(|s| !s.random),
            "ε = 0: no sample is random"
        );
        zero.push(out0);
        ones.push(out1);
    }
    assert_ne!(zero, ones, "ε = 1 must play different games than ε = 0");
}
