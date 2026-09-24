//! H0 root-aggregation pessimism (`pess`): identity at 0, spec
//! round-trip, a mixed-lethal fixture, and determinism. Default
//! behaviour is bit-identical; the yardstick decides any flip.

use arena_engine::{
    acting_player, apply, legal_actions, new_game, play_game, policy_rng, Action, AnyPolicy,
    CardDb, First, GameConfig, Phase, PlayerId, Policy, H0, MAX_ACTIONS, MAX_TURNS,
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
    let decks = load_pinned_corpus_decks_sorted(db);
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

fn set_body(state: &mut arena_engine::State, who: PlayerId, slot: u8, atk: i32, def: i32) {
    let f = state.field_inst_mut(who, slot).unwrap();
    f.attack = atk;
    f.defense = def;
    f.max_defense = def;
    f.flags.summoning_sick = false;
    f.flags.attacks_left = 1;
}

/// Adapted `four_attacker_state` / `storm_lethal_opp_state`: A at 14
/// so the visible 4×3 board is not lethal (12) but 12+2 Storm is.
/// No ready body — the only off-ramp is a 1/1 Ward; the greedy line
/// is the 10-atk Storm. B's hidden pool is 1 Storm + 3 Vanilla.
fn mixed_lethal_opp_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 41);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    for _ in 0..4 {
        let slot = put_field(db, &mut st, PlayerId::B, VANILLA);
        set_body(&mut st, PlayerId::B, slot, 3, 3);
    }
    put_hand(db, &mut st, PlayerId::A, STORM);
    st.player_mut(PlayerId::A).hand[0].attack = 10;
    put_hand(db, &mut st, PlayerId::A, WARD);
    {
        let w = st.player_mut(PlayerId::A).hand.last_mut().unwrap();
        w.attack = 1;
        w.defense = 1;
        w.max_defense = 1;
    }
    give_pp(&mut st, PlayerId::A, 3, 3);
    st.player_mut(PlayerId::A).leader_defense = 14;
    st.player_mut(PlayerId::B).leader_defense = 14;
    st.player_mut(PlayerId::B).deck.clear();
    put_hand(db, &mut st, PlayerId::B, VANILLA);
    put_deck(db, &mut st, PlayerId::B, STORM);
    put_deck(db, &mut st, PlayerId::B, VANILLA);
    put_deck(db, &mut st, PlayerId::B, VANILLA);
    give_pp(&mut st, PlayerId::B, 4, 4);
    assert!(matches!(st.phase, Phase::Main));
    assert_eq!(acting_player(&st), PlayerId::A);
    assert_eq!(st.active, PlayerId::A);
    st
}

fn check_pess0_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:pess=0").unwrap().spec(), "h0");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260919u64.wrapping_add(i as u64);
        let mut main = H0::default();
        let mut named = parse_h0("h0");
        let mut zero = parse_h0("h0:pess=0");
        let mut rng_m = policy_rng(seed);
        let mut rng_n = policy_rng(seed);
        let mut rng_z = policy_rng(seed);
        let im = main.choose(&db, state, &legal, &mut rng_m);
        let inn = named.choose(&db, state, &legal, &mut rng_n);
        let iz = zero.choose(&db, state, &legal, &mut rng_z);
        assert_eq!(im, inn, "h0 vs H0::default at state {i}");
        assert_eq!(im, iz, "h0 vs h0:pess=0 at state {i}");
        assert!(im < legal.len());
    }
}

#[test]
fn pess0_identity_smoke() {
    // 8, not 20: a 20-state default-H0 walk is ~50s in CI debug and the
    // `ci` job is a hard timeout. Release still covers 200.
    check_pess0_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn pess0_identity_200_midgame() {
    check_pess0_identity(200);
}

#[test]
fn spec_pess() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:pess=0").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:pess=0.5").unwrap().spec(),
        "h0:pess=0.5"
    );
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    let again = AnyPolicy::parse_spec("h0:pess=0.5").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
    for bad in ["h0:pess=1.5", "h0:pess=-0.1", "h0:pess=x"] {
        let e = AnyPolicy::parse_spec(bad).unwrap_err();
        assert!(e.contains("pess"), "{bad} → {e}");
    }
}

/// 1/1 Ward vs 10-atk Storm. Seed 7 is the olethal fixture seed;
/// `pess=0` takes the Storm, `pess=1` takes the Ward.
#[test]
fn pess_separates_mixed_lethal() {
    let db = load_db();
    let st = mixed_lethal_opp_state(&db);
    const SEED: u64 = 7;
    // Pre-flip lcap/clip/fusemacro: pessimism separation calibrated on old defaults.
    let mut mean = parse_h0("h0:pess=0,lcap=1,clip=0,fusemacro=0");
    let mut pess1 = parse_h0("h0:pess=1,lcap=1,clip=0,fusemacro=0");
    let (mi, ma) = pick(&mut mean, &db, &st, SEED);
    let (pi, pa) = pick(&mut pess1, &db, &st, SEED);
    eprintln!(
        "mixed-lethal pess=0={ma:?} idx={mi}  pess=1={pa:?} idx={pi} \
         cands0={} chose0={} cands1={} chose1={}",
        mean.stats.cands_with_lethal_root,
        mean.stats.chose_with_lethal_root,
        pess1.stats.cands_with_lethal_root,
        pess1.stats.chose_with_lethal_root
    );
    assert_ne!(
        mi, pi,
        "h0:pess=1 and h0:pess=0 must choose differently on mixed_lethal_opp_state seed {SEED} \
         (pess=0={ma:?} pess=1={pa:?})"
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
    const SEED: u64 = 20260919;
    let first = play_pair_spec(&db, spec, spec, n, SEED);
    let again = play_pair_spec(&db, spec, spec, n, SEED);
    assert_eq!(first, again, "{spec} must be deterministic");
}

#[test]
fn pess_determinism_smoke() {
    check_determinism("h0:pess=0.5,depth=2,nodes=200", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn pess_determinism_20_games() {
    check_determinism("h0:pess=0.5", 20);
}
