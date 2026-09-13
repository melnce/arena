//! H0 opponent model: odepth=0 identity, antisymmetry, four-attacker Ward,
//! play-then-attack lethal, determinism.

use arena_engine::{
    acting_player, apply, legal_actions, new_game, play_game, policy_rng, Action, AnyPolicy,
    CardDb, CardId, First, GameConfig, Phase, PlayerId, Policy, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

/// Fox of Purity — 2-cost 1/3, printed Ward, no Fanfare.
const WARD: &str = "10061120";
/// Synth Vanilla — 2/2 body used as the four attackers / my board.
const VANILLA: &str = "88001110";
/// Troue, Heroic Visionary — 3-cost 2/1 Storm (Engage-Drain only).
const STORM: &str = "10461110";

const FINITE_WIN: f32 = 80.0;

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

fn is_play_id(state: &arena_engine::State, who: PlayerId, a: &Action, id: &str) -> bool {
    let want = cid(id);
    match a {
        Action::Play { hand } => state
            .player(who)
            .hand
            .get(*hand as usize)
            .is_some_and(|c| c.card == want),
        _ => false,
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

fn four_attacker_state(db: &CardDb, my_leader: i32) -> arena_engine::State {
    four_attacker_state_ward_def(db, my_leader, 15)
}

fn four_attacker_state_ward_def(db: &CardDb, my_leader: i32, ward_def: i32) -> arena_engine::State {
    let mut st = started(db, 41);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let mine = put_field(db, &mut st, PlayerId::A, VANILLA);
    set_body(&mut st, PlayerId::A, mine, 2, 2);
    // Summoning-sick so the board body is not a free face attack after Ward
    // (a combinable lure makes both specs play Ward first).
    st.field_inst_mut(PlayerId::A, mine)
        .unwrap()
        .flags
        .summoning_sick = true;
    for _ in 0..4 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    // PP-exclusive lure: 3-cost Storm (10461110) mutated to 10 attack.
    // 3 PP buys Storm *or* the 2-cost Ward, not both. Greedy (9 incoming)
    // takes the +45 face hit; the candidate (12 incoming = lethal) plays Ward.
    put_hand(db, &mut st, PlayerId::A, STORM);
    st.player_mut(PlayerId::A).hand[0].attack = 10;
    put_hand(db, &mut st, PlayerId::A, WARD);
    // 1/3 dies to the first 3-atk and the live leaf falls through the ±80
    // clamp (`finite`), so Ward and dying look identical. A 1/15 Ward keeps
    // the "I lived" value above -FINITE_WIN. The wv=300 fixture uses the
    // printed 1/3 (see `four_attacker_normal_ward_wv300_plays_ward`).
    {
        let w = st.player_mut(PlayerId::A).hand.last_mut().unwrap();
        w.defense = ward_def;
        w.max_defense = ward_def;
    }
    give_pp(&mut st, PlayerId::A, 3, 3);
    st.player_mut(PlayerId::A).leader_defense = my_leader;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert!(matches!(st.phase, Phase::Main));
    assert_eq!(acting_player(&st), PlayerId::A);
    assert_eq!(st.active, PlayerId::A);
    assert!(!st.player(PlayerId::B).deck.is_empty(), "opp deck present");
    st
}

fn check_odepth0_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:odepth=0").unwrap().spec(), "h0");
    // odepth=0 must ignore obeam (same path as main's greedy line).
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260912u64.wrapping_add(i as u64);
        let mut a = H0::default();
        let mut b = parse_h0("h0:obeam=99");
        assert_eq!(a.odepth, 0);
        assert_eq!(b.odepth, 0);
        let mut rng_a = policy_rng(seed);
        let mut rng_b = policy_rng(seed);
        let ia = a.choose(&db, state, &legal, &mut rng_a);
        let ib = b.choose(&db, state, &legal, &mut rng_b);
        assert_eq!(ia, ib, "odepth=0 vs obeam=99 at state {i}");
        assert!(ia < legal.len());
    }
}

#[test]
fn odepth0_identity_smoke() {
    check_odepth0_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn odepth0_identity_200_midgame() {
    check_odepth0_identity(200);
}

#[test]
fn spec_odepth_obeam() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:odepth=0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:obeam=3").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:odepth=5").unwrap().spec(),
        "h0:odepth=5"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:odepth=5,obeam=3").unwrap().spec(),
        "h0:odepth=5"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:odepth=5,obeam=2").unwrap().spec(),
        "h0:odepth=5,obeam=2"
    );
    let e = AnyPolicy::parse_spec("h0:odepth=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
    let e = AnyPolicy::parse_spec("h0:obeam=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
    let e = AnyPolicy::parse_spec("h0:odpth=5").unwrap_err();
    assert!(e.contains("odpth"), "{e}");
}

fn check_antisymmetry(spec: &str, states: &[arena_engine::State], db: &CardDb) {
    let h = parse_h0(spec);
    for (i, state) in states.iter().enumerate() {
        for (lhs, rhs) in [(PlayerId::A, PlayerId::B), (PlayerId::B, PlayerId::A)] {
            let a = h.evaluate(db, state, lhs);
            let b = h.evaluate(db, state, rhs);
            if a.is_infinite() || b.is_infinite() {
                assert_eq!(a, -b, "{spec} inf at state {i} {lhs:?}");
                continue;
            }
            // Within 1e-5, not bit-exact: f32 negation of the summed terms.
            assert!(
                (a + b).abs() < 1e-5,
                "{spec} state {i} {lhs:?}: {a} + {b} = {}",
                a + b
            );
        }
    }
}

#[test]
fn value_is_antisymmetric_v0_and_v1() {
    let db = load_db();
    let states = collect_states(&db, 200, false);
    assert_eq!(states.len(), 200);
    check_antisymmetry("h0", &states, &db);
    check_antisymmetry("h0:value=v1", &states, &db);
}

#[test]
fn four_attacker_ward_candidate_plays_ward() {
    let db = load_db();
    // A combinable board face-attack made both specs play Ward first (depth-6
    // search does Ward then attack). The lure is therefore a 3-cost 10-atk
    // Storm that spends the same 3 PP as the Ward line.
    let st = four_attacker_state(&db, 12);
    let mut greedy = parse_h0("h0");
    let mut cand = parse_h0("h0:odepth=5,obeam=3");
    let (gi, ga) = pick(&mut greedy, &db, &st, 7);
    let (ci, ca) = pick(&mut cand, &db, &st, 7);
    eprintln!("four-attacker greedy={ga:?} idx={gi} candidate={ca:?} idx={ci}");
    assert!(
        is_play_id(&st, PlayerId::A, &ca, WARD),
        "candidate must play Ward {WARD}, got {ca:?} idx={ci}"
    );
    assert_ne!(
        gi, ci,
        "odepth=0 must not also play the Ward (got {ga:?}); lure is 10-atk Storm at 3 PP"
    );
    assert!(
        !is_play_id(&st, PlayerId::A, &ga, WARD),
        "odepth=0 played the Ward too: {ga:?}"
    );
}

#[test]
fn four_attacker_normal_ward_wv300_plays_ward() {
    let db = load_db();
    // Printed 1/3 Ward on the same lure. Depth-6 own-turn search would
    // otherwise play the 10-atk Storm and snipe a 3/3 — that also lives at
    // 3 life and outscores the Ward chump. 3/11 keeps incoming at 12 so
    // Storm cannot cut an attacker; the only survival is the Ward
    // (alive at 3 ≈ −76, which beats dead at −300 and loses to −80).
    let mut st = four_attacker_state_ward_def(&db, 12, 3);
    for slot in 0..5u8 {
        if let Some(f) = st.field_inst_mut(PlayerId::B, slot) {
            f.defense = 11;
            f.max_defense = 11;
        }
    }
    let mut cand80 = parse_h0("h0:odepth=5,obeam=3");
    let mut cand300 = parse_h0("h0:odepth=5,obeam=3,wv=300");
    let (i80, a80) = pick(&mut cand80, &db, &st, 7);
    let (i300, a300) = pick(&mut cand300, &db, &st, 7);
    eprintln!("1/3-Ward wv=80={a80:?} idx={i80}  wv=300={a300:?} idx={i300}");
    assert!(
        is_play_id(&st, PlayerId::A, &a300, WARD),
        "wv=300 must play Ward {WARD}, got {a300:?} idx={i300}"
    );
}

#[test]
fn four_attacker_at_13_is_finite_and_legal() {
    let db = load_db();
    let st = four_attacker_state(&db, 13);
    let mut cand = parse_h0("h0:odepth=5,obeam=3");
    let (_, a) = pick(&mut cand, &db, &st, 7);
    let legal = legal_actions(&db, &st);
    assert!(legal.contains(&a), "decision must be legal: {a:?}");
    let v = cand.evaluate(&db, &st, PlayerId::A);
    assert!(
        v.is_finite(),
        "search position value must be finite, got {v}"
    );
}

fn storm_lethal_opp_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 42);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let att = put_field(db, &mut st, PlayerId::B, VANILLA);
    set_body(&mut st, PlayerId::B, att, 3, 3);
    put_hand(db, &mut st, PlayerId::B, STORM);
    give_pp(&mut st, PlayerId::B, 3, 3);
    // 3 (board) + 2 (storm) = 5. Evolve is available so the greedy 3-step
    // line can waste a ply and miss the storm attack.
    set_round(&mut st, PlayerId::B, 5);
    give_pp(&mut st, PlayerId::B, 3, 5);
    st.player_mut(PlayerId::B).ep = 1;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    st.player_mut(PlayerId::A).leader_defense = 5;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

#[test]
fn play_then_attack_opp_lethal_values() {
    let db = load_db();
    let st = storm_lethal_opp_state(&db);
    let mut greedy = parse_h0("h0");
    let mut cand = parse_h0("h0:odepth=5,obeam=3");
    let gv = greedy.opponent_value(&db, &st, PlayerId::A);
    let cv = cand.opponent_value(&db, &st, PlayerId::A);
    eprintln!("play-then-attack greedy={gv} candidate={cv}");
    assert!(
        cv <= -FINITE_WIN + 1e-3,
        "candidate must see opponent lethal, got {cv}"
    );
    assert!(gv.is_finite(), "greedy line must stay finite, got {gv}");
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
    const SEED: u64 = 20260912;
    let first = play_pair_spec(&db, spec, spec, n, SEED);
    let again = play_pair_spec(&db, spec, spec, n, SEED);
    assert_eq!(first, again, "{spec} must be deterministic");
}

#[test]
fn odepth4_determinism_smoke() {
    check_determinism("h0:depth=3,beam=2,k=1,nodes=200,odepth=2", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn odepth4_determinism_20_games() {
    check_determinism("h0:odepth=4", 20);
}

#[test]
fn reset_stats_clears_counters() {
    let db = load_db();
    let st = started(&db, 1);
    let legal = legal_actions(&db, &st);
    let mut h = H0::fast();
    let mut rng = policy_rng(1);
    let _ = h.choose(&db, &st, &legal, &mut rng);
    assert!(h.stats.decisions >= 1);
    h.reset_stats();
    assert_eq!(h.stats.decisions, 0);
    assert_eq!(h.stats.nodes, 0);
}
