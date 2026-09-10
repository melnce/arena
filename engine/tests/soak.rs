//! Seeded random-legal soak. Opt-in via `ARENA_SOAK_GAMES` (CI sets 100).

use arena_engine::{
    apply, hash, legal_actions, new_game, policy_rng, zone_count, First, GameConfig, Phase,
    PlayerId, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

fn play_game(
    db: &arena_engine::CardDb,
    seed: u64,
    decks: &[arena_engine::CardId],
    n_cap: u32,
) -> SoakOut {
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: decks.to_vec(),
            deck_b: decks.to_vec(),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut policy = policy_rng(seed);
    let mut actions = 0u32;
    let start_account = zone_count(state.player(PlayerId::A));
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        if state.turn > MAX_TURNS || actions >= n_cap {
            break;
        }
        let h = hash(&state);
        let legal = legal_actions(db, &state);
        assert_eq!(hash(&state), h, "legal_actions mutated");
        if legal.is_empty() {
            break;
        }
        let idx = policy.gen_range(legal.len() as u32) as usize;
        let act = legal[idx].clone();
        apply(db, &mut state, act).expect("applied action was legal");
        actions += 1;
        for p in PlayerId::ALL {
            let pl = state.player(p);
            assert!(pl.hand.len() <= 9);
            assert!(pl.field_count() <= 5);
            assert!(pl.pp <= pl.pp_max + 1);
            assert!(pl.leader_defense <= pl.leader_max);
            let z = zone_count(pl);
            assert!(
                z + 64 >= start_account,
                "zone accounting collapsed unexpectedly"
            );
        }
    }
    SoakOut {
        actions,
        turns: state.turn,
        terminal: state.winner.is_some(),
    }
}

struct SoakOut {
    actions: u32,
    turns: u32,
    terminal: bool,
}

#[test]
fn soak_random_legal() {
    let n: u32 = std::env::var("ARENA_SOAK_GAMES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        eprintln!("soak skipped (set ARENA_SOAK_GAMES)");
        return;
    }
    let db = load_db();
    let decks = load_deck_file("engine/tests/fixtures/decks/basic-neutral-forest.json");
    assert!(deck_ready(&db, &decks), "basic-neutral-forest must load");
    let mut terminals = 0u32;
    let mut actions = 0u64;
    let mut turns = 0u64;
    for i in 0..n {
        let o = play_game(&db, 1000 + u64::from(i), &decks, MAX_ACTIONS);
        actions += u64::from(o.actions);
        turns += u64::from(o.turns);
        if o.terminal {
            terminals += 1;
        }
    }
    eprintln!(
        "soak {n} games: terminals={terminals} mean_actions={} mean_turns={}",
        actions / u64::from(n.max(1)),
        turns / u64::from(n.max(1))
    );
}

#[test]
fn soak_ramp_skips_when_missing() {
    let db = load_db();
    let decks = load_deck_file("engine/tests/fixtures/decks/ramp-dragon-37772.json");
    if deck_ready(&db, &decks) {
        let _ = play_game(&db, 7, &decks, 40);
    } else {
        eprintln!("skip: Ramp Dragon 37772 card files are not on this tree yet (RD partition)");
    }
}

#[test]
fn clone_replay_determinism_one_game() {
    let db = load_db();
    let decks = load_deck_file("engine/tests/fixtures/decks/basic-neutral-forest.json");
    let mut a = new_game(
        &db,
        GameConfig {
            seed: 99,
            deck_a: decks.clone(),
            deck_b: decks,
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let mut b = a.clone();
    let mut policy = policy_rng(99);
    for _ in 0..80 {
        if a.winner.is_some() {
            break;
        }
        let legal = legal_actions(&db, &a);
        if legal.is_empty() {
            break;
        }
        let idx = policy.gen_range(legal.len() as u32) as usize;
        let act = legal[idx].clone();
        apply(&db, &mut a, act.clone()).unwrap();
        apply(&db, &mut b, act).unwrap();
        assert_eq!(hash(&a), hash(&b));
    }
}
