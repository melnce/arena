//! Default-off search levers `spreadpess` and `wargmax`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use arena_engine::{
    apply, apply_neutral, legal_actions, new_game, policy_rng, AnyPolicy, CardDb, First,
    GameConfig, Phase, Policy, State, H0, MAX_ACTIONS, MAX_TURNS,
};
use serde_json::Value;

mod common;
use common::*;

const SWEEP_STACK: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,mull=leaf,mulls=256";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecideOutcome {
    idx: usize,
    value_bits: Option<u32>,
    explain: Option<String>,
    nodes: u32,
}

fn decide_state(spec: &str, db: &CardDb, st: &State, seed: u64) -> DecideOutcome {
    let legal = legal_actions(db, st);
    let mut h0 = parse_h0(spec);
    h0.threads = 1;
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let idx = h0.choose(db, st, &legal, &mut rng);
    DecideOutcome {
        idx,
        value_bits: h0.last_value().map(f32::to_bits),
        explain: h0
            .take_explain()
            .map(|r| serde_json::to_string(&r).expect("explain json")),
        nodes: h0.stats.nodes as u32,
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

fn review_fixture(review: &str, name: &str) -> PathBuf {
    repo_root()
        .join("py/tests/fixtures")
        .join(review)
        .join("games")
        .join(name)
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

fn replay_capture(db: &CardDb, cap: &Value, n: usize) -> State {
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

struct SelfPlayCase {
    state: State,
    seed: u64,
    label: String,
}

fn meta_selfplay_cases(db: &CardDb) -> Vec<SelfPlayCase> {
    let decks = load_pinned_corpus_decks_sorted(db);
    let mut out = Vec::new();
    let mut seed = 9_000u64;
    while out.len() < 40 {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 7) % decks.len()];
        let Ok(mut state) = new_game(
            db,
            GameConfig {
                seed,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: First::A,
                opening_hands: None,
            },
        ) else {
            seed += 1;
            continue;
        };
        let mut rng = policy_rng(seed);
        let mut steps = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            if steps >= 12 && matches!(state.phase, Phase::Main) && state.turn >= 4 {
                let legal = legal_actions(db, &state);
                if legal.len() > 1 {
                    out.push(SelfPlayCase {
                        state: state.clone(),
                        seed: seed.wrapping_add(steps as u64),
                        label: format!("selfplay seed {seed} step {steps}"),
                    });
                    if out.len() >= 40 {
                        break;
                    }
                }
            }
            if state.turn > MAX_TURNS || steps >= MAX_ACTIONS {
                break;
            }
            let legal = legal_actions(db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            let a = legal[idx].clone();
            if apply(db, &mut state, a).is_err() {
                break;
            }
            steps += 1;
        }
        seed += 1;
    }
    out
}

#[test]
fn spec_spreadpess_wargmax() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:spreadpess=0").unwrap().spec(),
        "h0"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:spreadpess=0.25").unwrap().spec(),
        "h0:spreadpess=0.25"
    );
    assert_eq!(AnyPolicy::parse_spec("h0:wargmax=0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:wargmax=1").unwrap().spec(),
        "h0:wargmax=1"
    );
    for bad in ["h0:spreadpess=1.5", "h0:spreadpess=x", "h0:wargmax=2"] {
        assert!(AnyPolicy::parse_spec(bad).is_err(), "{bad}");
    }
    let served = AnyPolicy::parse_spec(SWEEP_STACK).unwrap().spec();
    assert!(!served.contains("spreadpess"));
    assert!(!served.contains("wargmax"));
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn levers_off_identity_review_and_selfplay() {
    let db = load_recorded_db();
    let base = SWEEP_STACK;
    for extra in [",spreadpess=0", ",wargmax=0", ",spreadpess=0,wargmax=0"] {
        let spec = format!("{base}{extra}");
        for m in REVIEW_MOMENTS {
            let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
            let cap: Value = serde_json::from_str(&raw).expect("json");
            let seed = policy_seed(&cap, m.ply);
            let st = replay_capture(&db, &cap, m.ply);
            let a = decide_state(base, &db, &st, seed);
            let b = decide_state(&spec, &db, &st, seed);
            assert_eq!(b.idx, a.idx, "{}/{}@{}", m.review, m.game, m.ply);
            assert_eq!(b.value_bits, a.value_bits, "value");
            assert_eq!(b.explain, a.explain, "explain");
            assert_eq!(b.nodes, a.nodes, "nodes");
        }
    }
    let cases = meta_selfplay_cases(&db);
    assert!(cases.len() >= 40, "cases={}", cases.len());
    for case in &cases {
        let a = decide_state(base, &db, &case.state, case.seed);
        let b = decide_state(
            &format!("{base},spreadpess=0,wargmax=0"),
            &db,
            &case.state,
            case.seed,
        );
        assert_eq!(b.idx, a.idx, "{}", case.label);
        assert_eq!(b.value_bits, a.value_bits, "value {}", case.label);
        assert_eq!(b.explain, a.explain, "explain {}", case.label);
        assert_eq!(b.nodes, a.nodes, "nodes {}", case.label);
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn spreadpess_or_wargmax_moves_on_scan() {
    let db = load_recorded_db();
    let base = SWEEP_STACK;
    let mut found_spread = false;
    let mut found_vote = false;
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let st = replay_capture(&db, &cap, m.ply);
        let plain = decide_state(base, &db, &st, seed);
        if !found_spread {
            let sp = decide_state(&format!("{base},spreadpess=0.5"), &db, &st, seed);
            if sp.idx != plain.idx || sp.value_bits != plain.value_bits {
                found_spread = true;
            }
        }
        if !found_vote {
            let wv = decide_state(&format!("{base},wargmax=1"), &db, &st, seed);
            if wv.idx != plain.idx {
                found_vote = true;
            }
        }
    }
    for case in meta_selfplay_cases(&db) {
        let plain = decide_state(base, &db, &case.state, case.seed);
        if !found_spread {
            let sp = decide_state(
                &format!("{base},spreadpess=0.5"),
                &db,
                &case.state,
                case.seed,
            );
            if sp.idx != plain.idx || sp.value_bits != plain.value_bits {
                found_spread = true;
            }
        }
        if !found_vote {
            let wv = decide_state(&format!("{base},wargmax=1"), &db, &case.state, case.seed);
            if wv.idx != plain.idx {
                found_vote = true;
            }
        }
        if found_spread && found_vote {
            break;
        }
    }
    assert!(
        found_spread,
        "spreadpess=0.5 must change a decision on scan"
    );
    assert!(found_vote, "wargmax=1 must change a decision on scan");
}

#[test]
#[ignore = "writes /opt/cursor/artifacts/h0_search_profile.json"]
fn write_search_profile_artifact() {
    use std::time::Instant;
    let db = load_recorded_db();
    let specs = [
        ("sweep_scale", SWEEP_STACK),
        (
            "served",
            "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,mull=leaf,mulls=256",
        ),
    ];
    let mut report = serde_json::Map::new();
    for (label, spec) in specs {
        let mut ms = Vec::new();
        let mut nodes = Vec::new();
        let mut spreads = Vec::new();
        let mut margins = Vec::new();
        for m in REVIEW_MOMENTS {
            let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
            let cap: Value = serde_json::from_str(&raw).expect("json");
            let seed = policy_seed(&cap, m.ply);
            let st = replay_capture(&db, &cap, m.ply);
            let t0 = Instant::now();
            let out = decide_state(spec, &db, &st, seed);
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            nodes.push(out.nodes as f64);
            if let Some(explain) = out.explain {
                let v: Value = serde_json::from_str(&explain).expect("json");
                let mut world_spread = 0.0f32;
                let mut aggs = Vec::new();
                for c in v["candidates"].as_array().into_iter().flatten() {
                    let mut raw = Vec::new();
                    for w in c["worlds"].as_array().into_iter().flatten() {
                        if w["skipped"].as_bool() == Some(true) {
                            continue;
                        }
                        if let Some(r) = w["raw"].as_f64() {
                            raw.push(r as f32);
                        }
                    }
                    if raw.len() >= 2 {
                        world_spread = world_spread.max(
                            raw.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
                                - raw.iter().cloned().fold(f32::INFINITY, f32::min),
                        );
                    }
                    if let Some(a) = c["root_agg"].as_f64() {
                        aggs.push(a as f32);
                    }
                }
                spreads.push(world_spread as f64);
                aggs.sort_by(|a, b| b.partial_cmp(a).unwrap());
                if aggs.len() >= 2 {
                    margins.push((aggs[0] - aggs[1]) as f64);
                }
            }
        }
        let mean = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len().max(1) as f64;
        report.insert(
            label.into(),
            serde_json::json!({
                "wall_ms_mean": mean(&ms),
                "nodes_mean": mean(&nodes),
                "world_spread_mean": mean(&spreads),
                "margin_mean": mean(&margins),
                "n_review": ms.len(),
            }),
        );
    }
    let path = std::path::Path::new("/opt/cursor/artifacts/h0_search_profile.json");
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    std::fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::Value::Object(report)).unwrap(),
    )
    .expect("write profile");
}
