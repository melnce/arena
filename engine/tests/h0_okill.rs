//! H0 reply-model kill shapes (`okill`) and greedy play→evolve (`omacro`).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use arena_engine::{
    apply, apply_neutral, forced_lethal, legal_actions, new_game, play_game, policy_rng, Action,
    AnyPolicy, AttackTarget, CardDb, First, GameConfig, LethalVerdict, PlayerId, H0,
};
use serde_json::Value;

mod common;
use common::*;

const VANILLA: &str = "88001110";
const WARD: &str = "10061120";
const STORM: &str = "10461110";

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
