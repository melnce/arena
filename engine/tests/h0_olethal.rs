//! H0 cheap opponent model: `olethal=1` glance-level lethal sweep and
//! `osteps=N` greedy cutoff. Identity at the sweep-5 defaults
//! (`olethal=1,osteps=6`); the pre-flip pair stays reachable. Fixtures
//! for the four-attacker Ward, play-then-face Storm, no false positive,
//! and the fourth-attack `osteps` line.

use arena_engine::{
    acting_player, apply, legal_actions, new_game, play_game, policy_rng, Action, AnyPolicy,
    CardDb, CardId, First, GameConfig, Phase, PlayerId, Policy, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

/// Fox of Purity — 2-cost 1/3, printed Ward, no Fanfare.
const WARD: &str = "10061120";
/// Synth Vanilla — 2/2 body used as the four attackers / my board.
const VANILLA: &str = "88001110";
/// Troue, Heroic Visionary — 3-cost 2/1 Storm (Engage-Drain only).
const STORM: &str = "10461110";

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

fn pick(h: &mut H0, db: &CardDb, state: &arena_engine::State, seed: u64) -> (usize, Action) {
    let legal = legal_actions(db, state);
    assert!(!legal.is_empty(), "no legal actions");
    let mut rng = policy_rng(seed);
    let i = h.choose(db, state, &legal, &mut rng);
    assert!(i < legal.len(), "illegal index {i}");
    (i, legal[i].clone())
}

fn is_play_id(state: &arena_engine::State, who: PlayerId, a: &Action, id: &str) -> bool {
    let want = cid(id);
    match a {
        Action::Play { hand } => state
            .player(who)
            .hand
            .get(*hand as usize)
            .is_some_and(|c| c.card == want),
        _ => false,
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

fn four_attacker_state(db: &CardDb, my_leader: i32) -> arena_engine::State {
    four_attacker_state_ward_def(db, my_leader, 15)
}

fn four_attacker_state_ward_def(db: &CardDb, my_leader: i32, ward_def: i32) -> arena_engine::State {
    let mut st = started(db, 41);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let mine = put_field(db, &mut st, PlayerId::A, VANILLA);
    set_body(&mut st, PlayerId::A, mine, 2, 2);
    st.field_inst_mut(PlayerId::A, mine)
        .unwrap()
        .flags
        .summoning_sick = true;
    for _ in 0..4 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    put_hand(db, &mut st, PlayerId::A, STORM);
    st.player_mut(PlayerId::A).hand[0].attack = 10;
    put_hand(db, &mut st, PlayerId::A, WARD);
    {
        let w = st.player_mut(PlayerId::A).hand.last_mut().unwrap();
        w.defense = ward_def;
        w.max_defense = ward_def;
    }
    give_pp(&mut st, PlayerId::A, 3, 3);
    st.player_mut(PlayerId::A).leader_defense = my_leader;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert!(matches!(st.phase, Phase::Main));
    assert_eq!(acting_player(&st), PlayerId::A);
    assert_eq!(st.active, PlayerId::A);
    assert!(!st.player(PlayerId::B).deck.is_empty(), "opp deck present");
    st
}

fn four_attacker_normal_ward_bodies(db: &CardDb) -> arena_engine::State {
    let mut st = four_attacker_state_ward_def(db, 12, 3);
    for slot in 0..5u8 {
        if let Some(f) = st.field_inst_mut(PlayerId::B, slot) {
            f.defense = 11;
            f.max_defense = 11;
        }
    }
    st
}

fn storm_lethal_opp_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 42);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let att = put_field(db, &mut st, PlayerId::B, VANILLA);
    set_body(&mut st, PlayerId::B, att, 3, 3);
    put_hand(db, &mut st, PlayerId::B, STORM);
    give_pp(&mut st, PlayerId::B, 3, 3);
    set_round(&mut st, PlayerId::B, 5);
    give_pp(&mut st, PlayerId::B, 3, 5);
    st.player_mut(PlayerId::B).ep = 1;
    st.player_mut(PlayerId::B).evolved_this_turn = false;
    st.player_mut(PlayerId::A).leader_defense = 5;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    st
}

fn no_lethal_opp_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 43);
    skip_to_player_turn(db, &mut st, PlayerId::B, 1);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    for _ in 0..2 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    st.player_mut(PlayerId::A).leader_defense = 20;
    st.player_mut(PlayerId::B).leader_defense = 20;
    assert_eq!(st.active, PlayerId::B);
    assert!(st.winner.is_none());
    st
}

fn check_olethal1_osteps6_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:olethal=1,osteps=6")
            .unwrap()
            .spec(),
        "h0"
    );
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260914u64.wrapping_add(i as u64);
        let mut main = H0::default();
        let mut named = parse_h0("h0");
        let mut pair = parse_h0("h0:olethal=1,osteps=6");
        let mut rng_m = policy_rng(seed);
        let mut rng_n = policy_rng(seed);
        let mut rng_p = policy_rng(seed);
        let im = main.choose(&db, state, &legal, &mut rng_m);
        let inn = named.choose(&db, state, &legal, &mut rng_n);
        let ip = pair.choose(&db, state, &legal, &mut rng_p);
        assert_eq!(im, inn, "h0 vs H0::default at state {i}");
        assert_eq!(im, ip, "h0 vs h0:olethal=1,osteps=6 at state {i}");
        assert!(im < legal.len());
    }
}

#[test]
fn olethal1_osteps6_identity_smoke() {
    // 8, not 20: a 20-state default-H0 walk is ~50s in CI debug and the
    // `ci` job is a hard 10-minute timeout. Release still covers 200.
    check_olethal1_osteps6_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn olethal1_osteps6_identity_200_midgame() {
    check_olethal1_osteps6_identity(200);
}

/// `h0:olethal=0,osteps=3` is still a reachable, deterministic policy
/// and still differs from the default on at least one of the identity
/// states.
fn check_preflip_reachable(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(
        AnyPolicy::parse_spec("h0:olethal=0,osteps=3")
            .unwrap()
            .spec(),
        "h0:olethal=0,osteps=3"
    );
    let mut differ = 0u32;
    let mut compared = 0u32;
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260914u64.wrapping_add(i as u64);
        let mut def = H0::default();
        let mut pre = parse_h0("h0:olethal=0,osteps=3");
        let mut pre_again = parse_h0("h0:olethal=0,osteps=3");
        let mut rng_d = policy_rng(seed);
        let mut rng_p = policy_rng(seed);
        let mut rng_p2 = policy_rng(seed);
        let id = def.choose(&db, state, &legal, &mut rng_d);
        let ip = pre.choose(&db, state, &legal, &mut rng_p);
        let ip2 = pre_again.choose(&db, state, &legal, &mut rng_p2);
        assert_eq!(
            ip, ip2,
            "h0:olethal=0,osteps=3 must be deterministic at state {i}"
        );
        assert!(ip < legal.len());
        compared += 1;
        if id != ip {
            differ += 1;
        }
    }
    assert!(compared > 0, "no comparable states");
    if n >= 200 {
        assert!(
            differ >= 1,
            "h0:olethal=0,osteps=3 must differ from the default on ≥ 1 of {compared} states"
        );
    }
}

#[test]
fn olethal0_osteps3_reachable_smoke() {
    check_preflip_reachable(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn olethal0_osteps3_reachable_200_midgame() {
    check_preflip_reachable(200);
}

#[test]
fn spec_olethal_osteps() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:olethal=1,osteps=6")
            .unwrap()
            .spec(),
        "h0"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:olethal=0,osteps=3")
            .unwrap()
            .spec(),
        "h0:olethal=0,osteps=3"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:olethal=0").unwrap().spec(),
        "h0:olethal=0"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:osteps=3").unwrap().spec(),
        "h0:osteps=3"
    );
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    let both = AnyPolicy::parse_spec("h0:olethal=1,osteps=6").unwrap();
    assert_eq!(both.spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec(&both.spec()).unwrap(), both);
    let e = AnyPolicy::parse_spec("h0:olethal=2").unwrap_err();
    assert!(e.contains("2"), "{e}");
    let e = AnyPolicy::parse_spec("h0:osteps=x").unwrap_err();
    assert!(e.contains("x"), "{e}");
}

fn greedy_line(
    db: &CardDb,
    state: &arena_engine::State,
    who: PlayerId,
    osteps: u32,
    h: &H0,
) -> Vec<Action> {
    let mut s = state.clone();
    let mut steps = 0u32;
    let mut out = Vec::new();
    while s.winner.is_none()
        && !matches!(s.phase, Phase::Terminal)
        && acting_player(&s) == who
        && steps < osteps + 3
        && s.turn <= MAX_TURNS
    {
        let legal = legal_actions(db, &s);
        if legal.is_empty() {
            break;
        }
        if let Some(end) = legal.iter().position(|a| matches!(a, Action::EndTurn)) {
            if steps >= osteps {
                out.push(legal[end].clone());
                if apply(db, &mut s, legal[end].clone()).is_err() {
                    break;
                }
                break;
            }
        }
        let mut best_i = 0usize;
        let mut best_v = f32::NEG_INFINITY;
        for (i, a) in legal.iter().enumerate() {
            let mut t = s.clone();
            if apply(db, &mut t, a.clone()).is_err() {
                continue;
            }
            let v = h.evaluate(db, &t, who);
            if v > best_v {
                best_v = v;
                best_i = i;
            }
        }
        out.push(legal[best_i].clone());
        if apply(db, &mut s, legal[best_i].clone()).is_err() {
            break;
        }
        steps += 1;
    }
    out
}

#[test]
fn four_attacker_olethal_plays_ward() {
    let db = load_db();
    let tall = four_attacker_state(&db, 12);
    let bodies = four_attacker_normal_ward_bodies(&db);
    for (label, st) in [("1/15-Ward", &tall), ("1/3-Ward-3/11", &bodies)] {
        // `olethal=1` is now the default; named so the fixture stays explicit.
        let mut sweep = parse_h0("h0:olethal=1,wv=300");
        let mut greedy = parse_h0("h0:olethal=0,osteps=3,wv=300");
        let (si, sa) = pick(&mut sweep, &db, st, 7);
        let (gi, ga) = pick(&mut greedy, &db, st, 7);
        eprintln!("{label} olethal={sa:?} idx={si}  greedy={ga:?} idx={gi}");
        assert!(
            is_play_id(st, PlayerId::A, &sa, WARD),
            "{label}: olethal=1 must play Ward {WARD}, got {sa:?} idx={si}"
        );
    }
}

#[test]
fn play_then_attack_olethal_sees_storm() {
    let db = load_db();
    let st = storm_lethal_opp_state(&db);
    // `olethal=1` is now the default; named so the fixture stays explicit.
    let mut sweep = parse_h0("h0:olethal=1,wv=300");
    let mut greedy = parse_h0("h0:olethal=0,osteps=3,wv=300");
    let sv = sweep.opponent_value(&db, &st, PlayerId::A);
    let gv = greedy.opponent_value(&db, &st, PlayerId::A);
    eprintln!("play-then-attack olethal={sv} greedy={gv}");
    assert!(
        (sv + 300.0).abs() < 1e-3,
        "olethal=1 must see play-then-face lethal as -300, got {sv}"
    );
    assert!(gv.is_finite(), "greedy line must stay finite, got {gv}");
    assert!(
        gv > -300.0 + 1.0,
        "greedy without the sweep should stay above -300, got {gv}"
    );
}

#[test]
fn no_false_positive_empty_hand_two_bodies() {
    let db = load_db();
    let st = no_lethal_opp_state(&db);
    // `olethal=1` is now the default; named so the fixture stays explicit.
    let mut with = parse_h0("h0:olethal=1");
    let mut without = parse_h0("h0:olethal=0,osteps=3");
    let a = with.opponent_value(&db, &st, PlayerId::A);
    let b = without.opponent_value(&db, &st, PlayerId::A);
    eprintln!(
        "no-lethal olethal={a} greedy={b} checks={} found={}",
        with.stats.opp_lethal_checks, with.stats.opp_lethal_found
    );
    assert!(
        (a - b).abs() < 1e-5,
        "sweep must not change the greedy value: {a} vs {b}"
    );
    assert_eq!(with.stats.opp_lethal_found, 0);
    assert_eq!(with.stats.opp_lethal_checks, 1);
}

#[test]
fn osteps6_reaches_fourth_attack() {
    let db = load_db();
    let st = four_attacker_state(&db, 12);
    // `osteps=6` is now the default; named so the fixture stays explicit.
    let mut long = parse_h0("h0:osteps=6,wv=300");
    let mut short = parse_h0("h0:osteps=3,wv=300");
    let (li, la) = pick(&mut long, &db, &st, 7);
    let (si, sa) = pick(&mut short, &db, &st, 7);
    eprintln!("osteps=6={la:?} idx={li}  osteps=3={sa:?} idx={si}");
    let mut after_end = st.clone();
    apply(&db, &mut after_end, Action::EndTurn).expect("EndTurn");
    let mut v6 = parse_h0("h0:osteps=6,wv=300");
    let mut v3 = parse_h0("h0:osteps=3,wv=300");
    let ov6 = v6.opponent_value(&db, &after_end, PlayerId::A);
    let ov3 = v3.opponent_value(&db, &after_end, PlayerId::A);
    eprintln!("after EndTurn opponent_value osteps=6={ov6} osteps=3={ov3}");
    if !is_play_id(&st, PlayerId::A, &la, WARD) {
        let line = greedy_line(&db, &after_end, PlayerId::B, 6, &v6);
        eprintln!("osteps=6 did not play Ward; greedy line from EndTurn: {line:?}");
        eprintln!(
            "finding: greedy_index ordering wasted the extra steps (not a reason to touch it)"
        );
    }
    assert!(
        is_play_id(&st, PlayerId::A, &la, WARD),
        "osteps=6 must play Ward {WARD}, got {la:?} idx={li}"
    );
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

fn check_determinism(spec: &str, n: u32) {
    let db = load_db();
    const SEED: u64 = 20260914;
    let first = play_pair_spec(&db, spec, spec, n, SEED);
    let again = play_pair_spec(&db, spec, spec, n, SEED);
    assert_eq!(first, again, "{spec} must be deterministic");
}

#[test]
fn olethal_osteps_determinism_smoke() {
    // `olethal=1,osteps=6` is now the default; named so the fixture stays explicit.
    check_determinism("h0:olethal=1,osteps=6,depth=2,nodes=200", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn olethal_osteps_determinism_20_games() {
    // `olethal=1,osteps=6` is now the default; named so the fixture stays explicit.
    check_determinism("h0:olethal=1,osteps=6", 20);
}
