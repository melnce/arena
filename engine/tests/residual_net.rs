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
