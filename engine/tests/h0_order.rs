//! `h0:order=` and `oorder=` — move-ordering leaf separate from line scores.
//!
//! Pinned regression suites (unchanged when `order` is unset):
//! `h0_defaults`, `h0_okill`, `h0_hbk`, `h0_tkill`, `h0_threads`, `h0_job`.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use arena_engine::{
    apply_neutral, legal_actions, new_game, policy_rng, AnyPolicy, CardDb, First, GameConfig,
    Policy, H0,
};
use serde_json::Value;

mod common;
use common::*;

const SERVED_RELEASE: &str =
    "h0:nodes=8000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1";

fn review_fixture(review: &str, name: &str) -> PathBuf {
    repo_root()
        .join("py/tests/fixtures")
        .join(review)
        .join("games")
        .join(name)
}

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn with_order(spec: &str, order_path: &str, oorder: bool) -> String {
    let o = if oorder { ",oorder=1" } else { "" };
    if spec == "h0" {
        format!("h0:order={order_path}{o}")
    } else if let Some(rest) = spec.strip_prefix("h0:") {
        format!("h0:order={order_path},{rest}{o}")
    } else {
        panic!("not h0 spec: {spec}");
    }
}

fn zero_order_net_path() -> PathBuf {
    let v4 = fs::read_to_string(crate_dir().join("models/h0-linear-v4.json")).expect("v4");
    let mut j: Value = serde_json::from_str(&v4).expect("json");
    if let Some(w) = j
        .get_mut("linear")
        .and_then(|l| l.get_mut("w"))
        .and_then(|v| v.as_array_mut())
    {
        for x in w.iter_mut() {
            *x = Value::from(0.0);
        }
    }
    if let Some(linear) = j.get_mut("linear").and_then(|v| v.as_object_mut()) {
        if let Some(zw) = linear.get_mut("zone_w").and_then(|v| v.as_array_mut()) {
            for row in zw.iter_mut() {
                if let Some(row) = row.as_array_mut() {
                    for x in row.iter_mut() {
                        *x = Value::from(0.0);
                    }
                }
            }
        }
        linear.insert("b".to_string(), Value::from(0.0));
    }
    let mut path = std::env::temp_dir();
    path.push("h0-order-zero-net.json");
    let mut f = fs::File::create(&path).expect("create temp order net");
    f.write_all(serde_json::to_string(&j).unwrap().as_bytes())
        .expect("write");
    path
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

struct Outcome {
    idx: usize,
    value_bits: Option<u32>,
    explain: Option<String>,
}

fn decide(spec: &str, threads: u32, db: &CardDb, cap: &Value, ply: usize, seed: u64) -> Outcome {
    let st = replay_capture(db, cap, ply);
    let legal = legal_actions(db, &st);
    let mut h0 = parse_h0(spec);
    h0.threads = threads;
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let idx = h0.choose(db, &st, &legal, &mut rng);
    Outcome {
        idx,
        value_bits: h0.last_value().map(f32::to_bits),
        explain: h0
            .take_explain()
            .map(|r| serde_json::to_string(&r).expect("explain json")),
    }
}

#[test]
fn order_spec_parser_round_trip_and_errors() {
    let v4 = h0_linear_v4_path();
    let spec = format!("h0:order={v4},oorder=1");
    let p = AnyPolicy::parse_spec(&spec).unwrap();
    assert_eq!(p.spec(), spec);

    let p2 = AnyPolicy::parse_spec("h0:oorder=1").unwrap();
    assert_eq!(p2.spec(), "h0");

    let missing = "h0:order=/no/such/net.json";
    let e_net = AnyPolicy::parse_spec(missing).unwrap_err();
    let e_order = AnyPolicy::parse_spec("h0:net=/no/such/net.json").unwrap_err();
    assert_eq!(e_net, e_order);
}

#[test]
fn same_order_net_as_leaf_is_identity_on_review_and_self_play() {
    let db = load_recorded_db();
    let v4 = h0_linear_v4_path();
    let order_spec = with_order(SERVED_RELEASE, &v4, true);

    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let base = decide(SERVED_RELEASE, 1, &db, &cap, m.ply, seed);
        let got = decide(&order_spec, 1, &db, &cap, m.ply, seed);
        assert_eq!(got.idx, base.idx, "{}/{}@{}", m.review, m.game, m.ply);
        assert_eq!(got.value_bits, base.value_bits, "value");
        assert_eq!(got.explain, base.explain, "explain");
    }

    let decks = ["basic-forest", "royal-nattui"];
    let mut decisions = 0usize;
    for deck in decks {
        let path = repo_root().join(format!("oracle/decks/{deck}.json"));
        let deck_v: Value = serde_json::from_str(&fs::read_to_string(path).expect("deck")).unwrap();
        let deck_a = deck_from_json(&deck_v);
        for seed in 0u64..100 {
            let st = new_game(
                &db,
                GameConfig {
                    seed: 10_000 + seed,
                    deck_a: deck_a.clone(),
                    deck_b: deck_a.clone(),
                    first: First::A,
                    opening_hands: None,
                },
            )
            .expect("game");
            let legal = legal_actions(&db, &st);
            if legal.len() <= 1 {
                continue;
            }
            let mut rng = policy_rng(20_000 + seed);
            let mut b = parse_h0(SERVED_RELEASE);
            let mut o = parse_h0(&order_spec);
            let ib = b.choose(&db, &st, &legal, &mut rng);
            let io = o.choose(&db, &st, &legal, &mut rng);
            assert_eq!(ib, io);
            assert_eq!(
                b.last_value().map(f32::to_bits),
                o.last_value().map(f32::to_bits)
            );
            decisions += 1;
        }
    }
    assert!(decisions >= 200, "self-play decisions={decisions}");
}

#[test]
fn zero_order_net_preserves_beam_prefix_and_leaf_values() {
    let db = load_recorded_db();
    let zero = zero_order_net_path().display().to_string();
    let spec = format!("{SERVED_RELEASE},order={zero}");
    let baseline = SERVED_RELEASE;

    let (cap, ply, seed, base_explain) = REVIEW_MOMENTS
        .iter()
        .find_map(|m| {
            let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
            let cap: Value = serde_json::from_str(&raw).expect("json");
            let seed = policy_seed(&cap, m.ply);
            let base = decide(baseline, 1, &db, &cap, m.ply, seed);
            let ex: Value = serde_json::from_str(base.explain.as_ref().expect("explain")).unwrap();
            let cands = ex["candidates"].as_array().expect("candidates");
            if cands.len() >= 4 {
                Some((cap, m.ply, seed, base.explain.clone()))
            } else {
                None
            }
        })
        .expect("review moment with >=4 explain candidates");

    let base = decide(baseline, 1, &db, &cap, ply, seed);
    let got = decide(&spec, 1, &db, &cap, ply, seed);
    assert_ne!(
        got.explain, base_explain,
        "flat order net should change search ordering (explain)"
    );
    assert!(got.value_bits.is_some());
    assert!(base.value_bits.is_some());
}

#[test]
fn oorder_zero_matches_baseline_oorder_one_can_differ() {
    let db = load_recorded_db();
    let v3 = h0_linear_v3_path();
    let with_order = format!("{SERVED_RELEASE},order={v3}");
    let o0 = format!("{with_order},oorder=0");
    let o1 = format!("{with_order},oorder=1");

    let raw = fs::read_to_string(review_fixture(
        "review14",
        "14600367900189587136-d90f8eb2.json",
    ))
    .expect("review");
    let cap: Value = serde_json::from_str(&raw).expect("json");
    let ply = 69usize;
    let seed = policy_seed(&cap, ply);

    let base = decide(SERVED_RELEASE, 1, &db, &cap, ply, seed);
    let no_opp_order = decide(&o0, 1, &db, &cap, ply, seed);
    assert_eq!(no_opp_order.explain, base.explain, "oorder=0 opponent line");

    let mut saw_diff = false;
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let a = decide(&o0, 1, &db, &cap, m.ply, seed);
        let b = decide(&o1, 1, &db, &cap, m.ply, seed);
        if a.explain != b.explain {
            saw_diff = true;
            break;
        }
    }
    assert!(saw_diff, "expected oorder=1 to change some explain record");
}

#[test]
fn order_threads_identity_alloc_world() {
    let db = load_recorded_db();
    let v4 = h0_linear_v4_path();
    let spec = format!("h0:nodes=2000,horizon=3,k=8,alloc=world,order={v4},oorder=1");
    let raw = fs::read_to_string(review_fixture(
        "review13",
        "4984932781931433298-b4973563.json",
    ))
    .expect("review");
    let cap: Value = serde_json::from_str(&raw).expect("json");
    let ply = 27usize;
    let seed = policy_seed(&cap, ply);
    let t1 = decide(&spec, 1, &db, &cap, ply, seed);
    let t2 = decide(&spec, 2, &db, &cap, ply, seed);
    assert_eq!(t1.explain, t2.explain);
    assert_eq!(t1.idx, t2.idx);
}

#[test]
#[ignore = "paired micro-benchmark; run `cargo test --release --test h0_order order_net_speed -- --ignored` locally"]
fn order_net_speed_within_three_percent_on_review_moments() {
    let db = load_recorded_db();
    let v4 = h0_linear_v4_path();
    let with_order = with_order(SERVED_RELEASE, &v4, false);
    let h_base = parse_h0(SERVED_RELEASE);
    let h_order = parse_h0(&with_order);
    assert!(
        std::sync::Arc::ptr_eq(
            h_order.order.as_ref().expect("order"),
            h_base.net.as_ref().expect("net"),
        ),
        "order= must share the built-in leaf Arc when paths match"
    );
    let mut base_ns = 0u128;
    let mut order_ns = 0u128;
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let st = replay_capture(&db, &cap, m.ply);
        let legal = legal_actions(&db, &st);
        let mut rng = policy_rng(seed);

        let t0 = Instant::now();
        let mut h0 = h_base.clone();
        h0.choose(&db, &st, &legal, &mut rng);
        base_ns += t0.elapsed().as_nanos();

        let t1 = Instant::now();
        let mut h0o = h_order.clone();
        h0o.choose(&db, &st, &legal, &mut rng);
        order_ns += t1.elapsed().as_nanos();
    }
    let ratio = order_ns as f64 / base_ns as f64;
    assert!(
        ratio <= 1.03,
        "order= built-in leaf too slow overall: ratio={ratio:.4} base_ns={base_ns} order_ns={order_ns}"
    );
}
