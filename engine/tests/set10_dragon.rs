//! Set 10 Dragoncraft: printed behaviour, Jerva Q&A (X=0), Clue-in-hand
//! discard/draw, Leizaut X∈{0,1,3}, Bird's-Eye leftmost, Dragonewt Scout Strike.

use arena_engine::{apply, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const MARAUDER: &str = "11041110";
const MANIFEST: &str = "11042310";
const ICENEWT: &str = "11041120";
const DANCE: &str = "11041310";
const SCOUT: &str = "11042120";
const BIRDS: &str = "11043310";
const MUGGER: &str = "11043110";
const JERVA: &str = "11044120";
const SMUGGLER: &str = "11042110";
const LEIZAUT: &str = "11044110";
const MISCHIEF: &str = "11012110";
const TANK: &str = "89500001"; // 2/10
const PAD: &str = "88001110"; // 1/1

const SET10_DRAGON: &[&str] = &[
    MARAUDER, MANIFEST, ICENEWT, DANCE, SCOUT, BIRDS, MUGGER, JERVA, SMUGGLER, LEIZAUT,
];

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    // Preserve an already-raised max (Overflow setups); never clamp it up to 10.
    let max = st.player(st.active).pp_max.max(pp);
    give_pp(st, st.active, pp, max);
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn hand_count(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    st.player(who)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == id)
        .count()
}

fn push_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).unwrap();
    let id_alloc = st.alloc_id();
    st.player_mut(who)
        .deck
        .push(arena_engine::CardInstance::from_card(card, id_alloc));
}

#[test]
fn set10_dragon_require_supported() {
    let db = load_db();
    for id in SET10_DRAGON {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    db.require_supported(cid(MISCHIEF))
        .unwrap_or_else(|e| panic!("{MISCHIEF}: {e}"));
}

#[test]
fn marauder_last_words_adds_evidence_not_on_banish() {
    let db = load_db();
    let mut st = started(&db, 110_418);
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, MARAUDER);
    put_field(&db, &mut st, opp, PAD);
    let hand0 = hand_count(&st, opp, EVIDENCE);
    play_pp(&db, &mut st, SMUGGLER, 7);
    assert_eq!(
        hand_count(&st, opp, EVIDENCE),
        hand0 + 1,
        "Marauder LW: Add a Divine Evidence"
    );

    // Negative: banish does not fire Last Words.
    let mut st = started(&db, 110_420);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, MARAUDER);
    let hand0 = hand_count(&st, opp, EVIDENCE);
    play_pp(&db, &mut st, "11064110", 6); // Lapinette Fanfare banishes
    assert_eq!(
        hand_count(&st, opp, EVIDENCE),
        hand0,
        "negative: banish does not fire Marauder Last Words"
    );
    let _ = me;
}

#[test]
fn collection_manifest_clue_discard_draws_base_cost_5() {
    let db = load_db();
    let mut st = started(&db, 110_421);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(me).deck.clear();
    for id in [PAD, PAD, "10002110", LEIZAUT] {
        push_deck(&db, &mut st, me, id);
    }
    put_hand(&db, &mut st, me, EVIDENCE);
    let hand_ev = hand_count(&st, me, EVIDENCE);
    play_pp(&db, &mut st, MANIFEST, 2);
    assert!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense
            < 10,
        "dealt 4 to the tank"
    );
    assert_eq!(hand_count(&st, me, EVIDENCE), hand_ev - 1, "discarded Clue");
    assert!(
        st.player(me).hand.iter().any(|c| c.base_cost >= 5),
        "drew a base cost ≥ 5"
    );

    // Negative: no Clue in hand → no draw.
    let mut st = started(&db, 110_422);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(me).deck.clear();
    push_deck(&db, &mut st, me, LEIZAUT);
    push_deck(&db, &mut st, me, LEIZAUT);
    let hand0 = st.player(me).hand.len();
    play_pp(&db, &mut st, MANIFEST, 2);
    assert_eq!(
        st.player(me).hand.len(),
        hand0,
        "negative: no Clue in hand → no draw"
    );
}

#[test]
fn freezing_icenewt_enhance_three_and_buff() {
    let db = load_db();
    let mut st = started(&db, 110_423);
    let me = PlayerId::A;
    for _ in 0..3 {
        put_field(&db, &mut st, PlayerId::B, PAD);
    }
    give_pp(&mut st, me, 7, 7);
    let h = put_hand(&db, &mut st, me, ICENEWT);
    play(&db, &mut st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    let ice = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ICENEWT)
        .unwrap();
    assert_eq!((ice.attack, ice.defense), (7, 7), "+5/+5 on Enhance");
    assert!(
        st.player(PlayerId::B).field.iter().flatten().count() < 3,
        "3×3 damage should kill some 1/1 pads"
    );

    let mut st = started(&db, 110_424);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, ICENEWT, 3);
    let ice = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ICENEWT)
        .unwrap();
    assert_eq!(
        (ice.attack, ice.defense),
        (2, 2),
        "negative: no enhance buff"
    );
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        7,
        "one hit of 3"
    );
}

#[test]
fn dance_overflow_gains_max_pp() {
    let db = load_db();
    let mut st = started(&db, 110_425);
    let me = PlayerId::A;
    st.player_mut(me).pp_max = 7;
    // Restore is a no-op at full defense — damage first so the heal is observable.
    st.player_mut(me).leader_defense = 19;
    let def0 = st.player(me).leader_defense;
    play_pp(&db, &mut st, DANCE, 3);
    assert_eq!(st.player(me).leader_defense, def0 + 1);
    assert_eq!(st.player(me).pp_max, 8, "Overflow → +1 max PP");

    let mut st = started(&db, 110_426);
    let me = PlayerId::A;
    st.player_mut(me).pp_max = 6;
    play_pp(&db, &mut st, DANCE, 3);
    assert_eq!(
        st.player(me).pp_max,
        6,
        "negative: not Overflow → no max PP"
    );
}

#[test]
fn dragonewt_scout_strike_evolves_other() {
    let db = load_db();
    let mut st = started(&db, 110_427);
    let me = PlayerId::A;
    let scout = put_field(&db, &mut st, me, SCOUT);
    let ally = put_field(&db, &mut st, me, PAD);
    put_field(&db, &mut st, PlayerId::B, PAD);
    if let Some(f) = st.field_inst_mut(me, scout) {
        f.flags.summoning_sick = false;
        f.traits.ambush = None;
        f.traits.storm = Some(true);
    }
    let ep0 = st.player(me).ep;
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(scout),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .expect("attack");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert!(
        st.field_inst(me, ally).unwrap().evolved,
        "Strike evolves another unevolved ally"
    );
    assert_eq!(st.player(me).ep, ep0, "effect evolve spends no EP");
    assert!(
        !st.field_inst(me, scout).unwrap().evolved,
        "negative: does not evolve self"
    );
}

#[test]
fn birds_eye_two_leftmost_and_fewer() {
    let db = load_db();
    let mut st = started(&db, 110_429);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, PAD); // synth vanilla 2/2
    put_field(&db, &mut st, me, PAD);
    put_field(&db, &mut st, me, TANK); // tank 2/10
    give_pp(&mut st, me, 3, 3);
    let h = put_hand(&db, &mut st, me, BIRDS);
    play(&db, &mut st, h);
    // Modes: 0 Mischief, 1 leftmost, 2 Evidence×3, 3 damage
    choose(&db, &mut st, 1); // leftmost
    choose(&db, &mut st, 1); // Evidence (old 2 → 1 after removal)
    let atks: Vec<i32> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() != MISCHIEF)
        .map(|c| c.attack)
        .collect();
    assert_eq!(atks[0], 3, "leftmost pad +1");
    assert_eq!(atks[1], 3, "second leftmost pad +1");
    assert_eq!(atks[2], 2, "rightmost tank unchanged");
    assert_eq!(hand_count(&st, me, EVIDENCE), 3);

    // Fewer than 2: only the one present is buffed.
    let mut st = started(&db, 110_430);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, PAD);
    give_pp(&mut st, me, 3, 3);
    let h = put_hand(&db, &mut st, me, BIRDS);
    play(&db, &mut st, h);
    choose(&db, &mut st, 1);
    choose(&db, &mut st, 2);
    assert_eq!(
        st.player(me).field.iter().flatten().next().unwrap().attack,
        3,
        "single ally still gets the leftmost buff"
    );
}

#[test]
fn mugger_clue_ramps() {
    let db = load_db();
    let mut st = started(&db, 110_431);
    let me = PlayerId::A;
    // Already at enough max PP to cast; Fanfare should raise it by 1 more.
    st.player_mut(me).pp_max = 4;
    put_hand(&db, &mut st, me, EVIDENCE);
    let max0 = st.player(me).pp_max;
    play_pp(&db, &mut st, MUGGER, 4);
    assert_eq!(st.player(me).pp_max, max0 + 1);
    assert_eq!(hand_count(&st, me, EVIDENCE), 0);

    let mut st = started(&db, 110_432);
    let me = PlayerId::A;
    st.player_mut(me).pp_max = 4;
    let max0 = st.player(me).pp_max;
    play_pp(&db, &mut st, MUGGER, 4);
    assert_eq!(st.player(me).pp_max, max0, "negative: no Clue → no ramp");
}

#[test]
fn jerva_x_zero_deals_nothing() {
    let db = load_db();
    let mut st = started(&db, 110_433);
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    assert_eq!(st.player(opp).leader_defense, st.player(opp).leader_max);
    play_pp(&db, &mut st, JERVA, 6);
    assert_eq!(
        st.player(opp)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        10,
        "Q&A: X=0 deals nothing at all"
    );

    let mut st = started(&db, 110_434);
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, PAD);
    st.player_mut(opp).leader_defense = 15; // X = 5
    play_pp(&db, &mut st, JERVA, 6);
    let sum: i32 = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .map(|c| c.defense)
        .sum();
    assert!(sum < 11, "X=5 split damages enemy followers; sum={sum}");
}

#[test]
fn leizaut_x_zero_one_three_clues() {
    let db = load_db();
    // X=0
    let mut st = started(&db, 110_435);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    st.player_mut(me).deck.clear();
    for id in [PAD, PAD, PAD, LEIZAUT] {
        push_deck(&db, &mut st, me, id);
    }
    play_pp(&db, &mut st, LEIZAUT, 10);
    assert_eq!(
        st.player(opp)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        10,
        "X=0: no damage loops"
    );
    assert!(st.player(me).deck.iter().all(|c| c.base_cost > 4));

    // X=1
    let mut st = started(&db, 110_436);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    put_hand(&db, &mut st, me, EVIDENCE);
    st.player_mut(me).deck.clear();
    for _ in 0..3 {
        push_deck(&db, &mut st, me, PAD);
    }
    play_pp(&db, &mut st, LEIZAUT, 10);
    assert_eq!(
        st.player(opp)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        2,
        "X=1: one deal of 8"
    );

    // X=3
    let mut st = started(&db, 110_437);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    for _ in 0..3 {
        put_field(&db, &mut st, opp, TANK);
    }
    for _ in 0..3 {
        put_hand(&db, &mut st, me, EVIDENCE);
    }
    st.player_mut(me).deck.clear();
    play_pp(&db, &mut st, LEIZAUT, 10);
    let alive = st.player(opp).field.iter().flatten().count();
    assert!(alive < 3, "X=3: three deals of 8; alive={alive}");
}
