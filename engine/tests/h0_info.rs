//! H0 information regime (`info`): identity at the default (`fair`),
//! per-mode determinize unit tests, Fair seed-stability and variation,
//! spec round-trip, and play determinism. `info=draws` and `info=all`
//! remain available as explicit spec keys; `all` is hard-mode sparring.

use std::collections::BTreeMap;

use arena_engine::determinize::{determinize, determinize_with, Info};
use arena_engine::{
    apply, legal_actions, new_game, play_game, policy_rng, search_key, AnyPolicy, CardDb, First,
    GameConfig, Phase, PlayerId, Policy, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

fn collect_states(db: &CardDb, n: usize, midgame_only: bool) -> Vec<arena_engine::State> {
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

fn zone_ids(items: &[arena_engine::CardInstance]) -> Vec<u32> {
    items.iter().map(|c| c.id).collect()
}

fn zone_multiset(items: &[arena_engine::CardInstance]) -> BTreeMap<(u32, u32), u32> {
    let mut m = BTreeMap::new();
    for c in items {
        *m.entry((c.card.0, c.id)).or_insert(0) += 1;
    }
    m
}

/// Mixed hidden cards on both sides so shuffle / resample have something
/// to permute. Built on `started` so public-knowledge bookkeeping is real.
fn mixed_info_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 41);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    st.player_mut(PlayerId::A).deck.clear();
    st.player_mut(PlayerId::B).deck.clear();
    put_hand(db, &mut st, PlayerId::A, "10461110");
    put_hand(db, &mut st, PlayerId::A, "10061120");
    put_deck(db, &mut st, PlayerId::A, "88001110");
    put_deck(db, &mut st, PlayerId::A, "10461110");
    put_deck(db, &mut st, PlayerId::A, "10061120");
    put_deck(db, &mut st, PlayerId::A, "88001110");
    put_hand(db, &mut st, PlayerId::B, "88001110");
    put_hand(db, &mut st, PlayerId::B, "10461110");
    put_deck(db, &mut st, PlayerId::B, "10061120");
    put_deck(db, &mut st, PlayerId::B, "88001110");
    put_deck(db, &mut st, PlayerId::B, "10461110");
    assert!(matches!(st.phase, Phase::Main));
    st
}

fn check_fair_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:info=fair").unwrap().spec(), "h0");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260919u64.wrapping_add(i as u64);
        let mut main = H0::default();
        let mut named = parse_h0("h0");
        let mut fair = parse_h0("h0:info=fair");
        let mut rng_m = policy_rng(seed);
        let mut rng_n = policy_rng(seed);
        let mut rng_d = policy_rng(seed);
        let im = main.choose(&db, state, &legal, &mut rng_m);
        let inn = named.choose(&db, state, &legal, &mut rng_n);
        let idr = fair.choose(&db, state, &legal, &mut rng_d);
        assert_eq!(im, inn, "h0 vs H0::default at state {i}");
        assert_eq!(im, idr, "h0 vs h0:info=fair at state {i}");
        assert!(im < legal.len());
    }
}

#[test]
fn fair_identity_smoke() {
    check_fair_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn fair_identity_200_midgame() {
    check_fair_identity(200);
}

#[test]
fn determinize_draws_leaves_own_deck() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let me = PlayerId::A;
    let got = determinize(&st, me, 7);
    let via = determinize_with(&st, me, 7, Info::Draws);
    assert_eq!(
        zone_ids(&got.player(me).deck),
        zone_ids(&st.player(me).deck)
    );
    assert_eq!(
        zone_ids(&got.player(me).hand),
        zone_ids(&st.player(me).hand)
    );
    assert_eq!(
        zone_ids(&got.player(me).deck),
        zone_ids(&via.player(me).deck)
    );
    assert_eq!(
        zone_ids(&got.player(me.opponent()).hand),
        zone_ids(&via.player(me.opponent()).hand)
    );
    assert_eq!(
        zone_ids(&got.player(me.opponent()).deck),
        zone_ids(&via.player(me.opponent()).deck)
    );
    assert_eq!(
        zone_multiset(&got.player(me.opponent()).hand)
            .into_iter()
            .chain(zone_multiset(&got.player(me.opponent()).deck))
            .collect::<BTreeMap<_, _>>(),
        zone_multiset(&st.player(me.opponent()).hand)
            .into_iter()
            .chain(zone_multiset(&st.player(me.opponent()).deck))
            .collect::<BTreeMap<_, _>>()
    );
}

#[test]
fn determinize_fair_keeps_own_hand_permutes_own_deck() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let me = PlayerId::A;
    let seed = 11u64;
    let fair = determinize_with(&st, me, seed, Info::Fair);
    let draws = determinize_with(&st, me, seed, Info::Draws);
    assert_eq!(
        zone_ids(&fair.player(me).hand),
        zone_ids(&st.player(me).hand),
        "Fair must not touch the perspective hand"
    );
    assert_eq!(
        zone_multiset(&fair.player(me).deck),
        zone_multiset(&st.player(me).deck),
        "Fair own deck is a permutation of the same multiset"
    );
    assert_eq!(
        zone_ids(&fair.player(me.opponent()).hand),
        zone_ids(&draws.player(me.opponent()).hand),
        "Fair opponent hand matches Draws (same seed)"
    );
    assert_eq!(
        zone_ids(&fair.player(me.opponent()).deck),
        zone_ids(&draws.player(me.opponent()).deck),
        "Fair opponent deck matches Draws (same seed)"
    );
}

#[test]
fn determinize_all_is_clone_plus_reseed() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let me = PlayerId::A;
    let seed = 13u64;
    let got = determinize_with(&st, me, seed, Info::All);
    assert_eq!(
        search_key(&got),
        search_key(&st),
        "All keeps the full state except rng"
    );
    assert_eq!(
        zone_ids(&got.player(me).hand),
        zone_ids(&st.player(me).hand)
    );
    assert_eq!(
        zone_ids(&got.player(me).deck),
        zone_ids(&st.player(me).deck)
    );
    assert_eq!(
        zone_ids(&got.player(me.opponent()).hand),
        zone_ids(&st.player(me.opponent()).hand)
    );
    assert_eq!(
        zone_ids(&got.player(me.opponent()).deck),
        zone_ids(&st.player(me.opponent()).deck)
    );
    let mut want = st.clone();
    want.rng.reseed(seed);
    assert_eq!(
        format!("{:?}", got.rng),
        format!("{:?}", want.rng),
        "All reseeds State.rng from seed"
    );
}

#[test]
fn fair_is_deterministic_given_seed() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let a = determinize_with(&st, PlayerId::A, 99, Info::Fair);
    let b = determinize_with(&st, PlayerId::A, 99, Info::Fair);
    assert_eq!(
        zone_ids(&a.player(PlayerId::A).deck),
        zone_ids(&b.player(PlayerId::A).deck)
    );
    assert_eq!(
        zone_ids(&a.player(PlayerId::B).hand),
        zone_ids(&b.player(PlayerId::B).hand)
    );
    assert_eq!(
        zone_ids(&a.player(PlayerId::B).deck),
        zone_ids(&b.player(PlayerId::B).deck)
    );
    assert_eq!(search_key(&a), search_key(&b));
}

#[test]
fn fair_varies_own_future_across_seeds() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let me = PlayerId::A;
    let first = zone_ids(&determinize_with(&st, me, 1, Info::Fair).player(me).deck);
    let differs = (2u64..=16).any(|seed| {
        zone_ids(&determinize_with(&st, me, seed, Info::Fair).player(me).deck) != first
    });
    assert!(
        differs,
        "Fair must permute own deck order on at least one seed"
    );
}

#[test]
fn spec_info_round_trips() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:info=fair").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:info=draws").unwrap().spec(),
        "h0:info=draws"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:info=all").unwrap().spec(),
        "h0:info=all"
    );
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    let again = AnyPolicy::parse_spec("h0:info=draws").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
    let e = AnyPolicy::parse_spec("h0:info=bogus").unwrap_err();
    assert!(e.contains("info"), "info=bogus → {e}");
    assert!(
        e.contains("fair") && e.contains("draws") && e.contains("all"),
        "error must list the three values: {e}"
    );
}

#[test]
fn all_builds_one_root() {
    let db = load_db();
    let st = started(&db, 1);
    let legal = legal_actions(&db, &st);
    assert!(legal.len() > 1, "need a searched decision");
    let mut all = parse_h0("h0:info=all");
    assert_eq!(all.determinizations, 4);
    let mut rng = policy_rng(1);
    all.choose(&db, &st, &legal, &mut rng);
    assert_eq!(all.stats.roots, 1, "info=all must build one root");
    let mut draws = parse_h0("h0:info=draws");
    let mut rng = policy_rng(1);
    draws.choose(&db, &st, &legal, &mut rng);
    assert_eq!(draws.stats.roots, 4, "info=draws still builds k roots");
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
    const SEED: u64 = 20260919;
    let first = play_pair_spec(&db, spec, spec, n, SEED);
    let again = play_pair_spec(&db, spec, spec, n, SEED);
    assert_eq!(first, again, "{spec} must be deterministic");
}

#[test]
fn info_fair_determinism_smoke() {
    check_determinism("h0:info=fair,depth=2,nodes=200", 2);
}

#[test]
fn info_all_determinism_smoke() {
    check_determinism("h0:info=all,depth=2,nodes=200", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn info_fair_determinism_20_games() {
    check_determinism("h0:info=fair", 20);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn info_all_determinism_20_games() {
    check_determinism("h0:info=all", 20);
}
