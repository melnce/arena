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

use arena_engine::{
    apply, legal_actions, new_game, play_game, policy_rng, to_neutral, trace::fnv1a64, AnyPolicy,
    CardDb, First, GameConfig, PlayerId, Policy,
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

/// Pre-flip `h0` action fingerprints on eight meta-deck / seed pairs (`main@2e40d4d`).
/// `h0:mull=rule,info=fair` must reproduce them exactly.
const LEGACY_FINGERPRINTS: [u64; 8] = [
    0x8a3c_ed65_9428_71b4,
    0xf00a_d8f4_4136_9f74,
    0xe0ff_3ba0_8237_47ae,
    0xc4e8_ab63_aec9_6246,
    0x7404_fbcb_4cae_7f60,
    0xdfee_fa09_e8aa_3e5c,
    0x953e_ffd4_7650_d22e,
    0x48b6_cc9c_3605_0fc5,
];

#[test]
#[ignore]
fn print_legacy_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let fp = action_fingerprint(&db, "h0", *seed, &deck);
        println!("seed={seed} deck={} fp=0x{:016x}", stems[i], fp);
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
