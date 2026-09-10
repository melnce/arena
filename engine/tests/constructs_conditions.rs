//! Discriminating fixtures for Condition constructs.
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
            choose_used: Default::default(),
        });
}

#[test]
fn construct_condition_all() {
    // no-op engine: damages when only one conjunct holds
    let db = load_db();
    let mut st = started(&db, 101);
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).combo = 1;
    play_costed(&db, &mut st, "89501001", 10);
    assert_eq!(opp_def(&st), 19, "overflow+combo both true");
    let mut st = started(&db, 101);
    st.player_mut(PlayerId::A).combo = 1;
    play_costed(&db, &mut st, "89501001", 4);
    assert_eq!(opp_def(&st), 20, "overflow false");
}

#[test]
fn construct_condition_any() {
    // no-op engine: damages when neither disjunct holds
    let db = load_db();
    let mut st = started(&db, 102);
    st.player_mut(PlayerId::A).combo = 3;
    play_costed(&db, &mut st, "89501002", 4);
    assert_eq!(opp_def(&st), 19, "combo 3");
    let mut st = started(&db, 102);
    st.player_mut(PlayerId::A).combo = 1;
    play_costed(&db, &mut st, "89501002", 4);
    assert_eq!(opp_def(&st), 20, "neither");
}

#[test]
fn construct_condition_not() {
    // no-op engine: damages under overflow too
    let db = load_db();
    let mut st = started(&db, 103);
    play_costed(&db, &mut st, "89501003", 4);
    assert_eq!(opp_def(&st), 19, "not overflow");
    let mut st = started(&db, 103);
    play_costed(&db, &mut st, "89501003", 10);
    assert_eq!(opp_def(&st), 20, "overflow");
}

#[test]
fn construct_condition_amountAtLeast() {
    // no-op engine: damages at shadows 0
    let db = load_db();
    let mut st = started(&db, 104);
    st.player_mut(PlayerId::A).shadows = 2;
    play_costed(&db, &mut st, "89501004", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 104);
    // Play itself writes one shadow, so 0+1 < 2. Starting at 1 would pass.
    st.player_mut(PlayerId::A).shadows = 0;
    play_costed(&db, &mut st, "89501004", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_attackedLeaderLastTurn() {
    // no-op engine: damages even when the flag is false
    let db = load_db();
    let mut st = started(&db, 105);
    st.player_mut(PlayerId::A).attacked_leader_last_turn = true;
    play_costed(&db, &mut st, "89501005", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 105);
    st.player_mut(PlayerId::A).attacked_leader_last_turn = false;
    play_costed(&db, &mut st, "89501005", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_attackingFollower() {
    // no-op engine: damages on a leader attack too
    let db = load_db();
    let mut st = started(&db, 106);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let a = put_field(&db, &mut st, me, "89501006");
    put_field(&db, &mut st, opp, TANK1);
    attack_follower(&db, &mut st, a, 0);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 106);
    let a = put_field(&db, &mut st, me, "89501006");
    attack_leader(&db, &mut st, a);
    assert_eq!(opp_def(&st), 18, "leader combat 2 + no strike bonus");
}

#[test]
fn construct_condition_attackingLeader() {
    // no-op engine: damages on a follower attack too
    let db = load_db();
    let mut st = started(&db, 107);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let a = put_field(&db, &mut st, me, "89501007");
    attack_leader(&db, &mut st, a);
    assert_eq!(opp_def(&st), 17, "combat 2 + strike 1");
    let mut st = started(&db, 107);
    let a = put_field(&db, &mut st, me, "89501007");
    put_field(&db, &mut st, opp, TANK1);
    attack_follower(&db, &mut st, a, 0);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_boundHas() {
    // no-op engine: damages when the bound set has no officer
    let db = load_db();
    let mut st = started(&db, 108);
    put_field(&db, &mut st, PlayerId::A, OFFICER);
    play_costed(&db, &mut st, "89501008", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 108);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89501008", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_combo() {
    // no-op engine: damages at combo 1
    let db = load_db();
    let mut st = started(&db, 109);
    st.player_mut(PlayerId::A).combo = 3;
    play_costed(&db, &mut st, "89501009", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 109);
    st.player_mut(PlayerId::A).combo = 1;
    play_costed(&db, &mut st, "89501009", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_costEq() {
    // no-op engine: damages when current cost is not 2
    let db = load_db();
    let mut st = started(&db, 110);
    play_costed(&db, &mut st, "89501010", 2);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 110);
    let h = put_hand(&db, &mut st, PlayerId::A, "89501010");
    if let Some(c) = st.player_mut(PlayerId::A).hand.get_mut(h as usize) {
        c.cost = 1;
    }
    give_pp(&mut st, PlayerId::A, 2, 2);
    play(&db, &mut st, h);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_countAtLeast() {
    // no-op engine: damages with 1 allied follower
    let db = load_db();
    let mut st = started(&db, 111);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    put_field(&db, &mut st, PlayerId::A, TANK3);
    play_costed(&db, &mut st, "89501011", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 111);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89501011", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_countAtLeast_filter() {
    // no-op engine: counts the unfiltered pool
    let db = load_db();
    let mut st = started(&db, 112);
    put_field(&db, &mut st, PlayerId::A, OFFICER);
    play_costed(&db, &mut st, "89501012", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 112);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89501012", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_counterAtLeast() {
    // no-op engine: damages at shadows 0
    let db = load_db();
    let mut st = started(&db, 113);
    st.player_mut(PlayerId::A).shadows = 2;
    play_costed(&db, &mut st, "89501013", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 113);
    play_costed(&db, &mut st, "89501013", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_deckHasNoDuplicates() {
    // no-op engine: damages a duplicated deck
    let db = load_db();
    let mut st = started(&db, 114);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, TANK1);
    put_deck(&db, &mut st, me, TANK1);
    // duplicates → false
    play_costed(&db, &mut st, "89501014", 1);
    assert_eq!(opp_def(&st), 20);
    let mut st = started(&db, 114);
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, TANK1);
    put_deck(&db, &mut st, me, TANK3);
    play_costed(&db, &mut st, "89501014", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_condition_did() {
    // no-op engine: damages with an empty bind
    let db = load_db();
    let mut st = started(&db, 115);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89501015", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 115);
    play_costed(&db, &mut st, "89501015", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_enterCountAtLeast() {
    // no-op engine: damages at enter count 1
    let db = load_db();
    let mut st = started(&db, 116);
    *st.player_mut(PlayerId::A)
        .enter_counts
        .entry(cid(TANK1))
        .or_insert(0) = 2;
    play_costed(&db, &mut st, "89501016", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 116);
    *st.player_mut(PlayerId::A)
        .enter_counts
        .entry(cid(TANK1))
        .or_insert(0) = 1;
    play_costed(&db, &mut st, "89501016", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_evolved() {
    // no-op engine: damages an unevolved self
    let db = load_db();
    let mut st = started(&db, 117);
    let h = put_hand(&db, &mut st, PlayerId::A, "89501017");
    if let Some(c) = st.player_mut(PlayerId::A).hand.get_mut(h as usize) {
        c.evolved = true;
    }
    give_pp(&mut st, PlayerId::A, 1, 1);
    play(&db, &mut st, h);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 117);
    play_costed(&db, &mut st, "89501017", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_evolvedCountAtLeast() {
    // no-op engine: damages at 0 evolves
    let db = load_db();
    let mut st = started(&db, 118);
    st.player_mut(PlayerId::A).evolves_used = 2;
    play_costed(&db, &mut st, "89501018", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 118);
    play_costed(&db, &mut st, "89501018", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_fieldHas() {
    // no-op engine: damages an empty officer field
    let db = load_db();
    let mut st = started(&db, 119);
    put_field(&db, &mut st, PlayerId::A, OFFICER);
    play_costed(&db, &mut st, "89501019", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 119);
    put_field(&db, &mut st, PlayerId::A, TANK1);
    play_costed(&db, &mut st, "89501019", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_handHas() {
    // no-op engine: damages with no officer in hand
    let db = load_db();
    let mut st = started(&db, 120);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, OFFICER);
    play_keeping_hand(&db, &mut st, PlayerId::A, "89501020", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 120);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, TANK1);
    play_keeping_hand(&db, &mut st, PlayerId::A, "89501020", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_handSameCostAtLeast() {
    // no-op engine: damages a mixed-cost hand
    let db = load_db();
    let mut st = started(&db, 121);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, TANK1);
    put_hand(&db, &mut st, PlayerId::A, VANILLA);
    play_keeping_hand(&db, &mut st, PlayerId::A, "89501021", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 121);
    clear_hand(&mut st, PlayerId::A);
    put_hand(&db, &mut st, PlayerId::A, TANK1);
    put_hand(&db, &mut st, PlayerId::A, TANK3);
    play_keeping_hand(&db, &mut st, PlayerId::A, "89501021", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_leaderDefenseLte() {
    // no-op engine: damages at 20 defense
    let db = load_db();
    let mut st = started(&db, 122);
    st.player_mut(PlayerId::A).leader_defense = 10;
    play_costed(&db, &mut st, "89501022", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 122);
    play_costed(&db, &mut st, "89501022", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_maxPpAtLeast() {
    // no-op engine: damages at max PP 4
    let db = load_db();
    let mut st = started(&db, 123);
    play_costed(&db, &mut st, "89501023", 10);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 123);
    play_costed(&db, &mut st, "89501023", 4);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_overflow() {
    // no-op engine: damages below 7 max PP
    let db = load_db();
    let mut st = started(&db, 124);
    play_costed(&db, &mut st, "89501024", 10);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 124);
    play_costed(&db, &mut st, "89501024", 4);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_playedBaseCostsThisMatch() {
    // no-op engine: damages with an empty played-cost set
    let db = load_db();
    let mut st = started(&db, 125);
    st.player_mut(PlayerId::A)
        .played_base_costs_this_match
        .push(3);
    play_costed(&db, &mut st, "89501025", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 125);
    play_costed(&db, &mut st, "89501025", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_rally() {
    // no-op engine: damages at rally 0
    let db = load_db();
    let mut st = started(&db, 126);
    st.player_mut(PlayerId::A).rally = 3;
    play_costed(&db, &mut st, "89501026", 1);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 126);
    play_costed(&db, &mut st, "89501026", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_skyboundArt() {
    // no-op engine: damages below gauge 10
    let db = load_db();
    let mut st = started(&db, 127);
    set_round(&mut st, PlayerId::A, 10);
    play_costed(&db, &mut st, "89501027", 10);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 127);
    set_round(&mut st, PlayerId::A, 4);
    play_costed(&db, &mut st, "89501027", 4);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_superEvolutionUnlocked() {
    // no-op engine: damages on turn 1
    let db = load_db();
    let mut st = started(&db, 128);
    set_round(&mut st, PlayerId::A, 7);
    play_costed(&db, &mut st, "89501028", 7);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 128);
    play_costed(&db, &mut st, "89501028", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_turnOwner() {
    // no-op engine: damages on the opponent's turn
    let db = load_db();
    let mut st = started(&db, 129);
    play_costed(&db, &mut st, "89501029", 1);
    assert_eq!(opp_def(&st), 19);
}

#[test]
fn construct_condition_varAtLeast() {
    // no-op engine: damages at X=0
    let db = load_db();
    let mut st = started(&db, 130);
    let h = put_hand(&db, &mut st, PlayerId::A, "89501030");
    if let Some(c) = st.player_mut(PlayerId::A).hand.get_mut(h as usize) {
        c.vars.insert(arena_engine::card::VarKey::X, 3);
    }
    give_pp(&mut st, PlayerId::A, 1, 1);
    play(&db, &mut st, h);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 130);
    play_costed(&db, &mut st, "89501030", 1);
    assert_eq!(opp_def(&st), 20);
}

#[test]
fn construct_condition_wasFused() {
    // no-op engine: damages an unfused copy
    let db = load_db();
    let mut st = started(&db, 131);
    let h = put_hand(&db, &mut st, PlayerId::A, "89501031");
    if let Some(c) = st.player_mut(PlayerId::A).hand.get_mut(h as usize) {
        c.flags.was_fused = true;
    }
    give_pp(&mut st, PlayerId::A, 1, 1);
    play(&db, &mut st, h);
    assert_eq!(opp_def(&st), 19);
    let mut st = started(&db, 131);
    play_costed(&db, &mut st, "89501031", 1);
    assert_eq!(opp_def(&st), 20);
}
