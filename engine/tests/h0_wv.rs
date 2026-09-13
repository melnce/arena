//! H0 root-level terminal stand-in (`wv`): identity at 80, dead-below-alive
//! at 300, spec round-trip, determinism.

use arena_engine::{
    apply, legal_actions, new_game, play_game, policy_rng, AnyPolicy, CardDb, CardId, First,
    GameConfig, Phase, PlayerId, Policy, H0, MAX_ACTIONS, MAX_TURNS,
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

fn check_wv80_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:wv=80").unwrap().spec(), "h0");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260913u64.wrapping_add(i as u64);
        let mut main = H0::default();
        let mut named = parse_h0("h0");
        let mut eighty = parse_h0("h0:wv=80");
        let mut rng_m = policy_rng(seed);
        let mut rng_n = policy_rng(seed);
        let mut rng_e = policy_rng(seed);
        let im = main.choose(&db, state, &legal, &mut rng_m);
        let inn = named.choose(&db, state, &legal, &mut rng_n);
        let ie = eighty.choose(&db, state, &legal, &mut rng_e);
        assert_eq!(im, inn, "h0 vs H0::default at state {i}");
        assert_eq!(im, ie, "h0 vs h0:wv=80 at state {i}");
        assert!(im < legal.len());
    }
}

#[test]
fn wv80_identity_smoke() {
    check_wv80_identity(20);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn wv80_identity_200_midgame() {
    check_wv80_identity(200);
}

#[test]
fn spec_wv() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:wv=80").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:wv=300").unwrap().spec(),
        "h0:wv=300"
    );
    let again = AnyPolicy::parse_spec("h0:wv=300").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
    let e = AnyPolicy::parse_spec("h0:wv=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

fn dead_opp_won(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 42);
    st.winner = Some(PlayerId::B);
    st.phase = Phase::Terminal;
    st
}

fn live_empty_opp_turn(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 42);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    st.player_mut(PlayerId::A).leader_defense = 1;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    assert!(st.winner.is_none());
    st
}

#[test]
fn dead_is_below_alive() {
    let db = load_db();
    let dead = dead_opp_won(&db);
    let live = live_empty_opp_turn(&db);
    let mut h80 = parse_h0("h0:wv=80");
    let mut h300 = parse_h0("h0:wv=300");
    let dead80 = h80.opponent_value(&db, &dead, PlayerId::A);
    let live80 = h80.opponent_value(&db, &live, PlayerId::A);
    let leaf = h80.evaluate(&db, &live, PlayerId::A);
    eprintln!(
        "wv=80 dead={dead80} live_opp={live80} live_leaf={leaf} \
         (want live ≈ -85.5 = 4.5×(1-20) plus small terms)"
    );
    assert!(
        (dead80 + 80.0).abs() < 1e-3,
        "wv=80 dead should be -80, got {dead80}"
    );
    assert!(
        (leaf + 85.5).abs() < 5.0,
        "live leaf should be ≈ -85.5, got {leaf}"
    );
    assert!(
        dead80 > live80,
        "wv=80 inversion: dead {dead80} should score above live {live80}"
    );
    let dead300 = h300.opponent_value(&db, &dead, PlayerId::A);
    let live300 = h300.opponent_value(&db, &live, PlayerId::A);
    eprintln!("wv=300 dead={dead300} live_opp={live300} live_leaf={leaf}");
    assert!(
        (dead300 + 300.0).abs() < 1e-3,
        "wv=300 dead should be -300, got {dead300}"
    );
    assert!(
        (live300 - live80).abs() < 1e-3,
        "live value must not depend on wv: {live300} vs {live80}"
    );
    assert!(
        dead300 < live300,
        "wv=300: dead {dead300} must be strictly below live {live300}"
    );
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
fn wv300_determinism_smoke() {
    check_determinism("h0:wv=300,depth=2,nodes=200", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn wv300_determinism_20_games() {
    check_determinism("h0:wv=300", 20);
}
