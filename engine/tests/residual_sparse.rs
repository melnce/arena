//! Residual net sparse first layer vs dense reference on review + random obs.

use std::collections::BTreeMap;
use std::fs;

use arena_engine::{
    apply_neutral, encode_with_vocab, new_game, vocab, AnyPolicy, EncodingVersion, First,
    GameConfig, Observation, PlayerId, ValueNet,
};

mod common;
use common::*;

fn tiny_path() -> std::path::PathBuf {
    repo_root().join("py/tests/fixtures/order_residual/tiny-residual.json")
}

fn review_path(review: &str, game: &str) -> std::path::PathBuf {
    repo_root()
        .join("py/tests/fixtures")
        .join(review)
        .join("games")
        .join(game)
}

fn deck_from_json(v: &serde_json::Value) -> Vec<arena_engine::CardId> {
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

fn replay_state(
    db: &arena_engine::CardDb,
    cap: &serde_json::Value,
    ply: usize,
) -> Option<arena_engine::State> {
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
    for step in actions.iter().take(ply) {
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
        if apply_neutral(db, &mut st, &neu).is_err() {
            return None;
        }
    }
    Some(st)
}

fn collect_review_obs(want: usize) -> Vec<Observation> {
    let db = load_recorded_db();
    let moments = [
        ("review13", "4984932781931433298-b4973563.json", 27usize),
        ("review13", "4984932781931433298-b4973563.json", 45),
        ("review14", "14600367900189587136-d90f8eb2.json", 69),
        ("review14", "14600367900189587136-d90f8eb2.json", 40),
        ("review14", "6598261483642061665-d90f8eb2.json", 27),
        ("review14", "6598261483642061665-d90f8eb2.json", 52),
        ("review14", "6598261483642061665-d90f8eb2.json", 91),
        ("review14", "12644260486301391828-d90f8eb2.json", 15),
        ("review14", "12644260486301391828-d90f8eb2.json", 35),
        ("review14", "6598261483642061665-d90f8eb2.json", 60),
        ("review14", "14600367900189587136-d90f8eb2.json", 55),
        ("review13", "4984932781931433298-b4973563.json", 60),
    ];
    let mut out = Vec::new();
    for (rev, name, ply) in moments {
        let raw = fs::read_to_string(review_path(rev, name)).expect("review");
        let cap: serde_json::Value = serde_json::from_str(&raw).expect("json");
        let Some(st) = replay_state(&db, &cap, ply) else {
            continue;
        };
        for p in [PlayerId::A, PlayerId::B] {
            let root_vocab = vocab(&st);
            out.push(encode_with_vocab(
                &st,
                p,
                &root_vocab,
                EncodingVersion::V3,
                Some(&db),
            ));
            if out.len() >= want {
                return out;
            }
        }
    }
    for games_dir in ["review13", "review14"] {
        let dir = repo_root()
            .join("py/tests/fixtures")
            .join(games_dir)
            .join("games");
        for entry in fs::read_dir(dir).expect("games dir").flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let cap: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(path).expect("read")).expect("json");
            let max_ply = cap["actions"].as_array().map(|a| a.len()).unwrap_or(0);
            for ply in (4..max_ply).step_by(3) {
                let Some(st) = replay_state(&db, &cap, ply) else {
                    continue;
                };
                let root_vocab = vocab(&st);
                out.push(encode_with_vocab(
                    &st,
                    PlayerId::A,
                    &root_vocab,
                    EncodingVersion::V3,
                    Some(&db),
                ));
                if out.len() >= want {
                    return out;
                }
            }
        }
    }
    out
}

fn random_obs(seed: u64) -> Observation {
    const FEAT: usize = 961;
    let mut st = seed;
    let mut features = vec![0.0f32; FEAT];
    for _ in 0..14 {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1);
        let i = (st as usize) % FEAT;
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1);
        features[i] = ((st >> 32) as i32 as f32) * 0.01;
    }
    let mut ids = vec![0u32; 230];
    for id in &mut ids {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1);
        *id = (st >> 32) as u32 % 10_000_000 + 10_000_000;
    }
    Observation { features, ids }
}

#[test]
fn residual_sparse_matches_dense_reference() {
    let net = ValueNet::load(tiny_path()).expect("tiny");
    let mut obs = collect_review_obs(50);
    assert!(obs.len() >= 50, "review observations={}", obs.len());
    for s in 0..20 {
        obs.push(random_obs(77_000 + s));
    }

    for clip in [0.0f32, 5.0f32, 2.5f32] {
        for ob in &obs {
            let fast = if clip == 0.0 {
                net.value(ob)
            } else {
                net.value_clipped(ob, clip)
            };
            let dense = net.value_residual_dense(ob, clip);
            assert!(
                (fast - dense).abs() <= 1e-4,
                "clip={clip} fast={fast} dense={dense}"
            );
        }
    }
}

#[test]
fn residual_net_spec_round_trip() {
    let path_s = tiny_path().display().to_string();
    let spec = format!("h0:net={path_s}");
    let parsed = AnyPolicy::parse_spec(&spec).unwrap();
    assert_eq!(parsed.spec(), format!("h0:value=net,net={path_s}"));
}
