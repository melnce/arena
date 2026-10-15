//! Residual value net: parser errors and spec round-trip.

use std::fs;
use std::path::PathBuf;

use arena_engine::{AnyPolicy, ValueNet};

mod common;
use common::*;

fn tiny_residual_path() -> PathBuf {
    repo_root().join("py/tests/fixtures/order_residual/tiny-residual.json")
}

#[test]
fn residual_parser_round_trip_and_rejects() {
    let path = tiny_residual_path();
    let p = format!("h0:order={}", path.display());
    let parsed = AnyPolicy::parse_spec(&p).expect("parse");
    assert_eq!(parsed.spec(), p);

    let text = fs::read_to_string(&path).expect("tiny");
    let j: serde_json::Value = serde_json::from_str(&text).expect("json");

    let mut bad = j.clone();
    bad["encoding"] = serde_json::json!(1);
    assert!(ValueNet::from_json(&serde_json::to_string(&bad).unwrap()).is_err());

    bad = j.clone();
    bad["race"] = serde_json::json!({"version": 1});
    assert!(ValueNet::from_json(&serde_json::to_string(&bad).unwrap()).is_err());

    bad = j.clone();
    bad["mlp"]["w1"][0].as_array_mut().unwrap().pop();
    assert!(ValueNet::from_json(&serde_json::to_string(&bad).unwrap()).is_err());

    bad = j.clone();
    bad["mlp"]["emb"].as_array_mut().unwrap().pop();
    assert!(ValueNet::from_json(&serde_json::to_string(&bad).unwrap()).is_err());

    bad = j.clone();
    bad["mlp"]["w1"] = serde_json::json!([]);
    assert!(ValueNet::from_json(&serde_json::to_string(&bad).unwrap()).is_err());

    bad = j.clone();
    bad["mlp"]["w2"][0] = serde_json::json!("NaN");
    assert!(ValueNet::from_json(&serde_json::to_string(&bad).unwrap()).is_err());

    bad = j.clone();
    let feat_len = bad["feature_len"].as_u64().unwrap() as usize;
    let n_zones = bad["mlp"]["emb"].as_array().unwrap().len();
    let emb_dim_new = 11usize;
    assert!(n_zones * emb_dim_new > 128);
    for zone in bad["mlp"]["emb"].as_array_mut().unwrap() {
        for row in zone.as_array_mut().unwrap() {
            let old = row.as_array().unwrap();
            let mut new = vec![0.0f32; emb_dim_new];
            for (i, v) in old.iter().enumerate() {
                new[i] = v.as_f64().unwrap_or(0.0) as f32;
            }
            *row = serde_json::json!(new);
        }
    }
    let in_dim = feat_len + n_zones * emb_dim_new;
    for row in bad["mlp"]["w1"].as_array_mut().unwrap() {
        let old = row.as_array().unwrap();
        let mut new = vec![0.0f32; in_dim];
        for (i, v) in old.iter().enumerate() {
            new[i] = v.as_f64().unwrap_or(0.0) as f32;
        }
        *row = serde_json::json!(new);
    }
    let err = ValueNet::from_json(&serde_json::to_string(&bad).unwrap()).unwrap_err();
    assert!(
        err.contains("emb") && err.contains("128"),
        "unexpected err: {err}"
    );

    let path_s = path.display().to_string();
    let net_spec = format!("h0:net={path_s}");
    let net_canon = format!("h0:value=net,net={path_s}");
    assert_eq!(AnyPolicy::parse_spec(&net_spec).unwrap().spec(), net_canon);
    let order_spec = format!("h0:order={path_s}");
    assert!(AnyPolicy::parse_spec(&order_spec)
        .unwrap()
        .spec()
        .contains(&path_s));
}
