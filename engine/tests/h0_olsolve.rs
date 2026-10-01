//! H0 opponent-lethal solver (`olsolve`): identity at default off,
//! ward-first and combination fixtures the glance misses, node
//! accounting, and determinism.

use arena_engine::policy::ChoosePath;
use arena_engine::{
    apply, forced_lethal, legal_actions, new_game, play_game, policy_rng, Action, AnyPolicy,
    CardDb, First, GameConfig, LethalVerdict, Phase, PlayerId, Policy, PvEnd, H0, MAX_ACTIONS,
    MAX_TURNS,
};

mod common;
use common::*;

const VANILLA: &str = "88001110";
const STORM: &str = "10461110";
/// Fox of Purity — printed Ward.
const WARD: &str = "10061120";

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

fn is_neg_wv(v: f32, wv: f32) -> bool {
    (v + wv).abs() < 1e-3
}

/// Opponent turn: two 3/3s kill a 6-def Ward, then two more face for
/// lethal — the glance never tries attack-into-Ward first.
fn ward_first_opp_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 81);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let ward = put_field(db, &mut st, PlayerId::A, WARD);
    set_body(&mut st, PlayerId::A, ward, 1, 6);
    for _ in 0..4 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    st.player_mut(PlayerId::A).leader_defense = 6;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    assert!(st.winner.is_none());
    st
}

/// Bot turn: can trade into an attacker or `EndTurn`. After `EndTurn` the
/// opponent has the ward-first kill above.
fn ward_first_bot_turn_state(db: &CardDb) -> (arena_engine::State, Action, Action) {
    let mut st = started(db, 82);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let ward = put_field(db, &mut st, PlayerId::A, WARD);
    set_body(&mut st, PlayerId::A, ward, 1, 6);
    for _ in 0..4 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    let mine = put_field(db, &mut st, PlayerId::A, VANILLA);
    set_body(&mut st, PlayerId::A, mine, 2, 2);
    st.player_mut(PlayerId::A).leader_defense = 6;
    st.player_mut(PlayerId::B).leader_defense = 20;
    give_pp(&mut st, PlayerId::A, 0, 3);
    assert_eq!(st.active, PlayerId::A);
    let legal = legal_actions(db, &st);
    let trade = legal
        .iter()
        .find(|a| matches!(a, Action::Attack { .. }))
        .cloned()
        .expect("trade attack");
    let end = legal
        .iter()
        .find(|a| matches!(a, Action::EndTurn))
        .cloned()
        .expect("EndTurn");
    (st, trade, end)
}

/// Two Storms, empty board — two `Play`s deep; copied from `lethal.rs`.
fn two_storm_lethal_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 83);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    put_hand(db, &mut st, PlayerId::B, STORM);
    put_hand(db, &mut st, PlayerId::B, STORM);
    set_round(&mut st, PlayerId::B, 6);
    give_pp(&mut st, PlayerId::B, 6, 6);
    st.player_mut(PlayerId::B).ep = 0;
    st.player_mut(PlayerId::B).sep = 0;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    let storm_attack = st
        .player(PlayerId::B)
        .hand
        .iter()
        .find(|c| c.card == cid(STORM))
        .map(|c| c.attack)
        .expect("Storm in hand");
    st.player_mut(PlayerId::A).leader_defense = storm_attack * 2;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

fn glance_misses(db: &CardDb, state: &arena_engine::State, wv: f32) -> bool {
    let mut h = parse_h0_v1(&format!("h0:olethal=1,oevo=1,wv={wv}"));
    let v = h.opponent_value(db, state, PlayerId::A);
    assert_eq!(h.stats.opp_lethal_checks, 1);
    assert_eq!(h.stats.opp_lethal_found, 0);
    !is_neg_wv(v, wv)
}

fn score_candidate_world(
    spec: &str,
    db: &CardDb,
    st: &arena_engine::State,
    pick: &Action,
) -> (f32, PvEnd) {
    let legal = legal_actions(db, st);
    let idx = legal.iter().position(|a| a == pick).expect("action legal");
    let mut h0 = parse_h0(spec);
    h0.arm_explain();
    let mut rng = policy_rng(71);
    let _ = h0.choose(db, st, &legal, &mut rng);
    let rec = h0.take_explain().expect("explain");
    assert_eq!(rec.path, ChoosePath::Search);
    let cand = rec
        .candidates
        .iter()
        .find(|c| c.legal_index == idx)
        .expect("candidate");
    let world = &cand.worlds[0];
    assert!(!world.skipped, "world skipped");
    (world.raw, world.end.expect("end"))
}

#[test]
fn spec_olsolve_round_trip() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:olsolve=200").unwrap().spec(),
        "h0:olsolve=200"
    );
    let both = AnyPolicy::parse_spec("h0:olsolve=1000,horizon=3").unwrap();
    assert_eq!(both.spec(), "h0:olsolve=1000,horizon=3");
    assert_eq!(AnyPolicy::parse_spec(&both.spec()).unwrap(), both);
    let e = AnyPolicy::parse_spec("h0:olsolve=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

#[test]
fn olsolve0_identity_smoke() {
    let db = load_db();
    let st = ward_first_opp_lethal_state(&db);
    let mut off = parse_h0_v1("h0:olsolve=0,wv=300");
    let mut def = parse_h0_v1("h0:wv=300");
    let a = off.opponent_value(&db, &st, PlayerId::A);
    let b = def.opponent_value(&db, &st, PlayerId::A);
    assert_eq!(off.stats.opp_solver_calls, 0);
    assert_eq!(def.stats.opp_solver_calls, 0);
    assert!((a - b).abs() < 1e-5, "{a} vs {b}");
}

#[test]
fn ward_first_glance_misses_solver_finds() {
    let db = load_db();
    let st = ward_first_opp_lethal_state(&db);
    assert!(glance_misses(&db, &st, 300.0));
    match forced_lethal(&db, &st, 2_000) {
        LethalVerdict::Lethal { line, .. } => {
            let mut walk = st.clone();
            for a in &line {
                apply(&db, &mut walk, a.clone()).expect("line applies");
            }
            assert_eq!(walk.winner, Some(PlayerId::B));
        }
        other => panic!("expected Lethal, got {other:?}"),
    }
    let mut solve = parse_h0_v1("h0:olsolve=200,wv=300");
    let mut off = parse_h0_v1("h0:olsolve=0,wv=300");
    let sv = solve.opponent_value(&db, &st, PlayerId::A);
    let ov = off.opponent_value(&db, &st, PlayerId::A);
    assert!(is_neg_wv(sv, 300.0), "olsolve must see -wv, got {sv}");
    assert!(
        !is_neg_wv(ov, 300.0),
        "olsolve=0 must not see -wv, got {ov}"
    );
    assert_eq!(solve.stats.opp_solver_calls, 1);
    assert_eq!(solve.stats.opp_solver_found, 1);
    assert_eq!(solve.stats.opp_lethal_found, 0);
    assert!(solve.stats.opp_solver_nodes > 0);
}

#[test]
fn ward_first_end_turn_scores_solver_leaf() {
    let db = load_db();
    let (st, end) = {
        let (st, _trade, end) = ward_first_bot_turn_state(&db);
        (st, end)
    };
    let base = "h0:depth=6,beam=1,k=1,nodes=200,lcap=0.0125,value=v0,olethal=1,tt=0,alloc=fair";
    let (raw0, end0) = score_candidate_world(&format!("{base},olsolve=0"), &db, &st, &end);
    let (raw1, end1) = score_candidate_world(&format!("{base},olsolve=200"), &db, &st, &end);
    assert!(!is_neg_wv(raw0, 80.0), "olsolve=0 EndTurn raw={raw0}");
    assert!(is_neg_wv(raw1, 80.0), "olsolve=200 EndTurn raw={raw1}");
    assert_eq!(end1, PvEnd::OppSolver);
    assert_ne!(end0, PvEnd::OppSolver);
}

#[test]
fn two_storm_combination_solver_finds() {
    let db = load_db();
    let st = two_storm_lethal_state(&db);
    assert!(glance_misses(&db, &st, 300.0));
    match forced_lethal(&db, &st, 2_000) {
        LethalVerdict::Lethal { .. } => {}
        other => panic!("expected Lethal, got {other:?}"),
    }
    let mut solve = parse_h0_v1("h0:olsolve=200,wv=300");
    let mut off = parse_h0_v1("h0:olsolve=0,wv=300");
    let sv = solve.opponent_value(&db, &st, PlayerId::A);
    let ov = off.opponent_value(&db, &st, PlayerId::A);
    assert!(is_neg_wv(sv, 300.0), "olsolve must see -wv, got {sv}");
    assert!(!is_neg_wv(ov, 300.0), "olsolve=0 must not, got {ov}");
    assert_eq!(solve.stats.opp_solver_calls, 1);
    assert_eq!(solve.stats.opp_solver_found, 1);
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

#[test]
fn olsolve_determinism_smoke() {
    let db = load_db();
    const SEED: u64 = 20260929;
    const SPEC: &str = "h0:olsolve=200,depth=2,nodes=200";
    let first = play_pair_spec(&db, SPEC, SPEC, 2, SEED);
    let again = play_pair_spec(&db, SPEC, SPEC, 2, SEED);
    assert_eq!(first, again);
}

const GAMES_PER_DECK: u32 = 2;
const MAX_SNAPSHOTS_PER_GAME: usize = 8;

#[derive(Clone)]
struct SearchSnapshot {
    state: arena_engine::State,
    choose_seed: u64,
}

fn sample_snapshots_evenly(snaps: &[SearchSnapshot], max: usize) -> Vec<SearchSnapshot> {
    if snaps.len() <= max {
        return snaps.to_vec();
    }
    let step = snaps.len() as f64 / max as f64;
    (0..max)
        .map(|i| snaps[(i as f64 * step) as usize].clone())
        .collect()
}

fn collect_search_snapshots(db: &CardDb) -> Vec<SearchSnapshot> {
    let decks = load_pinned_corpus_decks_sorted(db);
    let mut snaps = Vec::new();
    for deck in decks.iter().take(16) {
        let mut deck_snaps = Vec::new();
        for g in 0..GAMES_PER_DECK {
            let seed = 20260929u64.wrapping_add(u64::from(g));
            let Ok(mut state) = new_game(
                db,
                GameConfig {
                    seed,
                    deck_a: deck.clone(),
                    deck_b: deck.clone(),
                    first: First::A,
                    opening_hands: None,
                },
            ) else {
                continue;
            };
            let mut rng = policy_rng(seed);
            let mut nact = 0u32;
            let mut game_snaps = Vec::new();
            while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
                if state.turn > MAX_TURNS || nact >= MAX_ACTIONS {
                    break;
                }
                let legal = legal_actions(db, &state);
                if legal.is_empty() {
                    break;
                }
                if matches!(state.phase, Phase::Main) && state.turn >= 3 {
                    let choose_seed = seed
                        .wrapping_add(u64::from(state.turn) * 97)
                        .wrapping_add(u64::from(nact) * 131);
                    game_snaps.push(SearchSnapshot {
                        state: state.clone(),
                        choose_seed,
                    });
                }
                let idx = rng.gen_range(legal.len() as u32) as usize;
                if apply(db, &mut state, legal[idx].clone()).is_err() {
                    break;
                }
                nact += 1;
            }
            deck_snaps.extend(sample_snapshots_evenly(&game_snaps, MAX_SNAPSHOTS_PER_GAME));
        }
        snaps.extend(deck_snaps);
    }
    snaps
}

struct SolverAccounting {
    over_node_cap: u64,
    solver_calls: u64,
    solver_found: u64,
    solver_unknown: u64,
    solver_nodes: u64,
}

fn audit_olsolve_decisions(db: &CardDb, nodes: u32, snaps: &[SearchSnapshot]) -> SolverAccounting {
    let spec = with_v2_net(&format!("h0:nodes={nodes},horizon=3,olsolve=1000"));
    let mut out = SolverAccounting {
        over_node_cap: 0,
        solver_calls: 0,
        solver_found: 0,
        solver_unknown: 0,
        solver_nodes: 0,
    };
    for snap in snaps {
        let mut h0 = parse_h0(&spec);
        h0.arm_explain();
        let legal = legal_actions(db, &snap.state);
        let mut rng = policy_rng(snap.choose_seed);
        let _ = h0.choose(db, &snap.state, &legal, &mut rng);
        let rec = h0.take_explain().unwrap_or_else(|| {
            panic!("olsolve decision missing explain");
        });
        if rec.nodes > rec.node_cap {
            out.over_node_cap += 1;
        }
        out.solver_calls += h0.stats.opp_solver_calls;
        out.solver_found += h0.stats.opp_solver_found;
        out.solver_unknown += h0.stats.opp_solver_unknown;
        out.solver_nodes += h0.stats.opp_solver_nodes;
    }
    out
}

fn olsolve_accounting_matches_meta_games_at_nodes(nodes: u32) {
    let db = load_db();
    let snaps = collect_search_snapshots(&db);
    assert!(!snaps.is_empty(), "need search snapshots");
    eprintln!("h0_olsolve meta nodes={nodes}: {} snapshots", snaps.len());
    let audit = audit_olsolve_decisions(&db, nodes, &snaps);
    assert_eq!(
        audit.over_node_cap, 0,
        "nodes={nodes}: rec.nodes must stay within node_cap"
    );
    eprintln!(
        "nodes={nodes}: solver_calls={} found={} unknown={} solver_nodes={}",
        audit.solver_calls, audit.solver_found, audit.solver_unknown, audit.solver_nodes
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn olsolve_accounting_matches_meta_games_nodes2000() {
    olsolve_accounting_matches_meta_games_at_nodes(2000);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn olsolve_accounting_matches_meta_games_nodes16000() {
    olsolve_accounting_matches_meta_games_at_nodes(16000);
}

fn pick_drift_report(base_spec: &str, olsolve: u32, limit: usize) {
    let db = load_db();
    let snaps = collect_search_snapshots(&db);
    let base = with_v2_net(base_spec);
    let on_spec = if base_spec == "h0" {
        format!("h0:olsolve={olsolve}")
    } else {
        format!("{base_spec},olsolve={olsolve}")
    };
    let on = with_v2_net(&on_spec);
    let mut compared = 0usize;
    let mut changed = 0usize;
    for snap in snaps.iter().take(limit) {
        let legal = legal_actions(&db, &snap.state);
        if legal.is_empty() {
            continue;
        }
        let mut rng0 = policy_rng(snap.choose_seed);
        let mut rng1 = policy_rng(snap.choose_seed);
        let mut h0 = parse_h0(&base);
        let mut h1 = parse_h0(&on);
        let i0 = h0.choose(&db, &snap.state, &legal, &mut rng0);
        let i1 = h1.choose(&db, &snap.state, &legal, &mut rng1);
        compared += 1;
        if i0 != i1 {
            changed += 1;
        }
    }
    eprintln!(
        "{on_spec}: compared={compared} changed={changed} ({:.1}%)",
        100.0 * changed as f64 / compared.max(1) as f64
    );
}

#[test]
#[ignore = "measurement helper for PR / olsolve_cost_bench companion"]
fn olsolve_pick_drift_report() {
    pick_drift_report("h0", 200, 128);
    pick_drift_report("h0:nodes=16000,horizon=3", 200, 128);
    pick_drift_report("h0:nodes=16000,horizon=3", 1000, 128);
}
