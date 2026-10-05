//! Timing sanity for turn-world keys (ignored; run manually).

use std::time::Instant;

use arena_engine::{
    apply, legal_actions, new_game, policy_rng, AnyPolicy, CardDb, First, GameConfig, Phase,
    Policy, H0,
};

mod common;
use common::*;

const SERVED: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn midgame_states(db: &CardDb, n: usize) -> Vec<arena_engine::State> {
    let decks = load_pinned_corpus_decks_sorted(db);
    let mut out = Vec::new();
    let mut seed = 100u64;
    while out.len() < n && seed < 10_000 {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            db,
            GameConfig {
                seed,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: First::A,
                opening_hands: None,
            },
        ) else {
            seed += 1;
            continue;
        };
        let mut rng = policy_rng(seed);
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            if state.turn >= 4 && matches!(state.phase, Phase::Main) {
                out.push(state.clone());
                if out.len() >= n {
                    break;
                }
            }
            let legal = legal_actions(db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
        }
        seed += 1;
    }
    out
}

#[test]
#[ignore]
fn timing_sanity_turn_world_keys() {
    let db = load_db();
    let states = midgame_states(&db, 5);
    assert_eq!(states.len(), 5);
    let specs = [
        ("served", SERVED),
        ("wseed=turn", &format!("{SERVED},wseed=turn")),
        ("lostrank=2000", &format!("{SERVED},lostrank=2000")),
    ];
    for (label, spec) in specs {
        let mut total_ms = 0u128;
        for (i, state) in states.iter().enumerate() {
            let legal = legal_actions(&db, state);
            if legal.len() <= 1 {
                continue;
            }
            let mut h0 = parse_h0(spec);
            let mut rng = policy_rng(5000 + i as u64);
            let start = Instant::now();
            let _ = h0.choose(&db, state, &legal, &mut rng);
            total_ms += start.elapsed().as_millis();
        }
        println!("{label}: {:.1} ms/decision", total_ms as f64 / 5.0);
    }
}
