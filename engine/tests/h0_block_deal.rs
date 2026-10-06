//! H0 block dealing (`deal=block`): disjoint hands across worlds, determinism,
//! unbiased marginals, turn-stable `deal_seed`, and timing parity.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use arena_engine::determinize::{determinize_block, determinize_with_stats, Info};
use arena_engine::policy::{ChoosePath, Deal};
use arena_engine::{
    apply, legal_actions, new_game, policy_rng, AnyPolicy, CardDb, First, GameConfig, Phase,
    PlayerId, Policy, H0,
};

mod common;
use common::*;

const SERVED_SPEC: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn collect_midgame(db: &CardDb, n: usize) -> Vec<arena_engine::State> {
    let decks = load_pinned_corpus_decks_sorted(db);
    let mut out = Vec::with_capacity(n);
    let mut seed = 1u64;
    while out.len() < n {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            db,
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
        let mut nact = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            if state.turn >= 3 && matches!(state.phase, Phase::Main) {
                out.push(state.clone());
                if out.len() >= n {
                    break;
                }
            }
            if state.turn > arena_engine::MAX_TURNS || nact >= arena_engine::MAX_ACTIONS {
                break;
            }
            let legal = legal_actions(db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
            nact += 1;
        }
        seed += 1;
        if seed > 50_000 {
            break;
        }
    }
    out
}

/// Opponent has four unknown hand cards and a large unknown pool (≥ 8 × h).
fn block_deal_fixture(db: &CardDb) -> (arena_engine::State, PlayerId) {
    let mut st = started(db, 4242);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    st.player_mut(PlayerId::A).deck.clear();
    st.player_mut(PlayerId::B).deck.clear();
    put_hand(db, &mut st, PlayerId::A, "88001110");
    for _ in 0..4 {
        put_hand(db, &mut st, PlayerId::B, "88001110");
    }
    for _ in 0..28 {
        put_deck(db, &mut st, PlayerId::B, "88001110");
    }
    assert_eq!(st.player(PlayerId::B).hand.len(), 4);
    assert!(st.player(PlayerId::B).deck.len() >= 28);
    (st, PlayerId::A)
}

fn unknown_hand_ids(state: &arena_engine::State, me: PlayerId) -> Vec<u32> {
    let opp = me.opponent();
    let (_, additions) = state.player(opp).derive_public_knowledge();
    let mut add_left: BTreeMap<arena_engine::CardId, u32> = BTreeMap::new();
    for id in additions {
        *add_left.entry(id).or_insert(0) += 1;
    }
    state
        .player(opp)
        .hand
        .iter()
        .filter(|inst| {
            if inst.flags.was_fused {
                return false;
            }
            if let Some(n) = add_left.get_mut(&inst.card) {
                if *n > 0 {
                    *n -= 1;
                    return false;
                }
            }
            true
        })
        .map(|c| c.id)
        .collect()
}

fn opponent_zone_multiset(state: &arena_engine::State, me: PlayerId) -> BTreeMap<(u32, u32), u32> {
    let opp = me.opponent();
    let mut m = BTreeMap::new();
    for inst in state
        .player(opp)
        .hand
        .iter()
        .chain(state.player(opp).deck.iter())
        .chain(state.player(opp).cemetery.iter())
        .chain(state.player(opp).banished.iter())
    {
        *m.entry((inst.card.0, inst.id)).or_insert(0) += 1;
    }
    m
}

#[test]
fn spec_round_trip() {
    assert_eq!(
        AnyPolicy::parse_spec("h0:deal=block").unwrap().spec(),
        "h0:deal=block"
    );
    assert_eq!(AnyPolicy::parse_spec("h0:deal=indep").unwrap().spec(), "h0");
    assert!(AnyPolicy::parse_spec("h0:deal=x").is_err());
    let h = parse_h0("h0:deal=block");
    assert_eq!(h.deal, Deal::Block);
}

#[test]
fn disjoint_hands_across_worlds() {
    let db = load_db();
    let (st, me) = block_deal_fixture(&db);
    let deal_seed = 0xDEAD_BEEF_CAFE_0001;
    let mut all_ids = BTreeSet::new();
    for world in 0..8u32 {
        let root_seed = 1000 + world as u64;
        let block = determinize_block(
            &st,
            me,
            root_seed,
            deal_seed,
            world,
            Info::Open,
            None,
            true,
            None,
        );
        let indep = determinize_with_stats(&st, me, root_seed, Info::Open, None);
        let ids = unknown_hand_ids(&block, me);
        assert_eq!(ids.len(), 4, "world {world} hand size");
        assert_eq!(
            block.player(me.opponent()).deck.len(),
            indep.player(me.opponent()).deck.len(),
            "world {world} deck size"
        );
        assert_eq!(
            opponent_zone_multiset(&block, me),
            opponent_zone_multiset(&indep, me),
            "world {world} opponent multiset"
        );
        for id in ids {
            assert!(
                all_ids.insert(id),
                "duplicate instance id {id} in world {world}"
            );
        }
    }
    assert_eq!(all_ids.len(), 32);
}

#[test]
fn block_deal_is_deterministic() {
    let db = load_db();
    let (st, me) = block_deal_fixture(&db);
    let a = determinize_block(&st, me, 7, 99, 3, Info::Open, None, true, None);
    let b = determinize_block(&st, me, 7, 99, 3, Info::Open, None, true, None);
    assert_eq!(
        unknown_hand_ids(&a, me),
        unknown_hand_ids(&b, me),
        "same (root_seed, deal_seed, world) -> same hand"
    );
}

#[test]
fn block_deal_unbiased_marginals() {
    let db = load_db();
    let (st, me) = block_deal_fixture(&db);
    let opp = me.opponent();
    let pool_len = st.player(opp).hand.len() + st.player(opp).deck.len();
    let h = 4usize;
    let p = h as f64 / pool_len as f64;
    let n = 4000usize;
    let mut counts0: BTreeMap<u32, u32> = BTreeMap::new();
    let mut counts5: BTreeMap<u32, u32> = BTreeMap::new();
    let pool_ids: BTreeSet<u32> = st
        .player(opp)
        .hand
        .iter()
        .chain(st.player(opp).deck.iter())
        .map(|c| c.id)
        .collect();
    for deal_seed in 1..=n as u64 {
        let s0 = determinize_block(&st, me, 1, deal_seed, 0, Info::Open, None, true, None);
        let s5 = determinize_block(&st, me, 2, deal_seed, 5, Info::Open, None, true, None);
        for id in unknown_hand_ids(&s0, me) {
            counts0.entry(id).or_insert(0);
            *counts0.get_mut(&id).unwrap() += 1;
        }
        for id in unknown_hand_ids(&s5, me) {
            counts5.entry(id).or_insert(0);
            *counts5.get_mut(&id).unwrap() += 1;
        }
    }
    let se = (p * (1.0 - p) / n as f64).sqrt();
    let tol = 4.0 * se;
    for id in pool_ids {
        let f0 = counts0.get(&id).copied().unwrap_or(0) as f64 / n as f64;
        let f5 = counts5.get(&id).copied().unwrap_or(0) as f64 / n as f64;
        assert!(
            (f0 - p).abs() <= tol,
            "world 0 card {id}: freq {f0} vs p={p} (tol {tol})"
        );
        assert!(
            (f5 - p).abs() <= tol,
            "world 5 card {id}: freq {f5} vs p={p} (tol {tol})"
        );
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

fn search_fixture_state(db: &CardDb) -> arena_engine::State {
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

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn wbase_gives_stable_deal_seed() {
    let db = load_db();
    let state = search_fixture_state(&db);
    let legal = legal_actions(&db, &state);
    assert!(legal.len() > 1, "need a search decision");
    let spec = format!("{SERVED_SPEC},deal=block,wbase=424242");
    let mut a = parse_h0(&spec);
    a.arm_explain();
    let _ = a.choose(&db, &state, &legal, &mut policy_rng(1));
    let rec_a = a.take_explain().expect("explain");
    let mut b = parse_h0(&spec);
    b.arm_explain();
    let _ = b.choose(&db, &state, &legal, &mut policy_rng(999));
    let rec_b = b.take_explain().expect("explain");
    assert_eq!(rec_a.path, ChoosePath::Search);
    assert_eq!(rec_a.deal, "block");
    assert_eq!(rec_b.deal, "block");
    assert_eq!(rec_a.deal_seed, rec_b.deal_seed);
    assert!(rec_a.deal_seed.is_some());
}

#[test]
#[ignore = "manual timing report only; CI machines vary"]
fn timing_parity_with_served_spec() {
    let db = load_db();
    let states = collect_midgame(&db, 5);
    assert_eq!(states.len(), 5, "need five mid-game positions");
    let indep_spec = SERVED_SPEC;
    let block_spec = format!("{SERVED_SPEC},deal=block");
    let mut indep_ms = 0.0;
    let mut block_ms = 0.0;
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.len() < 2 {
            continue;
        }
        let seed = 5000 + i as u64;
        let t0 = Instant::now();
        let mut h0 = parse_h0(indep_spec);
        let _ = h0.choose(&db, state, &legal, &mut policy_rng(seed));
        indep_ms += t0.elapsed().as_secs_f64() * 1000.0;
        let t1 = Instant::now();
        let mut h0 = parse_h0(&block_spec);
        let _ = h0.choose(&db, state, &legal, &mut policy_rng(seed));
        block_ms += t1.elapsed().as_secs_f64() * 1000.0;
    }
    eprintln!(
        "timing: indep {:.1} ms/decision, block {:.1} ms/decision (5 positions)",
        indep_ms / 5.0,
        block_ms / 5.0
    );
}
