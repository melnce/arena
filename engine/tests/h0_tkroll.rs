//! H0 roll-confirmed own-turn kill (`tkroll`): spec round-trip, identity at
//! default off, rerolled confirmation, sure-kill fixtures, gamble safety,
//! and no-change when the roll check takes nothing.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use arena_engine::determinize::{determinize_with, determinize_with_stats, Info};
use arena_engine::policy::ChoosePath;
use arena_engine::{
    apply, apply_neutral, confirm_lethal_line_rerolled, forced_lethal, forced_lethal_accepting,
    from_neutral, legal_actions, new_game, policy_rng, roll_confirm_seed, search_key, Action,
    AnyPolicy, CardDb, First, GameConfig, LethalVerdict, Phase, PlayerId, Policy, H0, MAX_ACTIONS,
    MAX_TURNS,
};
use serde_json::Value;

mod common;
use common::*;

const STORM: &str = "10461110";
const PLAIN_SPEC: &str = "h0:nodes=16000,horizon=3";
const ROLL_SPEC: &str = "h0:nodes=16000,horizon=3,tkill=10000,tkroll=8";
const DET_ONLY_SPEC: &str = "h0:nodes=16000,horizon=3,tkill=10000";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn tkroll_fixture_dir() -> PathBuf {
    fixtures_dir().join("tkroll")
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

fn build_h0_roots(st: &arena_engine::State, me: PlayerId, seed: u64) -> Vec<arena_engine::State> {
    let h0 = parse_h0(ROLL_SPEC);
    let k = h0.determinizations.max(1);
    let mut rng = policy_rng(seed);
    let mut roots = Vec::with_capacity(k as usize);
    for _ in 0..k {
        if h0.info == Info::Open {
            roots.push(determinize_with_stats(
                st,
                me,
                rng.next_u64(),
                h0.info,
                None,
            ));
        } else {
            roots.push(determinize_with(st, me, rng.next_u64(), h0.info));
        }
    }
    roots
}

fn hand_card_ids(st: &arena_engine::State, who: PlayerId) -> Vec<String> {
    st.player(who)
        .hand
        .iter()
        .map(|c| c.card.to_string())
        .collect()
}

#[derive(Debug)]
struct RerollFailure {
    root_idx: usize,
    reroll: u32,
    step: usize,
    kind: &'static str,
    action: String,
    root0_key: u64,
    root_key: u64,
    root0_opp_hand: Vec<String>,
    root_opp_hand: Vec<String>,
    root0_opp_deck_len: usize,
    root_opp_deck_len: usize,
    root0_leader_def: i32,
    root_leader_def: i32,
    root0_phase: String,
    root_phase: String,
}

fn replay_line_failure(
    db: &CardDb,
    state: &arena_engine::State,
    me: PlayerId,
    line: &[Action],
    seed: u64,
) -> Option<(usize, String)> {
    let mut s = state.clone();
    s.reseed(seed);
    for (step, a) in line.iter().enumerate() {
        if apply(db, &mut s, a.clone()).is_err() {
            return Some((step, "illegal_apply".to_string()));
        }
    }
    if s.winner != Some(me) {
        return Some((line.len(), "no_kill".to_string()));
    }
    None
}

fn diagnose_roll_confirm(
    db: &CardDb,
    roots: &[arena_engine::State],
    me: PlayerId,
    line: &[Action],
    tkroll: u32,
) -> Vec<RerollFailure> {
    let mut out = Vec::new();
    let root0_key = search_key(&roots[0]);
    let root0_opp_hand = hand_card_ids(&roots[0], me.opponent());
    let root0_opp_deck_len = roots[0].player(me.opponent()).deck.len();
    let root0_leader_def = roots[0].player(me.opponent()).leader_defense;
    let root0_phase = format!("{:?}", roots[0].phase);
    for (root_idx, root) in roots.iter().enumerate() {
        let pos_key = search_key(root);
        for reroll in 0..tkroll {
            let seed = roll_confirm_seed(pos_key, reroll);
            if let Some((step, kind)) = replay_line_failure(db, root, me, line, seed) {
                out.push(RerollFailure {
                    root_idx,
                    reroll,
                    step,
                    kind: if kind == "illegal_apply" {
                        "illegal_apply"
                    } else {
                        "no_kill"
                    },
                    action: format!("{:?}", line.get(step).unwrap_or(&Action::EndTurn)),
                    root0_key,
                    root_key: pos_key,
                    root0_opp_hand: root0_opp_hand.clone(),
                    root_opp_hand: hand_card_ids(root, me.opponent()),
                    root0_opp_deck_len,
                    root_opp_deck_len: root.player(me.opponent()).deck.len(),
                    root0_leader_def,
                    root_leader_def: root.player(me.opponent()).leader_defense,
                    root0_phase: root0_phase.clone(),
                    root_phase: format!("{:?}", root.phase),
                });
                break;
            }
        }
    }
    out
}

fn reroll_pass_counts(
    db: &CardDb,
    roots: &[arena_engine::State],
    me: PlayerId,
    line: &[Action],
    tkroll: u32,
) -> Vec<(usize, u32)> {
    roots
        .iter()
        .enumerate()
        .map(|(ri, root)| {
            let pos_key = search_key(root);
            let mut pass = 0u32;
            for reroll in 0..tkroll {
                let seed = roll_confirm_seed(pos_key, reroll);
                if replay_line_failure(db, root, me, line, seed).is_none() {
                    pass += 1;
                }
            }
            (ri, pass)
        })
        .collect()
}

fn solver_line_on_first_root(
    db: &CardDb,
    st: &arena_engine::State,
    me: PlayerId,
    seed: u64,
    budget: u32,
) -> Option<Vec<Action>> {
    let h0 = parse_h0(ROLL_SPEC);
    let mut rng = policy_rng(seed);
    let root = determinize_with(st, me, rng.next_u64(), h0.info);
    match forced_lethal(db, &root, budget) {
        LethalVerdict::Lethal { line, .. } => Some(line),
        _ => None,
    }
}

fn accepted_line_on_first_root(
    db: &CardDb,
    st: &arena_engine::State,
    me: PlayerId,
    seed: u64,
    budget: u32,
    tkroll: u32,
) -> Option<Vec<Action>> {
    let h0 = parse_h0(ROLL_SPEC);
    let k = h0.determinizations.max(1);
    let mut rng = policy_rng(seed);
    let mut roots = Vec::with_capacity(k as usize);
    for _ in 0..k {
        roots.push(determinize_with(st, me, rng.next_u64(), h0.info));
    }
    let accept = |line: &[Action]| {
        roots.iter().all(|root| {
            let pos_key = search_key(root);
            let seeds: Vec<u64> = (0..tkroll).map(|i| roll_confirm_seed(pos_key, i)).collect();
            confirm_lethal_line_rerolled(db, root, me, line, &seeds)
        })
    };
    match forced_lethal_accepting(db, &roots[0], budget, accept) {
        LethalVerdict::Lethal { line, .. } => Some(line),
        _ => None,
    }
}

fn reroll_seeds(state: &arena_engine::State, n: u32) -> Vec<u64> {
    let pos_key = search_key(state);
    (0..n).map(|i| roll_confirm_seed(pos_key, i)).collect()
}

fn count_reroll_kills(
    db: &CardDb,
    state: &arena_engine::State,
    me: PlayerId,
    line: &[Action],
    rerolls: u32,
) -> u32 {
    let pos_key = search_key(state);
    let mut kills = 0u32;
    for i in 0..rerolls {
        let seed = roll_confirm_seed(pos_key, i);
        if confirm_lethal_line_rerolled(db, state, me, line, &[seed]) {
            kills += 1;
        }
    }
    kills
}

fn audit_line_from_json(db: &CardDb, state: &arena_engine::State, line: &Value) -> Vec<Action> {
    let mut s = state.clone();
    let mut out = Vec::new();
    for step in line.as_array().expect("audit line array") {
        let mut body = serde_json::Map::new();
        for (k, v) in step.as_object().expect("line step object") {
            body.insert(k.clone(), v.clone());
        }
        let neu: arena_engine::NeutralAction =
            serde_json::from_value(Value::Object(body)).expect("audit line step");
        let a = from_neutral(&s, &neu).expect("audit line action");
        apply(db, &mut s, a.clone()).expect("audit line replay");
        out.push(a);
    }
    out
}

fn count_audit_probe_kills(
    db: &CardDb,
    state: &arena_engine::State,
    me: PlayerId,
    line: &[Action],
    rerolls: u32,
) -> u32 {
    let mut kills = 0u32;
    for r in 0..rerolls {
        let mut s = state.clone();
        s.reseed(10_000 + u64::from(r));
        let mut ok = true;
        for a in line {
            if apply(db, &mut s, a.clone()).is_err() {
                ok = false;
                break;
            }
        }
        if ok && s.winner == Some(me) {
            kills += 1;
        }
    }
    kills
}

#[test]
fn spec_tkroll_round_trip() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:tkroll=0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:tkill=2000,tkroll=8")
            .unwrap()
            .spec(),
        "h0:tkill=2000,tkroll=8"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:tkill=2000").unwrap().spec(),
        "h0:tkill=2000"
    );
    let both = AnyPolicy::parse_spec("h0:tkill=1000,tkroll=8,horizon=3").unwrap();
    assert_eq!(both.spec(), "h0:tkill=1000,tkroll=8,horizon=3");
    assert_eq!(AnyPolicy::parse_spec(&both.spec()).unwrap(), both);
    let e = AnyPolicy::parse_spec("h0:tkroll=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

#[test]
fn tkroll0_identity_smoke() {
    let db = load_db();
    let st = two_storm_lethal_state(&db);
    let legal = legal_actions(&db, &st);
    let mut off = parse_h0("h0:tkroll=0");
    let mut def = parse_h0("h0");
    let mut rng0 = policy_rng(11);
    let mut rng1 = policy_rng(11);
    let i0 = off.choose(&db, &st, &legal, &mut rng0);
    let i1 = def.choose(&db, &st, &legal, &mut rng1);
    assert_eq!(off.stats.own_roll_calls, 0);
    assert_eq!(def.stats.own_roll_calls, 0);
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

#[test]
fn confirm_rerolled_rejects_rng_dependent_gamble() {
    let db = load_db();
    let path = tkroll_fixture_dir().join("fb-play-105-ply0079.json");
    let cap: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(&db, &cap, ply);
    let me = st.active;
    let legal = legal_actions(&db, &st);
    let seed = explain_seed(&cap, ply);
    let mut h0 = parse_h0(ROLL_SPEC);
    let mut rng = policy_rng(seed);
    let _ = h0.choose(&db, &st, &legal, &mut rng);
    let LethalVerdict::Lethal { line, .. } = forced_lethal(&db, &st, 50_000) else {
        panic!("gamble fixture must have a full solver kill on live state");
    };
    let kills = count_reroll_kills(&db, &st, me, &line, 32);
    assert!(
        kills < 30,
        "audit gamble line should not kill under most rerolls (got {kills}/32)"
    );
    let seeds = reroll_seeds(&st, 8);
    assert!(
        !confirm_lethal_line_rerolled(&db, &st, me, &line, &seeds),
        "8 rerolls must reject the audit gamble line on live state"
    );
}

#[test]
#[ignore = "PR diagnostic: per-root reroll confirmation failures"]
fn tkroll_fixture_root_diagnostics() {
    let db = load_db();
    let dir = tkroll_fixture_dir();
    for ent in fs::read_dir(&dir).expect("tkroll fixtures dir").flatten() {
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
        if cap["kind"].as_str() != Some("positive") {
            continue;
        }
        let ply = cap["ply"].as_u64().expect("ply") as usize;
        let st = replay_capture(&db, &cap, ply);
        let me = st.active;
        let seed = explain_seed(&cap, ply);
        let roots = build_h0_roots(&st, me, seed);
        let Some(line) = (match forced_lethal(&db, &roots[0], 10_000) {
            LethalVerdict::Lethal { line, .. } => Some(line),
            other => {
                eprintln!(
                    "\n=== {} ===\nroll solver on roots[0]: {other:?}",
                    path.display()
                );
                None
            }
        }) else {
            continue;
        };
        eprintln!("\n=== {} (ply {ply}, seed {seed}) ===", path.display());
        eprintln!("line_len={} line[0]={:?}", line.len(), line[0]);
        for (ri, pass) in reroll_pass_counts(&db, &roots, me, &line, 8) {
            eprintln!("root {ri}: {pass}/8 rerolls pass");
        }
        let fails = diagnose_roll_confirm(&db, &roots, me, &line, 8);
        if fails.is_empty() {
            eprintln!("all roots pass 8/8 rerolls");
            continue;
        }
        for f in &fails {
            eprintln!(
                "REJECT root={} reroll={} step={} kind={} action={}",
                f.root_idx, f.reroll, f.step, f.kind, f.action
            );
            eprintln!(
                "  root0_key={:#x} root_key={:#x} same_key={}",
                f.root0_key,
                f.root_key,
                f.root0_key == f.root_key
            );
            eprintln!(
                "  opp_hand root0={:?} root{}={:?}",
                f.root0_opp_hand, f.root_idx, f.root_opp_hand
            );
            eprintln!(
                "  opp_deck_len root0={} root{}={} leader_def root0={} root{}={} phase root0={} root{}={}",
                f.root0_opp_deck_len,
                f.root_idx,
                f.root_opp_deck_len,
                f.root0_leader_def,
                f.root_idx,
                f.root_leader_def,
                f.root0_phase,
                f.root_idx,
                f.root_phase
            );
        }
    }
}

#[test]
fn confirm_rerolled_accepts_sure_kill() {
    let db = load_db();
    let path = tkroll_fixture_dir().join("fa-play-174-ply0086.json");
    let cap: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(&db, &cap, ply);
    let legal = legal_actions(&db, &st);
    let seed = explain_seed(&cap, ply);
    let mut h0 = parse_h0(ROLL_SPEC);
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let _ = h0.choose(&db, &st, &legal, &mut rng);
    let rec = h0.take_explain().expect("explain");
    assert_eq!(rec.path, ChoosePath::TakeKillRoll, "sure-kill fixture");
    assert!(h0.stats.own_roll_taken > 0);
}

#[test]
fn confirm_rerolled_is_deterministic() {
    let db = load_db();
    let path = tkroll_fixture_dir().join("fa-play-98-ply0071.json");
    let cap: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(&db, &cap, ply);
    let me = st.active;
    let LethalVerdict::Lethal { line, .. } = forced_lethal(&db, &st, 50_000) else {
        panic!("fixture must have a full solver kill");
    };
    let seeds = reroll_seeds(&st, 8);
    let a = confirm_lethal_line_rerolled(&db, &st, me, &line, &seeds);
    let b = confirm_lethal_line_rerolled(&db, &st, me, &line, &seeds);
    assert_eq!(a, b);
}

fn assert_tkroll_takes_kill(db: &CardDb, path: &Path, cap: &Value) -> bool {
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(db, cap, ply);
    let me = st.active;
    let legal = legal_actions(db, &st);
    assert!(legal.len() > 1, "{path:?}: need multi-choice decision");
    let seed = explain_seed(cap, ply);

    let mut det_only = parse_h0(DET_ONLY_SPEC);
    let mut rng_det = policy_rng(seed);
    let _ = det_only.choose(db, &st, &legal, &mut rng_det);
    if det_only.stats.own_solver_taken > 0 {
        eprintln!("{path:?}: det-only already takes kill; fixture proves nothing for tkroll");
        return false;
    }

    let mut roll = parse_h0(ROLL_SPEC);
    roll.arm_explain();
    let mut rng_roll = policy_rng(seed);
    let roll_pick = roll.choose(db, &st, &legal, &mut rng_roll);
    let rec = roll.take_explain().expect("explain");
    if rec.path != ChoosePath::TakeKillRoll {
        eprintln!(
            "{path:?}: expected TakeKillRoll, got {:?} (roll found={} taken={} rejected={})",
            rec.path,
            roll.stats.own_roll_found,
            roll.stats.own_roll_taken,
            roll.stats.own_roll_rejected
        );
        return false;
    }
    if roll.stats.own_roll_taken == 0 {
        eprintln!("{path:?}: own_roll_taken=0");
        return false;
    }
    match forced_lethal(db, &st, 50_000) {
        LethalVerdict::Lethal { line, .. } => {
            let first = &line[0];
            let expected = legal
                .iter()
                .position(|a| a == first)
                .expect("line[0] legal");
            if roll_pick != expected {
                eprintln!("{path:?}: pick mismatch");
                return false;
            }
        }
        other => {
            eprintln!("{path:?}: expected full lethal at fixture, got {other:?}");
            return false;
        }
    }
    let (walk, ended_turn) = play_own_turn_with(db, &st, ROLL_SPEC, seed);
    if ended_turn {
        eprintln!("{path:?}: tkroll policy ended turn before lethal");
        return false;
    }
    if walk.winner != Some(me) {
        eprintln!("{path:?}: must win before EndTurn");
        return false;
    }
    true
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn tkroll_positive_fixtures_take_sure_kill() {
    let db = load_db();
    let dir = tkroll_fixture_dir();
    let mut converted = 0usize;
    let mut eligible = 0usize;
    let mut det_skipped = 0usize;
    // fb-play-122 and fb-play-28 include 2026-09-29 patched cards; replay is stale.
    let positive_total = 4usize;
    let mut replay_stale = 0usize;
    for ent in fs::read_dir(&dir).expect("tkroll fixtures dir").flatten() {
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
        if cap["kind"].as_str() != Some("positive") {
            continue;
        }
        if fixture_uses_balance_patch_cards(&cap) {
            eprintln!(
                "skip {path:?}: replay stale after 2026-09-29 balance patch (Disgraceful / Bewitching text)"
            );
            replay_stale += 1;
            continue;
        }
        let ply = cap["ply"].as_u64().expect("ply") as usize;
        let st = replay_capture(&db, &cap, ply);
        let legal = legal_actions(&db, &st);
        let seed = explain_seed(&cap, ply);
        let mut det_only = parse_h0(DET_ONLY_SPEC);
        let mut rng_det = policy_rng(seed);
        let _ = det_only.choose(&db, &st, &legal, &mut rng_det);
        if det_only.stats.own_solver_taken > 0 {
            eprintln!("skip {path:?}: det-only takes kill");
            det_skipped += 1;
            continue;
        }
        eligible += 1;
        if assert_tkroll_takes_kill(&db, &path, &cap) {
            converted += 1;
            eprintln!(
                "positive fixture {}: tkroll takes sure kill",
                path.display()
            );
        }
    }
    eprintln!(
        "positive summary: converted={converted}/{positive_total} eligible={eligible} det_skipped={det_skipped} replay_stale={replay_stale}"
    );
    assert_eq!(replay_stale, 2, "expected two stale positive fixtures (fb-play-122, fb-play-28)");
    assert!(
        converted >= 4,
        "need all {positive_total} replayable positive fixtures (converted={converted}, eligible={eligible}, det_skipped={det_skipped}); run tkroll_fixture_root_diagnostics --include-ignored"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn tkroll_gambles_not_taken_blindly() {
    let db = load_db();
    let dir = tkroll_fixture_dir();
    for ent in fs::read_dir(&dir).expect("tkroll fixtures dir").flatten() {
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read fixture")).expect("json");
        if cap["kind"].as_str() != Some("gamble") {
            continue;
        }
        if fixture_uses_balance_patch_cards(&cap) {
            eprintln!(
                "skip gamble {path:?}: replay stale after 2026-09-29 balance patch"
            );
            continue;
        }
        let ply = cap["ply"].as_u64().expect("ply") as usize;
        let st = replay_capture(&db, &cap, ply);
        let me = st.active;
        let legal = legal_actions(&db, &st);
        for bot in 0..8u64 {
            let seed = explain_seed(&cap, ply).wrapping_add(bot);
            let mut h0 = parse_h0(ROLL_SPEC);
            h0.arm_explain();
            let mut rng = policy_rng(seed);
            let pick = h0.choose(&db, &st, &legal, &mut rng);
            let rec = h0.take_explain().expect("explain");
            if rec.path != ChoosePath::TakeKillRoll {
                eprintln!(
                    "gamble {} bot seed {bot}: roll check did not take",
                    path.display()
                );
                continue;
            }
            let taken = &legal[pick];
            let line = accepted_line_on_first_root(&db, &st, me, seed, 10_000, 8)
                .or_else(|| solver_line_on_first_root(&db, &st, me, seed, 10_000))
                .expect("gamble fixture must have an accepted or solver line on first root");
            assert_eq!(
                taken,
                &line[0],
                "{} bot seed {bot}: must take accepted line[0]",
                path.display()
            );
            let kills = count_reroll_kills(&db, &st, me, &line, 32);
            eprintln!(
                "gamble {} bot seed {bot}: taken line kills {kills}/32 rerolls on true state",
                path.display()
            );
            assert!(
                kills >= 30,
                "{} bot seed {bot}: taken line must be near-sure (got {kills}/32)",
                path.display()
            );
        }
    }
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

fn collect_no_roll_kill_snapshots(db: &CardDb, target: usize) -> Vec<SearchSnapshot> {
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
            let seed = 50_000 + di as u64 * 100 + g as u64;
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
                    let mut probe = parse_h0("h0:tkill=2000,tkroll=8");
                    let mut prng = policy_rng(
                        seed.wrapping_add(u64::from(state.turn) * 97)
                            .wrapping_add(u64::from(nact) * 131),
                    );
                    let _ = probe.choose(db, &state, &legal, &mut prng);
                    if probe.stats.own_roll_taken == 0 {
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

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn no_kill_no_change_sample() {
    let db = load_db();
    let snaps = collect_no_roll_kill_snapshots(&db, 250);
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
        let mut base = parse_h0("h0:tkill=2000");
        let mut roll = parse_h0("h0:tkill=2000,tkroll=8");
        let i0 = base.choose(&db, &snap.state, &legal, &mut rng0);
        let i1 = roll.choose(&db, &snap.state, &legal, &mut rng1);
        compared += 1;
        assert_eq!(i0, i1, "seed={}", snap.choose_seed);
        assert_eq!(roll.stats.own_roll_taken, 0, "seed={}", snap.choose_seed);
    }
    assert!(compared >= 200, "compared only {compared}");
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
            let seed = 20261002u64
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
    roll_nodes_samples: Vec<u64>,
    roll_nodes_total: u64,
    roll_nodes_max: u64,
    roll_found: u64,
    roll_taken: u64,
    roll_rejected: u64,
    roll_lines_rejected: u64,
    roll_unknown: u64,
}

fn bench_choose(
    db: &CardDb,
    spec: &str,
    snap: &SearchSnapshot,
) -> (f64, u64, u64, u64, u64, u64, u64) {
    let mut h0 = parse_h0(spec);
    let legal = legal_actions(db, &snap.state);
    let mut rng = policy_rng(snap.choose_seed);
    let t0 = Instant::now();
    let _ = h0.choose(db, &snap.state, &legal, &mut rng);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    (
        ms,
        h0.stats.own_roll_nodes,
        h0.stats.own_roll_found,
        h0.stats.own_roll_taken,
        h0.stats.own_roll_rejected,
        h0.stats.own_roll_lines_rejected,
        h0.stats.own_roll_unknown,
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

#[test]
#[ignore = "measurement helper for PR cost table (paired meta mirrors)"]
fn tkroll_paired_cost_report() {
    let db = load_db();
    let snaps = collect_h0_selfplay_snapshots(&db);
    assert!(!snaps.is_empty(), "need h0 self-play snapshots");
    let specs = [
        "h0:tkill=2000",
        "h0:tkill=2000,tkroll=8",
        "h0:nodes=16000,horizon=3,tkill=10000",
        "h0:nodes=16000,horizon=3,tkill=10000,tkroll=8",
    ];
    let mut rows: Vec<BenchRow> = specs
        .iter()
        .map(|spec| BenchRow {
            spec: (*spec).to_string(),
            decisions: 0,
            ms_total: 0.0,
            ms_samples: Vec::new(),
            roll_nodes_samples: Vec::new(),
            roll_nodes_total: 0,
            roll_nodes_max: 0,
            roll_found: 0,
            roll_taken: 0,
            roll_rejected: 0,
            roll_lines_rejected: 0,
            roll_unknown: 0,
        })
        .collect();
    for snap in &snaps {
        for row in &mut rows {
            let (ms, nodes, found, taken, rejected, lines_rejected, unknown) =
                bench_choose(&db, &row.spec, snap);
            row.decisions += 1;
            row.ms_total += ms;
            row.ms_samples.push(ms);
            row.roll_nodes_samples.push(nodes);
            row.roll_nodes_total += nodes;
            row.roll_nodes_max = row.roll_nodes_max.max(nodes);
            row.roll_found += found;
            row.roll_taken += taken;
            row.roll_rejected += rejected;
            row.roll_lines_rejected += lines_rejected;
            row.roll_unknown += unknown;
        }
    }
    eprintln!(
        "{:<52} {:>10} {:>10} {:>10} {:>16} {:>10} {:>10} {:>10} {:>10} {:>12} {:>14} {:>12}",
        "spec",
        "decisions",
        "ms/dec",
        "p95_ms",
        "roll_nodes/dec",
        "p95_nodes",
        "nodes_max",
        "found/dec",
        "taken/dec",
        "rejected/dec",
        "lines_rej/dec",
        "unknown/dec",
    );
    for row in rows {
        let d = row.decisions as f64;
        eprintln!(
            "{:<52} {:>10} {:>10.2} {:>10.2} {:>16.2} {:>10} {:>10} {:>10.2} {:>10.2} {:>12.2} {:>14.2} {:>12.2}",
            row.spec,
            row.decisions,
            row.ms_total / d,
            p95(&row.ms_samples),
            row.roll_nodes_total as f64 / d,
            p95_u64(&row.roll_nodes_samples),
            row.roll_nodes_max,
            row.roll_found as f64 / d,
            row.roll_taken as f64 / d,
            row.roll_rejected as f64 / d,
            row.roll_lines_rejected as f64 / d,
            row.roll_unknown as f64 / d,
        );
    }
}

#[test]
#[ignore = "scan chance_kills for tkroll fixture replacements"]
fn scan_tkroll_fixture_candidates() {
    let db = load_db();
    let text = fs::read_to_string("/tmp/chance_kills.json").expect("chance_kills.json");
    let data: Value = serde_json::from_str(&text).expect("json");
    let skip = [
        "play-174", "play-386", "play-344", "play-64", "play-28", "play-103",
    ];
    for game in data["games"].as_array().expect("games") {
        let gid = game["game_id"].as_str().expect("game_id");
        if skip.contains(&gid) {
            continue;
        }
        for turn in game["chance_kill_turns"].as_array().unwrap_or(&vec![]) {
            let auditee = turn["auditee"].as_str().expect("auditee");
            let me = match auditee {
                "a" => PlayerId::A,
                "b" => PlayerId::B,
                _ => continue,
            };
            let first = turn["decisions"]
                .as_array()
                .expect("decisions")
                .iter()
                .min_by_key(|d| d["ply"].as_u64().unwrap_or(u64::MAX))
                .expect("decision");
            let ply = first["ply"].as_u64().expect("ply") as usize;
            let mut cap = serde_json::Map::new();
            cap.insert("seed".into(), game["seed"].clone());
            cap.insert("deckA".into(), game["deck_a_cards"].clone());
            cap.insert("deckB".into(), game["deck_b_cards"].clone());
            cap.insert("first".into(), game["first"].clone());
            cap.insert(
                "actions".into(),
                Value::Array(game["actions"].as_array().unwrap()[..ply].to_vec()),
            );
            cap.insert("ply".into(), first["ply"].clone());
            let st = replay_capture(&db, &Value::Object(cap), ply);
            if st.active != me {
                continue;
            }
            let legal = legal_actions(&db, &st);
            if legal.len() <= 1 {
                continue;
            }
            let audit_line = audit_line_from_json(&db, &st, &first["line"]);
            let audit_kills = count_audit_probe_kills(&db, &st, me, &audit_line, 32);
            if audit_kills != 32 {
                continue;
            }
            let seed = game["seed"].as_u64().expect("seed") + ply as u64;
            let mut det_only = parse_h0(DET_ONLY_SPEC);
            let mut rng_det = policy_rng(seed);
            let _ = det_only.choose(&db, &st, &legal, &mut rng_det);
            if det_only.stats.own_solver_taken > 0 {
                continue;
            }
            let mut roll = parse_h0(ROLL_SPEC);
            roll.arm_explain();
            let mut rng_roll = policy_rng(seed);
            let _ = roll.choose(&db, &st, &legal, &mut rng_roll);
            let rec = roll.take_explain().expect("explain");
            if rec.path == ChoosePath::TakeKillRoll && roll.stats.own_roll_taken > 0 {
                eprintln!(
                    "CANDIDATE {gid} ply={ply} auditee={auditee} audit_sure=32/32 det_miss roll_take lines_rejected={}",
                    roll.stats.own_roll_lines_rejected
                );
            }
        }
    }
}

#[test]
#[ignore = "measurement helper for 112-turn chance_kills replay"]
fn chance_kills_replay_report() {
    let db = load_db();
    let text = fs::read_to_string("/tmp/chance_kills.json").expect("chance_kills.json");
    let data: Value = serde_json::from_str(&text).expect("json");
    let specs = ["h0:tkill=2000,tkroll=8", "h0:tkill=10000,tkroll=8"];
    for spec in specs {
        let mut roll_takes = 0u32;
        let mut turn_wins = 0u32;
        let mut sure_takes_solver = 0u32;
        let mut sure_takes_audit = 0u32;
        let mut sure_wins = 0u32;
        let mut gamble_takes = 0u32;
        let mut n_sure_solver = 0u32;
        let mut n_sure_audit = 0u32;
        let mut n_gamble = 0u32;
        for game in data["games"].as_array().expect("games") {
            for turn in game["chance_kill_turns"].as_array().unwrap_or(&vec![]) {
                if turn["converted"].as_bool().unwrap_or(false) {
                    continue;
                }
                let auditee = turn["auditee"].as_str().expect("auditee");
                let me = match auditee {
                    "a" => PlayerId::A,
                    "b" => PlayerId::B,
                    _ => panic!("bad auditee"),
                };
                let decisions = turn["decisions"].as_array().expect("decisions");
                let first = decisions
                    .iter()
                    .min_by_key(|d| d["ply"].as_u64().unwrap_or(u64::MAX))
                    .expect("decision");
                let ply = first["ply"].as_u64().expect("ply") as usize;
                let mut cap = serde_json::Map::new();
                cap.insert("seed".into(), game["seed"].clone());
                cap.insert("deckA".into(), game["deck_a_cards"].clone());
                cap.insert("deckB".into(), game["deck_b_cards"].clone());
                cap.insert("first".into(), game["first"].clone());
                cap.insert(
                    "actions".into(),
                    Value::Array(game["actions"].as_array().unwrap()[..ply].to_vec()),
                );
                cap.insert("ply".into(), first["ply"].clone());
                let st = replay_capture(&db, &Value::Object(cap), ply);
                if st.active != me {
                    continue;
                }
                let legal = legal_actions(&db, &st);
                let seed = game["seed"].as_u64().expect("seed") + ply as u64;
                let audit_actions = audit_line_from_json(&db, &st, &first["line"]);
                let audit_probe_kills = count_audit_probe_kills(&db, &st, me, &audit_actions, 32);
                let sure_audit = audit_probe_kills == 32;
                if sure_audit {
                    n_sure_audit += 1;
                }
                let LethalVerdict::Lethal { line, .. } = forced_lethal(&db, &st, 50_000) else {
                    continue;
                };
                let solver_reroll_kills = count_reroll_kills(&db, &st, me, &line, 32);
                let sure_solver = solver_reroll_kills == 32;
                if sure_solver {
                    n_sure_solver += 1;
                } else if solver_reroll_kills <= 2 {
                    n_gamble += 1;
                }
                let mut h0 = parse_h0(spec);
                h0.arm_explain();
                let mut rng = policy_rng(seed);
                let _ = h0.choose(&db, &st, &legal, &mut rng);
                let rec = h0.take_explain().expect("explain");
                if rec.path == ChoosePath::TakeKillRoll {
                    roll_takes += 1;
                    if sure_solver {
                        sure_takes_solver += 1;
                    }
                    if sure_audit {
                        sure_takes_audit += 1;
                    }
                    if !sure_solver && solver_reroll_kills <= 2 {
                        gamble_takes += 1;
                    }
                    let (walk, ended_turn) = play_own_turn_with(&db, &st, spec, seed);
                    if !ended_turn && walk.winner == Some(me) {
                        turn_wins += 1;
                        if sure_solver {
                            sure_wins += 1;
                        }
                    }
                }
            }
        }
        eprintln!(
            "{spec}: roll_takes={roll_takes} turn_wins={turn_wins} \
             sure_takes_solver={sure_takes_solver}/{n_sure_solver} \
             sure_takes_audit={sure_takes_audit}/{n_sure_audit} \
             sure_wins={sure_wins} gamble_takes={gamble_takes}/{n_gamble}"
        );
        eprintln!(
            "  sure_solver: forced_lethal line on true state, roll_confirm_seed(search_key, r) r=0..31"
        );
        eprintln!(
            "  sure_audit: first lethal decision audit line on true state, reseed(10000+r) r=0..31"
        );
    }
}
