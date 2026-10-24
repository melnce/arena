//! H0 opponent next-turn lethal glance after greedy reply (`onext`).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use arena_engine::{
    acting_player, apply, apply_neutral, legal_actions, new_game, policy_rng, to_neutral,
    trace::fnv1a64, world, Action, AnyPolicy, First, GameConfig, H0Job, JobStep, Phase, PlayerId,
    Policy, State, H0,
};
use arena_engine::{MAX_ACTIONS, MAX_TURNS};
use serde_json::Value;

mod common;
use common::*;

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

const GATE_SEEDS: [u64; 8] = [11, 23, 37, 41, 53, 67, 79, 97];

const DEFAULT_OKILL_GATE_FPS: [u64; 8] = [
    0x2246_9fdb_d814_7321,
    0xd8c3_dcd1_2502_4f7f,
    0x649a_13b2_5e6f_3115,
    0xd0a1_8275_06c2_801e,
    0x4b8c_36bb_af3f_9a0e,
    0x081e_1488_1d35_a647,
    0x64a9_a3ca_860e_4884,
    0x41ad_f866_2960_4f21,
];

const OKILL7_GATE_FPS: [u64; 8] = [
    0x30c5_3aea_2056_8fa5,
    0x75bd_d260_1c71_039b,
    0x421d_14c6_1a80_f253,
    0x441c_f92b_de4f_a54d,
    0x8798_4655_cb16_36b9,
    0xd42b_48df_9776_92ac,
    0x9dca_b3cf_21fc_0b69,
    0x4b0e_bb6a_37da_9837,
];

const SERVED_SPEC: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1";
const SERVED_GATE_SEEDS: [u64; 3] = [11, 23, 37];
const SERVED_GATE_FPS: [u64; 3] = [
    0xe245_2508_3e83_6195,
    0xf56b_c488_8418_8f1e,
    0x240a_8a8f_d3c7_5241,
];

fn meta_deck_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .collect()
}

fn load_meta_deck(stem: &str) -> Vec<arena_engine::CardId> {
    load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")))
}

fn action_fingerprint(
    db: &arena_engine::CardDb,
    spec: &str,
    seed: u64,
    deck_a: &[arena_engine::CardId],
) -> u64 {
    let deck_b = load_meta_deck("meta-sword-rally");
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut a = parse_h0(spec);
    let mut b = parse_h0(spec);
    let mut rng = policy_rng(seed);
    let mut serialized = Vec::new();
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        let legal = legal_actions(db, &state);
        if legal.is_empty() {
            break;
        }
        let me = acting_player(&state);
        let idx = match me {
            PlayerId::A => a.choose(db, &state, &legal, &mut rng),
            PlayerId::B => b.choose(db, &state, &legal, &mut rng),
        };
        let action = legal[idx.min(legal.len().saturating_sub(1))].clone();
        serialized.push(serde_json::to_string(&to_neutral(&state, &action)).unwrap());
        apply(db, &mut state, action).expect("apply");
    }
    let count = serialized.len() as u64;
    let mut bytes = Vec::with_capacity(8 + serialized.iter().map(|s| s.len()).sum::<usize>());
    bytes.extend_from_slice(&count.to_le_bytes());
    for s in &serialized {
        bytes.extend_from_slice(s.as_bytes());
    }
    fnv1a64(&bytes)
}

fn collect_opp_reply_states(
    db: &arena_engine::CardDb,
    game_seeds: &[u64],
) -> Vec<(arena_engine::State, PlayerId)> {
    let stems = meta_deck_stems();
    let deck_b = load_meta_deck("meta-sword-rally");
    let mut out = Vec::new();
    for (gi, seed) in game_seeds.iter().enumerate() {
        let deck_a = load_meta_deck(&stems[gi % stems.len()]);
        let mut state = new_game(
            db,
            GameConfig {
                seed: *seed,
                deck_a,
                deck_b: deck_b.clone(),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        let mut a = parse_h0_v1("h0:nodes=200");
        let mut b = parse_h0_v1("h0:nodes=200");
        let mut rng = policy_rng(*seed);
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) && out.len() < 300 {
            let legal = legal_actions(db, &state);
            if legal.is_empty() {
                break;
            }
            let me = acting_player(&state);
            let idx = match me {
                PlayerId::A => a.choose(db, &state, &legal, &mut rng),
                PlayerId::B => b.choose(db, &state, &legal, &mut rng),
            };
            let action = legal[idx.min(legal.len().saturating_sub(1))].clone();
            if matches!(action, Action::EndTurn) {
                apply(db, &mut state, action).expect("apply");
                if state.winner.is_none()
                    && !matches!(state.phase, Phase::Terminal)
                    && acting_player(&state) == me.opponent()
                {
                    out.push((state.clone(), me));
                }
                continue;
            }
            apply(db, &mut state, action).expect("apply");
        }
    }
    out
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn onext_default_play_unchanged_gate() {
    let db = load_db();
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, "h0", *seed, &deck);
        assert_eq!(got, DEFAULT_OKILL_GATE_FPS[i], "h0 seed={seed}");
        let z = action_fingerprint(&db, "h0:onext=0", *seed, &deck);
        assert_eq!(z, got, "onext=0 seed={seed}");

        let ok7 = action_fingerprint(&db, "h0:okill=7", *seed, &deck);
        assert_eq!(ok7, OKILL7_GATE_FPS[i], "okill=7 seed={seed}");
        let ok7z = action_fingerprint(&db, "h0:okill=7,onext=0", *seed, &deck);
        assert_eq!(ok7z, ok7, "okill=7,onext=0 seed={seed}");
    }
    for (i, seed) in SERVED_GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let served = action_fingerprint(&db, SERVED_SPEC, *seed, &deck);
        assert_eq!(served, SERVED_GATE_FPS[i], "served seed={seed}");
        let served_z = action_fingerprint(&db, &format!("{SERVED_SPEC},onext=0"), *seed, &deck);
        assert_eq!(served_z, served, "served,onext=0 seed={seed}");
    }
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let d1 = action_fingerprint(&db, "h0:odepth=1", *seed, &deck);
        let d1o = action_fingerprint(&db, "h0:odepth=1,onext=40", *seed, &deck);
        assert_eq!(d1, d1o, "odepth=1 ignores onext seed={seed}");
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn onext_never_above_reply_line() {
    let db = load_db();
    let seeds: Vec<u64> = (1u64..40).collect();
    let states = collect_opp_reply_states(&db, &seeds);
    assert!(
        states.len() >= 200,
        "need >=200 reply states, got {}",
        states.len()
    );
    let mut lower0 = 0usize;
    let mut lower1 = 0usize;
    for (st, me) in states.iter().take(250) {
        let mut g0 = parse_h0_v1("h0:value=v0,nodes=1000000,onext=0,oopen=0");
        let mut g1 = parse_h0_v1("h0:value=v0,nodes=1000000,onext=40,oopen=0");
        let mut g2 = parse_h0_v1("h0:value=v0,nodes=1000000,onext=40,oopen=1");
        let v0 = g0.opponent_value(&db, st, *me);
        let v1 = g1.opponent_value(&db, st, *me);
        let v2 = g2.opponent_value(&db, st, *me);
        assert!(v1 <= v0 + 1e-5, "onext above greedy: v0={v0} v1={v1}");
        assert!(v2 <= v1 + 1e-5, "onext+oopen above onext: v1={v1} v2={v2}");
        if v1 < v0 - 1e-5 {
            lower0 += 1;
        }
        if v2 < v1 - 1e-5 {
            lower1 += 1;
        }
    }
    let n = 250.min(states.len());
    eprintln!(
        "onext lower share: oopen=0 {:.3} oopen=1 {:.3}",
        lower0 as f64 / n as f64,
        lower1 as f64 / n as f64,
    );
    assert!(
        lower0 > 0,
        "onext=40 must be strictly lower in at least one state (oopen=0)"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn onext_counters() {
    let db = load_db();
    let seeds: [u64; 4] = [42, 99, 123, 456];
    let states = collect_opp_reply_states(&db, &seeds);
    let mut off = parse_h0_v1("h0:value=v0,nodes=1000000,onext=0");
    let mut on = parse_h0_v1("h0:value=v0,nodes=1000000,onext=40");
    for (st, me) in &states[..20.min(states.len())] {
        off.opponent_value(&db, st, *me);
        on.opponent_value(&db, st, *me);
    }
    assert_eq!(off.stats.opp_next_checks, 0);
    assert_eq!(off.stats.opp_next_found, 0);
    assert_eq!(off.stats.opp_next_nodes, 0);
    assert!(on.stats.opp_next_found <= on.stats.opp_next_checks);
    if on.stats.opp_next_checks > 0 {
        assert!(on.stats.opp_next_nodes > 0);
    }
    assert_eq!(
        off.stats.opp_lethal_checks, on.stats.opp_lethal_checks,
        "onext must not change opp_lethal_checks"
    );
}

const DEFAULT_STRONG: &str =
    "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=8,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=h0-value-v64";

const SERVED_STACK_8W: &str =
    "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40";

const SERVED_JOB_BASE: &str =
    "h0:nodes=8000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40";

struct ReviewMoment {
    review: &'static str,
    game: &'static str,
    ply: usize,
}

const REVIEW_MOMENTS: [ReviewMoment; 12] = [
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 27,
    },
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 45,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 69,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 40,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 27,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 52,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 91,
    },
    ReviewMoment {
        review: "review14",
        game: "12644260486301391828-d90f8eb2.json",
        ply: 15,
    },
    ReviewMoment {
        review: "review14",
        game: "12644260486301391828-d90f8eb2.json",
        ply: 35,
    },
    ReviewMoment {
        review: "review14",
        game: "6598261483642061665-d90f8eb2.json",
        ply: 60,
    },
    ReviewMoment {
        review: "review14",
        game: "14600367900189587136-d90f8eb2.json",
        ply: 55,
    },
    ReviewMoment {
        review: "review13",
        game: "4984932781931433298-b4973563.json",
        ply: 60,
    },
];

fn review_fixture(review: &str, name: &str) -> PathBuf {
    repo_root()
        .join("py/tests/fixtures")
        .join(review)
        .join("games")
        .join(name)
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

fn replay_capture(db: &arena_engine::CardDb, cap: &Value, n: usize) -> State {
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

fn policy_seed(cap: &Value, ply: usize) -> u64 {
    let actions = cap["actions"].as_array().expect("actions");
    let bot_steps = actions
        .iter()
        .take(ply)
        .any(|s| s.get("mulligan").is_some() || s.get("bot_value").is_some());
    if bot_steps {
        let base = cap["seed"].as_u64().expect("seed");
        let mut n = 0u64;
        for step in actions.iter().take(ply) {
            if step.get("mulligan").is_some() || step.get("bot_value").is_some() {
                n += 1;
            }
        }
        base + n
    } else {
        cap["seed"].as_u64().expect("seed") + ply as u64
    }
}

struct DecideOutcome {
    idx: usize,
    value_bits: Option<u32>,
    explain: Option<String>,
    nodes: u32,
}

fn decide_state(spec: &str, db: &arena_engine::CardDb, st: &State, seed: u64) -> DecideOutcome {
    let legal = legal_actions(db, st);
    let mut h0 = parse_h0(spec);
    h0.threads = 1;
    h0.arm_explain();
    let mut rng = policy_rng(seed);
    let idx = h0.choose(db, st, &legal, &mut rng);
    DecideOutcome {
        idx,
        value_bits: h0.last_value().map(f32::to_bits),
        explain: h0
            .take_explain()
            .map(|r| serde_json::to_string(&r).expect("explain json")),
        nodes: h0.stats.nodes as u32,
    }
}

fn candidate_aggregates(explain: &str) -> Vec<f32> {
    let v: Value = serde_json::from_str(explain).expect("explain json");
    v["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .map(|c| c["root_agg"].as_f64().expect("root_agg") as f32)
        .collect()
}

/// `onextleaf=1` shifted at least one `root_agg` vs keys-off, with no greedy next-turn win
/// (`opp_next_own_wins == 0`) on this decision.
fn onextleaf_value_shift_without_own_win(
    db: &arena_engine::CardDb,
    base: &str,
    with_leaf: &str,
    st: &State,
    seed: u64,
) -> bool {
    let mut h0 = parse_h0(with_leaf);
    h0.threads = 1;
    let legal = legal_actions(db, st);
    let mut rng = policy_rng(seed);
    h0.choose(db, st, &legal, &mut rng);
    if h0.stats.opp_next_own_wins != 0 {
        return false;
    }
    let off = decide_state(base, db, st, seed);
    let on = decide_state(with_leaf, db, st, seed);
    candidate_aggregates(off.explain.as_ref().expect("explain off"))
        != candidate_aggregates(on.explain.as_ref().expect("explain on"))
}

fn explain_has_end(explain: &str, end: &str) -> bool {
    let v: Value = serde_json::from_str(explain).expect("explain json");
    v["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .flat_map(|c| c["worlds"].as_array())
        .flatten()
        .any(|w| w.get("end").and_then(|e| e.as_str()) == Some(end))
}

fn load_deck_file(path: PathBuf) -> Vec<arena_engine::CardId> {
    let map: BTreeMap<String, u32> =
        serde_json::from_str(&fs::read_to_string(path).expect("deck")).expect("deck json");
    let mut ids = Vec::new();
    for (k, n) in map {
        let id = arena_engine::CardId::parse(&k).expect("card id");
        for _ in 0..n {
            ids.push(id);
        }
    }
    ids
}

struct SelfPlayCase {
    label: String,
    state: State,
    seed: u64,
}

fn meta_selfplay_cases(db: &arena_engine::CardDb) -> Vec<SelfPlayCase> {
    let spec = "h0";
    let mut out = Vec::new();
    for stem in meta_deck_stems() {
        if stem == "meta-rune-test-subject" {
            continue;
        }
        let deck = load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")));
        for g in 0u64..2 {
            let seed = 20261011u64
                .wrapping_add(g * 97)
                .wrapping_add(stem.len() as u64 * 131);
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
            let mut pol = parse_h0(spec);
            let mut rng = policy_rng(seed);
            let mut nact = 0u32;
            while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
                if state.turn > MAX_TURNS || nact >= MAX_ACTIONS {
                    break;
                }
                let legal = legal_actions(db, &state);
                if legal.is_empty() {
                    break;
                }
                if state.active == PlayerId::A
                    && state.turn >= 4
                    && matches!(state.phase, Phase::Main)
                {
                    let choose_seed = seed
                        .wrapping_add(u64::from(state.turn) * 97)
                        .wrapping_add(u64::from(nact) * 131);
                    out.push(SelfPlayCase {
                        label: format!("selfplay/{stem}/g{g}/t{}/n{}", state.turn, nact),
                        state: state.clone(),
                        seed: choose_seed,
                    });
                    if out.len() >= 40 {
                        return out;
                    }
                }
                let idx = pol.choose(db, &state, &legal, &mut rng);
                if apply(db, &mut state, legal[idx].clone()).is_err() {
                    break;
                }
                nact += 1;
            }
        }
    }
    out
}

fn serde_round<T: serde::Serialize + serde::de::DeserializeOwned>(v: &T) -> T {
    serde_json::from_str(&serde_json::to_string(v).unwrap()).unwrap()
}

fn drive_job(spec: &str, db: &arena_engine::CardDb, st: &State, seed: u64) -> DecideOutcome {
    let (mut job, mut step) = H0Job::start(db, st, spec, seed).unwrap_or_else(|e| panic!("{e}"));
    loop {
        match step {
            JobStep::Done { index, value } => {
                return DecideOutcome {
                    idx: index as usize,
                    value_bits: value.map(f32::to_bits),
                    explain: None,
                    nodes: job.choose_stats().nodes as u32,
                };
            }
            JobStep::Kill { roll } => {
                if roll {
                    let out = serde_round(
                        &arena_engine::kill_roll(db, st, spec, seed).expect("kill_roll"),
                    );
                    step = job.provide_kill_roll(db, out).expect("roll");
                } else {
                    let out =
                        serde_round(&arena_engine::kill_det(db, st, spec, seed).expect("kill_det"));
                    step = job.provide_kill_det(db, out).expect("det");
                }
            }
            JobStep::AwaitRoll => step = JobStep::Kill { roll: true },
            JobStep::Worlds { budgets } => {
                let k = budgets.len();
                let mut outs = Vec::with_capacity(k);
                for (r, budget) in budgets.iter().enumerate() {
                    outs.push(serde_round(
                        &world(db, st, spec, seed, r, *budget).expect("world"),
                    ));
                }
                step = job.finish(db, &outs).expect("finish");
            }
        }
    }
}

#[test]
fn onext_spec() {
    let again = AnyPolicy::parse_spec("h0:onext=40,oopen=1").unwrap();
    assert_eq!(again.spec(), "h0:oopen=1,onext=40");
    assert_eq!(AnyPolicy::parse_spec("h0:onext=0").unwrap().spec(), "h0");
    assert!(AnyPolicy::parse_spec("h0:onext=x").is_err());

    let win = AnyPolicy::parse_spec("h0:onext=40,onextwin=40").unwrap();
    assert_eq!(win.spec(), "h0:onext=40,onextwin=40");
    let leaf = AnyPolicy::parse_spec("h0:onext=40,onextleaf=1").unwrap();
    assert_eq!(leaf.spec(), "h0:onext=40,onextleaf=1");
    assert_eq!(
        AnyPolicy::parse_spec("h0:onext=40,onextleaf=0")
            .unwrap()
            .spec(),
        "h0:onext=40"
    );
    assert!(AnyPolicy::parse_spec("h0:onextleaf=2").is_err());
    assert!(AnyPolicy::parse_spec("h0:onextwin=x").is_err());

    let both = AnyPolicy::parse_spec("h0:onext=40,onextwin=40,onextleaf=1").unwrap();
    assert_eq!(both.spec(), "h0:onext=40,onextwin=40,onextleaf=1");
    assert_eq!(
        AnyPolicy::parse_spec(&both.spec()).unwrap().spec(),
        both.spec()
    );

    let parsed = AnyPolicy::parse_spec(DEFAULT_STRONG).unwrap();
    let spec = parsed.spec();
    assert!(!spec.contains("onextwin"));
    assert!(!spec.contains("onextleaf"));
    match parsed {
        AnyPolicy::H0(h) => {
            assert_eq!(h.onextwin, 0);
            assert!(!h.onextleaf);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn onext_own_keys_off_identity() {
    let db = load_recorded_db();
    let base = SERVED_STACK_8W;
    let explicit = format!("{base},onextwin=0,onextleaf=0");
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let st = replay_capture(&db, &cap, m.ply);
        let a = decide_state(base, &db, &st, seed);
        let b = decide_state(&explicit, &db, &st, seed);
        assert_eq!(b.idx, a.idx, "{}/{}@{}", m.review, m.game, m.ply);
        assert_eq!(b.value_bits, a.value_bits, "value");
        assert_eq!(b.explain, a.explain, "explain");
        assert_eq!(b.nodes, a.nodes, "nodes");
    }
    let cases = meta_selfplay_cases(&db);
    assert!(cases.len() >= 40, "self-play cases={}", cases.len());
    for case in &cases {
        let a = decide_state(base, &db, &case.state, case.seed);
        let b = decide_state(&explicit, &db, &case.state, case.seed);
        assert_eq!(b.idx, a.idx, "{}", case.label);
        assert_eq!(b.value_bits, a.value_bits, "value {}", case.label);
        assert_eq!(b.explain, a.explain, "explain {}", case.label);
        assert_eq!(b.nodes, a.nodes, "nodes {}", case.label);
    }
    for key in ["onextwin=40", "onextleaf=1"] {
        let with_key = format!("h0:{key}");
        let plain = decide_state("h0:onext=0", &db, &cases[0].state, cases[0].seed);
        let keyed = decide_state(&with_key, &db, &cases[0].state, cases[0].seed);
        assert_eq!(keyed.idx, plain.idx);
        assert_eq!(keyed.value_bits, plain.value_bits);
        assert_eq!(keyed.explain, plain.explain);
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn onextwin_acts_on_self_play_scan() {
    let db = load_recorded_db();
    let base = SERVED_STACK_8W;
    let with_win = format!("{base},onextwin=40");
    let mut scan_cases: Vec<(State, u64, String)> = Vec::new();
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let st = replay_capture(&db, &cap, m.ply);
        scan_cases.push((st, seed, format!("{}/{}@{}", m.review, m.game, m.ply)));
    }
    for case in meta_selfplay_cases(&db) {
        scan_cases.push((case.state.clone(), case.seed, case.label.clone()));
    }
    let mut hit: Option<(State, u64, String)> = None;
    for (st, seed, label) in scan_cases {
        let mut h0 = parse_h0(&with_win);
        h0.threads = 1;
        h0.arm_explain();
        let legal = legal_actions(&db, &st);
        let mut rng = policy_rng(seed);
        h0.choose(&db, &st, &legal, &mut rng);
        let explain = h0
            .take_explain()
            .map(|r| serde_json::to_string(&r).expect("json"));
        if h0.stats.opp_next_own_wins > 0
            && explain
                .as_ref()
                .is_some_and(|e| explain_has_end(e, "own_next"))
        {
            hit = Some((st, seed, label));
            break;
        }
    }
    let (state, seed, label) = hit.expect("need opp_next_own_wins with own_next in explain");
    let case = SelfPlayCase { label, state, seed };
    let off = decide_state(base, &db, &case.state, case.seed);
    let on = decide_state(&with_win, &db, &case.state, case.seed);
    assert!(explain_has_end(
        on.explain.as_ref().expect("explain"),
        "own_next"
    ));
    let agg_off = candidate_aggregates(off.explain.as_ref().expect("explain off"));
    let agg_on = candidate_aggregates(on.explain.as_ref().expect("explain on"));
    assert_eq!(agg_off.len(), agg_on.len());
    let mut raised = false;
    for (a, b) in agg_off.iter().zip(agg_on.iter()) {
        assert!(*b >= *a - 1e-5, "candidate aggregate lowered: {a} -> {b}");
        if *b > *a + 1e-5 {
            raised = true;
        }
    }
    assert!(
        raised,
        "onextwin must raise at least one candidate aggregate"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn onextleaf_acts_on_review_moments() {
    let db = load_recorded_db();
    let base = SERVED_STACK_8W;
    let with_leaf = format!("{base},onextleaf=1");
    let mut leaf_hits = 0u64;
    let mut agg_diff = false;
    let mut saw_next_leaf = false;
    let mut value_shift_on_review = false;
    for m in REVIEW_MOMENTS {
        let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
        let cap: Value = serde_json::from_str(&raw).expect("json");
        let seed = policy_seed(&cap, m.ply);
        let st = replay_capture(&db, &cap, m.ply);
        let mut h0 = parse_h0(&with_leaf);
        h0.threads = 1;
        let legal = legal_actions(&db, &st);
        let mut rng = policy_rng(seed);
        h0.choose(&db, &st, &legal, &mut rng);
        leaf_hits += h0.stats.opp_next_leaf;
        if onextleaf_value_shift_without_own_win(&db, base, &with_leaf, &st, seed) {
            value_shift_on_review = true;
        }
        let off = decide_state(base, &db, &st, seed);
        let on = decide_state(&with_leaf, &db, &st, seed);
        let a0 = candidate_aggregates(off.explain.as_ref().expect("explain off"));
        let a1 = candidate_aggregates(on.explain.as_ref().expect("explain on"));
        if a0 != a1 {
            agg_diff = true;
        }
        if let Some(exp) = &on.explain {
            if explain_has_end(exp, "next_leaf") || explain_has_end(exp, "own_next") {
                saw_next_leaf = true;
            }
            let v: Value = serde_json::from_str(exp).expect("json");
            for c in v["candidates"].as_array().expect("candidates") {
                for w in c["worlds"].as_array().expect("worlds") {
                    if w.get("end").and_then(|e| e.as_str()) == Some("opp_next") {
                        let val = w["clamped"].as_f64().expect("clamped");
                        assert!(val <= -40.0 + 1e-4, "opp_next cap: {val}");
                    }
                }
            }
        }
    }
    let mut value_shift_without_own_win = value_shift_on_review;
    if !value_shift_on_review {
        for case in meta_selfplay_cases(&db) {
            if onextleaf_value_shift_without_own_win(&db, base, &with_leaf, &case.state, case.seed)
            {
                value_shift_without_own_win = true;
                assert!(
                    onextleaf_value_shift_without_own_win(
                        &db,
                        base,
                        &with_leaf,
                        &case.state,
                        case.seed,
                    ),
                    "pinned self-play case {} seed={} must qualify",
                    case.label,
                    case.seed,
                );
                break;
            }
        }
    }
    // Pinned review moment (YB-1): first in `REVIEW_MOMENTS` that shifts `root_agg` with
    // `opp_next_own_wins == 0` on the served stack with `onextleaf=1`.
    let pin = &REVIEW_MOMENTS[0];
    let pin_raw = fs::read_to_string(review_fixture(pin.review, pin.game)).expect("review");
    let pin_cap: Value = serde_json::from_str(&pin_raw).expect("json");
    let pin_seed = policy_seed(&pin_cap, pin.ply);
    let pin_st = replay_capture(&db, &pin_cap, pin.ply);
    assert!(
        onextleaf_value_shift_without_own_win(&db, base, &with_leaf, &pin_st, pin_seed),
        "pinned review {}/{}@{} must shift root_agg without opp_next_own_wins",
        pin.review,
        pin.game,
        pin.ply
    );
    assert!(leaf_hits > 0, "opp_next_leaf must be > 0 on review moments");
    assert!(
        agg_diff,
        "onextleaf must change a root aggregate on review moments"
    );
    assert!(
        saw_next_leaf,
        "at least one explain line must end next_leaf or own_next"
    );
    assert!(
        value_shift_without_own_win,
        "need onextleaf root_agg shift with opp_next_own_wins==0 (review or self-play scan)"
    );
}

#[test]
#[ignore = "release identity; slow.yml subset"]
fn onext_own_keys_job_matches_choose() {
    let db = load_recorded_db();
    for extra in [",onextwin=40", ",onextleaf=1"] {
        let spec = format!("{SERVED_JOB_BASE}{extra}");
        for m in REVIEW_MOMENTS {
            let raw = fs::read_to_string(review_fixture(m.review, m.game)).expect("review");
            let cap: Value = serde_json::from_str(&raw).expect("json");
            let seed = policy_seed(&cap, m.ply);
            let st = replay_capture(&db, &cap, m.ply);
            let choose = decide_state(&spec, &db, &st, seed);
            let job = drive_job(&spec, &db, &st, seed);
            assert_eq!(
                job.idx, choose.idx,
                "{spec} {}/{}@{}",
                m.review, m.game, m.ply
            );
            assert_eq!(job.value_bits, choose.value_bits, "value");
        }
    }
}
