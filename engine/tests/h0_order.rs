//! `h0:order=` and `oorder=` — move-ordering leaf separate from line scores.
//!
//! Pinned regression suites (unchanged when `order` is unset):
//! `h0_defaults`, `h0_okill`, `h0_hbk`, `h0_tkill`, `h0_threads`, `h0_job`.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use arena_engine::trace::fnv1a64;
use arena_engine::{
    apply, apply_neutral, legal_actions, new_game, policy_rng, AnyPolicy, CardDb, First,
    GameConfig, Phase, PlayerId, Policy, State, H0,
};
use arena_engine::{MAX_ACTIONS, MAX_TURNS};
use serde_json::Value;

mod common;
use common::*;

const SERVED_RELEASE: &str =
    "h0:nodes=8000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1";

const SERVED_LEAF_RELEASE: &str =
    "h0:nodes=8000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40";

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

fn order_copy_of_v4_elsewhere() -> PathBuf {
    let src = crate_dir().join("models/h0-linear-v4.json");
    let mut path = std::env::temp_dir();
    path.push("h0-order-v4-copy.json");
    fs::copy(&src, &path).expect("copy v4");
    path
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

    let v4 = h0_linear_v4_path();
    let both = format!("h0:net={v4},order={v4}");
    let h = parse_h0(&both);
    assert!(std::sync::Arc::ptr_eq(
        h.order.as_ref().expect("order"),
        h.net.as_ref().expect("net"),
    ));

    let builtin = AnyPolicy::parse_spec("h0:order=builtin").unwrap();
    assert_eq!(builtin.spec(), "h0:order=builtin");
    let builtin_o = AnyPolicy::parse_spec("h0:order=builtin,oorder=1").unwrap();
    assert_eq!(builtin_o.spec(), "h0:order=builtin,oorder=1");

    let v53 = h0_order_v53_path();
    let rt_builtin = AnyPolicy::parse_spec("h0:order=builtin,oorder=builtin").unwrap();
    assert_eq!(rt_builtin.spec(), "h0:order=builtin,oorder=builtin");
    let rt_split = AnyPolicy::parse_spec(&format!("h0:order={v4},oorder={v53}")).unwrap();
    assert_eq!(rt_split.spec(), format!("h0:order={v4},oorder={v53}"));
    let rt_oonly = AnyPolicy::parse_spec(&format!("h0:oorder={v53}")).unwrap();
    assert_eq!(rt_oonly.spec(), format!("h0:oorder={v53}"));

    assert_eq!(
        AnyPolicy::parse_spec("h0:oorder=").unwrap_err(),
        "missing key 'oorder'"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:oorder=2").unwrap_err(),
        "unknown oorder '2'"
    );
    let e_oorder_net = AnyPolicy::parse_spec("h0:oorder=/no/such/net.json").unwrap_err();
    assert_eq!(e_oorder_net, e_net);

    let same = format!("h0:order={v4},oorder={v4}");
    let h_same = parse_h0(&same);
    assert!(std::sync::Arc::ptr_eq(
        h_same.order.as_ref().expect("order"),
        h_same.oorder_net.as_ref().expect("oorder"),
    ));

    let h_ob = parse_h0("h0:oorder=builtin");
    assert!(std::sync::Arc::ptr_eq(
        h_ob.oorder_net.as_ref().expect("oorder"),
        &arena_engine::builtin_order(),
    ));

    let net_o = format!("h0:net={v4},oorder={v4}");
    let h_net_o = parse_h0(&net_o);
    assert!(std::sync::Arc::ptr_eq(
        h_net_o.oorder_net.as_ref().expect("oorder"),
        h_net_o.net.as_ref().expect("net"),
    ));

    let h_b = parse_h0("h0:order=builtin");
    assert!(
        std::sync::Arc::ptr_eq(
            h_b.order.as_ref().expect("order"),
            &arena_engine::builtin_order(),
        ),
        "order=builtin must load embedded h0-order-v53"
    );
}

fn h0_order_v53_path() -> String {
    repo_root()
        .join("engine/models/h0-order-v53.json")
        .display()
        .to_string()
}

fn h0_order_v59_path() -> String {
    repo_root()
        .join("engine/models/h0-order-v59.json")
        .display()
        .to_string()
}

fn h0_value_v64_path() -> String {
    repo_root()
        .join("engine/models/h0-value-v64.json")
        .display()
        .to_string()
}

struct SelfPlayCase {
    label: String,
    state: State,
    seed: u64,
}

fn meta_deck_stems() -> Vec<String> {
    let text = fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
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

/// Main-phase bot decisions at turn ≥ 4 from seeded self-play (`seed = 20261011 + g*97 + stem*131`).
fn meta_selfplay_cases(db: &CardDb) -> Vec<SelfPlayCase> {
    let spec = "h0";
    let mut out = Vec::new();
    for stem in meta_deck_stems() {
        let deck = load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")));
        for g in 0u64..2 {
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
                    out.push(SelfPlayCase {
                        label: format!("selfplay/{stem}/g{g}/t{}/n{}", state.turn, nact),
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

fn decide_state(spec: &str, threads: u32, db: &CardDb, st: &State, seed: u64) -> Outcome {
    let legal = legal_actions(db, st);
    let mut h0 = parse_h0(spec);
    h0.threads = threads;
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let idx = h0.choose(db, st, &legal, &mut rng);
    Outcome {
        idx,
        value_bits: h0.last_value().map(f32::to_bits),
        explain: h0
            .take_explain()
            .map(|r| serde_json::to_string(&r).expect("explain json")),
    }
}

fn explain_used_search(explain: &str, legal_len: usize) -> bool {
    if legal_len <= 1 {
        return false;
    }
    let ex: Value = serde_json::from_str(explain).expect("explain json");
    if ex.get("mulligan").is_some() {
        return false;
    }
    if ex.get("kill").is_some() || ex.get("tkill").is_some() {
        return false;
    }
    ex["nodes"].as_u64().unwrap_or(0) > 0
}

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn same_order_net_as_leaf_is_identity_on_review_and_self_play() {
    let db = load_recorded_db();
    let order_path = order_copy_of_v4_elsewhere().display().to_string();
    let order_spec = with_order(SERVED_RELEASE, &order_path, true);
    let h = parse_h0(&order_spec);
    assert!(
        !std::sync::Arc::ptr_eq(
            h.order.as_ref().expect("order"),
            h.net.as_ref().expect("net"),
        ),
        "copy path must load order from disk, not share built-in leaf"
    );

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

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn builtin_order_matches_file_on_review_and_self_play() {
    let db = load_recorded_db();
    let file_path = h0_order_v53_path();
    let file_spec = with_order(SERVED_RELEASE, &file_path, true);
    let builtin_spec = format!("{SERVED_RELEASE},order=builtin,oorder=1");

    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let from_file = decide(&file_spec, 1, &db, &cap, m.ply, seed);
        let from_builtin = decide(&builtin_spec, 1, &db, &cap, m.ply, seed);
        assert_eq!(
            from_builtin.idx, from_file.idx,
            "{}/{}@{}",
            m.review, m.game, m.ply
        );
        assert_eq!(from_builtin.value_bits, from_file.value_bits, "value");
        assert_eq!(from_builtin.explain, from_file.explain, "explain");
    }

    let cases = meta_selfplay_cases(&db);
    assert!(cases.len() >= 40, "self-play cases={}", cases.len());
    for case in &cases {
        let from_builtin = decide_state(&builtin_spec, 1, &db, &case.state, case.seed);
        let from_file = decide_state(&file_spec, 1, &db, &case.state, case.seed);
        assert_eq!(from_builtin.idx, from_file.idx, "{}", case.label);
        assert_eq!(
            from_builtin.value_bits, from_file.value_bits,
            "value {}",
            case.label
        );
        assert_eq!(
            from_builtin.explain, from_file.explain,
            "explain {}",
            case.label
        );
    }
}

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
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

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
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

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn oorder_model_matches_oorder_one_when_same() {
    let db = load_recorded_db();
    let v53 = h0_order_v53_path();
    let specs = [
        (
            format!("{SERVED_RELEASE},order=builtin,oorder=builtin"),
            format!("{SERVED_RELEASE},order=builtin,oorder=1"),
        ),
        (
            format!("{SERVED_RELEASE},order={v53},oorder={v53}"),
            format!("{SERVED_RELEASE},order={v53},oorder=1"),
        ),
    ];
    for (model_spec, one_spec) in specs {
        for m in REVIEW_MOMENTS {
            let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
            let cap: Value = serde_json::from_str(&raw).expect("json");
            let seed = policy_seed(&cap, m.ply);
            let a = decide(&model_spec, 1, &db, &cap, m.ply, seed);
            let b = decide(&one_spec, 1, &db, &cap, m.ply, seed);
            assert_eq!(a.idx, b.idx, "{}/{}@{}", m.review, m.game, m.ply);
            assert_eq!(a.value_bits, b.value_bits, "value");
            assert_eq!(a.explain, b.explain, "explain");
        }
    }
    let extra = format!("{SERVED_RELEASE},oopen=1,onext=40");
    let model_extra = format!("{extra},order=builtin,oorder=builtin");
    let one_extra = format!("{extra},order=builtin,oorder=1");
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let a = decide(&model_extra, 1, &db, &cap, m.ply, seed);
        let b = decide(&one_extra, 1, &db, &cap, m.ply, seed);
        assert_eq!(a.idx, b.idx, "oopen {}/{}@{}", m.review, m.game, m.ply);
        assert_eq!(a.value_bits, b.value_bits, "value oopen");
        assert_eq!(a.explain, b.explain, "explain oopen");
    }
}

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn oorder_zero_net_differs_from_oorder_one() {
    let db = load_recorded_db();
    let v53 = h0_order_v53_path();
    let zero = zero_order_net_path().display().to_string();
    let same_opp = format!("{SERVED_RELEASE},order={v53},oorder=1");
    let split = format!("{SERVED_RELEASE},order={v53},oorder={zero}");
    let mut saw_diff = false;
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let a = decide(&same_opp, 1, &db, &cap, m.ply, seed);
        let b = decide(&split, 1, &db, &cap, m.ply, seed);
        if a.idx != b.idx || a.value_bits != b.value_bits || a.explain != b.explain {
            saw_diff = true;
            eprintln!("oorder split differs at {}/{}@{}", m.review, m.game, m.ply);
            break;
        }
    }
    assert!(
        saw_diff,
        "expected order=v53,oorder=zero to differ from oorder=1"
    );
}

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
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

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn oorder_threads_identity_alloc_world() {
    let db = load_recorded_db();
    let v53 = h0_order_v53_path();
    let zero = zero_order_net_path().display().to_string();
    let spec = format!("{SERVED_RELEASE},order={v53},oorder={zero},alloc=world");
    let raw = fs::read_to_string(review_fixture(
        "review13",
        "4984932781931433298-b4973563.json",
    ))
    .expect("review");
    let cap: Value = serde_json::from_str(&raw).expect("json");
    let ply = 27usize;
    let seed = policy_seed(&cap, ply);
    let t1 = decide(&spec, 1, &db, &cap, ply, seed);
    let t4 = decide(&spec, 4, &db, &cap, ply, seed);
    assert_eq!(t1.explain, t4.explain);
    assert_eq!(t1.idx, t4.idx);
}

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn zero_residual_mlp_order_matches_v53_file() {
    let db = load_recorded_db();
    let v53 = h0_order_v53_path();
    let mut zero_path = std::env::temp_dir();
    zero_path.push("h0-order-zero-residual-mlp.json");
    let tiny =
        fs::read_to_string(repo_root().join("py/tests/fixtures/order_residual/tiny-residual.json"))
            .expect("tiny");
    let mut j: Value = serde_json::from_str(&tiny).expect("json");
    let hidden = j["mlp"]["w2"].as_array().expect("w2").len();
    j["mlp"]["w2"] = Value::Array(vec![Value::from(0.0); hidden]);
    j["mlp"]["b2"] = Value::from(0.0);
    fs::write(&zero_path, serde_json::to_string(&j).unwrap()).expect("write zero");
    let zero = zero_path.display().to_string();
    let base_spec = format!("{SERVED_RELEASE},order={v53},oorder=1");
    let zero_spec = format!("{SERVED_RELEASE},order={zero},oorder=1");

    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let from_v53 = decide(&base_spec, 1, &db, &cap, m.ply, seed);
        let from_zero = decide(&zero_spec, 1, &db, &cap, m.ply, seed);
        assert_eq!(
            from_zero.idx, from_v53.idx,
            "{}/{}@{}",
            m.review, m.game, m.ply
        );
        assert_eq!(from_zero.value_bits, from_v53.value_bits, "value");
        let ex_v53: Value =
            serde_json::from_str(from_v53.explain.as_ref().expect("explain")).unwrap();
        let ex_zero: Value =
            serde_json::from_str(from_zero.explain.as_ref().expect("explain")).unwrap();
        assert_eq!(ex_zero["nodes"], ex_v53["nodes"], "nodes");
    }
}

#[test]
fn order_v59_builtin_names_and_round_trip() {
    let h_v59 = parse_h0("h0:order=h0-order-v59");
    assert!(std::sync::Arc::ptr_eq(
        h_v59.order.as_ref().expect("order"),
        &arena_engine::builtin_order_v59(),
    ));
    let h_v53 = parse_h0("h0:order=h0-order-v53");
    assert!(std::sync::Arc::ptr_eq(
        h_v53.order.as_ref().expect("order"),
        &arena_engine::builtin_order(),
    ));
    let h_oo = parse_h0("h0:oorder=h0-order-v53");
    assert!(std::sync::Arc::ptr_eq(
        h_oo.oorder_net.as_ref().expect("oorder"),
        &arena_engine::builtin_order(),
    ));
    let h_same = parse_h0("h0:order=h0-order-v59,oorder=h0-order-v59");
    assert!(std::sync::Arc::ptr_eq(
        h_same.order.as_ref().expect("order"),
        h_same.oorder_net.as_ref().expect("oorder"),
    ));
    let rt = AnyPolicy::parse_spec("h0:order=h0-order-v59,oorder=h0-order-v53").unwrap();
    assert_eq!(rt.spec(), "h0:order=h0-order-v59,oorder=h0-order-v53");
}

#[test]
fn h0_order_v59_file_is_sweep_59() {
    let raw = fs::read_to_string(crate_dir().join("models/h0-order-v59.json")).expect("v59 file");
    let trimmed = raw.trim_end_matches(&['\r', '\n'][..]);
    assert_eq!(trimmed.len(), 2_141_245);
    assert_eq!(fnv1a64(trimmed.as_bytes()), 0x0452_0aa2_fc66_a8e1);
}

#[test]
fn value_v64_builtin_names_and_round_trip() {
    let h_v64 = parse_h0("h0:net=h0-value-v64");
    assert!(std::sync::Arc::ptr_eq(
        h_v64.net.as_ref().expect("net"),
        &arena_engine::builtin_value_v64(),
    ));
    let h_v4 = parse_h0("h0:net=h0-linear-v4");
    assert!(std::sync::Arc::ptr_eq(
        h_v4.net.as_ref().expect("net"),
        &arena_engine::builtin_net(),
    ));
    let h_bi = parse_h0("h0:net=builtin");
    assert!(std::sync::Arc::ptr_eq(
        h_bi.net.as_ref().expect("net"),
        &arena_engine::builtin_net(),
    ));
    let h_triple = parse_h0("h0:net=h0-value-v64,order=h0-value-v64,oorder=h0-value-v64");
    assert!(std::sync::Arc::ptr_eq(
        h_triple.net.as_ref().expect("net"),
        h_triple.order.as_ref().expect("order"),
    ));
    assert!(std::sync::Arc::ptr_eq(
        h_triple.net.as_ref().expect("net"),
        h_triple.oorder_net.as_ref().expect("oorder"),
    ));
    assert!(AnyPolicy::parse_spec("h0:value=v0,net=h0-value-v64")
        .unwrap_err()
        .contains("value=net"));
    let spec = format!("{SERVED_LEAF_RELEASE},net=h0-value-v64");
    let p = AnyPolicy::parse_spec(&spec).unwrap();
    let p2 = AnyPolicy::parse_spec(&p.spec()).unwrap();
    assert_eq!(p, p2);
}

#[test]
fn h0_value_v64_file_is_sweep_64() {
    let raw = fs::read_to_string(crate_dir().join("models/h0-value-v64.json")).expect("v64 file");
    let trimmed = raw.trim_end_matches(&['\r', '\n'][..]);
    assert_eq!(trimmed.len(), 1_410_407);
    assert_eq!(fnv1a64(trimmed.as_bytes()), 0xee9f_c906_068c_bff4);
}

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn value_v64_matches_file_on_review_and_self_play() {
    let db = load_recorded_db();
    let v64 = h0_value_v64_path();
    let name_spec = format!("{SERVED_LEAF_RELEASE},net=h0-value-v64");
    let file_spec = format!("{SERVED_LEAF_RELEASE},net={v64}");
    let linear_spec = SERVED_LEAF_RELEASE;

    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let from_file = decide(&file_spec, 1, &db, &cap, m.ply, seed);
        let from_name = decide(&name_spec, 1, &db, &cap, m.ply, seed);
        assert_eq!(
            from_name.idx, from_file.idx,
            "{}/{}@{}",
            m.review, m.game, m.ply
        );
        assert_eq!(from_name.value_bits, from_file.value_bits, "value");
        assert_eq!(from_name.explain, from_file.explain, "explain");
    }

    let mut v64_differs_from_linear = false;
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let v64_out = decide(&name_spec, 1, &db, &cap, m.ply, seed);
        let linear_out = decide(linear_spec, 1, &db, &cap, m.ply, seed);
        if v64_out.explain != linear_out.explain {
            v64_differs_from_linear = true;
            break;
        }
    }
    assert!(
        v64_differs_from_linear,
        "h0-value-v64 explain should differ from default leaf on some review moment"
    );

    let cases = meta_selfplay_cases(&db);
    assert!(cases.len() >= 40, "self-play cases={}", cases.len());
    let mut searched = 0usize;
    for case in &cases {
        let legal_len = legal_actions(&db, &case.state).len();
        let from_name = decide_state(&name_spec, 1, &db, &case.state, case.seed);
        let from_file = decide_state(&file_spec, 1, &db, &case.state, case.seed);
        assert_eq!(from_name.idx, from_file.idx, "{}", case.label);
        assert_eq!(
            from_name.value_bits, from_file.value_bits,
            "value {}",
            case.label
        );
        assert_eq!(
            from_name.explain, from_file.explain,
            "explain {}",
            case.label
        );
        if from_name
            .explain
            .as_ref()
            .is_some_and(|e| explain_used_search(e, legal_len))
        {
            searched += 1;
        }
    }
    assert!(
        searched * 2 >= cases.len(),
        "expected most self-play decisions through search, searched={searched} of {}",
        cases.len()
    );
}

#[cfg_attr(
    debug_assertions,
    ignore = "release only: the ci job runs it in its release step"
)]
#[test]
fn order_v59_matches_file_on_review_and_self_play() {
    let db = load_recorded_db();
    let v59 = h0_order_v59_path();
    let v53 = h0_order_v53_path();
    let name_spec = format!("{SERVED_RELEASE},order=h0-order-v59,oorder=h0-order-v53");
    let file_spec = format!("{SERVED_RELEASE},order={v59},oorder={v53}");
    let legacy_spec = format!("{SERVED_RELEASE},order=builtin,oorder=1");

    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let from_file = decide(&file_spec, 1, &db, &cap, m.ply, seed);
        let from_name = decide(&name_spec, 1, &db, &cap, m.ply, seed);
        assert_eq!(
            from_name.idx, from_file.idx,
            "{}/{}@{}",
            m.review, m.game, m.ply
        );
        assert_eq!(from_name.value_bits, from_file.value_bits, "value");
        assert_eq!(from_name.explain, from_file.explain, "explain");
    }

    let oopen_name = format!("{name_spec},oopen=1,onext=40");
    let oopen_file = format!("{file_spec},oopen=1,onext=40");
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let from_file = decide(&oopen_file, 1, &db, &cap, m.ply, seed);
        let from_name = decide(&oopen_name, 1, &db, &cap, m.ply, seed);
        assert_eq!(
            from_name.idx, from_file.idx,
            "oopen {}/{}@{}",
            m.review, m.game, m.ply
        );
        assert_eq!(from_name.value_bits, from_file.value_bits, "value oopen");
        assert_eq!(from_name.explain, from_file.explain, "explain oopen");
    }

    let mut v59_differs_from_v53 = false;
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let v59_out = decide(&name_spec, 1, &db, &cap, m.ply, seed);
        let v53_out = decide(&legacy_spec, 1, &db, &cap, m.ply, seed);
        if v59_out.explain != v53_out.explain {
            v59_differs_from_v53 = true;
            break;
        }
    }
    assert!(
        v59_differs_from_v53,
        "h0-order-v59 explain should differ from order=builtin,oorder=1 on some review moment"
    );

    let cases = meta_selfplay_cases(&db);
    assert!(cases.len() >= 40, "self-play cases={}", cases.len());
    let mut searched = 0usize;
    for case in &cases {
        let legal_len = legal_actions(&db, &case.state).len();
        let from_name = decide_state(&name_spec, 1, &db, &case.state, case.seed);
        let from_file = decide_state(&file_spec, 1, &db, &case.state, case.seed);
        assert_eq!(from_name.idx, from_file.idx, "{}", case.label);
        assert_eq!(
            from_name.value_bits, from_file.value_bits,
            "value {}",
            case.label
        );
        assert_eq!(
            from_name.explain, from_file.explain,
            "explain {}",
            case.label
        );
        if from_name
            .explain
            .as_ref()
            .is_some_and(|e| explain_used_search(e, legal_len))
        {
            searched += 1;
        }
    }
    assert!(
        searched * 2 >= cases.len(),
        "expected most self-play decisions through search, searched={searched} of {}",
        cases.len()
    );
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
        !std::sync::Arc::ptr_eq(
            h_order.order.as_ref().expect("order"),
            h_base.net.as_ref().expect("net"),
        ),
        "order=v4 path loads a separate Arc from the built-in leaf"
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
