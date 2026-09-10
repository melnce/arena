//! M2 wave 1 constructs: crystallize, hand-zone endOfTurn, summon from deck,
//! granted Last Words copy, split damage, necromancy evolve, Strike both leaders.

use arena_engine::{apply, legal_actions, snapshot, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

#[test]
fn hark_split_damage_oldest_first() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001320");
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.defense = 4;
        f.max_defense = 4;
    }
    if let Some(f) = st.field_inst_mut(opp, 1) {
        f.defense = 10;
        f.max_defense = 10;
    }
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10753310");
    assert!(
        !field_has(&st, opp, "88001110"),
        "oldest 4-def dies to the split"
    );
    let tank = st.player(opp).field[0].as_ref().unwrap();
    assert_eq!(tank.defense, 8, "remainder 2 spills to the next follower");
}

#[test]
fn garodeth_hand_end_of_turn_reduces_cost() {
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 12;
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10954120");
    assert_eq!(st.player(me).hand[h as usize].cost, 8);
    end_turn(&db, &mut st);
    // B's turn starts; cost drop already applied at A's end.
    assert_eq!(
        st.player(me)
            .hand
            .iter()
            .find(|c| c.card.as_str() == "10954120")
            .map(|c| c.cost),
        Some(7),
        "cost reduction persists on the hand instance"
    );
}

#[test]
fn garodeth_hand_end_of_turn_skips_when_defense_high() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 13;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10954120");
    end_turn(&db, &mut st);
    assert_eq!(
        st.player(me)
            .hand
            .iter()
            .find(|c| c.card.as_str() == "10954120")
            .map(|c| c.cost),
        Some(8)
    );
}

#[test]
fn adahime_summons_two_differently_named_from_deck() {
    let db = load_db();
    let mut st = started_decks(
        &db,
        4,
        &["10751120", "10751120", "10851120", "10851120"],
        &["88001110"],
    );
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    // leftover deck copies after opening / mulligan
    play_id(&db, &mut st, me, "10754110");
    let names: Vec<String> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .map(|c| c.card.as_str())
        .collect();
    assert!(names.contains(&"10754110".to_string()));
    let summoned: Vec<String> = names.into_iter().filter(|n| n != "10754110").collect();
    assert_eq!(summoned.len(), 2, "two deck summons");
    assert_ne!(summoned[0], summoned[1], "differently named");
    assert!(st.picks.iter().any(|p| {
        p.what == arena_engine::PickWhat::MultisetPick && p.among.as_deref() == Some("deck")
    }));
}

#[test]
fn adahime_field_full_skips_excess_no_rally() {
    let db = load_db();
    let mut st = started_decks(&db, 5, &["10751120", "10851120", "10951120"], &["88001110"]);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10754110");
    assert_eq!(field_count(&st, me), 5, "one slot: Adahime only");
    // played-follower Rally is deferred; failed summons add none
    assert!(
        !st.picks
            .iter()
            .any(|p| p.what == arena_engine::PickWhat::MultisetPick),
        "field-full skips the deck pick"
    );
}

#[test]
fn reapers_due_grants_last_words_copy() {
    let db = load_db();
    let mut st = started(&db, 6);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10953310");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    let snap = arena_engine::snapshot(&st);
    let granted = snap.players.a.field[slot as usize]
        .as_ref()
        .and_then(|f| f.granted.clone());
    assert_eq!(
        granted.as_deref(),
        Some(["lastWords".to_string()].as_slice())
    );
    // kill the host; Last Words summons a fresh print
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.defense = 0;
    }
    end_turn(&db, &mut st);
    // settle happens on the next quiet drain; force a no-op play path
    let copies = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "88001110")
        .count();
    assert!(
        copies >= 1,
        "granted Last Words summons a fresh printed copy"
    );
}

#[test]
fn void_colonel_crystallize_when_unaffordable() {
    let db = load_db();
    let mut st = started(&db, 7);
    let me = PlayerId::A;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    let rally_before = st.player(me).rally;
    play_id(&db, &mut st, me, "10952110");
    let f = st.player(me).field.iter().flatten().next().unwrap();
    assert_eq!(f.kind, arena_engine::card::CardKind::Amulet);
    assert_eq!(f.countdown, Some(4));
    assert_eq!(f.base_cost, 2);
    assert_eq!(
        st.player(me).rally,
        rally_before,
        "Crystallize play is an amulet and does not Rally"
    );
}

#[test]
fn void_colonel_normal_when_affordable() {
    let db = load_db();
    let mut st = started(&db, 8);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10952110");
    let f = st.player(me).field.iter().flatten().next().unwrap();
    assert_eq!(f.kind, arena_engine::card::CardKind::Follower);
    assert_eq!((f.attack, f.defense), (4, 6));
    assert!(f.traits.ward == Some(true));
}

#[test]
fn void_colonel_crystallize_last_words_summons_follower() {
    let db = load_db();
    let mut st = started(&db, 9);
    let me = PlayerId::A;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10952110");
    let slot = 0u8;
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).ok();
    // countdown to 0 at start of next own turn
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .any(|c| c.card.as_str() == "10952110"
                && c.kind == arena_engine::card::CardKind::Follower),
        "amulet Last Words summons a Void Colonel follower"
    );
}

#[test]
fn bibatii_necromancy_evolve_adds_depths() {
    let db = load_db();
    let mut st = started(&db, 10);
    let me = PlayerId::A;
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).shadows = 4;
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10654120");
    let f = st.player(me).field.iter().flatten().next().unwrap();
    assert!(f.evolved, "Necromancy (4) evolves");
    assert!(
        hand_has(&st, me, "90054330"),
        "anyEvolve fires on the effect-evolve"
    );
}

#[test]
fn lilith_strike_hits_both_leaders() {
    let db = load_db();
    let mut st = started(&db, 11);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "10851120");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.flags.summoning_sick = false;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .unwrap();
    assert_eq!(st.player(me).leader_defense, 19);
    assert_eq!(st.player(PlayerId::B).leader_defense, 18); // 1 strike + 1 combat
}

#[test]
fn netherworld_lieutenant_removes_last_words_from_the_copy() {
    let db = load_db();
    let mut st = started(&db, 12);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "10951120");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.defense = 0;
    }
    // settle deaths
    end_turn(&db, &mut st);
    let copy = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10951120")
        .expect("summoned copy");
    assert_eq!(copy.attack, 2);
    assert!(copy.traits.rush == Some(true));
    assert!(
        !copy.printed_tags.contains("lastWords"),
        "Last Words removed from the copy"
    );
}

#[test]
fn macmillan_fanfare_zombies_get_plus_one_rush_ward() {
    let db = load_db();
    let mut st = started(&db, 16);
    let me = PlayerId::A;
    give_pp(&mut st, me, 9, 9);
    st.player_mut(me).shadows = 10;
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10754120");
    let zombies: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "90051140")
        .collect();
    assert_eq!(zombies.len(), 3);
    for z in &zombies {
        assert_eq!(z.attack, 3, "Departed enter +1/+0 during your turn");
        assert_eq!(z.traits.rush, Some(true));
        assert_eq!(z.traits.ward, Some(true));
    }
    assert_eq!(st.player(PlayerId::B).leader_defense, 17);
}

#[test]
fn istyndet_crest_destroys_random_last_words_at_eot() {
    let db = load_db();
    let mut st = started(&db, 17);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10751120"); // Raz has Last Words
    put_field(&db, &mut st, opp, "88001110");
    st.player_mut(me)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: "crest:10954110".into(),
            countdown: None,
            faith: false,
            once_used: Vec::new(),
            granted_order: 0,
        });
    end_turn(&db, &mut st);
    assert!(
        !field_has(&st, me, "10751120"),
        "crest destroys a random allied Last Words card"
    );
    assert!(
        !field_has(&st, opp, "88001110"),
        "crest destroys a random enemy follower"
    );
}

#[test]
fn ghost_destroyed_is_banished_not_cemeteried() {
    let db = load_db();
    let mut st = started(&db, 13);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "90051130");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.defense = 0;
    }
    end_turn(&db, &mut st);
    assert!(
        st.player(me)
            .banished
            .iter()
            .any(|c| c.card.as_str() == "90051130"),
        "Ghost leave-banish"
    );
    assert!(
        !st.player(me)
            .cemetery
            .iter()
            .any(|c| c.card.as_str() == "90051130"),
        "Ghost does not cemetery"
    );
}

#[test]
fn adahime_does_not_give_herself_rush() {
    let db = load_db();
    let mut st = started_decks(&db, 14, &["10751120", "10951120"], &["88001110"]);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10754110");
    let ada = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10754110")
        .expect("Adahime");
    assert_ne!(ada.traits.rush, Some(true), "another allied — not self");
    for c in st.player(me).field.iter().flatten() {
        if c.card.as_str() != "10754110" {
            assert_eq!(
                c.traits.rush,
                Some(true),
                "{} summoned by Fanfare should get Rush",
                c.card
            );
        }
    }
}

#[test]
fn brew_on_full_field_is_not_playable() {
    let db = load_db();
    let mut st = started(&db, 18);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    put_field(&db, &mut st, me, "10031210");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    assert_eq!(field_count(&st, me), 5);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10031210");
    let legal = legal_actions(&db, &st);
    assert!(
        !legal.iter().any(|a| matches!(a, Action::Play { .. })),
        "full field: Brew is not playable"
    );
}

#[test]
fn gain_earth_sigil_on_full_board_with_brew_adds_to_stack() {
    let db = load_db();
    let mut st = started(&db, 19);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "10031210");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    assert_eq!(field_count(&st, me), 5);
    assert_eq!(st.player(me).earth, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001810");
    assert_eq!(st.player(me).earth, 2, "spell still raises the stack");
    assert!(
        !field_has(&st, me, "90031210"),
        "no Sediment appears; the Brew holds the stack"
    );
    assert!(field_has(&st, me, "10031210"));
}

#[test]
fn sigil_with_full_board_and_no_holder_is_lost_assumed() {
    // Assumption — owner has not ruled this half. Full board, no Earth Sigil
    // amulet: the Magic Sediment that would carry the gain cannot be summoned
    // (excess summons skipped), so earth stays 0.
    let db = load_db();
    let mut st = started(&db, 20);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    assert_eq!(field_count(&st, me), 5);
    assert_eq!(st.player(me).earth, 0);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001810");
    assert_eq!(
        st.player(me).earth,
        0,
        "no holder and no slot: gain is lost"
    );
    assert!(!field_has(&st, me, "90031210"));
}

#[test]
fn brew_engage_gains_a_sigil_and_stays() {
    let db = load_db();
    let mut st = started(&db, 15);
    let me = PlayerId::A;
    give_pp(&mut st, me, 2, 2);
    let slot = put_field(&db, &mut st, me, "10031210");
    assert_eq!(st.player(me).earth, 1);
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).unwrap();
    assert!(field_has(&st, me, "10031210"));
    assert_eq!(st.player(me).earth, 2);
    assert!(st.player(me).cemetery.is_empty());
}

#[test]
fn require_supported_abyss_closure() {
    let db = load_db();
    let ids = [
        "10052110", "10403110", "10403120", "10451120", "10452130", "10552110", "10651120",
        "10654120", "10751120", "10752110", "10753310", "10754110", "10754120", "10803310",
        "10851120", "10854110", "10951120", "10952110", "10953310", "10954110", "10954120",
        "90051110", "90051120", "90051130", "90051140", "90054330",
    ];
    for id in ids {
        db.require_supported(common::cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

fn se_raz(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId) -> u8 {
    let slot = put_field(db, st, who, "10751120");
    if let Some(f) = st.field_inst_mut(who, slot) {
        f.super_evolved = true;
        f.evolved = true;
        f.attack += 3;
        f.defense += 3;
        f.max_defense += 3;
        f.flags.summoning_sick = false;
    }
    slot
}

// ----- E31 -----

#[test]
fn se_follower_survives_ability_destroy_on_owners_turn() {
    // Official glossary, 2026-09-10 (Super-Evolution): "During your turn,
    // this follower can't be destroyed by abilities, and damage it takes is
    // reduced to 0." Rulebook Evolution stat bonuses: the follower stays a
    // legal random_target candidate; destroy fizzles.
    let db = load_db();
    let mut st = started(&db, 31);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    se_raz(&db, &mut st, me);
    se_raz(&db, &mut st, me);
    let col = put_field(&db, &mut st, opp, "10952110");
    if let Some(f) = st.field_inst_mut(opp, col) {
        f.defense = 1;
        f.max_defense = 1;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(0),
            target: AttackTarget::Slot(Slot(col)),
        },
    )
    .unwrap();
    assert_eq!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .filter(|c| c.card.as_str() == "10751120")
            .count(),
        2,
        "both SE Raz survive the Colonel's Last Words destroy"
    );
    assert!(
        st.picks
            .iter()
            .any(|p| p.what == arena_engine::PickWhat::RandomTarget),
        "SE follower remains in the destroy candidate pool"
    );
    assert!(
        !st.player(me)
            .cemetery
            .iter()
            .any(|c| c.card.as_str() == "10751120"),
        "destroy fizzled; Raz is not cemeteried"
    );
}

#[test]
fn se_follower_dies_to_ability_destroy_on_opponents_turn() {
    let db = load_db();
    let mut st = started(&db, 32);
    end_turn(&db, &mut st);
    let me = PlayerId::B;
    let opp = PlayerId::A;
    se_raz(&db, &mut st, opp);
    let col = put_field(&db, &mut st, me, "10952110");
    if let Some(f) = st.field_inst_mut(me, col) {
        f.defense = 1;
        f.max_defense = 1;
        f.flags.summoning_sick = false;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(col),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .unwrap();
    assert!(
        !field_has(&st, opp, "10751120"),
        "SE Raz is destroyable on the opponent's turn"
    );
    assert!(st
        .player(opp)
        .cemetery
        .iter()
        .any(|c| c.card.as_str() == "10751120"));
}

// ----- E32 -----

#[test]
fn last_words_raised_mid_wave_wait_behind_already_queued() {
    // Rulebook Grimnir example: resolving queued (1) queues Last Words (4);
    // already-queued (2) and (3) resolve before (4). Game-19 shape: Hark
    // kills B's Void Colonel, Lieutenant, and Bat together.
    let db = load_db();
    let mut st = started(&db, 33);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10952110");
    put_field(&db, &mut st, opp, "10952110");
    put_field(&db, &mut st, opp, "10951120");
    put_field(&db, &mut st, opp, "90051120");
    for slot in 0..3 {
        if let Some(f) = st.field_inst_mut(opp, slot) {
            f.defense = 1;
            f.max_defense = 1;
        }
    }
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10753310");
    let lt = st
        .player(opp)
        .cemetery
        .iter()
        .filter(|c| c.card.as_str() == "10951120")
        .count();
    assert_eq!(
        lt, 2,
        "Lieutenant summons its copy; A's Colonel Last Words then destroys that copy"
    );
    assert!(
        !field_has(&st, opp, "10951120"),
        "the summoned copy does not survive"
    );
}

// ----- E34 -----

#[test]
fn adahime_rush_waits_until_fanfare_choice_completes() {
    // Rulebook Fanfare and Enter-Play Trigger Order: other cards' enter
    // reactions drain after the played card's Fanfare, including across a
    // Fanfare choice (owner 2026-09-10 A2).
    let db = load_db();
    let mut st = started(&db, 34);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "10754110");
    give_pp(&mut st, me, 8, 8);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10954120");
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "Garodeth Fanfare offers a mode"
    );
    let snap = snapshot(&st);
    let g = snap.players.a.field[1]
        .as_ref()
        .expect("Garodeth at slot 1");
    assert_eq!(g.card, "10954120");
    assert!(
        !g.can_attack,
        "Adahime Rush has not resolved at the Fanfare choice"
    );
    assert!(
        !g.traits.iter().any(|t| t == "rush"),
        "no Rush at the choice node"
    );
    // Mode 2: Ward. Storm would also lift sickness; Ward does not.
    choose(&db, &mut st, 1);
    let snap = snapshot(&st);
    let g = snap.players.a.field[1]
        .as_ref()
        .expect("Garodeth after choice");
    assert!(
        g.traits.iter().any(|t| t == "rush"),
        "Adahime Rush after Fanfare completes"
    );
    assert!(g.can_attack, "Rush lifts summoning sickness");
    assert!(g.traits.iter().any(|t| t == "ward"));
}

// ----- E35 -----

fn destroyed(
    card: &str,
    base_cost: i32,
    owner: PlayerId,
) -> arena_engine::state::DestroyedRecord {
    arena_engine::state::DestroyedRecord {
        card: cid(card),
        base_cost,
        kind: arena_engine::card::CardKind::Follower,
        owner,
        from_field: true,
    }
}

#[test]
fn reanimate_grants_departed_and_macmillan_fires() {
    // Official glossary, 2026-09-10 (Reanimate): "Reanimate summons a copy
    // of the allied follower with the highest base cost destroyed that match
    // and gives it the Departed trait." Macmillan 10754120 then fires.
    // Game 26 i=120: reanimated Netherworld Lieutenant is 2/1 Rush Ward and
    // the enemy leader takes 1. The tribe is on the instance; a later printed
    // copy of the same card does not have it.
    let db = load_db();
    let mut st = started(&db, 35);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10754120");
    st.player_mut(me)
        .destroyed_history
        .push(destroyed("10951120", 2, me));
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001280");
    let f = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10951120")
        .expect("reanimated Lieutenant");
    assert_eq!(f.attack, 2, "Macmillan +1/+0 on the reanimated copy");
    assert_eq!(f.defense, 1);
    assert!(f.is_rush());
    assert!(f.is_ward());
    assert!(
        f.tribes.contains(&arena_engine::card::Tribe::Departed),
        "reanimated instance carries Departed"
    );
    assert_eq!(st.player(opp).leader_defense, 19);
    let printed = arena_engine::CardInstance::from_card(
        db.card(cid("10951120")).expect("Lieutenant"),
        0,
    );
    assert!(
        !printed
            .tribes
            .contains(&arena_engine::card::Tribe::Departed),
        "printed Lieutenant is not Departed; the tribe is instance-only"
    );
}

#[test]
fn reanimate_pick_weighted_by_destroyed_instances() {
    // Official glossary, 2026-09-10: "The more copies of a follower have been
    // destroyed, the more likely it is to be chosen." Two dead copies of A
    // and one of B at the same cost → A chosen ~2/3 over live RNG seeds.
    let db = load_db();
    let mut proto = started(&db, 36);
    let me = PlayerId::A;
    proto.player_mut(me).destroyed_history = vec![
        destroyed("88001110", 1, me),
        destroyed("88001110", 1, me),
        destroyed("88001150", 1, me),
    ];
    give_pp(&mut proto, me, 2, 2);
    proto.player_mut(me).hand.clear();
    put_hand(&db, &mut proto, me, "88001280");
    let mut a = 0u32;
    let mut b = 0u32;
    for seed in 0u64..300 {
        let mut st = proto.clone();
        arena_engine::reseed(&mut st, seed);
        play(&db, &mut st, 0);
        let id = st
            .player(me)
            .field
            .iter()
            .flatten()
            .find(|c| c.card.as_str() == "88001110" || c.card.as_str() == "88001150")
            .map(|c| c.card.as_str().to_string())
            .expect("reanimated one of the cost-1 followers");
        if id == "88001110" {
            a += 1;
        } else {
            b += 1;
        }
    }
    assert_eq!(a + b, 300);
    assert!(
        (180..=220).contains(&a),
        "A (two instances) chosen {a}/300, expected ~200 (2/3); B={b}"
    );
}
