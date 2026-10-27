//! Play reactions on follower / amulet plays resolve after the whole body (E45).

use arena_engine::event::Event;
use arena_engine::{apply, Action, CardInstance, PlayerId};

mod common;
use common::*;

const WORLD: &str = "10503210";
const VANILLA: &str = "88001110";
const TANK: &str = "89500001";
const WARDEN: &str = "10903110";
const JAILOR: &str = "10901110";
const MOELLE: &str = "10811130";
const FORESIGHT: &str = "10031310";
const WITCH_BREW: &str = "10031210";
const THESTAE: &str = "10714110";

fn wog_at_count(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, n: i32) {
    put_field(db, st, who, WORLD);
    if let Some(w) = st
        .player_mut(who)
        .field
        .iter_mut()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
    {
        w.countdown = Some(n);
    }
}

fn clear_hand_deck(st: &mut arena_engine::State, who: PlayerId) {
    st.player_mut(who).hand.clear();
    st.player_mut(who).deck.clear();
}

fn fill_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str, n: usize) {
    let card = db.card(cid(id)).expect("card");
    for _ in 0..n {
        let id = st.alloc_id();
        st.player_mut(who).deck.push(CardInstance::from_card(card, id));
    }
}

fn play_events(
    db: &arena_engine::CardDb,
    st: &mut arena_engine::State,
    hand: u8,
) -> Vec<Event> {
    let mut ev = apply(db, st, Action::Play { hand }).expect("play");
    while matches!(st.phase, arena_engine::Phase::Choice { .. }) {
        ev.extend(apply(db, st, Action::Choose(0)).expect("choose"));
    }
    ev
}

fn play_kind_tags(events: &[Event]) -> Vec<&'static str> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Damage { .. } => Some("damage"),
            Event::Destroy { .. } => Some("destroy"),
            Event::Draw { .. } => Some("draw"),
            _ => None,
        })
        .collect()
}

#[test]
fn warden_of_selflessness_wog_after_whole_fanfare_body() {
    let db = load_db();
    let mut st = started(&db, 801);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    clear_hand_deck(&mut st, me);
    fill_deck(&db, &mut st, me, VANILLA, 5);
    wog_at_count(&db, &mut st, me, 1);
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, WARDEN);
    give_pp(&mut st, me, 10, 10);
    put_hand(&db, &mut st, me, WARDEN);
    let events = play_events(&db, &mut st, 0);
    assert_eq!(field_def(&st, opp, TANK), Some(9));
    assert_eq!(field_def(&st, opp, WARDEN), Some(3));
    assert_eq!(
        play_kind_tags(&events),
        ["damage", "damage", "destroy", "draw", "draw"]
    );
    let hand_ids: Vec<String> = st
        .player(me)
        .hand
        .iter()
        .map(|c| c.card.as_str().to_string())
        .collect();
    assert_eq!(
        hand_ids,
        vec![JAILOR.to_string(), VANILLA.to_string(), VANILLA.to_string()]
    );
    assert_eq!(field_count(&st, me), 1);
    assert!(field_has(&st, me, WARDEN));
    assert!(!field_has(&st, me, WORLD));
}

#[test]
fn moelle_wog_after_body_play_time_pick_first() {
    let db = load_db();
    let mut st = started(&db, 802);
    let me = PlayerId::A;
    clear_hand_deck(&mut st, me);
    fill_deck(&db, &mut st, me, VANILLA, 5);
    wog_at_count(&db, &mut st, me, 1);
    give_pp(&mut st, me, 10, 10);
    put_hand(&db, &mut st, me, FORESIGHT);
    let moelle_hand = put_hand(&db, &mut st, me, MOELLE);
    let events = play_events(&db, &mut st, moelle_hand);
    assert_eq!(st.player(me).hand.len(), 3);
    assert_eq!(
        play_kind_tags(&events),
        ["draw", "destroy", "draw", "draw"]
    );
}

#[test]
fn witch_brew_amulet_wog_after_fanfare_draw() {
    let db = load_db();
    let mut st = started(&db, 803);
    let me = PlayerId::A;
    clear_hand_deck(&mut st, me);
    fill_deck(&db, &mut st, me, VANILLA, 5);
    wog_at_count(&db, &mut st, me, 1);
    give_pp(&mut st, me, 10, 10);
    put_hand(&db, &mut st, me, WITCH_BREW);
    let events = play_events(&db, &mut st, 0);
    assert_eq!(
        play_kind_tags(&events),
        ["draw", "destroy", "draw", "draw"]
    );
}

#[test]
fn foresight_spell_wog_before_draw_unchanged() {
    let db = load_db();
    let mut st = started(&db, 804);
    let me = PlayerId::A;
    clear_hand_deck(&mut st, me);
    fill_deck(&db, &mut st, me, VANILLA, 5);
    wog_at_count(&db, &mut st, me, 1);
    give_pp(&mut st, me, 10, 10);
    put_hand(&db, &mut st, me, FORESIGHT);
    let events = play_events(&db, &mut st, 0);
    assert_eq!(
        play_kind_tags(&events),
        ["destroy", "draw", "draw", "draw"]
    );
}

#[test]
fn thestae_owner_setup_wog_count_after_fanfare() {
    // Owner 2026-10-27: WoG after Thestae Fanfare — both orderings match this setup.
    let db = load_db();
    let mut st = started(&db, 805);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    clear_hand_deck(&mut st, me);
    wog_at_count(&db, &mut st, me, 3);
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, THESTAE);
    give_pp(&mut st, me, 10, 10);
    put_hand(&db, &mut st, me, THESTAE);
    play_events(&db, &mut st, 0);
    let wog = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
        .expect("wog");
    assert_eq!(wog.countdown, Some(2));
    assert_eq!(field_def(&st, opp, TANK), Some(7));
}
