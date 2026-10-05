//! H0 turn-stable worlds (`wseed=turn`, `wbase`) and lost-turn tie-break (`lostrank`).

use arena_engine::determinize::determinize_with;
use arena_engine::policy::ChoosePath;
use arena_engine::{
    apply, legal_actions, new_game, policy_rng, Action, AnyPolicy, CardDb, First, GameConfig,
    Phase, PlayerId, Policy, Xoshiro256ss, H0,
};
mod common;
use common::*;

const SERVED_SPEC: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000";
const TURN_SPEC: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,wseed=turn";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn first_legal() -> arena_engine::FirstLegal {
    match AnyPolicy::parse_spec("first-legal").unwrap() {
        AnyPolicy::FirstLegal(p) => p,
        other => panic!("first-legal parsed as {other:?}"),
    }
}

const VANILLA: &str = "88001110";

fn set_body(state: &mut arena_engine::State, who: PlayerId, slot: u8, atk: i32, def: i32) {
    let f = state.field_inst_mut(who, slot).unwrap();
    f.attack = atk;
    f.defense = def;
    f.max_defense = def;
    f.flags.summoning_sick = false;
    f.flags.attacks_left = 1;
}

/// Bot at 1 HP vs overkill on board; ward play beats ending on lost rerank.
fn lost_turn_bot_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 9001);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    for _ in 0..5 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 20, 20);
    }
    st.player_mut(PlayerId::A).leader_defense = 5;
    let mine = put_field(db, &mut st, PlayerId::A, VANILLA);
    set_body(&mut st, PlayerId::A, mine, 1, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    assert_eq!(arena_engine::acting_player(&st), PlayerId::A);
    st
}

const LOST_SPEC: &str =
    "h0:value=v0,olethal=1,horizon=3,depth=3,k=1,nodes=32000,hbcheck=0,info=all,wv=400";

fn world_seeds_from_base(base: u64, k: u32) -> Vec<u64> {
    let mut rng = Xoshiro256ss::from_seed(base);
    (0..k).map(|_| rng.next_u64()).collect()
}

fn opponent_hands_match_worlds(
    before: &arena_engine::State,
    after: &arena_engine::State,
    base: u64,
    k: u32,
) {
    let me = arena_engine::acting_player(before);
    let opp = me.opponent();
    let info = arena_engine::policy::Info::Open;
    for seed in world_seeds_from_base(base, k) {
        let r0 = determinize_with(before, me, seed, info);
        let r1 = determinize_with(after, me, seed, info);
        let h0 = r0
            .player(opp)
            .hand
            .iter()
            .map(|c| c.card.to_string())
            .collect::<Vec<_>>();
        let h1 = r1
            .player(opp)
            .hand
            .iter()
            .map(|c| c.card.to_string())
            .collect::<Vec<_>>();
        assert_eq!(h0, h1);
    }
}

fn world_raw_vectors(rec: &arena_engine::ExplainRecord) -> Vec<Vec<f32>> {
    rec.candidates
        .iter()
        .map(|c| c.worlds.iter().map(|w| w.raw).collect())
        .collect()
}

#[test]
fn spec_round_trip() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:wseed=turn").unwrap().spec(),
        "h0:wseed=turn"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:wbase=42").unwrap().spec(),
        "h0:wbase=42"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:lostrank=2000").unwrap().spec(),
        "h0:lostrank=2000"
    );
    let both = AnyPolicy::parse_spec("h0:wseed=turn,wbase=1,lostrank=2000").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&both.spec()).unwrap(), both);
    assert!(AnyPolicy::parse_spec("h0:wseed=x").is_err());
    assert!(AnyPolicy::parse_spec("h0:lostrank=1000001").is_err());
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn wseed_off_has_no_wbase() {
    let db = load_db();
    let decks = load_pinned_corpus_decks_sorted(&db);
    let mut state = new_game(
        &db,
        GameConfig {
            seed: 77,
            deck_a: decks[0].clone(),
            deck_b: decks[1].clone(),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut rng = policy_rng(77);
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        let legal = legal_actions(&db, &state);
        if legal.len() <= 1 {
            break;
        }
        let me = arena_engine::acting_player(&state);
        if me == PlayerId::A
            && matches!(
                state.phase,
                Phase::Main | Phase::Combat | Phase::Choice { .. }
            )
        {
            let mut h0 = parse_h0(SERVED_SPEC);
            h0.arm_explain();
            let _ = h0.choose(&db, &state, &legal, &mut rng);
            let rec = h0.take_explain().expect("explain");
            if rec.path == ChoosePath::Search {
                assert!(rec.wbase.is_none());
                assert!(!rec.wbase_reused);
                return;
            }
        }
        let idx = rng.gen_range(legal.len() as u32) as usize;
        apply(&db, &mut state, legal[idx].clone()).expect("apply");
    }
    panic!("no search decision found");
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn wseed_turn_reuses_base_within_turn() {
    let db = load_db();
    let mut state = started(&db, 101);
    clear_hand(&mut state, PlayerId::A);
    put_hand(&db, &mut state, PlayerId::A, VANILLA);
    put_hand(&db, &mut state, PlayerId::A, VANILLA);
    give_pp(&mut state, PlayerId::A, 10, 10);
    let mut h0 = parse_h0(TURN_SPEC);
    let mut rng = policy_rng(101);
    let legal = legal_actions(&db, &state);
    h0.arm_explain();
    let idx0 = h0.choose(&db, &state, &legal, &mut rng);
    let rec0 = h0.take_explain().expect("explain");
    assert_eq!(rec0.path, ChoosePath::Search);
    let base = rec0.wbase.expect("wbase");
    assert!(!rec0.wbase_reused);
    let before = state.clone();
    apply(&db, &mut state, legal[idx0].clone()).expect("apply");
    let legal2 = legal_actions(&db, &state);
    assert!(legal2.len() > 1);
    h0.arm_explain();
    let _ = h0.choose(&db, &state, &legal2, &mut rng);
    let rec1 = h0.take_explain().expect("explain");
    assert_eq!(rec1.path, ChoosePath::Search);
    assert_eq!(rec1.wbase, Some(base));
    assert!(rec1.wbase_reused);
    opponent_hands_match_worlds(&before, &state, base, rec0.k);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn wseed_turn_new_base_on_next_turn() {
    let db = load_db();
    let decks = load_pinned_corpus_decks_sorted(&db);
    let mut state = new_game(
        &db,
        GameConfig {
            seed: 202,
            deck_a: decks[4].clone(),
            deck_b: decks[5].clone(),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut h0 = parse_h0(TURN_SPEC);
    let mut rng = policy_rng(202);
    let mut bases: Vec<u64> = Vec::new();
    let mut last_turn = state.turn;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) && bases.len() < 2 {
        let legal = legal_actions(&db, &state);
        if legal.is_empty() {
            break;
        }
        let me = arena_engine::acting_player(&state);
        if me == PlayerId::A
            && matches!(
                state.phase,
                Phase::Main | Phase::Combat | Phase::Choice { .. }
            )
        {
            h0.arm_explain();
            let _ = h0.choose(&db, &state, &legal, &mut rng);
            let rec = h0.take_explain().expect("explain");
            if rec.path == ChoosePath::Search {
                let base = rec.wbase.expect("wbase");
                if bases.is_empty() || state.turn != last_turn {
                    bases.push(base);
                    last_turn = state.turn;
                }
            }
        }
        let idx = if arena_engine::acting_player(&state) == PlayerId::A {
            h0.choose(&db, &state, &legal, &mut rng)
        } else {
            first_legal().choose(&db, &state, &legal, &mut rng)
        };
        apply(&db, &mut state, legal[idx].clone()).expect("apply");
    }
    assert_eq!(bases.len(), 2);
    assert_ne!(bases[0], bases[1], "next turn should draw a new base");
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn wbase_deterministic_across_call_seeds() {
    let db = load_db();
    let state = lost_turn_bot_state(&db);
    let legal = legal_actions(&db, &state);
    let spec = format!("{SERVED_SPEC},wbase=424242");
    let mut a = parse_h0(&spec);
    a.arm_explain();
    let _ = a.choose(&db, &state, &legal, &mut policy_rng(1));
    let rec_a = a.take_explain().expect("explain");
    let mut b = parse_h0(&spec);
    b.arm_explain();
    let _ = b.choose(&db, &state, &legal, &mut policy_rng(999));
    let rec_b = b.take_explain().expect("explain");
    assert_eq!(rec_a.path, ChoosePath::Search);
    assert_eq!(rec_a.wbase, Some(424242));
    assert_eq!(world_raw_vectors(&rec_a), world_raw_vectors(&rec_b));
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn no_wbase_reuse_across_games() {
    let db = load_db();
    let decks = load_pinned_corpus_decks_sorted(&db);
    let cfg = GameConfig {
        seed: 404,
        deck_a: decks[0].clone(),
        deck_b: decks[1].clone(),
        first: First::A,
        opening_hands: None,
    };
    let mut bases = Vec::new();
    for game in 0..2 {
        let mut state = new_game(&db, cfg.clone()).expect("new_game");
        let mut h0 = parse_h0(TURN_SPEC);
        let mut rng = policy_rng(404 + game);
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            if arena_engine::acting_player(&state) == PlayerId::A
                && matches!(
                    state.phase,
                    Phase::Main | Phase::Combat | Phase::Choice { .. }
                )
            {
                h0.arm_explain();
                let _ = h0.choose(&db, &state, &legal, &mut rng);
                let rec = h0.take_explain().expect("explain");
                if rec.path == ChoosePath::Search {
                    bases.push(rec.wbase.expect("wbase"));
                    break;
                }
            }
            let idx = h0.choose(&db, &state, &legal, &mut rng);
            apply(&db, &mut state, legal[idx].clone()).expect("apply");
        }
    }
    assert_eq!(bases.len(), 2);
    assert_ne!(bases[0], bases[1]);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn lostrank_reranks_lost_turn() {
    let db = load_db();
    let st = lost_turn_bot_state(&db);
    let legal = legal_actions(&db, &st);
    assert!(legal.len() > 1);
    let seed = 77;
    let off_spec = format!("{LOST_SPEC},lostrank=0");
    let on_spec = format!("{LOST_SPEC},lostrank=2000");
    let mut off = parse_h0(&off_spec);
    off.arm_explain();
    let mut rng0 = policy_rng(seed);
    let pick0 = off.choose(&db, &st, &legal, &mut rng0);
    let rec0 = off.take_explain().expect("explain");
    assert!(rec0.candidates.iter().all(|c| c.root_agg <= -399.999));
    let mut on = parse_h0(&on_spec);
    on.arm_explain();
    let mut rng1 = policy_rng(seed);
    let pick1 = on.choose(&db, &st, &legal, &mut rng1);
    let rec1 = on.take_explain().expect("explain");
    assert!(rec1.lost_rerank.is_some(), "lost rerank should fire");
    assert_eq!(on.last_value(), off.last_value());
    assert_eq!(off.last_value(), Some(-400.0));
    assert_ne!(pick0, pick1, "lostrank should change the pick");
    let lr = rec1.lost_rerank.unwrap();
    assert_eq!(lr.chosen_index, pick1);
    assert!(lr.nodes > 0);
    assert_eq!(lr.aggregates.len(), rec1.candidates.len());
    assert!(
        matches!(
            legal[pick0],
            Action::Attack {
                target: arena_engine::AttackTarget::Slot(_),
                ..
            }
        ),
        "lostrank=0 should take the first tied loss"
    );
    assert!(
        matches!(
            legal[pick1],
            Action::Attack {
                target: arena_engine::AttackTarget::Leader,
                ..
            }
        ),
        "lostrank should prefer the face attack line"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn lostrank_absent_when_not_lost() {
    let db = load_db();
    let decks = load_pinned_corpus_decks_sorted(&db);
    let mut state = new_game(
        &db,
        GameConfig {
            seed: 505,
            deck_a: decks[8].clone(),
            deck_b: decks[9].clone(),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let spec0 = format!("{SERVED_SPEC},lostrank=0");
    let spec2 = format!("{SERVED_SPEC},lostrank=2000");
    let mut rng = policy_rng(505);
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        let legal = legal_actions(&db, &state);
        if legal.len() <= 1 {
            break;
        }
        let me = arena_engine::acting_player(&state);
        if me == PlayerId::A
            && matches!(
                state.phase,
                Phase::Main | Phase::Combat | Phase::Choice { .. }
            )
        {
            let mut a = parse_h0(&spec0);
            a.arm_explain();
            let mut rng0 = policy_rng(505);
            let pick0 = a.choose(&db, &state, &legal, &mut rng0);
            let rec0 = a.take_explain().expect("explain");
            let mut b = parse_h0(&spec2);
            b.arm_explain();
            let mut rng1 = policy_rng(505);
            let pick1 = b.choose(&db, &state, &legal, &mut rng1);
            let rec1 = b.take_explain().expect("explain");
            if rec0.path == ChoosePath::Search {
                let all_lost = rec0.candidates.iter().all(|c| c.root_agg <= -79.999);
                if !all_lost {
                    assert_eq!(pick0, pick1);
                    assert!(rec1.lost_rerank.is_none());
                    return;
                }
            }
        }
        let idx = rng.gen_range(legal.len() as u32) as usize;
        apply(&db, &mut state, legal[idx].clone()).expect("apply");
    }
    panic!("no non-lost search position found");
}
