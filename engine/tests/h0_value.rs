//! H0 value=v1: NeedsTable fixtures and leaf-value identity / sensitivity.

use std::path::PathBuf;

use arena_engine::{
    apply, legal_actions, new_game, play_game, policy_rng, AnyPolicy, CardDb, CardId, CardNeeds,
    First, GameConfig, Phase, PlayerId, ValueVersion, Weights, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

const INF: f32 = 1.0e9;

/// Copy of `value` on main @ 6a7530f — the v0 reference.
fn value_main(state: &arena_engine::State, me: PlayerId) -> f32 {
    if state.winner == Some(me) {
        return INF;
    }
    if let Some(w) = state.winner {
        if w != me {
            return -INF;
        }
    }
    let opp = me.opponent();
    let p = state.player(me);
    let o = state.player(opp);
    let mut v = 4.5 * (p.leader_defense - o.leader_defense) as f32;
    v += board_score_main(p) - board_score_main(o);
    v += 0.55 * (p.hand.len() as f32 - o.hand.len() as f32);
    v += 0.18 * (next_pp_main(p) - next_pp_main(o));
    v += 0.35 * (p.ep - o.ep) as f32;
    v += 0.45 * (p.sep - o.sep) as f32;
    v += 0.25 * (p.crests.len() as f32 - o.crests.len() as f32);
    for c in &p.crests {
        if c.countdown.is_some() {
            v += 0.15;
        }
    }
    for c in &o.crests {
        if c.countdown.is_some() {
            v -= 0.15;
        }
    }
    v
}

fn next_pp_main(p: &arena_engine::PlayerState) -> f32 {
    let mut m = p.pp_max;
    if m < 10 {
        m += 1;
    }
    m as f32
}

fn board_score_main(p: &arena_engine::PlayerState) -> f32 {
    let mut s = 0.0f32;
    for c in p.field.iter().flatten() {
        s += (c.attack + c.defense) as f32;
        if c.is_ward() {
            s += 2.2;
        }
        if c.is_storm() {
            s += 1.6;
        }
        if c.evolved {
            s += 1.1;
        }
    }
    s
}

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

fn pick_need(db: &CardDb, pred: impl Fn(&CardNeeds) -> bool) -> (CardId, &CardNeeds) {
    let mut hits: Vec<(CardId, &CardNeeds)> = db.needs().iter().filter(|(_, n)| pred(n)).collect();
    hits.sort_by_key(|(id, _)| *id);
    hits.into_iter()
        .next()
        .unwrap_or_else(|| panic!("no card matched the needs predicate"))
}

fn necromancy_ns(text: &str) -> Vec<i32> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(pos) = rest.find("Necromancy (") {
        rest = &rest[pos + "Necromancy (".len()..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            continue;
        }
        if rest.as_bytes().get(digits.len()) == Some(&b')') {
            if let Ok(n) = digits.parse::<i32>() {
                out.push(n);
            }
        }
    }
    out
}

#[test]
fn needs_table_fixtures_from_walk() {
    let db = load_db();
    let table = db.needs();
    eprintln!(
        "NeedsTable len={} skipped={}",
        table.len(),
        table.skipped().len()
    );
    let mut pay_s = 0usize;
    let mut pay_e = 0usize;
    let mut pay_f = 0usize;
    let mut rally = 0usize;
    let mut boost = 0usize;
    let mut lw = 0usize;
    for (_, n) in table.iter() {
        if !n.shadows.is_empty() {
            pay_s += 1;
        }
        if !n.earth.is_empty() {
            pay_e += 1;
        }
        if !n.faith.is_empty() {
            pay_f += 1;
        }
        if !n.rally.is_empty() {
            rally += 1;
        }
        if n.spellboost {
            boost += 1;
        }
        if n.last_words {
            lw += 1;
        }
    }
    eprintln!(
        "needs columns: shadows={pay_s} earth={pay_e} faith={pay_f} rally={rally} spellboost={boost} last_words={lw}"
    );
    for s in table.skipped() {
        eprintln!("skipped {} on {}", s.kind, s.id.as_str());
    }

    let (shadows_id, shadows_n) = pick_need(&db, |n| n.shadows.iter().any(|&x| x > 0));
    assert!(
        !shadows_n.shadows.is_empty(),
        "{} shadows list empty",
        shadows_id.as_str()
    );
    let (earth_id, earth_n) = pick_need(&db, |n| n.earth.iter().any(|&x| x > 0));
    assert!(
        !earth_n.earth.is_empty(),
        "{} earth list empty",
        earth_id.as_str()
    );
    let (rally_id, rally_n) = pick_need(&db, |n| !n.rally.is_empty());
    assert!(
        !rally_n.rally.is_empty(),
        "{} rally list empty",
        rally_id.as_str()
    );
    let (boost_id, boost_n) = pick_need(&db, |n| n.spellboost);
    assert!(boost_n.spellboost);
    let (lw_id, lw_n) = pick_need(&db, |n| n.last_words);
    assert!(lw_n.last_words);
    eprintln!(
        "fixture ids: pay_shadows={} pay_earth={} rally={} spellboost={} last_words={}",
        shadows_id.as_str(),
        earth_id.as_str(),
        rally_id.as_str(),
        boost_id.as_str(),
        lw_id.as_str()
    );

    let vanilla = cid("88001110");
    assert!(
        table.get(vanilla).is_none(),
        "vanilla follower {} must have no needs entry",
        vanilla.as_str()
    );

    let abyss = load_deck_file("oracle/decks/abyss-p8rfn.json");
    let mut necro = 0usize;
    for id in abyss
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
    {
        let Ok(card) = db.card(id) else {
            continue;
        };
        let blob = serde_json::to_string(card).expect("card json");
        let ns = necromancy_ns(&blob);
        if ns.is_empty() {
            continue;
        }
        necro += 1;
        let needs = table
            .get(id)
            .unwrap_or_else(|| panic!("{} prints Necromancy but has no needs entry", id.as_str()));
        for n in ns {
            assert!(
                needs.shadows.contains(&n),
                "{} Necromancy ({n}) missing from shadows {:?}",
                id.as_str(),
                needs.shadows
            );
        }
    }
    assert_eq!(necro, 3, "abyss-p8rfn should have three Necromancy cards");
}

#[test]
fn v0_bit_identical_on_200_midgame_states() {
    let db = load_db();
    let states = collect_states(&db, 200, true);
    assert_eq!(states.len(), 200, "could not reach 200 mid-game states");
    let h0 = parse_h0("h0");
    let h0v0 = parse_h0("h0:value=v0");
    assert_eq!(h0.value, ValueVersion::V0);
    assert_eq!(h0v0.value, ValueVersion::V0);
    for (i, state) in states.iter().enumerate() {
        for me in [PlayerId::A, PlayerId::B] {
            let want = value_main(state, me);
            let a = h0.evaluate(&db, state, me);
            let b = h0v0.evaluate(&db, state, me);
            assert_eq!(
                a.to_bits(),
                want.to_bits(),
                "h0 vs main at state {i} {me:?}"
            );
            assert_eq!(
                b.to_bits(),
                want.to_bits(),
                "h0:value=v0 vs main at state {i} {me:?}"
            );
        }
    }
}

fn blank_value_state(db: &CardDb) -> arena_engine::State {
    let mut state = started(db, 7);
    clear_hand(&mut state, PlayerId::A);
    clear_hand(&mut state, PlayerId::B);
    state.player_mut(PlayerId::A).shadows = 0;
    state.player_mut(PlayerId::B).shadows = 0;
    state.player_mut(PlayerId::A).earth = 0;
    state.player_mut(PlayerId::B).earth = 0;
    state.player_mut(PlayerId::A).faith = 0;
    state.player_mut(PlayerId::B).faith = 0;
    state.player_mut(PlayerId::A).rally = 0;
    state.player_mut(PlayerId::B).rally = 0;
    state
}

#[test]
fn v1_sensitivity_shadows_earth_last_words() {
    let db = load_db();
    let w = Weights::default();
    let v1 = parse_h0("h0:value=v1");
    let v0 = parse_h0("h0");

    let (shadows_id, shadows_n) = pick_need(&db, |n| {
        n.shadows.contains(&6) && n.earth.is_empty() && n.faith.is_empty() && n.rally.is_empty()
    });
    assert!(shadows_n.shadows.contains(&6));
    let mut s0 = blank_value_state(&db);
    put_hand(&db, &mut s0, PlayerId::A, &shadows_id.as_str());
    let mut s6 = s0.clone();
    s6.player_mut(PlayerId::A).shadows = 6;
    let d_v1 = v1.evaluate(&db, &s6, PlayerId::A) - v1.evaluate(&db, &s0, PlayerId::A);
    let d_v0 = v0.evaluate(&db, &s6, PlayerId::A) - v0.evaluate(&db, &s0, PlayerId::A);
    let want = w.need * 1.0 + 6.0 * w.shadows;
    assert!(
        (d_v1 - want).abs() < 1e-4,
        "shadows Δ v1={d_v1} want={want} card={}",
        shadows_id.as_str()
    );
    assert_eq!(d_v0, 0.0, "v0 must ignore shadows");

    let (earth_id, _) = pick_need(&db, |n| {
        !n.earth.is_empty() && n.shadows.is_empty() && n.faith.is_empty() && n.rally.is_empty()
    });
    let mut e0 = blank_value_state(&db);
    put_hand(&db, &mut e0, PlayerId::A, &earth_id.as_str());
    let mut e6 = e0.clone();
    e6.player_mut(PlayerId::A).earth = 6;
    let d_v1 = v1.evaluate(&db, &e6, PlayerId::A) - v1.evaluate(&db, &e0, PlayerId::A);
    let d_v0 = v0.evaluate(&db, &e6, PlayerId::A) - v0.evaluate(&db, &e0, PlayerId::A);
    let want = w.need * 1.0 + 6.0 * w.earth;
    assert!(
        (d_v1 - want).abs() < 1e-4,
        "earth Δ v1={d_v1} want={want} card={}",
        earth_id.as_str()
    );
    assert_eq!(d_v0, 0.0, "v0 must ignore earth");

    let (lw_id, _) = pick_need(&db, |n| n.last_words);
    let mut base = blank_value_state(&db);
    let with = {
        let mut s = base.clone();
        put_field(&db, &mut s, PlayerId::A, &lw_id.as_str());
        s
    };
    put_field(&db, &mut base, PlayerId::A, "88001110");
    // Isolate the Last Words term: same field occupancy vs vanilla, or
    // subtract v0's board delta if stats differ.
    let d_v1 = v1.evaluate(&db, &with, PlayerId::A) - v1.evaluate(&db, &base, PlayerId::A);
    let d_v0 = v0.evaluate(&db, &with, PlayerId::A) - v0.evaluate(&db, &base, PlayerId::A);
    let d_lw = d_v1 - d_v0;
    assert!(
        (d_lw - w.last_words).abs() < 1e-4,
        "last_words extra {d_lw} want {} (v1={d_v1} v0={d_v0}) card={}",
        w.last_words,
        lw_id.as_str()
    );
}

#[test]
fn zero_weight_removes_its_term() {
    let db = load_db();
    let w = Weights::default();
    let (shadows_id, _) = pick_need(&db, |n| {
        n.shadows.contains(&6) && n.earth.is_empty() && n.faith.is_empty() && n.rally.is_empty()
    });
    let mut s0 = blank_value_state(&db);
    put_hand(&db, &mut s0, PlayerId::A, &shadows_id.as_str());
    let mut s6 = s0.clone();
    s6.player_mut(PlayerId::A).shadows = 6;

    let no_need = parse_h0("h0:value=v1,w_need=0");
    let d = no_need.evaluate(&db, &s6, PlayerId::A) - no_need.evaluate(&db, &s0, PlayerId::A);
    assert!(
        (d - 6.0 * w.shadows).abs() < 1e-4,
        "w_need=0 Δ={d} want={}",
        6.0 * w.shadows
    );

    let no_sh = parse_h0("h0:value=v1,w_shadows=0");
    let d = no_sh.evaluate(&db, &s6, PlayerId::A) - no_sh.evaluate(&db, &s0, PlayerId::A);
    assert!(
        (d - w.need).abs() < 1e-4,
        "w_shadows=0 Δ={d} want={}",
        w.need
    );
}

#[test]
fn spec_value_and_weights() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:value=v0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:value=v1").unwrap().spec(),
        "h0:value=v1"
    );
    let again = AnyPolicy::parse_spec("h0:value=v1").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
    let custom = AnyPolicy::parse_spec("h0:value=v1,w_shadows=0").unwrap();
    assert_eq!(custom.spec(), "h0:value=v1,w_shadows=0");
    let e = AnyPolicy::parse_spec("h0:value=v2").unwrap_err();
    assert!(e.contains("v2"), "{e}");
    let e = AnyPolicy::parse_spec("h0:w_shadows=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

#[test]
fn v1_deterministic_for_a_seed() {
    let db = load_db();
    let decks = load_deck_file("oracle/decks/basic-forest.json");
    assert!(deck_ready(&db, &decks));
    const SEED: u64 = 20260912;
    let mut first = Vec::new();
    for pass in 0..2 {
        let mut got = Vec::new();
        for i in 0..20u32 {
            let first_seat = if i % 2 == 0 { First::A } else { First::B };
            let mut state = new_game(
                &db,
                GameConfig {
                    seed: SEED.wrapping_add(u64::from(i)),
                    deck_a: decks.clone(),
                    deck_b: decks.clone(),
                    first: first_seat,
                    opening_hands: None,
                },
            )
            .unwrap();
            let mut rng = policy_rng(SEED.wrapping_add(u64::from(i)));
            // `H0::fast()` + v1 so this stays inside the debug `cargo test`
            // budget (CI `ci` is 10 minutes). Full-depth `h0:value=v1` is
            // the Python yardstick determinism test.
            let mut a = H0::fast();
            a.value = ValueVersion::V1;
            let mut b = H0::fast();
            let out = play_game(&db, &mut state, &mut a, &mut b, &mut rng);
            got.push((out.winner, out.turns, out.actions, out.end));
        }
        if pass == 0 {
            first = got;
        } else {
            assert_eq!(first, got, "h0:value=v1 vs h0 must be deterministic");
        }
    }
}

#[test]
fn needs_table_is_wasm_shape() {
    let _ = PathBuf::from(".");
    let db = load_db();
    assert!(!db.needs().is_empty());
}
