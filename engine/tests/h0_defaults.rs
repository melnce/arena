//! Gate tests for the h0 default flip (built-in mulligan table + info=open).
//!
//! ## Legacy fingerprint procedure
//!
//! Pinned values in `LEGACY_FINGERPRINTS` were captured on **`main` at commit
//! `2e40d4d`** with plain `h0` (pre-flip defaults: `mull=rule`, `info=fair`).
//! Reproduce:
//!
//! ```text
//! git checkout 2e40d4d
//! # apply this file's FNV-1a `action_fingerprint` helper and run:
//! cargo test --release --test h0_defaults print_legacy_fingerprints -- --nocapture
//! ```
//!
//! On this branch, `h0:mull=rule,info=fair` must match those pins; bare `h0`
//! must match `h0:mull=engine/models/mulligan-v1.json,info=open`.
//!
//! `print_legacy_fingerprints` uses `h0:mull=rule,info=fair` (not bare `h0`).
//! On `main@3ebef26` the helper mistakenly called `action_fingerprint` with
//! `"h0"`, so legacy pins there were mislabeled; this branch fixes that
//! copy-paste bug.
//!
//! ## Play-time + Effect::Pay re-pins (`main@3ebef26` → branch)
//!
//! Proof workflow: `dump_gate_hashes` / `dump_gate_actions` on this branch vs
//! `git worktree` at `main@3ebef26`; canonical `hash(state)` matches at every
//! step before the cited `n`, then diverges on a play whose play-time walk
//! (mode / discard / destroy picks, or `Effect::Pay` gating) locks before play
//! reactions. When h0's first neutral-action diff is later, it is a search
//! cascade from that engine step (same pre-hash, different simulated play line).
//!
//! | spec | seed | deck | old → new | n | engine hash first diff | play-time step |
//! |------|------|------|-----------|---|------------------------|----------------|
//! | default | 37 | meta-dragon-aggro | `0xe0ff…` → `0x0c5e…` | 88 | 79 | Spilling Red `10642310` play + picks before attack |
//! | default | 41 | meta-dragon-ramp | `0x9e54…` → `0x09d1…` | 47 | 17 | Yidmetra `90024320` faith tick after Enhanced pick; cascade → Burnite `10744110` vs Lyria `10403120` |
//! | default | 53 | meta-forest-combo | `0x7404…` → `0x7393…` | 86 | 37 | Miroku `10514120` mode pick at play; cascade → attack vs play |
//! | default | 67 | meta-haven-amulet | `0x50a9…` → `0x3705…` | 20 | 21 | Timepiece `10762210` play vs attack (sim line differs) |
//! | legacy | 37 | meta-dragon-aggro | `0xe0ff…` → `0x0c5e…` | 88 | 79 | same Spilling Red row |
//! | legacy | 41 | meta-dragon-ramp | `0xc4e8…` → `0x3207…` | 51 | 52 | Lumiore `10844120` discard pick (`10744110` vs `10042310`) |
//! | legacy | 53 | meta-forest-combo | `0x7404…` → `0x7393…` | 86 | 37 | same Miroku row |
//! | legacy | 67 | meta-haven-amulet | `0xdfee…` → `0x00cc…` | 87 | 41 | cascade → Lyanthoth `10664120` play+picks vs engage |

use arena_engine::{
    apply, hash, legal_actions, new_game, play_game, policy_rng, to_neutral, trace::fnv1a64,
    AnyPolicy, CardDb, First, GameConfig, PlayerId, Policy,
};

mod common;
use common::*;

const GATE_SEEDS: [u64; 8] = [11, 23, 37, 41, 53, 67, 79, 97];

fn parse_h0(spec: &str) -> arena_engine::H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn meta_deck_stems() -> Vec<String> {
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

fn load_meta_deck(stem: &str) -> Vec<arena_engine::CardId> {
    load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")))
}

/// FNV-1a 64 over little-endian action count plus each neutral-action JSON blob.
fn action_fingerprint(db: &CardDb, spec: &str, seed: u64, deck_a: &[arena_engine::CardId]) -> u64 {
    let deck_b = load_meta_deck("meta-sword-rally");
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut a = parse_h0(spec);
    let mut b = parse_h0(spec);
    let mut rng = policy_rng(seed);
    let mut serialized = Vec::new();
    while state.winner.is_none() && !matches!(state.phase, arena_engine::Phase::Terminal) {
        let legal = legal_actions(db, &state);
        if legal.is_empty() {
            break;
        }
        let me = arena_engine::acting_player(&state);
        let idx = match me {
            PlayerId::A => a.choose(db, &state, &legal, &mut rng),
            PlayerId::B => b.choose(db, &state, &legal, &mut rng),
        };
        let action = legal[idx.min(legal.len().saturating_sub(1))].clone();
        serialized.push(serde_json::to_string(&to_neutral(&state, &action)).unwrap());
        apply(db, &mut state, action).expect("apply");
    }
    let count = serialized.len() as u64;
    let mut bytes = Vec::with_capacity(8 + serialized.iter().map(|s| s.len()).sum::<usize>());
    bytes.extend_from_slice(&count.to_le_bytes());
    for s in &serialized {
        bytes.extend_from_slice(s.as_bytes());
    }
    fnv1a64(&bytes)
}

/// Default `h0` action fingerprints on eight meta-deck / seed pairs.
/// Re-pinned after play-time selection + `Effect::Pay` gating (2026-09-27); see
/// module docs for per-seed proof (`main@3ebef26` old → branch new, step `n`).
const DEFAULT_FINGERPRINTS: [u64; 8] = [
    0x8a3c_ed65_9428_71b4,
    0x8cd3_2204_3bbb_ebaf,
    0x0c5e_00eb_14bd_ce85,
    0x09d1_f4dd_e963_0cb7,
    0x7393_0e18_abe2_6408,
    0x3705_78b3_04cc_e2be,
    0x2d18_3971_e453_91ae,
    0x23c5_a1fa_f13b_aff0,
];

/// Pre-flip `h0` action fingerprints (`h0:mull=rule,info=fair`). Re-pinned with
/// default table where play-time / Pay gating shifts the h0 action stream; see
/// module docs.
const LEGACY_FINGERPRINTS: [u64; 8] = [
    0x8a3c_ed65_9428_71b4,
    0xf00a_d8f4_4136_9f74,
    0x0c5e_00eb_14bd_ce85,
    0x3207_d65c_42f5_89a8,
    0x7393_0e18_abe2_6408,
    0x00cc_fe69_8d17_49cf,
    0x953e_ffd4_7650_d22e,
    0x48b6_cc9c_3605_0fc5,
];

/// Dump canonical state hash after each h0 decision (ignored). Usage:
/// `H0_DUMP_SEEDS=41 H0_SPEC=h0 cargo test --release --test h0_defaults dump_gate_hashes -- --ignored --nocapture`
#[test]
#[ignore]
fn dump_gate_hashes() {
    let db = load_db();
    let stems = meta_deck_stems();
    let spec = std::env::var("H0_SPEC").unwrap_or_else(|_| "h0".into());
    let seeds: Vec<u64> = std::env::var("H0_DUMP_SEEDS")
        .unwrap_or_else(|_| "41,53,67".into())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    for seed in seeds {
        let i = GATE_SEEDS
            .iter()
            .position(|&s| s == seed)
            .expect("gate seed");
        let deck = load_meta_deck(&stems[i]);
        let deck_b = load_meta_deck("meta-sword-rally");
        let mut state = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: deck,
                deck_b,
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut a = parse_h0(&spec);
        let mut b = parse_h0(&spec);
        let mut rng = policy_rng(seed);
        let mut n = 0u32;
        println!("seed={seed} n={n} hash=0x{:016x}", hash(&state));
        while state.winner.is_none() && !matches!(state.phase, arena_engine::Phase::Terminal) {
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let me = arena_engine::acting_player(&state);
            let idx = match me {
                PlayerId::A => a.choose(&db, &state, &legal, &mut rng),
                PlayerId::B => b.choose(&db, &state, &legal, &mut rng),
            };
            let action = legal[idx.min(legal.len().saturating_sub(1))].clone();
            apply(&db, &mut state, action).expect("apply");
            n += 1;
            println!("seed={seed} n={n} hash=0x{:016x}", hash(&state));
        }
    }
}

/// Dump neutral actions for gate seeds (ignored). Usage:
/// `H0_DUMP_SEEDS=41,53,67 H0_SPEC=h0 cargo test --release --test h0_defaults dump_gate_actions -- --ignored --nocapture`
#[test]
#[ignore]
fn dump_gate_actions() {
    let db = load_db();
    let stems = meta_deck_stems();
    let spec = std::env::var("H0_SPEC").unwrap_or_else(|_| "h0".into());
    let seeds: Vec<u64> = std::env::var("H0_DUMP_SEEDS")
        .unwrap_or_else(|_| "41,53,67".into())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    for seed in seeds {
        let i = GATE_SEEDS
            .iter()
            .position(|&s| s == seed)
            .expect("gate seed");
        let deck = load_meta_deck(&stems[i]);
        let deck_b = load_meta_deck("meta-sword-rally");
        let mut state = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: deck,
                deck_b,
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut a = parse_h0(&spec);
        let mut b = parse_h0(&spec);
        let mut rng = policy_rng(seed);
        let mut n = 0u32;
        while state.winner.is_none() && !matches!(state.phase, arena_engine::Phase::Terminal) {
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let me = arena_engine::acting_player(&state);
            let idx = match me {
                PlayerId::A => a.choose(&db, &state, &legal, &mut rng),
                PlayerId::B => b.choose(&db, &state, &legal, &mut rng),
            };
            let action = legal[idx.min(legal.len().saturating_sub(1))].clone();
            println!(
                "seed={seed} n={n} {}",
                serde_json::to_string(&to_neutral(&state, &action)).unwrap()
            );
            apply(&db, &mut state, action).expect("apply");
            n += 1;
        }
        println!("seed={seed} DONE actions={n}");
    }
}

#[test]
#[ignore]
fn print_default_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let fp = action_fingerprint(&db, "h0", *seed, &deck);
        println!("seed={seed} deck={} fp=0x{:016x}", stems[i], fp);
    }
}

/// Capture pre-flip pins. Must pass `h0:mull=rule,info=fair` (not bare `h0`).
#[test]
#[ignore]
fn print_legacy_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let fp = action_fingerprint(&db, "h0:mull=rule,info=fair", *seed, &deck);
        println!("seed={seed} deck={} fp=0x{:016x}", stems[i], fp);
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn default_h0_matches_pinned_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    assert_eq!(stems.len(), 16);
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, "h0", *seed, &deck);
        assert_eq!(
            got, DEFAULT_FINGERPRINTS[i],
            "default fingerprint seed={seed} deck={}",
            stems[i]
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn legacy_spec_matches_pre_flip_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    assert_eq!(stems.len(), 16);
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, "h0:mull=rule,info=fair", *seed, &deck);
        assert_eq!(
            got, LEGACY_FINGERPRINTS[i],
            "legacy fingerprint seed={seed} deck={}",
            stems[i]
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn default_matches_explicit_builtin_table_and_open() {
    let db = load_db();
    let table = repo_root()
        .join("engine/models/mulligan-v1.json")
        .display()
        .to_string();
    let explicit = format!("h0:mull={table},info=open");
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let def = action_fingerprint(&db, "h0", *seed, &deck);
        let named = action_fingerprint(&db, &explicit, *seed, &deck);
        assert_eq!(def, named, "seed={seed} deck={}", stems[i]);
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn default_uses_builtin_mulligan_table_on_meta_decks() {
    let db = load_db();
    let stems = meta_deck_stems();
    let mut mull_table = 0u64;
    let mut mull_fallback = 0u64;
    for (i, stem) in stems.iter().enumerate() {
        let deck_a = load_meta_deck(stem);
        let deck_b = load_meta_deck(&stems[(i + 1) % stems.len()]);
        let mut state = new_game(
            &db,
            GameConfig {
                seed: 1000 + i as u64,
                deck_a,
                deck_b,
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut a = parse_h0("h0");
        let mut b = parse_h0("h0");
        let mut rng = policy_rng(1000 + i as u64);
        let _ = play_game(&db, &mut state, &mut a, &mut b, &mut rng);
        mull_table += a.stats.mull_table + b.stats.mull_table;
        mull_fallback += a.stats.mull_fallback + b.stats.mull_fallback;
    }
    assert_eq!(mull_fallback, 0, "every meta deck must hit the table");
    assert!(mull_table > 0, "builtin table must be used");
}
