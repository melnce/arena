//! Alabaster Bahamut option 3 — banish all crests on both leaders, no Last Words.

use arena_engine::event::Event;
use arena_engine::{apply, Action, CardDb, Phase, PlayerId, Slot, State};

mod common;
use common::*;

const BAHAMUT: &str = "10804110";
const SATHANID_FAITH: &str = "faith:10614120";
const THESTAE: &str = "crest:10714110";
const GREAT_HART: &str = "crest:10714120";
const STARRY_SKY: &str = "crest:10412310";
const INSOMNIAC: &str = "crest:10532110";
const OPTION3: u8 = 2;

fn play_bahamut_option3(db: &CardDb, st: &mut State, me: PlayerId) -> Vec<Event> {
    give_pp(st, me, 9, 10);
    st.player_mut(me).hand.clear();
    play_id(db, st, me, BAHAMUT);
    let mut events = Vec::new();
    events.extend(apply(db, st, Action::Choose(OPTION3)).expect("mode choice"));
    while matches!(st.phase, Phase::Choice { .. }) {
        events.extend(apply(db, st, Action::Choose(0)).expect("drain choice"));
    }
    events
}

fn crest_remove_events(events: &[Event]) -> Vec<(PlayerId, String)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::CrestRemove { player, id } => Some((*player, id.clone())),
            _ => None,
        })
        .collect()
}

fn grant_evolve(db: &CardDb, st: &mut State, slot: u8) {
    let me = st.active;
    set_round(st, me, 5);
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

#[test]
fn option3_banishes_enemy_crests_keeps_enemy_faith() {
    let db = load_db();
    let mut st = started(&db, 10_804_101);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_faith_crest(&mut st, opp, SATHANID_FAITH, 4);
    put_crest_cd(&mut st, opp, THESTAE, 1);
    put_crest_cd(&mut st, opp, GREAT_HART, 2);
    let events = play_bahamut_option3(&db, &mut st, me);
    assert_eq!(crest_ids(&st, opp), vec![SATHANID_FAITH.to_string()]);
    assert_eq!(st.player(opp).faith, 4);
    assert_eq!(
        crest_remove_events(&events),
        vec![(opp, THESTAE.to_string()), (opp, GREAT_HART.to_string()),]
    );
}

#[test]
fn option3_banishes_both_sides_keeps_both_faiths() {
    let db = load_db();
    let mut st = started(&db, 10_804_102);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_faith_crest(&mut st, me, SATHANID_FAITH, 2);
    put_crest_cd(&mut st, me, STARRY_SKY, 3);
    put_faith_crest(&mut st, opp, SATHANID_FAITH, 5);
    put_crest_cd(&mut st, opp, THESTAE, 1);
    put_crest_cd(&mut st, opp, GREAT_HART, 2);
    let events = play_bahamut_option3(&db, &mut st, me);
    assert_eq!(crest_ids(&st, me), vec![SATHANID_FAITH.to_string()]);
    assert_eq!(crest_ids(&st, opp), vec![SATHANID_FAITH.to_string()]);
    assert_eq!(st.player(me).faith, 2);
    assert_eq!(st.player(opp).faith, 5);
    assert_eq!(crest_remove_events(&events).len(), 3);
}

#[test]
fn option3_banish_fires_no_crest_last_words() {
    let db = load_db();
    let mut st = started(&db, 10_804_103);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_crest(&mut st, me, STARRY_SKY);
    put_crest(&mut st, opp, INSOMNIAC);
    let opp_hp_before = st.player(opp).leader_defense;
    play_bahamut_option3(&db, &mut st, me);
    assert!(st.player(me).crests.is_empty());
    assert!(st.player(opp).crests.is_empty());
    assert_eq!(field_atk(&st, me, BAHAMUT), Some(13));
    assert_eq!(field_def(&st, me, BAHAMUT), Some(13));
    assert_eq!(st.player(opp).leader_defense, opp_hp_before);
    assert!(!hand_has(&st, me, "10412310"));
}

#[test]
fn destroy_your_crest_still_fires_last_words() {
    let db = load_db();
    let mut st = started(&db, 10_804_104);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10532110");
    assert!(st.player(me).crests.iter().any(|c| c.id == INSOMNIAC));
    let slot = st
        .player(me)
        .field
        .iter()
        .flatten()
        .position(|c| c.card.as_str() == "10532110")
        .unwrap() as u8;
    grant_evolve(&db, &mut st, slot);
    assert!(!st.player(me).crests.iter().any(|c| c.id == INSOMNIAC));
    assert_eq!(
        field_count(&st, opp),
        0,
        "vanilla took 3 from crest Last Words"
    );
    assert_eq!(
        field_def(&st, me, "10532110"),
        Some(2),
        "evolved 5/5 Insomniac Witch took 3 from crest Last Words"
    );
}