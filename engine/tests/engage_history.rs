//! Engage-history tracking: invisible in Set-10-free games; hashed when readable.

use arena_engine::{apply, hash, search_key, snapshot_json, Action, PlayerId, Slot};

mod common;
use common::*;

const AMULET_ENGAGE: &str = "89500021"; // construct helper Engage amulet
const READS_ENGAGE: &str = "89501032"; // condition_engagedAmuletThisTurn

#[test]
fn set10_free_engage_invisible_in_hash_search_key_json() {
    let db = load_db();
    let mut st = started(&db, 42_001);
    assert!(
        !st.track_engage_history,
        "vanilla decks must not track Engage history"
    );
    let slot = put_field(&db, &mut st, PlayerId::A, AMULET_ENGAGE);
    give_pp(&mut st, PlayerId::A, 10, 10);
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).expect("engage");
    assert!(st.player(PlayerId::A).engaged_amulet_this_turn);

    let sk = search_key(&st);
    let h = hash(&st);
    let json = snapshot_json(&st);

    let mut clone = st.clone();
    clone.player_mut(PlayerId::A).engaged_amulet_this_turn = false;
    clone.player_mut(PlayerId::A).engaged_clue_amulet_this_turn = false;
    clone.player_mut(PlayerId::B).engaged_amulet_this_turn = false;
    clone.player_mut(PlayerId::B).engaged_clue_amulet_this_turn = false;

    assert_eq!(
        sk,
        search_key(&clone),
        "search_key ignores flags when untracked"
    );
    assert_eq!(h, hash(&clone), "hash ignores flags when untracked");
    assert_eq!(
        json,
        snapshot_json(&clone),
        "snapshot_json ignores flags when untracked"
    );
    assert!(
        json["players"]["a"]
            .get("engaged_amulet_this_turn")
            .is_none(),
        "JSON must omit engage flags when untracked"
    );
}

#[test]
fn tracked_engage_changes_search_key_and_hash() {
    let db = load_db();
    // Fixture card that reads engagedAmuletThisTurn must turn tracking on.
    let mut st = started_decks(&db, 42_002, &[READS_ENGAGE], &["88001110"]);
    assert!(
        st.track_engage_history,
        "deck with engagedAmuletThisTurn reader must track"
    );
    let slot = put_field(&db, &mut st, PlayerId::A, AMULET_ENGAGE);
    let sk0 = search_key(&st);
    let h0 = hash(&st);
    give_pp(&mut st, PlayerId::A, 10, 10);
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).expect("engage");
    assert!(st.player(PlayerId::A).engaged_amulet_this_turn);
    assert_ne!(
        sk0,
        search_key(&st),
        "Engage must change search_key when tracked"
    );
    assert_ne!(h0, hash(&st), "Engage must change hash when tracked");
    let json = snapshot_json(&st);
    assert_eq!(
        json["players"]["a"]["engaged_amulet_this_turn"], true,
        "JSON must include engage flags when tracked"
    );
}

#[test]
fn both_players_engage_flags_clear_at_turn_start() {
    let db = load_db();
    let mut st = started(&db, 42_003);
    let slot = put_field(&db, &mut st, PlayerId::A, AMULET_ENGAGE);
    give_pp(&mut st, PlayerId::A, 10, 10);
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).expect("engage");
    assert!(st.player(PlayerId::A).engaged_amulet_this_turn);

    apply(&db, &mut st, Action::EndTurn).expect("end turn");
    // B's turn has started → both players' Engage history cleared.
    assert!(
        !st.player(PlayerId::A).engaged_amulet_this_turn
            && !st.player(PlayerId::A).engaged_clue_amulet_this_turn,
        "A's flags clear at next turn start"
    );
    assert!(
        !st.player(PlayerId::B).engaged_amulet_this_turn
            && !st.player(PlayerId::B).engaged_clue_amulet_this_turn,
        "B's flags clear at turn start"
    );
}
