//! Set-10 identity golden: bit-identical state hashes, legal lists, v3 encodings,
//! and snapshot JSON on every ply of a fixed schedule of seeded random-legal games.
//!
//! Generated once on unmodified `main` (`ARENA_SET10_IDENTITY_WRITE=1`). The golden
//! freezes every enumerated input (deck stems, deck card lists + file hashes, seeds
//! and seat pairing). Verification reads **only** `golden.json` — never `POOLS.json`
//! or live deck files — so later PRs that add pools/decks cannot touch this gate.
//!
//! Release runtime target: < 60 s.

use std::path::PathBuf;

use arena_engine::{
    apply, encode_version, hash, legal_actions, new_game, policy_rng, snapshot_json, to_neutral,
    trace::fnv1a64, CardDb, CardId, EncodingVersion, First, GameConfig, Phase, PlayerId,
    MAX_ACTIONS,
};

mod common;
use common::*;

const GOLDEN_REL: &str = "engine/tests/fixtures/set10_identity/golden.json";
/// Games per (deck, seat): ≥ 8 as required; 32 decks × 2 seats × 8 = 512 ≥ 300.
const GAMES_PER_DECK_SEAT: u32 = 8;
const BASE_SEED: u64 = 10_010_000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GoldenDeck {
    stem: String,
    /// FNV-1a 64 of the raw `oracle/decks/{stem}.json` bytes at generation time.
    file_fnv: String,
    /// Expanded 40-card list (card id strings), frozen — the only deck source at verify.
    cards: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GoldenGame {
    seed: u64,
    /// Index into `GoldenFile.decks`.
    deck_a: usize,
    deck_b: usize,
    /// Folded per-ply digest (state hash, legal-list hash, both-seat v3, snapshot JSON).
    digest: String,
    /// `hash(&state)` after the game ends (or hits the action cap).
    final_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GoldenFile {
    /// `git rev-parse HEAD` of the tree that generated this golden.
    main_sha: String,
    /// How the golden was produced (command + note).
    generated_by: String,
    games_per_deck_seat: u32,
    base_seed: u64,
    decks: Vec<GoldenDeck>,
    games: Vec<GoldenGame>,
    /// FNV-1a over every game's digest bytes (ordered).
    schedule_digest: String,
}

fn golden_path() -> PathBuf {
    repo_root().join(GOLDEN_REL)
}

fn hex_u64(h: u64) -> String {
    format!("0x{h:016x}")
}

fn fnv_update(h: u64, bytes: &[u8]) -> u64 {
    let mut h = h;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn fnv_u64(h: u64, v: u64) -> u64 {
    fnv_update(h, &v.to_le_bytes())
}

fn legal_list_hash(db: &CardDb, state: &arena_engine::State) -> u64 {
    let legal = legal_actions(db, state);
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_u64(h, legal.len() as u64);
    for a in &legal {
        let neut = to_neutral(state, a);
        let s = serde_json::to_string(&neut).expect("neutral action json");
        h = fnv_update(h, s.as_bytes());
    }
    h
}

fn encoding_v3_hash(db: &CardDb, state: &arena_engine::State) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for me in [PlayerId::A, PlayerId::B] {
        let obs = encode_version(state, me, EncodingVersion::V3, Some(db));
        h = fnv_u64(h, obs.features.len() as u64);
        for f in &obs.features {
            h = fnv_u64(h, u64::from(f.to_bits()));
        }
        h = fnv_u64(h, obs.ids.len() as u64);
        for id in &obs.ids {
            h = fnv_u64(h, u64::from(*id));
        }
    }
    h
}

fn snapshot_json_hash(state: &arena_engine::State) -> u64 {
    let json = snapshot_json(state);
    let bytes = serde_json::to_vec(&json).expect("snapshot json bytes");
    fnv1a64(&bytes)
}

fn fold_ply(acc: u64, db: &CardDb, state: &arena_engine::State) -> u64 {
    let mut h = acc;
    h = fnv_u64(h, hash(state));
    h = fnv_u64(h, legal_list_hash(db, state));
    h = fnv_u64(h, encoding_v3_hash(db, state));
    h = fnv_u64(h, snapshot_json_hash(state));
    h
}

fn parse_deck_cards(cards: &[String]) -> Vec<CardId> {
    cards
        .iter()
        .map(|s| CardId::parse(s).unwrap_or_else(|| panic!("bad card id in golden: {s}")))
        .collect()
}

fn play_digest(
    db: &CardDb,
    seed: u64,
    deck_a: &[CardId],
    deck_b: &[CardId],
) -> (u64, u64) {
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
    let mut digest = 0xcbf2_9ce4_8422_2325u64;
    let mut actions = 0u32;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        assert!(actions < MAX_ACTIONS, "seed={seed}: game did not terminate");
        digest = fold_ply(digest, db, &state);
        let legal = legal_actions(db, &state);
        assert!(!legal.is_empty(), "seed={seed}: empty legal before terminal");
        let idx = policy.gen_range(legal.len() as u32) as usize;
        apply(db, &mut state, legal[idx].clone()).unwrap_or_else(|e| {
            panic!("seed={seed}: legal action failed: {e:?}");
        });
        actions += 1;
    }
    // Terminal / post-game ply: include final public state once more.
    digest = fold_ply(digest, db, &state);
    (digest, hash(&state))
}

fn build_schedule(n_decks: usize) -> Vec<(u64, usize, usize)> {
    // Each deck appears ≥ GAMES_PER_DECK_SEAT times in seat A and seat B.
    // Opponent is the next stem (wrapping). Seeds are fully determined by indices.
    let mut out = Vec::with_capacity(n_decks * 2 * GAMES_PER_DECK_SEAT as usize);
    for i in 0..n_decks {
        let opp = (i + 1) % n_decks;
        for g in 0..GAMES_PER_DECK_SEAT {
            let seed_a = BASE_SEED + (i as u64) * 1_000 + u64::from(g) * 2;
            out.push((seed_a, i, opp)); // deck i as A
            let seed_b = BASE_SEED + (i as u64) * 1_000 + u64::from(g) * 2 + 1;
            out.push((seed_b, opp, i)); // deck i as B
        }
    }
    out
}

fn all_stems_from_pools() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: serde_json::Value = serde_json::from_str(&text).expect("pools json");
    pools["all"]
        .as_array()
        .expect("all pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .collect()
}

fn write_golden(db: &CardDb) -> GoldenFile {
    let stems = all_stems_from_pools();
    assert!(
        stems.len() >= 2,
        "POOLS.json pool \"all\" must have ≥ 2 decks"
    );

    let mut decks = Vec::with_capacity(stems.len());
    for stem in &stems {
        let path = repo_root().join(format!("oracle/decks/{stem}.json"));
        let raw = std::fs::read(&path).unwrap_or_else(|e| panic!("read {stem}: {e}"));
        let file_fnv = hex_u64(fnv1a64(&raw));
        let ids = load_deck_file(path);
        assert_eq!(ids.len(), 40, "deck {stem} must be 40 cards");
        assert!(deck_ready(db, &ids), "deck {stem} must load");
        decks.push(GoldenDeck {
            stem: stem.clone(),
            file_fnv,
            cards: ids.iter().map(|id| id.as_str().to_string()).collect(),
        });
    }

    let schedule = build_schedule(decks.len());
    assert!(
        schedule.len() >= 300,
        "need ≥ 300 games, got {}",
        schedule.len()
    );

    // Coverage: each deck × seat ≥ GAMES_PER_DECK_SEAT.
    let mut as_a = vec![0u32; decks.len()];
    let mut as_b = vec![0u32; decks.len()];
    for &(_, da, db) in &schedule {
        as_a[da] += 1;
        as_b[db] += 1;
    }
    for i in 0..decks.len() {
        assert!(
            as_a[i] >= GAMES_PER_DECK_SEAT && as_b[i] >= GAMES_PER_DECK_SEAT,
            "deck {} coverage A={} B={}",
            decks[i].stem,
            as_a[i],
            as_b[i]
        );
    }

    let parsed: Vec<Vec<CardId>> = decks.iter().map(|d| parse_deck_cards(&d.cards)).collect();
    let mut games = Vec::with_capacity(schedule.len());
    let mut schedule_h = 0xcbf2_9ce4_8422_2325u64;
    for (seed, da, db_idx) in schedule {
        let (digest, final_hash) = play_digest(db, seed, &parsed[da], &parsed[db_idx]);
        schedule_h = fnv_u64(schedule_h, digest);
        schedule_h = fnv_u64(schedule_h, final_hash);
        games.push(GoldenGame {
            seed,
            deck_a: da,
            deck_b: db_idx,
            digest: hex_u64(digest),
            final_hash: hex_u64(final_hash),
        });
    }

    let main_sha = std::env::var("ARENA_SET10_IDENTITY_SHA").unwrap_or_else(|_| {
        String::from_utf8(
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(repo_root())
                .output()
                .expect("git rev-parse")
                .stdout,
        )
        .expect("utf8")
        .trim()
        .to_string()
    });

    GoldenFile {
        main_sha,
        generated_by: format!(
            "ARENA_SET10_IDENTITY_WRITE=1 cargo test --release --test set10_identity -- --nocapture \
             on unmodified main (reads POOLS.json pool \"all\" once; freezes decks+seeds into {GOLDEN_REL})"
        ),
        games_per_deck_seat: GAMES_PER_DECK_SEAT,
        base_seed: BASE_SEED,
        decks,
        games,
        schedule_digest: hex_u64(schedule_h),
    }
}

fn verify_golden(db: &CardDb, golden: &GoldenFile) {
    assert_eq!(golden.games_per_deck_seat, GAMES_PER_DECK_SEAT);
    assert_eq!(golden.base_seed, BASE_SEED);
    assert!(
        golden.games.len() >= 300,
        "golden has {} games, need ≥ 300",
        golden.games.len()
    );
    assert!(
        golden.decks.len() >= 2,
        "golden must freeze ≥ 2 decks"
    );

    let parsed: Vec<Vec<CardId>> = golden
        .decks
        .iter()
        .map(|d| {
            assert_eq!(d.cards.len(), 40, "frozen deck {} must be 40 cards", d.stem);
            let ids = parse_deck_cards(&d.cards);
            assert!(
                deck_ready(db, &ids),
                "frozen deck {} missing cards in CardDb",
                d.stem
            );
            ids
        })
        .collect();

    // Recompute expected schedule shape from frozen deck count — must match golden.games.
    let expect = build_schedule(golden.decks.len());
    assert_eq!(
        expect.len(),
        golden.games.len(),
        "golden game count drifted from frozen schedule"
    );
    for (i, &(seed, da, db_idx)) in expect.iter().enumerate() {
        let g = &golden.games[i];
        assert_eq!(g.seed, seed, "game {i} seed");
        assert_eq!(g.deck_a, da, "game {i} deck_a");
        assert_eq!(g.deck_b, db_idx, "game {i} deck_b");
    }

    let mut schedule_h = 0xcbf2_9ce4_8422_2325u64;
    for (i, g) in golden.games.iter().enumerate() {
        let (digest, final_hash) =
            play_digest(db, g.seed, &parsed[g.deck_a], &parsed[g.deck_b]);
        assert_eq!(
            hex_u64(digest),
            g.digest,
            "game {i} seed={} digest mismatch (identity broken)",
            g.seed
        );
        assert_eq!(
            hex_u64(final_hash),
            g.final_hash,
            "game {i} seed={} final_hash mismatch (identity broken)",
            g.seed
        );
        schedule_h = fnv_u64(schedule_h, digest);
        schedule_h = fnv_u64(schedule_h, final_hash);
    }
    assert_eq!(
        hex_u64(schedule_h),
        golden.schedule_digest,
        "schedule_digest mismatch"
    );
}

#[test]
fn set10_identity_golden() {
    let db = load_db();
    let path = golden_path();
    let write = std::env::var("ARENA_SET10_IDENTITY_WRITE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let start = std::time::Instant::now();
    if write {
        let golden = write_golden(&db);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir golden dir");
        }
        let text = serde_json::to_string_pretty(&golden).expect("serialize golden");
        std::fs::write(&path, text.as_bytes()).expect("write golden");
        eprintln!(
            "set10_identity WRITE ok: {} decks, {} games, schedule={}, elapsed={:.2}s → {}",
            golden.decks.len(),
            golden.games.len(),
            golden.schedule_digest,
            start.elapsed().as_secs_f64(),
            path.display()
        );
        return;
    }

    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing {GOLDEN_REL}: {e}; generate on unmodified main with \
             ARENA_SET10_IDENTITY_WRITE=1 cargo test --release --test set10_identity -- --nocapture"
        );
    });
    let golden: GoldenFile = serde_json::from_str(&text).expect("parse golden");
    verify_golden(&db, &golden);
    let elapsed = start.elapsed().as_secs_f64();
    eprintln!(
        "set10_identity VERIFY ok: {} decks, {} games (main_sha={}), elapsed={:.2}s",
        golden.decks.len(),
        golden.games.len(),
        golden.main_sha,
        elapsed
    );
    assert!(
        elapsed < 60.0,
        "set10_identity exceeded 60s release budget: {elapsed:.2}s"
    );
}
