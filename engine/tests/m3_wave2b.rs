//! M3 wave 2b — Abysscraft + Portalcraft fixtures and `require_supported`.

use arena_engine::card::{CardKind, Tribe};
use arena_engine::{apply, Action, AttackTarget, CardInstance, Phase, PlayerId, Slot};

mod common;
use common::*;

const ABYSS: &[&str] = &[
    "10451110", "10451310", "10452110", "10452120", "10453110", "10453310", "10454110", "10454120",
    "10551110", "10551120", "10551310", "10552120", "10552310", "10553110", "10553310", "10651110",
    "10651310", "10652110", "10652120", "10652310", "10653110", "10653310", "10751110", "10751310",
    "10752120", "10752310", "10753110", "10851110", "10851130", "10852110", "10852120", "10852310",
    "10853110", "10853310", "10854120", "10951110", "10951310", "10952120", "10952310", "10953110",
];

const PORTAL: &[&str] = &[
    "10471110", "10472110", "10472120", "10472310", "10473110", "10473310", "10474110", "10474120",
    "10571110", "10571120", "10571310", "10572110", "10572120", "10572310", "10573110", "10671120",
    "10671310", "10672120", "10672310", "10673310", "10771120", "10772120", "10772310", "10773310",
    "10871110", "10871120", "10871130", "10872110", "10872120", "10872310", "10873110", "10873310",
    "10971110", "10971120", "10971310", "10972110", "10972120", "10972310", "10973110", "10973310",
    "10974110", "90071160",
];

#[test]
fn require_supported_abyss_portal_wave2b() {
    let db = load_db();
    for id in ABYSS.iter().chain(PORTAL) {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

fn destroyed(card: &str, base_cost: i32, owner: PlayerId) -> arena_engine::state::DestroyedRecord {
    arena_engine::state::DestroyedRecord {
        card: cid(card),
        base_cost,
        kind: CardKind::Follower,
        owner,
        from_field: true,
    }
}

fn push_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).expect(id);
    let inst = CardInstance::from_card(card, st.alloc_id());
    st.player_mut(who).deck.push(inst);
}

/// Fediel's double reanimate + evolve with Departed.
#[test]
fn fediel_double_reanimate_evolve_departed() {
    let db = load_db();
    let mut st = started(&db, 101);
    let me = PlayerId::A;
    st.player_mut(me).destroyed_history =
        vec![destroyed("10551110", 2, me), destroyed("88001110", 1, me)];
    st.player_mut(me).shadows = 6;
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10454110");
    let wolf = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10551110")
        .expect("reanimate 2");
    assert!(wolf.evolved);
    assert!(wolf.tribes.contains(&Tribe::Departed));
    let van = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001110")
        .expect("reanimate 1");
    assert!(van.evolved);
    assert!(van.tribes.contains(&Tribe::Departed));
}

/// Tyrannical Fists at 19/20 and at 20/20.
#[test]
fn tyrannical_fists_lowest_leaders_ties() {
    let db = load_db();
    let mut st = started(&db, 102);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).leader_defense = 19;
    st.player_mut(opp).leader_defense = 20;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10552310");
    assert_eq!(st.player(me).leader_defense, 16);
    assert_eq!(st.player(opp).leader_defense, 20);

    st.player_mut(me).leader_defense = 20;
    st.player_mut(opp).leader_defense = 20;
    give_pp(&mut st, me, 2, 2);
    play_id(&db, &mut st, me, "10552310");
    assert_eq!(st.player(me).leader_defense, 17);
    assert_eq!(st.player(opp).leader_defense, 17);
}

/// Corruption's two crests and the super-tier self-destroy.
#[test]
fn corruption_two_crests_super_destroys_own() {
    let db = load_db();
    let mut st = started(&db, 103);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.turn = 15;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10453310");
    assert!(
        st.player(me)
            .crests
            .iter()
            .all(|c| c.id != "crest:10453310"),
        "super skybound destroys your Corruption crest"
    );
    assert!(
        st.player(opp)
            .crests
            .iter()
            .any(|c| c.id == "crest:10453310"),
        "opponent keeps their Corruption crest"
    );
}

/// Belial's crest reaching 0 via SE and dealing 20.
#[test]
fn belial_crest_se_countdown_deals_20() {
    let db = load_db();
    let mut st = started(&db, 104);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.turn = 15;
    st.player_mut(me).turns_taken = 15;
    st.player_mut(me).sep = 1;
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10454120");
    assert!(st
        .player(me)
        .crests
        .iter()
        .any(|c| c.id == "crest:10454120"));
    if let Some(c) = st
        .player_mut(me)
        .crests
        .iter_mut()
        .find(|c| c.id == "crest:10454120")
    {
        c.countdown = Some(1);
    }
    let slot = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == "10454120"))
        .expect("Belial") as u8;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .unwrap();
    assert!(st
        .player(me)
        .crests
        .iter()
        .all(|c| c.id != "crest:10454120"));
    assert_eq!(st.player(opp).leader_defense, 0);
}

/// Lifestealer's transform keeping Skeletons' printed keywords and the restore
/// on either side's Skeleton death.
#[test]
fn lifestealer_transform_keywords_and_restore_either_side() {
    let db = load_db();
    let mut st = started(&db, 105);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let ally = put_field(&db, &mut st, me, "88001110");
    if let Some(f) = st.field_inst_mut(me, ally) {
        f.traits.storm = Some(true);
    }
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 9, 9);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10553110");
    let sk = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "90051110")
        .expect("ally skeleton");
    assert!(!sk.is_storm(), "plain printed Skeleton, no leftover Storm");
    assert!(sk.tribes.contains(&Tribe::Departed));
    assert_eq!((sk.attack, sk.defense), (1, 1));
    assert!(field_has(&st, opp, "90051110"));

    st.player_mut(me).leader_defense = 10;
    let ally_sk = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == "90051110"))
        .unwrap() as u8;
    if let Some(f) = st.field_inst_mut(me, ally_sk) {
        f.defense = 0;
    }
    end_turn(&db, &mut st);
    assert_eq!(st.player(me).leader_defense, 11, "ally Skeleton death");

    // B's turn: kill B's Skeleton so A's enemy_follower_destroyed fires.
    let enemy_sk = st
        .player(opp)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == "90051110"))
        .expect("enemy skeleton") as u8;
    if let Some(f) = st.field_inst_mut(opp, enemy_sk) {
        f.defense = 0;
    }
    end_turn(&db, &mut st);
    assert_eq!(st.player(me).leader_defense, 12, "enemy Skeleton death");
}

/// Allure's exact copy after banish.
#[test]
fn allure_exact_copy_after_banish() {
    let db = load_db();
    let mut st = started(&db, 106);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let slot = put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, slot) {
        f.attack = 5;
        f.defense = 5;
        f.max_defense = 5;
    }
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10652310");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(!field_has(&st, opp, "88001110"));
    let copy = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001110")
        .expect("exact copy");
    assert_eq!((copy.attack, copy.defense), (5, 5));
}

/// Deprived Destroyer's "if you selected one" gate with and without a selection.
#[test]
fn deprived_destroyer_did_gate() {
    let db = load_db();
    let mut st = started(&db, 107);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10653110");
    let d = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10653110")
        .unwrap();
    assert!(!d.evolved, "no selection → no evolve");
    assert!(!field_has(&st, me, "90051120"));

    let mut st = started(&db, 108);
    put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10653110");
    choose(&db, &mut st, 0);
    assert!(!field_has(&st, me, "88001110"));
    let d = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10653110")
        .unwrap();
    assert!(d.evolved);
    let bat = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "90051120")
        .expect("evolved Bat");
    assert!(bat.evolved);
}

/// Bittersweet Departures' two modes in listed order.
#[test]
fn bittersweet_departures_pick_two_listed_order() {
    let db = load_db();
    let mut st = started(&db, 109);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    st.player_mut(me).shadows = 0;
    play_id(&db, &mut st, me, "10852310");
    // pick mode 4 (shadows) first, then mode 1 (1 damage) — resolve listed order.
    choose(&db, &mut st, 3);
    choose(&db, &mut st, 0);
    assert_eq!(st.player(opp).leader_defense, 19);
    assert_eq!(st.player(me).shadows, 5, "1 shadow from the spell + Gain 4");
}

/// Rigor's same-cost check after the draw.
#[test]
fn rigor_same_cost_after_draw() {
    let db = load_db();
    let mut st = started(&db, 110);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    st.player_mut(me).deck.clear();
    put_hand(&db, &mut st, me, "10551110");
    put_hand(&db, &mut st, me, "10551110");
    put_hand(&db, &mut st, me, "10551110");
    for _ in 0..8 {
        push_deck(&db, &mut st, me, "10551110");
    }
    give_pp(&mut st, me, 2, 2);
    play_id(&db, &mut st, me, "10553310");
    end_turn(&db, &mut st);
    let sk = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "90051110")
        .expect("Skeleton after 4-of-a-cost");
    assert!(sk.is_ward());
}

/// Ebb and Flow's cost −1 under super-evolution.
#[test]
fn ebb_and_flow_cost_minus_one_under_se() {
    let db = load_db();
    let mut st = started(&db, 111);
    let me = PlayerId::A;
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).hand.clear();
    st.player_mut(me).deck.clear();
    put_hand(&db, &mut st, me, "88001110");
    for _ in 0..8 {
        push_deck(&db, &mut st, me, "10551110");
    }
    give_pp(&mut st, me, 2, 2);
    play_id(&db, &mut st, me, "10853310");
    choose(&db, &mut st, 0);
    let drawn: Vec<_> = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == "10551110")
        .collect();
    assert_eq!(drawn.len(), 2);
    assert!(drawn.iter().all(|c| c.cost == 1), "SE unlocked → cost −1");
}

/// Cassius with no Artifact in hand (0 damage, ability still resolves).
#[test]
fn cassius_no_artifact_in_hand_zero_damage() {
    let db = load_db();
    let mut st = started(&db, 112);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10473110");
    assert!(field_has(&st, me, "10473110"), "Fanfare still activates");
    assert!(
        field_has(&st, opp, "88001110"),
        "0 damage when the Artifact pick is empty"
    );
}

/// Beelzebub's "Takes 1 more damage" stacking twice and applying to a
/// super-evolved follower's destroy damage.
#[test]
fn beelzebub_damage_taken_stacks_and_se_knockback() {
    let db = load_db();
    let mut st = started(&db, 113);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 9, 9);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10474120");
    give_pp(&mut st, me, 9, 9);
    play_id(&db, &mut st, me, "10474120");
    assert_eq!(st.player(opp).damage_taken_bonus(), 2);

    let prey = put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, prey) {
        f.defense = 1;
        f.max_defense = 1;
    }
    let att = put_field(&db, &mut st, me, "88001110");
    if let Some(f) = st.field_inst_mut(me, att) {
        f.super_evolved = true;
        f.flags.summoning_sick = false;
        f.attack = 1;
    }
    let before = st.player(opp).leader_defense;
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(att),
            target: AttackTarget::Slot(Slot(prey)),
        },
    )
    .unwrap();
    assert_eq!(
        st.player(opp).leader_defense,
        before - 3,
        "SE knockback 1 + two Takes-1-more = 3"
    );
}

/// Lu Woh's enemy-hand +1/+0 and his crest's -3/-0 on a Storm attacker before combat.
#[test]
fn lu_woh_enemy_hand_and_crest_storm_leader_attack() {
    let db = load_db();
    let mut st = started(&db, 114);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.turn = 10;
    put_hand(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10474110");
    let storm = put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, storm) {
        f.traits.storm = Some(true);
        f.attack = 4;
        f.defense = 4;
        f.max_defense = 4;
        f.flags.summoning_sick = false;
    }
    let h = st
        .player(opp)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "88001110")
        .expect("hand follower");
    assert_eq!(h.attack, 3, "enemy-hand +1/+0");
    assert!(st
        .player(me)
        .crests
        .iter()
        .any(|c| c.id == "crest:10474110"));
    end_turn(&db, &mut st);
    let before = st.player(me).leader_defense;
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(storm),
            target: AttackTarget::Leader,
        },
    )
    .unwrap();
    let att = st.field_inst(opp, storm).expect("attacker");
    assert_eq!(att.attack, 1, "crest −3/−0 before combat");
    assert_eq!(st.player(me).leader_defense, before - 1);
}

/// Warp Slash's distinct-names X.
#[test]
fn warp_slash_distinct_artifact_names() {
    let db = load_db();
    let mut st = started(&db, 115);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).enter_counts.insert(cid("90073110"), 2);
    st.player_mut(me).enter_counts.insert(cid("90071160"), 1);
    put_field(&db, &mut st, opp, "88001320");
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10773310");
    let tank = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001320")
        .expect("tank");
    assert_eq!(tank.defense, tank.max_defense - 2, "X = 2 distinct names");
    assert_eq!(st.player(opp).leader_defense, 19);
}

/// New-Age Cartographer's exact copy from hand.
#[test]
fn new_age_cartographer_exact_copy_from_hand() {
    let db = load_db();
    let mut st = started(&db, 116);
    let me = PlayerId::A;
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    let slot = put_field(&db, &mut st, me, "10572110");
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90073120");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .unwrap();
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(
        st.player(me)
            .hand
            .iter()
            .any(|c| c.card.as_str() == "90073120"),
        "hand instance stays"
    );
    assert!(field_has(&st, me, "90073120"));
}

/// Cutthroat's banish-then-crest with a duplicate left in the deck (no crest).
#[test]
fn cutthroat_banish_then_no_crest_if_duplicate_left() {
    let db = load_db();
    let mut st = started(&db, 117);
    let me = PlayerId::A;
    st.player_mut(me).ep = 1;
    st.player_mut(me).turns_taken = 5;
    let slot = put_field(&db, &mut st, me, "10974110");
    st.player_mut(me).deck.clear();
    push_deck(&db, &mut st, me, "10974110");
    push_deck(&db, &mut st, me, "88001110");
    push_deck(&db, &mut st, me, "88001110");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .unwrap();
    assert!(st
        .player(me)
        .deck
        .iter()
        .all(|c| c.card.as_str() != "10974110"));
    assert!(
        st.player(me)
            .crests
            .iter()
            .all(|c| c.id != "crest:10974110"),
        "duplicate vanilla copies remain"
    );
}

/// Cutthroat crest: "Once on each of your turns, when you play a follower, evolve it."
/// Owner ruling: `oncePerTurn: true` + `when: {turnOwner: "self"}` — only during YOUR turns, once each.
#[test]
fn cutthroat_crest_once_per_your_turn() {
    let db = load_db();
    let mut st = started(&db, 121);
    let me = PlayerId::A;
    st.player_mut(me)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: "crest:10974110".into(),
            countdown: None,
            faith: false,
            once_used: Vec::new(),
            granted_order: 0,
            granted: vec![],
        });
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001110");
    let first = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001110")
        .expect("first play");
    assert!(first.evolved, "first follower play this turn evolves");
    play_id(&db, &mut st, me, "88001110");
    let unevolved = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "88001110" && !c.evolved)
        .count();
    assert_eq!(unevolved, 1, "second play same turn is not evolved");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    give_pp(&mut st, me, 1, 1);
    play_id(&db, &mut st, me, "88001110");
    let evolved = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "88001110" && c.evolved)
        .count();
    assert_eq!(evolved, 2, "oncePerTurn resets on the next of your turns");
}

/// Lunar Bunny evolving before the played spell resolves (E39).
#[test]
fn lunar_bunny_evolves_before_spell_body() {
    let db = load_db();
    let mut st = started(&db, 118);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "10572120");
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10453310");
    let bunny = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10572120")
        .expect("Bunny survives −2/−2 only if it evolved first");
    assert!(bunny.evolved);
    assert_eq!(bunny.defense, 1);
}

/// Kratos's copy without Last Words.
#[test]
fn kratos_copy_without_last_words() {
    let db = load_db();
    let mut st = started(&db, 119);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "10871110");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.defense = 0;
    }
    end_turn(&db, &mut st);
    let copy = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10871110")
        .expect("summoned Kratos");
    assert!(
        !copy.printed_tags.contains("lastWords"),
        "Last Words removed from the copy"
    );
}

/// Leona's Ambush surviving an evolve without damage.
#[test]
fn leona_ambush_survives_evolve_without_damage() {
    let db = load_db();
    let mut st = started(&db, 120);
    let me = PlayerId::A;
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    st.player_mut(me).ep = 1;
    let ally = put_field(&db, &mut st, me, "88001110");
    let leo = put_field(&db, &mut st, me, "10871120");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(leo),
            super_evolve: true,
        },
    )
    .unwrap();
    choose(&db, &mut st, ally);
    let f = st.field_inst(me, ally).unwrap();
    assert_eq!(f.traits.ambush, Some(true));
    assert!(f.flags.ambush_active);
    st.player_mut(me).evolved_this_turn = false;
    st.player_mut(me).ep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(ally),
            super_evolve: false,
        },
    )
    .unwrap();
    let f = st.field_inst(me, ally).unwrap();
    assert!(f.evolved);
    assert_eq!(
        f.traits.ambush,
        Some(true),
        "Ambush remains until the follower deals damage"
    );
    assert!(f.flags.ambush_active);
}

#[test]
fn self_consistency_abyss_pool_20_seeds() {
    let db = load_db();
    let path = "oracle/decks/abyss-pool.json";
    let decks = load_deck_file(path);
    assert_eq!(decks.len(), 40);
    assert!(deck_ready(&db, &decks));
    for g in 0u64..20 {
        emit_and_replay(&db, 20260910 + g, path);
    }
}

#[test]
fn self_consistency_portal_pool_20_seeds() {
    let db = load_db();
    let path = "oracle/decks/portal-pool.json";
    let decks = load_deck_file(path);
    assert_eq!(decks.len(), 40);
    assert!(deck_ready(&db, &decks));
    for g in 0u64..20 {
        emit_and_replay(&db, 20260910 + g, path);
    }
}

fn emit_and_replay(db: &arena_engine::CardDb, seed: u64, deck_path: &str) {
    use arena_engine::{
        from_neutral, legal_actions, new_game, policy_rng, snapshot_json, to_neutral, First,
        GameConfig, GameRng, OpeningHands,
    };
    let decks = load_deck_file(deck_path);
    let mut live = new_game(
        db,
        GameConfig {
            seed,
            deck_a: decks.clone(),
            deck_b: decks.clone(),
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let opening = OpeningHands {
        a: live
            .player(PlayerId::A)
            .hand
            .iter()
            .map(|c| c.card)
            .collect(),
        b: live
            .player(PlayerId::B)
            .hand
            .iter()
            .map(|c| c.card)
            .collect(),
    };
    let mut policy = policy_rng(seed);
    let mut recs = Vec::new();
    let mut i = 0u32;
    while live.winner.is_none() && !matches!(live.phase, Phase::Terminal) && i < 800 {
        let legal = legal_actions(db, &live);
        if legal.is_empty() {
            break;
        }
        let idx = policy.gen_range(legal.len() as u32) as usize;
        let action = legal[idx].clone();
        let neu = to_neutral(&live, &action);
        apply(db, &mut live, action).unwrap();
        recs.push((neu, live.picks.clone(), snapshot_json(&live)));
        i += 1;
    }
    let mut replay = new_game(
        db,
        GameConfig {
            seed,
            deck_a: decks.clone(),
            deck_b: decks,
            first: First::A,
            opening_hands: Some(opening),
        },
    )
    .unwrap();
    for (i, (neu, picks, snap)) in recs.iter().enumerate() {
        replay.rng = GameRng::scripted(picks.clone(), seed);
        let act = from_neutral(&replay, neu).expect("from_neutral");
        apply(db, &mut replay, act).unwrap_or_else(|e| panic!("replay seed={seed} i={i}: {e}"));
        let got = snapshot_json(&replay);
        if let Some((path, a, b)) = arena_engine::replay_state_diff(&got, snap) {
            panic!("seed {seed} i={i} {path}: arena={a} trace={b}");
        }
    }
}
