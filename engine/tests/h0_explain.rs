//! H0 explain sink: identity, tie set, leaf/raw invariant, PV replay.

use arena_engine::policy::Info;
use arena_engine::{
    apply, determinize::determinize_with, from_neutral, legal_actions, new_game, policy_rng,
    AnyPolicy, CardDb, ChoosePath, First, Phase, Policy, PvEnd, H0,
};

mod common;
use common::*;

const SPECS: &[&str] = &[
    "h0",
    "h0:lcap=1,clip=0,fusemacro=0",
    "h0:value=v0",
    "h0:nodes=6000",
    "h0:k=1",
    "h0:info=all",
    "h0:fusemacro=0",
];

const REPLAY_SPECS: &[&str] = &[
    "h0:value=v0,olethal=0,osteps=0,tt=0",
    "h0:value=v0,olethal=0,osteps=0,tt=1",
    "h0:k=1,info=all,value=v0,olethal=0,osteps=0,odepth=0,tt=0",
    "h0:value=v0,olethal=0,osteps=0,tt=0,fusemacro=1",
];

fn collect_states(db: &CardDb, n: usize, midgame_only: bool) -> Vec<arena_engine::State> {
    let decks = load_pinned_corpus_decks_sorted(db);
    let mut out = Vec::with_capacity(n);
    let mut seed = 1u64;
    while out.len() < n {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            db,
            arena_engine::GameConfig {
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
            let take = !midgame_only || (state.turn >= 3 && matches!(state.phase, Phase::Main));
            if take {
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

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn rebuild_roots(
    state: &arena_engine::State,
    seed: u64,
    k: u32,
    info: Info,
) -> Vec<arena_engine::State> {
    let me = arena_engine::acting_player(state);
    let mut rng = policy_rng(seed);
    let mut roots = Vec::with_capacity(k as usize);
    for _ in 0..k {
        roots.push(determinize_with(state, me, rng.next_u64(), info));
    }
    roots
}

fn check_explain_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    for spec in SPECS {
        for (i, state) in states.iter().enumerate() {
            let legal = legal_actions(&db, state);
            if legal.is_empty() {
                continue;
            }
            let seed = 20260923u64.wrapping_add(i as u64);
            let mut plain = parse_h0(spec);
            let mut armed = parse_h0(spec);
            let mut rng_p = policy_rng(seed);
            let mut rng_a = policy_rng(seed);
            let ip = plain.choose(&db, state, &legal, &mut rng_p);
            armed.arm_explain();
            let ia = armed.choose(&db, state, &legal, &mut rng_a);
            assert_eq!(ip, ia, "{spec} action mismatch at state {i}");
            assert_eq!(
                plain.stats.nodes, armed.stats.nodes,
                "{spec} nodes mismatch at state {i}: plain={} armed={}",
                plain.stats.nodes, armed.stats.nodes
            );
            let rec = armed.take_explain().expect("explain record");
            assert!(
                rec.tie_set.contains(&rec.chosen_index),
                "{spec} chosen not in tie set at state {i}"
            );
            let min_tie = *rec.tie_set.iter().min().expect("nonempty tie set");
            assert_eq!(
                rec.chosen_index, min_tie,
                "{spec} tie-break not lowest index at state {i}"
            );
            if rec.path == ChoosePath::Unscored {
                assert_eq!(
                    rec.chosen_index,
                    *rec.tie_set.iter().min().unwrap(),
                    "{spec} unscored tie-break at state {i}"
                );
            }
            if matches!(rec.path, ChoosePath::Search | ChoosePath::Unscored) {
                let k = rec.k;
                for cand in &rec.candidates {
                    assert_eq!(
                        cand.worlds.len(),
                        k as usize,
                        "{spec} candidate {} has {} worlds, want k={} at state {i}",
                        cand.legal_index,
                        cand.worlds.len(),
                        k
                    );
                }
            }
        }
    }
}

#[test]
fn explain_identity_smoke() {
    check_explain_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn explain_identity_200_midgame() {
    check_explain_identity(200);
}

fn check_leaf_equals_raw(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    for spec in SPECS {
        for (i, state) in states.iter().enumerate() {
            let legal = legal_actions(&db, state);
            if legal.is_empty() {
                continue;
            }
            let seed = 20260924u64.wrapping_add(i as u64);
            let mut h0 = parse_h0(spec);
            h0.arm_explain();
            let mut rng = policy_rng(seed);
            let _ = h0.choose(&db, state, &legal, &mut rng);
            let rec = h0.take_explain().expect("explain");
            if !matches!(rec.path, ChoosePath::Search | ChoosePath::Unscored) {
                continue;
            }
            for cand in &rec.candidates {
                for world in &cand.worlds {
                    if world.skipped {
                        continue;
                    }
                    let Some(leaf) = world.leaf.as_ref() else {
                        continue;
                    };
                    assert!(
                        (leaf.value - world.raw).abs() < 1e-6,
                        "{spec} leaf != raw at state {i} cand {} world {}: leaf={} raw={}",
                        cand.legal_index,
                        world.r,
                        leaf.value,
                        world.raw
                    );
                }
            }
        }
    }
}

#[test]
fn leaf_equals_raw_smoke() {
    check_leaf_equals_raw(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn leaf_equals_raw_200_midgame() {
    check_leaf_equals_raw(200);
}

fn check_pv_replay_raw(n: usize, min_replayed: u32) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    for spec in REPLAY_SPECS {
        let h0_cfg = parse_h0(spec);
        let tt_off = !spec.contains("tt=1");
        let info = h0_cfg.info;
        let mut replayed = 0u32;
        for (i, state) in states.iter().enumerate() {
            let legal = legal_actions(&db, state);
            if legal.is_empty() {
                continue;
            }
            let seed = 20260925u64.wrapping_add(i as u64);
            let mut h0 = parse_h0(spec);
            h0.arm_explain();
            let mut rng = policy_rng(seed);
            let _ = h0.choose(&db, state, &legal, &mut rng);
            let rec = h0.take_explain().expect("explain");
            if rec.path != ChoosePath::Search {
                continue;
            }
            let me = arena_engine::acting_player(state);
            let roots = rebuild_roots(state, seed, rec.k, info);
            for cand in &rec.candidates {
                for world in &cand.worlds {
                    if world.skipped || world.pv.is_empty() {
                        continue;
                    }
                    let Some(end) = world.end else {
                        continue;
                    };
                    if tt_off && end == PvEnd::Tt {
                        panic!("{spec} tt end with tt=0 at state {i} world {}", world.r);
                    }
                    if end != PvEnd::Depth && end != PvEnd::Cap && end != PvEnd::OppReply {
                        continue;
                    }
                    if world.pv_len > 12 {
                        continue;
                    }
                    let root = &roots[world.r as usize];
                    let mut s = root.clone();
                    for neu in &world.pv {
                        let a = from_neutral(&s, neu).expect("pv action maps");
                        apply(&db, &mut s, a).expect("pv replay legal");
                    }
                    let v = h0_cfg.evaluate(&db, &s, me);
                    assert!(
                        (v - world.raw).abs() < 1e-3,
                        "{spec} pv replay != raw at state {i} world {}: replay={v} raw={}",
                        world.r,
                        world.raw
                    );
                    replayed += 1;
                }
            }
        }
        assert!(
            replayed >= min_replayed,
            "{spec}: only {replayed} replayed worlds (need >= {min_replayed})"
        );
        eprintln!("{spec}: replayed {replayed} worlds");
    }
}

#[test]
fn pv_replay_raw_smoke() {
    check_pv_replay_raw(8, 1);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn pv_replay_raw_200_midgame() {
    check_pv_replay_raw(200, 100);
}

#[test]
fn bench_nodes_unchanged_without_sink() {
    let db = load_db();
    // Debug CI: eight states (~1 min). Release nightly (slow.yml): full forty-state check.
    let n = if cfg!(debug_assertions) { 8 } else { 40 };
    let states = collect_states(&db, n, true);
    for spec in SPECS {
        for (i, state) in states.iter().enumerate() {
            let legal = legal_actions(&db, state);
            if legal.is_empty() {
                continue;
            }
            let seed = 20260926u64.wrapping_add(i as u64);
            let mut a = parse_h0(spec);
            let mut b = parse_h0(spec);
            let mut rng_a = policy_rng(seed);
            let mut rng_b = policy_rng(seed);
            let _ = a.choose(&db, state, &legal, &mut rng_a);
            let _ = b.choose(&db, state, &legal, &mut rng_b);
            assert_eq!(a.stats.nodes, b.stats.nodes, "{spec} bench nodes at {i}");
            assert!(a.take_explain().is_none());
        }
    }
}
