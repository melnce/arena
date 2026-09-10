//! Cross-ability bindings (Amorous Necromancer 10052120 pattern) and
//! `maxPpAtLeast` (Dragonsign 10042310).

use arena_engine::{
    apply, new_game, picks_from_trace_rng, Action, Card, CardDb, First, GameConfig, LoadError,
    OpeningHands, PlayerId, Slot,
};

mod common;
use common::*;

/// Rulebook: a super-evolve fires both Evolve and Super-Evolve lines unless
/// the super line says "instead". Bindings share that resolution.
#[test]
fn cross_ability_bindings_evolve_super_evolve() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001510");
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .unwrap();
    let vanillas: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "88001110")
        .collect();
    assert_eq!(vanillas.len(), 2, "evolve line summons 2");
    assert!(
        vanillas.iter().all(|c| c.is_drain()),
        "super-evolve grantTraits bound g — Amorous 10052120"
    );
}

#[test]
fn evolve_without_super_does_not_grant_bound_drain() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001510");
    st.player_mut(me).turns_taken = 5;
    st.player_mut(me).ep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .unwrap();
    let vanillas: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "88001110")
        .collect();
    assert_eq!(vanillas.len(), 2);
    assert!(
        vanillas.iter().all(|c| !c.is_drain()),
        "super line did not fire; bound grant is not applied"
    );
}

#[test]
fn bound_ref_without_live_binding_is_empty_not_error() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    // Super-evolve with no prior `as` in this resolution: empty set, no panic.
    let slot = put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .unwrap();
    assert!(!st.player(me).field[slot as usize]
        .as_ref()
        .unwrap()
        .is_drain());
}

#[test]
fn card_db_rejects_ref_with_no_as_on_same_card() {
    let text = std::fs::read_to_string(fixtures_dir().join("invalid/unbound_ref.json")).unwrap();
    let card: Card = serde_json::from_str(&text).unwrap();
    assert_eq!(card.unbound_refs(), vec!["missing".to_string()]);

    let tmp = std::env::temp_dir().join("arena-unbound-ref-load");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("88009990.json"), &text).unwrap();
    let mut db = CardDb::load(repo_root()).unwrap();
    match db.load_extra_dir(&tmp) {
        Err(LoadError::UnboundRef { name, .. }) => assert_eq!(name, "missing"),
        other => panic!("expected UnboundRef, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn dragonsign_max_pp_at_least_10() {
    let db = load_db();
    let mut st = started(&db, 4);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 9);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10042310");
    let deck_before = st.player(me).deck.len();
    play(&db, &mut st, h);
    assert_eq!(st.player(me).pp_max, 10);
    assert_eq!(
        st.player(me).deck.len(),
        deck_before - 1,
        "maxPpAtLeast 10 draws after gaining the 10th orb"
    );

    give_pp(&mut st, me, 3, 8);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10042310");
    let deck_before = st.player(me).deck.len();
    let hand_before = st.player(me).hand.len();
    play(&db, &mut st, h);
    assert_eq!(st.player(me).pp_max, 9);
    assert_eq!(st.player(me).deck.len(), deck_before);
    assert_eq!(st.player(me).hand.len(), hand_before - 1);
}

#[test]
fn opening_hands_header_deals_without_rolling() {
    let db = load_db();
    let decks = pad_deck(&["88001110", "88001120", "88001130", "88001210"], 40);
    let opening = OpeningHands {
        a: vec![
            cid("88001110"),
            cid("88001120"),
            cid("88001130"),
            cid("88001210"),
        ],
        b: vec![
            cid("88001110"),
            cid("88001120"),
            cid("88001130"),
            cid("88001210"),
        ],
    };
    let st = new_game(
        &db,
        GameConfig {
            seed: 99,
            deck_a: decks.clone(),
            deck_b: decks,
            first: First::A,
            opening_hands: Some(opening),
        },
    )
    .unwrap();
    let ids: Vec<String> = st
        .player(PlayerId::A)
        .hand
        .iter()
        .map(|c| c.card.as_str())
        .collect();
    assert_eq!(ids, vec!["88001110", "88001120", "88001130", "88001210"]);
}

#[test]
fn raw_picks_are_ignored() {
    let v = serde_json::json!([
        {"what": "raw", "kind": "shuffle", "deck": "a"},
        {"what": "draw", "chose": "88001110"}
    ]);
    let picks = picks_from_trace_rng(&v);
    assert_eq!(picks.len(), 1);
    assert_eq!(picks[0].what.as_str(), "draw");
}
