//! H0 reply-model kill shapes (`okill`) and greedy play→evolve (`omacro`).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use arena_engine::{
    apply, apply_neutral, forced_lethal, legal_actions, new_game, play_game, policy_rng,
    to_neutral, trace::fnv1a64, Action, AnyPolicy, AttackTarget, CardDb, First, GameConfig,
    LethalVerdict, PlayerId, Policy, PvEnd, H0,
};
use serde_json::Value;

mod common;
use common::*;

const VANILLA: &str = "88001110";
const WARD: &str = "10061120";
const STORM: &str = "10461110";
const BITTERSWEET: &str = "10852310";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn set_body(state: &mut arena_engine::State, who: PlayerId, slot: u8, atk: i32, def: i32) {
    let f = state.field_inst_mut(who, slot).unwrap();
    f.attack = atk;
    f.defense = def;
    f.max_defense = def;
    f.flags.summoning_sick = false;
    f.flags.attacks_left = 1;
}

fn is_neg_wv(v: f32, wv: f32) -> bool {
    (v + wv).abs() < 1e-3
}

fn review5_fixture(name: &str) -> PathBuf {
    repo_root().join("py/tests/fixtures/review5").join(name)
}

fn review12_fixture(name: &str) -> PathBuf {
    repo_root().join("py/tests/fixtures/review12").join(name)
}

const GATE_SEEDS: [u64; 8] = [11, 23, 37, 41, 53, 67, 79, 97];

/// Default `h0` fingerprints on eight meta-deck / seed pairs (seat B
/// `meta-sword-rally`). Pinned on `main@ed574a4`.
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

/// `h0:okill=7` fingerprints on the same gate pairs. Pinned on `main@ed574a4`.
const OKILL7_GATE_FPS: [u64; 8] = [
    0x30c5_3aea_2056_8fa5,
    0x518a_c121_a849_591a,
    0x421d_14c6_1a80_f253,
    0x002a_10c1_1b84_7922,
    0xf99c_656c_a428_56fd,
    0x64db_f95d_b4ff_7de8,
    0xa325_ad5c_60c2_7038,
    0x2132_7f6e_557c_4256,
];

/// Served spec fingerprints (seeds 11, 23, 37). Pinned on `main@ed574a4`.
const SERVED_SPEC: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1";
const SERVED_GATE_SEEDS: [u64; 3] = [11, 23, 37];
const SERVED_GATE_FPS: [u64; 3] = [
    0xe245_2508_3e83_6195,
    0xcf64_964d_6a48_10c2,
    0x240a_8a8f_d3c7_5241,
];

const REVIEW12_GAME6: &str = "4280399497766595080-104cf827.json";
const REVIEW12_GAME6_PLY: usize = 89;
const REVIEW12_GAME6_BASE: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,info=all";
const REVIEW12_GAME6_SPEC: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,info=all,okill=8";

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

fn replay_review5(db: &CardDb, fixture: &str, n: usize) -> arena_engine::State {
    let raw = fs::read_to_string(review5_fixture(fixture)).expect("fixture");
    let cap: Value = serde_json::from_str(&raw).expect("json");
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
            let s = step["reseed"].as_u64().expect("reseed");
            st.reseed(s);
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

#[test]
fn review5_1537_after_ward_clear_core_sees_lethal() {
    let db = load_db();
    let mut st = replay_review5(&db, "15374915787985923294-5ce21003.json", 75);
    apply(&db, &mut st, Action::EndTurn).expect("EndTurn");
    for a in [
        Action::Attack {
            attacker: arena_engine::Slot(1),
            target: arena_engine::AttackTarget::Slot(arena_engine::Slot(1)),
        },
        Action::Attack {
            attacker: arena_engine::Slot(0),
            target: arena_engine::AttackTarget::Slot(arena_engine::Slot(2)),
        },
        Action::Play { hand: 2 },
        Action::Evolve {
            slot: arena_engine::Slot(0),
            super_evolve: false,
        },
        Action::Attack {
            attacker: arena_engine::Slot(0),
            target: arena_engine::AttackTarget::Slot(arena_engine::Slot(1)),
        },
    ] {
        apply(&db, &mut st, a).expect("ward clear step");
    }
    let mut h0 = parse_h0_v1("h0:olethal=1,okill=0,wv=300,nodes=16000");
    let v = h0.opponent_value(&db, &st, PlayerId::B);
    eprintln!(
        "1537 post-ward-clear okill=0 v={v} found={}",
        h0.stats.opp_lethal_found
    );
    assert!(
        is_neg_wv(v, 300.0),
        "core sweep must see lethal after wards cleared, got {v}"
    );
}

#[test]
fn review5_1537_after_end_turn_okill1() {
    let db = load_db();
    let mut st = replay_review5(&db, "15374915787985923294-5ce21003.json", 75);
    apply(&db, &mut st, Action::EndTurn).expect("EndTurn");
    assert_eq!(st.active, PlayerId::A);
    assert!(
        matches!(
            forced_lethal(&db, &st, 50_000),
            LethalVerdict::Lethal { .. }
        ),
        "review5 1537 must be lethal after EndTurn"
    );
    let mut off = parse_h0_v1("h0:olethal=1,okill=0,wv=300,nodes=16000");
    let mut on = parse_h0_v1("h0:olethal=1,okill=1,wv=300,nodes=16000");
    let ov = off.opponent_value(&db, &st, PlayerId::B);
    let nv = on.opponent_value(&db, &st, PlayerId::B);
    eprintln!(
        "1537 opp_value off={ov} on={nv} found_off={} found_on={} ward={}",
        off.stats.opp_lethal_found, on.stats.opp_lethal_found, on.stats.opp_lethal_ward_found
    );
    assert!(!is_neg_wv(ov, 300.0), "okill=0 must miss, got {ov}");
    assert!(
        is_neg_wv(nv, 300.0),
        "okill=1 must see ward-break kill, got {nv}"
    );
}

#[test]
fn review5_1043_after_end_turn_okill2() {
    let db = load_db();
    let mut st = replay_review5(&db, "10436817046835498199-5ce21003.json", 109);
    apply(&db, &mut st, Action::EndTurn).expect("EndTurn");
    assert_eq!(st.active, PlayerId::A);
    assert!(
        matches!(
            forced_lethal(&db, &st, 50_000),
            LethalVerdict::Lethal { .. }
        ),
        "review5 1043 must be lethal after EndTurn"
    );
    let mut off = parse_h0_v1("h0:olethal=1,okill=0,wv=300,nodes=16000");
    let mut on = parse_h0_v1("h0:olethal=1,okill=2,wv=300,nodes=16000");
    let ov = off.opponent_value(&db, &st, PlayerId::B);
    let nv = on.opponent_value(&db, &st, PlayerId::B);
    eprintln!(
        "1043 opp_value off={ov} on={nv} found_off={} found_on={} slot={}",
        off.stats.opp_lethal_found, on.stats.opp_lethal_found, on.stats.opp_lethal_slot_found
    );
    assert!(!is_neg_wv(ov, 300.0), "okill=0 must miss, got {ov}");
    assert!(
        is_neg_wv(nv, 300.0),
        "okill=2 must see slot-free kill, got {nv}"
    );
}

/// Opponent turn: must break a Ward before face — `okill=1` sees it.
fn ward_first_opp_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 91);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let ward = put_field(db, &mut st, PlayerId::A, WARD);
    set_body(&mut st, PlayerId::A, ward, 1, 6);
    for _ in 0..4 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    st.player_mut(PlayerId::A).leader_defense = 6;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

#[test]
fn spec_okill_omacro() {
    assert_eq!(
        AnyPolicy::parse_spec("h0:okill=7").unwrap().spec(),
        "h0:okill=7"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:omacro=1").unwrap().spec(),
        "h0:omacro=1"
    );
    let again = AnyPolicy::parse_spec("h0:okill=4,omacro=1").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
}

/// A full board must trade to play Storm; `okill=2` sees the slot-free line.
fn slot_free_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 95);
    skip_to_player_turn(db, &mut st, PlayerId::A, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    for _ in 0..5 {
        let slot = put_field(db, &mut st, PlayerId::A, VANILLA);
        set_body(&mut st, PlayerId::A, slot, 1, 1);
    }
    let body = put_field(db, &mut st, PlayerId::B, VANILLA);
    set_body(&mut st, PlayerId::B, body, 4, 2);
    put_hand(db, &mut st, PlayerId::A, STORM);
    give_pp(&mut st, PlayerId::A, 3, 6);
    st.player_mut(PlayerId::B).leader_defense = 6;
    st.player_mut(PlayerId::A).leader_defense = 20;
    assert_eq!(st.active, PlayerId::A);
    st
}

#[test]
fn slot_free_has_sacrifice_attacks() {
    let db = load_db();
    let st = slot_free_lethal_state(&db);
    let legal = legal_actions(&db, &st);
    let trades = legal
        .iter()
        .filter(|a| {
            matches!(
                a,
                Action::Attack {
                    target: AttackTarget::Slot(_),
                    ..
                }
            )
        })
        .count();
    assert!(trades > 0, "expected sacrifice attacks, got {trades}");
}

#[test]
fn okill_slot_free_fixture() {
    let db = load_db();
    const WV: f32 = 300.0;
    let st = slot_free_lethal_state(&db);
    let mut off = parse_h0_v1("h0:olethal=1,okill=0,wv=300,lcap=1,clip=0,fusemacro=0");
    let mut on = parse_h0_v1("h0:olethal=1,okill=2,wv=300,lcap=1,clip=0,fusemacro=0");
    let ov = off.opponent_value(&db, &st, PlayerId::B);
    let nv = on.opponent_value(&db, &st, PlayerId::B);
    assert!(
        !is_neg_wv(ov, WV),
        "okill=0 must miss slot-free kill, got {ov}"
    );
    assert!(
        is_neg_wv(nv, WV),
        "okill=2 must see slot-free kill, got {nv} found={} slot={}",
        on.stats.opp_lethal_found,
        on.stats.opp_lethal_slot_found
    );
    assert_eq!(on.stats.opp_lethal_slot_found, 1);
}

#[test]
fn okill_ward_break_fixture() {
    let db = load_db();
    const WV: f32 = 300.0;
    let st = ward_first_opp_lethal_state(&db);
    let mut off = parse_h0_v1("h0:olethal=1,okill=0,wv=300,lcap=1,clip=0,fusemacro=0");
    let mut on = parse_h0_v1("h0:olethal=1,okill=1,wv=300,lcap=1,clip=0,fusemacro=0");
    let ov = off.opponent_value(&db, &st, PlayerId::A);
    let nv = on.opponent_value(&db, &st, PlayerId::A);
    assert!(
        !is_neg_wv(ov, WV),
        "okill=0 must miss ward-first kill, got {ov}"
    );
    assert!(
        is_neg_wv(nv, WV),
        "okill=1 must see ward-first kill, got {nv}"
    );
    assert_eq!(on.stats.opp_lethal_found, 1);
    assert_eq!(on.stats.opp_lethal_ward_found, 1);
    assert_eq!(off.stats.opp_lethal_found, 0);
}

fn play_pair_spec(
    db: &CardDb,
    spec_a: &str,
    spec_b: &str,
    n: u32,
    seed: u64,
) -> Vec<(Option<PlayerId>, u32, u32)> {
    let decks = load_deck_file("oracle/decks/basic-forest.json");
    assert!(deck_ready(db, &decks));
    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let first = if i % 2 == 0 { First::A } else { First::B };
        let mut state = new_game(
            db,
            GameConfig {
                seed: seed.wrapping_add(u64::from(i)),
                deck_a: decks.clone(),
                deck_b: decks.clone(),
                first,
                opening_hands: None,
            },
        )
        .unwrap();
        let mut rng = policy_rng(seed.wrapping_add(u64::from(i)));
        let mut a = parse_h0(spec_a);
        let mut b = parse_h0(spec_b);
        let o = play_game(db, &mut state, &mut a, &mut b, &mut rng);
        out.push((o.winner, o.turns, o.actions));
    }
    out
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn okill0_omacro0_play_unchanged() {
    let db = load_db();
    const SEED: u64 = 20260919;
    let base = play_pair_spec(&db, &with_v2_net("h0"), &with_v2_net("h0"), 20, SEED);
    let off = play_pair_spec(
        &db,
        &with_v2_net("h0:okill=0,omacro=0"),
        &with_v2_net("h0:okill=0,omacro=0"),
        20,
        SEED,
    );
    assert_eq!(off, base, "okill=0,omacro=0 must match default h0 play");
}

#[test]
fn okill_omacro_determinism_smoke() {
    let db = load_db();
    const SEED: u64 = 20260919;
    let spec = "h0:okill=7,omacro=1,depth=2,nodes=200";
    let first = play_pair_spec(&db, spec, spec, 2, SEED);
    let again = play_pair_spec(&db, spec, spec, 2, SEED);
    assert_eq!(first, again);
}

fn meta_deck_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: Value = serde_json::from_str(&text).expect("pools json");
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

/// FNV-1a 64 over little-endian action count plus each neutral-action JSON blob.
fn action_fingerprint(db: &CardDb, spec: &str, seed: u64, deck_a: &[arena_engine::CardId]) -> u64 {
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
    while state.winner.is_none() && !matches!(state.phase, arena_engine::Phase::Terminal) {
        let legal = legal_actions(db, &state);
        if legal.is_empty() {
            break;
        }
        let me = arena_engine::acting_player(&state);
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

fn replay_review12(db: &CardDb, fixture: &str, n: usize) -> arena_engine::State {
    let raw = fs::read_to_string(review12_fixture(fixture)).expect("fixture");
    let cap: Value = serde_json::from_str(&raw).expect("json");
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
            let s = step["reseed"].as_u64().expect("reseed");
            st.reseed(s);
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

fn explain_seed(cap: &Value, ply: usize) -> u64 {
    cap["seed"].as_u64().expect("seed") + ply as u64
}

fn review12_v3_spec(base: &str) -> String {
    format!("{},net={}", base, h0_linear_v3_path())
}

fn two_storm_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 42);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    put_hand(db, &mut st, PlayerId::B, STORM);
    put_hand(db, &mut st, PlayerId::B, STORM);
    set_round(&mut st, PlayerId::B, 6);
    give_pp(&mut st, PlayerId::B, 6, 6);
    st.player_mut(PlayerId::B).ep = 0;
    st.player_mut(PlayerId::B).sep = 0;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    let storm_attack = st
        .player(PlayerId::B)
        .hand
        .iter()
        .find(|c| c.card == arena_engine::CardId::parse(STORM).unwrap())
        .map(|c| c.attack)
        .expect("Storm in hand");
    st.player_mut(PlayerId::A).leader_defense = storm_attack * 2;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

fn opp_lethal_world_count(cand: &arena_engine::policy::CandidateRecord) -> usize {
    cand.worlds
        .iter()
        .filter(|w| w.end == Some(PvEnd::OppLethal))
        .count()
}

fn opp_reply_world_count(cand: &arena_engine::policy::CandidateRecord) -> usize {
    cand.worlds
        .iter()
        .filter(|w| w.end == Some(PvEnd::OppReply))
        .count()
}

fn end_turn_candidate(
    rec: &arena_engine::policy::ExplainRecord,
) -> &arena_engine::policy::CandidateRecord {
    rec.candidates
        .iter()
        .find(|c| matches!(c.action, arena_engine::NeutralAction::EndTurn { .. }))
        .expect("EndTurn candidate")
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn okill_default_play_unchanged_gate() {
    let db = load_db();
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, "h0", *seed, &deck);
        assert_eq!(
            got, DEFAULT_OKILL_GATE_FPS[i],
            "h0 fingerprint seed={seed} deck={}",
            stems[i]
        );
        let ok7 = action_fingerprint(&db, "h0:okill=7", *seed, &deck);
        assert_eq!(
            ok7, OKILL7_GATE_FPS[i],
            "okill=7 fingerprint seed={seed} deck={}",
            stems[i]
        );
        if let Some(j) = SERVED_GATE_SEEDS.iter().position(|&s| s == *seed) {
            let served = action_fingerprint(&db, SERVED_SPEC, *seed, &deck);
            assert_eq!(
                served, SERVED_GATE_FPS[j],
                "served fingerprint seed={seed} deck={}",
                stems[i]
            );
        }
    }
}

#[test]
fn okill8_two_storm_fixture() {
    let db = load_db();
    const WV: f32 = 300.0;
    let st = two_storm_lethal_state(&db);
    let mut off = parse_h0_v1("h0:olethal=1,okill=0,wv=300,lcap=1,clip=0,fusemacro=0");
    let mut seven = parse_h0_v1("h0:olethal=1,okill=7,wv=300,lcap=1,clip=0,fusemacro=0");
    let mut on = parse_h0_v1("h0:olethal=1,okill=8,wv=300,lcap=1,clip=0,fusemacro=0");
    let ov = off.opponent_value(&db, &st, PlayerId::A);
    let sv = seven.opponent_value(&db, &st, PlayerId::A);
    let nv = on.opponent_value(&db, &st, PlayerId::A);
    assert!(
        !is_neg_wv(ov, WV),
        "okill=0 must miss two-play kill, got {ov}"
    );
    assert!(
        !is_neg_wv(sv, WV),
        "okill=7 must miss two-play kill, got {sv}"
    );
    assert!(
        is_neg_wv(nv, WV),
        "okill=8 must see two-play kill, got {nv} found={} two={}",
        on.stats.opp_lethal_found,
        on.stats.opp_lethal_two_found
    );
    assert_eq!(on.stats.opp_lethal_two_found, 1);
}

/// Two-play lethal through Bittersweet Departures' mode choice (mode 1 plus a second
/// mode), which `okill=0` and `okill=7` miss and `okill=8` finds. Passes in either
/// play order (Bittersweet first resolves its choice before relevance). The Choice
/// fix is pinned by the unit test
/// `policy::h0::okill_two_choice_second_tests::opp_lethal_two_after_storm_sees_bittersweet_choice_kill`.
fn two_play_choice_second_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 43);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    put_hand(db, &mut st, PlayerId::B, STORM);
    put_hand(db, &mut st, PlayerId::B, BITTERSWEET);
    set_round(&mut st, PlayerId::B, 6);
    give_pp(&mut st, PlayerId::B, 6, 6);
    st.player_mut(PlayerId::B).ep = 0;
    st.player_mut(PlayerId::B).sep = 0;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    let storm_attack = st
        .player(PlayerId::B)
        .hand
        .iter()
        .find(|c| c.card == arena_engine::CardId::parse(STORM).unwrap())
        .map(|c| c.attack)
        .expect("Storm in hand");
    const BITTERSWEET_FACE: i32 = 1;
    st.player_mut(PlayerId::A).leader_defense = storm_attack + BITTERSWEET_FACE;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

#[test]
fn okill8_two_play_choice_second_fixture() {
    let db = load_db();
    const WV: f32 = 300.0;
    let st = two_play_choice_second_lethal_state(&db);
    let mut off = parse_h0_v1("h0:olethal=1,okill=0,wv=300,lcap=1,clip=0,fusemacro=0");
    let mut seven = parse_h0_v1("h0:olethal=1,okill=7,wv=300,lcap=1,clip=0,fusemacro=0");
    let mut on = parse_h0_v1("h0:olethal=1,okill=8,wv=300,lcap=1,clip=0,fusemacro=0");
    let ov = off.opponent_value(&db, &st, PlayerId::A);
    let sv = seven.opponent_value(&db, &st, PlayerId::A);
    let nv = on.opponent_value(&db, &st, PlayerId::A);
    assert!(
        !is_neg_wv(ov, WV),
        "okill=0 must miss two-play kill, got {ov}"
    );
    assert!(
        !is_neg_wv(sv, WV),
        "okill=7 must miss two-play kill, got {sv}"
    );
    assert!(
        is_neg_wv(nv, WV),
        "okill=8 must see two-play kill via choice second play, got {nv} found={} two={}",
        on.stats.opp_lethal_found,
        on.stats.opp_lethal_two_found
    );
    assert_eq!(on.stats.opp_lethal_two_found, 1);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn review12_game6_okill8_sees_sephie_kill() {
    let db = load_db();
    let raw = fs::read_to_string(review12_fixture(REVIEW12_GAME6)).expect("fixture");
    let cap: Value = serde_json::from_str(&raw).expect("json");
    let st = replay_review12(&db, REVIEW12_GAME6, REVIEW12_GAME6_PLY);
    assert_eq!(st.active, PlayerId::B, "bot must be to move");
    let spec_on = review12_v3_spec(REVIEW12_GAME6_SPEC);
    let spec_off = review12_v3_spec(REVIEW12_GAME6_BASE);
    let base_seed = explain_seed(&cap, REVIEW12_GAME6_PLY);
    for seed in [1u64, 2, 3] {
        let mut on = parse_h0(&spec_on);
        on.arm_explain();
        let legal = legal_actions(&db, &st);
        let mut rng = policy_rng(base_seed + seed - 1);
        let pick = on.choose(&db, &st, &legal, &mut rng);
        let rec = on.take_explain().expect("explain on");
        let end_cand = end_turn_candidate(&rec);
        let opp_lethal = opp_lethal_world_count(end_cand);
        if opp_lethal < 8 {
            eprintln!(
                "seed={seed}: EndTurn opp_lethal {opp_lethal}/8 worlds={:?}",
                end_cand
                    .worlds
                    .iter()
                    .map(|w| (w.r, w.end))
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(
            opp_lethal, 8,
            "seed={seed}: EndTurn must end opp_lethal in all 8 worlds"
        );
        let chosen = &legal[pick];
        assert!(
            matches!(
                chosen,
                Action::Attack {
                    target: AttackTarget::Slot(arena_engine::Slot(1)),
                    ..
                }
            ),
            "seed={seed}: must attack Sephie on slot 1, got {chosen:?}"
        );

        let mut off = parse_h0(&spec_off);
        off.arm_explain();
        let mut rng_off = policy_rng(base_seed + seed - 1);
        off.choose(&db, &st, &legal, &mut rng_off);
        let rec_off = off.take_explain().expect("explain off");
        let end_off = end_turn_candidate(&rec_off);
        assert_eq!(
            opp_reply_world_count(end_off),
            8,
            "seed={seed}: without okill=8 EndTurn must end opp_reply 8/8"
        );
    }
}
