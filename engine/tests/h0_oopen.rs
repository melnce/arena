//! H0 opponent reply opening search (`oopen`).

use arena_engine::{
    acting_player, apply, legal_actions, new_game, policy_rng, to_neutral, trace::fnv1a64, Action,
    AnyPolicy, CardDb, First, GameConfig, Phase, PlayerId, Policy, H0,
};
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

fn action_fingerprint(db: &CardDb, spec: &str, seed: u64, deck_a: &[arena_engine::CardId]) -> u64 {
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
    db: &CardDb,
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
fn oopen_default_play_unchanged_gate() {
    let db = load_db();
    let stems = meta_deck_stems();
    for (i, seed) in GATE_SEEDS.iter().enumerate() {
        let deck = load_meta_deck(&stems[i]);
        let got = action_fingerprint(&db, "h0", *seed, &deck);
        assert_eq!(
            got, DEFAULT_OKILL_GATE_FPS[i],
            "h0 fingerprint seed={seed} deck={}",
            stems[i]
        );
        let z = action_fingerprint(&db, "h0:oopen=0", *seed, &deck);
        assert_eq!(z, got, "oopen=0 seed={seed}");
        let ok7 = action_fingerprint(&db, "h0:okill=7", *seed, &deck);
        assert_eq!(
            ok7, OKILL7_GATE_FPS[i],
            "okill=7 fingerprint seed={seed} deck={}",
            stems[i]
        );
        let ok7z = action_fingerprint(&db, "h0:okill=7,oopen=0", *seed, &deck);
        assert_eq!(ok7z, ok7, "okill=7,oopen=0 seed={seed}");
        if let Some(j) = SERVED_GATE_SEEDS.iter().position(|&s| s == *seed) {
            let served = action_fingerprint(&db, SERVED_SPEC, *seed, &deck);
            assert_eq!(
                served, SERVED_GATE_FPS[j],
                "served fingerprint seed={seed} deck={}",
                stems[i]
            );
            let served_z = action_fingerprint(&db, &format!("{SERVED_SPEC},oopen=0"), *seed, &deck);
            assert_eq!(served_z, served, "served,oopen=0 seed={seed}");
        }
        let d1 = action_fingerprint(&db, "h0:odepth=1", *seed, &deck);
        let d1o = action_fingerprint(&db, "h0:odepth=1,oopen=2", *seed, &deck);
        assert_eq!(d1, d1o, "odepth=1 ignores oopen seed={seed}");
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn oopen_never_above_greedy_line() {
    let db = load_db();
    let seeds: Vec<u64> = (1u64..40).collect();
    let states = collect_opp_reply_states(&db, &seeds);
    assert!(
        states.len() >= 200,
        "expected >=200 opponent-turn states, got {}",
        states.len()
    );
    let mut lower_1 = 0usize;
    let mut lower_2 = 0usize;
    let mut strict_1 = false;
    let mut strict_2 = false;
    for (st, me) in states.iter().take(250) {
        let mut g0 = parse_h0_v1("h0:value=v0,nodes=1000000,oopen=0");
        let mut g1 = parse_h0_v1("h0:value=v0,nodes=1000000,oopen=1");
        let mut g2 = parse_h0_v1("h0:value=v0,nodes=1000000,oopen=2");
        let v0 = g0.opponent_value(&db, st, *me);
        let v1 = g1.opponent_value(&db, st, *me);
        let v2 = g2.opponent_value(&db, st, *me);
        assert!(v1 <= v0 + 1e-5, "oopen=1 above greedy: v0={v0} v1={v1}");
        assert!(v2 <= v1 + 1e-5, "oopen=2 above oopen=1: v1={v1} v2={v2}");
        if v1 < v0 - 1e-5 {
            lower_1 += 1;
        }
        if v2 < v1 - 1e-5 {
            lower_2 += 1;
        }
        if v1 < v0 - 0.01 {
            strict_1 = true;
        }
        if v2 < v1 - 0.01 {
            strict_2 = true;
        }
    }
    eprintln!(
        "oopen lower share: oopen=1 {:.3} oopen=2 {:.3}",
        lower_1 as f64 / 250.0,
        lower_2 as f64 / 250.0
    );
    assert!(
        strict_1,
        "oopen=1 must be >0.01 lower in at least one state"
    );
    assert!(
        strict_2,
        "oopen=2 must be >0.01 lower in at least one state"
    );
}

#[test]
fn oopen_counters() {
    let db = load_db();
    let seeds: Vec<u64> = (100u64..110).collect();
    let states = collect_opp_reply_states(&db, &seeds);
    assert!(!states.is_empty());
    let mut off = parse_h0_v1("h0:value=v0,nodes=1000000,oopen=0");
    let mut on = parse_h0_v1("h0:value=v0,nodes=1000000,oopen=1");
    for (st, me) in &states {
        off.opponent_value(&db, st, *me);
        on.opponent_value(&db, st, *me);
    }
    assert_eq!(off.stats.opp_open_replies, 0);
    assert_eq!(off.stats.opp_open_lines, 0);
    assert_eq!(off.stats.opp_open_lower, 0);
    assert_eq!(off.stats.opp_open_nodes, 0);
    assert!(on.stats.opp_open_replies > 0);
    assert!(on.stats.opp_open_lower <= on.stats.opp_open_replies);
    assert!(on.stats.opp_open_nodes > 0);
}

#[test]
fn oopen_spec() {
    let again = AnyPolicy::parse_spec("h0:oopen=2,obeam=5").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
    assert_eq!(AnyPolicy::parse_spec("h0:oopen=0").unwrap().spec(), "h0");
    assert!(AnyPolicy::parse_spec("h0:oopen=x").is_err());
}
