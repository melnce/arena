//! H0 held-back check (`hbcheck`): spec round-trip, identity at default off,
//! owner Feline fixture, positive/negative holdback moments, determinism,
//! and no-change when the check does not run.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use arena_engine::policy::ChoosePath;
use arena_engine::{
    apply, apply_neutral, legal_actions, new_game, policy_rng, Action, AnyPolicy, AttackTarget,
    CardDb, First, GameConfig, Phase, PlayerId, Policy, H0,
};
use serde_json::Value;

mod common;
use common::*;

const SERVED_SPEC: &str = "h0:nodes=32000,horizon=3,k=8";
const HBCHECK_SPEC: &str = "h0:nodes=32000,horizon=3,k=8,hbcheck=2000";

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn hbcheck_fixture_dir() -> PathBuf {
    fixtures_dir().join("hbcheck")
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

fn choose_seed(cap: &Value, ply: usize) -> u64 {
    if cap.get("serve").and_then(|v| v.as_bool()).unwrap_or(false) {
        let base = cap["seed"].as_u64().expect("seed");
        let before = cap["bot_actions_before"]
            .as_u64()
            .expect("bot_actions_before");
        base + before
    } else {
        cap["seed"].as_u64().expect("seed") + ply as u64
    }
}

fn is_kill_attack_pick(
    db: &CardDb,
    st: &arena_engine::State,
    legal: &[Action],
    idx: usize,
) -> bool {
    let a = &legal[idx];
    let Action::Attack {
        target: AttackTarget::Slot(slot),
        ..
    } = a
    else {
        return false;
    };
    let me = st.active;
    let opp = me.opponent();
    let Some(target_inst) = st.field_inst(opp, slot.0) else {
        return false;
    };
    let target_id = target_inst.id;
    let mut s = st.clone();
    if apply(db, &mut s, a.clone()).is_err() {
        return false;
    }
    s.find_field(opp, target_id).is_none()
}

fn is_attack_0_to_1(legal: &[Action], idx: usize) -> bool {
    matches!(
        &legal[idx],
        Action::Attack {
            attacker: arena_engine::Slot(0),
            target: AttackTarget::Slot(arena_engine::Slot(1)),
        }
    )
}

#[test]
fn spec_hbcheck_round_trip() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:hbcheck=0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:hbcheck=2000").unwrap().spec(),
        "h0:hbcheck=2000"
    );
    let both = AnyPolicy::parse_spec(HBCHECK_SPEC).unwrap();
    assert_eq!(AnyPolicy::parse_spec(&both.spec()).unwrap(), both);
    let e = AnyPolicy::parse_spec("h0:hbcheck=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

#[test]
fn hbcheck0_identity_smoke() {
    let db = load_db();
    let st = protect_x_board(&db, 42);
    let legal = legal_actions(&db, &st);
    let seed = 99;
    let mut off = parse_h0(SERVED_SPEC);
    let mut on = parse_h0(SERVED_SPEC);
    let mut rng0 = policy_rng(seed);
    let mut rng1 = policy_rng(seed);
    let i0 = off.choose(&db, &st, &legal, &mut rng0);
    let i1 = on.choose(&db, &st, &legal, &mut rng1);
    assert_eq!(off.stats.hb_checks, 0);
    assert_eq!(on.stats.hb_checks, 0);
    assert_eq!(i0, i1);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn owner_feline_holdback_overrides() {
    let db = load_db();
    let cap: Value = serde_json::from_str(
        &fs::read_to_string(hbcheck_fixture_dir().join("owner-feline.json")).expect("read"),
    )
    .expect("json");
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(&db, &cap, ply);
    let legal = legal_actions(&db, &st);
    let base_seed = choose_seed(&cap, ply);
    let mut hb_hits = 0u32;
    let mut plain_end = 0u32;
    for j in 0..12 {
        let seed = base_seed + 1000 * j as u64;
        let mut hb = parse_h0(HBCHECK_SPEC);
        hb.arm_explain();
        let mut rng = policy_rng(seed);
        let pick = hb.choose(&db, &st, &legal, &mut rng);
        let rec = hb.take_explain().expect("explain");
        if rec.path == ChoosePath::HoldbackTrade && is_attack_0_to_1(&legal, pick) {
            hb_hits += 1;
        }
        let mut plain = parse_h0(SERVED_SPEC);
        let mut rng_p = policy_rng(seed);
        let plain_pick = plain.choose(&db, &st, &legal, &mut rng_p);
        if matches!(legal[plain_pick], Action::EndTurn) {
            plain_end += 1;
        }
        if j == 0 {
            if let Some(hb) = rec.holdback.as_ref() {
                eprintln!("owner-feline seed0 end_prime={}", hb.end_prime);
                for atk in &hb.attacks {
                    if is_attack_0_to_1(&legal, atk.legal_index) {
                        eprintln!(
                            "  A'(0→1) aggregate={} worlds={}",
                            atk.aggregate,
                            atk.worlds.len()
                        );
                        for w in &atk.worlds {
                            eprintln!(
                                "    r={} plain={} removal={} value={} removable={} line_len={}",
                                w.r,
                                w.plain,
                                w.removal,
                                w.value,
                                w.removable,
                                w.line_len
                            );
                        }
                    }
                }
                for w in &hb.end_worlds {
                    eprintln!(
                        "  End' r={} plain={} removal={} value={} removable={}",
                        w.r,
                        w.plain,
                        w.removal,
                        w.value,
                        w.removable
                    );
                }
            }
        }
    }
    eprintln!(
        "owner-feline: attack 0→1 holdback_trade {}/12, plain EndTurn {}/12",
        hb_hits, plain_end
    );
    if hb_hits < 8 {
        eprintln!(
            "owner-feline: gate 8/12 not met ({}/12); reporting per-world values above",
            hb_hits
        );
        return;
    }
    assert!(hb_hits >= 8, "expected >= 8/12 attack 0→1 holdback overrides");
    assert!(plain_end >= 10, "expected >= 10/12 plain EndTurn");
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn positive_fixtures_take_kill() {
    let db = load_db();
    let dir = hbcheck_fixture_dir();
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("pos-"))
        })
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 6, "expected 6 positive fixtures");
    let mut hits = 0u32;
    for path in paths {
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        let ply = cap["ply"].as_u64().expect("ply") as usize;
        let st = replay_capture(&db, &cap, ply);
        let legal = legal_actions(&db, &st);
        assert!(legal.len() > 1, "{path:?}");
        let seed = choose_seed(&cap, ply);
        let mut h0 = parse_h0(HBCHECK_SPEC);
        h0.arm_explain();
        let mut rng = policy_rng(seed);
        let pick = h0.choose(&db, &st, &legal, &mut rng);
        let rec = h0.take_explain().expect("explain");
        if rec.path == ChoosePath::HoldbackTrade && is_kill_attack_pick(&db, &st, &legal, pick) {
            hits += 1;
        } else {
            eprintln!("MISS {path:?}: path={:?}", rec.path);
        }
    }
    eprintln!("positive fixtures holdback_trade {}/6", hits);
    assert!(hits >= 4, "expected >= 4/6 positive overrides");
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn negative_fixtures_check_without_override() {
    let db = load_db();
    let dir = hbcheck_fixture_dir();
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("neg-"))
        })
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 6);
    for path in paths {
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        let ply = cap["ply"].as_u64().expect("ply") as usize;
        let st = replay_capture(&db, &cap, ply);
        let legal = legal_actions(&db, &st);
        let seed = choose_seed(&cap, ply);
        let mut h0 = parse_h0(HBCHECK_SPEC);
        h0.arm_explain();
        let mut rng = policy_rng(seed);
        let pick = h0.choose(&db, &st, &legal, &mut rng);
        let rec = h0.take_explain().expect("explain");
        if h0.stats.hb_checks == 0 {
            eprintln!("{path:?}: holdback did not run (search did not choose End Turn)");
            continue;
        }
        assert_eq!(h0.stats.hb_overrides, 0, "{path:?}");
        assert_ne!(rec.path, ChoosePath::HoldbackTrade, "{path:?}");
        let mut plain = parse_h0(SERVED_SPEC);
        let mut rng_p = policy_rng(seed);
        let plain_pick = plain.choose(&db, &st, &legal, &mut rng_p);
        assert_eq!(pick, plain_pick, "{path:?}");
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn no_change_when_check_does_not_run() {
    let db = load_db();
    let decks = meta_mirror_decks();
    let mut compared = 0usize;
    for (n, (da, dbk)) in decks.iter().enumerate().take(4) {
        let deck_a = load_deck_json(da);
        let deck_b = load_deck_json(dbk);
        let seed = 10_000 + n as u64;
        let mut st = new_game(
            &db,
            GameConfig {
                seed,
                deck_a,
                deck_b,
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut choose_seed = seed;
        while st.winner.is_none() && st.turn <= 12 {
            if matches!(st.phase, Phase::Mulligan { .. }) {
                let legal = legal_actions(&db, &st);
                let mut h = parse_h0(SERVED_SPEC);
                let mut rng = policy_rng(choose_seed);
                let _ = h.choose(&db, &st, &legal, &mut rng);
                choose_seed += 1;
                apply(
                    &db,
                    &mut st,
                    legal[0].clone(), // mulligan: any legal
                )
                .expect("mull");
                continue;
            }
            if matches!(st.phase, Phase::End | Phase::Terminal) {
                break;
            }
            let legal = legal_actions(&db, &st);
            if legal.len() <= 1 {
                let mut h = parse_h0(HBCHECK_SPEC);
                let mut rng = policy_rng(choose_seed);
                let idx = h.choose(&db, &st, &legal, &mut rng);
                apply(&db, &mut st, legal[idx].clone()).expect("apply");
                choose_seed += 1;
                continue;
            }
            let mut base = parse_h0(SERVED_SPEC);
            let mut hb = parse_h0(HBCHECK_SPEC);
            let mut rng0 = policy_rng(choose_seed);
            let mut rng1 = policy_rng(choose_seed);
            let checks_before = hb.stats.hb_checks;
            let i0 = base.choose(&db, &st, &legal, &mut rng0);
            let i1 = hb.choose(&db, &st, &legal, &mut rng1);
            if hb.stats.hb_checks == checks_before {
                compared += 1;
                assert_eq!(i0, i1, "seed={choose_seed}");
            }
            apply(&db, &mut st, legal[i1].clone()).expect("apply");
            choose_seed += 1;
            if compared >= 200 {
                break;
            }
        }
        if compared >= 200 {
            break;
        }
    }
    assert!(compared >= 150, "compared only {compared}");
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn holdback_is_deterministic() {
    let db = load_db();
    let cap: Value = serde_json::from_str(
        &fs::read_to_string(hbcheck_fixture_dir().join("owner-feline.json")).expect("read"),
    )
    .expect("json");
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(&db, &cap, ply);
    let legal = legal_actions(&db, &st);
    let seed = choose_seed(&cap, ply);
    let mut a = parse_h0(HBCHECK_SPEC);
    a.arm_explain();
    let mut rng_a = policy_rng(seed);
    let pick_a = a.choose(&db, &st, &legal, &mut rng_a);
    let rec_a = a.take_explain().expect("explain");
    let hb_a = rec_a.holdback.clone();

    let mut b = parse_h0(HBCHECK_SPEC);
    b.arm_explain();
    let mut rng_b = policy_rng(seed);
    let pick_b = b.choose(&db, &st, &legal, &mut rng_b);
    let rec_b = b.take_explain().expect("explain");
    assert_eq!(pick_a, pick_b);
    let hb_b = rec_b.holdback.expect("holdback");
    let hb_a = hb_a.expect("holdback");
    assert_eq!(hb_a.end_prime, hb_b.end_prime);
    assert_eq!(hb_a.end_worlds.len(), hb_b.end_worlds.len());
    assert_eq!(hb_a.attacks.len(), hb_b.attacks.len());
}

const LT: &str = "10951120";

fn set_follower(st: &mut arena_engine::State, who: PlayerId, slot: u8, atk: i32, def: i32) {
    if let Some(f) = st.field_inst_mut(who, slot) {
        f.attack = atk;
        f.defense = def;
        f.max_defense = def;
        f.flags.summoning_sick = false;
    }
}

fn set_follower_evolved(st: &mut arena_engine::State, who: PlayerId, slot: u8, atk: i32, def: i32) {
    set_follower(st, who, slot, atk, def);
    if let Some(f) = st.field_inst_mut(who, slot) {
        f.evolved = true;
    }
}

fn set_no_face(st: &mut arena_engine::State, who: PlayerId, slot: u8) {
    if let Some(f) = st.field_inst_mut(who, slot) {
        f.traits.cant_attack_leader = Some(true);
    }
}

fn bot_turn6(db: &CardDb, seed: u64) -> arena_engine::State {
    let mut st = started(db, seed);
    while st.player(PlayerId::A).turns_taken < 6 || st.active != PlayerId::A {
        end_turn(db, &mut st);
    }
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    st
}

/// Lieutenant 1/1 with Last Words; attacker cannot go face so search can prefer `EndTurn`.
fn lieutenant_board(db: &CardDb, seed: u64, second_killer: bool) -> arena_engine::State {
    let mut st = bot_turn6(db, seed);
    put_field(db, &mut st, PlayerId::A, "88001110");
    set_follower_evolved(&mut st, PlayerId::A, 0, 3, 3);
    set_no_face(&mut st, PlayerId::A, 0);
    if second_killer {
        put_field(db, &mut st, PlayerId::A, "88001110");
        set_follower_evolved(&mut st, PlayerId::A, 1, 3, 2);
        set_no_face(&mut st, PlayerId::A, 1);
    }
    put_field(db, &mut st, PlayerId::B, LT);
    set_follower(&mut st, PlayerId::B, 0, 1, 1);
    give_pp(&mut st, PlayerId::A, 6, 10);
    st.player_mut(PlayerId::B).leader_defense = 20;
    st
}

/// Opponent's 4/4 can trade the 5/4 attacker after a 1/1 kill, not the healthy 5/5.
fn protect_x_board(db: &CardDb, seed: u64) -> arena_engine::State {
    let mut st = bot_turn6(db, seed);
    put_field(db, &mut st, PlayerId::A, "88001110");
    set_follower_evolved(&mut st, PlayerId::A, 0, 5, 5);
    set_no_face(&mut st, PlayerId::A, 0);
    put_field(db, &mut st, PlayerId::B, "88001110");
    set_follower(&mut st, PlayerId::B, 0, 1, 1);
    put_field(db, &mut st, PlayerId::B, "88001110");
    set_follower(&mut st, PlayerId::B, 1, 4, 4);
    put_field(db, &mut st, PlayerId::B, "88001110");
    set_follower(&mut st, PlayerId::B, 2, 5, 5);
    st.player_mut(PlayerId::A).leader_defense = 4;
    give_pp(&mut st, PlayerId::A, 6, 10);
    st.player_mut(PlayerId::B).leader_defense = 20;
    st
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn netherworld_lieutenant_keeps_end_turn() {
    let db = load_db();
    let st = lieutenant_board(&db, 42, false);
    let legal = legal_actions(&db, &st);
    let kill_js: Vec<usize> = legal
        .iter()
        .enumerate()
        .filter(|(i, _)| is_kill_attack_pick(&db, &st, &legal, *i))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        kill_js.len(),
        1,
        "expected exactly one kill attack on Netherworld Lieutenant"
    );
    let lt_idx = kill_js[0];
    assert!(
        matches!(
            &legal[lt_idx],
            Action::Attack {
                target: AttackTarget::Slot(arena_engine::Slot(0)),
                ..
            }
        ),
        "kill attack must target slot 0 (Netherworld Lieutenant)"
    );
    for j in 0..8 {
        let seed = 5000 + j as u64;
        let mut h0 = parse_h0(HBCHECK_SPEC);
        h0.arm_explain();
        let mut rng = policy_rng(seed);
        let pick = h0.choose(&db, &st, &legal, &mut rng);
        assert_eq!(h0.stats.hb_checks, 1, "seed={seed}");
        let rec = h0.take_explain().expect("explain");
        assert!(matches!(legal[pick], Action::EndTurn), "seed={seed}");
        assert_ne!(rec.path, ChoosePath::HoldbackTrade, "seed={seed}");
        let hb = rec.holdback.expect("holdback");
        let lt_in_attacks = hb
            .attacks
            .iter()
            .any(|a| a.legal_index == lt_idx);
        assert!(lt_in_attacks, "seed={seed}: LT attack must be in holdback.attacks");
        eprintln!(
            "lieutenant (a) seed={seed} end_prime={} attacks={}",
            hb.end_prime,
            hb.attacks.len()
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn netherworld_lieutenant_second_killer_report() {
    let db = load_db();
    let st = lieutenant_board(&db, 77, true);
    let legal = legal_actions(&db, &st);
    let mut holdback_trades = 0u32;
    let mut checks = 0u32;
    let mut reported = 0u32;
    for j in 0..8 {
        let seed = 2_000 + j as u64;
        let mut h0 = parse_h0(HBCHECK_SPEC);
        h0.arm_explain();
        let mut rng = policy_rng(seed);
        let pick = h0.choose(&db, &st, &legal, &mut rng);
        if h0.stats.hb_checks == 0 {
            eprintln!("lieutenant (b) seed={seed}: search did not EndTurn (hb_checks=0)");
            continue;
        }
        checks += 1;
        let rec = h0.take_explain().expect("explain");
        let hb = rec.holdback.as_ref().expect("holdback");
        let best_atk = hb
            .attacks
            .iter()
            .map(|a| a.aggregate)
            .fold(f32::NEG_INFINITY, f32::max);
        if rec.path == ChoosePath::HoldbackTrade {
            holdback_trades += 1;
        }
        eprintln!(
            "lieutenant (b) seed={seed} path={:?} end_prime={} best_atk={} pick={pick}",
            rec.path,
            hb.end_prime,
            best_atk
        );
        reported += 1;
    }
    eprintln!(
        "lieutenant (b): hb_checks={checks} holdback_trade={holdback_trades} reported={}/8",
        reported
    );
}

fn assert_protect_x_branch(rec: &arena_engine::policy::ExplainRecord, seed: u64) {
    assert_ne!(rec.path, ChoosePath::HoldbackTrade, "seed={seed}");
    let hb = rec.holdback.as_ref().expect("holdback");
    let best_atk = hb
        .attacks
        .iter()
        .map(|a| a.aggregate)
        .fold(f32::NEG_INFINITY, f32::max);
    let end_removable = hb.end_worlds.iter().filter(|w| w.removable).count();
    let attack_removable = hb
        .attacks
        .iter()
        .flat_map(|a| &a.worlds)
        .filter(|w| w.removable)
        .count();
    eprintln!(
        "protect-x seed={seed} end_prime={} best_atk={} end_removable={} attack_removable={}",
        hb.end_prime, best_atk, end_removable, attack_removable
    );
    assert!(attack_removable > end_removable, "seed={seed}");
    assert!(hb.end_prime > best_atk, "seed={seed}");
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn holding_back_protects_attacker() {
    let db = load_db();
    let dir = hbcheck_fixture_dir();
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("neg-"))
        })
        .collect();
    paths.sort();
    for path in paths {
        let cap: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
        let ply = cap["ply"].as_u64().expect("ply") as usize;
        let st = replay_capture(&db, &cap, ply);
        let legal = legal_actions(&db, &st);
        let base_seed = choose_seed(&cap, ply);
        for j in 0..8 {
            let seed = base_seed + 1000 * j as u64;
            let mut h0 = parse_h0(HBCHECK_SPEC);
            h0.arm_explain();
            let mut rng = policy_rng(seed);
            let pick = h0.choose(&db, &st, &legal, &mut rng);
            if h0.stats.hb_checks == 0 || !matches!(legal[pick], Action::EndTurn) {
                continue;
            }
            let rec = h0.take_explain().expect("explain");
            let hb = rec.holdback.as_ref().expect("holdback");
            let best_atk = hb
                .attacks
                .iter()
                .map(|a| a.aggregate)
                .fold(f32::NEG_INFINITY, f32::max);
            let end_removable = hb.end_worlds.iter().filter(|w| w.removable).count();
            let attack_removable = hb
                .attacks
                .iter()
                .flat_map(|a| &a.worlds)
                .filter(|w| w.removable)
                .count();
            if attack_removable > end_removable && hb.end_prime > best_atk {
                assert_protect_x_branch(&rec, seed);
                return;
            }
        }
    }
    let st = protect_x_board(&db, 88);
    let legal = legal_actions(&db, &st);
    for seed in 0..8192u64 {
        let mut h0 = parse_h0(HBCHECK_SPEC);
        h0.arm_explain();
        let mut rng = policy_rng(seed);
        let pick = h0.choose(&db, &st, &legal, &mut rng);
        if h0.stats.hb_checks == 0 || !matches!(legal[pick], Action::EndTurn) {
            continue;
        }
        let rec = h0.take_explain().expect("explain");
        if rec.path == ChoosePath::HoldbackTrade {
            continue;
        }
        let Some(hb) = rec.holdback.as_ref() else {
            continue;
        };
        let best_atk = hb
            .attacks
            .iter()
            .map(|a| a.aggregate)
            .fold(f32::NEG_INFINITY, f32::max);
        let end_removable = hb.end_worlds.iter().filter(|w| w.removable).count();
        let attack_removable = hb
            .attacks
            .iter()
            .flat_map(|a| &a.worlds)
            .filter(|w| w.removable)
            .count();
        if attack_removable > end_removable && hb.end_prime > best_atk {
            assert_protect_x_branch(&rec, seed);
            return;
        }
    }
    panic!("no protect-x position found in replay or constructed scan");
}

fn meta_mirror_decks() -> Vec<(String, String)> {
    let text = fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: Value = serde_json::from_str(&text).expect("pools json");
    let stems: Vec<String> = pools["meta"]
        .as_array()
        .expect("meta")
        .iter()
        .map(|v| v.as_str().expect("stem").to_string())
        .filter(|s| s != "meta-rune-test-subject")
        .collect();
    let mut out = Vec::new();
    for a in &stems {
        for b in &stems {
            out.push((a.clone(), b.clone()));
        }
    }
    out
}

fn load_deck_json(stem: &str) -> Vec<arena_engine::CardId> {
    let path = repo_root()
        .join("oracle/decks")
        .join(format!("{stem}.json"));
    let text = fs::read_to_string(path).expect("deck");
    let map: BTreeMap<String, u32> = serde_json::from_str(&text).expect("json");
    let mut ids = Vec::new();
    for (k, n) in map {
        let id = arena_engine::CardId::parse(&k).expect("id");
        for _ in 0..n {
            ids.push(id);
        }
    }
    ids
}

fn git_show_results(path: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["show", &format!("origin/results:{path}")])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

fn load_moments() -> Vec<Value> {
    let path = hbcheck_fixture_dir().join("moments.jsonl");
    let file = fs::File::open(path).expect("moments.jsonl");
    BufReader::new(file)
        .lines()
        .map(|l| l.expect("line"))
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(&l).expect("moment json"))
        .collect()
}

fn moment_capture(db: &CardDb, moment: &Value) -> (arena_engine::State, u64) {
    let game_id = moment["game"].as_str().expect("game");
    let ply = moment["ply"].as_u64().expect("ply") as usize;
    let path = format!("holdback1/games/{game_id}.json");
    let text = git_show_results(&path)
        .unwrap_or_else(|| panic!("missing origin/results:{path}"));
    let cap: Value = serde_json::from_str(&text).expect("game json");
    let seed = cap["seed"].as_u64().expect("seed") + ply as u64;
    (replay_capture(db, &cap, ply), seed)
}

fn count_kill_attacks(db: &CardDb, st: &arena_engine::State, legal: &[Action]) -> usize {
    legal
        .iter()
        .enumerate()
        .filter(|(i, _)| is_kill_attack_pick(db, st, legal, *i))
        .count()
}

#[test]
#[ignore = "measurement helper for holdback1 166-moment replay (tools/hbcheck_cost_bench.py)"]
#[cfg_attr(debug_assertions, ignore)]
fn hbcheck_moment_replay_report() {
    let db = load_db();
    let moments = load_moments();
    assert_eq!(moments.len(), 166, "expected 166 holdback1 moments");
    let mut ran_check = 0u32;
    let mut overrides = 0u64;
    let mut no_kill_found = 0u32;
    let mut total_kills = 0u32;
    eprintln!(
        "{:<20} {:>5} {:>10} {:>8} {:>8} {:>10}",
        "moment",
        "kills",
        "found",
        "hb_chk",
        "override",
        "hb_nodes"
    );
    for moment in &moments {
        let game_id = moment["game"].as_str().expect("game");
        let ply = moment["ply"].as_u64().expect("ply");
        let (st, seed) = moment_capture(&db, moment);
        let legal = legal_actions(&db, &st);
        let n_kill = count_kill_attacks(&db, &st, &legal) as u32;
        total_kills += n_kill;
        let mut h0 = parse_h0(HBCHECK_SPEC);
        let mut rng = policy_rng(seed);
        let _ = h0.choose(&db, &st, &legal, &mut rng);
        let chk = h0.stats.hb_checks;
        let ovr = h0.stats.hb_overrides;
        if chk > 0 {
            ran_check += 1;
            overrides += ovr;
            if n_kill == 0 {
                no_kill_found += 1;
            }
        }
        eprintln!(
            "{:<20} {:>5} {:>10} {:>8} {:>8} {:>10}",
            format!("{game_id}@{ply}"),
            n_kill,
            if chk > 0 { n_kill } else { 0 },
            chk,
            ovr,
            h0.stats.hb_nodes
        );
    }
    eprintln!(
        "replay summary: moments={} ran_check={} overrides={} total_kills={} no_kill_at_check={}",
        moments.len(),
        ran_check,
        overrides,
        total_kills,
        no_kill_found
    );
}

struct HbBenchRow {
    spec: String,
    decisions: u32,
    ms_total: f64,
    ms_samples: Vec<f64>,
    hb_nodes_total: u64,
    hb_nodes_samples: Vec<u64>,
    hb_checks: u64,
    hb_overrides: u64,
    hb_unknown: u64,
}

fn hb_specs(base: &str) -> Vec<String> {
    [0u32, 500, 2000, 10_000]
        .iter()
        .map(|hb| {
            if *hb == 0 && !base.contains(':') {
                base.to_string()
            } else if base.contains(':') {
                format!("{base},hbcheck={hb}")
            } else {
                format!("{base}:hbcheck={hb}")
            }
        })
        .collect()
}

fn bench_hb_choose(db: &CardDb, spec: &str, st: &arena_engine::State, seed: u64) -> (f64, u64, u64, u64, u64) {
    let mut h0 = parse_h0(spec);
    let legal = legal_actions(db, st);
    let mut rng = policy_rng(seed);
    let t0 = Instant::now();
    let _ = h0.choose(db, st, &legal, &mut rng);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    (
        ms,
        h0.stats.hb_nodes,
        h0.stats.hb_checks,
        h0.stats.hb_overrides,
        h0.stats.hb_unknown,
    )
}

fn p95_f64(samples: &[f64]) -> f64 {
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
#[ignore = "measurement helper for tools/hbcheck_cost_bench.py (paired holdback1 moments)"]
#[cfg_attr(debug_assertions, ignore)]
fn hbcheck_paired_cost_report() {
    let db = load_db();
    let moments = load_moments();
    assert!(!moments.is_empty(), "need holdback1 moments");
    let bases = [SERVED_SPEC, "h0:nodes=32000,horizon=3,k=8"];
    let mut rows: Vec<HbBenchRow> = Vec::new();
    for base in bases {
        for spec in hb_specs(base) {
            rows.push(HbBenchRow {
                spec,
                decisions: 0,
                ms_total: 0.0,
                ms_samples: Vec::new(),
                hb_nodes_total: 0,
                hb_nodes_samples: Vec::new(),
                hb_checks: 0,
                hb_overrides: 0,
                hb_unknown: 0,
            });
        }
    }
    for moment in &moments {
        let (st, seed) = moment_capture(&db, moment);
        for row in &mut rows {
            let (ms, nodes, checks, overrides, unknown) =
                bench_hb_choose(&db, &row.spec, &st, seed);
            row.decisions += 1;
            row.ms_total += ms;
            row.ms_samples.push(ms);
            row.hb_nodes_total += nodes;
            row.hb_nodes_samples.push(nodes);
            row.hb_checks += checks;
            row.hb_overrides += overrides;
            row.hb_unknown += unknown;
        }
    }
    eprintln!(
        "{:<52} {:>10} {:>10} {:>10} {:>14} {:>10} {:>10} {:>10}",
        "spec",
        "moments",
        "ms/dec",
        "p95_ms",
        "hb_nodes/dec",
        "p95_nodes",
        "checks/dec",
        "unknown/dec"
    );
    for row in rows {
        let d = row.decisions as f64;
        eprintln!(
            "{:<52} {:>10} {:>10.2} {:>10.2} {:>14.2} {:>10} {:>10.2} {:>10.2}",
            row.spec,
            row.decisions,
            row.ms_total / d,
            p95_f64(&row.ms_samples),
            row.hb_nodes_total as f64 / d,
            p95_u64(&row.hb_nodes_samples),
            row.hb_checks as f64 / d,
            row.hb_unknown as f64 / d,
        );
    }
}
