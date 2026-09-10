//! Opt-in throughput floor, same shape as soak (`ARENA_BENCH_MIN_GAMES_PER_SEC`).
//! CI sets the floor so a 9× `when`-scan regression fails and a 1.5× wobble does not.

use std::time::Instant;

use arena_engine::{
    apply, legal_actions, new_game, policy_rng, First, GameConfig, Phase, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

fn play_game(
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
    let mut actions = 0u32;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        if state.turn > MAX_TURNS || actions >= MAX_ACTIONS {
            break;
        }
        let legal = legal_actions(db, &state);
        if legal.is_empty() {
            break;
        }
        let idx = policy.gen_range(legal.len() as u32) as usize;
        if apply(db, &mut state, legal[idx].clone()).is_err() {
            break;
        }
        actions += 1;
    }
    actions
}

#[test]
fn ramp_mirror_throughput_floor() {
    let floor: f64 = std::env::var("ARENA_BENCH_MIN_GAMES_PER_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    if floor <= 0.0 {
        eprintln!("throughput skipped (set ARENA_BENCH_MIN_GAMES_PER_SEC)");
        return;
    }
    let db = load_db();
    let decks = load_deck_file("oracle/decks/ramp-37772.json");
    assert!(
        deck_ready(&db, &decks),
        "oracle/decks/ramp-37772.json must load"
    );
    const GAMES: u32 = 200;
    const SEED: u64 = 7;
    let t0 = Instant::now();
    let mut actions = 0u64;
    for i in 0..GAMES {
        actions += u64::from(play_game(
            &db,
            SEED.wrapping_add(u64::from(i)),
            &decks,
            &decks,
        ));
    }
    let secs = t0.elapsed().as_secs_f64().max(1e-9);
    let gps = f64::from(GAMES) / secs;
    eprintln!(
        "throughput {GAMES} ramp-mirror games: {gps:.1} games/s, {:.0} actions/s (floor {floor})",
        actions as f64 / secs
    );
    assert!(
        gps >= floor,
        "ramp mirror {gps:.1} games/s below floor {floor}"
    );
}

fn pool_throughput_floor(path: &str, label: &str) {
    let floor: f64 = std::env::var("ARENA_BENCH_MIN_GAMES_PER_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    if floor <= 0.0 {
        eprintln!("throughput skipped (set ARENA_BENCH_MIN_GAMES_PER_SEC)");
        return;
    }
    let db = load_db();
    let decks = load_deck_file(path);
    assert!(deck_ready(&db, &decks), "{path} must load");
    const GAMES: u32 = 200;
    const SEED: u64 = 7;
    let t0 = Instant::now();
    let mut actions = 0u64;
    for i in 0..GAMES {
        actions += u64::from(play_game(
            &db,
            SEED.wrapping_add(u64::from(i)),
            &decks,
            &decks,
        ));
    }
    let secs = t0.elapsed().as_secs_f64().max(1e-9);
    let gps = f64::from(GAMES) / secs;
    eprintln!(
        "throughput {GAMES} {label} games: {gps:.1} games/s, {:.0} actions/s (floor {floor})",
        actions as f64 / secs
    );
    assert!(gps >= floor, "{label} {gps:.1} games/s below floor {floor}");
}

#[test]
fn sword_pool_throughput_floor() {
    pool_throughput_floor("oracle/decks/sword-pool.json", "sword-pool");
}

#[test]
fn forest_pool_throughput_floor() {
    pool_throughput_floor("oracle/decks/forest-pool.json", "forest-pool");
}
