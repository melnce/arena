//! H0 held-back check (`hbcheck`): spec round-trip, identity at default off,
//! owner Feline fixture, positive/negative holdback moments, determinism,
//! and no-change when the check does not run.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

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
    if st.field_inst(opp, slot.0).is_none() {
        return false;
    }
    let mut s = st.clone();
    if apply(db, &mut s, a.clone()).is_err() {
        return false;
    }
    s.field_inst(opp, slot.0).is_none()
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

fn load_fixture(name: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(hbcheck_fixture_dir().join(name)).expect("read"))
        .expect("json")
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
        if rec.path == ChoosePath::HoldbackTrade {
            hb_hits += 1;
            assert!(
                is_kill_attack_pick(&db, &st, &legal, pick),
                "holdback pick must be kill attack"
            );
        }
        let mut plain = parse_h0(SERVED_SPEC);
        let mut rng_p = policy_rng(seed);
        let plain_pick = plain.choose(&db, &st, &legal, &mut rng_p);
        if matches!(legal[plain_pick], Action::EndTurn) {
            plain_end += 1;
        }
    }
    eprintln!(
        "owner-feline: hb holdback_trade {}/12, plain EndTurn {}/12",
        hb_hits, plain_end
    );
    // Symmetric End′/A′ with bot finish: fewer overrides than the asymmetric brief
    // estimate, but the owner moment should still flip on a clear majority of seeds.
    assert!(hb_hits >= 4, "expected >= 4/12 holdback overrides");
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
    set_follower_evolved(&mut st, PlayerId::A, 0, 1, 1);
    set_no_face(&mut st, PlayerId::A, 0);
    if second_killer {
        put_field(db, &mut st, PlayerId::A, "88001110");
        set_follower_evolved(&mut st, PlayerId::A, 1, 2, 2);
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
    let cap = load_fixture("neg-play-100177-ply0007.json");
    let ply = cap["ply"].as_u64().expect("ply") as usize;
    let st = replay_capture(&db, &cap, ply);
    let legal = legal_actions(&db, &st);
    let base_seed = choose_seed(&cap, ply);
    let kill_js: Vec<usize> = legal
        .iter()
        .enumerate()
        .filter(|(i, _)| is_kill_attack_pick(&db, &st, &legal, *i))
        .map(|(i, _)| i)
        .collect();
    assert!(
        !kill_js.is_empty(),
        "expected at least one kill attack on Netherworld Lieutenant"
    );
    for j in 0..8 {
        let seed = base_seed + 1000 * j as u64;
        let mut h0 = parse_h0(HBCHECK_SPEC);
        h0.arm_explain();
        let mut rng = policy_rng(seed);
        let pick = h0.choose(&db, &st, &legal, &mut rng);
        assert_eq!(h0.stats.hb_checks, 1, "seed={seed}");
        let rec = h0.take_explain().expect("explain");
        assert!(matches!(legal[pick], Action::EndTurn), "seed={seed}");
        assert_ne!(rec.path, ChoosePath::HoldbackTrade, "seed={seed}");
        let hb = rec.holdback.expect("holdback");
        let best_atk = hb
            .attacks
            .iter()
            .map(|a| a.aggregate)
            .fold(f32::NEG_INFINITY, f32::max);
        eprintln!(
            "lieutenant (a) seed={seed} end_prime={} best_atk={}",
            hb.end_prime, best_atk
        );
        assert!(
            hb.end_prime
                >= hb
                    .attacks
                    .iter()
                    .map(|a| a.aggregate)
                    .fold(f32::NEG_INFINITY, f32::max),
            "seed={seed}"
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn netherworld_lieutenant_second_killer_report() {
    let db = load_db();
    let st = lieutenant_board(&db, 77, true);
    let mut holdback_trades = 0u32;
    let mut checks = 0u32;
    for j in 0..8 {
        let seed = 2_000 + j as u64;
        let mut h0 = parse_h0(HBCHECK_SPEC);
        h0.arm_explain();
        let legal = legal_actions(&db, &st);
        let mut rng = policy_rng(seed);
        let pick = h0.choose(&db, &st, &legal, &mut rng);
        if h0.stats.hb_checks > 0 {
            checks += 1;
            let rec = h0.take_explain().expect("explain");
            if rec.path == ChoosePath::HoldbackTrade {
                holdback_trades += 1;
            }
            if let Some(hb) = rec.holdback {
                eprintln!(
                    "lieutenant (b) seed={seed} path={:?} end_prime={} pick={pick}",
                    rec.path, hb.end_prime
                );
            }
        }
    }
    eprintln!("lieutenant (b): hb_checks={checks} holdback_trade={holdback_trades}/8");
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
