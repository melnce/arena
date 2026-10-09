//! H0 holdback resample (`hbk`): default play unchanged, review13/14 moments.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use arena_engine::policy::ChoosePath;
use arena_engine::{
    apply_neutral, legal_actions, new_game, policy_rng, to_neutral, Action, AnyPolicy, CardDb,
    First, GameConfig, Policy, H0,
};
use serde_json::Value;

mod common;
use common::*;

const SERVED_SPEC: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1";

const GATE_SEEDS: [u64; 8] = [11, 23, 37, 41, 53, 67, 79, 97];

const DEFAULT_OKILL_GATE_FPS: [u64; 8] = [
    0x2246_9fdb_d814_7321,
    0x8867_a7c5_74fc_8fa9,
    0x649a_13b2_5e6f_3115,
    0xf257_1914_6cd5_5f83,
    0x3917_513b_bba7_4346,
    0x2cc7_6a3f_9f68_61e1,
    0x7dfb_9a9f_fc91_c0b1,
    0x8a65_79b5_792c_ac4d,
];

const SERVED_GATE_SEEDS: [u64; 3] = [11, 23, 37];
const SERVED_GATE_FPS: [u64; 3] = [
    0xe245_2508_3e83_6195,
    0xcf64_964d_6a48_10c2,
    0x240a_8a8f_d3c7_5241,
];

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

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

fn action_fingerprint(db: &CardDb, spec: &str, seed: u64, deck_a: &[arena_engine::CardId]) -> u64 {
    use arena_engine::trace::fnv1a64;
    use arena_engine::{apply, acting_player, Phase, PlayerId};

    let deck_b = load_meta_deck("meta-sword-rally");
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut a = parse_h0(spec);
    let mut b = parse_h0(spec);
    let mut rng = policy_rng(seed);
    let mut serialized = Vec::new();
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        let legal = legal_actions(db, &state);
        if legal.is_empty() {
            break;
        }
        let me = acting_player(&state);
        let idx = match me {
            PlayerId::A => a.choose(db, &state, &legal, &mut rng),
            PlayerId::B => b.choose(db, &state, &legal, &mut rng),
        };
        let action = legal[idx.min(legal.len().saturating_sub(1))].clone();
        serialized.push(serde_json::to_string(&to_neutral(&state, &action)).unwrap());
        apply(db, &mut state, action).expect("apply");
    }
    let count = serialized.len() as u64;
    let mut bytes = Vec::with_capacity(8 + serialized.iter().map(|s| s.len()).sum::<usize>());
    bytes.extend_from_slice(&count.to_le_bytes());
    for s in &serialized {
        bytes.extend_from_slice(s.as_bytes());
    }
    fnv1a64(&bytes)
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

fn load_meta_deck(stem: &str) -> Vec<arena_engine::CardId> {
    load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")))
}

struct Moment {
    review: &'static str,
    game: &'static str,
    ply: usize,
    bot_seed: u64,
    expect_flip_with_hbk: bool,
}

const NOISE: [Moment; 3] = [
    Moment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 27,
        bot_seed: 4984932781931433313,
        expect_flip_with_hbk: true,
    },
    Moment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 69,
        bot_seed: 14600367900189587171,
        expect_flip_with_hbk: true,
    },
    Moment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 27,
        bot_seed: 6598261483642061678,
        expect_flip_with_hbk: true,
    },
];

const CONSISTENT: [Moment; 2] = [
    Moment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 52,
        bot_seed: 6598261483642061688,
        expect_flip_with_hbk: false,
    },
    Moment {
        review: "review14",
        game: "12644260486301391828-d90f8eb2.json",
        ply: 15,
        bot_seed: 12644260486301391836,
        expect_flip_with_hbk: false,
    },
];

fn decide_moment(db: &CardDb, m: &Moment, spec: &str) -> (usize, Vec<Action>, arena_engine::State) {
    let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("fixture");
    let cap: Value = serde_json::from_str(&raw).expect("json");
    let st = replay_capture(db, &cap, m.ply);
    let legal = legal_actions(db, &st);
    let mut h0 = parse_h0(spec);
    h0.arm_explain();
    let mut rng = policy_rng(m.bot_seed);
    let pick = h0.choose(&db, &st, &legal, &mut rng);
    (pick, legal, st)
}

#[test]
fn spec_hbk_round_trip() {
    assert_eq!(AnyPolicy::parse_spec("h0:hbk=0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:hbk=16").unwrap().spec(),
        "h0:hbk=16"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn hbk_default_play_unchanged_gate() {
    let db = load_db();
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        for spec in ["h0", SERVED_SPEC, &format!("{SERVED_SPEC},hbk=0")] {
            let got = action_fingerprint(&db, spec, *seed, &deck);
            let want = if spec == "h0" {
                DEFAULT_OKILL_GATE_FPS[i]
            } else if let Some(j) = SERVED_GATE_SEEDS.iter().position(|&s| s == *seed) {
                SERVED_GATE_FPS[j]
            } else {
                continue;
            };
            assert_eq!(got, want, "{spec} fingerprint seed={seed} deck={}", stems[i]);
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn hbk_noise_moments_flip() {
    let db = load_db();
    let served_hbk = format!("{SERVED_SPEC},hbk=16");
    for m in NOISE {
        let (pick_served, legal, _) = decide_moment(&db, &m, SERVED_SPEC);
        assert!(
            matches!(legal[pick_served], Action::EndTurn),
            "{} ply {}: served spec must End Turn",
            m.game,
            m.ply
        );
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("fixture");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let st = replay_capture(&db, &cap, m.ply);
        let legal = legal_actions(&db, &st);
        let mut h0 = parse_h0(&served_hbk);
        h0.arm_explain();
        let mut rng = policy_rng(m.bot_seed);
        let pick_hbk = h0.choose(&db, &st, &legal, &mut rng);
        let rec = h0.take_explain().expect("explain");
        if m.expect_flip_with_hbk {
            assert_eq!(
                rec.path,
                ChoosePath::HoldbackResample,
                "{} ply {}: expected holdback_resample, got {:?} chosen={:?}",
                m.game,
                m.ply,
                rec.path,
                to_neutral(&st, &legal[pick_hbk])
            );
            let hr = rec.holdback_resample.expect("holdback_resample");
            assert!(hr.flipped, "{} ply {}: flipped must be true", m.game, m.ply);
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn hbk_consistent_moments_still_end_turn() {
    let db = load_db();
    let served_hbk = format!("{SERVED_SPEC},hbk=16");
    for m in CONSISTENT {
        let (pick, legal, _) = decide_moment(&db, &m, &served_hbk);
        assert!(
            matches!(legal[pick], Action::EndTurn),
            "{} ply {}: hbk=16 must still End Turn",
            m.game,
            m.ply
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn hbk_no_rng_consumption_when_keeps_end_turn() {
    let db = load_db();
    let m = &CONSISTENT[0];
    let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("fixture");
    let cap: Value = serde_json::from_str(&raw).expect("json");
    let st = replay_capture(&db, &cap, m.ply);
    let legal = legal_actions(&db, &st);
    let served_hbk = format!("{SERVED_SPEC},hbk=16");

    let mut off = parse_h0(SERVED_SPEC);
    off.arm_explain();
    let mut rng_off = policy_rng(m.bot_seed);
    let _ = off.choose(&db, &st, &legal, &mut rng_off);
    let rec_off = off.take_explain().expect("explain off");

    let mut on = parse_h0(&served_hbk);
    on.arm_explain();
    let mut rng_on = policy_rng(m.bot_seed);
    let _ = on.choose(&db, &st, &legal, &mut rng_on);
    let rec_on = on.take_explain().expect("explain on");

    assert!(
        rec_on.holdback_resample.as_ref().is_some_and(|r| !r.flipped),
        "hbk must run but not flip at Zoe moment"
    );
    assert_eq!(rec_off.candidates.len(), rec_on.candidates.len());
    for (a, b) in rec_off.candidates.iter().zip(rec_on.candidates.iter()) {
        assert_eq!(a.legal_index, b.legal_index);
        assert_eq!(a.root_agg, b.root_agg);
        assert_eq!(a.worst, b.worst);
        assert_eq!(a.n, b.n);
        assert_eq!(a.worlds.len(), b.worlds.len());
        for (wa, wb) in a.worlds.iter().zip(b.worlds.iter()) {
            assert_eq!(wa.raw, wb.raw);
            assert_eq!(wa.clamped, wb.clamped);
        }
    }
    assert_eq!(
        serde_json::to_value(&rec_off.holdback).unwrap(),
        serde_json::to_value(&rec_on.holdback).unwrap()
    );
}
