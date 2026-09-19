//! H0 opponent-lethal evolve glance (`oevo`): identity at the default,
//! spec round-trip, the play-then-evolve Storm fixture, no false
//! positive, and determinism. Default behaviour is bit-identical; the
//! yardstick decides any flip.

use arena_engine::{
    apply, legal_actions, new_game, play_game, policy_rng, Action, AnyPolicy, CardDb, CardId,
    First, GameConfig, Phase, PlayerId, Policy, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

/// Synth Vanilla — 2/2 body used as the on-board attacker
/// (`four_attacker_state` in `h0_olethal.rs` uses the same body).
const VANILLA: &str = "88001110";
/// Troue, Heroic Visionary — 3-cost 2/1 Storm (Engage-Drain only).
const STORM: &str = "10461110";

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

fn set_body(state: &mut arena_engine::State, who: PlayerId, slot: u8, atk: i32, def: i32) {
    let f = state.field_inst_mut(who, slot).unwrap();
    f.attack = atk;
    f.defense = def;
    f.max_defense = def;
    f.flags.summoning_sick = false;
    f.flags.attacks_left = 1;
}

/// Adapted from `h0_olethal::storm_lethal_opp_state`: Storm in hand, one
/// on-board attacker, evolve available. Leader defence is
/// `storm_attack + 2 + board_attack + extra` so the line is lethal only
/// with the evolve (`extra = 0`) or one off (`extra = 1`).
fn storm_lethal_opp_state(db: &CardDb, extra_def: i32) -> arena_engine::State {
    let mut st = started(db, 42);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let att = put_field(db, &mut st, PlayerId::B, VANILLA);
    set_body(&mut st, PlayerId::B, att, 3, 3);
    put_hand(db, &mut st, PlayerId::B, STORM);
    give_pp(&mut st, PlayerId::B, 3, 3);
    set_round(&mut st, PlayerId::B, 5);
    give_pp(&mut st, PlayerId::B, 3, 5);
    st.player_mut(PlayerId::B).ep = 1;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    let storm_attack = st
        .player(PlayerId::B)
        .hand
        .iter()
        .find(|c| c.card == cid(STORM))
        .map(|c| c.attack)
        .expect("Storm in hand");
    let board_attack: i32 = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .map(|c| c.attack)
        .sum();
    st.player_mut(PlayerId::A).leader_defense = storm_attack + 2 + board_attack + extra_def;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

/// Adapted from `h0_olethal::no_lethal_opp_state`: evolve is legal, but
/// two 3/3s (one of them +2) cannot reach 20 defence.
fn no_lethal_opp_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 43);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    for _ in 0..2 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    set_round(&mut st, PlayerId::B, 5);
    st.player_mut(PlayerId::B).ep = 1;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    st.player_mut(PlayerId::A).leader_defense = 20;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    assert!(st.winner.is_none());
    let legal = legal_actions(db, &st);
    assert!(
        legal.iter().any(|a| matches!(a, Action::Evolve { .. })),
        "evolve must be legal on the no-lethal fixture"
    );
    st
}

fn check_oevo0_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:oevo=0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260919u64.wrapping_add(i as u64);
        let mut main = H0::default();
        let mut named = parse_h0("h0");
        let mut off = parse_h0("h0:oevo=0");
        let mut rng_m = policy_rng(seed);
        let mut rng_n = policy_rng(seed);
        let mut rng_o = policy_rng(seed);
        let im = main.choose(&db, state, &legal, &mut rng_m);
        let inn = named.choose(&db, state, &legal, &mut rng_n);
        let io = off.choose(&db, state, &legal, &mut rng_o);
        assert_eq!(im, inn, "h0 vs H0::default at state {i}");
        assert_eq!(im, io, "h0 vs h0:oevo=0 at state {i}");
        assert!(im < legal.len());
    }
}

#[test]
fn oevo0_identity_smoke() {
    check_oevo0_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn oevo0_identity_200_midgame() {
    check_oevo0_identity(200);
}

#[test]
fn spec_oevo() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:oevo=0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:oevo=1").unwrap().spec(),
        "h0:oevo=1"
    );
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    let again = AnyPolicy::parse_spec("h0:oevo=1").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
    for bad in ["h0:oevo=2", "h0:oevo=x"] {
        let e = AnyPolicy::parse_spec(bad).unwrap_err();
        assert!(e.contains("oevo"), "{bad} → {e}");
    }
}

fn is_neg_wv(v: f32, wv: f32) -> bool {
    (v + wv).abs() < 1e-3
}

/// Play Storm → evolve it → attack with it and the on-board body.
/// Lethal only with the +2; `oevo=0` is today's miss.
#[test]
fn play_then_evolve_sees_storm() {
    let db = load_db();
    const WV: f32 = 300.0;
    let lethal = storm_lethal_opp_state(&db, 0);
    let miss = storm_lethal_opp_state(&db, 1);
    let mut off = parse_h0("h0:olethal=1,oevo=0,wv=300");
    let mut on = parse_h0("h0:olethal=1,oevo=1,wv=300");
    let ov = off.opponent_value(&db, &lethal, PlayerId::A);
    let nv = on.opponent_value(&db, &lethal, PlayerId::A);
    eprintln!(
        "evo-lethal oevo=0={ov} oevo=1={nv} checks0={} found0={} evo0={} \
         checks1={} found1={} evo1={} nodes1={}",
        off.stats.opp_lethal_checks,
        off.stats.opp_lethal_found,
        off.stats.opp_lethal_evo_found,
        on.stats.opp_lethal_checks,
        on.stats.opp_lethal_found,
        on.stats.opp_lethal_evo_found,
        on.stats.opp_lethal_nodes
    );
    assert!(
        !is_neg_wv(ov, WV),
        "h0:olethal=1,oevo=0 must miss the evolve lethal, got {ov}"
    );
    assert!(
        is_neg_wv(nv, WV),
        "h0:olethal=1,oevo=1 must see play-then-evolve lethal as -{WV}, got {nv}"
    );
    assert_eq!(on.stats.opp_lethal_found, 1);
    assert_eq!(on.stats.opp_lethal_evo_found, 1);
    assert_eq!(off.stats.opp_lethal_found, 0);
    assert_eq!(off.stats.opp_lethal_evo_found, 0);

    let mut off_m = parse_h0("h0:olethal=1,oevo=0,wv=300");
    let mut on_m = parse_h0("h0:olethal=1,oevo=1,wv=300");
    let om = off_m.opponent_value(&db, &miss, PlayerId::A);
    let nm = on_m.opponent_value(&db, &miss, PlayerId::A);
    eprintln!("off-by-one oevo=0={om} oevo=1={nm}");
    assert!(
        !is_neg_wv(om, WV) && !is_neg_wv(nm, WV),
        "one more leader defence must miss for both: oevo=0={om} oevo=1={nm}"
    );
}

#[test]
fn no_false_positive_evolve_available() {
    let db = load_db();
    const WV: f32 = 80.0;
    let st = no_lethal_opp_state(&db);
    let mut off = parse_h0("h0:olethal=1,oevo=0");
    let mut on = parse_h0("h0:olethal=1,oevo=1");
    let a = off.opponent_value(&db, &st, PlayerId::A);
    let b = on.opponent_value(&db, &st, PlayerId::A);
    eprintln!(
        "no-lethal oevo=0={a} oevo=1={b} checks1={} found1={} evo1={}",
        on.stats.opp_lethal_checks, on.stats.opp_lethal_found, on.stats.opp_lethal_evo_found
    );
    assert!(
        (a - b).abs() < 1e-5,
        "oevo must not change the greedy value: {a} vs {b}"
    );
    assert!(!is_neg_wv(a, WV) && !is_neg_wv(b, WV));
    assert_eq!(on.stats.opp_lethal_found, 0);
    assert_eq!(on.stats.opp_lethal_evo_found, 0);
    assert_eq!(off.stats.opp_lethal_found, 0);
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
fn oevo_determinism_smoke() {
    check_determinism("h0:oevo=1,depth=2,nodes=200", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn oevo_determinism_20_games() {
    check_determinism("h0:oevo=1", 20);
}
