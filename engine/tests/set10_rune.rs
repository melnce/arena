//! Set 10 Runecraft — Discerning Melody, Interrogation / Noeinhowell crests,
//! Leisurely Attendant, Infernal Chimera, Ruler of Tempests.

use arena_engine::{apply, Action, Phase, PlayerId, Slot};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const AIDE: &str = "11031110";
const WITCH: &str = "11031120";
const RELOCATION: &str = "11031310";
const ATTENDANT: &str = "11032110";
const RULER: &str = "11032120";
const MELODY: &str = "11032310";
const ASSISTANT: &str = "11033110";
const INTERROGATION: &str = "11033310";
const INTERROGATION_CREST: &str = "crest:11033310";
const NOEINHOWELL: &str = "11034110";
const NOEINHOWELL_CREST: &str = "crest:11034110";
const CHIMERA: &str = "11034120";
const GOLEM: &str = "90031120"; // Guardian Golem
const SEDIMENT: &str = "90031210";
const TANK: &str = "89500001";
const ENGAGE_HELPER: &str = "89500021";

const SET10_RUNE: &[&str] = &[
    AIDE,
    WITCH,
    RELOCATION,
    ATTENDANT,
    RULER,
    MELODY,
    ASSISTANT,
    INTERROGATION,
    NOEINHOWELL,
    CHIMERA,
];

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    give_pp(st, st.active, pp, pp.max(10));
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn engage_slot(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    give_pp(st, st.active, 10, 10);
    apply(db, st, Action::Engage { slot: Slot(slot) }).expect("engage");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

#[test]
fn set10_rune_require_supported() {
    let db = load_db();
    for id in SET10_RUNE {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    for crest in [INTERROGATION_CREST, NOEINHOWELL_CREST] {
        db.crest(crest).unwrap_or_else(|e| panic!("{crest}: {e}"));
    }
}

#[test]
fn discerning_melody_damage_requires_engaged_amulet_this_turn() {
    let db = load_db();
    // Negative: without Engage, summons Evidence but no damage.
    let mut st = started(&db, 10_300);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    play_pp(&db, &mut st, MELODY, 1);
    assert!(field_has(&st, PlayerId::A, EVIDENCE));
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0),
        "negative: no Engage this turn → no Melody damage"
    );

    // With Engage: deal 3 to a random enemy follower.
    let mut st = started(&db, 10_301);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    assert!(st.player(PlayerId::A).engaged_amulet_this_turn);
    play_pp(&db, &mut st, MELODY, 1);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 3),
        "engagedAmuletThisTurn → 3 damage"
    );
}

#[test]
fn interrogation_crest_three_engages_then_destroyed_fourth_noop() {
    let db = load_db();
    let mut st = started(&db, 10_310);
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(PlayerId::A).leader_defense = 10;
    play_pp(&db, &mut st, INTERROGATION, 2);
    assert!(hand_has(&st, PlayerId::A, EVIDENCE));
    assert!(st
        .player(PlayerId::A)
        .crests
        .iter()
        .any(|c| c.id == INTERROGATION_CREST));

    // 1st Engage: deal 3 to random enemy follower.
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 3),
        "ability 1: deal 3"
    );

    // 2nd Engage: restore 2 to leader.
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    let def0 = leader_def(&st, PlayerId::A);
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        leader_def(&st, PlayerId::A),
        def0 + 2,
        "ability 2: restore 2"
    );

    // 3rd Engage: recover 1 EP and destroy crest.
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    let ep0 = st.player(PlayerId::A).ep;
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        st.player(PlayerId::A).ep,
        ep0 + 1,
        "ability 3: recover 1 evolution point"
    );
    assert!(
        st.player(PlayerId::A)
            .crests
            .iter()
            .all(|c| c.id != INTERROGATION_CREST),
        "third activation destroys the crest"
    );

    // Negative: fourth Engage does nothing (crest gone).
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank1 = field_def(&st, PlayerId::B, TANK).unwrap();
    let def1 = leader_def(&st, PlayerId::A);
    let ep1 = st.player(PlayerId::A).ep;
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank1),
        "negative: fourth Engage deals no crest damage"
    );
    assert_eq!(leader_def(&st, PlayerId::A), def1);
    assert_eq!(st.player(PlayerId::A).ep, ep1);
}

#[test]
fn noeinhowell_crest_once_per_turn_pp_recover_not_above_max() {
    let db = load_db();
    let mut st = started(&db, 10_320);
    play_pp(&db, &mut st, NOEINHOWELL, 4);
    let slot_body = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == NOEINHOWELL)
                .map(|_| i as u8)
        })
        .expect("body");
    // Gain crest via Evolve.
    if st.player(PlayerId::A).turns_taken < 5 {
        set_round(&mut st, PlayerId::A, 5);
    }
    st.player_mut(PlayerId::A).ep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot_body),
            super_evolve: false,
        },
    )
    .expect("evolve");
    assert!(st
        .player(PlayerId::A)
        .crests
        .iter()
        .any(|c| c.id == NOEINHOWELL_CREST));

    // Clear Evidence so later Engages have field room.
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref().is_some_and(|c| c.card.as_str() != NOEINHOWELL) {
            *s = None;
        }
    }

    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    st.player_mut(PlayerId::A).pp_max = 10;
    st.player_mut(PlayerId::A).pp = 4;
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).expect("engage");
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(PlayerId::A).pp,
        5,
        "once per your turn: Engage recovers 1 PP"
    );

    // Clear Evidence / helpers so the field has room for fresh Engage amulets.
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref().is_some_and(|c| c.card.as_str() != NOEINHOWELL) {
            *s = None;
        }
    }

    // Second Engage same turn: no second recover.
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    st.player_mut(PlayerId::A).pp = 5;
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).expect("engage2");
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(PlayerId::A).pp,
        5,
        "negative: second Engage same turn does not recover again"
    );

    // Next own turn: recovers again.
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref().is_some_and(|c| c.card.as_str() != NOEINHOWELL) {
            *s = None;
        }
    }
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    st.player_mut(PlayerId::A).pp = st.player(PlayerId::A).pp_max - 2;
    let before = st.player(PlayerId::A).pp;
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).unwrap();
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(PlayerId::A).pp,
        before + 1,
        "recovers again next your turn"
    );

    // Negative: not above max PP.
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref().is_some_and(|c| c.card.as_str() != NOEINHOWELL) {
            *s = None;
        }
    }
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    let max = st.player(PlayerId::A).pp_max;
    st.player_mut(PlayerId::A).pp = max;
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).unwrap();
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(PlayerId::A).pp,
        max,
        "negative: recover must not exceed max PP"
    );
}

#[test]
fn leisurely_attendant_in_hand_each_engage_lowers_cost() {
    let db = load_db();
    let mut st = started(&db, 10_330);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, ATTENDANT);
    assert_eq!(st.player(PlayerId::A).hand[0].cost, 4, "printed cost 4");
    for expected in [3, 2, 1] {
        let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
        engage_slot(&db, &mut st, slot);
        let c = st
            .player(PlayerId::A)
            .hand
            .iter()
            .find(|c| c.card.as_str() == ATTENDANT)
            .expect("attendant still in hand");
        assert_eq!(
            c.cost, expected,
            "each Engage lowers cost by 1 → {expected}"
        );
    }
    // Negative: cost change is in hand only — a freshly put Attendant is still 4.
    let h = put_hand(&db, &mut st, PlayerId::A, ATTENDANT);
    assert_eq!(
        st.player(PlayerId::A).hand[h as usize].cost,
        4,
        "negative: a new copy is not pre-discounted"
    );
}

#[test]
fn infernal_chimera_x0_no_damage_with_earth_splits_including_leader() {
    let db = load_db();
    // Q&A: X=0 → no damage at all.
    let mut st = started(&db, 10_340);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    let leader0 = leader_def(&st, PlayerId::B);
    assert_eq!(st.player(PlayerId::A).earth, 0);
    play_pp(&db, &mut st, CHIMERA, 8);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0),
        "Q&A: X=0 deals no damage to followers"
    );
    assert_eq!(
        leader_def(&st, PlayerId::B),
        leader0,
        "Q&A: X=0 deals no damage to leader"
    );

    // With earth: split X among all enemies including leader.
    let mut st = started(&db, 10_341);
    put_field(&db, &mut st, PlayerId::A, SEDIMENT);
    st.player_mut(PlayerId::A).earth = 3;
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    let leader0 = leader_def(&st, PlayerId::B);
    play_pp(&db, &mut st, CHIMERA, 8);
    let tank_dmg = tank0 - field_def(&st, PlayerId::B, TANK).unwrap_or(0);
    let leader_dmg = leader0 - leader_def(&st, PlayerId::B);
    assert_eq!(
        tank_dmg + leader_dmg,
        3,
        "X=3 earth split across enemies incl leader; tank={tank_dmg} leader={leader_dmg}"
    );
    assert!(tank_dmg > 0 || leader_dmg > 0, "some damage must land");
}

#[test]
fn ruler_of_tempests_golem_enter_deals_3_after_play_sequence() {
    let db = load_db();
    let mut st = started(&db, 10_350);
    put_field(&db, &mut st, PlayerId::A, SEDIMENT);
    st.player_mut(PlayerId::A).earth = 1;
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    play_pp(&db, &mut st, RULER, 6);
    assert!(
        field_has(&st, PlayerId::A, GOLEM) || field_has(&st, PlayerId::A, "90031120"),
        "Earth Rite summons Guardian Golem"
    );
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 3),
        "Golem enter deals 3 after play sequence finishes"
    );
    // Negative: without earth, Fanfare Golem skips → no enter damage from that summon.
    let mut st = started(&db, 10_351);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    assert_eq!(st.player(PlayerId::A).earth, 0);
    play_pp(&db, &mut st, RULER, 6);
    assert!(
        !field_has(&st, PlayerId::A, GOLEM),
        "negative: no earth → no Golem"
    );
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0),
        "negative: no Golem enter → no 3 damage"
    );
}
