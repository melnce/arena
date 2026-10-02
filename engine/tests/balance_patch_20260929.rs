//! 2026-09-29 balance patch — stats and authored text for seven cards.

use arena_engine::{apply, legal_actions, Action, CardKind, Phase, PlayerId, Slot};

mod common;
use common::*;

const GOLDEN_KNIGHT: &str = "10423110";
const BEWITCHING: &str = "10633310";
const OPEN_SEA_SCOUT: &str = "10921110";
const WHIRLPOOL_GUNNER: &str = "10922110";
const DISGRACEFUL: &str = "10972310";
const SOULFORGE: &str = "10973310";
const CUTTHROAT: &str = "10974110";
const MECHANIZED_BEAST: &str = "10071130";
const VANILLA: &str = "88001110";
const SPELL: &str = "88001300";
/// Fixture neutrals — one copy each so `deckHasNoDuplicates` can be true.
const UNIQUE_PAD: &[&str] = &[
    "88001110", "88001120", "88001130", "88001140", "88001150", "88001160", "88001170", "88001180",
    "88001190", "88001200", "88001210", "88001220", "88001230", "88001240", "88001250", "88001260",
    "88001270", "88001280", "88001290", "88001300", "88001310", "88001320", "88001330", "88001340",
    "88001350", "88001360", "88001370", "88001380", "88001390", "88001400", "88001410", "88001420",
    "88001430", "88001440", "88001450", "88001460", "88001470", "88001480", "88001490", "88001500",
];

fn fill_unique_deck(
    db: &arena_engine::CardDb,
    st: &mut arena_engine::State,
    who: PlayerId,
    leading: &[&str],
) {
    st.player_mut(who).deck.clear();
    let mut used = std::collections::HashSet::new();
    for id in leading {
        put_deck(db, st, who, id);
        used.insert(id);
    }
    for id in UNIQUE_PAD {
        if st.player(who).deck.len() >= 40 {
            break;
        }
        if used.contains(id) {
            continue;
        }
        put_deck(db, st, who, id);
        used.insert(id);
    }
    assert_eq!(st.player(who).deck.len(), 40, "unique pad pool");
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

fn deck_len(st: &arena_engine::State, who: PlayerId) -> usize {
    st.player(who).deck.len()
}

fn count_card(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    let id = cid(id);
    let in_hand = st.player(who).hand.iter().filter(|c| c.card == id).count();
    let in_deck = st.player(who).deck.iter().filter(|c| c.card == id).count();
    let on_field = st
        .player(who)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card == id)
        .count();
    in_hand + in_deck + on_field
}

fn drain_choice(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

#[test]
fn patch_stats_from_db() {
    let db = load_db();
    let gk = db.card(cid(GOLDEN_KNIGHT)).unwrap();
    assert_eq!(gk.cost(), 6);
    assert_eq!(gk.attack(), 6);
    assert_eq!(gk.defense(), 6);

    let bew = db.card(cid(BEWITCHING)).unwrap();
    assert_eq!(bew.cost(), 4);
    assert_eq!(bew.kind(), CardKind::Spell);

    let scout = db.card(cid(OPEN_SEA_SCOUT)).unwrap();
    assert_eq!(scout.attack(), 2);
    assert_eq!(scout.defense(), 2);

    let whirl = db.card(cid(WHIRLPOOL_GUNNER)).unwrap();
    assert_eq!(whirl.attack(), 4);
    assert_eq!(whirl.defense(), 2);

    let soul = db.card(cid(SOULFORGE)).unwrap();
    assert_eq!(soul.cost(), 3);

    let cut = db.card(cid(CUTTHROAT)).unwrap();
    assert_eq!(cut.cost(), 1);
    assert_eq!(cut.attack(), 1);
    assert_eq!(cut.defense(), 1);
}

#[test]
fn golden_knight_six_pp_one_mode() {
    let db = load_db();
    let mut st = started(&db, 901);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, GOLDEN_KNIGHT);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "6 PP: pick one mode"
    );
    choose(&db, &mut st, 1);
    drain_choice(&db, &mut st);
    assert!(field_has(&st, me, GOLDEN_KNIGHT));
}

#[test]
fn golden_knight_seven_pp_one_mode() {
    let db = load_db();
    let mut st = started(&db, 902);
    let me = PlayerId::A;
    give_pp(&mut st, me, 7, 7);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, GOLDEN_KNIGHT);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "7 PP: normal, not Enhance"
    );
}

#[test]
fn golden_knight_eight_pp_enhance_all_modes() {
    let db = load_db();
    let mut st = started(&db, 903);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, VANILLA);
    st.player_mut(me).leader_defense = 16;
    let me_def_before = st.player(me).leader_defense;
    give_pp(&mut st, me, 8, 8);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, GOLDEN_KNIGHT);
    assert!(
        matches!(st.phase, Phase::Main),
        "Enhance (8): Activate all of them instead — no mode pause"
    );
    let gk = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == GOLDEN_KNIGHT)
        .expect("Golden Knight on field");
    assert!(gk.super_evolved, "mode 1 super-evolve");
    assert_eq!(
        field_count(&st, opp),
        0,
        "mode 2 dealt 4 to all enemy followers"
    );
    assert_eq!(
        st.player(me).leader_defense,
        me_def_before + 4,
        "mode 3 restore 4 defense to your leader"
    );
}

#[test]
fn bewitching_three_pp_not_playable() {
    let db = load_db();
    let mut st = started(&db, 904);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, BEWITCHING);
    assert!(
        !playable_id(&db, &st, BEWITCHING),
        "cost 4 — 3 PP not playable"
    );
}

#[test]
fn bewitching_four_and_five_pp_normal() {
    let db = load_db();
    let me = PlayerId::A;

    let mut st = started(&db, 905);
    give_pp(&mut st, me, 4, 4);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, BEWITCHING);
    assert!(matches!(st.phase, Phase::Choice { .. }), "4 PP: one mode");

    let mut st = started(&db, 906);
    give_pp(&mut st, me, 5, 5);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, BEWITCHING);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "5 PP: still normal"
    );
}

#[test]
fn bewitching_six_pp_enhance_all_modes() {
    let db = load_db();
    let mut st = started(&db, 907);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, BEWITCHING);
    assert!(
        matches!(st.phase, Phase::Main),
        "Enhance (6): Activate all of them instead"
    );
    let spawns = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "10631110")
        .count();
    assert_eq!(spawns, 3, "mode 1 + mode 2 Enhance summons");
}

#[test]
fn disgraceful_search_draw_takes_portal_bane_follower() {
    let db = load_db();
    let mut st = started(&db, 908);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, MECHANIZED_BEAST);
    put_deck(&db, &mut st, me, "10471110");
    put_deck(&db, &mut st, me, SPELL);
    while st.player(me).deck.len() < 40 {
        put_deck(&db, &mut st, me, VANILLA);
    }
    give_pp(&mut st, me, 1, 1);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, SPELL);
    play_id(&db, &mut st, me, DISGRACEFUL);
    drain_choice(&db, &mut st);
    assert!(
        hand_has(&st, me, MECHANIZED_BEAST),
        "Draw a Portalcraft follower with Bane — takes Mechanized Beast"
    );
}

#[test]
fn disgraceful_no_bane_follower_draws_nothing() {
    let db = load_db();
    let mut st = started(&db, 909);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, "10471110");
    put_deck(&db, &mut st, me, SPELL);
    while st.player(me).deck.len() < 40 {
        put_deck(&db, &mut st, me, VANILLA);
    }
    let before = deck_len(&st, me);
    give_pp(&mut st, me, 1, 1);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, SPELL);
    play_id(&db, &mut st, me, DISGRACEFUL);
    drain_choice(&db, &mut st);
    assert_eq!(
        deck_len(&st, me),
        before,
        "no Portalcraft Bane follower — search draw fizzles, deck size untouched"
    );
    assert!(!hand_has(&st, me, MECHANIZED_BEAST));
}

#[test]
fn disgraceful_then_draw_two_after_search_clears_duplicate_pair() {
    let db = load_db();
    let mut st = started(&db, 910);
    let me = PlayerId::A;
    fill_unique_deck(&db, &mut st, me, &[CUTTHROAT, CUTTHROAT]);
    give_pp(&mut st, me, 1, 1);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, SPELL);
    let hand_before = st.player(me).hand.len();
    play_id(&db, &mut st, me, DISGRACEFUL);
    drain_choice(&db, &mut st);
    assert_eq!(
        count_card(&st, me, CUTTHROAT),
        2,
        "search drew one Cutthroat"
    );
    assert!(
        st.player(me).hand.len() >= hand_before + 2,
        "Then, if there are no duplicates in your deck, draw 2 cards"
    );
}

#[test]
fn disgraceful_no_bonus_draw_when_duplicate_remains() {
    let db = load_db();
    let mut st = started(&db, 911);
    let me = PlayerId::A;
    fill_unique_deck(&db, &mut st, me, &[CUTTHROAT, CUTTHROAT, CUTTHROAT]);
    give_pp(&mut st, me, 1, 1);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, SPELL);
    let hand_before = st.player(me).hand.len();
    play_id(&db, &mut st, me, DISGRACEFUL);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(me).hand.len(),
        hand_before,
        "duplicate remains after search — no +2"
    );
}

#[test]
fn disgraceful_empty_hand_both_draws_still_resolve() {
    let db = load_db();
    let mut st = started(&db, 912);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, MECHANIZED_BEAST);
    while st.player(me).deck.len() < 40 {
        put_deck(&db, &mut st, me, VANILLA);
    }
    give_pp(&mut st, me, 1, 1);
    clear_hand(&mut st, me);
    assert!(!playable_id(&db, &st, DISGRACEFUL));
    play_id(&db, &mut st, me, "89701006");
    assert!(
        hand_has(&st, me, MECHANIZED_BEAST),
        "discard fizzles on empty hand; Draw a Portalcraft follower with Bane still runs"
    );
}

#[test]
fn cutthroat_evolve_draws_deck_copy_nothing_banished() {
    let db = load_db();
    let mut st = started(&db, 913);
    let me = PlayerId::A;
    st.player_mut(me).ep = 1;
    st.player_mut(me).turns_taken = 5;
    let slot = put_field(&db, &mut st, me, CUTTHROAT);
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, CUTTHROAT);
    while st.player(me).deck.len() < 40 {
        put_deck(&db, &mut st, me, VANILLA);
    }
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .unwrap();
    assert_eq!(
        count_card(&st, me, CUTTHROAT),
        2,
        "Draw a Portalcraft follower with Bane — deck copy can be drawn; nothing banished"
    );
}

#[test]
fn cutthroat_evolve_crest_iff_no_duplicates_after_draw() {
    let db = load_db();
    let mut st = started(&db, 914);
    let me = PlayerId::A;
    st.player_mut(me).ep = 1;
    st.player_mut(me).turns_taken = 5;
    let slot = put_field(&db, &mut st, me, CUTTHROAT);
    fill_unique_deck(&db, &mut st, me, &[CUTTHROAT, CUTTHROAT, CUTTHROAT]);
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .unwrap();
    assert!(
        st.player(me)
            .crests
            .iter()
            .all(|c| c.id != "crest:10974110"),
        "two Cutthroats remain in deck after search draw — no crest"
    );

    let mut st = started(&db, 915);
    st.player_mut(me).ep = 1;
    st.player_mut(me).turns_taken = 5;
    let slot = put_field(&db, &mut st, me, CUTTHROAT);
    fill_unique_deck(&db, &mut st, me, &[CUTTHROAT]);
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .unwrap();
    assert!(
        st.player(me)
            .crests
            .iter()
            .any(|c| c.id == "crest:10974110"),
        "search draw leaves no duplicates — gain Crest"
    );
}

#[test]
fn cutthroat_evolve_crest_check_without_bane_in_deck() {
    let db = load_db();
    let mut st = started(&db, 916);
    let me = PlayerId::A;
    st.player_mut(me).ep = 1;
    st.player_mut(me).turns_taken = 5;
    let slot = put_field(&db, &mut st, me, CUTTHROAT);
    fill_unique_deck(&db, &mut st, me, &[]);
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .unwrap();
    assert!(
        st.player(me)
            .crests
            .iter()
            .any(|c| c.id == "crest:10974110"),
        "no Bane follower in deck — search fizzles; crest check still runs on unique deck"
    );
}

#[test]
fn soulforge_playable_at_three_pp() {
    let db = load_db();
    let mut st = started(&db, 917);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, VANILLA);
    give_pp(&mut st, me, 3, 3);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, SOULFORGE);
    assert!(
        playable_id(&db, &st, SOULFORGE),
        "cost 3 — playable at 3 PP"
    );
}

#[test]
fn open_sea_scout_and_whirlpool_gunner_stats_on_field() {
    let db = load_db();
    let mut st = started(&db, 918);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, OPEN_SEA_SCOUT);
    let scout = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == OPEN_SEA_SCOUT)
        .expect("scout");
    assert_eq!(scout.attack, 2);
    assert_eq!(scout.defense, 2);

    give_pp(&mut st, me, 3, 3);
    play_id(&db, &mut st, me, WHIRLPOOL_GUNNER);
    let gunner = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WHIRLPOOL_GUNNER)
        .expect("gunner");
    assert_eq!(gunner.attack, 4);
    assert_eq!(gunner.defense, 2);
}
