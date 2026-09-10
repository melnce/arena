//! Discriminating fixtures for ability-level constructs.
//! Each `construct_<slug>` fails if the engine ignores that construct.
#![allow(non_snake_case, unused_imports, dead_code)]

use arena_engine::{apply, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

const VANILLA: &str = "88001110";
const TANK1: &str = "89500001";
const TANK3: &str = "89500002";
const WARD: &str = "89500003";
const LW_DUMMY: &str = "89500004";
const OFFICER: &str = "89500005";
const FOREST: &str = "89500006";
const ATK5: &str = "89500010";
const COST5: &str = "89500011";
const HIGH_DEF: &str = "89500024";
const EVO: &str = "89500028";

fn opp_def(st: &arena_engine::State) -> i32 {
    leader_def(st, PlayerId::B)
}

fn play_costed(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    let me = st.active;
    clear_hand(st, me);
    give_pp(st, me, pp, pp.max(1));
    play_id(db, st, me, id);
    drain_choice(db, st);
}

fn evolve_slot(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8, supered: bool) {
    let me = st.active;
    if supered {
        if st.player(me).turns_taken < 7 {
            set_round(st, me, 7);
        }
        st.player_mut(me).sep = st.player(me).sep.max(1);
    } else {
        if st.player(me).turns_taken < 5 {
            set_round(st, me, 5);
        }
        st.player_mut(me).ep = st.player(me).ep.max(1);
    }
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: supered,
        },
    )
    .expect("evolve");
}

fn attack_leader(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    apply(
        db,
        st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .expect("attack leader");
}

fn attack_follower(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8, def: u8) {
    apply(
        db,
        st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Slot(Slot(def)),
        },
    )
    .expect("attack follower");
}

fn put_crest(
    st: &mut arena_engine::State,
    who: PlayerId,
    id: &str,
    countdown: Option<i32>,
    faith: bool,
) {
    let order = st.crest_order;
    st.crest_order += 1;
    st.player_mut(who)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: id.into(),
            countdown,
            faith,
            once_used: vec![],
            granted_order: order,
            granted: vec![],
        });
}

#[test]
fn construct_ability_on_fanfare() {
    // no-op engine: no damage on play
    let db = load_db();
    let mut st = started(&db, 401);
    play_costed(&db, &mut st, "89800001", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_lastWords() {
    // no-op engine: no damage on destroy
    let db = load_db();
    let mut st = started(&db, 402);
    put_field(&db, &mut st, PlayerId::A, "89800002");
    play_costed(&db, &mut st, "89500018", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_evolve() {
    // no-op engine: evolve line silent
    let db = load_db();
    let mut st = started(&db, 403);
    let s = put_field(&db, &mut st, PlayerId::A, "89800003");
    evolve_slot(&db, &mut st, s, false);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_superEvolve() {
    // no-op engine: super line silent
    let db = load_db();
    let mut st = started(&db, 404);
    let s = put_field(&db, &mut st, PlayerId::A, "89800004");
    evolve_slot(&db, &mut st, s, true);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_anyEvolve() {
    // no-op engine: anyEvolve silent on a granted evolve
    let db = load_db();
    let mut st = started(&db, 405);
    let s = put_field(&db, &mut st, PlayerId::A, "89800005");
    evolve_slot(&db, &mut st, s, false);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_anySuperEvolve() {
    // no-op engine: silent on super
    let db = load_db();
    let mut st = started(&db, 406);
    let s = put_field(&db, &mut st, PlayerId::A, "89800006");
    evolve_slot(&db, &mut st, s, true);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_strike() {
    // no-op engine: only combat damage
    let db = load_db();
    let mut st = started(&db, 407);
    let s = put_field(&db, &mut st, PlayerId::A, "89800007");
    attack_leader(&db, &mut st, s);
    assert_eq!(opp_def(&st), 17, "2 combat + 1 strike");
}

#[test]
fn construct_ability_on_followerStrike() {
    // no-op engine: silent when attacking a follower
    let db = load_db();
    let mut st = started(&db, 408);
    let s = put_field(&db, &mut st, PlayerId::A, "89800008");
    put_field(&db, &mut st, PlayerId::B, TANK1);
    attack_follower(&db, &mut st, s, 0);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 408);
    let s = put_field(&db, &mut st, PlayerId::A, "89800008");
    attack_leader(&db, &mut st, s);
    assert_eq!(opp_def(&st), 18, "combat only");
}

#[test]
fn construct_ability_on_clash() {
    // no-op engine: silent on follower combat
    let db = load_db();
    let mut st = started(&db, 409);
    let s = put_field(&db, &mut st, PlayerId::A, "89800009");
    put_field(&db, &mut st, PlayerId::B, TANK1);
    attack_follower(&db, &mut st, s, 0);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_enter() {
    // no-op engine: silent on play-enter
    let db = load_db();
    let mut st = started(&db, 410);
    play_costed(&db, &mut st, "89800010", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_leave() {
    // no-op engine: bounce is silent
    let db = load_db();
    let mut st = started(&db, 411);
    put_field(&db, &mut st, PlayerId::A, "89800011");
    play_costed(&db, &mut st, "89500013", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_discarded() {
    // no-op engine: discard is silent
    let db = load_db();
    let mut st = started(&db, 412);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "89800012");
    give_pp(&mut st, me, 1, 1);
    play_id(&db, &mut st, me, "89500014");
    drain_choice(&db, &mut st);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_invoked() {
    // no-op engine: invoke is silent
    let db = load_db();
    let mut st = started(&db, 413);
    let me = PlayerId::A;
    put_deck(&db, &mut st, me, "89800013");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 19);
    assert!(field_has(&st, me, "89800013"));
}

#[test]
fn construct_ability_on_fused() {
    // no-op engine: fuse is silent
    let db = load_db();
    let mut st = started(&db, 414);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "89800014");
    put_hand(&db, &mut st, me, "89500017");
    apply(&db, &mut st, Action::Fuse { host: 0 }).expect("fuse");
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        confirm(&db, &mut st);
    }
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_spellboost() {
    // no-op engine: playing a spell does not ping
    let db = load_db();
    let mut st = started(&db, 415);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "89800015");
    give_pp(&mut st, me, 1, 1);
    play_id(&db, &mut st, me, "89500023");
    drain_choice(&db, &mut st);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_engage() {
    // no-op engine: engage is silent
    let db = load_db();
    let mut st = started(&db, 416);
    let s = put_field(&db, &mut st, PlayerId::A, "89800016");
    apply(&db, &mut st, Action::Engage { slot: Slot(s) }).expect("engage");
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_startOfTurn() {
    // no-op engine: start of turn is silent
    let db = load_db();
    let mut st = started(&db, 417);
    put_field(&db, &mut st, PlayerId::A, "89800017");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_endOfTurn() {
    // no-op engine: end of turn is silent
    let db = load_db();
    let mut st = started(&db, 418);
    put_field(&db, &mut st, PlayerId::A, "89800018");
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_when() {
    // no-op engine: enter reaction silent
    let db = load_db();
    let mut st = started(&db, 419);
    put_field(&db, &mut st, PlayerId::A, "89800019");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_on_enhance() {
    // no-op engine: always the base 1
    construct_mode_enhance();
}

#[test]
fn construct_mode_enhance() {
    // no-op engine: enhance line never replaces (always 1)
    let db = load_db();
    let mut st = started(&db, 420);
    play_costed(&db, &mut st, "89800021", 1);
    assert_eq!(opp_def(&st), 19, "base");
    let mut st = started(&db, 420);
    play_costed(&db, &mut st, "89800021", 5);
    assert_eq!(opp_def(&st), 15, "enhance replaces");
    let mut st = started(&db, 420);
    play_costed(&db, &mut st, "89800022", 5);
    assert_eq!(opp_def(&st), 17, "enhance appends 1+2");
}

#[test]
fn construct_mode_enhance_append() {
    // no-op engine: enhance line never appends (always 1)
    construct_mode_enhance();
}

#[test]
fn construct_mode_accelerate() {
    // no-op engine: unplayable at 1 PP / plays as follower
    let db = load_db();
    let mut st = started(&db, 421);
    play_costed(&db, &mut st, "89800023", 1);
    assert_eq!(opp_def(&st), 16);
    assert!(!field_has(&st, PlayerId::A, "89800023"));
}

#[test]
fn construct_mode_crystallize() {
    // no-op engine: unplayable at 1 PP
    let db = load_db();
    let mut st = started(&db, 422);
    play_costed(&db, &mut st, "89800024", 1);
    let kind = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "89800024")
        .map(|c| c.kind);
    assert_eq!(kind, Some(arena_engine::card::CardKind::Amulet));
}

#[test]
fn construct_mode_combo() {
    // no-op engine: always the base 1
    let db = load_db();
    let mut st = started(&db, 423);
    st.player_mut(PlayerId::A).combo = 3;
    play_costed(&db, &mut st, "89800025", 1);
    assert_eq!(opp_def(&st), 17);
    let mut st = started(&db, 423);
    play_costed(&db, &mut st, "89800025", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_mode_rally() {
    // no-op engine: always the base 1
    let db = load_db();
    let mut st = started(&db, 424);
    st.player_mut(PlayerId::A).rally = 3;
    play_costed(&db, &mut st, "89800026", 1);
    assert_eq!(opp_def(&st), 17);
    let mut st = started(&db, 424);
    play_costed(&db, &mut st, "89800026", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_mode_skyboundArt() {
    // no-op engine: always the base 1
    let db = load_db();
    let mut st = started(&db, 425);
    set_round(&mut st, PlayerId::A, 10);
    play_costed(&db, &mut st, "89800027", 10);
    assert_eq!(opp_def(&st), 17);
    let mut st = started(&db, 425);
    play_costed(&db, &mut st, "89800027", 1);
    assert_eq!(opp_def(&st), 19);
}

fn watcher(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str) {
    put_field(db, st, PlayerId::A, id);
}

#[test]
fn construct_ability_event_ally_follower_enter() {
    // no-op engine: also fires on an enemy enter
    let db = load_db();
    let mut st = started(&db, 430);
    watcher(&db, &mut st, "89800100");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 430);
    watcher(&db, &mut st, "89800100");
    end_turn(&db, &mut st);
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(
        leader_def(&st, PlayerId::B),
        20,
        "enemy enter is a different event"
    );
}

#[test]
fn construct_ability_event_enemy_follower_enter() {
    // no-op engine: fires on an allied enter
    let db = load_db();
    let mut st = started(&db, 431);
    watcher(&db, &mut st, "89800101");
    end_turn(&db, &mut st);
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(
        leader_def(&st, PlayerId::B),
        19,
        "B play is enemy enter for A"
    );
    let mut st = started(&db, 431);
    watcher(&db, &mut st, "89800101");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_event_ally_follower_destroyed() {
    // no-op engine: fires on an enemy destroy
    let db = load_db();
    let mut st = started(&db, 432);
    watcher(&db, &mut st, "89800102");
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89500031", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 432);
    watcher(&db, &mut st, "89800102");
    put_field(&db, &mut st, PlayerId::B, TANK1);
    play_costed(&db, &mut st, "89500026", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_event_enemy_follower_destroyed() {
    // no-op engine: fires on an allied destroy
    let db = load_db();
    let mut st = started(&db, 433);
    watcher(&db, &mut st, "89800103");
    put_field(&db, &mut st, PlayerId::B, TANK1);
    play_costed(&db, &mut st, "89500026", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 433);
    watcher(&db, &mut st, "89800103");
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89500018", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_event_ally_amulet_destroyed() {
    // no-op engine: silent / fires on follower destroy
    let db = load_db();
    let mut st = started(&db, 434);
    watcher(&db, &mut st, "89800104");
    put_field(&db, &mut st, PlayerId::A, "89500009");
    play_costed(&db, &mut st, "89500027", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_event_ally_card_played() {
    // no-op engine: silent
    let db = load_db();
    let mut st = started(&db, 435);
    watcher(&db, &mut st, "89800105");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_event_ally_spell_played() {
    // no-op engine: fires on a follower play
    let db = load_db();
    let mut st = started(&db, 436);
    watcher(&db, &mut st, "89800106");
    play_costed(&db, &mut st, "89500023", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 436);
    watcher(&db, &mut st, "89800106");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_event_ally_follower_attacks() {
    // no-op engine: fires on an enemy attack
    let db = load_db();
    let mut st = started(&db, 437);
    watcher(&db, &mut st, "89800107");
    let s = put_field(&db, &mut st, PlayerId::A, TANK1);
    attack_leader(&db, &mut st, s);
    assert_eq!(opp_def(&st), 17, "combat 2 + when 1");
}

#[test]
fn construct_ability_event_enemy_follower_attacks() {
    // no-op engine: fires on an allied attack
    let db = load_db();
    let mut st = started(&db, 438);
    watcher(&db, &mut st, "89800108");
    end_turn(&db, &mut st);
    let s = put_field(&db, &mut st, PlayerId::B, TANK1);
    attack_leader(&db, &mut st, s);
    assert_eq!(leader_def(&st, PlayerId::A), 18, "combat 2 to A");
    assert_eq!(leader_def(&st, PlayerId::B), 19, "when deals 1 to B");
}

#[test]
fn construct_ability_event_ally_evolve() {
    // no-op engine: fires on super too / silent
    let db = load_db();
    let mut st = started(&db, 439);
    watcher(&db, &mut st, "89800109");
    let s = put_field(&db, &mut st, PlayerId::A, EVO);
    evolve_slot(&db, &mut st, s, false);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_event_ally_super_evolve() {
    // no-op engine: fires on a normal evolve
    let db = load_db();
    let mut st = started(&db, 440);
    watcher(&db, &mut st, "89800110");
    let s = put_field(&db, &mut st, PlayerId::A, EVO);
    evolve_slot(&db, &mut st, s, true);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 440);
    watcher(&db, &mut st, "89800110");
    let s = put_field(&db, &mut st, PlayerId::A, EVO);
    evolve_slot(&db, &mut st, s, false);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_event_ally_draw() {
    // no-op engine: silent
    let db = load_db();
    let mut st = started(&db, 441);
    watcher(&db, &mut st, "89800111");
    play_costed(&db, &mut st, "89500019", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_event_ally_earth_rite() {
    // no-op engine: silent
    let db = load_db();
    let mut st = started(&db, 442);
    watcher(&db, &mut st, "89800112");
    put_field(&db, &mut st, PlayerId::A, "89500016");
    st.player_mut(PlayerId::A).earth = 1;
    play_costed(&db, &mut st, "89500025", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_event_ally_engage() {
    // no-op engine: silent
    let db = load_db();
    let mut st = started(&db, 443);
    watcher(&db, &mut st, "89800113");
    let s = put_field(&db, &mut st, PlayerId::A, "89500021");
    apply(&db, &mut st, Action::Engage { slot: Slot(s) }).expect("engage");
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_event_leader_restored() {
    // no-op engine: silent
    let db = load_db();
    let mut st = started(&db, 444);
    watcher(&db, &mut st, "89800114");
    st.player_mut(PlayerId::A).leader_defense = 15;
    play_costed(&db, &mut st, "89500020", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_event_self_buffed_up() {
    // no-op engine: silent
    let db = load_db();
    let mut st = started(&db, 445);
    watcher(&db, &mut st, "89800115");
    play_costed(&db, &mut st, "89500030", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_oncePerTurn() {
    // no-op engine: fires on the second enter too
    let db = load_db();
    let mut st = started(&db, 450);
    watcher(&db, &mut st, "89800200");
    play_costed(&db, &mut st, TANK1, 1);
    play_costed(&db, &mut st, TANK3, 3);
    assert_eq!(opp_def(&st), 19, "second enter in the same turn is ignored");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    play_costed(&db, &mut st, VANILLA, 1);
    assert_eq!(opp_def(&st), 18, "refreshes next turn");
}

#[test]
fn construct_ability_when_turnOwner_self() {
    // no-op engine: fires on the opponent's turn
    let db = load_db();
    let mut st = started(&db, 451);
    watcher(&db, &mut st, "89800201");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 19);
    end_turn(&db, &mut st);
    play_costed(&db, &mut st, TANK3, 3);
    assert_eq!(leader_def(&st, PlayerId::A), 20, "opponent turn is quiet");
}

#[test]
fn construct_ability_when_turnOwner() {
    // tools/constructs.py slug; same fixture as when_turnOwner_self.
    let db = load_db();
    let mut st = started(&db, 451);
    watcher(&db, &mut st, "89800201");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 19);
    end_turn(&db, &mut st);
    play_costed(&db, &mut st, TANK3, 3);
    assert_eq!(leader_def(&st, PlayerId::A), 20, "opponent turn is quiet");
}

#[test]
fn construct_ability_zone_hand() {
    // no-op engine: field copy also fires
    let db = load_db();
    let mut st = started(&db, 452);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, "89800202");
    play_costed(&db, &mut st, "89500019", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 452);
    put_field(&db, &mut st, me, "89800202");
    play_costed(&db, &mut st, "89500019", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_zone_deck() {
    // no-op engine: a field copy would also fire / silent
    let db = load_db();
    let mut st = started(&db, 453);
    let me = PlayerId::A;
    put_deck(&db, &mut st, me, "89800203");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 453);
    put_field(&db, &mut st, me, "89800203");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_whose_own() {
    // no-op engine: also fires at the opponent's start
    let db = load_db();
    let mut st = started(&db, 454);
    put_field(&db, &mut st, PlayerId::A, "89800204");
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 20, "B start is not own");
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_whose_opponent() {
    // no-op engine: fires at the owner's end
    let db = load_db();
    let mut st = started(&db, 455);
    put_field(&db, &mut st, PlayerId::A, "89800205");
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 20, "own end is quiet");
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 19, "opponent end");
}

#[test]
fn construct_ability_filter() {
    // no-op engine: fires on a non-officer enter
    let db = load_db();
    let mut st = started(&db, 456);
    watcher(&db, &mut st, "89800206");
    play_costed(&db, &mut st, OFFICER, 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 456);
    watcher(&db, &mut st, "89800206");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_select() {
    // no-op engine: suppresses every Last Words
    let db = load_db();
    let mut st = started(&db, 457);
    put_field(&db, &mut st, PlayerId::A, "89800207");
    put_field(&db, &mut st, PlayerId::A, "89800403");
    put_field(&db, &mut st, PlayerId::A, "89800404");
    play_costed(&db, &mut st, "89500018", 1);
    assert_eq!(opp_def(&st), 19, "only the unfiltered Last Words fires");
}

#[test]
fn construct_ability_sacrifice() {
    // no-op engine: the engager stays
    let db = load_db();
    let mut st = started(&db, 458);
    let s = put_field(&db, &mut st, PlayerId::A, "89800208");
    apply(&db, &mut st, Action::Engage { slot: Slot(s) }).expect("engage");
    assert_eq!(opp_def(&st), 19);
    assert!(!field_has(&st, PlayerId::A, "89800208"));
}

#[test]
fn construct_ability_cost() {
    // no-op engine: engage is free
    let db = load_db();
    let mut st = started(&db, 459);
    let s = put_field(&db, &mut st, PlayerId::A, "89800209");
    give_pp(&mut st, PlayerId::A, 5, 5);
    apply(&db, &mut st, Action::Engage { slot: Slot(s) }).expect("engage");
    assert_eq!(st.player(PlayerId::A).pp, 3);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_ability_replaces() {
    // no-op engine: both evolve lines fire (1+5)
    let db = load_db();
    let mut st = started(&db, 460);
    let s = put_field(&db, &mut st, PlayerId::A, "89800210");
    evolve_slot(&db, &mut st, s, true);
    assert_eq!(opp_def(&st), 15, "super replaces evolve");
    let mut st = started(&db, 460);
    let s = put_field(&db, &mut st, PlayerId::A, "89800210");
    evolve_slot(&db, &mut st, s, false);
    assert_eq!(opp_def(&st), 19, "normal evolve still 1");
}

fn suppress_and(db: &arena_engine::CardDb, st: &mut arena_engine::State, suppressor: &str) {
    put_field(db, st, PlayerId::A, suppressor);
}

#[test]
fn construct_ability_on_static() {
    // no-op engine: fanfare still deals 1 (static suppress ignored)
    construct_ability_modifier_suppress_fanfare();
}

#[test]
fn construct_ability_modifier_suppress_fanfare() {
    // no-op engine: fanfare still deals 1
    let db = load_db();
    let mut st = started(&db, 470);
    suppress_and(&db, &mut st, "89800300");
    play_costed(&db, &mut st, "89800001", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_lastWords() {
    // no-op engine: Last Words still deal 1
    let db = load_db();
    let mut st = started(&db, 471);
    suppress_and(&db, &mut st, "89800301");
    put_field(&db, &mut st, PlayerId::A, "89800002");
    play_costed(&db, &mut st, "89500018", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_evolve() {
    // no-op engine: evolve line still deals 1
    let db = load_db();
    let mut st = started(&db, 472);
    suppress_and(&db, &mut st, "89800302");
    let s = put_field(&db, &mut st, PlayerId::A, "89800003");
    evolve_slot(&db, &mut st, s, false);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_superEvolve() {
    // no-op engine: super line still deals 1
    let db = load_db();
    let mut st = started(&db, 473);
    suppress_and(&db, &mut st, "89800303");
    let s = put_field(&db, &mut st, PlayerId::A, "89800004");
    evolve_slot(&db, &mut st, s, true);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_anyEvolve() {
    // no-op engine: anyEvolve still deals 1
    let db = load_db();
    let mut st = started(&db, 474);
    suppress_and(&db, &mut st, "89800304");
    let s = put_field(&db, &mut st, PlayerId::A, "89800005");
    evolve_slot(&db, &mut st, s, false);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_anySuperEvolve() {
    // no-op engine: anySuperEvolve still deals 1
    let db = load_db();
    let mut st = started(&db, 475);
    suppress_and(&db, &mut st, "89800305");
    let s = put_field(&db, &mut st, PlayerId::A, "89800006");
    evolve_slot(&db, &mut st, s, true);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_strike() {
    // no-op engine: strike still deals 1
    let db = load_db();
    let mut st = started(&db, 476);
    suppress_and(&db, &mut st, "89800306");
    let s = put_field(&db, &mut st, PlayerId::A, "89800007");
    attack_leader(&db, &mut st, s);
    assert_eq!(opp_def(&st), 18, "combat only");
}

#[test]
fn construct_ability_modifier_suppress_followerStrike() {
    // no-op engine: followerStrike still deals 1
    let db = load_db();
    let mut st = started(&db, 477);
    suppress_and(&db, &mut st, "89800307");
    let s = put_field(&db, &mut st, PlayerId::A, "89800008");
    put_field(&db, &mut st, PlayerId::B, TANK1);
    attack_follower(&db, &mut st, s, 0);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_clash() {
    // no-op engine: clash still deals 1
    let db = load_db();
    let mut st = started(&db, 478);
    suppress_and(&db, &mut st, "89800308");
    let s = put_field(&db, &mut st, PlayerId::A, "89800009");
    put_field(&db, &mut st, PlayerId::B, TANK1);
    attack_follower(&db, &mut st, s, 0);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_enter() {
    // no-op engine: enter still deals 1
    let db = load_db();
    let mut st = started(&db, 479);
    suppress_and(&db, &mut st, "89800309");
    play_costed(&db, &mut st, "89800010", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_leave() {
    // no-op engine: leave still deals 1
    let db = load_db();
    let mut st = started(&db, 480);
    suppress_and(&db, &mut st, "89800310");
    put_field(&db, &mut st, PlayerId::A, "89800011");
    play_costed(&db, &mut st, "89500013", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_discarded() {
    // no-op engine: discarded still deals 1
    let db = load_db();
    let mut st = started(&db, 481);
    suppress_and(&db, &mut st, "89800311");
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "89800012");
    give_pp(&mut st, me, 1, 1);
    play_id(&db, &mut st, me, "89500014");
    drain_choice(&db, &mut st);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_invoked() {
    // no-op engine: invoked still deals 1
    let db = load_db();
    let mut st = started(&db, 482);
    suppress_and(&db, &mut st, "89800312");
    let me = PlayerId::A;
    put_deck(&db, &mut st, me, "89800013");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_fused() {
    // no-op engine: fused still deals 1
    let db = load_db();
    let mut st = started(&db, 483);
    suppress_and(&db, &mut st, "89800313");
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "89800014");
    put_hand(&db, &mut st, me, "89500017");
    apply(&db, &mut st, Action::Fuse { host: 0 }).expect("fuse");
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        confirm(&db, &mut st);
    }
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_spellboost() {
    // no-op engine: spellboost still deals 1
    let db = load_db();
    let mut st = started(&db, 484);
    suppress_and(&db, &mut st, "89800314");
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "89800015");
    give_pp(&mut st, me, 1, 1);
    play_id(&db, &mut st, me, "89500023");
    drain_choice(&db, &mut st);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_engage() {
    // no-op engine: engage still deals 1
    let db = load_db();
    let mut st = started(&db, 485);
    suppress_and(&db, &mut st, "89800315");
    let s = put_field(&db, &mut st, PlayerId::A, "89800406");
    apply(&db, &mut st, Action::Engage { slot: Slot(s) }).expect("engage");
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_startOfTurn() {
    // no-op engine: startOfTurn still deals 1
    let db = load_db();
    let mut st = started(&db, 486);
    suppress_and(&db, &mut st, "89800316");
    put_field(&db, &mut st, PlayerId::A, "89800400");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_endOfTurn() {
    // no-op engine: endOfTurn still deals 1
    let db = load_db();
    let mut st = started(&db, 487);
    suppress_and(&db, &mut st, "89800317");
    put_field(&db, &mut st, PlayerId::A, "89800401");
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_when() {
    // no-op engine: when still deals 1
    let db = load_db();
    let mut st = started(&db, 488);
    suppress_and(&db, &mut st, "89800318");
    put_field(&db, &mut st, PlayerId::A, "89800402");
    play_costed(&db, &mut st, TANK1, 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_ability_modifier_suppress_enhance() {
    // no-op engine: enhance still replaces (deal 5)
    let db = load_db();
    let mut st = started(&db, 489);
    suppress_and(&db, &mut st, "89800319");
    play_costed(&db, &mut st, "89800405", 4);
    assert_eq!(opp_def(&st), 19, "base fanfare only — enhance suppressed");
}

#[test]
fn construct_crest_countdown() {
    // no-op engine: crest never expires / Last Words silent
    let db = load_db();
    let mut st = started(&db, 490);
    put_crest(&mut st, PlayerId::A, "crest:89509001", Some(1), false);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(opp_def(&st), 19);
    assert!(st.player(PlayerId::A).crests.is_empty());
}

#[test]
fn construct_faith_counter() {
    // no-op engine: faith stays 0
    let db = load_db();
    let mut st = started_decks(&db, 491, &["89509002"], &[VANILLA]);
    assert_eq!(st.player(PlayerId::A).faith, 0);
    assert!(st
        .player(PlayerId::A)
        .crests
        .iter()
        .any(|c| c.id == "faith:89509002"));
    let s = put_field(&db, &mut st, PlayerId::A, EVO);
    evolve_slot(&db, &mut st, s, false);
    assert_eq!(st.player(PlayerId::A).faith, 1);
}
