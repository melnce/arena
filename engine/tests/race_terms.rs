//! Race inputs and optional linear-leaf race block.

use arena_engine::{
    encode::{race_features, BOARD_WIDTH, LAYOUT, RACE_LEN, RACE_NAMES},
    ValueNet,
};

mod common;

const EPS: f32 = 1e-6;
const VAL_EPS: f32 = 1e-3;

fn fixture() -> serde_json::Value {
    let text = std::fs::read_to_string(common::fixtures_dir().join("race/parity.json"))
        .expect("parity fixture");
    serde_json::from_str(&text).expect("parse parity.json")
}

#[test]
fn layout_offsets() {
    fn off(name: &str) -> usize {
        LAYOUT
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.offset)
            .expect(name)
    }
    assert_eq!(off("me_scalars"), 41);
    assert_eq!(off("opp_scalars"), 70);
    assert_eq!(off("own_board"), 153);
    assert_eq!(off("opp_board"), 253);
    assert_eq!(BOARD_WIDTH, 20);
}

#[test]
fn race_features_match_fixture() {
    let fix = fixture();
    for row in fix["rows"].as_array().expect("rows") {
        let feat: Vec<f32> = row["features"]
            .as_array()
            .expect("features")
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        let got = race_features(&feat);
        let want: Vec<f32> = row["race"]
            .as_array()
            .expect("race")
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        for j in 0..RACE_LEN {
            assert!(
                (got[j] - want[j]).abs() < EPS,
                "row race[{j}]: got {} want {}",
                got[j],
                want[j]
            );
        }
    }
}

#[test]
fn model_value_matches_fixture() {
    let fix = fixture();
    let model_text = fix["model"].to_string();
    let model = ValueNet::from_json(&model_text).expect("load model");
    for row in fix["rows"].as_array().expect("rows") {
        let feat: Vec<f32> = row["features"]
            .as_array()
            .expect("features")
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        let ids: Vec<u32> = row["ids"]
            .as_array()
            .expect("ids")
            .iter()
            .map(|v| v.as_u64().unwrap() as u32)
            .collect();
        let obs = arena_engine::Observation {
            features: feat,
            ids,
        };
        let got = model.value(&obs);
        let want = row["value"].as_f64().unwrap() as f32;
        assert!(
            (got - want).abs() < VAL_EPS,
            "value: got {} want {}",
            got,
            want
        );
    }
}

#[test]
fn zero_weight_race_block_is_bit_identical() {
    let fix = fixture();
    let mut doc = fix["model"].clone();
    let race = doc["race"].as_object_mut().expect("race");
    for w in race["w"].as_array_mut().expect("w") {
        *w = serde_json::json!(0.0);
    }
    let with_text = doc.to_string();
    let with_race = ValueNet::from_json(&with_text).expect("race model");
    doc.as_object_mut().expect("obj").remove("race");
    let without_text = doc.to_string();
    let without = ValueNet::from_json(&without_text).expect("plain model");
    for row in fix["rows"].as_array().expect("rows") {
        let feat: Vec<f32> = row["features"]
            .as_array()
            .expect("features")
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        let ids: Vec<u32> = row["ids"]
            .as_array()
            .expect("ids")
            .iter()
            .map(|v| v.as_u64().unwrap() as u32)
            .collect();
        let obs = arena_engine::Observation {
            features: feat,
            ids,
        };
        let a = with_race.value(&obs);
        let b = without.value(&obs);
        assert_eq!(a.to_bits(), b.to_bits(), "bit-identical value");
    }
}

#[test]
fn race_pre_hand_computed() {
    let mut feat = vec![0.0f32; 567];
    feat[41] = 3.0;
    feat[253] = 5.0;
    feat[253 + 19] = 1.0;
    let r = race_features(&feat);
    let w: Vec<f32> = (0..RACE_LEN).map(|i| (i as f32 + 1.0) * 0.1).collect();
    let race_pre = r.iter().zip(w.iter()).map(|(rv, wi)| wi * *rv).sum::<f32>();
    let zone_w: Vec<Vec<f32>> = (0..5).map(|_| vec![0.0]).collect();
    let json = serde_json::json!({
        "arch": "linear",
        "encoding": 2,
        "feature_len": 567,
        "feat_mean": vec![0.0; 567],
        "feat_std": vec![1.0; 567],
        "vocab": [0u32],
        "zones": [
            {"name": "own_hand", "id_offset": 0, "count": 9},
            {"name": "own_deck", "id_offset": 9, "count": 96, "hist_offset": 353},
            {"name": "opp_board", "id_offset": 105, "count": 5},
            {"name": "own_board", "id_offset": 110, "count": 5},
            {"name": "opp_pool", "id_offset": 115, "count": 96, "hist_offset": 449},
        ],
        "scale": 1.0,
        "linear": {"w": vec![0.0; 567], "zone_w": zone_w, "b": 0.0},
        "race": {
            "version": 1,
            "names": RACE_NAMES,
            "mean": vec![0.0; RACE_LEN],
            "std": vec![1.0; RACE_LEN],
            "w": w,
        },
    });
    let model = ValueNet::from_json(&json.to_string()).expect("model");
    let obs = arena_engine::Observation {
        features: feat,
        ids: vec![0u32; 220],
    };
    let v = model.value(&obs);
    assert!(
        (v - race_pre.tanh()).abs() < EPS,
        "got {} want {}",
        v,
        race_pre.tanh()
    );
}

#[test]
fn parse_errors() {
    let fix = fixture();
    let base = fix["model"].clone();

    let mlp = serde_json::json!({
        "arch": "mlp",
        "feature_len": 567,
        "encoding": 2,
        "feat_mean": vec![0.0; 567],
        "feat_std": vec![1.0; 567],
        "vocab": [0u32],
        "zones": base["zones"].clone(),
        "scale": 60.0,
        "mlp": {
            "emb": [[0.0]],
            "w1": [[0.0]],
            "b1": [0.0],
            "w2": [0.0],
            "b2": 0.0
        },
        "race": base["race"].clone()
    });
    let err = ValueNet::from_json(&mlp.to_string()).unwrap_err();
    assert!(err.contains("race"), "{err}");

    let mut bad_ver = base.clone();
    bad_ver["race"]["version"] = serde_json::json!(2);
    let err = ValueNet::from_json(&bad_ver.to_string()).unwrap_err();
    assert!(err.contains("race.version"), "{err}");

    let mut bad_names = base.clone();
    bad_names["race"]["names"] = serde_json::json!(["wrong"]);
    let err = ValueNet::from_json(&bad_names.to_string()).unwrap_err();
    assert!(err.contains("race.names"), "{err}");

    let mut bad_len = base.clone();
    bad_len["race"]["w"] = serde_json::json!([0.0]);
    let err = ValueNet::from_json(&bad_len.to_string()).unwrap_err();
    assert!(err.contains("want 12"), "{err}");

    let mut bad_std = base.clone();
    bad_std["race"]["std"] = serde_json::json!(vec![0.0; 12]);
    let err = ValueNet::from_json(&bad_std.to_string()).unwrap_err();
    assert!(err.contains("std"), "{err}");

    let mut nan = base.clone();
    nan["race"]["w"][0] = serde_json::json!(f64::NAN);
    let err = ValueNet::from_json(&nan.to_string()).unwrap_err();
    assert!(err.contains("w"), "{err}");
}

#[test]
fn race_names_order() {
    assert_eq!(RACE_NAMES.len(), RACE_LEN);
    assert_eq!(RACE_NAMES[0], "lo5_me");
    assert_eq!(RACE_NAMES[RACE_LEN - 1], "inter_opp");
}
