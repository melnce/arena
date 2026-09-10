//! M2 wave 1 constructs: crystallize, hand-zone endOfTurn, summon from deck,
//! granted Last Words copy, split damage, necromancy evolve, Strike both leaders.

use arena_engine::{apply, Action, AttackTarget, Phase, PlayerId, Slot};

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
