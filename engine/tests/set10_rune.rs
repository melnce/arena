//! Set 10 Runecraft — Discerning Melody, Interrogation / Noeinhowell crests,
//! Leisurely Attendant, Infernal Chimera, Ruler of Tempests.

use arena_engine::{apply, Action, Phase, PlayerId, Slot};
use arena_engine::state::LeaderMod;

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

    // Fanfare summons Evidence; field Engage restores 1.
    let mut st = started(&db, 10_331);
    play_pp(&db, &mut st, ATTENDANT, 4);
    assert!(field_has(&st, PlayerId::A, EVIDENCE), "Fanfare summons Evidence");
    st.player_mut(PlayerId::A).leader_defense = 10;
    // Clear Evidence (engaged away) and Engage a helper while Attendant is on field.
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref().is_some_and(|c| c.card.as_str() != ATTENDANT) {
            *s = None;
        }
    }
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        leader_def(&st, PlayerId::A),
        11,
        "field Engage restores 1 defense to leader"
    );

    // Negative: Engage on opponent's turn does not restore for "whenever you Engage".
    // (Engage is only legal on your turn — assert no restore when Attendant is absent.)
    let mut st = started(&db, 10_332);
    st.player_mut(PlayerId::A).leader_defense = 10;
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        leader_def(&st, PlayerId::A),
        10,
        "negative: without Attendant on field, Engage does not restore"
    );
}

#[test]
fn infernal_chimera_split_all_enemies_exact_allocations() {
    let db = load_db();
    // X=3, one 1-defense follower: oldest-first fill → 1 to it, 2 to leader.
    let mut st = started(&db, 10_340);
    st.player_mut(PlayerId::A).earth = 3;
    let slot = put_field(&db, &mut st, PlayerId::B, TANK);
    st.field_inst_mut(PlayerId::B, slot).unwrap().defense = 1;
    st.field_inst_mut(PlayerId::B, slot).unwrap().max_defense = 1;
    let leader0 = leader_def(&st, PlayerId::B);
    play_pp(&db, &mut st, CHIMERA, 8);
    assert!(
        !field_has(&st, PlayerId::B, TANK),
        "1-defense follower takes 1 and dies"
    );
    assert_eq!(
        leader_def(&st, PlayerId::B),
        leader0 - 2,
        "remaining 2 of X=3 goes to the leader (all enemies, not followers-only)"
    );

    // No followers: all X to the leader.
    let mut st = started(&db, 10_341);
    st.player_mut(PlayerId::A).earth = 3;
    let leader0 = leader_def(&st, PlayerId::B);
    play_pp(&db, &mut st, CHIMERA, 8);
    assert_eq!(
        leader_def(&st, PlayerId::B),
        leader0 - 3,
        "no enemy followers → all X to the leader"
    );

    // X=0 with Barrier follower + leader that takes 1 more damage → nothing changes.
    let mut st = started(&db, 10_342);
    assert_eq!(st.player(PlayerId::A).earth, 0);
    let slot = put_field(&db, &mut st, PlayerId::B, TANK);
    st.field_inst_mut(PlayerId::B, slot).unwrap().traits.barrier = Some(true);
    st.player_mut(PlayerId::B).leader_mods.push(LeaderMod {
        max_defense: None,
        damage_cap: None,
        damage_taken_bonus: 1,
        until: None,
    });
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    let leader0 = leader_def(&st, PlayerId::B);
    let barrier0 = st
        .field_inst(PlayerId::B, slot)
        .unwrap()
        .traits
        .barrier;
    play_pp(&db, &mut st, CHIMERA, 8);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0),
        "Q&A: X=0 deals no follower damage"
    );
    assert_eq!(
        leader_def(&st, PlayerId::B),
        leader0,
        "Q&A: X=0 deals no leader damage even with takes-1-more"
    );
    assert_eq!(
        st.field_inst(PlayerId::B, slot).unwrap().traits.barrier,
        barrier0,
        "negative: X=0 does not consume Barrier"
    );
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

    // Negative: non-Golem ally enter does not deal 3.
    let mut st = started(&db, 10_351);
    put_field(&db, &mut st, PlayerId::A, SEDIMENT);
    st.player_mut(PlayerId::A).earth = 1;
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, RULER, 6);
    let tank_after = field_def(&st, PlayerId::B, TANK).unwrap();
    put_field(&db, &mut st, PlayerId::A, AIDE); // non-Golem enter
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank_after),
        "negative: non-Golem allied enter does not deal 3"
    );

    // Negative: enemy Golem enter does not deal 3.
    let mut st = started(&db, 10_352);
    put_field(&db, &mut st, PlayerId::A, SEDIMENT);
    st.player_mut(PlayerId::A).earth = 1;
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, RULER, 6);
    let tank_after = field_def(&st, PlayerId::B, TANK).unwrap();
    put_field(&db, &mut st, PlayerId::B, GOLEM);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank_after),
        "negative: enemy Golem enter does not trigger"
    );

    // Evolve replicates Fanfare Earth Rite.
    let mut st = started(&db, 10_353);
    put_field(&db, &mut st, PlayerId::A, SEDIMENT);
    st.player_mut(PlayerId::A).earth = 2;
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, RULER, 6);
    let golems_after_ff = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == GOLEM)
        .count();
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == RULER)
                .map(|_| i as u8)
        })
        .unwrap();
    evolve_slot_local(&db, &mut st, slot, false);
    let golems = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == GOLEM)
        .count();
    assert!(
        golems > golems_after_ff,
        "Evolve replicates Fanfare → another Golem"
    );
}

fn evolve_slot_local(
    db: &arena_engine::CardDb,
    st: &mut arena_engine::State,
    slot: u8,
    supered: bool,
) {
    let me = st.active;
    if supered {
        if st.player(me).turns_taken < 7 {
            set_round(st, me, 7);
        }
        st.player_mut(me).sep = st.player(me).sep.max(1);
    } else {
        if st.player(me).turns_taken < 5 {
            set_round(st, me, 5);
        }
        st.player_mut(me).ep = st.player(me).ep.max(1);
    }
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: supered,
        },
    )
    .expect("evolve");
    drain_choice(db, st);
}

#[test]
fn dedicated_aide_fanfare_evidence_engage_plus_1_1() {
    let db = load_db();
    let mut st = started(&db, 10_360);
    play_pp(&db, &mut st, AIDE, 3);
    assert!(field_has(&st, PlayerId::A, EVIDENCE), "Fanfare summons Evidence");
    let atk0 = field_atk(&st, PlayerId::A, AIDE).unwrap();
    let def0 = field_def(&st, PlayerId::A, AIDE).unwrap();
    // Evidence was summoned — Engage it (or a helper if Evidence was consumed).
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.kind == arena_engine::CardKind::Amulet)
                .map(|_| i as u8)
        })
        .unwrap_or_else(|| put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER));
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        field_atk(&st, PlayerId::A, AIDE),
        Some(atk0 + 1),
        "Engage gives Aide +1/+1"
    );
    assert_eq!(
        field_def(&st, PlayerId::A, AIDE),
        Some(def0 + 1),
        "Engage gives Aide +1/+1"
    );

    // Negative: no Engage → stats stay printed after Fanfare.
    let mut st = started(&db, 10_361);
    play_pp(&db, &mut st, AIDE, 3);
    assert_eq!(field_atk(&st, PlayerId::A, AIDE), Some(atk0));
    assert_eq!(field_def(&st, PlayerId::A, AIDE), Some(def0));
}

#[test]
fn wily_witch_both_modes() {
    let db = load_db();
    // Mode 1: Earth Rite (1) — deal 2 to a random enemy follower.
    let mut st = started(&db, 10_370);
    st.player_mut(PlayerId::A).earth = 1;
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    give_pp(&mut st, PlayerId::A, 2, 2);
    let h = put_hand(&db, &mut st, PlayerId::A, WITCH);
    play(&db, &mut st, h);
    // Choose mode 0 (Earth Rite).
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 2),
        "mode 1: Earth Rite deals 2"
    );
    assert_eq!(st.player(PlayerId::A).earth, 0, "Earth Rite spends 1 earth");

    // Mode 2: Gain 2 earth sigils.
    let mut st = started(&db, 10_371);
    assert_eq!(st.player(PlayerId::A).earth, 0);
    give_pp(&mut st, PlayerId::A, 2, 2);
    let h = put_hand(&db, &mut st, PlayerId::A, WITCH);
    play(&db, &mut st, h);
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 1);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        st.player(PlayerId::A).earth,
        2,
        "mode 2: Gain 2 earth sigils"
    );

    // Negative: mode 1 with 0 earth deals nothing.
    let mut st = started(&db, 10_372);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    give_pp(&mut st, PlayerId::A, 2, 2);
    let h = put_hand(&db, &mut st, PlayerId::A, WITCH);
    play(&db, &mut st, h);
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        drain_choice(&db, &mut st);
    }
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0),
        "negative: Earth Rite with 0 earth deals no damage"
    );
}

#[test]
fn atelier_relocation_enhance_8_gives_storm_through_binding() {
    let db = load_db();
    const CLAY: &str = "90031110";
    // Base: 3 Clay Golems, no Storm.
    let mut st = started(&db, 10_380);
    play_pp(&db, &mut st, RELOCATION, 4);
    let golems: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == CLAY)
        .collect();
    assert_eq!(golems.len(), 3, "summons 3 Clay Golems");
    assert!(
        golems.iter().all(|c| c.traits.storm != Some(true)),
        "negative: without Enhance, no Storm"
    );

    // Enhance (8): Give them Storm via the summon binding.
    let mut st = started(&db, 10_381);
    play_pp(&db, &mut st, RELOCATION, 8);
    let golems: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == CLAY)
        .collect();
    assert_eq!(golems.len(), 3);
    assert!(
        golems.iter().all(|c| c.traits.storm == Some(true)),
        "Enhance (8): Give them Storm (bound group)"
    );
}

#[test]
fn dramatic_assistant_engage_deals_5_evolve_recovers_1_pp() {
    let db = load_db();
    let mut st = started(&db, 10_390);
    play_pp(&db, &mut st, ASSISTANT, 5);
    assert!(field_has(&st, PlayerId::A, AIDE), "Fanfare summons Dedicated Aide");
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    // Clear Evidence from Aide Fanfare if present; Engage a helper.
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref()
            .is_some_and(|c| c.card.as_str() != ASSISTANT && c.card.as_str() != AIDE)
        {
            *s = None;
        }
    }
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 5),
        "Engage deals 5 to a random enemy follower"
    );

    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == ASSISTANT)
                .map(|_| i as u8)
        })
        .unwrap();
    // set_round inside evolve_slot_local sets pp=turns; re-seat after to observe recover.
    if st.player(PlayerId::A).turns_taken < 5 {
        set_round(&mut st, PlayerId::A, 5);
    }
    st.player_mut(PlayerId::A).ep = st.player(PlayerId::A).ep.max(1);
    st.player_mut(PlayerId::A).pp_max = 10;
    st.player_mut(PlayerId::A).pp = 3;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve");
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(PlayerId::A).pp,
        4,
        "Evolve: Recover 1 play point"
    );

    // Negative: without an enemy follower, Engage deals nothing (leader untouched).
    let mut st = started(&db, 10_391);
    play_pp(&db, &mut st, ASSISTANT, 5);
    let leader0 = leader_def(&st, PlayerId::B);
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref()
            .is_some_and(|c| c.card.as_str() != ASSISTANT && c.card.as_str() != AIDE)
        {
            *s = None;
        }
    }
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        leader_def(&st, PlayerId::B),
        leader0,
        "negative: Engage 5 damage targets followers only"
    );
}

#[test]
fn noeinhowell_engage_deals_3_to_random_enemy_follower() {
    let db = load_db();
    let mut st = started(&db, 10_395);
    play_pp(&db, &mut st, NOEINHOWELL, 7);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let tank0 = field_def(&st, PlayerId::B, TANK).unwrap();
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref().is_some_and(|c| c.card.as_str() != NOEINHOWELL) {
            *s = None;
        }
    }
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        field_def(&st, PlayerId::B, TANK),
        Some(tank0 - 3),
        "Engage: deal 3 to a random enemy follower"
    );

    // Negative: no enemy follower → leader untouched by this trigger.
    let mut st = started(&db, 10_396);
    play_pp(&db, &mut st, NOEINHOWELL, 7);
    let leader0 = leader_def(&st, PlayerId::B);
    for s in st.player_mut(PlayerId::A).field.iter_mut() {
        if s.as_ref().is_some_and(|c| c.card.as_str() != NOEINHOWELL) {
            *s = None;
        }
    }
    let slot = put_field(&db, &mut st, PlayerId::A, ENGAGE_HELPER);
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        leader_def(&st, PlayerId::B),
        leader0,
        "negative: Engage 3 damage is follower-only"
    );
}
