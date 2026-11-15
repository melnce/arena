//! Wall-time table for fixed review + meta self-play decisions (`threads=1`).
//!
//! Run: `cargo test --release -p arena-engine --test h0_speed_profile -- --ignored --nocapture`

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use arena_engine::{
    apply, apply_neutral, legal_actions, new_game, policy_rng, AnyPolicy, CardDb, First,
    GameConfig, Phase, Policy, H0,
};
use serde_json::Value;

mod common;
use common::*;

const SWEEP: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=h0-value-v87,mull=leaf,mulls=256";
const SERVED: &str = "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=h0-value-v87,mull=leaf,mulls=256";
const QUARTER_K4: &str = "h0:nodes=8000,horizon=3,k=4,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=h0-value-v87,mull=leaf,mulls=256";
const QUARTER_K8: &str = "h0:nodes=8000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=h0-value-v87,mull=leaf,mulls=256";

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

fn review_fixture(review: &str, name: &str) -> PathBuf {
    repo_root()
        .join("py/tests/fixtures")
        .join(review)
        .join("games")
        .join(name)
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
    for step in cap["actions"].as_array().expect("actions").iter().take(n) {
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

fn deck_from_json(v: &Value) -> Vec<arena_engine::CardId> {
    let map = v.as_object().expect("deck object");
    let mut sorted = std::collections::BTreeMap::new();
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

struct Case {
    #[allow(dead_code)]
    label: String,
    state: arena_engine::State,
    seed: u64,
}

fn review_cases(db: &CardDb) -> Vec<Case> {
    let mut out = Vec::new();
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let st = replay_capture(db, &cap, m.ply);
        if st.turn < 4 {
            continue;
        }
        out.push(Case {
            label: format!("{}/{}@{}", m.review, m.game, m.ply),
            state: st,
            seed: policy_seed(&cap, m.ply),
        });
    }
    out
}

fn meta_selfplay_cases(db: &CardDb) -> Vec<Case> {
    let text = fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS");
    let pools: Value = serde_json::from_str(&text).expect("json");
    let stems: Vec<String> = pools["meta"]
        .as_array()
        .expect("meta")
        .iter()
        .map(|v| v.as_str().expect("stem").to_string())
        .filter(|s| s != "meta-rune-test-subject")
        .collect();
    let mut out = Vec::new();
    for stem in stems {
        let deck = load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")));
        for g in 0u64..3 {
            let seed = 20261011u64
                .wrapping_add(g * 97)
                .wrapping_add(stem.len() as u64 * 131);
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
            let mut pol = match AnyPolicy::parse_spec("h0").expect("h0") {
                AnyPolicy::H0(h) => h,
                _ => panic!("h0"),
            };
            let mut rng = policy_rng(seed);
            let mut nact = 0u32;
            while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
                let legal = legal_actions(db, &state);
                if legal.is_empty() {
                    break;
                }
                if state.active == arena_engine::PlayerId::A
                    && state.turn >= 4
                    && matches!(state.phase, Phase::Main)
                {
                    let choose_seed = seed
                        .wrapping_add(u64::from(state.turn) * 97)
                        .wrapping_add(u64::from(nact) * 131);
                    out.push(Case {
                        label: format!("meta/{stem}/g{g}/t{}", state.turn),
                        state: state.clone(),
                        seed: choose_seed,
                    });
                    if out.len() >= 40 {
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

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

#[derive(serde::Serialize)]
struct Row {
    wall_ms_mean: f64,
    nodes_mean: f64,
    roots_mean: f64,
    opp_open_nodes_mean: f64,
    opp_lethal_nodes_mean: f64,
    hb_nodes_mean: f64,
    nodes_per_cap_hit: f64,
    n: usize,
}

fn bench_spec(db: &CardDb, spec: &str, cases: &[Case]) -> Row {
    let mut ms = Vec::with_capacity(cases.len());
    let mut nodes = Vec::with_capacity(cases.len());
    let mut roots = Vec::with_capacity(cases.len());
    let mut opp_open = Vec::with_capacity(cases.len());
    let mut opp_lethal = Vec::with_capacity(cases.len());
    let mut hb = Vec::with_capacity(cases.len());
    let mut cap_hits = 0u64;
    for case in cases {
        let legal = legal_actions(db, &case.state);
        let mut h0 = parse_h0(spec);
        h0.threads = 1;
        let mut rng = policy_rng(case.seed);
        let t0 = Instant::now();
        let _ = h0.choose(db, &case.state, &legal, &mut rng);
        ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        let st = &h0.stats;
        nodes.push(st.nodes as f64);
        roots.push(st.roots as f64);
        opp_open.push(st.opp_open_nodes as f64);
        opp_lethal.push(st.opp_lethal_nodes as f64);
        hb.push(st.hb_nodes as f64 + st.hbk_nodes as f64);
        cap_hits += st.cap_hits;
    }
    let mean = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len().max(1) as f64;
    Row {
        wall_ms_mean: mean(&ms),
        nodes_mean: mean(&nodes),
        roots_mean: mean(&roots),
        opp_open_nodes_mean: mean(&opp_open),
        opp_lethal_nodes_mean: mean(&opp_lethal),
        hb_nodes_mean: mean(&hb),
        nodes_per_cap_hit: cap_hits as f64 / cases.len() as f64,
        n: cases.len(),
    }
}

#[test]
#[ignore = "writes /opt/cursor/artifacts/h0_speed_timing.json"]
fn write_speed_timing_artifact() {
    let db = load_recorded_db();
    let mut cases = review_cases(&db);
    cases.extend(meta_selfplay_cases(&db));
    let report = serde_json::json!({
        "build": "release",
        "cases": cases.len(),
        "review_only": bench_spec(&db, SWEEP, &review_cases(&db)),
        "sweep": bench_spec(&db, SWEEP, &cases),
        "served": bench_spec(&db, SERVED, &cases),
        "quarter_k4": bench_spec(&db, QUARTER_K4, &cases),
        "quarter_k8": bench_spec(&db, QUARTER_K8, &cases),
        "machine": {
            "nproc": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0),
            "cpu": std::fs::read_to_string("/proc/cpuinfo")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("model name"))
                        .map(|l| l.split_once(':').map(|(_, v)| v.trim()).unwrap_or(""))
                        .map(str::to_string)
                }),
        },
    });
    let path = std::path::Path::new("/opt/cursor/artifacts/h0_speed_timing.json");
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    fs::write(path, serde_json::to_string_pretty(&report).unwrap()).expect("write");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

#[cfg(feature = "h0-prof")]
fn bench_spec_prof(db: &CardDb, spec: &str, cases: &[Case]) -> arena_engine::DecisionProf {
    use arena_engine::{merge_decisions, take_decision};
    let mut rows = Vec::with_capacity(cases.len());
    for case in cases {
        let legal = legal_actions(db, &case.state);
        let mut h0 = parse_h0(spec);
        h0.threads = 1;
        let mut rng = policy_rng(case.seed);
        let t0 = Instant::now();
        let _ = h0.choose(db, &case.state, &legal, &mut rng);
        rows.push(take_decision(t0.elapsed()));
    }
    merge_decisions(&rows)
}

#[test]
#[ignore = "writes /opt/cursor/artifacts/h0_prof_sweep_served.json"]
#[cfg(feature = "h0-prof")]
fn write_prof_artifact() {
    let db = load_recorded_db();
    let mut cases = review_cases(&db);
    cases.extend(meta_selfplay_cases(&db));
    let report = serde_json::json!({
        "cases": cases.len(),
        "sweep": bench_spec_prof(&db, SWEEP, &cases),
        "served": bench_spec_prof(&db, SERVED, &cases),
    });
    let path = std::path::Path::new("/opt/cursor/artifacts/h0_prof_sweep_served.json");
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    fs::write(path, serde_json::to_string_pretty(&report).unwrap()).expect("write");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

fn quarter_selfplay_game_seconds(db: &CardDb, _spec: &str, k: u8) -> f64 {
    let spec = if k == 4 {
        QUARTER_K4.to_string()
    } else {
        QUARTER_K8.to_string()
    };
    let text = fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS");
    let pools: Value = serde_json::from_str(&text).expect("json");
    let stem = pools["meta"]
        .as_array()
        .expect("meta")
        .iter()
        .find(|v| v.as_str() != Some("meta-rune-test-subject"))
        .and_then(|v| v.as_str())
        .expect("stem");
    let deck = load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")));
    let seed = 20261099u64;
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck.clone(),
            deck_b: deck,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut pol = parse_h0(&spec);
    pol.threads = 1;
    let mut rng = policy_rng(seed);
    let t0 = Instant::now();
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        let legal = legal_actions(db, &state);
        if legal.is_empty() {
            break;
        }
        let idx = pol.choose(db, &state, &legal, &mut rng);
        if apply(db, &mut state, legal[idx].clone()).is_err() {
            break;
        }
    }
    t0.elapsed().as_secs_f64()
}

#[test]
#[ignore = "quarter-scale self-play wall time"]
fn write_quarter_selfplay_timing() {
    let db = load_recorded_db();
    let report = serde_json::json!({
        "quarter_k4_game_s": quarter_selfplay_game_seconds(&db, QUARTER_K4, 4),
        "quarter_k8_game_s": quarter_selfplay_game_seconds(&db, QUARTER_K8, 8),
    });
    let path = std::path::Path::new("/opt/cursor/artifacts/h0_quarter_selfplay.json");
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    fs::write(path, serde_json::to_string_pretty(&report).unwrap()).expect("write");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
