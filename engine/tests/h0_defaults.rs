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
//! On this branch, `h0:mull=rule,info=fair,net=<v1 path>` must match those pins;
//! bare `h0` must match `h0:mull=engine/models/mulligan-v2.json,info=open` and
//! `h0:net=<v4 path>` (the yardstick candidate); `h0:net=<v3 path>` reproduces
//! the pre-v4 default pins; `h0:net=<v2 path>` reproduces the pre-v3 default pins.
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
//! | 41 | meta-dragon-ramp | `0x9e542bf8f4f4ba5c` → `0x09d1f4dde9630cb7` | 47 | match | 17 | 18 | **hashΔ 17:** after Depths of the Eld Sword `90024320` (Enhanced) play; main `EffectSelect` destroy pick with Faith: Yidmetra `faith:10624120` tick first; branch `PlaySelect` + `deferred_rx=1`. **reconv 18.** **n 47:** `hash@46` match; main idx 4 plays Lyria, Skydestined `10403120`; branch idx 2 plays Burnite, Anathema of Ash `10744110`. |
//! | 53 | meta-forest-combo | `0x7404fbcb4cae7f60` → `0x73930e18abe26408` | 86 | match | 37 | 38 | **hashΔ 37:** after Miroku, Swarmpetal `10514120` play; main `ModeSelect`; branch `PlaySelect` mode + `deferred_rx=1`. **reconv 38.** **n 86:** `hash@85` match; main idx 1 plays Yidmetra, Eld Sword `10624120`; branch idx 8 attacks (`87` plays Depths of the Eld Sword `90024320` Enhanced). |
//! | 67 | meta-haven-amulet | `0x50a92b69688f99c3` → `0x85c938a069126ff1` | 20 | match | — | — | **n 20:** actions `0..19` identical; `hash@20` match; legal identical (10 actions); main idx 5 attack slot 0; branch idx 2 play Timepiece of Perfection `10762210` (no play-time pick on either root). **explain@20 cand[2]:** worlds 0–2 match (leaf −6.2134); **world[3]** diverges — main hits node cap after engage (`pv_len=2`); branch `pv_len=9` leaf +4.4162 with opp Depths of the Eld Sword `90024320` (Enhanced) `choose` at PV step 5 while Faith: Yidmetra `faith:10624120` is deferred → `root_agg` −6.2134 → −3.5560 (beats main best attack −5.8025). |
//! | 79 | meta-haven-evo | `0x42fbd544e452ebf0` → `0x2d183971e45391ae` | 63 | match | — | — | **n 63:** actions `0..62` identical; `hash@63` match; legal identical (4 actions); main idx 0 play Depths of the Eld Sword `90024320` (Enhanced) → `64` `EffectSelect` destroy; branch idx 1 attack (`64` plays Depths with `PlaySelect` + `deferred_rx=1`). |
//!
//! ## Earrings Engage re-pins (`main@99b7c05` → branch)
//!
//! Sacrifice Engage replicates Fanfare (`10761210`). Gate fingerprints recomputed from each
//! test's own `h0` spec (not the `print_*` helpers). **`n`** is the 0-based index printed
//! by `dump_gate_actions` (same convention as the play-time table above: first neutral-action
//! JSON where branch and `main@99b7c05` differ). Proof (c): `git show origin/main:engine/src/apply.rs`
//! + `origin/main` pins → `cargo test --release --test h0_defaults -- --include-ignored`
//!   → **26 passed** (2026-10-11).
//!
//! Where the first divergence is a **`meta-sword-rally` (seat B) move** before any Earrings
//! play on that trace, B's search simulates A's turns in its worlds; A's sacrifice Engage now
//! replicates Fanfare and cycles a hand card, so B's policy picks change — proof (c) ties those
//! rows to this fix.
//!
//! | file / table | seed | deck | main@99b7c05 → branch | n | divergence |
//! |--------------|------|------|------------------------|---|------------|
//! | `DEFAULT_FINGERPRINTS` | 67 | meta-haven-amulet | `0x25a8e3162fe653ad` → `0x86671e5ef842ccfb` | 31 | branch `engage` slot 2 (`10761210`); main `play` hand 3 `10661210`. Earrings on field at n. |
//! | `DEFAULT_FINGERPRINTS` | 97 | meta-haven-kukishiro | `0x28f80dd9b44b4e7b` → `0xf4c105a794883210` | 61 | branch A `attack`; main A `play` `10761210` (Earrings on field; prior A `engage` slot 0 @ 60). |
//! | `LEGACY_FINGERPRINTS` | 67 | meta-haven-amulet | `0xf0a3386e8682af1e` → `0x6af15e5560a10a58` | 19 | branch A `engage` slot 1; main A `engage` slot 3. Earrings on field at n. |
//! | `LEGACY_FINGERPRINTS` | 79 | meta-haven-evo | `0x953effd47650d22e` → `0x4e0f03454ca12711` | 30 | branch B `play` `10723110`; main B `attack` leader (seat-B first diff). |
//! | `LEGACY_FINGERPRINTS` | 97 | meta-haven-kukishiro | `0x5b5a5be5f92e3c7c` → `0x97344d31d7152b48` | 11 | branch B `attack` leader; main B `attack` slot 1 (seat-B first diff). |
//! | `V1_DEFAULT_FINGERPRINTS` | 67 | meta-haven-amulet | `0x85c938a069126ff1` → `0xb1abe142aa179c09` | 49 | branch B `play` `90021120`; main B `attack` slot 3 leader (seat-B first diff). |
//! | `V1_DEFAULT_FINGERPRINTS` | 79 | meta-haven-evo | `0x2d183971e45391ae` → `0x1aab6bacaa68cfb1` | 55 | branch A `choose` `10963210`; main A `choose` `10863210`. |
//! | `V1_DEFAULT_FINGERPRINTS` | 97 | meta-haven-kukishiro | `0x5b5a5be5f92e3c7c` → `0x97344d31d7152b48` | 11 | same as legacy 97 (seat-B first diff). |
//! | `V2_DEFAULT_FINGERPRINTS` | 67 | meta-haven-amulet | `0x8f6bb18b4445e0bd` → `0x9b0b1dba8bb63158` | 48 | branch B `play` `90024330`; main B `attack` slot 2 slot 3 (seat-B first diff). |
//! | `V2_DEFAULT_FINGERPRINTS` | 79 | meta-haven-evo | `0x5cca9048c37e96c9` → `0x6c3461eed232775e` | 58 | branch B `attack` slot 0 slot 0; main B `attack` slot 1 slot 0 (seat-B first diff). |
//! | `V2_DEFAULT_FINGERPRINTS` | 97 | meta-haven-kukishiro | `0x46e6103324ea959f` → `0xccfa14793ef8e148` | 4 | branch B `play` `10021110`; main B `bonus_pp` (seat-B first diff). |
//! | `V3_DEFAULT_FINGERPRINTS` | 67 | meta-haven-amulet | `0x783df18c0ca289f1` → `0x9e4a4092a024a9e7` | 99 | branch A `engage` slot 1; main A `play` hand 6 `90064320`. Earrings on field at n. |
//! | `V3_DEFAULT_FINGERPRINTS` | 79 | meta-haven-evo | `0x3f38eeac5db259c1` → `0xfd6aac8f03e94988` | 48 | branch A `engage` slot 0; main A `play` `10863210`. Earrings on field at n. |
//! | `V3_DEFAULT_FINGERPRINTS` | 97 | meta-haven-kukishiro | `0x857a2b3ba078359e` → `0x435b799878a022c3` | 4 | branch B `play` `10021110`; main B `bonus_pp` (seat-B first diff). |
//! | `h0_okill` `DEFAULT_OKILL_GATE_FPS` | 67 | meta-haven-amulet | `0x25a8e3162fe653ad` → `0x86671e5ef842ccfb` | 31 | same as default `h0`. |
//! | `h0_okill` `DEFAULT_OKILL_GATE_FPS` | 97 | meta-haven-kukishiro | `0x28f80dd9b44b4e7b` → `0xf4c105a794883210` | 61 | same as default `h0`. |
//! | `h0_okill` `OKILL7_GATE_FPS` | 67 | meta-haven-amulet | `0x25a8e3162fe653ad` → `0x356d7e0a857b5dec` | 31 | `h0:okill=7`; same as default `h0` seed 67. |
//! | `h0_okill` `OKILL7_GATE_FPS` | 97 | meta-haven-kukishiro | `0x84f14a0177e6c228` → `0x0389b4048b50d258` | 61 | `h0:okill=7`; same as default `h0` seed 97. |
//! | `h0_hbk` `DEFAULT_OKILL_GATE_FPS` | 67, 97 | haven | same as `h0_okill` DEFAULT | 31 / 61 | `hbk_default_play_unchanged_gate` uses `load_db()` (current rules). |
//! | `h0_info` `other_info_modes_play_unchanged_on_meta_games` | 99 | haven-amulet vs dragon-ramp | play tuples in `h0_info.rs` | — | revert proof: `main` `apply.rs` + old tuples → **2 passed** (`other_info_modes_*`, `okill_omacro_*`). |
//! | `h0_info` `okill_omacro_defaults_play_unchanged_on_meta_games` | 99 | haven-amulet vs dragon-ramp | play tuples in `h0_info.rs` | — | same revert proof as row above. |
//!
//! ### legacy `h0:mull=rule,info=fair`
//!
//! | seed | deck | main@b5b822c → branch | n | hash@n | hashΔ | reconv | proof |
//! |------|------|------------------------|---|--------|-------|--------|-------|
//! | 37 | meta-dragon-aggro | `0xe0ff3ba0823747ae` → `0x0c5e00eb14bdce85` | 88 | match | 79 | 80 | same as default 37 |
//! | 41 | meta-dragon-ramp | `0xc4e8ab63aec96246` → `0x3207d65c42f589a8` | 51 | match | 52 | 53 | **hashΔ 52:** Lumiore & Argente, Shining Wings `10844120` discard (`PlaySelect` both); main hash shifts on first discard pick; branch stays at `hash@51`. **reconv 53.** **n 51:** `hash@50` match; main discards Dragonsign `10042310` then Burnite, Anathema of Ash `10744110`; branch reverses order. |
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

/// Default `h0` action fingerprints on eight meta-deck / seed pairs.
/// Computed on `main@7d7d2d1` with `mulligan-v2` as the built-in table.
const DEFAULT_FINGERPRINTS: [u64; 8] = [
    0x4087_4d81_d117_4ee9,
    0x8867_a7c5_74fc_8fa9,
    0x649a_13b2_5e6f_3115,
    0xf257_1914_6cd5_5f83,
    0x362b_e20c_ddf6_d64c,
    0x8667_1e5e_f842_ccfb,
    0xfd90_619b_00c6_58f3,
    0xf4c1_05a7_9488_3210,
];

/// Pre-v4 default `h0` fingerprints (`main@3742a1b`, built-in `h0-linear-v3`).
const V3_DEFAULT_FINGERPRINTS: [u64; 8] = [
    0x2246_9fdb_d814_7321,
    0x6a97_4302_9171_efbc,
    0x120b_7fb2_41cf_3f44,
    0xfd6c_73d8_9fd1_d09d,
    0xd9fb_a1c8_8f98_0106,
    0x9e4a_4092_a024_a9e7,
    0xfd6a_ac8f_03e9_4988,
    0x435b_7998_78a0_22c3,
];

/// Pre-v3 default `h0` fingerprints (`main@a69b248`, built-in `h0-linear-v2`).
const V2_DEFAULT_FINGERPRINTS: [u64; 8] = [
    0x9e78_76d1_68f6_7ca1,
    0x19e7_2e1b_1adf_b216,
    0x122c_fa71_850c_7637,
    0x9475_f3fc_7d65_58c7,
    0x3633_2073_4090_3124,
    0x9b0b_1dba_8bb6_3158,
    0x6c34_61ee_d232_775e,
    0xccfa_1479_3ef8_e148,
];

/// Pre-v2 default `h0` fingerprints (`main@063bdd4`, built-in `h0-linear-v1`).
const V1_DEFAULT_FINGERPRINTS: [u64; 8] = [
    0x8a3c_ed65_9428_71b4,
    0xe2f8_652b_1c6e_d321,
    0x0c5e_00eb_14bd_ce85,
    0xa024_135f_cf64_4006,
    0x7099_b9ba_b571_926b,
    0xb1ab_e142_aa17_9c09,
    0x1aab_6bac_aa68_cfb1,
    0x9734_4d31_d715_2b48,
];

/// Pre-flip `h0` action fingerprints (`h0:mull=rule,info=fair`). Re-pinned with
/// play-time / Pay gating on top of `main@b5b822c`; see module docs.
const LEGACY_FINGERPRINTS: [u64; 8] = [
    0x8a3c_ed65_9428_71b4,
    0x5fe8_8bb6_8e5d_65fc,
    0x0c5e_00eb_14bd_ce85,
    0x3d7f_2f1b_9db5_99cc,
    0x7099_b9ba_b571_926b,
    0x6af1_5e55_60a1_0a58,
    0x4e0f_0345_4ca1_2711,
    0x9734_4d31_d715_2b48,
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

/// Dump h0 explain at step `H0_DUMP_AT` (ignored). Replay with `H0_DUMP_SEEDS` /
/// `H0_DUMP_AT`; prints each candidate's `root_agg` and principal-variation prefix.
#[test]
#[ignore]
fn dump_gate_explain_at() {
    let db = load_db();
    let stems = meta_deck_stems();
    let spec = std::env::var("H0_SPEC").unwrap_or_else(|_| with_v2_net("h0"));
    let seed: u64 = std::env::var("H0_DUMP_SEEDS")
        .unwrap_or_else(|_| "67".into())
        .parse()
        .expect("seed");
    let at: u32 = std::env::var("H0_DUMP_AT")
        .unwrap_or_else(|_| "20".into())
        .parse()
        .expect("at");
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
    let mut h0 = parse_h0(&spec);
    let mut opp = parse_h0(&spec);
    let mut rng = policy_rng(seed);
    let mut n = 0u32;
    while state.winner.is_none() && !matches!(state.phase, arena_engine::Phase::Terminal) {
        if n == at {
            let legal = legal_actions(&db, &state);
            h0.arm_explain();
            let me = arena_engine::acting_player(&state);
            let idx = match me {
                PlayerId::A => h0.choose(&db, &state, &legal, &mut rng),
                PlayerId::B => opp.choose(&db, &state, &legal, &mut rng),
            };
            let rec = h0.take_explain().expect("explain");
            println!(
                "seed={seed} at={at} hash=0x{:016x} path={:?} chosen={} legal={}",
                hash(&state),
                rec.path,
                idx,
                legal.len()
            );
            let focus: Option<usize> = std::env::var("H0_EXPLAIN_CAND")
                .ok()
                .and_then(|s| s.parse().ok());
            for cand in &rec.candidates {
                if focus.is_some_and(|f| f != cand.legal_index) {
                    continue;
                }
                println!(
                    "  cand[{}] agg={:.4} worst={:.4} n={} action={}",
                    cand.legal_index,
                    cand.root_agg,
                    cand.worst,
                    cand.n,
                    serde_json::to_string(&cand.action).unwrap()
                );
                for (wi, w) in cand.worlds.iter().enumerate() {
                    let pv =
                        w.pv.iter()
                            .map(|a| serde_json::to_string(a).unwrap_or_else(|_| "?".into()))
                            .collect::<Vec<_>>()
                            .join(" | ");
                    println!(
                        "    world[{wi}] raw={:.4} clamped={:.4} end={:?} pv_len={} pv={}",
                        w.raw, w.clamped, w.end, w.pv_len, pv
                    );
                    if let Some(leaf) = &w.leaf {
                        println!(
                            "      leaf value={:.4} phase={} turn={} active={}",
                            leaf.value, leaf.phase, leaf.turn, leaf.active
                        );
                    }
                }
            }
            return;
        }
        let legal = legal_actions(&db, &state);
        if legal.is_empty() {
            break;
        }
        let me = arena_engine::acting_player(&state);
        let idx = match me {
            PlayerId::A => h0.choose(&db, &state, &legal, &mut rng),
            PlayerId::B => opp.choose(&db, &state, &legal, &mut rng),
        };
        let action = legal[idx.min(legal.len().saturating_sub(1))].clone();
        apply(&db, &mut state, action).expect("apply");
        n += 1;
    }
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
                serde_json::to_string(&to_neutral(
                    &state,
                    &legal[idx.min(legal.len().saturating_sub(1))]
                ))
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
    let spec = std::env::var("H0_SPEC").unwrap_or_else(|_| "h0".into());
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let fp = action_fingerprint(&db, &spec, *seed, &deck);
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
#[ignore]
fn print_v1_net_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    let v1 = h0_linear_v1_path();
    let spec = format!("h0:net={v1}");
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let fp = action_fingerprint(&db, &spec, *seed, &deck);
        println!("seed={seed} deck={} fp=0x{:016x}", stems[i], fp);
    }
}

#[test]
#[ignore]
fn print_v2_net_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    let v2 = h0_linear_v2_path();
    let spec = format!("h0:net={v2}");
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let fp = action_fingerprint(&db, &spec, *seed, &deck);
        println!("seed={seed} deck={} fp=0x{:016x}", stems[i], fp);
    }
}

#[test]
#[ignore]
fn print_v3_net_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    let v3 = h0_linear_v3_path();
    let spec = format!("h0:net={v3}");
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let fp = action_fingerprint(&db, &spec, *seed, &deck);
        println!("seed={seed} deck={} fp=0x{:016x}", stems[i], fp);
    }
}

#[test]
#[ignore]
fn print_v4_net_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    let v4 = h0_linear_v4_path();
    let spec = format!("h0:net={v4}");
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let fp = action_fingerprint(&db, &spec, *seed, &deck);
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
    let v1 = h0_linear_v1_path();
    let spec = format!("h0:mull=rule,info=fair,net={v1}");
    assert_eq!(stems.len(), 16);
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, &spec, *seed, &deck);
        assert_eq!(
            got, LEGACY_FINGERPRINTS[i],
            "legacy fingerprint seed={seed} deck={}",
            stems[i]
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn v1_net_matches_pre_v2_default_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    let v1 = h0_linear_v1_path();
    let mull = repo_root()
        .join("engine/models/mulligan-v1.json")
        .display()
        .to_string();
    let spec = format!("h0:mull={mull},net={v1}");
    assert_eq!(stems.len(), 16);
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, &spec, *seed, &deck);
        assert_eq!(
            got, V1_DEFAULT_FINGERPRINTS[i],
            "v1 net fingerprint seed={seed} deck={}",
            stems[i]
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn v3_net_matches_pre_v4_default_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    let v3 = h0_linear_v3_path();
    let mull = repo_root()
        .join("engine/models/mulligan-v1.json")
        .display()
        .to_string();
    let spec = format!("h0:mull={mull},net={v3}");
    assert_eq!(stems.len(), 16);
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, &spec, *seed, &deck);
        assert_eq!(
            got, V3_DEFAULT_FINGERPRINTS[i],
            "v3 net fingerprint seed={seed} deck={}",
            stems[i]
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn v2_net_matches_pre_v3_default_fingerprints() {
    let db = load_db();
    let stems = meta_deck_stems();
    let v2 = h0_linear_v2_path();
    let mull = repo_root()
        .join("engine/models/mulligan-v1.json")
        .display()
        .to_string();
    let spec = format!("h0:mull={mull},net={v2}");
    assert_eq!(stems.len(), 16);
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, &spec, *seed, &deck);
        assert_eq!(
            got, V2_DEFAULT_FINGERPRINTS[i],
            "v2 net fingerprint seed={seed} deck={}",
            stems[i]
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn default_matches_explicit_builtin_table_and_open() {
    let db = load_db();
    let table = repo_root()
        .join("engine/models/mulligan-v2.json")
        .display()
        .to_string();
    let v4 = h0_linear_v4_path();
    let explicit_mull = format!("h0:mull={table},info=open");
    let explicit_net = format!("h0:net={v4}");
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let def = action_fingerprint(&db, "h0", *seed, &deck);
        let named_mull = action_fingerprint(&db, &explicit_mull, *seed, &deck);
        let named_net = action_fingerprint(&db, &explicit_net, *seed, &deck);
        assert_eq!(def, named_mull, "mull seed={seed} deck={}", stems[i]);
        assert_eq!(def, named_net, "net seed={seed} deck={}", stems[i]);
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

#[test]
fn h0_default_wseed_is_off() {
    let h = parse_h0("h0");
    assert_eq!(h.wseed, arena_engine::policy::Wseed::Off);
}

#[test]
fn h0_default_wbase_is_none() {
    let h = parse_h0("h0");
    assert!(h.wbase.is_none());
}

#[test]
fn h0_default_lostrank_is_zero() {
    let h = parse_h0("h0");
    assert_eq!(h.lostrank, 0);
}

#[test]
fn h0_default_fuseguard_is_zero() {
    let h = parse_h0("h0");
    assert!(!h.fuseguard);
}

#[test]
fn h0_default_deal_is_indep() {
    let h = parse_h0("h0");
    assert_eq!(h.deal, arena_engine::policy::Deal::Indep);
}

#[test]
fn h0_default_hread_is_off() {
    let h = parse_h0("h0");
    assert!(h.hread.is_none());
}

#[test]
fn h0_default_hreadm_is_256() {
    let h = parse_h0("h0");
    assert_eq!(h.hreadm, 256);
}
