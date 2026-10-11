//! H0 stepped job API identity vs [`H0::choose`] (threads=1 baseline).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use arena_engine::policy::SearchStats;
use arena_engine::{
    apply, apply_neutral, legal_actions, new_game, policy_rng, run_job_choose, AnyPolicy, CardDb,
    First, GameConfig, Phase, PlayerId, Policy, H0,
};
use arena_engine::{MAX_ACTIONS, MAX_TURNS};
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

/// `py/serve.py` `DEFAULT_STRONG` as of base `d07388f`.
const SERVED_STRONG: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=8";
/// `py/serve.py` `DEFAULT_CHEAT` as of base `d07388f`.
const SERVED_CHEAT: &str =
    "h0:nodes=32000,horizon=3,k=8,info=all,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=8";
const STRONG_K16: &str =
    "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=8";
const STRONG_THREADS4: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=4";
const FAIR_6000: &str = "h0:nodes=6000";

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
    cap: Option<Value>,
    state: Option<arena_engine::State>,
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
        cap: Some(cap),
        state: None,
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
        cap: Some(cap),
        state: None,
        ply,
        seed,
    }
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

fn load_engine_fixtures(db: &CardDb, dir: &Path, filter: fn(&str) -> bool) -> Vec<Case> {
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
            cap: Some(cap),
            state: None,
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
            cap: Some(cap),
            state: None,
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

fn meta_deck_stems() -> Vec<String> {
    let text =
        fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .filter(|s| s != "meta-rune-test-subject")
        .collect()
}

fn load_deck_file(path: PathBuf) -> Vec<arena_engine::CardId> {
    let map: BTreeMap<String, u32> =
        serde_json::from_str(&fs::read_to_string(path).expect("deck")).expect("deck json");
    let mut ids = Vec::new();
    for (k, n) in map {
        let id = arena_engine::CardId::parse(&k).expect("card id");
        for _ in 0..n {
            ids.push(id);
        }
    }
    ids
}

/// Twenty bot Main-phase decisions at turn ≥ 4 from seeded self-play (`seed = 20261011 + g*97 + stem*131`).
fn meta_selfplay_cases(db: &CardDb) -> Vec<Case> {
    let spec = "h0";
    let mut out = Vec::new();
    for stem in meta_deck_stems() {
        let deck = load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")));
        for g in 0u64..2 {
            let seed = 20261011u64.wrapping_add(g * 97).wrapping_add(stem.len() as u64 * 131);
            let mut state = new_game(
                db,
                GameConfig {
                    seed,
                    deck_a: deck.clone(),
                    deck_b: deck.clone(),
                    first: First::A,
                    opening_hands: None,
                },
            )
            .expect("new_game");
            let mut pol = parse_h0(spec);
            let mut rng = policy_rng(seed);
            let mut nact = 0u32;
            while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
                if state.turn > MAX_TURNS || nact >= MAX_ACTIONS {
                    break;
                }
                let legal = legal_actions(db, &state);
                if legal.is_empty() {
                    break;
                }
                if state.active == PlayerId::A
                    && state.turn >= 4
                    && matches!(state.phase, Phase::Main)
                {
                    let choose_seed = seed
                        .wrapping_add(u64::from(state.turn) * 97)
                        .wrapping_add(u64::from(nact) * 131);
                    out.push(Case {
                        label: format!("selfplay/{stem}/g{g}/t{}/n{}", state.turn, nact),
                        cap: None,
                        state: Some(state.clone()),
                        ply: 0,
                        seed: choose_seed,
                    });
                    if out.len() >= 20 {
                        return out;
                    }
                }
                let idx = pol.choose(db, &state, &legal, &mut rng);
                if apply(db, &mut state, legal[idx].clone()).is_err() {
                    break;
                }
                nact += 1;
            }
        }
    }
    out
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

fn case_state(db: &CardDb, case: &Case) -> arena_engine::State {
    if let Some(st) = &case.state {
        return st.clone();
    }
    replay_capture(db, case.cap.as_ref().expect("cap"), case.ply)
}

fn decide_choose(spec: &str, db: &CardDb, case: &Case) -> Outcome {
    let st = case_state(db, case);
    let legal = legal_actions(db, &st);
    let mut h0 = parse_h0(spec);
    h0.threads = 1;
    let mut rng = policy_rng(case.seed);
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
    case: &Case,
    reverse_worlds: bool,
    roll_before_det: bool,
    serde_roundtrip: bool,
) -> Outcome {
    let st = case_state(db, case);
    let (idx, value, stats) = run_job_choose(
        db,
        &st,
        spec,
        case.seed,
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
    let baseline = decide_choose(spec, db, case);
    let got = decide_job(spec, db, case, reverse_worlds, roll_before_det, serde_roundtrip);
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
    let mut cases = collect_cases(&db);
    cases.extend(meta_selfplay_cases(&db));
    assert!(cases.len() >= 20, "fixture + self-play cases: {}", cases.len());
    check_job_identity(
        &cases,
        &[
            SERVED_STRONG,
            SERVED_CHEAT,
            STRONG_K16,
            STRONG_THREADS4,
            FAIR_6000,
            "h0",
            S1,
            S2,
            S3,
        ],
    );
}
