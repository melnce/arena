//! `horizon` / `hres`: finish-and-reply leaf scoring (default off).

use arena_engine::policy::ChoosePath;
use arena_engine::{
    apply, legal_actions, new_game, policy_rng, Action, AnyPolicy, CardDb, First,
    GameConfig, Phase, PlayerId, Policy, PvEnd, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

const VANILLA: &str = "88001110";
const STORM: &str = "10461110";
/// Minimal lethal budget (1 node) on an 80-node cap.
const TEST_SPEC_PREFIX: &str = "lcap=0.0125,nodes=80";

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

fn lethal_opponent_board(st: &mut arena_engine::State, db: &CardDb) {
    clear_hand(st, PlayerId::A);
    clear_hand(st, PlayerId::B);
    for _ in 0..4 {
        let slot = put_field(db, st, PlayerId::B, VANILLA);
        set_body(st, PlayerId::B, slot, 3, 3);
    }
    st.player_mut(PlayerId::A).leader_defense = 12;
    st.player_mut(PlayerId::B).leader_defense = 20;
}

/// Opponent lethal; bot has one attacker and `EndTurn` (two search candidates).
fn lethal_attack_end_state(db: &CardDb) -> (arena_engine::State, Action, Action) {
    let mut st = started(db, 71);
    lethal_opponent_board(&mut st, db);
    let slot = put_field(db, &mut st, PlayerId::A, VANILLA);
    set_body(&mut st, PlayerId::A, slot, 2, 2);
    give_pp(&mut st, PlayerId::A, 0, 3);
    let legal = legal_actions(db, &st);
    let attack = legal
        .iter()
        .find(|a| matches!(a, Action::Attack { .. }))
        .cloned()
        .expect("attack");
    let end = legal
        .iter()
        .find(|a| matches!(a, Action::EndTurn))
        .cloned()
        .expect("EndTurn");
    (st, attack, end)
}

/// Opponent lethal; bot can play Storm (listed before other candidates).
fn lethal_storm_state(db: &CardDb) -> (arena_engine::State, Action) {
    let mut st = started(db, 72);
    lethal_opponent_board(&mut st, db);
    put_hand(db, &mut st, PlayerId::A, STORM);
    give_pp(&mut st, PlayerId::A, 3, 3);
    let legal = legal_actions(db, &st);
    let play = legal
        .iter()
        .find(|a| matches!(a, Action::Play { .. }))
        .cloned()
        .expect("play");
    (st, play)
}

fn score_candidate_world(
    spec: &str,
    db: &CardDb,
    st: &arena_engine::State,
    pick: &Action,
) -> (f32, PvEnd) {
    let legal = legal_actions(db, st);
    let idx = legal.iter().position(|a| a == pick).expect("action legal");
    let mut h0 = parse_h0(spec);
    h0.arm_explain();
    let mut rng = policy_rng(71);
    let _ = h0.choose(db, st, &legal, &mut rng);
    let rec = h0.take_explain().expect("explain");
    assert_eq!(rec.path, ChoosePath::Search);
    let cand = rec
        .candidates
        .iter()
        .find(|c| c.legal_index == idx)
        .expect("candidate");
    let world = &cand.worlds[0];
    assert!(!world.skipped, "world skipped");
    (world.raw, world.end.expect("end"))
}

fn three_attacker_depth_zero_state(db: &CardDb) -> (arena_engine::State, Action) {
    let mut st = started(db, 73);
    lethal_opponent_board(&mut st, db);
    give_pp(&mut st, PlayerId::A, 0, 3);
    for _ in 0..3 {
        let s = put_field(db, &mut st, PlayerId::A, VANILLA);
        set_body(&mut st, PlayerId::A, s, 2, 2);
    }
    let legal = legal_actions(db, &st);
    let first = legal
        .iter()
        .find(|a| matches!(a, Action::Attack { .. }))
        .cloned()
        .expect("attack");
    (st, first)
}

#[test]
fn horizon1_end_turn_at_cap_scores_reply() {
    let db = load_db();
    let (st, play) = lethal_storm_state(&db);
    // Fair alloc: play's pair cap is 1 (mid-turn bare), then EndTurn's cap is 2
    // and one apply leaves nodes==cap with the turn over.
    let base = "h0:depth=6,beam=1,k=1,nodes=2,lcap=0.0125,value=v0,olethal=1,tt=0,alloc=fair";
    let legal = legal_actions(&db, &st);
    let end = legal
        .iter()
        .find(|a| matches!(a, Action::EndTurn))
        .cloned()
        .expect("EndTurn");
    let (v0, end0) = score_candidate_world(base, &db, &st, &end);
    let (v1, end1) = score_candidate_world(&format!("{base},horizon=1"), &db, &st, &end);
    assert_ne!(play, end);
    assert_eq!(end0, PvEnd::Cap);
    assert_eq!(v1, -80.0);
    assert!(matches!(end1, PvEnd::CapReply | PvEnd::OppLethal));
    assert!(v0 > v1, "answered leaf should be worse than bare cap");
}

#[test]
fn horizon2_depth0_mid_turn_scores_reply() {
    let db = load_db();
    let (st, first) = three_attacker_depth_zero_state(&db);
    let base =
        format!("h0:depth=3,beam=1,k=1,{TEST_SPEC_PREFIX},value=v0,olethal=1,tt=0,alloc=root");
    let (v1, end1) = score_candidate_world(&base, &db, &st, &first);
    let (v2, end2) = score_candidate_world(&format!("{base},horizon=2"), &db, &st, &first);
    assert!(v1 > -40.0, "depth-0 bare leaf: {v1} end={end1:?}");
    assert_eq!(v2, -80.0);
    assert_eq!(end1, PvEnd::Depth);
    assert!(matches!(end2, PvEnd::DepthReply | PvEnd::OppLethal));
}

#[test]
fn horizon2_cap_mid_turn_scores_reply() {
    let db = load_db();
    let (st, play) = lethal_storm_state(&db);
    let base = "h0:depth=6,beam=1,k=1,nodes=1,lcap=0.5,value=v0,olethal=1,tt=0,alloc=root";
    let (_v1, end1) = score_candidate_world(&format!("{base},horizon=1"), &db, &st, &play);
    let (v2, end2) = score_candidate_world(&format!("{base},horizon=2"), &db, &st, &play);
    assert_eq!(end1, PvEnd::Cap);
    assert_eq!(v2, -80.0);
    assert!(matches!(end2, PvEnd::CapReply | PvEnd::OppLethal));
}

#[test]
fn horizon3_greedy_finish_shows_in_explain() {
    let db = load_db();
    let (st, play) = lethal_storm_state(&db);
    let spec =
        "h0:depth=6,beam=1,k=1,nodes=2,lcap=0.5,value=v0,olethal=1,tt=0,horizon=3,alloc=root";
    let (raw, end) = score_candidate_world(spec, &db, &st, &play);
    assert!(matches!(end, PvEnd::CapReply | PvEnd::OppLethal));
    assert_eq!(raw, -80.0);
    let mut h0 = parse_h0(spec);
    h0.arm_explain();
    let legal = legal_actions(&db, &st);
    let mut rng = policy_rng(73);
    let _ = h0.choose(&db, &st, &legal, &mut rng);
    let rec = h0.take_explain().expect("explain");
    let idx = legal.iter().position(|a| a == &play).unwrap();
    let world = &rec
        .candidates
        .iter()
        .find(|c| c.legal_index == idx)
        .unwrap()
        .worlds[0];
    assert!(world.pv_len > 1, "greedy finish should add actions");
}

#[test]
fn horizon3_nothing_left_matches_level2() {
    let db = load_db();
    let (st, _attack, end) = lethal_attack_end_state(&db);
    let base = format!("h0:depth=6,beam=1,k=1,{TEST_SPEC_PREFIX},value=v0,olethal=1,tt=0");
    let (v2, _) = score_candidate_world(&format!("{base},horizon=2"), &db, &st, &end);
    let (v3, _) = score_candidate_world(&format!("{base},horizon=3"), &db, &st, &end);
    assert_eq!(v2, v3);
}

#[test]
fn horizon_fallback_when_turn_cannot_end() {
    let db = load_db();
    let mut st = started(&db, 74);
    lethal_opponent_board(&mut st, &db);
    give_pp(&mut st, PlayerId::A, 10, 10);
    play_id(&db, &mut st, PlayerId::A, "89201150");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    let legal = legal_actions(&db, &st);
    let _choose = legal
        .iter()
        .find(|a| matches!(a, Action::Choose(_)))
        .cloned()
        .expect("choice");
    let spec =
        "h0:depth=6,beam=1,k=1,nodes=2,lcap=0.5,hres=1,value=v0,olethal=1,tt=0,horizon=2,alloc=root";
    let mut h0 = parse_h0(spec);
    h0.arm_explain();
    let mut rng = policy_rng(74);
    let _ = h0.choose(&db, &st, &legal, &mut rng);
    assert!(h0.stats.horizon_fallback > 0 || h0.stats.horizon_leaves > 0);
}

#[test]
fn spec_horizon_hres() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    let p = AnyPolicy::parse_spec("h0:horizon=2,hres=300").unwrap();
    let AnyPolicy::H0(ref h) = p else {
        panic!("expected H0");
    };
    assert_eq!(h.horizon, 2);
    assert_eq!(h.hres, 300);
    assert_eq!(AnyPolicy::parse_spec(&p.spec()).unwrap(), p, "round-trip");
    assert!(AnyPolicy::parse_spec("h0:horizon=4").is_err());
    assert!(AnyPolicy::parse_spec("h0:hres=0").is_err());
    assert!(AnyPolicy::parse_spec("h0:horizn=1").is_err());
}

fn meta_deck_stems() -> Vec<String> {
    let text = std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json"))
        .expect("POOLS.json");
    let pools: serde_json::Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .filter(|s| s != "meta-rune-test-subject")
        .collect()
}

fn collect_meta_decisions(db: &CardDb, spec: &str, games_per_deck: u32) -> (u64, u64) {
    let mut bad_ends = 0u64;
    let mut fallback = 0u64;
    for stem in meta_deck_stems() {
        let deck = load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")));
        for g in 0..games_per_deck {
            let seed = 20260927u64
                .wrapping_add(u64::from(g))
                .wrapping_add(stem.len() as u64 * 97);
            let Ok(mut state) = new_game(
                db,
                GameConfig {
                    seed,
                    deck_a: deck.clone(),
                    deck_b: deck.clone(),
                    first: First::A,
                    opening_hands: None,
                },
            ) else {
                continue;
            };
            let mut h0 = parse_h0(spec);
            let mut rng = policy_rng(seed);
            let mut nact = 0u32;
            while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
                if state.turn > MAX_TURNS || nact >= MAX_ACTIONS {
                    break;
                }
                let legal = legal_actions(db, &state);
                if legal.is_empty() {
                    break;
                }
                if matches!(state.phase, Phase::Main) && state.turn >= 3 {
                    h0.arm_explain();
                    let _ = h0.choose(db, &state, &legal, &mut rng);
                    if let Some(rec) = h0.take_explain() {
                        if matches!(rec.path, ChoosePath::Search | ChoosePath::Unscored) {
                            for cand in &rec.candidates {
                                for world in &cand.worlds {
                                    if world.skipped {
                                        continue;
                                    }
                                    if matches!(world.end, Some(PvEnd::Depth) | Some(PvEnd::Cap)) {
                                        bad_ends += 1;
                                    }
                                }
                            }
                        }
                    }
                    fallback += h0.stats.horizon_fallback;
                    h0.reset_stats();
                }
                let idx = rng.gen_range(legal.len() as u32) as usize;
                if apply(db, &mut state, legal[idx].clone()).is_err() {
                    break;
                }
                nact += 1;
            }
        }
    }
    (bad_ends, fallback)
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn horizon2_no_bare_depth_or_cap_in_meta_games() {
    let db = load_db();
    for nodes in [2000, 16000] {
        let spec = format!("h0:nodes={nodes},horizon=2");
        let (bad, fallback) = collect_meta_decisions(&db, &spec, 2);
        assert_eq!(
            bad, fallback,
            "{spec}: depth/cap ends should match fallback"
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn horizon3_no_bare_depth_or_cap_in_meta_games() {
    let db = load_db();
    for nodes in [2000, 16000] {
        let spec = format!("h0:nodes={nodes},horizon=3");
        let (bad, fallback) = collect_meta_decisions(&db, &spec, 2);
        assert_eq!(
            bad, fallback,
            "{spec}: depth/cap ends should match fallback"
        );
    }
}
