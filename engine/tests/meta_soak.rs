//! Meta-deck soak: every `oracle/decks/meta-*.json` deck plays random legal
//! actions to terminal without legal/apply mismatch. Fails on `main` when
//! Calge-Danthla Evolve is offered but apply returns Unsupported.

use arena_engine::{
    apply, hash, legal_actions, new_game, policy_rng, zone_count, First, GameConfig, Phase,
    PlayerId, MAX_ACTIONS,
};

mod common;
use common::*;

fn meta_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: serde_json::Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .collect()
}

fn play_pair(
    db: &arena_engine::CardDb,
    seed: u64,
    deck_a: &[arena_engine::CardId],
    deck_b: &[arena_engine::CardId],
) -> u32 {
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b: deck_b.to_vec(),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut policy = policy_rng(seed);
    let start_account = zone_count(state.player(PlayerId::A));
    let mut actions = 0u32;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        assert!(actions < MAX_ACTIONS, "game did not terminate");
        let h = hash(&state);
        let legal = legal_actions(db, &state);
        assert_eq!(hash(&state), h, "legal_actions mutated");
        assert!(!legal.is_empty(), "legal_actions empty before terminal");
        let idx = policy.gen_range(legal.len() as u32) as usize;
        let act = legal[idx].clone();
        apply(db, &mut state, act.clone()).unwrap_or_else(|e| {
            panic!("seed={seed} legal action failed: {e:?} act={act:?}");
        });
        actions += 1;
        for p in PlayerId::ALL {
            let pl = state.player(p);
            assert!(pl.hand.len() <= 9);
            assert!(pl.field_count() <= 5);
            let z = zone_count(pl);
            assert!(z + 64 >= start_account);
        }
    }
    actions
}

#[test]
fn meta_deck_soak_matrix() {
    let n: u32 = std::env::var("ARENA_META_SOAK_GAMES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        eprintln!("meta_soak skipped (set ARENA_META_SOAK_GAMES)");
        return;
    }
    let db = load_db();
    let stems = meta_stems();
    assert!(!stems.is_empty());
    let decks: Vec<Vec<arena_engine::CardId>> = stems
        .iter()
        .map(|s| load_deck_file(format!("oracle/decks/{s}.json")))
        .collect();
    for d in &decks {
        assert!(deck_ready(&db, d), "meta deck must load");
        assert_eq!(d.len(), 40);
    }

    let start = std::time::Instant::now();
    let mut games = 0u32;
    let mut actions = 0u64;
    for (i, da) in decks.iter().enumerate() {
        for (j, dbk) in decks.iter().enumerate() {
            if i == j {
                continue;
            }
            for g in 0..n {
                let seed = 50_000 + (i as u64) * 1_000 + (j as u64) * 10 + u64::from(g);
                let a = play_pair(&db, seed, da, dbk);
                actions += u64::from(a);
                games += 1;
            }
        }
    }
    let elapsed = start.elapsed();
    eprintln!(
        "meta_soak {games} games ({} decks, N={n}): mean_actions={} elapsed={:.2}s",
        stems.len(),
        actions / u64::from(games.max(1)),
        elapsed.as_secs_f64()
    );
}
