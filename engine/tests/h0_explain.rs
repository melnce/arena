//! H0 explain sink: identity, tie set, principal-variation replay.

use arena_engine::policy::Info;
use arena_engine::{
    apply, determinize::determinize_with, from_neutral, legal_actions, new_game, policy_rng,
    AnyPolicy, CardDb, ChoosePath, First, Phase, Policy, PvEnd, H0,
};

mod common;
use common::*;

const SPECS: &[&str] = &[
    "h0",
    "h0:value=v0",
    "h0:nodes=6000",
    "h0:k=1",
    "h0:info=all",
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

fn check_pv_replay(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    let spec = "h0:k=1,info=all,value=v0,olethal=0,osteps=0,odepth=0";
    let h0_cfg = parse_h0(spec);
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
        if rec.path != ChoosePath::Search {
            continue;
        }
        let me = arena_engine::acting_player(state);
        let mut rng_root = policy_rng(seed);
        let root = determinize_with(state, me, rng_root.next_u64(), Info::All);
        for cand in &rec.candidates {
            for world in &cand.worlds {
                if world.skipped || world.pv.is_empty() {
                    continue;
                }
                let Some(end) = world.end else {
                    continue;
                };
                if end != PvEnd::Depth && end != PvEnd::OppReply {
                    continue;
                }
                let Some(leaf) = world.leaf.as_ref() else {
                    continue;
                };
                let mut s = root.clone();
                for neu in &world.pv {
                    let a = from_neutral(&s, neu).expect("pv action maps");
                    apply(&db, &mut s, a).expect("pv replay legal");
                }
                let v = h0_cfg.evaluate(&db, &s, me);
                assert!(
                    (v - leaf.value).abs() < 1e-3,
                    "pv replay value mismatch at state {i}: got {v} want {want} end={end:?}",
                    want = leaf.value
                );
            }
        }
    }
}

#[test]
fn pv_replay_smoke() {
    check_pv_replay(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn pv_replay_200_midgame() {
    check_pv_replay(200);
}

#[test]
fn bench_nodes_unchanged_without_sink() {
    let db = load_db();
    let states = collect_states(&db, 40, true);
    for spec in SPECS {
        for (i, state) in states.iter().enumerate() {
            let legal = legal_actions(&db, state);
            if legal.is_empty() {
                continue;
            }
            let seed = 20260925u64.wrapping_add(i as u64);
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

/// Builds a mid-game position from `meta-rune-test-subject` with Sephie host
/// fuse legal at usable PP &lt; 2, runs the motivating explain spec, and
/// writes JSON to `tmp_motivating_explain.json` at the repo root.
#[test]
#[ignore]
fn motivating_sephie_fuse_explain_fixture() {
    use std::io::Write;

    use arena_engine::{
        apply, legal_actions, new_game, to_neutral, Action, GameConfig, PlayerId, Policy,
    };

    let db = load_db();
    let deck_a = load_deck_file(repo_root().join("oracle/decks/meta-rune-test-subject.json"));
    let deck_b = load_deck_file(repo_root().join("oracle/decks/rune-pool.json"));
    let mut state = new_game(
        &db,
        GameConfig {
            seed: 10934110,
            deck_a,
            deck_b,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    apply(
        &db,
        &mut state,
        Action::MulliganConfirm { swap: [false; 4] },
    )
    .unwrap();
    apply(
        &db,
        &mut state,
        Action::MulliganConfirm { swap: [false; 4] },
    )
    .unwrap();
    let me = PlayerId::A;
    give_pp(&mut state, me, 1, 6);
    clear_hand(&mut state, me);
    let host = put_hand(&db, &mut state, me, "10934110");
    let _partner = put_hand(&db, &mut state, me, "10931110");
    let legal = legal_actions(&db, &state);
    assert!(
        legal
            .iter()
            .any(|a| matches!(a, Action::Fuse { host: h } if *h == host)),
        "Sephie fuse must be legal: {legal:?}"
    );
    let spec = "h0:value=v0,osteps=0,olethal=0,nodes=200000";
    let mut h0 = parse_h0(spec);
    h0.arm_explain();
    let mut rng = policy_rng(10934110);
    let idx = h0.choose(&db, &state, &legal, &mut rng);
    let explain = h0.take_explain().expect("explain");
    let chosen = to_neutral(&state, &legal[idx]);
    let value = h0.last_value();
    let out = serde_json::json!({
        "fixture": "meta-rune-test-subject vs rune-pool",
        "usable_pp": 1,
        "spec": spec,
        "chosen": chosen,
        "value": value,
        "explain": explain,
    });
    let path = repo_root().join("tmp_motivating_explain.json");
    let mut f = std::fs::File::create(&path).expect("create");
    write!(f, "{}", serde_json::to_string_pretty(&out).unwrap()).expect("write");
    eprintln!("wrote {}", path.display());
}
