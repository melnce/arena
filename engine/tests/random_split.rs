//! Depths of the Eld Crystals (`90034330`) and Calge-Danthla (`10634120`).

use std::collections::BTreeMap;

use arena_engine::{
    apply, from_neutral, legal_actions, new_game, snapshot_json, Action, CardDb, First, GameConfig,
    GameRng, LoadError, Phase, PlayerId, Slot,
};

mod common;
use common::*;

const CALGE: &str = "10634120";
const DEPTHS: &str = "90034330";
const CRYSTALSPAWN: &str = "10631110";
const VANILLA: &str = "88001110";

fn grant_evolve(db: &CardDb, st: &mut arena_engine::State, slot: u8) {
    let me = st.active;
    if st.player(me).turns_taken < 5 {
        set_round(st, me, 5);
    }
    st.player_mut(me).ep = st.player(me).ep.max(1);
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve");
}

fn depths_in_hand(st: &arena_engine::State, who: PlayerId) -> Option<u8> {
    st.player(who)
        .hand
        .iter()
        .position(|c| c.card.as_str() == DEPTHS)
        .map(|i| i as u8)
}

fn newest_crystalspawn(st: &arena_engine::State, who: PlayerId) -> Option<(i32, i32)> {
    let mut last = None;
    for c in st.player(who).field.iter().flatten() {
        if c.card.as_str() == CRYSTALSPAWN {
            last = Some((c.attack, c.defense));
        }
    }
    last
}

fn play_depths(db: &CardDb, st: &mut arena_engine::State, who: PlayerId) {
    let pos = depths_in_hand(st, who).expect("Depths in hand");
    give_pp(st, who, 10, 10);
    apply(db, st, Action::Play { hand: pos }).expect("play Depths");
    drain_choice(db, st);
}

fn drain_choice(db: &CardDb, st: &mut arena_engine::State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn calge_setup(db: &CardDb, seed: u64) -> arena_engine::State {
    let mut st = started_decks(db, seed, &[CALGE], &[VANILLA]);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(db, &mut st, me, CALGE);
    give_pp(&mut st, me, 10, 10);
    play_id(db, &mut st, me, CALGE);
    drain_choice(db, &mut st);
    let slot = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == CALGE))
        .expect("Calge on field") as u8;
    grant_evolve(db, &mut st, slot);
    st
}

#[test]
fn calge_evolve_legal_and_adds_depths() {
    let db = load_db();
    let st = calge_setup(&db, 1);
    let me = PlayerId::A;
    assert!(hand_has(&st, me, DEPTHS), "Evolve adds Depths to hand");
    let legal = legal_actions(&db, &st);
    assert!(
        !legal.is_empty(),
        "state after evolve should still have legal actions"
    );
}

#[test]
fn depths_faith_zero_summons_base_crystalspawn() {
    let db = load_db();
    let mut st = calge_setup(&db, 2);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).faith = 0;
    let def_before = leader_def(&st, me);
    let opp_def_before = leader_def(&st, opp);
    play_depths(&db, &mut st, me);
    assert_eq!(
        newest_crystalspawn(&st, me),
        Some((1, 1)),
        "faith 0 still summons a 1/1 Crystalspawn"
    );
    assert_eq!(leader_def(&st, me), def_before, "faith 0 restores 0");
    assert_eq!(leader_def(&st, opp), opp_def_before, "faith 0 deals 0");
    assert_eq!(
        st.player(me).faith,
        1,
        "faith is read not spent; +1 when the new Crystalspawn enters"
    );
}

#[test]
fn depths_faith_k_splits_exactly() {
    let db = load_db();
    for faith in [1, 2, 3, 5] {
        let mut st = calge_setup(&db, 10 + faith as u64);
        let me = PlayerId::A;
        let opp = PlayerId::B;
        st.player_mut(me).faith = faith;
        let opp_def_before = leader_def(&st, opp);
        play_depths(&db, &mut st, me);
        let (atk, def) = newest_crystalspawn(&st, me).expect("Crystalspawn summoned");
        let x = atk - 1;
        assert_eq!(x, def - 1, "buff is +X/+X");
        let z = opp_def_before - leader_def(&st, opp);
        let y = faith - x - z;
        assert!(
            (0..=faith).contains(&y),
            "X+Y+Z == faith at faith={faith}, got x={x} y={y} z={z}"
        );
        assert_eq!(
            st.player(me).faith,
            faith + 1,
            "faith is read not spent; +1 when the new Crystalspawn enters"
        );
    }
}

#[test]
fn depths_faith_three_distribution() {
    let db = load_db();
    let mut hist: BTreeMap<(i32, i32, i32), u32> = BTreeMap::new();
    const TRIALS: u32 = 27 * 300;
    for seed in 0..TRIALS {
        let mut st = calge_setup(&db, 1000 + seed as u64);
        let me = PlayerId::A;
        st.player_mut(me).faith = 3;
        let def_before = leader_def(&st, me);
        let opp_def_before = leader_def(&st, PlayerId::B);
        play_depths(&db, &mut st, me);
        let (atk, _) = newest_crystalspawn(&st, me).expect("spawn");
        let x = atk - 1;
        let z = opp_def_before - leader_def(&st, PlayerId::B);
        let y = 3 - x - z;
        assert!((0..=3).contains(&y), "X+Y+Z == 3, got x={x} y={y} z={z}");
        let _ = def_before;
        *hist.entry((x, y, z)).or_insert(0) += 1;
    }
    let p111 = hist.get(&(1, 1, 1)).copied().unwrap_or(0) as f64 / TRIALS as f64;
    let p300 = hist.get(&(3, 0, 0)).copied().unwrap_or(0) as f64 / TRIALS as f64;
    let tol = 0.02;
    assert!(
        (p111 - 6.0 / 27.0).abs() < tol,
        "P(1,1,1)={p111}, want 6/27"
    );
    assert!(
        (p300 - 1.0 / 27.0).abs() < tol,
        "P(3,0,0)={p300}, want 1/27"
    );
}

#[test]
fn depths_scripted_split_counts() {
    let db = load_db();
    let mut st = calge_setup(&db, 42);
    let me = PlayerId::A;
    st.player_mut(me).faith = 3;
    st.rng = GameRng::scripted(
        vec![arena_engine::Pick {
            what: arena_engine::PickWhat::RandomSplit,
            among: None,
            chose: arena_engine::PickChose::Counts(vec![1, 1, 1]),
        }],
        42,
    );
    play_depths(&db, &mut st, me);
    assert_eq!(newest_crystalspawn(&st, me), Some((2, 2)));
    assert_eq!(
        leader_def(&st, me),
        20,
        "Y=1 restore is capped at max defense"
    );
    assert_eq!(leader_def(&st, PlayerId::B), 19);
}

#[test]
fn depths_trace_replay_round_trip() {
    let db = load_db();
    let seed = 4242u64;
    let me = PlayerId::A;
    let mut live = calge_setup(&db, seed);
    live.player_mut(me).faith = 3;
    let hand = depths_in_hand(&live, me).expect("Depths");
    give_pp(&mut live, me, 10, 10);
    let act = Action::Play { hand };
    let neu = arena_engine::to_neutral(&live, &act);
    apply(&db, &mut live, act).unwrap();
    let snap = snapshot_json(&live);
    let picks = live.picks.clone();

    let mut replay = calge_setup(&db, seed);
    replay.player_mut(me).faith = 3;
    give_pp(&mut replay, me, 10, 10);
    replay.rng = GameRng::scripted(picks, seed);
    let replay_act = from_neutral(&replay, &neu).expect("from_neutral");
    apply(&db, &mut replay, replay_act).unwrap();
    let got = snapshot_json(&replay);
    if let Some((path, a, b)) = arena_engine::replay_state_diff(&got, &snap) {
        panic!("replay {path}: arena={a} trace={b}");
    }
}

#[test]
fn gate_rejects_deck_with_unsupported_created_card() {
    let db = load_db();
    let deck = pad_deck(&["89999110"], 40);
    let err = new_game(
        &db,
        GameConfig {
            seed: 1,
            deck_a: deck,
            deck_b: pad_deck(&[VANILLA], 40),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect_err("deck with unsupported created token should fail");
    match err {
        LoadError::Unsupported(u) => {
            assert_eq!(u.card, "89999120");
            assert_eq!(u.construct, "op:counter skyboundHand");
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

#[test]
fn meta_rune_crystal_deck_loads_after_closure() {
    let db = load_db();
    let deck = load_deck_file("oracle/decks/meta-rune-crystal.json");
    assert!(deck_ready(&db, &deck));
    new_game(
        &db,
        GameConfig {
            seed: 1,
            deck_a: deck,
            deck_b: load_deck_file("oracle/decks/meta-sword-rally.json"),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("meta-rune-crystal loads when Depths is supported");
}

#[test]
fn construct_random_split_fixture_playable() {
    let db = load_db();
    db.require_supported(cid("89200330"))
        .expect("construct fixture supported");
    let mut st = started(&db, 99);
    st.player_mut(PlayerId::A).hand.clear();
    put_hand(&db, &mut st, PlayerId::A, "89200330");
    give_pp(&mut st, PlayerId::A, 10, 10);
    let legal = legal_actions(&db, &st);
    assert!(
        legal.iter().any(|a| matches!(a, Action::Play { .. })),
        "randomSplit construct spell is playable"
    );
    play_id(&db, &mut st, PlayerId::A, "89200330");
    assert!(
        leader_def(&st, PlayerId::B) < 20,
        "inner damage runs after split"
    );
}
