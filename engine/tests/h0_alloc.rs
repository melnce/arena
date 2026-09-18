//! H0 budget allocation: alloc=fair identity, unbounded-cap agreement,
//! fair-share mechanism, determinism. `alloc=root` stays reachable.

use arena_engine::{
    apply, legal_actions, new_game, play_game, policy_rng, Action, AnyPolicy, CardDb, CardId,
    First, GameConfig, Phase, PlayerId, Policy, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

fn collect_states(db: &CardDb, n: usize, midgame_only: bool) -> Vec<arena_engine::State> {
    let dir = repo_root().join("oracle/decks");
    let mut decks: Vec<Vec<CardId>> = std::fs::read_dir(&dir)
        .expect("oracle/decks")
        .filter_map(|e| {
            let p = e.ok()?.path();
            (p.extension()? == "json").then_some(p)
        })
        .map(load_deck_file)
        .filter(|d| deck_ready(db, d) && d.len() == 40)
        .collect();
    decks.sort_by_key(|d| d.iter().map(|c| c.0).collect::<Vec<_>>());
    assert!(
        !decks.is_empty(),
        "need at least one oracle deck of 40 cards"
    );
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
            let take = !midgame_only || (state.turn >= 3 && matches!(state.phase, Phase::Main));
            if take {
                out.push(state.clone());
                if out.len() >= n {
                    break;
                }
            }
            if state.turn > MAX_TURNS || nact >= MAX_ACTIONS {
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

fn pick(h: &mut H0, db: &CardDb, state: &arena_engine::State, seed: u64) -> (usize, Action) {
    let legal = legal_actions(db, state);
    assert!(!legal.is_empty(), "no legal actions");
    let mut rng = policy_rng(seed);
    let i = h.choose(db, state, &legal, &mut rng);
    assert!(i < legal.len(), "illegal index {i}");
    (i, legal[i].clone())
}

fn check_fair_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, false);
    assert_eq!(states.len(), n, "could not reach {n} states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:alloc=fair").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:alloc=root").unwrap().spec(),
        "h0:alloc=root"
    );
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    let e = AnyPolicy::parse_spec("h0:alloc=x").unwrap_err();
    assert!(e.contains("alloc"), "{e}");
    assert!(e.contains("x"), "{e}");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260917u64.wrapping_add(i as u64);
        let mut main = H0::default();
        let mut named = parse_h0("h0");
        let mut fair = parse_h0("h0:alloc=fair");
        let mut rng_m = policy_rng(seed);
        let mut rng_n = policy_rng(seed);
        let mut rng_f = policy_rng(seed);
        let im = main.choose(&db, state, &legal, &mut rng_m);
        let inn = named.choose(&db, state, &legal, &mut rng_n);
        let iff = fair.choose(&db, state, &legal, &mut rng_f);
        assert_eq!(im, inn, "h0 vs H0::default at state {i}");
        assert_eq!(im, iff, "h0 vs h0:alloc=fair at state {i}");
        assert!(im < legal.len());
    }
}

#[test]
fn alloc_fair_identity_smoke() {
    check_fair_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn alloc_fair_identity_200() {
    check_fair_identity(200);
}

/// `alloc=root` is still a reachable, deterministic policy and still
/// differs from the default on at least one of the identity states.
fn check_root_reachable(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, false);
    assert_eq!(states.len(), n, "could not reach {n} states");
    assert_eq!(
        AnyPolicy::parse_spec("h0:alloc=root").unwrap().spec(),
        "h0:alloc=root"
    );
    let mut differ = 0u32;
    let mut compared = 0u32;
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260917u64.wrapping_add(i as u64);
        let mut def = H0::default();
        let mut root = parse_h0("h0:alloc=root");
        let mut root_again = parse_h0("h0:alloc=root");
        let mut rng_d = policy_rng(seed);
        let mut rng_r = policy_rng(seed);
        let mut rng_r2 = policy_rng(seed);
        let id = def.choose(&db, state, &legal, &mut rng_d);
        let ir = root.choose(&db, state, &legal, &mut rng_r);
        let ir2 = root_again.choose(&db, state, &legal, &mut rng_r2);
        assert_eq!(ir, ir2, "h0:alloc=root must be deterministic at state {i}");
        assert!(ir < legal.len());
        compared += 1;
        if id != ir {
            differ += 1;
        }
    }
    assert!(compared > 0, "no comparable states");
    if n >= 200 {
        assert!(
            differ >= 1,
            "h0:alloc=root must differ from the default on ≥ 1 of {compared} states"
        );
    }
}

#[test]
fn alloc_root_reachable_smoke() {
    check_root_reachable(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn alloc_root_reachable_200() {
    check_root_reachable(200);
}

fn check_unbounded_identity(n: usize, spec_root: &str, spec_fair: &str) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260917u64.wrapping_add(i as u64);
        let mut root = parse_h0(spec_root);
        let mut fair = parse_h0(spec_fair);
        let (ia, aa) = pick(&mut root, &db, state, seed);
        let (ib, ab) = pick(&mut fair, &db, state, seed);
        assert_eq!(
            ia, ib,
            "unbounded alloc differ state {i}: root {aa:?} idx={ia}  fair {ab:?} idx={ib}"
        );
        assert_eq!(
            root.stats.nodes, fair.stats.nodes,
            "unbounded nodes differ state {i}: root={} fair={}",
            root.stats.nodes, fair.stats.nodes
        );
    }
}

#[test]
fn unbounded_alloc_identity_smoke() {
    check_unbounded_identity(
        8,
        "h0:depth=3,beam=2,k=1,nodes=1000000,alloc=root",
        "h0:depth=3,beam=2,k=1,nodes=1000000,alloc=fair",
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn unbounded_alloc_identity_200_midgame() {
    check_unbounded_identity(
        200,
        "h0:alloc=root,nodes=4000000000",
        "h0:alloc=fair,nodes=4000000000",
    );
}

fn check_fair_mechanism(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    let mut root = parse_h0("h0:alloc=root,nodes=2000");
    let mut fair = parse_h0("h0:alloc=fair,nodes=2000");
    let mut max_pairs: u32 = 0;
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        max_pairs = max_pairs.max(4 * legal.len() as u32);
        let seed = 20260917u64.wrapping_add(i as u64);
        let _ = pick(&mut root, &db, state, seed);
        let _ = pick(&mut fair, &db, state, seed);
    }
    assert!(root.stats.decisions > 0);
    assert!(fair.stats.decisions > 0);
    eprintln!(
        "alloc mechanism n={n}: root pairs_skipped={} nodes={} decisions={} \
         fair pairs_skipped={} nodes={} decisions={} max_legal_pairs={max_pairs}",
        root.stats.pairs_skipped,
        root.stats.nodes,
        root.stats.decisions,
        fair.stats.pairs_skipped,
        fair.stats.nodes,
        fair.stats.decisions
    );
    // The flip's whole point: default (`fair`) skips ~0 pairs; `alloc=root`
    // skips thousands. 200 mid-game, this box: root pairs_skipped=3182, fair=0.
    if n >= 200 {
        assert!(
            root.stats.pairs_skipped > 0,
            "root must skip pairs at 2000 nodes (got 0)"
        );
        const MIN_SHARE: u32 = 24;
        let floor_binds = MIN_SHARE.saturating_mul(max_pairs) > 2000;
        if fair.stats.pairs_skipped != 0 {
            assert!(
                floor_binds,
                "fair pairs_skipped={} but MIN_SHARE×pairs ({MIN_SHARE}×{max_pairs}) ≤ cap",
                fair.stats.pairs_skipped
            );
            eprintln!(
                "fair pairs_skipped={} because MIN_SHARE×pairs ({MIN_SHARE}×{max_pairs}) > 2000",
                fair.stats.pairs_skipped
            );
        }
        let cap = 2000u64 * root.stats.decisions;
        assert!(
            root.stats.nodes <= cap,
            "root nodes {} > 2000 × {}",
            root.stats.nodes,
            root.stats.decisions
        );
        let cap_f = 2000u64 * fair.stats.decisions;
        assert!(
            fair.stats.nodes <= cap_f,
            "fair nodes {} > 2000 × {}",
            fair.stats.nodes,
            fair.stats.decisions
        );
        let lo = (root.stats.nodes as f64) * 0.95;
        let hi = (root.stats.nodes as f64) * 1.05;
        let fnodes = fair.stats.nodes as f64;
        assert!(
            fnodes >= lo && fnodes <= hi,
            "fair nodes {fnodes} not within 5% of root {}",
            root.stats.nodes
        );
    }
}

#[test]
fn fair_mechanism_smoke() {
    check_fair_mechanism(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn fair_mechanism_200_midgame() {
    check_fair_mechanism(200);
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

fn check_determinism(spec: &str, n: u32) {
    let db = load_db();
    const SEED: u64 = 20260917;
    let first = play_pair_spec(&db, spec, spec, n, SEED);
    let again = play_pair_spec(&db, spec, spec, n, SEED);
    assert_eq!(first, again, "{spec} must be deterministic");
}

#[test]
fn alloc_fair_determinism_smoke() {
    check_determinism("h0:alloc=fair,depth=2,nodes=200", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn alloc_fair_determinism_20_games() {
    check_determinism("h0:alloc=fair", 20);
}

#[test]
fn spec_alloc() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:alloc=fair").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:alloc=root").unwrap().spec(),
        "h0:alloc=root"
    );
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    let again = AnyPolicy::parse_spec("h0:alloc=root").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
    assert_eq!(again.spec(), "h0:alloc=root");
    let e = AnyPolicy::parse_spec("h0:alloc=x").unwrap_err();
    assert!(e.contains("alloc"), "{e}");
    let e = AnyPolicy::parse_spec("h0:alloc=2").unwrap_err();
    assert!(e.contains("alloc"), "{e}");
}
