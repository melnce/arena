//! Recorder fixtures: ε = 0 identity, one sample per decision, labels, ε = 1.

use arena_engine::{
    acting_player, apply, legal_actions, new_game, play_game, policy_rng, Action, AnyPolicy,
    CardId, End, First, FirstLegal, GameConfig, Observation, Outcome, Phase, PlayerId, Policy,
    Random, Recorder, H0, MAX_TURNS,
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
#[cfg_attr(debug_assertions, ignore)]
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

fn useful_local(state: &arena_engine::State, me: PlayerId, a: &Action) -> bool {
    match a {
        Action::BonusPp => !state.player(me).bonus_pp.active,
        _ => true,
    }
}

fn check_search_v(spec: &str, seeds: impl IntoIterator<Item = u64>) {
    for seed in seeds {
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
        );
        let mut b = Recorder::new(
            Box::new(AnyPolicy::parse_spec(spec).expect("spec b")),
            PlayerId::B,
        );
        let mut rng = policy_rng(seed);
        play_game(&db, &mut state, &mut a, &mut b, &mut rng);

        // Replay the same seed to classify each decision (single-candidate).
        let mut state = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: forest(),
                deck_b: forest(),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut ia = 0usize;
        let mut ib = 0usize;
        let mut rng = policy_rng(seed);
        let mut pa = AnyPolicy::parse_spec(spec).expect("spec a");
        let mut pb = AnyPolicy::parse_spec(spec).expect("spec b");
        let mut actions = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            if state.turn > MAX_TURNS {
                break;
            }
            if actions >= arena_engine::MAX_ACTIONS {
                break;
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let me = acting_player(&state);
            let mull = matches!(state.phase, Phase::Mulligan { .. });
            let useful = legal.iter().filter(|a| useful_local(&state, me, a)).count();
            let idx = match me {
                PlayerId::A => pa.choose(&db, &state, &legal, &mut rng),
                PlayerId::B => pb.choose(&db, &state, &legal, &mut rng),
            };
            let sample = match me {
                PlayerId::A => {
                    let s = &a.samples()[ia];
                    ia += 1;
                    s
                }
                PlayerId::B => {
                    let s = &b.samples()[ib];
                    ib += 1;
                    s
                }
            };
            if legal.len() <= 1 || mull {
                assert!(
                    sample.search_v.is_nan(),
                    "{spec} seed={seed} legal_len={} phase={} expected NaN, got {}",
                    legal.len(),
                    sample.phase,
                    sample.search_v
                );
            } else if useful >= 2 {
                assert!(
                    sample.search_v.is_finite(),
                    "{spec} seed={seed} expected finite search_v, got {}",
                    sample.search_v
                );
                assert!(
                    (-80.0..=80.0).contains(&sample.search_v),
                    "{spec} seed={seed} search_v {} outside [-80, 80]",
                    sample.search_v
                );
            } else {
                assert!(
                    sample.search_v.is_nan(),
                    "{spec} seed={seed} single-candidate expected NaN, got {}",
                    sample.search_v
                );
            }
            let idx = idx.min(legal.len().saturating_sub(1));
            apply(&db, &mut state, legal[idx].clone()).expect("apply");
            actions += 1;
        }
    }
}

#[test]
fn search_v_h0_fast_twenty() {
    check_search_v("h0-fast", 1u64..=20);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn search_v_h0_one() {
    check_search_v("h0", [1u64]);
}

#[test]
#[ignore]
fn search_v_h0_three() {
    check_search_v("h0", 1u64..=3);
}

#[test]
fn search_v_random_is_nan() {
    for seed in 1u64..=5 {
        let (out, a, b) = play_recorded_random(seed);
        let _ = out;
        for s in a.samples().iter().chain(b.samples()) {
            assert!(
                s.search_v.is_nan(),
                "Random recorder must write NaN, got {}",
                s.search_v
            );
        }
    }
}

fn play_recorded_random(seed: u64) -> (Outcome, Recorder, Recorder) {
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
    let mut a = Recorder::new(Box::new(Random), PlayerId::A);
    let mut b = Recorder::new(Box::new(Random), PlayerId::B);
    let mut rng = policy_rng(seed);
    let out = play_game(&db, &mut state, &mut a, &mut b, &mut rng);
    (out, a, b)
}

#[test]
fn last_value_consensus_lethal() {
    let db = load_db();
    let mut st = started_decks(&db, 11, &["10461110"], &["88001110"]);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let slot = put_field(&db, &mut st, PlayerId::A, "10461110");
    {
        let f = st.field_inst_mut(PlayerId::A, slot).unwrap();
        f.flags.summoning_sick = false;
        f.flags.attacks_left = 1;
    }
    st.player_mut(PlayerId::B).leader_defense = 2;
    let legal = legal_actions(&db, &st);
    assert!(
        legal.len() >= 2,
        "need a search (got {} legal)",
        legal.len()
    );
    let mut h0 = H0::default();
    let mut rng = policy_rng(1);
    h0.choose(&db, &st, &legal, &mut rng);
    assert_eq!(h0.last_value(), Some(80.0));
}

#[test]
fn last_value_midgame_deterministic() {
    let db = load_db();
    let deck = forest();
    let mut state = new_game(
        &db,
        GameConfig {
            seed: 11,
            deck_a: deck.clone(),
            deck_b: deck,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut fa = H0::fast();
    let mut fb = H0::fast();
    let mut rng = policy_rng(11);
    for _ in 0..80 {
        if state.winner.is_some() || matches!(state.phase, Phase::Terminal) {
            break;
        }
        let legal = legal_actions(&db, &state);
        if legal.is_empty() {
            break;
        }
        if legal.len() >= 2 && !matches!(state.phase, Phase::Mulligan { .. }) {
            let snap = state.clone();
            let mut h1 = H0::default();
            let mut r1 = policy_rng(99);
            h1.choose(&db, &snap, &legal, &mut r1);
            let v1 = h1.last_value();
            let mut h2 = H0::default();
            let mut r2 = policy_rng(99);
            h2.choose(&db, &snap, &legal, &mut r2);
            let v2 = h2.last_value();
            if let Some(v) = v1 {
                assert!(v.is_finite(), "last_value not finite: {v}");
                assert_eq!(v1, v2, "last_value must be deterministic");
                return;
            }
        }
        let me = acting_player(&state);
        let idx = match me {
            PlayerId::A => fa.choose(&db, &state, &legal, &mut rng),
            PlayerId::B => fb.choose(&db, &state, &legal, &mut rng),
        };
        let idx = idx.min(legal.len().saturating_sub(1));
        apply(&db, &mut state, legal[idx].clone()).expect("apply");
    }
    panic!("no mid-game search decision with a finite last_value");
}
