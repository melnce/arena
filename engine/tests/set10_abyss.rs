//! Set 10 Abysscraft: ally_card_banished routes (Clue merge, Interrogation,
//! Ghost EoT, Ghost leave-redirect), Gremory Necromancy shadow order,
//! Nocturne / Solemn / Amanuensis.

use arena_engine::{apply, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const GHOST: &str = "90051130";
const SKEL: &str = "90051110";
const ZOMBIE: &str = "90051140";
const RUNAWAY: &str = "11051310";
const LAST_WILL: &str = "11052310";
const INTERROGATION: &str = "11053310";
const UNDERTAKER: &str = "11051110";
const GREMORY: &str = "11054120";
const CAPRA: &str = "11051120";
const SOLEMN: &str = "11052110";
const NOCTURNE: &str = "11054110";
const CENTAUR: &str = "11052120";
const AMANUENSIS: &str = "11053110";
const PAD: &str = "88001110";
const TANK: &str = "89500001";

const SET10_ABYSS: &[&str] = &[
    RUNAWAY,
    LAST_WILL,
    INTERROGATION,
    UNDERTAKER,
    GREMORY,
    CAPRA,
    SOLEMN,
    NOCTURNE,
    CENTAUR,
    AMANUENSIS,
];

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    let max = st.player(st.active).pp_max.max(pp);
    give_pp(st, st.active, pp, max);
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn field_has(st: &arena_engine::State, who: PlayerId, id: &str) -> bool {
    st.player(who)
        .field
        .iter()
        .flatten()
        .any(|c| c.card.as_str() == id)
}

fn field_count(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    st.player(who)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == id)
        .count()
}

fn hand_count(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    st.player(who)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == id)
        .count()
}

fn skeletons(st: &arena_engine::State, who: PlayerId) -> usize {
    field_count(st, who, SKEL)
}

#[test]
fn set10_abyss_require_supported() {
    let db = load_db();
    for id in SET10_ABYSS {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

#[test]
fn nocturne_four_banish_routes_during_your_turn() {
    let db = load_db();

    // Route 1: Clue merge banishes the old Evidence → Skeleton.
    let mut st = started(&db, 110_510);
    let me = PlayerId::A;
    play_pp(&db, &mut st, NOCTURNE, 6);
    let sk0 = skeletons(&st, me);
    put_field(&db, &mut st, me, EVIDENCE);
    play_pp(&db, &mut st, EVIDENCE, 1); // merge banishes the first
    assert_eq!(
        skeletons(&st, me),
        sk0 + 1,
        "Clue merge raises ally_card_banished → Skeleton"
    );
    let skel = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == SKEL)
        .unwrap();
    assert_eq!((skel.attack, skel.defense), (1 + 2, 1 + 4));
    assert_eq!(skel.traits.ward, Some(true));

    // Route 2: Nocturne's Interrogation banishes allied Clues.
    let mut st = started(&db, 110_511);
    let me = PlayerId::A;
    play_pp(&db, &mut st, NOCTURNE, 6);
    put_field(&db, &mut st, me, EVIDENCE);
    // Give the Evidence 2 clues so Interrogation deals something; still banishes.
    if let Some(c) = st
        .player_mut(me)
        .field
        .iter_mut()
        .flatten()
        .find(|c| c.is_clue())
    {
        c.clue_count = 2;
    }
    put_field(&db, &mut st, PlayerId::B, TANK);
    let sk0 = skeletons(&st, me);
    play_pp(&db, &mut st, INTERROGATION, 1);
    assert_eq!(
        skeletons(&st, me),
        sk0 + 1,
        "Interrogation banish raises ally_card_banished"
    );

    // Route 3: Ghost end-of-turn banish.
    let mut st = started(&db, 110_512);
    let me = PlayerId::A;
    play_pp(&db, &mut st, NOCTURNE, 6);
    // Fanfare already summoned a Ghost with Bane; EoT banishes it.
    let sk0 = skeletons(&st, me);
    assert!(field_has(&st, me, GHOST));
    end_turn(&db, &mut st); // A's EoT → Ghost banishes
                            // After end_turn we're on B's turn; Skeleton should have been summoned on A.
    assert_eq!(
        skeletons(&st, me),
        sk0 + 1,
        "Ghost EoT banish raises ally_card_banished"
    );

    // Route 4: Ghost leave-redirect (destroyed → banished instead) during your turn.
    let mut st = started(&db, 110_516);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, NOCTURNE);
    put_field(&db, &mut st, me, GHOST);
    let sk0 = skeletons(&st, me);
    // Deal 2 to all followers — Ghost (1/1) dies and leave-redirects to banish.
    // Arcane Eruption costs 4 (not Earth-gated for the AoE clause).
    play_pp(&db, &mut st, "10032310", 4);
    assert_eq!(
        skeletons(&st, me),
        sk0 + 1,
        "Ghost leave-redirect banish during your turn raises ally_card_banished"
    );

    // Negative: same leave-redirect on the opponent's turn does not fire Nocturne.
    let mut st = started(&db, 110_515);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, NOCTURNE);
    end_turn(&db, &mut st); // B's turn — place Ghost after A's EoT so it isn't EoT-banished
    let ghost = put_field(&db, &mut st, me, GHOST);
    let sk0 = skeletons(&st, me);
    let atk = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, atk) {
        f.flags.summoning_sick = false;
        f.attack = 5;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(ghost)),
        },
    )
    .expect("opp kills ghost");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert_eq!(
        skeletons(&st, me),
        sk0,
        "negative: leave-redirect banish on opponent's turn does not summon"
    );
}

#[test]
fn nocturne_no_trigger_on_opponents_turn_merge() {
    let db = load_db();
    let mut st = started(&db, 110_517);
    let me = PlayerId::A;
    play_pp(&db, &mut st, NOCTURNE, 6);
    end_turn(&db, &mut st); // B
                            // On B's turn, put Evidence on A's field somehow — can't play for A.
                            // Instead: B plays Scene of Deicide? Negative covered above for leave-redirect.
                            // Clue merge on B's side shouldn't affect Nocturne.
    let sk0 = skeletons(&st, me);
    put_field(&db, &mut st, PlayerId::B, EVIDENCE);
    play_pp(&db, &mut st, EVIDENCE, 1); // B merges B's clues
    assert_eq!(
        skeletons(&st, me),
        sk0,
        "negative: enemy Clue merge is not allied banish for Nocturne"
    );
}

#[test]
fn interrogation_x_zero_deals_nothing_then_banishes() {
    let db = load_db();
    let mut st = started(&db, 110_518);
    put_field(&db, &mut st, PlayerId::B, TANK);
    // No clues.
    play_pp(&db, &mut st, INTERROGATION, 1);
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        10,
        "X=0 deals nothing"
    );

    let mut st = started(&db, 110_519);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, EVIDENCE);
    if let Some(c) = st
        .player_mut(me)
        .field
        .iter_mut()
        .flatten()
        .find(|c| c.is_clue())
    {
        c.clue_count = 3;
    }
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, INTERROGATION, 1);
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        7,
        "X=3 before banish"
    );
    assert!(!field_has(&st, me, EVIDENCE), "Clues banished");
}

#[test]
fn gremory_own_destruction_shadow_enables_necromancy() {
    let db = load_db();
    // Q&A: 1 shadow before destroy → destruction adds 1 → Necromancy (2) draws.
    let mut st = started(&db, 110_521);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).shadows = 1;
    let g = put_field(&db, &mut st, me, GREMORY);
    let hand0 = st.player(me).hand.len();
    // Lethal combat from B on B's turn.
    end_turn(&db, &mut st);
    let atk = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, atk) {
        f.flags.summoning_sick = false;
        f.attack = 5;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(g)),
        },
    )
    .expect("kill gremory");
    assert_eq!(
        st.player(me).hand.len(),
        hand0 + 1,
        "Q&A: own destruction shadow enables Necromancy (2)"
    );

    // Negative: 0 shadows before → 1 after destroy → Necromancy (2) fails.
    let mut st = started(&db, 110_522);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).shadows = 0;
    let g = put_field(&db, &mut st, me, GREMORY);
    let hand0 = st.player(me).hand.len();
    end_turn(&db, &mut st);
    let atk = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, atk) {
        f.flags.summoning_sick = false;
        f.attack = 5;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(g)),
        },
    )
    .expect("kill gremory");
    assert_eq!(
        st.player(me).hand.len(),
        hand0,
        "negative: only 1 shadow after destroy → Necromancy (2) fails"
    );
}

#[test]
fn solemn_buffs_random_departed_on_banish() {
    let db = load_db();
    let mut st = started(&db, 110_523);
    let me = PlayerId::A;
    play_pp(&db, &mut st, SOLEMN, 5); // summons Zombie
    assert!(field_has(&st, me, ZOMBIE));
    let z_atk = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ZOMBIE)
        .unwrap()
        .attack;
    put_field(&db, &mut st, me, EVIDENCE);
    play_pp(&db, &mut st, EVIDENCE, 1); // merge banishes
    let z = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ZOMBIE)
        .unwrap();
    assert_eq!(z.attack, z_atk + 2);
    assert_eq!(z.traits.rush, Some(true));

    // Negative: no Departed → nothing.
    let mut st = started(&db, 110_524);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, SOLEMN); // no fanfare zombie via put_field
    let _ = slot;
    put_field(&db, &mut st, me, PAD);
    put_field(&db, &mut st, me, EVIDENCE);
    let pad_atk = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == PAD)
        .unwrap()
        .attack;
    play_pp(&db, &mut st, EVIDENCE, 1);
    let pad_atk2 = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == PAD)
        .map(|c| c.attack)
        .unwrap_or(pad_atk);
    assert_eq!(pad_atk2, pad_atk, "negative: non-Departed not buffed");
}

#[test]
fn runaway_last_will_undertaker_smoke() {
    let db = load_db();
    let mut st = started(&db, 110_525);
    let me = PlayerId::A;
    play_pp(&db, &mut st, RUNAWAY, 1);
    assert_eq!(hand_count(&st, me, GHOST), 1);

    play_pp(&db, &mut st, LAST_WILL, 1);
    assert!(field_has(&st, me, EVIDENCE));
    assert_eq!(hand_count(&st, me, EVIDENCE), 1);

    let mut st = started(&db, 110_526);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, UNDERTAKER);
    put_field(&db, &mut st, me, PAD);
    end_turn(&db, &mut st);
    give_pp(&mut st, PlayerId::B, 7, 7);
    let h = put_hand(&db, &mut st, PlayerId::B, "11042110");
    play(&db, &mut st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert!(
        field_has(&st, me, EVIDENCE) || hand_count(&st, me, EVIDENCE) >= 1,
        "Undertaker LW: summon + add Evidence"
    );
    let _ = slot;
}
