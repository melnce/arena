//! H0 own-turn deterministic kill (`tkill`): identity at default off,
//! spec round-trip, missed-kill fixtures, choice-phase follow-through,
//! and no-change when solver empty.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use arena_engine::policy::ChoosePath;
use arena_engine::{
    apply, apply_neutral, forced_lethal, forced_lethal_det, legal_actions, new_game, policy_rng,
    Action, AnyPolicy, CardDb, First, GameConfig, LethalVerdict, Phase, PlayerId, Policy, H0,
    MAX_ACTIONS, MAX_TURNS,
};
use serde_json::Value;

mod common;
use common::*;

const STORM: &str = "10461110";
const OMEGOTEP: &str = "10604110";
const PLAIN_SPEC: &str = "h0:nodes=16000,horizon=3";
const KILL_SPEC: &str = "h0:nodes=16000,horizon=3,tkill=50000";

/// Audit captures where plain `h0:nodes=16000,horizon=3` plays the turn out
/// and ends without winning (64-game audit, seed 1).
const AUDIT_PLAIN_MISS: &[&str] = &["39-0063.json", "14-0066.json"];

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn tkill_fixture_dir() -> PathBuf {
    fixtures_dir().join("tkill")
}

fn deck_from_json(v: &Value) -> Vec<arena_engine::CardId> {
    let map = v.as_object().expect("deck object");
    let mut sorted = BTreeMap::new();
    for (id, n) in map {
        sorted.insert(id.clone(), n.as_u64().unwrap_or(0) as usize);
    }
    let mut out = Vec::new();
    for (id, count) in sorted {
        let cid = arena_engine::CardId::parse(&id).unwrap_or_else(|| panic!("bad id {id}"));
        out.extend(std::iter::repeat_n(cid, count));
    }
    out
}

fn replay_capture(db: &CardDb, cap: &Value, n: usize) -> arena_engine::State {
    let seed = cap["seed"].as_u64().expect("seed");
    let deck_a = deck_from_json(&cap["deckA"]);
    let deck_b = deck_from_json(&cap["deckB"]);
    let first = match cap.get("first").and_then(|v| v.as_str()) {
        Some("b") | Some("B") => First::B,
        Some("a") | Some("A") => First::A,
        _ => First::Coin,
    };
    let mut st = new_game(
        db,
        GameConfig {
            seed,
            deck_a,
            deck_b,
            first,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let actions = cap["actions"].as_array().expect("actions");
    for step in actions.iter().take(n) {
        if step.get("reseed").is_some() {
            st.reseed(step["reseed"].as_u64().expect("reseed"));
            continue;
        }
        let mut body = serde_json::Map::new();
        for (k, v) in step.as_object().expect("action object") {
            if k == "value" || k == "bot_value" {
                continue;
            }
            body.insert(k.clone(), v.clone());
        }
        let neu: arena_engine::NeutralAction =
            serde_json::from_value(Value::Object(body)).expect("action");
        apply_neutral(db, &mut st, &neu).expect("apply_neutral");
    }
    st
}

fn explain_seed(cap: &Value, ply: usize) -> u64 {
    cap["seed"].as_u64().expect("seed") + ply as u64
}

fn is_audit_fixture(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_digit()) && !name.contains("ply")
}

fn play_own_turn_with(
    db: &CardDb,
    st: &arena_engine::State,
    spec: &str,
    seed: u64,
) -> (arena_engine::State, bool) {
    let me = st.active;
    let mut walk = st.clone();
    let mut h0 = parse_h0(spec);
    let mut rng = policy_rng(seed);
    let mut ended_turn = false;
    while walk.winner.is_none() && walk.active == me {
        if matches!(walk.phase, Phase::End | Phase::Terminal) {
            break;
        }
        let legal = legal_actions(db, &walk);
        if legal.is_empty() {
            break;
        }
        let idx = h0.choose(db, &walk, &legal, &mut rng);
        let a = legal[idx].clone();
        if matches!(a, Action::EndTurn) {
            ended_turn = true;
            break;
        }
        apply(db, &mut walk, a).expect("apply");
    }
    (walk, ended_turn)
}

fn assert_tkill_takes_kill(db: &CardDb, path: &Path, cap: &Value) {
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(db, cap, ply);
    let me = st.active;
    let legal = legal_actions(db, &st);
    assert!(legal.len() > 1, "{path:?}: need multi-choice decision");
    let mut kill = parse_h0(KILL_SPEC);
    kill.arm_explain();
    let seed = explain_seed(cap, ply);
    let mut rng_kill = policy_rng(seed);
    let kill_pick = kill.choose(db, &st, &legal, &mut rng_kill);
    let rec = kill.take_explain().expect("explain");
    assert_eq!(rec.path, ChoosePath::TakeKill, "{path:?}");
    match forced_lethal_det(db, &st, 50_000) {
        LethalVerdict::Lethal { line, .. } => {
            let first = &line[0];
            let expected = legal
                .iter()
                .position(|a| a == first)
                .expect("line[0] legal");
            assert_eq!(kill_pick, expected, "{path:?}");
        }
        other => panic!("{path:?}: expected det lethal at fixture, got {other:?}"),
    }
    let (walk, ended_turn) = play_own_turn_with(db, &st, KILL_SPEC, seed);
    assert!(
        !ended_turn,
        "{path:?}: tkill policy ended turn before lethal"
    );
    assert_eq!(walk.winner, Some(me), "{path:?}: must win before EndTurn");
}

#[test]
fn spec_tkill_round_trip() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:tkill=0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:tkill=2000").unwrap().spec(),
        "h0:tkill=2000"
    );
    let both = AnyPolicy::parse_spec("h0:tkill=1000,horizon=3").unwrap();
    assert_eq!(both.spec(), "h0:tkill=1000,horizon=3");
    assert_eq!(AnyPolicy::parse_spec(&both.spec()).unwrap(), both);
    let e = AnyPolicy::parse_spec("h0:tkill=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

#[test]
fn tkill0_identity_smoke() {
    let db = load_db();
    let st = two_storm_lethal_state(&db);
    let legal = legal_actions(&db, &st);
    let mut off = parse_h0("h0:tkill=0");
    let mut def = parse_h0("h0");
    let mut rng0 = policy_rng(11);
    let mut rng1 = policy_rng(11);
    let i0 = off.choose(&db, &st, &legal, &mut rng0);
    let i1 = def.choose(&db, &st, &legal, &mut rng1);
    assert_eq!(off.stats.own_solver_calls, 0);
    assert_eq!(def.stats.own_solver_calls, 0);
    assert_eq!(i0, i1);
}

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

fn omegotep_rng_kill_state(db: &CardDb, seed: u64) -> arena_engine::State {
    let mut st = started(db, seed);
    skip_to_player_turn(db, &mut st, PlayerId::A, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    put_hand(db, &mut st, PlayerId::A, OMEGOTEP);
    set_round(&mut st, PlayerId::A, 10);
    give_pp(&mut st, PlayerId::A, 9, 10);
    st.player_mut(PlayerId::B).leader_defense = 2;
    st.player_mut(PlayerId::A).leader_defense = 20;
    st
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn forced_lethal_det_solver_behaviour() {
    let db = load_db();
    let det_st = two_storm_lethal_state(&db);
    match forced_lethal(&db, &det_st, 50_000) {
        LethalVerdict::Lethal { rng_dependent, .. } => assert!(!rng_dependent),
        other => panic!("expected deterministic Lethal, got {other:?}"),
    }
    match forced_lethal_det(&db, &det_st, 50_000) {
        LethalVerdict::Lethal { rng_dependent, .. } => assert!(!rng_dependent),
        other => panic!("expected Lethal from det solver, got {other:?}"),
    }

    let mut found = false;
    for seed in 1..8_000 {
        let st = omegotep_rng_kill_state(&db, seed);
        let full = forced_lethal(&db, &st, 2_000);
        let det = forced_lethal_det(&db, &st, 2_000);
        if matches!(
            full,
            LethalVerdict::Lethal {
                rng_dependent: true,
                ..
            }
        ) && matches!(
            det,
            LethalVerdict::None { .. } | LethalVerdict::Unknown { .. }
        ) {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "need a seed where full solver finds an rng-dependent kill and det does not"
    );

    let mut st = started(&db, 1);
    give_pp(&mut st, PlayerId::A, 10, 10);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, STORM);
    put_hand(&db, &mut st, PlayerId::A, STORM);
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().any(|a| matches!(a, Action::Play { .. })));
    assert!(matches!(
        forced_lethal_det(&db, &st, 0),
        LethalVerdict::Unknown { nodes: 0 }
    ));
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn audit_fixtures_plain_h0_ends_turn_without_win() {
    let db = load_recorded_db();
    for name in AUDIT_PLAIN_MISS {
        let path = tkill_fixture_dir().join(name);
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
        let ply = cap["ply"].as_u64().expect("ply") as usize;
        let st = replay_capture(&db, &cap, ply);
        let seed = explain_seed(&cap, ply);
        let spec = with_v2_net(PLAIN_SPEC);
        let (walk, ended_turn) = play_own_turn_with(&db, &st, &spec, seed);
        assert!(ended_turn, "{name}: plain h0 must EndTurn without winning");
        assert_eq!(walk.winner, None, "{name}: plain h0 must not win the game");
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn tkill_fixtures_take_deterministic_kill() {
    let db = load_recorded_db();
    let dir = tkill_fixture_dir();
    let entries = fs::read_dir(&dir).expect("tkill fixtures dir");
    let mut any = false;
    for ent in entries.flatten() {
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap();
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
        assert_tkill_takes_kill(&db, &path, &cap);
        any = true;
        if is_audit_fixture(name) {
            eprintln!("audit fixture {name}: tkill takes kill");
        } else {
            eprintln!("review fixture {name}: tkill takes kill (plain h0 may also convert)");
        }
    }
    assert!(
        any,
        "need at least one tkill fixture under engine/tests/fixtures/tkill/"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn choice_phase_kill_followed_through() {
    let db = load_recorded_db();
    let path = tkill_fixture_dir().join("9420-ply0072.json");
    let cap: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(&db, &cap, ply);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "fixture must be at a Choice decision"
    );
    match forced_lethal_det(&db, &st, 50_000) {
        LethalVerdict::Lethal { line, .. } => {
            assert!(
                matches!(line.first(), Some(Action::Choose { .. })),
                "kill line must start with Choose"
            );
        }
        other => panic!("expected det lethal at choice fixture, got {other:?}"),
    }
    assert_tkill_takes_kill(&db, &path, &cap);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn no_kill_no_change_sample() {
    let db = load_db();
    let snaps = collect_no_det_kill_snapshots(&db, 250);
    assert!(
        snaps.len() >= 200,
        "need >= 200 own-turn samples, got {}",
        snaps.len()
    );
    let mut compared = 0usize;
    for snap in snaps {
        let legal = legal_actions(&db, &snap.state);
        if legal.len() <= 1 {
            continue;
        }
        let mut rng0 = policy_rng(snap.choose_seed);
        let mut rng1 = policy_rng(snap.choose_seed);
        let mut off = parse_h0("h0");
        let mut on = parse_h0("h0:tkill=2000");
        let i0 = off.choose(&db, &snap.state, &legal, &mut rng0);
        let i1 = on.choose(&db, &snap.state, &legal, &mut rng1);
        compared += 1;
        assert_eq!(i0, i1, "seed={}", snap.choose_seed);
    }
    assert!(compared >= 200, "compared only {compared}");
}

#[derive(Clone)]
struct SearchSnapshot {
    state: arena_engine::State,
    choose_seed: u64,
}

fn own_turn_decision(state: &arena_engine::State, legal: &[Action]) -> bool {
    matches!(
        state.phase,
        Phase::Main | Phase::Combat | Phase::Choice { .. }
    ) && legal.len() > 1
}

fn collect_no_det_kill_snapshots(db: &CardDb, target: usize) -> Vec<SearchSnapshot> {
    let decks = [
        "oracle/decks/meta-forest-combo.json",
        "oracle/decks/meta-rune-crystal.json",
        "oracle/decks/meta-sword-rally.json",
        "oracle/decks/meta-abyss-midrange.json",
        "oracle/decks/meta-dragon-ramp.json",
        "oracle/decks/meta-haven-evo.json",
    ];
    let mut snaps = Vec::new();
    for (di, deck_path) in decks.iter().enumerate() {
        let deck = load_deck_file(deck_path);
        for g in 0..6 {
            let seed = 40_000 + di as u64 * 100 + g as u64;
            let mut state = new_game(
                db,
                GameConfig {
                    seed,
                    deck_a: deck.clone(),
                    deck_b: deck.clone(),
                    first: First::A,
                    opening_hands: None,
                },
            )
            .expect("new_game");
            let mut rng = policy_rng(seed);
            let mut nact = 0u32;
            while state.winner.is_none() && state.turn < 16 {
                let legal = legal_actions(db, &state);
                if legal.is_empty() {
                    break;
                }
                if state.active == PlayerId::A && own_turn_decision(&state, &legal) {
                    match forced_lethal_det(db, &state, 2_000) {
                        LethalVerdict::None { .. } | LethalVerdict::Unknown { .. } => {
                            snaps.push(SearchSnapshot {
                                state: state.clone(),
                                choose_seed: seed
                                    .wrapping_add(u64::from(state.turn) * 97)
                                    .wrapping_add(u64::from(nact) * 131),
                            });
                            if snaps.len() >= target {
                                return snaps;
                            }
                        }
                        LethalVerdict::Lethal { .. } => {}
                    }
                }
                let idx = (rng.next_u64() % legal.len() as u64) as usize;
                if apply(db, &mut state, legal[idx].clone()).is_err() {
                    break;
                }
                nact += 1;
            }
        }
    }
    snaps
}

fn meta_deck_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .filter(|s| s != "meta-rune-test-subject")
        .collect()
}

const GAMES_PER_DECK: u32 = 2;
const MAX_SNAPSHOTS_PER_GAME: usize = 6;

fn sample_snapshots_evenly(snaps: &[SearchSnapshot], max: usize) -> Vec<SearchSnapshot> {
    if snaps.len() <= max {
        return snaps.to_vec();
    }
    let mut out = Vec::with_capacity(max);
    for i in 0..max {
        let idx = i * (snaps.len() - 1) / (max - 1);
        out.push(snaps[idx].clone());
    }
    out
}

fn collect_h0_selfplay_snapshots(db: &CardDb) -> Vec<SearchSnapshot> {
    let spec = PLAIN_SPEC;
    let mut snaps = Vec::new();
    for stem in meta_deck_stems() {
        let deck = load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")));
        let mut deck_snaps = Vec::new();
        for g in 0..GAMES_PER_DECK {
            let seed = 20261001u64
                .wrapping_add(u64::from(g))
                .wrapping_add(stem.len() as u64 * 97);
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
            let mut pol = parse_h0(spec);
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
                if state.active == PlayerId::A
                    && state.turn >= 3
                    && own_turn_decision(&state, &legal)
                {
                    let choose_seed = seed
                        .wrapping_add(u64::from(state.turn) * 97)
                        .wrapping_add(u64::from(nact) * 131);
                    game_snaps.push(SearchSnapshot {
                        state: state.clone(),
                        choose_seed,
                    });
                }
                let idx = pol.choose(db, &state, &legal, &mut rng);
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

struct BenchRow {
    spec: String,
    decisions: usize,
    ms_total: f64,
    ms_samples: Vec<f64>,
    solver_nodes_samples: Vec<u64>,
    solver_nodes_total: u64,
    solver_nodes_max: u64,
    found: u64,
    taken: u64,
    rejected: u64,
    unknown: u64,
}

fn bench_choose(db: &CardDb, spec: &str, snap: &SearchSnapshot) -> (f64, u64, u64, u64, u64, u64) {
    let mut h0 = parse_h0(spec);
    let legal = legal_actions(db, &snap.state);
    let mut rng = policy_rng(snap.choose_seed);
    let t0 = Instant::now();
    let _ = h0.choose(db, &snap.state, &legal, &mut rng);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    (
        ms,
        h0.stats.own_solver_nodes,
        h0.stats.own_solver_found,
        h0.stats.own_solver_taken,
        h0.stats.own_solver_rejected,
        h0.stats.own_solver_unknown,
    )
}

fn p95(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut xs = samples.to_vec();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let i = ((0.95 * xs.len() as f64).ceil() as usize).saturating_sub(1);
    xs[i.min(xs.len() - 1)]
}

fn p95_u64(samples: &[u64]) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let mut xs = samples.to_vec();
    xs.sort_unstable();
    let i = ((0.95 * xs.len() as f64).ceil() as usize).saturating_sub(1);
    xs[i.min(xs.len() - 1)]
}

fn tkill_specs(base: &str) -> Vec<String> {
    [0u32, 500, 2000, 10_000]
        .iter()
        .map(|tk| {
            if *tk == 0 && !base.contains(':') {
                base.to_string()
            } else if base.contains(':') {
                format!("{base},tkill={tk}")
            } else {
                format!("{base}:tkill={tk}")
            }
        })
        .collect()
}

#[test]
#[ignore = "measurement helper for tools/tkill_cost_bench.py (paired meta mirrors)"]
fn tkill_paired_cost_report() {
    let db = load_db();
    let snaps = collect_h0_selfplay_snapshots(&db);
    assert!(!snaps.is_empty(), "need h0 self-play snapshots");
    eprintln!(
        "tkill paired bench: {} positions ({} games/deck, up to {} own-turn decisions/game from round 3)",
        snaps.len(),
        GAMES_PER_DECK,
        MAX_SNAPSHOTS_PER_GAME
    );
    let bases = [PLAIN_SPEC, "h0"];
    let mut rows: Vec<BenchRow> = Vec::new();
    for base in bases {
        for spec in tkill_specs(base) {
            rows.push(BenchRow {
                spec,
                decisions: 0,
                ms_total: 0.0,
                ms_samples: Vec::new(),
                solver_nodes_samples: Vec::new(),
                solver_nodes_total: 0,
                solver_nodes_max: 0,
                found: 0,
                taken: 0,
                rejected: 0,
                unknown: 0,
            });
        }
    }
    for snap in &snaps {
        for row in &mut rows {
            let (ms, nodes, found, taken, rejected, unknown) = bench_choose(&db, &row.spec, snap);
            row.decisions += 1;
            row.ms_total += ms;
            row.ms_samples.push(ms);
            row.solver_nodes_samples.push(nodes);
            row.solver_nodes_total += nodes;
            row.solver_nodes_max = row.solver_nodes_max.max(nodes);
            row.found += found;
            row.taken += taken;
            row.rejected += rejected;
            row.unknown += unknown;
        }
    }
    eprintln!(
        "{:<52} {:>10} {:>10} {:>10} {:>16} {:>10} {:>10} {:>10} {:>10} {:>12} {:>12}",
        "spec",
        "decisions",
        "ms/dec",
        "p95_ms",
        "solver_nodes/dec",
        "p95_nodes",
        "nodes_max",
        "found/dec",
        "taken/dec",
        "rejected/dec",
        "unknown/dec"
    );
    for row in rows {
        let d = row.decisions as f64;
        eprintln!(
            "{:<52} {:>10} {:>10.2} {:>10.2} {:>16.2} {:>10} {:>10} {:>10.2} {:>10.2} {:>12.2} {:>12.2}",
            row.spec,
            row.decisions,
            row.ms_total / d,
            p95(&row.ms_samples),
            row.solver_nodes_total as f64 / d,
            p95_u64(&row.solver_nodes_samples),
            row.solver_nodes_max,
            row.found as f64 / d,
            row.taken as f64 / d,
            row.rejected as f64 / d,
            row.unknown as f64 / d,
        );
    }
}
