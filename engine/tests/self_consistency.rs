//! Emit a trace and replay it — green for 50 seeds on the fixture deck we can build.

use arena_engine::{
    apply, from_neutral, legal_actions, new_game, policy_rng, snapshot_json, to_neutral, CardDb,
    First, GameConfig, GameRng, OpeningHands, Phase, PlayerId,
};

mod common;
use common::*;

fn emit_and_replay(db: &CardDb, seed: u64) {
    let decks = load_deck_file("engine/tests/fixtures/decks/basic-neutral-forest.json");
    let mut live = new_game(
        db,
        GameConfig {
            seed,
            deck_a: decks.clone(),
            deck_b: decks.clone(),
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let opening = OpeningHands {
        a: live
            .player(PlayerId::A)
            .hand
            .iter()
            .map(|c| c.card)
            .collect(),
        b: live
            .player(PlayerId::B)
            .hand
            .iter()
            .map(|c| c.card)
            .collect(),
    };
    let mut policy = policy_rng(seed);
    let mut recs = Vec::new();
    let mut i = 0u32;
    while live.winner.is_none() && !matches!(live.phase, Phase::Terminal) && i < 200 {
        let legal = legal_actions(db, &live);
        if legal.is_empty() {
            break;
        }
        let idx = policy.gen_range(legal.len() as u32) as usize;
        let action = legal[idx].clone();
        let neu = to_neutral(&live, &action);
        apply(db, &mut live, action).unwrap();
        recs.push((neu, live.picks.clone(), snapshot_json(&live)));
        i += 1;
    }

    let mut replay = new_game(
        db,
        GameConfig {
            seed,
            deck_a: decks.clone(),
            deck_b: decks,
            first: First::A,
            opening_hands: Some(opening),
        },
    )
    .unwrap();
    for (i, (neu, picks, snap)) in recs.iter().enumerate() {
        replay.rng = GameRng::scripted(picks.clone(), seed);
        let act = from_neutral(&replay, neu).expect("from_neutral");
        apply(db, &mut replay, act).unwrap_or_else(|e| panic!("replay seed={seed} i={i}: {e}"));
        let got = snapshot_json(&replay);
        if let Some((path, a, b)) = arena_engine::json_eq_first_diff(&got, snap, "") {
            panic!("seed {seed} i={i} {path}: arena={a} trace={b}");
        }
    }
}

#[test]
fn self_consistency_50_seeds() {
    let db = load_db();
    let decks = load_deck_file("engine/tests/fixtures/decks/basic-neutral-forest.json");
    assert!(deck_ready(&db, &decks));
    for seed in 1u64..=50 {
        emit_and_replay(&db, seed);
    }
}
