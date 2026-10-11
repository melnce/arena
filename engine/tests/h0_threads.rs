//! H0 `threads=N` identity: move, value, explain, and stats unchanged across thread counts.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use arena_engine::policy::SearchStats;
use arena_engine::{
    apply_neutral, legal_actions, new_game, policy_rng, AnyPolicy, CardDb, First, GameConfig,
    Policy, H0,
};
use serde_json::Value;

mod common;
use common::*;

const S1: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8";
const S2: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,alloc=world";
const S3: &str =
    "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,alloc=world";

const S1_DEBUG: &str =
    "h0:nodes=2000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8";
const S2_DEBUG: &str =
    "h0:nodes=2000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,alloc=world";
const S3_DEBUG: &str =
    "h0:nodes=2000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,alloc=world";

const RELEASE_THREADS: [u32; 4] = [1, 2, 3, 8];
const DEBUG_THREADS: [u32; 2] = [1, 2];

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

fn bot_policy_seed(cap: &Value, ply: usize) -> u64 {
    let base = cap["seed"].as_u64().expect("seed");
    let actions = cap["actions"].as_array().expect("actions");
    let mut n = 0u64;
    for step in actions.iter().take(ply) {
        if step.get("mulligan").is_some() || step.get("bot_value").is_some() {
            n += 1;
        }
    }
    base + n
}

fn policy_seed(cap: &Value, ply: usize) -> u64 {
    let actions = cap["actions"].as_array().expect("actions");
    let bot_steps = actions
        .iter()
        .take(ply)
        .any(|s| s.get("mulligan").is_some() || s.get("bot_value").is_some());
    if bot_steps {
        bot_policy_seed(cap, ply)
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

struct ReviewMoment {
    review: &'static str,
    game: &'static str,
    ply: usize,
}

const REVIEW_MOMENTS: [ReviewMoment; 12] = [
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 27,
    },
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 45,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 69,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 40,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 27,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 52,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 91,
    },
    ReviewMoment {
        review: "review14",
        game: "12644260486301391828-d90f8eb2.json",
        ply: 15,
    },
    ReviewMoment {
        review: "review14",
        game: "12644260486301391828-d90f8eb2.json",
        ply: 35,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 60,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 55,
    },
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 60,
    },
];

fn load_engine_fixtures(_db: &CardDb, dir: &Path, filter: fn(&str) -> bool) -> Vec<Case> {
    let mut out = Vec::new();
    for ent in fs::read_dir(dir).expect("read_dir") {
        let ent = ent.expect("dir entry");
        let path = ent.path();
        if !path.extension().is_some_and(|e| e == "json") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy();
        if !filter(&name) {
            continue;
        }
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        let ply = cap
            .get("ply")
            .and_then(|v| v.as_u64())
            .map(|p| p as usize)
            .or_else(|| ply_from_fixture_name(&name))
            .expect("ply");
        let seed = policy_seed(&cap, ply);
        let st = replay_capture(db, &cap, ply);
        if st.turn < 4 {
            continue;
        }
        out.push(Case {
            label: format!("{}/{}", dir.file_name().unwrap().to_string_lossy(), name),
            cap,
            ply,
            seed,
        });
    }
    out.sort_by(|a, b| a.label.cmp(&b.label));
    out
}

fn load_review_cases(db: &CardDb) -> Vec<Case> {
    let mut out = Vec::new();
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review game");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let st = replay_capture(db, &cap, m.ply);
        if st.turn < 4 {
            continue;
        }
        let seed = policy_seed(&cap, m.ply);
        out.push(Case {
            label: format!("{}/{}@{}", m.review, m.game, m.ply),
            cap,
            ply: m.ply,
            seed,
        });
    }
    out
}

fn collect_cases(db: &CardDb) -> Vec<Case> {
    let root = fixtures_dir();
    let mut cases = Vec::new();
    cases.extend(load_engine_fixtures(db, &root.join("tkill"), |_| true));
    cases.extend(load_engine_fixtures(db, &root.join("tkroll"), |_| true));
    cases.extend(load_engine_fixtures(db, &root.join("lethal"), |_| true));
    cases.extend(load_engine_fixtures(db, &root.join("hbcheck"), |name| {
        name.starts_with("pos-") || name.starts_with("neg-")
    }));
    cases.extend(load_review_cases(db));
    cases
}

#[derive(Clone)]
struct Outcome {
    idx: usize,
    value_bits: Option<u32>,
    explain: Option<String>,
    stats: SearchStats,
}

fn decide(spec: &str, threads: u32, db: &CardDb, cap: &Value, ply: usize, seed: u64) -> Outcome {
    let st = replay_capture(db, cap, ply);
    let legal = legal_actions(db, &st);
    let mut h0 = parse_h0(spec);
    h0.threads = threads;
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let idx = h0.choose(db, &st, &legal, &mut rng);
    let value_bits = h0.last_value().map(f32::to_bits);
    let explain = h0
        .take_explain()
        .map(|r| serde_json::to_string(&r).expect("explain json"));
    Outcome {
        idx,
        value_bits,
        explain,
        stats: h0.stats.clone(),
    }
}

fn stats_eq(a: &SearchStats, b: &SearchStats) -> bool {
    format!("{a:?}") == format!("{b:?}")
}

fn check_identity(cases: &[Case], specs: &[&str], threads: &[u32]) {
    let db = load_recorded_db();
    for spec in specs {
        for case in cases {
            let baseline = decide(spec, 1, &db, &case.cap, case.ply, case.seed);
            for &t in threads {
                if t == 1 {
                    continue;
                }
                let got = decide(spec, t, &db, &case.cap, case.ply, case.seed);
                assert_eq!(
                    got.idx, baseline.idx,
                    "{spec} threads={t} {} idx",
                    case.label
                );
                assert_eq!(
                    got.value_bits, baseline.value_bits,
                    "{spec} threads={t} {} value",
                    case.label
                );
                assert_eq!(
                    got.explain, baseline.explain,
                    "{spec} threads={t} {} explain",
                    case.label
                );
                assert!(
                    stats_eq(&got.stats, &baseline.stats),
                    "{spec} threads={t} {} stats\n{:?}\nvs\n{:?}",
                    case.label,
                    got.stats,
                    baseline.stats
                );
            }
        }
    }
}

#[test]
fn spec_alloc_world_and_threads_round_trip() {
    assert_eq!(
        AnyPolicy::parse_spec("h0:alloc=world").unwrap().spec(),
        "h0:alloc=world"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:threads=4").unwrap().spec(),
        "h0:threads=4"
    );
    let e0 = AnyPolicy::parse_spec("h0:threads=0").unwrap_err();
    assert!(e0.contains("threads"), "{e0}");
    let e65 = AnyPolicy::parse_spec("h0:threads=65").unwrap_err();
    assert!(e65.contains("threads"), "{e65}");
}

#[test]
fn threads_identity_debug_budgets() {
    let cases = debug_cases();
    assert_eq!(cases.len(), 4);
    // All three debug specs on the four pinned cases; threads ∈ {1, 2} only.
    check_identity(&cases, &[S1_DEBUG, S2_DEBUG, S3_DEBUG], &DEBUG_THREADS);
}

#[test]
#[ignore = "release identity; slow.yml"]
fn threads_identity_release_budgets() {
    let db = load_recorded_db();
    let cases = collect_cases(&db);
    assert!(
        cases.len() >= 10,
        "expected at least 10 review/engine cases, got {}",
        cases.len()
    );
    check_identity(&cases, &[S1, S2, S3], &RELEASE_THREADS);
}
