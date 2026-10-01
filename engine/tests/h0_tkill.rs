//! H0 own-turn deterministic kill (`tkill`): identity at default off,
//! spec round-trip, missed-kill fixtures, and no-change when solver empty.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use arena_engine::policy::ChoosePath;
use arena_engine::{
    apply, apply_neutral, forced_lethal, forced_lethal_det, legal_actions, new_game, policy_rng,
    Action, AnyPolicy, CardDb, First, GameConfig, LethalVerdict, Phase, PlayerId, Policy, H0,
};
use serde_json::Value;

mod common;
use common::*;

const STORM: &str = "10461110";
const OMEGOTEP: &str = "10604110";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn tkill_fixture_dir() -> PathBuf {
    fixtures_dir().join("tkill")
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

fn explain_seed(cap: &Value, ply: usize) -> u64 {
    cap["seed"].as_u64().expect("seed") + ply as u64
}

#[test]
fn spec_tkill_round_trip() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:tkill=0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:tkill=2000").unwrap().spec(),
        "h0:tkill=2000"
    );
    let both = AnyPolicy::parse_spec("h0:tkill=1000,horizon=3").unwrap();
    assert_eq!(both.spec(), "h0:tkill=1000,horizon=3");
    assert_eq!(AnyPolicy::parse_spec(&both.spec()).unwrap(), both);
    let e = AnyPolicy::parse_spec("h0:tkill=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

#[test]
fn tkill0_identity_smoke() {
    let db = load_db();
    let st = two_storm_lethal_state(&db);
    let legal = legal_actions(&db, &st);
    let mut off = parse_h0("h0:tkill=0");
    let mut def = parse_h0("h0");
    let mut rng0 = policy_rng(11);
    let mut rng1 = policy_rng(11);
    let i0 = off.choose(&db, &st, &legal, &mut rng0);
    let i1 = def.choose(&db, &st, &legal, &mut rng1);
    assert_eq!(off.stats.own_solver_calls, 0);
    assert_eq!(def.stats.own_solver_calls, 0);
    assert_eq!(i0, i1);
}

fn two_storm_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 83);
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
        .find(|c| c.card == cid(STORM))
        .map(|c| c.attack)
        .expect("Storm in hand");
    st.player_mut(PlayerId::A).leader_defense = storm_attack * 2;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

fn omegotep_rng_kill_state(db: &CardDb, seed: u64) -> arena_engine::State {
    let mut st = started(db, seed);
    skip_to_player_turn(db, &mut st, PlayerId::A, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    put_hand(db, &mut st, PlayerId::A, OMEGOTEP);
    set_round(&mut st, PlayerId::A, 10);
    give_pp(&mut st, PlayerId::A, 9, 10);
    st.player_mut(PlayerId::B).leader_defense = 2;
    st.player_mut(PlayerId::A).leader_defense = 20;
    st
}

#[test]
fn forced_lethal_det_solver_behaviour() {
    let db = load_db();
    let det_st = two_storm_lethal_state(&db);
    match forced_lethal(&db, &det_st, 50_000) {
        LethalVerdict::Lethal { rng_dependent, .. } => assert!(!rng_dependent),
        other => panic!("expected deterministic Lethal, got {other:?}"),
    }
    match forced_lethal_det(&db, &det_st, 50_000) {
        LethalVerdict::Lethal { rng_dependent, .. } => assert!(!rng_dependent),
        other => panic!("expected Lethal from det solver, got {other:?}"),
    }

    let mut found = false;
    for seed in 1..8_000 {
        let st = omegotep_rng_kill_state(&db, seed);
        let full = forced_lethal(&db, &st, 2_000);
        let det = forced_lethal_det(&db, &st, 2_000);
        if matches!(
            full,
            LethalVerdict::Lethal {
                rng_dependent: true,
                ..
            }
        ) && matches!(
            det,
            LethalVerdict::None { .. } | LethalVerdict::Unknown { .. }
        ) {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "need a seed where full solver finds an rng-dependent kill and det does not"
    );

    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, STORM);
    put_hand(&db, &mut st, PlayerId::A, STORM);
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().any(|a| matches!(a, Action::Play { .. })));
    assert!(matches!(
        forced_lethal_det(&db, &st, 0),
        LethalVerdict::Unknown { nodes: 0 }
    ));
}

#[test]
fn missed_kill_fixtures_taken_by_tkill() {
    let db = load_db();
    let dir = tkill_fixture_dir();
    let entries = fs::read_dir(&dir).expect("tkill fixtures dir");
    let mut any = false;
    for ent in entries.flatten() {
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
        let ply = cap["ply"].as_u64().expect("ply") as usize;
        let st = replay_capture(&db, &cap, ply);
        let me = st.active;
        let legal = legal_actions(&db, &st);
        assert!(legal.len() > 1, "{path:?}: need multi-choice decision");
        let mut plain = parse_h0("h0:nodes=16000,horizon=3");
        let mut kill = parse_h0("h0:nodes=16000,horizon=3,tkill=50000");
        kill.arm_explain();
        let seed = explain_seed(&cap, ply);
        let mut rng_plain = policy_rng(seed);
        let mut rng_kill = policy_rng(seed);
        let plain_pick = plain.choose(&db, &st, &legal, &mut rng_plain);
        let kill_pick = kill.choose(&db, &st, &legal, &mut rng_kill);
        let rec = kill.take_explain().expect("explain");
        assert_eq!(rec.path, ChoosePath::TakeKill, "{path:?}");
        match forced_lethal_det(&db, &st, 50_000) {
            LethalVerdict::Lethal { line, .. } => {
                let first = &line[0];
                let expected = legal
                    .iter()
                    .position(|a| a == first)
                    .expect("line[0] legal");
                assert_eq!(kill_pick, expected, "{path:?}");
            }
            other => panic!("{path:?}: expected det lethal at fixture, got {other:?}"),
        }
        assert_ne!(
            plain_pick, kill_pick,
            "{path:?}: plain h0 must still miss the kill"
        );
        let mut walk = st.clone();
        let mut h0 = parse_h0("h0:nodes=16000,horizon=3,tkill=50000");
        let mut rng = policy_rng(seed);
        while walk.winner.is_none() && walk.active == me {
            if matches!(walk.phase, Phase::End | Phase::Terminal) {
                break;
            }
            let legal = legal_actions(&db, &walk);
            if legal.is_empty() {
                break;
            }
            let idx = h0.choose(&db, &walk, &legal, &mut rng);
            let a = legal[idx].clone();
            if matches!(a, Action::EndTurn) {
                panic!("{path:?}: tkill policy ended turn before lethal");
            }
            apply(&db, &mut walk, a).expect("apply");
        }
        assert_eq!(walk.winner, Some(me), "{path:?}: must win before EndTurn");
        any = true;
    }
    assert!(
        any,
        "need at least one tkill fixture under engine/tests/fixtures/tkill/"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn no_kill_no_change_sample() {
    let db = load_db();
    let snaps = collect_no_det_kill_snapshots(&db, 250);
    assert!(
        snaps.len() >= 200,
        "need >= 200 own-turn samples, got {}",
        snaps.len()
    );
    let mut compared = 0usize;
    for snap in snaps {
        let legal = legal_actions(&db, &snap.state);
        if legal.len() <= 1 {
            continue;
        }
        let mut rng0 = policy_rng(snap.choose_seed);
        let mut rng1 = policy_rng(snap.choose_seed);
        let mut off = parse_h0("h0");
        let mut on = parse_h0("h0:tkill=2000");
        let i0 = off.choose(&db, &snap.state, &legal, &mut rng0);
        let i1 = on.choose(&db, &snap.state, &legal, &mut rng1);
        compared += 1;
        assert_eq!(i0, i1, "seed={}", snap.choose_seed);
    }
    assert!(compared >= 200, "compared only {compared}");
}

struct SearchSnapshot {
    state: arena_engine::State,
    choose_seed: u64,
}

fn collect_no_det_kill_snapshots(db: &CardDb, target: usize) -> Vec<SearchSnapshot> {
    let decks = [
        "oracle/decks/meta-forest-combo.json",
        "oracle/decks/meta-rune-crystal.json",
        "oracle/decks/meta-sword-rally.json",
        "oracle/decks/meta-abyss-midrange.json",
        "oracle/decks/meta-dragon-ramp.json",
        "oracle/decks/meta-haven-evo.json",
    ];
    let mut snaps = Vec::new();
    for (di, deck_path) in decks.iter().enumerate() {
        let deck = load_deck_file(deck_path);
        for g in 0..6 {
            let seed = 40_000 + di as u64 * 100 + g as u64;
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
            let mut rng = policy_rng(seed);
            let mut nact = 0u32;
            while state.winner.is_none() && state.turn < 16 {
                let legal = legal_actions(db, &state);
                if legal.is_empty() {
                    break;
                }
                if matches!(state.phase, Phase::Main | Phase::Combat)
                    && state.active == PlayerId::A
                    && legal.len() > 1
                {
                    match forced_lethal_det(db, &state, 2_000) {
                        LethalVerdict::None { .. } | LethalVerdict::Unknown { .. } => {
                            snaps.push(SearchSnapshot {
                                state: state.clone(),
                                choose_seed: seed
                                    .wrapping_add(u64::from(state.turn) * 97)
                                    .wrapping_add(u64::from(nact) * 131),
                            });
                            if snaps.len() >= target {
                                return snaps;
                            }
                        }
                        LethalVerdict::Lethal { .. } => {}
                    }
                }
                let idx = (rng.next_u64() % legal.len() as u64) as usize;
                if apply(db, &mut state, legal[idx].clone()).is_err() {
                    break;
                }
                nact += 1;
            }
        }
    }
    snaps
}
