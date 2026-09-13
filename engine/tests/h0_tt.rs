//! H0 per-decision transposition table: tt=0 identity, unbounded-cap
//! agreement, hits at the real cap, determinism.

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

fn check_tt0_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:tt=0").unwrap().spec(), "h0");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260913u64.wrapping_add(i as u64);
        let mut main = H0::default();
        let mut named = parse_h0("h0");
        let mut off = parse_h0("h0:tt=0");
        let mut rng_m = policy_rng(seed);
        let mut rng_n = policy_rng(seed);
        let mut rng_o = policy_rng(seed);
        let im = main.choose(&db, state, &legal, &mut rng_m);
        let inn = named.choose(&db, state, &legal, &mut rng_n);
        let io = off.choose(&db, state, &legal, &mut rng_o);
        assert_eq!(im, inn, "h0 vs H0::default at state {i}");
        assert_eq!(im, io, "h0 vs h0:tt=0 at state {i}");
        assert!(im < legal.len());
        assert_eq!(off.stats.tt_hits, 0);
        assert_eq!(off.stats.tt_stores, 0);
    }
}

#[test]
fn tt0_identity_smoke() {
    check_tt0_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn tt0_identity_200_midgame() {
    check_tt0_identity(200);
}

fn check_unbounded_identity(n: usize, spec_off: &str, spec_on: &str) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    let mut agree = 0u32;
    let mut differ: Vec<(usize, Action, Action)> = Vec::new();
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260913u64.wrapping_add(i as u64);
        let mut off = parse_h0(spec_off);
        let mut on = parse_h0(spec_on);
        let (ia, aa) = pick(&mut off, &db, state, seed);
        let (ib, ab) = pick(&mut on, &db, state, seed);
        if ia == ib {
            agree += 1;
        } else {
            eprintln!("unbounded differ state {i}: tt=0 {aa:?} idx={ia}  tt=1 {ab:?} idx={ib}");
            differ.push((i, aa, ab));
        }
    }
    let compared = agree + differ.len() as u32;
    assert!(compared > 0, "no comparable states");
    let rate = agree as f64 / compared as f64;
    eprintln!(
        "unbounded identity: {agree}/{compared} = {rate:.4} ({} differ)",
        differ.len()
    );
    if !differ.is_empty() {
        for (i, a, b) in &differ {
            eprintln!("  state {i}: {a:?} vs {b:?}");
        }
        assert!(
            rate >= 0.95,
            "unbounded tt=1 agreement {rate:.4} < 0.95; differ {:?}",
            differ
                .iter()
                .map(|(i, a, b)| (*i, format!("{a:?}"), format!("{b:?}")))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn unbounded_identity_smoke() {
    // Debug sibling: a tree small enough that `nodes=1e6` never binds.
    check_unbounded_identity(
        20,
        "h0:depth=3,beam=2,k=1,nodes=1000000",
        "h0:depth=3,beam=2,k=1,nodes=1000000,tt=1",
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn unbounded_identity_200_midgame() {
    check_unbounded_identity(200, "h0:nodes=1000000", "h0:nodes=1000000,tt=1");
}

fn check_table_does_work(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    let mut off = parse_h0("h0");
    let mut on = parse_h0("h0:tt=1");
    let mut hit_decisions = 0u32;
    let mut searched = 0u32;
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260913u64.wrapping_add(i as u64);
        let hits_before = on.stats.tt_hits;
        let _ = pick(&mut off, &db, state, seed);
        let _ = pick(&mut on, &db, state, seed);
        searched += 1;
        if on.stats.tt_hits > hits_before {
            hit_decisions += 1;
        }
    }
    assert!(searched > 0);
    let off_rate = off.stats.cap_hits as f64 / off.stats.decisions as f64;
    let on_rate = on.stats.cap_hits as f64 / on.stats.decisions as f64;
    eprintln!(
        "table work n={n}: hit_decisions={hit_decisions}/{searched} \
         cap_hit_rate tt=0={off_rate:.4} tt=1={on_rate:.4} \
         tt_hits={} tt_stores={}",
        on.stats.tt_hits, on.stats.tt_stores
    );
    if n >= 200 {
        assert!(
            hit_decisions * 2 >= searched,
            "tt_hits > 0 on {hit_decisions}/{searched} decisions (want ≥ half)"
        );
        assert!(
            on_rate < off_rate,
            "cap_hit_rate tt=1 ({on_rate:.4}) should be < tt=0 ({off_rate:.4})"
        );
    }
}

#[test]
fn table_does_work_smoke() {
    check_table_does_work(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn table_does_work_200_midgame() {
    check_table_does_work(200);
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
    const SEED: u64 = 20260913;
    let first = play_pair_spec(&db, spec, spec, n, SEED);
    let again = play_pair_spec(&db, spec, spec, n, SEED);
    assert_eq!(first, again, "{spec} must be deterministic");
}

#[test]
fn tt1_determinism_smoke() {
    check_determinism("h0:tt=1,depth=2,nodes=200", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn tt1_determinism_20_games() {
    check_determinism("h0:tt=1", 20);
}

#[test]
fn spec_tt() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:tt=0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:tt=1").unwrap().spec(), "h0:tt=1");
    assert_eq!(
        AnyPolicy::parse_spec("h0:tt=1,odepth=5").unwrap().spec(),
        "h0:odepth=5,tt=1"
    );
    let e = AnyPolicy::parse_spec("h0:tt=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
    let e = AnyPolicy::parse_spec("h0:tt=2").unwrap_err();
    assert!(e.contains("2"), "{e}");
    let e = AnyPolicy::parse_spec("h0:t=1").unwrap_err();
    assert!(e.contains("t"), "{e}");
}
