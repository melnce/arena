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
//!
//! ## Play-time re-pins (`main@b5b822c` → branch)
//!
//! Proofs: branch vs `git worktree` at `main@b5b822c` using `dump_gate_hashes`,
//! `dump_gate_actions`, `dump_gate_legal_at` (`H0_DUMP_SEEDS`, `H0_DUMP_AT`,
//! `H0_VERBOSE=1`). **`n`** = first neutral-action index where fingerprints diverge
//! (requires `hash@n` match on both engines). **`hash@k`** = canonical `hash(state)`
//! after action `k−1`. **`hashΔ`** = first step where hashes differ (intermediate
//! play-time state); **`reconv`** = step where hashes match again.
//!
//! Unchanged on `main@b5b822c` (same pin on branch): default seeds 11, 23, 97;
//! legacy seeds 11, 23, 67, 79, 97. (`main@b5b822c` bonus-PP already moved
//! default 97 and legacy 67/97 — those values are the `main@b5b822c` column.)
//!
//! ### default `h0`
//!
//! | seed | deck | main@b5b822c → branch | n | hash@n | hashΔ | reconv | proof |
//! |------|------|------------------------|---|--------|-------|--------|-------|
//! | 37 | meta-dragon-aggro | `0xe0ff3ba0823747ae` → `0x0c5e00eb14bdce85` | 88 | match | 79 | 80 | **hashΔ 79:** after Spilling Red `10642310` discard pick (`choose` `90044330`); main `PendingKind::EffectSelect` + faith tick in snapshot; branch `PlaySelect` destroy pick (`step 1/2`, `pending_work=1`). **reconv 80.** **n 88:** `hash@87` match; main attacks; branch plays Spilling Red then play-time discard+destroy picks (`89`–`90`). |
//! | 41 | meta-dragon-ramp | `0x9e542bf8f4f4ba5c` → `0x09d1f4dde9630cb7` | 47 | match | 17 | 18 | **hashΔ 17:** after Yidmetra `90024320` play; main `EffectSelect` Enhanced destroy; branch `PlaySelect` + `deferred_rx=1`. **reconv 18.** **n 47:** `hash@46` match; main plays Lyria `10403120`; branch plays Burnite `10744110` (h0 sim values shifted by play-time apply). |
//! | 53 | meta-forest-combo | `0x7404fbcb4cae7f60` → `0x73930e18abe26408` | 86 | match | 37 | 38 | **hashΔ 37:** after Miroku `10514120` play; main `ModeSelect`; branch `PlaySelect` mode + `deferred_rx=1`. **reconv 38.** **n 86:** `hash@85` match; main plays Bell-ring Spirit `10624120`; branch attacks first (`87` plays Spirit). |
//! | 67 | meta-haven-amulet | `0x50a92b69688f99c3` → `0x85c938a069126ff1` | 20 | match | — | — | **n 20:** actions `0..19` identical; `hash@20` match; legal set identical (10 actions); main pick idx 5 attack slot 0; branch pick idx 2 play Timepiece `10762210`. No prior hashΔ. h0 1-ply values differ because `apply()` simulation of candidates hits play-time ordering deeper in the tree. |
//! | 79 | meta-haven-evo | `0x42fbd544e452ebf0` → `0x2d183971e45391ae` | 63 | match | — | — | **n 63:** actions `0..62` identical; `hash@63` match; legal identical (4 actions); main pick idx 0 play Yidmetra `90024320` → `64` `EffectSelect` Enhanced destroy; branch pick idx 1 attack (Yidmetra played later with `PlaySelect` + `deferred_rx=1`). |
//!
//! ### legacy `h0:mull=rule,info=fair`
//!
//! | seed | deck | main@b5b822c → branch | n | hash@n | hashΔ | reconv | proof |
//! |------|------|------------------------|---|--------|-------|--------|-------|
//! | 37 | meta-dragon-aggro | `0xe0ff3ba0823747ae` → `0x0c5e00eb14bdce85` | 88 | match | 79 | 80 | same as default 37 |
//! | 41 | meta-dragon-ramp | `0xc4e8ab63aec96246` → `0x3207d65c42f589a8` | 51 | match | 52 | 53 | **hashΔ 52:** Lumiore `10844120` discard (`PlaySelect` both); main hash shifts on first discard pick; branch stays at `hash@51`. **reconv 53.** **n 51:** `hash@50` match; main discards `10042310` then `10744110`; branch reverses order. |
//! | 53 | meta-forest-combo | `0x7404fbcb4cae7f60` → `0x73930e18abe26408` | 86 | match | 37 | 38 | same as default 53 |

use arena_engine::{
    apply, hash, legal_actions, new_game, play_game, policy_rng, to_neutral, trace::fnv1a64,
    Action, AnyPolicy, CardDb, First, GameConfig, PlayerId, Policy,
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

/// Full neutral-action trace for divergence analysis.
fn action_trace(
    db: &CardDb,
    spec: &str,
    seed: u64,
    deck_a: &[arena_engine::CardId],
) -> Vec<String> {
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
    serialized
}

/// Default `h0` action fingerprints on eight meta-deck / seed pairs (`main@1f8b068`).
/// Captured before leaf-encoding-v2; bare `h0` must reproduce them exactly.
const DEFAULT_FINGERPRINTS: [u64; 8] = [
    0x8a3c_ed65_9428_71b4,
    0x8cd3_2204_3bbb_ebaf,
    0x0c5e_00eb_14bd_ce85,
    0x09d1_f4dd_e963_0cb7,
    0x7393_0e18_abe2_6408,
    0x85c9_38a0_6912_6ff1,
    0x2d18_3971_e453_91ae,
    0x5b5a_5be5_f92e_3c7c,
];

/// Pre-flip `h0` action fingerprints (`h0:mull=rule,info=fair`). Re-pinned with
/// play-time / Pay gating on top of `main@b5b822c`; see module docs.
const LEGACY_FINGERPRINTS: [u64; 8] = [
    0x8a3c_ed65_9428_71b4,
    0xf00a_d8f4_4136_9f74,
    0x0c5e_00eb_14bd_ce85,
    0x3207_d65c_42f5_89a8,
    0x7393_0e18_abe2_6408,
    0x8fe4_b401_a51d_973b,
    0x953e_ffd4_7650_d22e,
    0x5b5a_5be5_f92e_3c7c,
];

/// Dump canonical state hash after each h0 decision (ignored).
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
        if std::env::var("H0_VERBOSE").ok().as_deref() == Some("1") {
            println!(
                "seed={seed} n={n} phase={:?} pending={}",
                state.phase,
                pending_summary(&state)
            );
        }
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
            if std::env::var("H0_VERBOSE").ok().as_deref() == Some("1") {
                println!(
                    "seed={seed} n={n} phase={:?} pending={}",
                    state.phase,
                    pending_summary(&state)
                );
            }
        }
    }
}

fn pending_summary(state: &arena_engine::State) -> String {
    let mut parts = vec![format!("phase={:?}", state.phase)];
    if let Some(p) = &state.pending_play_choices {
        parts.push(format!(
            "play_choices step={}/{} deferred_rx={}",
            p.step_idx,
            p.steps.len(),
            p.deferred_rx.len()
        ));
    }
    if state.deferred_play_rx.is_some() {
        parts.push("deferred_play_rx".into());
    }
    if !state.pending_work.is_empty() {
        parts.push(format!("pending_work={}", state.pending_work.len()));
    }
    parts.join(" ")
}

/// Dump legal action count + fingerprint at step `H0_DUMP_AT` (ignored).
#[test]
#[ignore]
fn dump_gate_legal_at() {
    let db = load_db();
    let stems = meta_deck_stems();
    let spec = std::env::var("H0_SPEC").unwrap_or_else(|_| "h0".into());
    let seed: u64 = std::env::var("H0_DUMP_SEEDS")
        .unwrap_or_else(|_| "67".into())
        .parse()
        .expect("seed");
    let at: u32 = std::env::var("H0_DUMP_AT")
        .unwrap_or_else(|_| "20".into())
        .parse()
        .expect("at");
    let i = GATE_SEEDS.iter().position(|&s| s == seed).expect("gate seed");
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
        if n == at {
            let legal = legal_actions(&db, &state);
            let me = arena_engine::acting_player(&state);
            let idx = match me {
                PlayerId::A => a.choose(&db, &state, &legal, &mut rng),
                PlayerId::B => b.choose(&db, &state, &legal, &mut rng),
            };
            println!(
                "seed={seed} at={at} hash=0x{:016x} legal={} pick={} action={}",
                hash(&state),
                legal.len(),
                idx,
                serde_json::to_string(&to_neutral(&state, &legal[idx.min(legal.len().saturating_sub(1))]))
                    .unwrap()
            );
            for (j, act) in legal.iter().enumerate() {
                println!(
                    "  legal[{j}] {}",
                    serde_json::to_string(&to_neutral(&state, act)).unwrap()
                );
            }
            return;
        }
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
    }
}

/// Dump neutral actions for gate seeds (ignored).
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
fn dump_trace_for_diff() {
    let db = load_db();
    let cases = [
        ("h0", 97u64, "meta-haven-kukishiro"),
        ("h0:mull=rule,info=fair", 67, "meta-haven-amulet"),
        ("h0:mull=rule,info=fair", 97, "meta-haven-kukishiro"),
    ];
    for (spec, seed, stem) in cases {
        let deck = load_meta_deck(stem);
        let trace = action_trace(&db, spec, seed, &deck);
        println!("BEGIN {spec} seed={seed} deck={stem}");
        for (i, a) in trace.iter().enumerate() {
            println!("{i}|{a}");
        }
        println!("END {spec} seed={seed}");
    }
}

#[test]
#[ignore]
fn trace_bonus_pp_divergence_cases() {
    let db = load_db();
    let cases = [
        ("h0", 97u64, "meta-haven-kukishiro"),
        ("h0:mull=rule,info=fair", 67, "meta-haven-amulet"),
        ("h0:mull=rule,info=fair", 97, "meta-haven-kukishiro"),
    ];
    for (spec, seed, stem) in cases {
        let deck = load_meta_deck(stem);
        let trace = action_trace(&db, spec, seed, &deck);
        println!(
            "=== {spec} seed={seed} deck={stem} actions={} ===",
            trace.len()
        );
        let deck_b = load_meta_deck("meta-sword-rally");
        let mut state = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: deck.clone(),
                deck_b,
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut a = parse_h0(spec);
        let mut b = parse_h0(spec);
        let mut rng = policy_rng(seed);
        for (i, expected) in trace.iter().enumerate() {
            let me = arena_engine::acting_player(&state);
            let pre_active = state.player(me).bonus_pp.active;
            let pre_locked = state.player(me).bonus_pp.locked;
            let legal = legal_actions(&db, &state);
            let idx = match me {
                PlayerId::A => a.choose(&db, &state, &legal, &mut rng),
                PlayerId::B => b.choose(&db, &state, &legal, &mut rng),
            };
            let action = legal[idx.min(legal.len().saturating_sub(1))].clone();
            let got = serde_json::to_string(&to_neutral(&state, &action)).unwrap();
            assert_eq!(&got, expected, "action {i}");
            let is_eot = matches!(action, Action::EndTurn);
            apply(&db, &mut state, action).expect("apply");
            if is_eot && pre_active && !pre_locked {
                println!(
                    "  EOT turn={} action={i}: player {:?} ended with unspent bonus orb",
                    state.turn, me
                );
            }
        }
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
