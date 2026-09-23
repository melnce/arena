//! `lcap` (consensus-lethal budget) and `clip` (learned-leaf input clamp).

use arena_engine::policy::{builtin_net, ChoosePath};
use arena_engine::{
    apply, encode_with_vocab, legal_actions, new_game, policy_rng, vocab, AnyPolicy, Phase, Policy,
    H0,
};

mod common;
use common::*;

fn collect_states(
    db: &arena_engine::CardDb,
    n: usize,
    midgame_only: bool,
) -> Vec<arena_engine::State> {
    let decks = load_pinned_corpus_decks_sorted(db);
    let mut out = Vec::with_capacity(n);
    let mut seed = 1u64;
    while out.len() < n {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            db,
            arena_engine::GameConfig {
                seed,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: arena_engine::First::A,
                opening_hands: None,
            },
        ) else {
            seed += 1;
            continue;
        };
        let mut rng = policy_rng(seed);
        let mut nact = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            let take = !midgame_only || (state.turn >= 3 && matches!(state.phase, Phase::Main));
            if take {
                out.push(state.clone());
                if out.len() >= n {
                    break;
                }
            }
            if state.turn > arena_engine::MAX_TURNS || nact >= arena_engine::MAX_ACTIONS {
                break;
            }
            let legal = legal_actions(db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
            nact += 1;
        }
        seed += 1;
        if seed > 50_000 {
            break;
        }
    }
    out
}

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn choose_on_states(spec: &str, states: &[arena_engine::State]) -> (u64, u64) {
    let db = load_db();
    let mut h0 = parse_h0(spec);
    let mut unscored = 0u64;
    let mut lethal_nodes = 0u64;
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        h0.reset_stats();
        let seed = 20260927u64.wrapping_add(i as u64);
        let mut rng = policy_rng(seed);
        let _ = h0.choose(&db, state, &legal, &mut rng);
        unscored += h0.stats.unscored;
        lethal_nodes += h0.stats.lethal_nodes;
        let budget = (h0.lcap * h0.node_cap as f32).floor() as u32;
        assert!(
            h0.stats.lethal_nodes <= u64::from(budget),
            "{spec} lethal_nodes {} > budget {} at state {i}",
            h0.stats.lethal_nodes,
            budget
        );
    }
    (unscored, lethal_nodes)
}

fn count_unscored_via_explain(spec: &str, states: &[arena_engine::State]) -> u64 {
    let db = load_db();
    let mut h0 = parse_h0(spec);
    let mut n = 0u64;
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        h0.reset_stats();
        h0.arm_explain();
        let seed = 20260927u64.wrapping_add(i as u64);
        let mut rng = policy_rng(seed);
        let _ = h0.choose(&db, state, &legal, &mut rng);
        let rec = h0.take_explain().expect("explain");
        if rec.path == ChoosePath::Unscored {
            n += 1;
            assert_eq!(h0.stats.unscored, 1, "stats counter at state {i}");
        } else {
            assert_eq!(h0.stats.unscored, 0, "stats counter at state {i}");
        }
    }
    n
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn lcap_default_has_unscored_midgame() {
    let db = load_db();
    let states = collect_states(&db, 200, true);
    assert_eq!(states.len(), 200, "could not reach 200 mid-game states");
    let unscored = count_unscored_via_explain("h0", &states);
    assert!(
        unscored >= 1,
        "expected at least one unscored decision with default h0"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn lcap_fraction_eliminates_unscored() {
    let db = load_db();
    let states = collect_states(&db, 200, true);
    assert_eq!(states.len(), 200);
    for spec in ["h0:lcap=0.5", "h0:lcap=0.25"] {
        let unscored = count_unscored_via_explain(spec, &states);
        assert_eq!(unscored, 0, "{spec} should have zero unscored");
        let (_, _) = choose_on_states(spec, &states);
    }
}

fn model_stats() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let model: serde_json::Value =
        serde_json::from_str(include_str!("../models/h0-linear-v1.json")).expect("model");
    let mean = model["feat_mean"]
        .as_array()
        .expect("mean")
        .iter()
        .map(|x| x.as_f64().expect("f") as f32)
        .collect();
    let std = model["feat_std"]
        .as_array()
        .expect("std")
        .iter()
        .map(|x| x.as_f64().expect("f") as f32)
        .collect();
    let w = model["linear"]["w"]
        .as_array()
        .expect("w")
        .iter()
        .map(|x| x.as_f64().expect("f") as f32)
        .collect();
    (mean, std, w)
}

fn std_inputs(mean: &[f32], std: &[f32], obs: &arena_engine::Observation) -> Vec<f32> {
    obs.features
        .iter()
        .zip(mean.iter())
        .zip(std.iter())
        .map(|((f, m), s)| (*f - *m) / *s)
        .collect()
}

#[test]
fn clip_matches_unclipped_when_inputs_in_range() {
    let db = load_db();
    let states = collect_states(&db, 200, true);
    assert_eq!(states.len(), 200);
    let net = builtin_net();
    let (mean, std, _) = model_stats();
    let mut checked = 0u32;
    for state in &states {
        for me in [arena_engine::PlayerId::A, arena_engine::PlayerId::B] {
            let v = vocab(state);
            let obs = encode_with_vocab(state, me, &v);
            let xs = std_inputs(&mean, &std, &obs);
            if xs.iter().all(|x| (-5.0..=5.0).contains(x)) {
                let plain = net.value(&obs);
                let clipped = net.value_clipped(&obs, 5.0);
                assert_eq!(plain.to_bits(), clipped.to_bits());
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "expected some in-range observations");
}

#[test]
fn clip_limits_choice_indicator_spike() {
    let db = load_db();
    let states = collect_states(&db, 200, true);
    let net = builtin_net();
    let (_, _, weights) = model_stats();
    let w18 = weights[18];
    let max_delta = 60.0 * 5.0 * w18.abs();

    let mut found = false;
    for state in &states {
        for me in [arena_engine::PlayerId::A, arena_engine::PlayerId::B] {
            let v = vocab(state);
            let obs = encode_with_vocab(state, me, &v);
            let base = net.value(&obs);
            if base.abs() > 20.0 {
                continue;
            }
            let mut perturbed = obs.clone();
            perturbed.features[18] = 1.0;
            let unclipped = net.value(&perturbed);
            let delta = (unclipped - base).abs();
            if delta <= 10.0 {
                continue;
            }
            let clipped = net.value_clipped(&perturbed, 5.0);
            let clipped_delta = (clipped - base).abs();
            assert!(
                clipped_delta <= max_delta + 1e-3,
                "clipped delta {clipped_delta} > {max_delta}"
            );
            assert!(delta > 10.0);
            found = true;
            break;
        }
        if found {
            break;
        }
    }
    assert!(found, "expected a perturbation with unclipped delta > 10");
}

#[test]
fn spec_round_trips_lcap_clip() {
    for s in ["h0:lcap=0.5", "h0:clip=5", "h0:lcap=0.25,clip=5"] {
        let parsed = AnyPolicy::parse_spec(s).unwrap_or_else(|e| panic!("{s}: {e}"));
        let again = AnyPolicy::parse_spec(&parsed.spec())
            .unwrap_or_else(|e| panic!("{}: {e}", parsed.spec()));
        assert_eq!(parsed, again, "{s} → {}", parsed.spec());
    }
    assert_eq!(
        AnyPolicy::parse_spec("h0:lcap=1,clip=0").unwrap().spec(),
        "h0"
    );
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");

    let e = AnyPolicy::parse_spec("h0:lcap=0").unwrap_err();
    assert!(e.contains("lcap"), "{e}");
    let e = AnyPolicy::parse_spec("h0:lcap=1.5").unwrap_err();
    assert!(e.contains("lcap"), "{e}");
    let e = AnyPolicy::parse_spec("h0:clip=-1").unwrap_err();
    assert!(e.contains("clip"), "{e}");
    let e = AnyPolicy::parse_spec("h0:lcap=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

#[test]
fn fast_keeps_lcap_clip_defaults() {
    let fast = H0::fast();
    let def = H0::default();
    assert_eq!(fast.lcap, def.lcap);
    assert_eq!(fast.clip, def.clip);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn default_bench_identity_seed5() {
    let db = load_db();
    let decks = (
        load_deck_file("oracle/decks/meta-rune-test-subject.json"),
        load_deck_file("oracle/decks/meta-sword-rally.json"),
    );
    if !deck_ready(&db, &decks.0) || !deck_ready(&db, &decks.1) {
        return;
    }
    let seed = 5u64;
    let games = 4u32;
    let mut a_wins = 0u32;
    let mut b_wins = 0u32;
    let mut stats = arena_engine::policy::SearchStats::default();
    for g in 0..games {
        let s = seed.wrapping_add(g as u64);
        let mut state = new_game(
            &db,
            arena_engine::GameConfig {
                seed: s,
                deck_a: decks.0.clone(),
                deck_b: decks.1.clone(),
                first: arena_engine::First::A,
                opening_hands: None,
            },
        )
        .unwrap();
        let mut rng = policy_rng(s);
        let mut pol_a = parse_h0("h0");
        let mut pol_b = parse_h0("h0:value=v0");
        let out = arena_engine::play_game(&db, &mut state, &mut pol_a, &mut pol_b, &mut rng);
        stats.add(&pol_a.stats);
        if out.winner == Some(arena_engine::PlayerId::A) {
            a_wins += 1;
        }
        if out.winner == Some(arena_engine::PlayerId::B) {
            b_wins += 1;
        }
    }
    assert_eq!(a_wins, 1, "a_wins baseline");
    assert_eq!(b_wins, 3, "b_wins baseline");
    assert_eq!(stats.decisions, 203, "decisions baseline");
    assert_eq!(stats.nodes, 227_933, "nodes baseline");
    assert_eq!(stats.cap_hits, 34, "cap_hits baseline");
    assert_eq!(stats.pairs_skipped, 0, "pairs_skipped baseline");
    assert_eq!(stats.candidates, 787, "candidates baseline");
    assert_eq!(stats.roots, 660, "roots baseline");
    assert_eq!(stats.opp_leaves, 10_163, "opp_leaves baseline");
    assert_eq!(stats.opp_cap_hits, 1_228, "opp_cap_hits baseline");
    assert_eq!(stats.tt_hits, 3_795, "tt_hits baseline");
    assert_eq!(stats.tt_stores, 32_573, "tt_stores baseline");
    assert_eq!(
        stats.opp_lethal_checks, 10_163,
        "opp_lethal_checks baseline"
    );
    assert_eq!(stats.opp_lethal_found, 4_858, "opp_lethal_found baseline");
    assert_eq!(
        stats.opp_lethal_evo_found, 312,
        "opp_lethal_evo_found baseline"
    );
    assert_eq!(stats.opp_lethal_nodes, 51_554, "opp_lethal_nodes baseline");
    assert_eq!(
        stats.chose_with_lethal_root, 21,
        "chose_with_lethal_root baseline"
    );
    assert_eq!(
        stats.cands_with_lethal_root, 143,
        "cands_with_lethal_root baseline"
    );
    // New counters only — must not change play.
    assert_eq!(stats.lethal_nodes, 9_783, "lethal_nodes tally");
    assert_eq!(stats.unscored, 0, "unscored tally");
}
