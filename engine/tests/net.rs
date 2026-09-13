//! Learned leaf (`value=net`): identity, forward pass, search, spec.

use std::path::PathBuf;
use std::time::Instant;

use arena_engine::{
    apply, encode, encode_with_vocab, legal_actions, new_game, play_game, policy_rng, vocab,
    AnyPolicy, CardId, End, First, GameConfig, Observation, Phase, PlayerId, Policy, Random,
    ValueNet, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

fn collect_states(
    db: &arena_engine::CardDb,
    n: usize,
    midgame_only: bool,
) -> Vec<arena_engine::State> {
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

fn net_dir() -> PathBuf {
    fixtures_dir().join("net")
}

fn check_v0_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:value=v0").unwrap().spec(), "h0");
    let mut agree = 0u32;
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260913u64.wrapping_add(i as u64);
        let mut a = parse_h0("h0");
        let mut b = parse_h0("h0:value=v0");
        let mut ra = policy_rng(seed);
        let mut rb = policy_rng(seed);
        let ia = a.choose(&db, state, &legal, &mut ra);
        let ib = b.choose(&db, state, &legal, &mut rb);
        assert_eq!(ia, ib, "h0 vs h0:value=v0 at state {i}");
        agree += 1;
    }
    eprintln!("v0 choose identity: {agree}/{n} mid-game states agreed");
}

#[test]
fn v0_choose_identity_smoke() {
    check_v0_identity(20);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn v0_choose_identity_200_midgame() {
    check_v0_identity(200);
}

#[test]
fn encode_with_vocab_matches_encode_on_200_states() {
    let db = load_db();
    let states = collect_states(&db, 200, true);
    assert_eq!(states.len(), 200);
    for (i, state) in states.iter().enumerate() {
        for me in [PlayerId::A, PlayerId::B] {
            let a = encode(state, me);
            let v = vocab(state);
            let b = encode_with_vocab(state, me, &v);
            assert_eq!(
                a.features.len(),
                b.features.len(),
                "feat len state {i} {me:?}"
            );
            for (j, (x, y)) in a.features.iter().zip(b.features.iter()).enumerate() {
                assert_eq!(x.to_bits(), y.to_bits(), "feat[{j}] state {i} {me:?}");
            }
            assert_eq!(a.ids, b.ids, "ids state {i} {me:?}");
        }
    }
}

fn f32_list(v: &serde_json::Value) -> Vec<f32> {
    v.as_array()
        .expect("array")
        .iter()
        .map(|x| x.as_f64().expect("f") as f32)
        .collect()
}

fn u32_list(v: &serde_json::Value) -> Vec<u32> {
    v.as_array()
        .expect("array")
        .iter()
        .map(|x| x.as_u64().expect("u") as u32)
        .collect()
}

#[test]
fn forward_pass_matches_python_to_1e4() {
    let dir = net_dir();
    let linear = ValueNet::load(dir.join("tiny-linear.json")).expect("tiny-linear");
    let mlp = ValueNet::load(dir.join("tiny-mlp.json")).expect("tiny-mlp");
    let pred: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("tiny-predictions.json")).expect("preds"),
    )
    .expect("preds json");
    let observations = pred["observations"].as_array().expect("obs");
    let lin_p = f32_list(&pred["linear"]);
    let mlp_p = f32_list(&pred["mlp"]);
    assert_eq!(observations.len(), 20);
    assert_eq!(lin_p.len(), 20);
    assert_eq!(mlp_p.len(), 20);
    for (i, obs) in observations.iter().enumerate() {
        let o = Observation {
            features: f32_list(&obs["features"]),
            ids: u32_list(&obs["ids"]),
        };
        let lv = linear.value(&o);
        let mv = mlp.value(&o);
        assert!(
            (lv - lin_p[i]).abs() < 1e-4,
            "linear[{i}] rust={lv} py={}",
            lin_p[i]
        );
        assert!(
            (mv - mlp_p[i]).abs() < 1e-4,
            "mlp[{i}] rust={mv} py={}",
            mlp_p[i]
        );
    }
}

fn play_net_vs_random(path: &str, n: u32, seed: u64) -> Vec<(Option<PlayerId>, u32, u32, End)> {
    let db = load_db();
    let decks = load_deck_file("oracle/decks/basic-forest.json");
    assert!(deck_ready(&db, &decks));
    let spec = format!("h0:value=net,net={path}");
    let mut out = Vec::new();
    for i in 0..n {
        let first = if i % 2 == 0 { First::A } else { First::B };
        let mut state = new_game(
            &db,
            GameConfig {
                seed: seed.wrapping_add(u64::from(i)),
                deck_a: decks.clone(),
                deck_b: decks.clone(),
                first,
                opening_hands: None,
            },
        )
        .unwrap();
        let mut a = parse_h0(&spec);
        let mut b = Random;
        let mut rng = policy_rng(seed.wrapping_add(u64::from(i)));
        let r = play_game(&db, &mut state, &mut a, &mut b, &mut rng);
        assert_ne!(r.end, End::Illegal, "game {i} illegal");
        assert!(
            matches!(r.end, End::Lethal | End::Deckout),
            "game {i} end={:?}",
            r.end
        );
        out.push((r.winner, r.turns, r.actions, r.end));
    }
    out
}

#[test]
fn search_runs_smoke() {
    let path = net_dir().join("tiny-mlp.json");
    let got = play_net_vs_random(&path.display().to_string(), 1, 20260913);
    assert_eq!(got.len(), 1);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn search_runs_10_games_and_determinism() {
    let path = net_dir().join("tiny-mlp.json");
    let p = path.display().to_string();
    let a = play_net_vs_random(&p, 10, 20260913);
    let b = play_net_vs_random(&p, 10, 20260913);
    assert_eq!(a, b, "10 games twice must be identical");
}

#[test]
fn spec_round_trip_missing_net_and_file() {
    let path = net_dir().join("tiny-linear.json");
    let spec = format!("h0:value=net,net={}", path.display());
    let parsed = AnyPolicy::parse_spec(&spec).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(parsed.spec(), spec);
    assert_eq!(AnyPolicy::parse_spec(&parsed.spec()).unwrap(), parsed);

    let e = AnyPolicy::parse_spec("h0:value=net").unwrap_err();
    assert!(e.contains("net"), "{e}");

    let missing = "/tmp/arena-no-such-net.json";
    let e = AnyPolicy::parse_spec(&format!("h0:value=net,net={missing}")).unwrap_err();
    assert!(e.contains(missing), "{e}");

    let a = AnyPolicy::parse_spec(&spec).unwrap();
    let b = AnyPolicy::parse_spec(&format!("h0:value=net,net={}", path.display())).unwrap();
    assert_eq!(a, b);
    let other = AnyPolicy::parse_spec(&format!(
        "h0:value=net,net={}",
        net_dir().join("tiny-mlp.json").display()
    ))
    .unwrap();
    assert_ne!(a, other);
}

#[test]
fn evaluate_cost_us_per_call() {
    let db = load_db();
    let states = collect_states(&db, 200, true);
    assert_eq!(states.len(), 200);
    let v0 = H0::default();
    let linear = parse_h0(&format!(
        "h0:value=net,net={}",
        net_dir().join("tiny-linear.json").display()
    ));
    let mlp = parse_h0(&format!(
        "h0:value=net,net={}",
        net_dir().join("tiny-mlp.json").display()
    ));
    let mut times = Vec::new();
    for (name, h) in [("v0", &v0), ("linear", &linear), ("mlp", &mlp)] {
        // Warm the caches / page in the model.
        for s in states.iter().take(4) {
            let _ = h.evaluate(&db, s, PlayerId::A);
        }
        let t0 = Instant::now();
        let mut n = 0u32;
        for s in &states {
            for me in [PlayerId::A, PlayerId::B] {
                let _ = h.evaluate(&db, s, me);
                n += 1;
            }
        }
        let us = t0.elapsed().as_secs_f64() * 1.0e6 / f64::from(n);
        eprintln!("{name}: {us:.2} µs/call ({n} calls)");
        times.push((name, us));
    }
    assert_eq!(times.len(), 3);
}
