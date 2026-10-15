//! PR micro-benchmark (ignored in CI). Run with:
//! `cargo test --release --test bench_residual_order -- --ignored --nocapture`

use std::fs;
use std::time::Instant;

use arena_engine::{encode_with_vocab, vocab, AnyPolicy, EncodingVersion, PlayerId, Policy, H0};

mod common;
use common::*;

#[test]
#[ignore]
fn bench_residual_order_vs_builtin() {
    let tiny = repo_root().join("py/tests/fixtures/order_residual/bench-residual.json");
    if !tiny.exists() {
        eprintln!("skip: generate bench-residual.json first");
        return;
    }
    let s = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000";
    let builtin = format!("{s},order=builtin,oorder=1");
    let residual = format!("{s},order={},oorder=1", tiny.display());
    let db = load_recorded_db();
    let cap: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            repo_root().join("py/tests/fixtures/review13/games/4984932781931433298-b4973563.json"),
        )
        .expect("review"),
    )
    .expect("json");
    let st = replay_capture(&db, &cap, 27);
    let legal = arena_engine::legal_actions(&db, &st);
    let seed = cap["seed"].as_u64().expect("seed") + 27;

    let mut b_times = Vec::new();
    let mut r_times = Vec::new();
    for _ in 0..20 {
        let t0 = Instant::now();
        let mut h = parse_h0(&builtin);
        let mut rng = arena_engine::policy_rng(seed);
        h.choose(&db, &st, &legal, &mut rng);
        b_times.push(t0.elapsed().as_secs_f64());

        let t1 = Instant::now();
        let mut h = parse_h0(&residual);
        let mut rng = arena_engine::policy_rng(seed);
        h.choose(&db, &st, &legal, &mut rng);
        r_times.push(t1.elapsed().as_secs_f64());
    }
    let med_b = median(&b_times);
    let med_r = median(&r_times);
    let ratio = r_times.iter().sum::<f64>() / b_times.iter().sum::<f64>();

    let net = arena_engine::ValueNet::load(&tiny).expect("net");
    let root_vocab = vocab(&st);
    let obs = encode_with_vocab(
        &st,
        PlayerId::A,
        &root_vocab,
        EncodingVersion::V3,
        Some(&db),
    );
    let n = 12_000u32;
    let t0 = Instant::now();
    for _ in 0..n {
        net.value_clipped(&obs, 5.0);
    }
    let micro_r = t0.elapsed().as_secs_f64() / n as f64;
    let builtin_net = arena_engine::builtin_order();
    let t1 = Instant::now();
    for _ in 0..n {
        builtin_net.value_clipped(&obs, 5.0);
    }
    let micro_b = t1.elapsed().as_secs_f64() / n as f64;

    println!(
        "median_builtin_s={med_b:.4} median_residual_s={med_r:.4} total_ratio={ratio:.4} value_clipped_builtin_s={micro_b:.7} value_clipped_residual_s={micro_r:.7}"
    );
}

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap() {
        AnyPolicy::H0(h) => h,
        other => panic!("{other:?}"),
    }
}

fn replay_capture(
    db: &arena_engine::CardDb,
    cap: &serde_json::Value,
    n: usize,
) -> arena_engine::State {
    use arena_engine::{apply_neutral, new_game, First, GameConfig};
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
    .expect("game");
    let actions = cap["actions"].as_array().expect("actions");
    for step in actions.iter().take(n) {
        if step.get("reseed").is_some() {
            st.reseed(step["reseed"].as_u64().expect("reseed"));
            continue;
        }
        let mut body = serde_json::Map::new();
        for (k, v) in step.as_object().expect("step") {
            if k == "value" || k == "bot_value" {
                continue;
            }
            body.insert(k.clone(), v.clone());
        }
        let neu: arena_engine::NeutralAction =
            serde_json::from_value(serde_json::Value::Object(body)).expect("action");
        apply_neutral(db, &mut st, &neu).expect("apply");
    }
    st
}

fn deck_from_json(v: &serde_json::Value) -> Vec<arena_engine::CardId> {
    use std::collections::BTreeMap;
    let map = v.as_object().expect("deck");
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

fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[s.len() / 2]
}
