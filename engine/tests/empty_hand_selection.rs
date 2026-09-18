//! Empty / unsatisfiable hand-selection: later sentences still resolve.
//!
//! Official Q&A (Burnite / Rodeo / Fan of Otohime / Cassius) and the owner's
//! 2026-09-08 sentence-structure ruling: an unfulfillable selection fizzles;
//! every later sentence in the same ability still runs. A term that reads
//! *the selected card* takes its empty value. Playability of spells whose
//! body *is* the selection does not move.

use arena_engine::{apply, legal_actions, Action, Phase, PlayerId, Slot};

mod common;
use common::*;

const ARISTOCRAT: &str = "10521110";
const KIMIKA: &str = "10842120";
const SPILLING: &str = "10642310";
const BANISHMENT: &str = "10972310";
const VANILLA: &str = "88001110";
const SPELL: &str = "88001300";
const INSTEAD_FIZZLE: &str = "89701005";

fn playable(db: &arena_engine::CardDb, st: &arena_engine::State, hand: u8) -> bool {
    legal_actions(db, st)
        .iter()
        .any(|a| matches!(a, Action::Play { hand: h } if *h == hand))
}

fn playable_id(db: &arena_engine::CardDb, st: &arena_engine::State, id: &str) -> bool {
    legal_actions(db, st).iter().any(|a| match a {
        Action::Play { hand } => st
            .player(st.active)
            .hand
            .get(*hand as usize)
            .is_some_and(|c| c.card.as_str() == id),
        _ => false,
    })
}

/// 1 — Aristocrat, empty hand: leader +3; nothing discarded; hand unchanged.
#[test]
fn aristocrat_empty_hand_restores_3() {
    let db = load_db();
    let mut st = started(&db, 18);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, ARISTOCRAT);
    assert!(
        playable(&db, &st, h),
        "Aristocrat (follower) must stay playable with an empty hand"
    );
    play(&db, &mut st, h);
    assert!(
        !matches!(st.phase, Phase::Choice { .. }),
        "empty hand must not open a discard choice"
    );
    assert!(field_has(&st, me, ARISTOCRAT));
    assert_eq!(
        st.player(me).hand.len(),
        0,
        "nothing discarded; hand still empty"
    );
    assert_eq!(
        st.player(me).leader_defense,
        13,
        "empty-hand Fanfare restores 3, not 0 or 6"
    );
}

/// 2 — Aristocrat, spell in hand, spell selected: leader +6.
#[test]
fn aristocrat_selected_spell_restores_6() {
    let db = load_db();
    let mut st = started(&db, 19);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, SPELL);
    play_id(&db, &mut st, me, ARISTOCRAT);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert_eq!(st.player(me).leader_defense, 16);
    assert!(!hand_has(&st, me, SPELL));
}

/// 3 — Aristocrat, follower in hand, follower selected: leader +3.
#[test]
fn aristocrat_selected_follower_restores_3() {
    let db = load_db();
    let mut st = started(&db, 20);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, VANILLA);
    play_id(&db, &mut st, me, ARISTOCRAT);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert_eq!(st.player(me).leader_defense, 13);
    assert!(!hand_has(&st, me, VANILLA));
}

/// 4 — Stale-state trap: a spell discarded earlier this match, then
/// Aristocrat with an empty hand, must restore 3 (not 6).
#[test]
fn aristocrat_empty_hand_after_earlier_spell_discard_restores_3() {
    let db = load_db();
    let mut st = started(&db, 21);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, VANILLA);
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, SPELL);
    play_id(&db, &mut st, me, KIMIKA);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(
        !hand_has(&st, me, SPELL),
        "a spell was discarded earlier this match"
    );
    assert_eq!(st.player(me).leader_defense, 11, "Kimika restore 1");
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, ARISTOCRAT);
    assert!(
        !matches!(st.phase, Phase::Choice { .. }),
        "Aristocrat empty hand must not open a choice"
    );
    assert_eq!(
        st.player(me).leader_defense,
        14,
        "empty-hand restore is 3 from this resolution's bind, not +6 from a prior discard"
    );
}

/// 8 — Kimika, empty hand: draws 1 and restores 1.
#[test]
fn kimika_empty_hand_draws_and_restores() {
    let db = load_db();
    let mut st = started(&db, 22);
    let me = PlayerId::A;
    st.player_mut(me).leader_defense = 10;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, VANILLA);
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, KIMIKA);
    assert_eq!(st.player(me).leader_defense, 11);
    assert_eq!(st.player(me).hand.len(), 1);
    assert!(hand_has(&st, me, VANILLA));
}

/// 10 — Control: Spilling Red, empty hand, unplayable.
#[test]
fn spilling_red_empty_hand_unplayable() {
    let db = load_db();
    let mut st = started(&db, 23);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, VANILLA);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, SPILLING);
    assert!(
        !playable_id(&db, &st, SPILLING),
        "Spilling Red empty hand stays unplayable"
    );
}

/// 11 — Control: Spilling Red, cards in hand, empty enemy board, unplayable.
#[test]
fn spilling_red_no_enemy_follower_unplayable() {
    let db = load_db();
    let mut st = started(&db, 24);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, VANILLA);
    put_hand(&db, &mut st, me, SPILLING);
    assert!(
        !playable_id(&db, &st, SPILLING),
        "Spilling Red with no enemy follower stays unplayable"
    );
}

/// 12 — Control: Disgraceful Banishment is a spell, so empty hand stays
/// unplayable on main (playability must not move). The same effect list on
/// a follower — discard fizzles, the "instead" keyed on deck uniqueness
/// still draws 3.
#[test]
fn disgraceful_banishment_empty_hand_unplayable_instead_still_fires_on_follower() {
    let db = load_db();
    let me = PlayerId::A;

    let mut st = started(&db, 25);
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, VANILLA);
    put_deck(&db, &mut st, me, "88001120");
    put_deck(&db, &mut st, me, "88001130");
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, BANISHMENT);
    assert!(
        !playable_id(&db, &st, BANISHMENT),
        "Disgraceful Banishment empty hand stays unplayable — do not change playability"
    );

    let mut st = started(&db, 26);
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, VANILLA);
    put_deck(&db, &mut st, me, "88001120");
    put_deck(&db, &mut st, me, "88001130");
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, INSTEAD_FIZZLE);
    assert_eq!(
        st.player(me).hand.len(),
        3,
        "discard fizzled; no-duplicates instead still draws 3"
    );
}

/// Open question — Extravagance of the Goldbloom `10521310` playability
/// with no spell in hand is left exactly as on main (unplayable).
#[test]
fn extravagance_no_spell_in_hand_stays_unplayable() {
    let db = load_db();
    let mut st = started(&db, 27);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, VANILLA);
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10521310");
    assert!(!playable_id(&db, &st, "10521310"));
    put_hand(&db, &mut st, me, VANILLA);
    assert!(
        !playable_id(&db, &st, "10521310"),
        "follower-only hand is still no legal spell to select"
    );
}

/// Open question — Goddess of Starlight `10502110` Evolve with fewer than
/// 3 cards: main opens a choice for as many as it can (0 → no node).
#[test]
fn goddess_evolve_fewer_than_three_matches_main() {
    let db = load_db();
    let me = PlayerId::A;
    for n in [0usize, 1, 2] {
        let mut st = started(&db, 28);
        set_round(&mut st, me, 5);
        st.player_mut(me).ep = 1;
        st.player_mut(me).field = Default::default();
        let slot = put_field(&db, &mut st, me, "10502110");
        clear_hand(&mut st, me);
        for _ in 0..n {
            put_hand(&db, &mut st, me, VANILLA);
        }
        apply(
            &db,
            &mut st,
            Action::Evolve {
                slot: Slot(slot),
                super_evolve: false,
            },
        )
        .expect("evolve goddess");
        if n == 0 {
            assert!(
                !matches!(st.phase, Phase::Choice { .. }),
                "empty hand: selection fizzles, no node"
            );
        } else {
            assert!(
                matches!(st.phase, Phase::Choice { .. }),
                "n={n}: main offers as many as it can"
            );
        }
    }
}
