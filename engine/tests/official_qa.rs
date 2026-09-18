//! Official Cygames per-card Q&A pins.
//!
//! Test names are the published question text (slugified) so a Q&A-pin
//! metric can count them. The real cards for Burnite / Rodeo / Fan of
//! Otohime / Ralmia are not in the authored pool; fixtures reproduce the
//! printed construction. Cassius is the real card.

use arena_engine::event::{Event, EventTarget};
use arena_engine::{apply, Action, Phase, PlayerId, Slot};

mod common;
use common::*;

const BURNITE: &str = "89701001";
const RODEO: &str = "89701002";
const OTOHIME: &str = "89701003";
const RALMIA: &str = "89701004";
const CASSIUS: &str = "10473110";
const VANILLA: &str = "88001110";
const TANK: &str = "88001320";
const ARTIFACT_A: &str = "90071140";

/// If I play Burnite, Anathema of Flame when it's the only card in my hand, will its Fanfare ability activate?
#[test]
fn if_i_play_burnite_anathema_of_flame_when_its_the_only_card_in_my_hand_will_its_fanfare_ability_activate(
) {
    let db = load_db();
    let mut st = started(&db, 41);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, VANILLA);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, BURNITE);
    let evs = apply(&db, &mut st, Action::Play { hand: h }).expect("play burnite");
    assert!(
        field_has(&st, me, BURNITE),
        "Yes. Fanfare activates without discarding a card"
    );
    assert_eq!(
        field_def(&st, opp, TANK),
        Some(10),
        "deal 0 — empty selected-card cost"
    );
    assert_eq!(field_def(&st, opp, VANILLA), Some(2));
    let zero_hits = evs
        .iter()
        .filter(|e| {
            matches!(
                e,
                Event::Damage {
                    amount: 0,
                    target: EventTarget::Slot(..),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        zero_hits, 2,
        "the Deal-X clause still runs; X is 0, not skipped"
    );
    assert!(
        !matches!(st.phase, Phase::Choice { .. }),
        "no discard when the hand is empty after play"
    );
}

/// If I play Rodeo, Anathema of Judgment when it's the only card in my hand, will its Fanfare ability activate?
#[test]
fn if_i_play_rodeo_anathema_of_judgment_when_its_the_only_card_in_my_hand_will_its_fanfare_ability_activate(
) {
    let db = load_db();
    let mut st = started(&db, 42);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    for id in ["10703210", "10503210", "10031210"] {
        put_deck(&db, &mut st, me, id);
    }
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, RODEO);
    assert!(field_has(&st, me, RODEO), "Yes. Fanfare activates");
    let amulets: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() != RODEO)
        .map(|c| c.card.as_str().to_string())
        .collect();
    assert_eq!(
        amulets.len(),
        3,
        "summon 3 random differently named amulets from deck; no discard"
    );
    let mut names = amulets.clone();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 3, "differently named");
}

/// Can I engage Fan of Otohime even if my hand is empty?
#[test]
fn can_i_engage_fan_of_otohime_even_if_my_hand_is_empty() {
    let db = load_db();
    let mut st = started(&db, 43);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    let slot = put_field(&db, &mut st, me, OTOHIME);
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).expect("engage");
    drain_choice(&db, &mut st);
    assert!(
        field_has(&st, me, "90021110"),
        "Yes. Summon an Otohime's Bodyguard without discarding a card"
    );
    assert_eq!(st.player(me).hand.len(), 0);
    assert!(field_has(&st, me, OTOHIME));
}

/// If I play Cassius, Sky-Yearning Arrival without any Artifact followers in my hand, does his Fanfare ability still activate?
#[test]
fn if_i_play_cassius_sky_yearning_arrival_without_any_artifact_followers_in_my_hand_does_his_fanfare_ability_still_activate(
) {
    let db = load_db();
    let mut st = started(&db, 44);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, VANILLA);
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, VANILLA);
    let h = put_hand(&db, &mut st, me, CASSIUS);
    let evs = apply(&db, &mut st, Action::Play { hand: h }).expect("play cassius");
    assert!(
        field_has(&st, me, CASSIUS),
        "Yes. You'll just deal 0 damage to all enemy followers"
    );
    assert!(
        field_has(&st, opp, VANILLA),
        "0 damage — no Artifact follower was selected"
    );
    assert!(hand_has(&st, me, VANILLA), "non-matching hand card stays");
    assert!(
        evs.iter().any(|e| matches!(
            e,
            Event::Damage {
                amount: 0,
                target: EventTarget::Slot(..),
                ..
            }
        )),
        "Fanfare's deal-X still resolves at X=0"
    );
}

/// Will Ralmia, Sonic Boom's Fanfare ability activate even if I don't have 3 Artifact followers in my hand that cost 5 or less?
#[test]
fn will_ralmia_sonic_booms_fanfare_ability_activate_even_if_i_dont_have_3_artifact_followers_in_my_hand_that_cost_5_or_less(
) {
    let db = load_db();
    let mut st = started(&db, 45);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, ARTIFACT_A);
    play_id(&db, &mut st, me, RALMIA);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "select as many as you can"
    );
    choose(&db, &mut st, 0);
    assert!(
        field_has(&st, me, RALMIA),
        "Yes. Fanfare activates with a short pool"
    );
    let copies = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == ARTIFACT_A)
        .count();
    assert_eq!(copies, 1, "summon an exact copy of each selected");
    assert!(
        hand_has(&st, me, ARTIFACT_A),
        "the hand card itself is selected, not discarded"
    );

    let mut st = started(&db, 46);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, VANILLA);
    play_id(&db, &mut st, me, RALMIA);
    assert!(
        field_has(&st, me, RALMIA),
        "Yes — even with nothing matching"
    );
    assert!(
        !matches!(st.phase, Phase::Choice { .. }),
        "empty matching pool fizzles the select"
    );
    assert_eq!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .filter(|c| c.card.as_str() != RALMIA)
            .count(),
        0,
        "nothing to copy"
    );
}
