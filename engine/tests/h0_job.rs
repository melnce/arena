//! H0 stepped job API identity vs [`H0::choose`] (threads=1 baseline).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use arena_engine::policy::SearchStats;
use arena_engine::{
    apply_neutral, legal_actions, new_game, policy_rng, run_job_choose, AnyPolicy, CardDb, First,
    GameConfig, Policy, H0,
};
use serde_json::Value;

mod common;
use common::*;

const S1_DEBUG: &str =
    "h0:nodes=2000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8";
const S2_DEBUG: &str =
    "h0:nodes=2000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,alloc=world";
const S3_DEBUG: &str =
    "h0:nodes=2000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,alloc=world";

const S1: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8";
const S2: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,alloc=world";
const S3: &str =
    "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,alloc=world";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn deck_from_json(v: &Value) -> Vec<arena_engine::CardId> {
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

fn replay_capture(db: &CardDb, cap: &Value, n: usize) -> arena_engine::State {
    let seed = cap["seed"].as_u64().expect("seed");
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
    .expect("new_game");
    let actions = cap["actions"].as_array().expect("actions");
    for step in actions.iter().take(n) {
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
        let neu: arena_engine::NeutralAction =
            serde_json::from_value(Value::Object(body)).expect("action");
        apply_neutral(db, &mut st, &neu).expect("apply_neutral");
    }
    st
}

fn policy_seed(cap: &Value, ply: usize) -> u64 {
    let actions = cap["actions"].as_array().expect("actions");
    let bot_steps = actions
        .iter()
        .take(ply)
        .any(|s| s.get("mulligan").is_some() || s.get("bot_value").is_some());
    if bot_steps {
        let base = cap["seed"].as_u64().expect("seed");
        let mut n = 0u64;
        for step in actions.iter().take(ply) {
            if step.get("mulligan").is_some() || step.get("bot_value").is_some() {
                n += 1;
            }
        }
        base + n
    } else {
        cap["seed"].as_u64().expect("seed") + ply as u64
    }
}

fn ply_from_fixture_name(name: &str) -> Option<usize> {
    let stem = Path::new(name).file_stem()?.to_str()?;
    let ply = stem.rsplit("-ply").next()?;
    if stem.contains("-ply") {
        ply.parse().ok()
    } else {
        None
    }
}

struct Case {
    label: String,
    cap: Value,
    ply: usize,
    seed: u64,
}

fn load_fixture_case(path: &Path, label: &str) -> Case {
    let cap: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read fixture")).expect("json");
    let name = path.file_name().unwrap().to_string_lossy();
    let ply = cap
        .get("ply")
        .and_then(|v| v.as_u64())
        .map(|p| p as usize)
        .or_else(|| ply_from_fixture_name(&name))
        .expect("ply");
    let seed = policy_seed(&cap, ply);
    Case {
        label: label.to_string(),
        cap,
        ply,
        seed,
    }
}

fn review_fixture(review: &str, name: &str) -> PathBuf {
    repo_root()
        .join("py/tests/fixtures")
        .join(review)
        .join("games")
        .join(name)
}

fn load_review_case(review: &str, game: &str, ply: usize) -> Case {
    let raw = fs::read_to_string(review_fixture(review, game)).expect("review game");
    let cap: Value = serde_json::from_str(&raw).expect("json");
    let seed = policy_seed(&cap, ply);
    Case {
        label: format!("{review}/{game}@{ply}"),
        cap,
        ply,
        seed,
    }
}

fn debug_cases() -> Vec<Case> {
    let root = fixtures_dir();
    vec![
        load_fixture_case(
            &root.join("lethal/review15-g4-fuse-kill.json"),
            "lethal/review15-g4-fuse-kill.json",
        ),
        load_fixture_case(
            &root.join("tkroll/fa-play-174-ply0086.json"),
            "tkroll/fa-play-174-ply0086.json",
        ),
        load_review_case("review14", "6598261483642061665-d90f8eb2.json", 27),
        load_review_case("review14", "14600367900189587136-d90f8eb2.json", 40),
    ]
}

fn stats_eq(a: &SearchStats, b: &SearchStats) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

struct Outcome {
    idx: usize,
    value_bits: Option<u32>,
    stats: SearchStats,
}

fn decide_choose(spec: &str, db: &CardDb, cap: &Value, ply: usize, seed: u64) -> Outcome {
    let st = replay_capture(db, cap, ply);
    let legal = legal_actions(db, &st);
    let mut h0 = parse_h0(spec);
    h0.threads = 1;
    let mut rng = policy_rng(seed);
    let idx = h0.choose(db, &st, &legal, &mut rng);
    Outcome {
        idx,
        value_bits: h0.last_value().map(f32::to_bits),
        stats: h0.stats.clone(),
    }
}

fn decide_job(
    spec: &str,
    db: &CardDb,
    cap: &Value,
    ply: usize,
    seed: u64,
    reverse_worlds: bool,
    roll_before_det: bool,
    serde_roundtrip: bool,
) -> Outcome {
    let st = replay_capture(db, cap, ply);
    let (idx, value, stats) = run_job_choose(
        db,
        &st,
        spec,
        seed,
        reverse_worlds,
        roll_before_det,
        serde_roundtrip,
    )
    .unwrap_or_else(|e| panic!("job choose: {e}"));
    Outcome {
        idx,
        value_bits: value.map(f32::to_bits),
        stats,
    }
}

fn assert_job_matches(
    spec: &str,
    case: &Case,
    db: &CardDb,
    reverse_worlds: bool,
    roll_before_det: bool,
    serde_roundtrip: bool,
) {
    let baseline = decide_choose(spec, db, &case.cap, case.ply, case.seed);
    let got = decide_job(
        spec,
        db,
        &case.cap,
        case.ply,
        case.seed,
        reverse_worlds,
        roll_before_det,
        serde_roundtrip,
    );
    assert_eq!(got.idx, baseline.idx, "{} {} idx", spec, case.label);
    assert_eq!(
        got.value_bits,
        baseline.value_bits,
        "{} {} value",
        spec,
        case.label
    );
    assert!(
        stats_eq(&got.stats, &baseline.stats),
        "{} {} stats\n{:?}\nvs\n{:?}",
        spec,
        case.label,
        got.stats,
        baseline.stats
    );
}

fn check_job_identity(cases: &[Case], specs: &[&str]) {
    let db = load_recorded_db();
    for spec in specs {
        for case in cases {
            assert_job_matches(spec, case, &db, false, false, false);
            assert_job_matches(spec, case, &db, true, false, false);
            assert_job_matches(spec, case, &db, false, false, true);
            assert_job_matches(spec, case, &db, false, true, false);
        }
    }
}

#[test]
fn threads_identity_debug_budgets() {
    let cases = debug_cases();
    assert_eq!(cases.len(), 4);
    check_job_identity(&cases, &[S1_DEBUG, S2_DEBUG, S3_DEBUG]);
}

#[test]
#[ignore = "release identity; slow.yml"]
fn threads_identity_release_budgets() {
    let db = load_recorded_db();
    let root = fixtures_dir();
    let mut cases = Vec::new();
    for ent in fs::read_dir(root.join("tkill")).expect("tkill dir") {
        let path = ent.expect("ent").path();
        if path.extension().is_some_and(|e| e == "json") {
            cases.push(load_fixture_case(
                &path,
                path.file_name().unwrap().to_string_lossy().as_ref(),
            ));
        }
    }
    assert!(cases.len() >= 3);
    check_job_identity(&cases, &[S1, S2, S3]);
}
