//! `mull=leaf`: leaf-scored opening mulligan (default off).

use arena_engine::{
    acting_player, apply, legal_actions, new_game, policy_rng, Action, AnyPolicy, CardDb, First,
    GameConfig, H0Job, JobStep, MullMode, OpeningHands, Phase, PlayerId, Policy, H0,
};

mod common;
use common::*;

const LEAF: &str = "h0:net=h0-value-v69,mull=leaf";
const SERVED_PLUS_LEAF: &str = "h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=h0-value-v69,mull=leaf";

const OPPONENTS: [&str; 4] = [
    "meta-sword-rally",
    "meta-forest-combo",
    "meta-dragon-ramp",
    "meta-haven-evo",
];

const SEEDS: [u64; 4] = [11, 29, 47, 83];

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn meta_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: serde_json::Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("stem").to_string())
        .collect()
}

fn load_meta(stem: &str) -> Vec<arena_engine::CardId> {
    load_deck_file(repo_root().join(format!("oracle/decks/{stem}.json")))
}

fn reach_meta_mulligan(
    db: &CardDb,
    meta: &[arena_engine::CardId],
    opp: &[arena_engine::CardId],
    seed: u64,
    meta_is_second: bool,
) -> arena_engine::State {
    let (deck_a, deck_b) = if meta_is_second {
        (opp.to_vec(), meta.to_vec())
    } else {
        (meta.to_vec(), opp.to_vec())
    };
    let mut st = new_game(
        db,
        GameConfig {
            seed,
            deck_a,
            deck_b,
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut builtin = parse_h0("h0:mull=builtin");
    let mut rng = policy_rng(seed);
    let meta_seat = if meta_is_second {
        PlayerId::B
    } else {
        PlayerId::A
    };
    loop {
        match st.phase {
            Phase::Mulligan { player } if player == meta_seat => return st,
            Phase::Mulligan { .. } => {
                let legal = legal_actions(db, &st);
                let idx = builtin.choose(db, &st, &legal, &mut rng);
                apply(db, &mut st, legal[idx].clone()).expect("builtin mull");
            }
            _ => panic!("expected mulligan, got {:?}", st.phase),
        }
    }
}

fn job_mull_index(spec: &str, db: &CardDb, st: &arena_engine::State, seed: u64) -> usize {
    let (job, step) = H0Job::start(db, st, spec, seed).expect("job start");
    match step {
        JobStep::Done { index, .. } => {
            assert!(job.choose_stats().decisions >= 1);
            index as usize
        }
        _ => panic!("unexpected job step for mulligan"),
    }
}

#[test]
fn mull_leaf_spec_round_trips() {
    for spec in [
        "h0:mull=leaf",
        "h0:mull=leaf,mulls=16",
        "h0:mull=leaf,mulls=64,mullm=1.5",
    ] {
        let parsed = AnyPolicy::parse_spec(spec).unwrap();
        assert_eq!(parsed.spec(), spec, "round trip {spec}");
        let again = AnyPolicy::parse_spec(&parsed.spec()).unwrap();
        assert_eq!(again, parsed);
    }
    let served = AnyPolicy::parse_spec(SERVED_PLUS_LEAF).unwrap();
    let again = AnyPolicy::parse_spec(&served.spec()).unwrap();
    assert_eq!(again, served);
}

#[test]
fn mull_leaf_spec_errors() {
    let e = AnyPolicy::parse_spec("h0:mulls=0").unwrap_err();
    assert!(e.contains("mulls"), "{e}");
    let e = AnyPolicy::parse_spec("h0:mull=leaf,mulls=0").unwrap_err();
    assert!(e.contains("mulls"), "{e}");
    let e = AnyPolicy::parse_spec("h0:mull=leaf,mulls=257").unwrap_err();
    assert!(e.contains("mulls"), "{e}");
    let e = AnyPolicy::parse_spec("h0:mull=leaf,mullm=-1").unwrap_err();
    assert!(e.contains("mullm"), "{e}");
    let e = AnyPolicy::parse_spec("h0:mulls=8").unwrap_err();
    assert!(e.contains("mulls requires mull=leaf"), "{e}");
    let e = AnyPolicy::parse_spec("h0:mullm=1").unwrap_err();
    assert!(e.contains("mullm requires mull=leaf"), "{e}");
    let e = AnyPolicy::parse_spec("h0:mull=leafy").unwrap_err();
    assert!(!e.is_empty(), "{e}");
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn mull_leaf_meta_pool() {
    let db = load_db();
    let stems = meta_stems();
    assert_eq!(stems.len(), 16);
    for stem in &stems {
        let meta = load_meta(stem);
        for opp_name in OPPONENTS {
            let opp = load_meta(opp_name);
            for meta_is_second in [false, true] {
                for &seed in &SEEDS {
                    let st = reach_meta_mulligan(&db, &meta, &opp, seed, meta_is_second);
                    let legal = legal_actions(&db, &st);
                    assert!(legal.len() >= 2, "{stem} legal={}", legal.len());
                    let mut h = parse_h0(LEAF);
                    let mut rng = policy_rng(seed ^ 0xdead_beef);
                    let idx = h.choose(&db, &st, &legal, &mut rng);
                    assert!(matches!(legal[idx], Action::MulliganConfirm { .. }));
                    let mut h2 = parse_h0(LEAF);
                    let mut rng2 = policy_rng(seed ^ 0xdead_beef);
                    let idx2 = h2.choose(&db, &st, &legal, &mut rng2);
                    assert_eq!(idx, idx2, "{stem} {opp_name} seed={seed}");
                    let job_idx = job_mull_index(LEAF, &db, &st, seed ^ 0xdead_beef);
                    assert_eq!(idx, job_idx, "job {stem} seed={seed}");

                    h.arm_explain();
                    let mut rng3 = policy_rng(seed ^ 0xdead_beef);
                    let idx3 = h.choose(&db, &st, &legal, &mut rng3);
                    assert_eq!(idx, idx3);
                    let rec = h.take_explain().expect("explain");
                    let m = rec.mull.as_ref().expect("mull explain");
                    assert_eq!(m.mode, "leaf");
                    assert_eq!(m.samples, 32);
                    assert_eq!(m.kept, 32, "{stem} dropped samples");
                    assert_eq!(m.values.len(), legal.len());
                    let chosen_v = m.values[idx3];
                    for (i, &v) in m.values.iter().enumerate() {
                        if v.is_nan() {
                            continue;
                        }
                        assert!(
                            v <= chosen_v + 1e-4 || (v == chosen_v && i >= idx3),
                            "idx={idx3} i={i} v={v} chosen={chosen_v}"
                        );
                    }

                    let mut margin = parse_h0("h0:net=h0-value-v69,mull=leaf,mullm=1000");
                    let mut rng4 = policy_rng(seed ^ 0xdead_beef);
                    let table_idx = parse_h0("h0:net=h0-value-v69,mull=builtin").choose(
                        &db,
                        &st,
                        &legal,
                        &mut policy_rng(seed ^ 0xdead_beef),
                    );
                    let margin_idx = margin.choose(&db, &st, &legal, &mut rng4);
                    assert_eq!(margin_idx, table_idx, "mullm table {stem} seed={seed}");

                    let mut builtin = parse_h0("h0:net=h0-value-v69,mull=builtin");
                    let builtin_idx =
                        builtin.choose(&db, &st, &legal, &mut policy_rng(seed ^ 0xcafe));
                    let _ = builtin_idx;
                }
            }
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn mull_leaf_scored_position_shape() {
    let db = load_db();
    let meta = load_meta("meta-rune-test-subject");
    let opp = load_meta("meta-sword-rally");
    let st = reach_meta_mulligan(&db, &meta, &opp, 99, false);
    let legal = legal_actions(&db, &st);
    let h = parse_h0(LEAF);
    let me = acting_player(&st);
    let pos = h
        .mull_leaf_scored_position(&db, &st, &legal, 0, 12345)
        .expect("position");
    assert_eq!(pos.active, me);
    assert!(matches!(pos.phase, Phase::Main | Phase::Combat));
    assert_eq!(pos.turn, 1);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn mull_leaf_common_random_numbers_duplicate_slot() {
    let db = load_db();
    let dup = cid("10403120");
    let st = new_game(
        &db,
        GameConfig {
            seed: 555,
            deck_a: load_deck_file("oracle/decks/meta-rune-test-subject.json"),
            deck_b: load_deck_file("oracle/decks/meta-sword-rally.json"),
            first: First::A,
            opening_hands: Some(OpeningHands {
                a: vec![dup, dup, cid("10031210"), cid("10433310")],
                b: vec![],
            }),
        },
    )
    .expect("new_game");
    let legal = legal_actions(&db, &st);
    let swap0 = legal
        .iter()
        .position(|a| matches!(a, Action::MulliganConfirm { swap } if swap[0] && !swap[1]))
        .expect("swap slot 0");
    let swap1 = legal
        .iter()
        .position(|a| matches!(a, Action::MulliganConfirm { swap } if swap[1] && !swap[0]))
        .expect("swap slot 1");
    let h = parse_h0("h0:net=h0-value-v69,mull=leaf,mulls=1");
    let v0 = h
        .mull_leaf_scored_position(&db, &st, &legal, swap0, 7777)
        .map(|s| h.evaluate(&db, &s, PlayerId::A));
    let v1 = h
        .mull_leaf_scored_position(&db, &st, &legal, swap1, 7777)
        .map(|s| h.evaluate(&db, &s, PlayerId::A));
    let (Some(v0), Some(v1)) = (v0, v1) else {
        return;
    };
    let diff = (v0 - v1).abs();
    assert!(
        diff < 1e-3,
        "duplicate-slot swap values differ by {diff} (v0={v0} v1={v1}); report in PR if intentional"
    );
}

#[test]
fn mull_leaf_mode_is_leaf() {
    let h = parse_h0("h0:mull=leaf");
    assert_eq!(h.mull, MullMode::Leaf);
    assert_eq!(h.mulls, 32);
    assert_eq!(h.mullm, 0.0);
}

const STACK: &str = "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=h0-value-v69";

#[test]
#[ignore = "PR measurements only"]
fn mull_leaf_pr_measurements() {
    use std::time::Instant;
    let db = load_db();
    let stems = meta_stems();
    let leaf_spec = format!("{STACK},mull=leaf");
    let mut diff_mask = 0u64;
    let mut total = 0u64;
    let mut builtin_cards = 0f64;
    let mut leaf_cards = 0f64;
    let mut same_32_256 = 0u64;
    let mut mullm1_table = 0u64;
    let mut mullm2_table = 0u64;
    let mut times_32 = Vec::new();
    let mut times_256 = Vec::new();
    for stem in &stems {
        let meta = load_meta(stem);
        for opp_name in OPPONENTS {
            let opp = load_meta(opp_name);
            for meta_is_second in [false, true] {
                for &seed in &SEEDS {
                    let st = reach_meta_mulligan(&db, &meta, &opp, seed, meta_is_second);
                    let legal = legal_actions(&db, &st);
                    total += 1;
                    let mut builtin = parse_h0(&format!("{STACK},mull=builtin"));
                    let mut leaf = parse_h0(&leaf_spec);
                    let bidx = builtin.choose(&db, &st, &legal, &mut policy_rng(seed));
                    let lidx = leaf.choose(&db, &st, &legal, &mut policy_rng(seed));
                    if bidx != lidx {
                        diff_mask += 1;
                    }
                    let swap = |idx: usize| match &legal[idx] {
                        Action::MulliganConfirm { swap } => swap.iter().filter(|s| **s).count(),
                        _ => 0,
                    };
                    builtin_cards += swap(bidx) as f64;
                    leaf_cards += swap(lidx) as f64;
                    let mut h32 = parse_h0(&format!("{STACK},mull=leaf,mulls=32"));
                    let mut h256 = parse_h0(&format!("{STACK},mull=leaf,mulls=256"));
                    let i32 = h32.choose(&db, &st, &legal, &mut policy_rng(seed ^ 1));
                    let i256 = h256.choose(&db, &st, &legal, &mut policy_rng(seed ^ 1));
                    if i32 == i256 {
                        same_32_256 += 1;
                    }
                    let mut hm1 = parse_h0(&format!("{STACK},mull=leaf,mullm=1"));
                    let mut hm2 = parse_h0(&format!("{STACK},mull=leaf,mullm=2"));
                    let tidx = builtin.choose(&db, &st, &legal, &mut policy_rng(seed ^ 2));
                    if hm1.choose(&db, &st, &legal, &mut policy_rng(seed ^ 2)) == tidx {
                        mullm1_table += 1;
                    }
                    if hm2.choose(&db, &st, &legal, &mut policy_rng(seed ^ 2)) == tidx {
                        mullm2_table += 1;
                    }
                    for (spec, bucket) in [
                        (format!("{STACK},mull=leaf,mulls=32"), &mut times_32),
                        (format!("{STACK},mull=leaf,mulls=256"), &mut times_256),
                    ] {
                        let mut h = parse_h0(&spec);
                        let t0 = Instant::now();
                        let _ = h.choose(&db, &st, &legal, &mut policy_rng(seed ^ 3));
                        bucket.push(t0.elapsed().as_secs_f64() * 1000.0);
                    }
                }
            }
        }
    }
    fn pct(n: u64, d: u64) -> f64 {
        100.0 * n as f64 / d as f64
    }
    fn median(v: &mut [f64]) -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    }
    fn p90(v: &[f64]) -> f64 {
        let mut w = v.to_vec();
        w.sort_by(|a, b| a.partial_cmp(b).unwrap());
        w[(w.len() as f64 * 0.9) as usize]
    }
    eprintln!("mull_leaf measurements total={total}");
    eprintln!(
        "leaf vs builtin diff_mask={}/{} ({:.1}%)",
        diff_mask,
        total,
        pct(diff_mask, total)
    );
    eprintln!(
        "mean cards sent back builtin={:.3} leaf={:.3}",
        builtin_cards / total as f64,
        leaf_cards / total as f64
    );
    eprintln!(
        "mulls 32 vs 256 same={}/{} ({:.1}%)",
        same_32_256,
        total,
        pct(same_32_256, total)
    );
    eprintln!(
        "mullm=1 kept table={}/{} ({:.1}%)",
        mullm1_table,
        total,
        pct(mullm1_table, total)
    );
    eprintln!(
        "mullm=2 kept table={}/{} ({:.1}%)",
        mullm2_table,
        total,
        pct(mullm2_table, total)
    );
    eprintln!(
        "wall ms mulls=32 median={:.2} p90={:.2} max={:.2}",
        median(&mut times_32.clone()),
        p90(&times_32),
        times_32.iter().cloned().fold(0.0, f64::max)
    );
    eprintln!(
        "wall ms mulls=256 median={:.2} p90={:.2} max={:.2}",
        median(&mut times_256.clone()),
        p90(&times_256),
        times_256.iter().cloned().fold(0.0, f64::max)
    );
}
