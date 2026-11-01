//! Set-10 Neutral soak: ≥200 seeded games of `set10-neutral` vs two meta decks.
//! Invariants: at most one Clue amulet per side; every field Clue has count ≥ 1.
//! Also a short h0 pass that must offer/choose Engage on a Clue amulet sometimes.

use arena_engine::{
    apply, hash, legal_actions, new_game, policy_rng, zone_count, Action, AnyPolicy, First,
    GameConfig, Phase, PlayerId, Policy, MAX_ACTIONS,
};

mod common;
use common::*;

fn assert_clue_invariants(st: &arena_engine::State) {
    for p in PlayerId::ALL {
        let clues: Vec<_> = st
            .player(p)
            .field
            .iter()
            .flatten()
            .filter(|c| c.is_clue())
            .collect();
        assert!(
            clues.len() <= 1,
            "{p:?}: at most one Clue amulet, got {}",
            clues.len()
        );
        for c in clues {
            assert!(
                c.clue_count >= 1,
                "{p:?}: Clue amulet count must be ≥ 1, got {}",
                c.clue_count
            );
        }
    }
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
        assert!(actions < MAX_ACTIONS, "seed={seed}: did not terminate");
        let h = hash(&state);
        let legal = legal_actions(db, &state);
        assert_eq!(hash(&state), h, "legal_actions mutated");
        assert!(!legal.is_empty(), "seed={seed}: empty legal");
        assert_clue_invariants(&state);
        let idx = policy.gen_range(legal.len() as u32) as usize;
        apply(db, &mut state, legal[idx].clone()).unwrap_or_else(|e| {
            panic!("seed={seed}: {e:?}");
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
    assert_clue_invariants(&state);
    actions
}

#[test]
fn set10_neutral_vs_meta_soak() {
    let n: u32 = std::env::var("ARENA_SET10_SOAK_GAMES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    let db = load_db();
    let set10 = load_deck_file("engine/tests/fixtures/decks/set10-neutral.json");
    assert!(deck_ready(&db, &set10));
    assert_eq!(set10.len(), 40);
    let meta_a = load_deck_file("oracle/decks/meta-sword-rally.json");
    let meta_b = load_deck_file("oracle/decks/meta-haven-kukishiro.json");
    assert!(deck_ready(&db, &meta_a) && deck_ready(&db, &meta_b));

    let start = std::time::Instant::now();
    let mut games = 0u32;
    let mut actions = 0u64;
    // Half vs each meta, both seatings.
    let per = (n / 4).max(1);
    for (i, meta) in [meta_a.as_slice(), meta_b.as_slice()].into_iter().enumerate() {
        for g in 0..per {
            let seed = 60_000 + (i as u64) * 1_000 + u64::from(g);
            actions += u64::from(play_pair(&db, seed, &set10, meta));
            games += 1;
            actions += u64::from(play_pair(&db, seed + 500, meta, &set10));
            games += 1;
        }
    }
    eprintln!(
        "set10_soak {games} games: mean_actions={} elapsed={:.2}s",
        actions / u64::from(games.max(1)),
        start.elapsed().as_secs_f64()
    );
    assert!(games >= 200, "need ≥ 200 games, got {games}");
}

#[test]
fn set10_h0_engages_clue_sometimes() {
    let db = load_db();
    let set10 = load_deck_file("engine/tests/fixtures/decks/set10-neutral.json");
    let meta = load_deck_file("oracle/decks/meta-sword-rally.json");
    let mut offered = 0u32;
    let mut chosen = 0u32;
    for g in 0..40u64 {
        let seed = 70_000 + g;
        let mut state = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: set10.clone(),
                deck_b: meta.clone(),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut a = match AnyPolicy::parse_spec("h0").expect("h0") {
            AnyPolicy::H0(h) => h,
            other => panic!("{other:?}"),
        };
        let mut b = match AnyPolicy::parse_spec("h0").expect("h0") {
            AnyPolicy::H0(h) => h,
            other => panic!("{other:?}"),
        };
        let mut rng = policy_rng(seed);
        let mut actions = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            assert!(actions < MAX_ACTIONS);
            assert_clue_invariants(&state);
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let me = arena_engine::acting_player(&state);
            for act in &legal {
                if let Action::Engage { slot } = act {
                    if state
                        .field_inst(me, slot.0)
                        .is_some_and(|c| c.is_clue())
                    {
                        offered += 1;
                    }
                }
            }
            let idx = match me {
                PlayerId::A => a.choose(&db, &state, &legal, &mut rng),
                PlayerId::B => b.choose(&db, &state, &legal, &mut rng),
            };
            let act = legal[idx.min(legal.len().saturating_sub(1))].clone();
            if let Action::Engage { slot } = &act {
                if state
                    .field_inst(me, slot.0)
                    .is_some_and(|c| c.is_clue())
                {
                    chosen += 1;
                }
            }
            apply(&db, &mut state, act).expect("h0 legal");
            actions += 1;
        }
    }
    eprintln!("set10_h0 clue engage offered={offered} chosen={chosen}");
    assert!(offered > 0, "Engage on a Clue amulet must be offered sometimes");
    assert!(chosen > 0, "h0 must choose Clue Engage sometimes");
}
