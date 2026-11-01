//! Set 10 Abysscraft: ally_card_banished routes (Clue merge, Interrogation,
//! Ghost EoT, Ghost leave-redirect), Gremory Necromancy shadow order,
//! Nocturne / Solemn / Amanuensis / Capra / Centaur.

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
const SCENE_OF_DEICIDE: &str = "11002210";
const PAD: &str = "88001110"; // 2/2
const TANK: &str = "89500001"; // 2/10
const GREMORY_CREST: &str = "crest:11054120";

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

fn evolve_now(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8, super_evo: bool) {
    let me = st.active;
    st.player_mut(me).turns_taken = st.player(me).turns_taken.max(5);
    if super_evo {
        st.player_mut(me).turns_taken = st.player(me).turns_taken.max(7);
        st.player_mut(me).sep = 1;
    } else {
        st.player_mut(me).ep = 1;
    }
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: super_evo,
        },
    )
    .expect("evolve");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
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
fn vengeful_capra_last_words_damages_leader() {
    let db = load_db();
    let mut st = started(&db, 110_501);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let c = put_field(&db, &mut st, me, CAPRA);
    let def0 = st.player(me).leader_defense;
    // Kill Capra on A's turn via enemy attack after end_turn.
    end_turn(&db, &mut st);
    let atk = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, atk) {
        f.flags.summoning_sick = false;
        f.attack = 7;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(c)),
        },
    )
    .expect("kill capra");
    assert_eq!(
        st.player(me).leader_defense,
        def0 - 2,
        "Last Words: Deal 2 damage to your leader"
    );

    // Negative: banish does not fire Last Words.
    let mut st = started(&db, 110_502);
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, CAPRA);
    let def0 = st.player(opp).leader_defense;
    play_pp(&db, &mut st, "11064110", 6); // Lapinette banishes enemy Capra
    assert_eq!(
        st.player(opp).leader_defense,
        def0,
        "negative: banish does not fire Capra Last Words"
    );
}

#[test]
fn restive_centaur_necromancy_strike_and_below_four_negative() {
    let db = load_db();
    let mut st = started(&db, 110_503);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).shadows = 4;
    let cent = put_field(&db, &mut st, me, CENTAUR);
    assert_eq!(
        st.field_inst(me, cent).unwrap().traits.rush,
        Some(true),
        "Rush"
    );
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, PAD);
    if let Some(f) = st.field_inst_mut(me, cent) {
        f.flags.summoning_sick = false;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(cent),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .expect("strike");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert_eq!(st.player(me).shadows, 0, "Necromancy (4) spent 4");
    // Tank 10−4=6 (if still alive after combat); pad 2−4 dies to AoE.
    // Combat: Centaur 6 atk into Tank 2/10 — tank takes 6 → 4, then Strike AoE −4 → 0.
    assert_eq!(
        st.player(opp).field.iter().flatten().count(),
        0,
        "Strike Necromancy (4): Deal 4 to all enemy followers"
    );

    // Negative: below 4 shadows → no AoE.
    let mut st = started(&db, 110_504);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).shadows = 3;
    let cent = put_field(&db, &mut st, me, CENTAUR);
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, PAD);
    if let Some(f) = st.field_inst_mut(me, cent) {
        f.flags.summoning_sick = false;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(cent),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .expect("strike");
    assert_eq!(st.player(me).shadows, 3, "negative: below 4 → no spend");
    // Tank takes only combat 6 → def 4; pad untouched at 2.
    let defs: Vec<(String, i32)> = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .map(|c| (c.card.as_str().to_string(), c.defense))
        .collect();
    assert!(
        defs.iter().any(|(id, d)| id == TANK && *d == 4),
        "tank combat only: {defs:?}"
    );
    assert!(
        defs.iter().any(|(id, d)| id == PAD && *d == 2),
        "pad untouched without Necromancy: {defs:?}"
    );
}

#[test]
fn attending_amanuensis_banish_restore_fanfare_bane_super_evolve() {
    let db = load_db();
    let mut st = started(&db, 110_505);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    play_pp(&db, &mut st, AMANUENSIS, 7);
    let ghost = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == GHOST)
        .expect("Fanfare Ghost");
    assert_eq!(ghost.traits.bane, Some(true), "Fanfare: Ghost + Bane");

    // Banish allied card during your turn → restore 2.
    let def0 = st.player(me).leader_defense;
    put_field(&db, &mut st, me, EVIDENCE);
    play_pp(&db, &mut st, EVIDENCE, 1); // merge banishes
    assert_eq!(st.player(me).leader_defense, def0 + 2, "banish → restore 2");

    // Super-Evolve replicates Fanfare → another Ghost with Bane.
    let mut st = started(&db, 110_506);
    let me = PlayerId::A;
    let a = put_field(&db, &mut st, me, AMANUENSIS);
    let g0 = field_count(&st, me, GHOST);
    evolve_now(&db, &mut st, a, true);
    assert_eq!(
        field_count(&st, me, GHOST),
        g0 + 1,
        "Super-Evolve replicates Fanfare Ghost"
    );
    assert!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .filter(|c| c.card.as_str() == GHOST)
            .all(|c| c.traits.bane == Some(true)),
        "replicated Ghost has Bane"
    );

    // Negative: banish on opponent's turn does not restore.
    let mut st = started(&db, 110_507);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, AMANUENSIS);
    st.player_mut(me).leader_defense = 10;
    let def0 = st.player(me).leader_defense;
    end_turn(&db, &mut st); // B's turn
    put_field(&db, &mut st, me, GHOST);
    // Ghost EoT will fire at end of B's turn — wait, Ghost EoT is "at the end of your turn"
    // (Ghost's owner). Put Evidence merge isn't available on B. Kill Ghost via attack:
    let ghost = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == GHOST))
        .unwrap() as u8;
    let atk = put_field(&db, &mut st, PlayerId::B, TANK);
    if let Some(f) = st.field_inst_mut(PlayerId::B, atk) {
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
    .expect("kill ghost → leave-redirect banish");
    assert_eq!(
        st.player(me).leader_defense,
        def0,
        "negative: opponent's-turn banish does not restore"
    );
}

#[test]
fn gremory_evolve_crest_eot_shadows_and_ward() {
    let db = load_db();
    let mut st = started(&db, 110_508);
    let me = PlayerId::A;
    let g = put_field(&db, &mut st, me, GREMORY);
    assert_eq!(
        st.field_inst(me, g).unwrap().traits.ward,
        Some(true),
        "Ward"
    );
    let sh0 = st.player(me).shadows;
    evolve_now(&db, &mut st, g, false);
    assert!(
        st.player(me).crests.iter().any(|c| c.id == GREMORY_CREST),
        "Evolve → Crest: Gremory"
    );
    end_turn(&db, &mut st); // crest EoT on A's end
    assert_eq!(st.player(me).shadows, sh0 + 2, "crest EoT: gain 2 shadows");

    // Negative: without Evolve, no crest EoT shadows.
    let mut st = started(&db, 110_509);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, GREMORY);
    let sh0 = st.player(me).shadows;
    end_turn(&db, &mut st);
    assert_eq!(
        st.player(me).shadows,
        sh0,
        "negative: no crest → no EoT shadows"
    );
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
fn nocturne_four_banish_routes_during_your_turn() {
    let db = load_db();

    // Route 1: Clue merge banishes the old Evidence → Skeleton.
    let mut st = started(&db, 110_510);
    let me = PlayerId::A;
    play_pp(&db, &mut st, NOCTURNE, 6);
    let ghost = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == GHOST)
        .expect("Fanfare Ghost");
    assert_eq!(ghost.traits.bane, Some(true), "Fanfare: Ghost + Bane");
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
    let sk0 = skeletons(&st, me);
    assert!(field_has(&st, me, GHOST));
    end_turn(&db, &mut st); // A's EoT → Ghost banishes
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
    play_pp(&db, &mut st, "10032310", 4); // Arcane Eruption
    assert_eq!(
        skeletons(&st, me),
        sk0 + 1,
        "Ghost leave-redirect banish during your turn raises ally_card_banished"
    );

    // Evolve: Add a Last Will and Testament to your hand.
    let mut st = started(&db, 110_5161);
    let me = PlayerId::A;
    let n = put_field(&db, &mut st, me, NOCTURNE);
    let h0 = hand_count(&st, me, LAST_WILL);
    evolve_now(&db, &mut st, n, false);
    assert_eq!(
        hand_count(&st, me, LAST_WILL),
        h0 + 1,
        "Evolve: Add Last Will and Testament"
    );
}

#[test]
fn nocturne_opponents_turn_negative_via_scene_of_deicide() {
    let db = load_db();
    // Scene of Deicide at B's EoT: banishes an enemy follower and summons enemy
    // Divine Evidence on A's field. If A already has Evidence, the merge banishes
    // the old Clue during B's turn — Nocturne must not summon.
    let mut st = started(&db, 110_515);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, NOCTURNE);
    put_field(&db, &mut st, me, EVIDENCE);
    put_field(&db, &mut st, me, PAD); // Scene needs an enemy follower to banish
    end_turn(&db, &mut st); // B
    play_pp(&db, &mut st, SCENE_OF_DEICIDE, 6);
    let sk0 = skeletons(&st, me);
    end_turn(&db, &mut st); // B's EoT: Scene fires, may merge Evidence on A
    assert_eq!(
        skeletons(&st, me),
        sk0,
        "negative: Scene-of-Deicide merge/banish on opponent's turn does not summon"
    );

    // Also: Ghost leave-redirect on opponent's turn.
    let mut st = started(&db, 110_5151);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, NOCTURNE);
    end_turn(&db, &mut st);
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
fn interrogation_x_zero_barrier_stays_then_banishes() {
    let db = load_db();
    // X=0 with Barrier: Barrier stays, no damage.
    let mut st = started(&db, 110_518);
    let opp = PlayerId::B;
    let t = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, t) {
        f.traits.barrier = Some(true);
    }
    play_pp(&db, &mut st, INTERROGATION, 1);
    let f = st.field_inst(opp, t).unwrap();
    assert_eq!(f.defense, 10, "X=0 deals nothing");
    assert_eq!(
        f.traits.barrier,
        Some(true),
        "X=0: Barrier stays like Jerva"
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
        "X=3 before banish → 10−3=7"
    );
    assert!(!field_has(&st, me, EVIDENCE), "Clues banished");
}

#[test]
fn solemn_buffs_plus_two_defense_during_your_turn_and_evolve() {
    let db = load_db();
    let mut st = started(&db, 110_523);
    let me = PlayerId::A;
    play_pp(&db, &mut st, SOLEMN, 5); // summons Zombie
    assert!(field_has(&st, me, ZOMBIE));
    let (z_atk, z_def) = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ZOMBIE)
        .map(|c| (c.attack, c.defense))
        .unwrap();
    put_field(&db, &mut st, me, EVIDENCE);
    play_pp(&db, &mut st, EVIDENCE, 1); // merge banishes
    let z = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ZOMBIE)
        .unwrap();
    assert_eq!(z.attack, z_atk + 2, "+2 attack");
    assert_eq!(z.defense, z_def + 2, "+2 defense");
    assert_eq!(z.traits.rush, Some(true));

    // Evolve: Summon a Divine Evidence.
    let mut st = started(&db, 110_5231);
    let me = PlayerId::A;
    let s = put_field(&db, &mut st, me, SOLEMN);
    let e0 = field_count(&st, me, EVIDENCE);
    evolve_now(&db, &mut st, s, false);
    assert_eq!(
        field_count(&st, me, EVIDENCE),
        e0 + 1,
        "Evolve: Summon a Divine Evidence"
    );

    // Negative: banish on opponent's turn does not buff.
    let mut st = started(&db, 110_524);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    play_pp(&db, &mut st, SOLEMN, 5);
    let z_atk = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ZOMBIE)
        .unwrap()
        .attack;
    end_turn(&db, &mut st); // B's turn
    let ghost = put_field(&db, &mut st, me, GHOST);
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
    .expect("opp kills ghost → leave-redirect banish on B's turn");
    let z_atk2 = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ZOMBIE)
        .map(|c| c.attack)
        .unwrap_or(z_atk);
    assert_eq!(
        z_atk2, z_atk,
        "negative: During your turn — no buff on opponent's-turn banish"
    );
}

#[test]
fn runaway_last_will_negatives_and_undertaker_both() {
    let db = load_db();
    // Runaway: Add a Ghost to your hand.
    let mut st = started(&db, 110_525);
    let me = PlayerId::A;
    let h0 = hand_count(&st, me, GHOST);
    play_pp(&db, &mut st, RUNAWAY, 1);
    assert_eq!(hand_count(&st, me, GHOST), h0 + 1);
    // Negative: does not summon Ghost to field.
    assert!(
        !field_has(&st, me, GHOST),
        "negative: Runaway adds to hand, does not summon"
    );

    // Last Will: Summon Evidence + add Evidence.
    let mut st = started(&db, 110_5251);
    let me = PlayerId::A;
    let hand0 = hand_count(&st, me, EVIDENCE);
    play_pp(&db, &mut st, LAST_WILL, 1);
    assert!(field_has(&st, me, EVIDENCE), "summons Evidence");
    assert_eq!(
        hand_count(&st, me, EVIDENCE),
        hand0 + 1,
        "adds Evidence to hand"
    );
    // Negative: not just one or the other — both zones get one.
    assert!(
        field_count(&st, me, EVIDENCE) >= 1 && hand_count(&st, me, EVIDENCE) >= 1,
        "negative guard: both summon AND add"
    );

    // Undertaker LW: BOTH summon AND add (not ||).
    let mut st = started(&db, 110_526);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let slot = put_field(&db, &mut st, me, UNDERTAKER);
    end_turn(&db, &mut st);
    let atk = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, atk) {
        f.flags.summoning_sick = false;
        f.attack = 5;
    }
    let hand0 = hand_count(&st, me, EVIDENCE);
    let field0 = field_count(&st, me, EVIDENCE);
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(slot)),
        },
    )
    .expect("kill undertaker");
    assert!(
        field_count(&st, me, EVIDENCE) >= field0 + 1,
        "Undertaker LW: Summon a Divine Evidence"
    );
    assert!(
        hand_count(&st, me, EVIDENCE) >= hand0 + 1,
        "Undertaker LW: Add a Divine Evidence to your hand"
    );
}
