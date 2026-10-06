//! Encoding version 3: crests, amulet countdowns, enter counts, cemetery.

use arena_engine::determinize::{determinize_with, Info};
use arena_engine::{
    apply, encode_version, encode_with_vocab, legal_actions, new_game, policy_rng, vocab, CardId,
    EncodingVersion, First, GameConfig, Observation, Phase, PlayerId, Policy, ValueNet,
};

mod common;
use common::*;

const V2_PREFIX: usize = arena_engine::encode::LEN_V2;
const V3_EXTRA_OFF: usize = arena_engine::encode::LEN_V2;
const V3_EXTRA_LEN: usize = arena_engine::encode::LEN_V3 - arena_engine::encode::LEN_V2;
const OPEN_WORLDS: u64 = 8;

fn offset_v3(name: &str) -> usize {
    Observation::LAYOUT_V3_EXTRA
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.offset)
        .unwrap_or_else(|| panic!("missing layout field {name}"))
}

fn meta_deck_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: serde_json::Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .collect()
}

fn f32_list(v: &serde_json::Value) -> Vec<f32> {
    v.as_array()
        .expect("array")
        .iter()
        .map(|x| x.as_f64().expect("f") as f32)
        .collect()
}

fn write_h0_v3_zero_stack_file(path: &std::path::Path) -> std::sync::Arc<ValueNet> {
    let base: serde_json::Value =
        serde_json::from_str(include_str!("../models/h0-linear-v3.json")).expect("base json");
    let vocab = base["vocab"].clone();
    let vocab_len = vocab.as_array().expect("vocab").len();
    let mut w = f32_list(&base["linear"]["w"]);
    w.resize(arena_engine::encode::LEN_V3, 0.0);
    let mut mean = f32_list(&base["feat_mean"]);
    mean.resize(arena_engine::encode::LEN_V3, 0.0);
    let mut std = f32_list(&base["feat_std"]);
    std.resize(arena_engine::encode::LEN_V3, 1.0);
    let base_zw: Vec<Vec<f32>> = base["linear"]["zone_w"]
        .as_array()
        .expect("zone_w")
        .iter()
        .map(|row| f32_list(row))
        .collect();
    let mut zone_w = base_zw;
    for _ in 5..13 {
        zone_w.push(vec![0.0; vocab_len]);
    }
    let json = serde_json::json!({
        "arch": "linear",
        "encoding": 3,
        "feature_len": arena_engine::encode::LEN_V3,
        "feat_mean": mean,
        "feat_std": std,
        "vocab": vocab,
        "zones": [
            {"name": "own_hand", "id_offset": 0, "count": 9},
            {"name": "own_deck", "id_offset": 9, "count": 96, "hist_offset": 353},
            {"name": "opp_board", "id_offset": 105, "count": 5},
            {"name": "own_board", "id_offset": 110, "count": 5},
            {"name": "opp_pool", "id_offset": 115, "count": 96, "hist_offset": 449},
            {"name": "own_crests", "id_offset": 220, "count": 5},
            {"name": "opp_crests", "id_offset": 225, "count": 5},
            {"name": "own_amulet_soon", "id_offset": 110, "count": 5, "hist_offset": 567},
            {"name": "opp_amulet_soon", "id_offset": 105, "count": 5, "hist_offset": 572},
            {"name": "own_entered", "id_offset": 9, "count": 96, "hist_offset": 577},
            {"name": "opp_entered", "id_offset": 115, "count": 96, "hist_offset": 673},
            {"name": "own_cemetery", "id_offset": 9, "count": 96, "hist_offset": 769},
            {"name": "opp_cemetery", "id_offset": 115, "count": 96, "hist_offset": 865},
        ],
        "scale": base["scale"],
        "linear": {"w": w, "zone_w": zone_w, "b": base["linear"]["b"]},
    });
    let text = serde_json::to_string_pretty(&json).expect("ser");
    std::fs::write(path, &text).expect("write");
    ValueNet::from_json(&text).expect("v3 zero-stack model")
}

#[test]
fn v3_prefix_matches_v2_on_meta_states() {
    let db = load_db();
    let decks: Vec<Vec<CardId>> = meta_deck_stems()
        .iter()
        .map(|stem| load_deck_file(format!("oracle/decks/{stem}.json")))
        .filter(|d| deck_ready(&db, d) && d.len() == 40)
        .collect();
    assert!(!decks.is_empty());
    let mut checked = 0usize;
    let mut seed = 1u64;
    while checked < 200 && seed < 2_000 {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            &db,
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
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) && checked < 200 {
            for me in [PlayerId::A, PlayerId::B] {
                let v2 = encode_version(&state, me, EncodingVersion::V2, Some(&db));
                let v3 = encode_version(&state, me, EncodingVersion::V3, Some(&db));
                assert_eq!(
                    &v2.features[..V2_PREFIX],
                    &v3.features[..V2_PREFIX],
                    "v2 prefix feat seed {seed}"
                );
                assert_eq!(
                    &v2.ids[..Observation::IDS_LEN],
                    &v3.ids[..Observation::IDS_LEN],
                    "v2 prefix ids seed {seed}"
                );
                checked += 1;
                if checked >= 200 {
                    break;
                }
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() || state.turn > 30 {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(&db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
        }
        seed += 1;
    }
    assert!(checked >= 200, "expected >=200 states, got {checked}");
}

#[test]
fn v3_block_values_on_constructed_states() {
    let db = load_db();
    let mut st = started_decks(&db, 9, &["90031210"], &["90031210"]);
    let me = PlayerId::A;
    let opp = me.opponent();
    st.player_mut(me).crests.push(arena_engine::state::CrestInstance {
        id: "crest:10714110".into(),
        countdown: Some(2),
        faith: false,
        once_used: vec![],
        granted_order: 1,
        granted: vec![],
        choose_used: Default::default(),
    });
    st.player_mut(opp).crests.push(arena_engine::state::CrestInstance {
        id: "faith:10554110".into(),
        countdown: None,
        faith: true,
        once_used: vec![],
        granted_order: 1,
        granted: vec![],
        choose_used: Default::default(),
    });
    let amulet_id = cid("90031210");
    let mut own_amulet =
        arena_engine::state::CardInstance::from_card(db.card(amulet_id).expect("card"), st.alloc_id());
    own_amulet.countdown = Some(3);
    let mut opp_amulet =
        arena_engine::state::CardInstance::from_card(db.card(amulet_id).expect("card"), st.alloc_id());
    opp_amulet.countdown = Some(1);
    st.player_mut(me).field[0] = Some(own_amulet);
    st.player_mut(opp).field[0] = Some(opp_amulet);
    let card = amulet_id;
    let v = vocab(&st);
    let idx = v.binary_search(&card).expect("vocab");
    st.player_mut(me).enter_counts.insert(card, 2);
    st.player_mut(opp).enter_counts.insert(card, 1);
    let pub_inst = st.player(me).field[0].as_ref().unwrap().clone();
    st.player_mut(me).cemetery.push(pub_inst);
    let hidden_card = db.card(card).expect("card");
    let hidden = arena_engine::state::CardInstance::from_card(hidden_card, st.alloc_id());
    let pub_opp = st.player(opp).field[0].as_ref().unwrap().clone();
    st.player_mut(opp).cemetery.push(hidden.clone());
    st.player_mut(opp).cemetery.push(pub_opp);
    st.player_mut(opp).hidden_removals.push(hidden.id);
    st.player_mut(opp).hidden_removal_cards.push(hidden.card);

    let obs = encode_version(&st, me, EncodingVersion::V3, Some(&db));
    assert_eq!(obs.ids[220], 10714110);
    assert_eq!(obs.ids[225], 10554110);
    assert_eq!(obs.features[offset_v3("own_amulet_soon")], 1.0 / 3.0);
    assert_eq!(obs.features[offset_v3("opp_amulet_soon")], 1.0);
    assert_eq!(obs.features[offset_v3("own_entered_hist") + idx], 2.0);
    assert_eq!(obs.features[offset_v3("opp_entered_hist") + idx], 1.0);
    assert_eq!(obs.features[offset_v3("own_cemetery_hist") + idx], 1.0);
    assert_eq!(obs.features[offset_v3("opp_cemetery_hist") + idx], 1.0);
}

#[test]
fn v3_open_worlds_match_real_state() {
    let db = load_db();
    let decks: Vec<Vec<CardId>> = meta_deck_stems()
        .iter()
        .map(|stem| load_deck_file(format!("oracle/decks/{stem}.json")))
        .filter(|d| deck_ready(&db, d) && d.len() == 40)
        .collect();
    let mut states_with_hidden = 0usize;
    let mut worlds_checked = 0usize;
    let mut mismatches = 0usize;
    let mut seed = 1u64;
    while states_with_hidden < 100 && seed < 2_000 {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            &db,
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
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            for me in [PlayerId::A, PlayerId::B] {
                let opp = me.opponent();
                if state.player(opp).hidden_removals.is_empty()
                    && state.player(opp).cemetery.is_empty()
                {
                    continue;
                }
                states_with_hidden += 1;
                let root_vocab = vocab(&state);
                let real = encode_with_vocab(
                    &state,
                    me,
                    &root_vocab,
                    EncodingVersion::V3,
                    Some(&db),
                );
                let ref_extra = &real.features[V3_EXTRA_OFF..V3_EXTRA_OFF + V3_EXTRA_LEN];
                let ref_crests = &real.ids[220..230];
                for w in 0..OPEN_WORLDS {
                    worlds_checked += 1;
                    let world = determinize_with(&state, me, seed * 1_000 + w, Info::Open);
                    let enc = encode_with_vocab(
                        &world,
                        me,
                        &root_vocab,
                        EncodingVersion::V3,
                        Some(&db),
                    );
                    let extra = &enc.features[V3_EXTRA_OFF..V3_EXTRA_OFF + V3_EXTRA_LEN];
                    let crests = &enc.ids[220..230];
                    if extra != ref_extra || crests != ref_crests {
                        mismatches += 1;
                    }
                }
            }
            if state.turn > 30 {
                break;
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(&db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
        }
        seed += 1;
    }
    assert!(states_with_hidden >= 20, "got {states_with_hidden}");
    assert_eq!(mismatches, 0, "world mismatches={mismatches}");
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn v3_zero_stack_matches_h0_linear_v3() {
    let db = load_db();
    let base = ValueNet::from_json(include_str!("../models/h0-linear-v3.json")).expect("v2");
    let path = h0_linear_v3_path();
    let tmp = std::env::temp_dir().join("arena-v3-zero-stack.json");
    let stacked = write_h0_v3_zero_stack_file(&tmp);
    let stacked_spec = format!("h0:net={}", tmp.display());
    let file_h = parse_h0_v2(&format!("h0:net={path}"));
    let stacked_h = parse_h0_v2(&stacked_spec);

    let decks: Vec<Vec<CardId>> = meta_deck_stems()
        .iter()
        .map(|stem| load_deck_file(format!("oracle/decks/{stem}.json")))
        .filter(|d| deck_ready(&db, d) && d.len() == 40)
        .collect();
    let mut checked = 0usize;
    let mut seed = 1u64;
    while checked < 200 && seed < 2_000 {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            &db,
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
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) && checked < 200 {
            for me in [PlayerId::A, PlayerId::B] {
                let obs = encode_version(&state, me, EncodingVersion::V3, Some(&db));
                let v_base = base.value(&obs);
                let v_stack = stacked.value(&obs);
                let vc_base = base.value_clipped(&obs, 5.0);
                let vc_stack = stacked.value_clipped(&obs, 5.0);
                assert_eq!(v_base.to_bits(), v_stack.to_bits());
                assert_eq!(vc_base.to_bits(), vc_stack.to_bits());
                let ev_base = file_h.evaluate(&db, &state, me);
                let ev_stack = stacked_h.evaluate(&db, &state, me);
                assert_eq!(ev_base.to_bits(), ev_stack.to_bits());
                checked += 1;
                if checked >= 200 {
                    break;
                }
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() || state.turn > 30 {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(&db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
        }
        seed += 1;
    }
    assert!(checked >= 200);

    for (g, seed) in [(0, 11u64), (1, 22), (2, 33), (3, 44)] {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 2 + 1) % decks.len()];
        let mut state = new_game(
            &db,
            GameConfig {
                seed,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("game");
        let mut a = match arena_engine::AnyPolicy::parse_spec("h0").expect("h0") {
            arena_engine::AnyPolicy::H0(h) => h,
            other => panic!("{other:?}"),
        };
        let mut b = parse_h0_v2(&stacked_spec);
        let mut actions = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) && actions < 500 {
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let actor = arena_engine::acting_player(&state);
            let va = a.evaluate(&db, &state, actor);
            let vb = b.evaluate(&db, &state, actor);
            assert_eq!(
                va.to_bits(),
                vb.to_bits(),
                "evaluate game {g} action {actions}"
            );
            let mut ra = policy_rng(seed.wrapping_add(actions as u64));
            let mut rb = policy_rng(seed.wrapping_add(actions as u64));
            let ia = a.choose(&db, &state, &legal, &mut ra);
            let ib = b.choose(&db, &state, &legal, &mut rb);
            assert_eq!(ia, ib, "game {g} action {actions}");
            apply(&db, &mut state, legal[ia as usize].clone()).expect("apply");
            actions += 1;
        }
    }
}

#[test]
fn v3_parser_rejects_bad_zone_count() {
    let zone_w: Vec<Vec<f32>> = (0..12).map(|_| vec![0.0]).collect();
    let json = serde_json::json!({
        "arch": "linear", "encoding": 3, "feature_len": 961,
        "feat_mean": vec![0.0; 961], "feat_std": vec![1.0; 961],
        "vocab": [0u32], "zones": [],
        "scale": 1.0, "linear": {"w": vec![0.0; 961], "zone_w": zone_w, "b": 0.0},
    });
    let err = ValueNet::from_json(&json.to_string()).unwrap_err();
    assert!(err.contains("zones"), "{err}");
}

#[test]
fn v3_parser_rejects_nonzero_index0_new_rows() {
    let mut zone_w: Vec<Vec<f32>> = (0..13).map(|_| vec![0.0, 1.0]).collect();
    zone_w[5][0] = 0.1;
    let json = serde_json::json!({
        "arch": "linear", "encoding": 3, "feature_len": 961,
        "feat_mean": vec![0.0; 961], "feat_std": vec![1.0; 961],
        "vocab": [0u32, 1u32],
        "zones": [
            {"name": "own_hand", "id_offset": 0, "count": 9},
            {"name": "own_deck", "id_offset": 9, "count": 96, "hist_offset": 353},
            {"name": "opp_board", "id_offset": 105, "count": 5},
            {"name": "own_board", "id_offset": 110, "count": 5},
            {"name": "opp_pool", "id_offset": 115, "count": 96, "hist_offset": 449},
            {"name": "own_crests", "id_offset": 220, "count": 5},
            {"name": "opp_crests", "id_offset": 225, "count": 5},
            {"name": "own_amulet_soon", "id_offset": 110, "count": 5, "hist_offset": 567},
            {"name": "opp_amulet_soon", "id_offset": 105, "count": 5, "hist_offset": 572},
            {"name": "own_entered", "id_offset": 9, "count": 96, "hist_offset": 577},
            {"name": "opp_entered", "id_offset": 115, "count": 96, "hist_offset": 673},
            {"name": "own_cemetery", "id_offset": 9, "count": 96, "hist_offset": 769},
            {"name": "opp_cemetery", "id_offset": 115, "count": 96, "hist_offset": 865},
        ],
        "scale": 1.0,
        "linear": {"w": vec![0.0; 961], "zone_w": zone_w, "b": 0.0},
    });
    let err = ValueNet::from_json(&json.to_string()).unwrap_err();
    assert!(err.contains("index 0"), "{err}");
}

#[test]
fn v3_parser_rejects_mlp() {
    let json = serde_json::json!({
        "arch": "mlp", "encoding": 3, "feature_len": 961,
        "feat_mean": vec![0.0; 961], "feat_std": vec![1.0; 961],
        "vocab": [0u32],
        "zones": [{"name": "own_hand", "id_offset": 0, "count": 9}],
        "scale": 1.0,
        "mlp": {"emb": [[0.0]], "w1": [[0.0]], "b1": [0.0], "w2": [0.0], "b2": 0.0},
    });
    let err = ValueNet::from_json(&json.to_string()).unwrap_err();
    assert!(err.contains("linear"), "{err}");
}

#[test]
fn v3_model_from_env_loads() {
    let Ok(path) = std::env::var("ARENA_MODEL_PATH") else {
        return;
    };
    let text = std::fs::read_to_string(&path).expect("read model");
    let _ = ValueNet::from_json(&text).expect("load trainer model");
}

#[test]
fn v3_trainer_stack_unknown_vocab_matches_base() {
    use std::process::Command;

    let db = load_db();
    let base = ValueNet::from_json(include_str!("../models/h0-linear-v3.json")).expect("base");
    let tmp = std::env::temp_dir();
    let data = tmp.join("arena-v3-trainer-unk-data");
    let out = tmp.join("arena-v3-trainer-unk-stack.json");
    let _ = std::fs::remove_dir_all(&data);
    let _ = std::fs::remove_file(&out);
    let script = repo_root().join("engine/tests/fixtures/stack_unknown_vocab_train.py");
    let status = Command::new("python3")
        .arg(&script)
        .arg(&data)
        .arg(&out)
        .status()
        .expect("python trainer");
    assert!(status.success(), "trainer subprocess failed");

    let stacked_text = std::fs::read_to_string(&out).expect("stack json");
    let stacked = ValueNet::from_json(&stacked_text).expect("stacked model");
    let mut st = started_decks(&db, 42, &["88001140"], &["90031210"]);
    let me = PlayerId::A;
    let card = cid("88001140");
    let inst = arena_engine::state::CardInstance::from_card(
        db.card(card).expect("card"),
        st.alloc_id(),
    );
    st.player_mut(me).hand.push(inst);
    let obs = encode_version(&st, me, EncodingVersion::V3, Some(&db));
    let v_base = base.value(&obs);
    let v_stack = stacked.value(&obs);
    assert_eq!(v_base.to_bits(), v_stack.to_bits(), "unknown vocab card value");
}

#[test]
fn v1_v2_v3_leaf_and_race_models_still_load() {
    let _ = ValueNet::from_json(include_str!("../models/h0-linear-v1.json")).expect("v1");
    let _ = ValueNet::from_json(include_str!("../models/h0-linear-v2.json")).expect("v2");
    let _ = ValueNet::from_json(include_str!("../models/h0-linear-v3.json")).expect("v3 leaf");
    let race = std::fs::read_to_string(fixtures_dir().join("race/parity.json")).expect("race");
    let v: serde_json::Value = serde_json::from_str(&race).expect("json");
    let _ = ValueNet::from_json(&v["model"].to_string()).expect("race model");
}

#[test]
#[ignore]
fn v3_evaluate_cost_us() {
    use std::time::Instant;

    let db = load_db();
    let decks: Vec<Vec<CardId>> = meta_deck_stems()
        .iter()
        .map(|stem| load_deck_file(format!("oracle/decks/{stem}.json")))
        .filter(|d| deck_ready(&db, d) && d.len() == 40)
        .collect();
    let mut states = Vec::new();
    let mut seed = 1u64;
    while states.len() < 200 {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            &db,
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
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            if state.turn >= 2 {
                states.push(state.clone());
                if states.len() >= 200 {
                    break;
                }
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(&db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
        }
        seed += 1;
    }
    let v2 = ValueNet::from_json(include_str!("../models/h0-linear-v3.json")).expect("v2");
    let tmp = std::env::temp_dir().join("arena-v3-zero-stack.json");
    let v3 = write_h0_v3_zero_stack_file(&tmp);
    let benches: [(&str, EncodingVersion, &ValueNet); 2] = [
        ("h0-linear-v3 (v2)", EncodingVersion::V2, &v2),
        ("v3 zero-stack", EncodingVersion::V3, v3.as_ref()),
    ];
    for (name, enc, net) in benches {
        for s in states.iter().take(4) {
            for me in [PlayerId::A, PlayerId::B] {
                let obs = encode_version(s, me, enc, Some(&db));
                let _ = net.value(&obs);
            }
        }
        let t0 = Instant::now();
        let mut n = 0u32;
        for s in &states {
            for me in [PlayerId::A, PlayerId::B] {
                let obs = encode_version(s, me, enc, Some(&db));
                let _ = net.value(&obs);
                n += 1;
            }
        }
        let us = t0.elapsed().as_secs_f64() * 1.0e6 / f64::from(n);
        eprintln!("{name}: {us:.2} µs/call ({n} calls)");
    }
}

#[test]
#[ignore]
fn v3_h0_decision_ms() {
    use std::time::Instant;

    use arena_engine::{AnyPolicy, H0};

    const SERVED: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000";

    fn parse_h0(spec: &str) -> H0 {
        match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
            AnyPolicy::H0(h) => h,
            other => panic!("{spec} parsed as {other:?}"),
        }
    }

    let db = load_db();
    let decks: Vec<Vec<CardId>> = meta_deck_stems()
        .iter()
        .map(|stem| load_deck_file(format!("oracle/decks/{stem}.json")))
        .filter(|d| deck_ready(&db, d) && d.len() == 40)
        .collect();
    let mut states = Vec::new();
    let mut seed = 100u64;
    while states.len() < 50 {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            &db,
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
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            if state.turn >= 4 && matches!(state.phase, Phase::Main) {
                states.push(state.clone());
                if states.len() >= 50 {
                    break;
                }
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(&db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
        }
        seed += 1;
    }
    let tmp = std::env::temp_dir().join("arena-v3-zero-stack.json");
    write_h0_v3_zero_stack_file(&tmp);
    let stack_spec = format!("h0:net={}", tmp.display());
    let specs = [
        ("h0-v2 builtin", SERVED),
        (
            "h0-v3 zero-stack",
            &format!(
                "{stack_spec},nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000"
            ),
        ),
    ];
    for (name, spec) in specs {
        let mut h0 = parse_h0(spec);
        let mut times = Vec::new();
        for (i, state) in states.iter().enumerate() {
            let legal = legal_actions(&db, state);
            if legal.len() <= 1 {
                continue;
            }
            let mut rng = policy_rng(10_000 + i as u64);
            let t0 = Instant::now();
            let _ = h0.choose(&db, state, &legal, &mut rng);
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        let n = times.len();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = times[n / 2];
        eprintln!("{name}: median {med:.1} ms/decision (n={n})");
    }
}
