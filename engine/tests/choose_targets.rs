//! `pick: choose` must apply (and bind) only the caller-chosen targets.
//! Five ops used to re-resolve their selector inside `apply_effect`.

use arena_engine::card::VarKey;
use arena_engine::{apply, Action, Phase, PlayerId, Slot};

mod common;
use common::*;

const ELMOTT: &str = "10433110";
const TANK_ONE: &str = "89500001";
const HIGH_DEF: &str = "89500024";
const TANK_THREE: &str = "89500002";
const LW_PING: &str = "89100030";
const AWED: &str = "10461210";
const KEY_SPIRIT: &str = "10931120";
const STORMY_BLAST: &str = "10831310";
const SAMMY: &str = "10832110";
const LOVE_BOMB: &str = "10442310";
const DESTROYER: &str = "10653110";
const CHOOSE_COST: &str = "89900110";
const CHOOSE_COUNTDOWN: &str = "89900120";
const CD_AMULET: &str = "89100020";
const BIG: &str = "88001320";
const VANILLA: &str = "88001110";

fn play_and_choose0(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    give_pp(st, st.active, pp, pp.max(1));
    clear_hand(st, st.active);
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "{id} should offer a choose"
    );
    choose(db, st, 0);
}

fn play_elmott_choose0(db: &arena_engine::CardDb, st: &mut arena_engine::State, enemies: &[&str]) {
    let opp = PlayerId::B;
    for id in enemies {
        put_field(db, st, opp, id);
    }
    play_and_choose0(db, st, ELMOTT, 3);
}

fn printed_def(id: &str) -> i32 {
    match id {
        TANK_ONE | TANK_THREE => 10,
        HIGH_DEF => 12,
        other => panic!("unexpected dummy {other}"),
    }
}

fn last_words(st: &arena_engine::State, who: PlayerId, slot: u8) -> bool {
    st.field_inst(who, slot)
        .is_some_and(|c| c.printed_tags.contains("lastWords"))
}

fn hand_cost(st: &arena_engine::State, who: PlayerId, id: &str) -> i32 {
    let id = cid(id);
    st.player(who)
        .hand
        .iter()
        .find(|c| c.card == id)
        .map(|c| c.cost)
        .unwrap_or_else(|| panic!("hand missing {id}"))
}

fn hand_x(st: &arena_engine::State, who: PlayerId, id: &str) -> i32 {
    let id = cid(id);
    st.player(who)
        .hand
        .iter()
        .find(|c| c.card == id)
        .and_then(|c| c.vars.get(&VarKey::X).copied())
        .unwrap_or(0)
}

fn grant_evolve(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    let me = st.active;
    if st.player(me).turns_taken < 5 {
        set_round(st, me, 5);
    }
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

/// Elmott 1 / 2 / 3 enemy boards: only the chosen follower takes 3.
#[test]
fn elmott_damages_only_the_chosen_follower() {
    let db = load_db();
    let boards: [&[&str]; 3] = [
        &[TANK_ONE],
        &[TANK_ONE, HIGH_DEF],
        &[TANK_ONE, HIGH_DEF, TANK_THREE],
    ];
    for enemies in boards {
        let mut st = started(&db, 11);
        play_elmott_choose0(&db, &mut st, enemies);
        let opp = PlayerId::B;
        assert_eq!(
            field_def(&st, opp, enemies[0]),
            Some(printed_def(enemies[0]) - 3),
            "chosen {} should be at printed − 3 ({:?})",
            enemies[0],
            enemies
        );
        for &id in &enemies[1..] {
            assert_eq!(
                field_def(&st, opp, id),
                Some(printed_def(id)),
                "unchosen {id} must stay at printed defense ({enemies:?})"
            );
        }
    }
}

/// Ability-stripping is the same bug; dummies hide it.
#[test]
fn elmott_strips_abilities_only_from_the_chosen_follower() {
    let db = load_db();
    let mut st = started(&db, 12);
    let opp = PlayerId::B;
    let chosen = put_field(&db, &mut st, opp, LW_PING);
    let other = put_field(&db, &mut st, opp, LW_PING);
    for slot in [chosen, other] {
        if let Some(f) = st.field_inst_mut(opp, slot) {
            f.defense = 10;
            f.max_defense = 10;
        }
    }
    assert!(last_words(&st, opp, chosen));
    assert!(last_words(&st, opp, other));
    play_and_choose0(&db, &mut st, ELMOTT, 3);
    assert!(
        !last_words(&st, opp, chosen),
        "chosen last words must be stripped"
    );
    assert!(
        last_words(&st, opp, other),
        "unchosen last words must remain"
    );
    assert_eq!(
        st.field_inst(opp, chosen).map(|c| c.defense),
        Some(7),
        "chosen LW dummy 10 → 7"
    );
    assert_eq!(
        st.field_inst(opp, other).map(|c| c.defense),
        Some(10),
        "unchosen LW dummy stays at 10"
    );
}

#[test]
fn awed_and_inspired_transforms_only_the_chosen_follower() {
    let db = load_db();
    let mut st = started(&db, 13);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, TANK_ONE);
    put_field(&db, &mut st, me, HIGH_DEF);
    let awed = put_field(&db, &mut st, me, AWED);
    give_pp(&mut st, me, 2, 2);
    apply(&db, &mut st, Action::Engage { slot: Slot(awed) }).expect("engage");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(
        field_has(&st, me, AWED),
        "chosen Tank One becomes Awed and Inspired"
    );
    assert!(
        field_has(&st, me, HIGH_DEF),
        "unchosen High Def must not transform"
    );
    assert!(
        !field_has(&st, me, TANK_ONE),
        "chosen Tank One is the one that transformed"
    );
    assert_eq!(
        field_count(&st, me),
        2,
        "engage sacrificed the original Awed"
    );
}

#[test]
fn key_spirit_spellboosts_only_the_chosen_hand_card() {
    let db = load_db();
    let mut st = started(&db, 14);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, KEY_SPIRIT);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, STORMY_BLAST);
    put_hand(&db, &mut st, me, SAMMY);
    grant_evolve(&db, &mut st, slot);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert_eq!(
        hand_x(&st, me, STORMY_BLAST),
        6,
        "chosen Stormy Blast X starts at 2, plus 4 spellboosts"
    );
    assert_eq!(
        hand_cost(&st, me, SAMMY),
        5,
        "unchosen Sammy & Marie stays at printed cost 5"
    );
}

#[test]
fn choose_cost_sets_only_the_chosen_hand_card() {
    let db = load_db();
    let mut st = started(&db, 15);
    let me = PlayerId::A;
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, BIG);
    put_hand(&db, &mut st, me, VANILLA);
    give_pp(&mut st, me, 0, 1);
    let h = put_hand(&db, &mut st, me, CHOOSE_COST);
    play(&db, &mut st, h);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert_eq!(hand_cost(&st, me, BIG), 0, "chosen Synth Big cost → 0");
    assert_eq!(
        hand_cost(&st, me, VANILLA),
        1,
        "unchosen vanilla stays at printed cost 1"
    );
}

#[test]
fn choose_countdown_ticks_only_the_chosen_amulet() {
    let db = load_db();
    let mut st = started(&db, 16);
    let me = PlayerId::A;
    let chosen = put_field(&db, &mut st, me, CD_AMULET);
    let other = put_field(&db, &mut st, me, CD_AMULET);
    play_and_choose0(&db, &mut st, CHOOSE_COUNTDOWN, 0);
    assert_eq!(
        st.field_inst(me, chosen).and_then(|c| c.countdown),
        Some(2),
        "chosen countdown 3 → 2"
    );
    assert_eq!(
        st.field_inst(me, other).and_then(|c| c.countdown),
        Some(3),
        "unchosen countdown stays 3"
    );
}

#[test]
fn maximum_love_bomb_damages_only_the_chosen_enemy() {
    let db = load_db();
    let mut st = started(&db, 17);
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK_ONE);
    put_field(&db, &mut st, opp, HIGH_DEF);
    put_field(&db, &mut st, opp, TANK_THREE);
    play_and_choose0(&db, &mut st, LOVE_BOMB, 2);
    assert_eq!(field_def(&st, opp, TANK_ONE), Some(7));
    assert_eq!(field_def(&st, opp, HIGH_DEF), Some(12));
    assert_eq!(field_def(&st, opp, TANK_THREE), Some(10));
}

#[test]
fn deprived_destroyer_destroys_exactly_one_allied_dummy() {
    let db = load_db();
    let mut st = started(&db, 18);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, TANK_ONE);
    put_field(&db, &mut st, me, HIGH_DEF);
    put_field(&db, &mut st, me, TANK_THREE);
    play_and_choose0(&db, &mut st, DESTROYER, 5);
    drain_choice(&db, &mut st);
    assert!(
        !field_has(&st, me, TANK_ONE),
        "chosen Tank One is destroyed"
    );
    assert!(field_has(&st, me, HIGH_DEF), "unchosen High Def remains");
    assert!(
        field_has(&st, me, TANK_THREE),
        "unchosen Tank Three remains"
    );
}

#[test]
fn remove_abilities_empty_pool_fizzles_without_panic() {
    let db = load_db();
    let mut st = started(&db, 19);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    clear_hand(&mut st, me);
    let h = put_hand(&db, &mut st, me, ELMOTT);
    play(&db, &mut st, h);
    assert!(
        !matches!(st.phase, Phase::Choice { .. }),
        "empty enemy pool must fizzle, not offer a choice"
    );
    assert!(field_has(&st, me, ELMOTT));
    assert_eq!(field_count(&st, PlayerId::B), 0);
}
