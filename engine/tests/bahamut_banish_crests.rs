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
const OPTION2: u8 = 1;
const OPTION3: u8 = 2;
const AMANUENSIS: &str = "11053110";
/// Clue amulet — only Engage text; banish from Bahamut option 2 does not trigger its Engage.
const DIVINE_EVIDENCE: &str = "90001210";

fn play_bahamut_mode(db: &CardDb, st: &mut State, me: PlayerId, mode: u8) -> Vec<Event> {
    give_pp(st, me, 9, 10);
    st.player_mut(me).hand.clear();
    play_id(db, st, me, BAHAMUT);
    let mut events = Vec::new();
    events.extend(apply(db, st, Action::Choose(mode)).expect("mode choice"));
    while matches!(st.phase, Phase::Choice { .. }) {
        events.extend(apply(db, st, Action::Choose(0)).expect("drain choice"));
    }
    events
}

fn play_bahamut_option3(db: &CardDb, st: &mut State, me: PlayerId) -> Vec<Event> {
    play_bahamut_mode(db, st, me, OPTION3)
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

#[test]
fn owner_capture_bahamut_option3() {
    // Origin: captures/2026-11/376746249247561826-f03b433a-808hmo71ga.json @ engine fb67809
    let db = load_db();
    let path =
        crate_dir().join("tests/fixtures/captures/376746249247561826-f03b433a-808hmo71ga.json");
    let text = std::fs::read_to_string(&path).expect("capture fixture");
    let cap: serde_json::Value = serde_json::from_str(&text).expect("json");
    let st = replay_capture_to_ply(&db, &cap, 73).expect("replay through action 72");
    assert_eq!(crest_ids(&st, PlayerId::A), vec![] as Vec<String>);
    assert_eq!(
        crest_ids(&st, PlayerId::B),
        vec![SATHANID_FAITH.to_string()]
    );
    assert_eq!(st.player(PlayerId::B).faith, 4);
}

fn amanuensis_banish_setup(db: &CardDb, seed: u64) -> State {
    let mut st = started(db, seed);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 15;
    put_field(db, &mut st, me, AMANUENSIS);
    drain_choice(db, &mut st);
    st
}

#[test]
fn option2_banish_amulet_triggers_amuensis_heal() {
    let db = load_db();
    let me = PlayerId::A;
    let mut st = amanuensis_banish_setup(&db, 10_804_105);
    put_field(&db, &mut st, me, DIVINE_EVIDENCE);
    assert_eq!(st.player(me).leader_defense, 15);
    play_bahamut_mode(&db, &mut st, me, OPTION2);
    assert!(
        !field_has(&st, me, DIVINE_EVIDENCE),
        "option 2 banishes allied amulets"
    );
    assert_eq!(st.player(me).leader_defense, 17, "ally_card_banished heal");
}

#[test]
fn option3_banish_crest_does_not_trigger_field_banish_reaction() {
    let db = load_db();
    let me = PlayerId::A;
    let mut st = amanuensis_banish_setup(&db, 10_804_106);
    put_crest_cd(&mut st, me, STARRY_SKY, 2);
    play_bahamut_option3(&db, &mut st, me);
    assert_eq!(
        st.player(me).leader_defense,
        15,
        "crest banish is not a field banish"
    );
}
