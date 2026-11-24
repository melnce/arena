//! Per-action `hash(state)` over replayable captures — unchanged vs main baseline.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use arena_engine::{apply_neutral, hash, new_game, CardDb, First, GameConfig, NeutralAction};
use flate2::read::GzDecoder;
use serde_json::Value;

mod common;
use common::*;

const BASELINE_REL: &str = "engine/tests/fixtures/recording_action_hash_baseline.json";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Baseline {
    live_actions: u64,
    live_replays: u32,
    live_digest: String,
    recorded_actions: u64,
    recorded_replays: u32,
    recorded_digest: String,
    oracle_traces: u32,
    oracle_actions: u64,
    meta_games: u32,
    meta_actions: u64,
}

fn baseline_path() -> PathBuf {
    repo_root().join(BASELINE_REL)
}

fn fnv_chain(h: u64, v: u64) -> u64 {
    let mut h = h;
    for b in v.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn deck_from_json(v: &Value) -> Vec<arena_engine::CardId> {
    if let Some(ids) = v.as_array() {
        return ids
            .iter()
            .map(|x| arena_engine::CardId::parse(x.as_str().expect("deck id")).expect("card id"))
            .collect();
    }
    use std::collections::BTreeMap;
    let map = v.as_object().expect("deck object");
    let mut sorted = BTreeMap::new();
    for (id, n) in map {
        sorted.insert(id.clone(), n.as_u64().unwrap_or(0) as usize);
    }
    let mut out = Vec::new();
    for (id, count) in sorted {
        let cid = arena_engine::CardId::parse(&id).unwrap_or_else(|| panic!("bad id {id}"));
        out.extend(std::iter::repeat_n(cid, count));
    }
    out
}

fn replay_capture_hashes(db: &CardDb, cap: &Value) -> Result<(u64, u64), String> {
    let seed = cap["seed"]
        .as_u64()
        .or_else(|| cap["seed"].as_str().and_then(|s| s.parse().ok()))
        .expect("seed");
    let deck_a = deck_from_json(&cap["deckA"]);
    let deck_b = deck_from_json(&cap["deckB"]);
    let first = match cap.get("first").and_then(|v| v.as_str()) {
        Some("b") | Some("B") => First::B,
        Some("a") | Some("A") => First::A,
        _ => First::Coin,
    };
    let mut st = new_game(
        db,
        GameConfig {
            seed,
            deck_a,
            deck_b,
            first,
            opening_hands: None,
        },
    )
    .map_err(|e| e.to_string())?;
    let actions = cap["actions"].as_array().expect("actions");
    let mut chain = 0xcbf29ce484222325u64;
    let mut n = 0u64;
    for step in actions {
        if step.get("reseed").is_some() {
            st.reseed(step["reseed"].as_u64().expect("reseed"));
            continue;
        }
        let mut body = serde_json::Map::new();
        for (k, v) in step.as_object().expect("action object") {
            if k == "value" || k == "bot_value" {
                continue;
            }
            body.insert(k.clone(), v.clone());
        }
        let neu: NeutralAction =
            serde_json::from_value(Value::Object(body)).expect("neutral action");
        if let Err(e) = apply_neutral(db, &mut st, &neu) {
            return Err(format!("step {n}: {e:?}"));
        }
        chain = fnv_chain(chain, hash(&st));
        n += 1;
    }
    Ok((n, chain))
}

fn walk_json(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    let mut ents: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    ents.sort_by_key(|e| e.file_name());
    for e in ents {
        let p = e.path();
        if p.is_dir() {
            walk_json(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(p);
        }
    }
}

fn capture_json_files() -> Vec<PathBuf> {
    let root = repo_root();
    let mut files = Vec::new();
    for rel in [
        "engine/tests/fixtures",
        "py/tests/fixtures",
        "ui/tests/fixtures",
    ] {
        walk_json(&root.join(rel), &mut files);
    }
    files.retain(|p| {
        if p.ends_with("meta_games.json") {
            return false;
        }
        let Ok(text) = fs::read_to_string(p) else {
            return false;
        };
        let Ok(val) = serde_json::from_str::<Value>(&text) else {
            return false;
        };
        val.get("actions").and_then(|a| a.as_array()).is_some()
            && val.get("deckA").and_then(|d| d.as_object()).is_some()
            && val.get("deckB").and_then(|d| d.as_object()).is_some()
    });
    files.sort();
    files
}

fn oracle_trace_files() -> Vec<PathBuf> {
    fn walk_gz(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        for e in rd.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.is_dir() {
                walk_gz(&p, out);
            } else if p
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.ends_with(".jsonl.gz"))
            {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk_gz(&repo_root().join("oracle/traces"), &mut files);
    files.sort();
    files
}

fn gunzip(path: &Path) -> String {
    let raw = fs::read(path).unwrap();
    let mut d = GzDecoder::new(&raw[..]);
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();
    s
}

fn scan_captures(db: &CardDb) -> (u32, u64, u64) {
    let mut replays = 0u32;
    let mut actions = 0u64;
    let mut chain = 0xcbf29ce484222325u64;
    for path in capture_json_files() {
        let text = fs::read_to_string(&path).unwrap();
        let val: Value = serde_json::from_str(&text).unwrap();
        let Ok((n, h)) = replay_capture_hashes(db, &val) else {
            continue;
        };
        replays += 1;
        actions += n;
        chain = fnv_chain(chain, h);
    }
    (replays, actions, chain)
}

fn count_oracle_trace_actions() -> (u32, u64) {
    let files = oracle_trace_files();
    let mut actions = 0u64;
    for path in &files {
        let text = gunzip(path);
        actions += text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .skip(1)
            .count() as u64;
    }
    (files.len() as u32, actions)
}

fn meta_game_action_count() -> (u32, u64) {
    let path = fixtures_dir().join("h0_exact/meta_games.json");
    let games: Vec<Value> = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let n = games.len() as u32;
    let acts = games
        .iter()
        .map(|g| g["actions"].as_array().map(|a| a.len()).unwrap_or(0) as u64)
        .sum();
    (n, acts)
}

fn build_baseline(live: &CardDb, recorded: &CardDb) -> Baseline {
    let (live_replays, live_actions, live_digest) = scan_captures(live);
    let (recorded_replays, recorded_actions, recorded_digest) = scan_captures(recorded);
    let (oracle_traces, oracle_actions) = count_oracle_trace_actions();
    let (meta_games, meta_actions) = meta_game_action_count();
    Baseline {
        live_actions,
        live_replays,
        live_digest: format!("0x{live_digest:016x}"),
        recorded_actions,
        recorded_replays,
        recorded_digest: format!("0x{recorded_digest:016x}"),
        oracle_traces,
        oracle_actions,
        meta_games,
        meta_actions,
    }
}

#[test]
fn recording_action_hashes_match_baseline() {
    let merged = build_baseline(&load_db(), &load_recorded_db());
    if std::env::var("ARENA_RECORDING_HASH_WRITE").ok().as_deref() == Some("1") {
        fs::write(
            baseline_path(),
            serde_json::to_string_pretty(&merged).unwrap(),
        )
        .expect("write baseline");
        return;
    }
    let text = fs::read_to_string(baseline_path())
        .expect("baseline missing — run with ARENA_RECORDING_HASH_WRITE=1");
    let want: Baseline = serde_json::from_str(&text).expect("baseline json");
    assert_eq!(merged.live_actions, want.live_actions, "live actions");
    assert_eq!(merged.live_replays, want.live_replays, "live replays");
    assert_eq!(merged.live_digest, want.live_digest, "live digest");
    assert_eq!(
        merged.recorded_actions, want.recorded_actions,
        "recorded actions"
    );
    assert_eq!(
        merged.recorded_replays, want.recorded_replays,
        "recorded replays"
    );
    assert_eq!(
        merged.recorded_digest, want.recorded_digest,
        "recorded digest"
    );
    assert_eq!(merged.oracle_traces, want.oracle_traces, "oracle traces");
    assert_eq!(merged.oracle_actions, want.oracle_actions, "oracle actions");
    assert_eq!(merged.meta_games, want.meta_games, "meta games");
    assert_eq!(
        merged.meta_actions, want.meta_actions,
        "meta game action rows"
    );
}
