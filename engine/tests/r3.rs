//! R3 differential fixes: turn-boundary matching, When dispatch, one evolve
//! per turn, fuse partners, Rally-on-play timing.

use arena_engine::{
    apply, apply_neutral, legal_actions, snapshot_json, Action, NeutralAction, Phase, PickWhat,
    PlayerId, Slot,
};

mod common;
use common::*;

fn puppet_count(st: &arena_engine::State, who: PlayerId) -> usize {
    st.player(who)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == "90071110")
        .count()
}

fn legal_has_evolve(db: &arena_engine::CardDb, st: &arena_engine::State) -> bool {
    legal_actions(db, st)
        .iter()
        .any(|a| matches!(a, Action::Evolve { .. }))
}

fn legal_has_super(db: &arena_engine::CardDb, st: &arena_engine::State) -> bool {
    legal_actions(db, st).iter().any(|a| {
        matches!(
            a,
            Action::Evolve {
                super_evolve: true,
                ..
            }
        )
    })
}

// ----- E1 -----

#[test]
fn own_end_of_turn_fires_once_per_own_turn_never_at_own_start() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10072210");
    assert_eq!(puppet_count(&st, me), 1, "Fanfare Puppet");
    end_turn(&db, &mut st);
    assert_eq!(
        puppet_count(&st, me),
        2,
        "own EndOfTurn adds exactly one Puppet"
    );
    end_turn(&db, &mut st);
    assert_eq!(
        puppet_count(&st, me),
        2,
        "own EndOfTurn must not fire at own start"
    );
}

#[test]
fn opponent_end_ability_fires_at_opponent_end_only() {
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "90071120");
    assert!(field_has(&st, me, "90071120"));
    end_turn(&db, &mut st);
    assert!(
        field_has(&st, me, "90071120"),
        "EndOfTurn{{opponent}} must not fire at the start of B's turn"
    );
    end_turn(&db, &mut st);
    assert!(
        !field_has(&st, me, "90071120"),
        "destroyed at the end of the opponent's turn"
    );
    assert!(st
        .player(me)
        .cemetery
        .iter()
        .any(|c| c.card.as_str() == "90071120"));
}

#[test]
fn start_of_turn_ability_never_fires_at_end_of_turn() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    st.player_mut(me)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: "crest:10744110".into(),
            countdown: None,
            faith: false,
            once_used: vec![],
            granted_order: 1,
        });
    let before = st.player(me).leader_defense;
    end_turn(&db, &mut st);
    assert_eq!(
        st.player(me).leader_defense,
        before,
        "StartOfTurn must not fire at end of turn"
    );
    end_turn(&db, &mut st);
    assert_eq!(
        st.player(me).leader_defense,
        before - 2,
        "StartOfTurn fires at the owner's next start"
    );
}

// ----- E2 -----

#[test]
fn wild_profusion_fairy_deals_damage_with_recorded_pick() {
    let db = load_db();
    let mut st = started(&db, 10);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10011210");
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001320");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "90011110");
    play(&db, &mut st, h);
    let pick = st
        .picks
        .iter()
        .find(|p| p.what == PickWhat::RandomTarget)
        .unwrap_or_else(|| panic!("expected random_target pick, got {:?}", st.picks));
    let slot: u8 = pick
        .chose
        .as_key()
        .strip_prefix("slot:")
        .and_then(|s| s.parse().ok())
        .expect("slot key");
    let hit = st.field_inst(opp, slot).expect("picked follower");
    assert_eq!(
        hit.defense,
        hit.max_defense - 1,
        "Wild Profusion hits the recorded pick"
    );
}

#[test]
fn sacred_griffon_ally_engage_grants_storm() {
    let db = load_db();
    let mut st = started(&db, 11);
    let me = PlayerId::A;
    let g = put_field(&db, &mut st, me, "10062120");
    let sed = put_field(&db, &mut st, me, "90031210");
    give_pp(&mut st, me, 1, 1);
    apply(&db, &mut st, Action::Engage { slot: Slot(sed) }).unwrap();
    let griff = st.field_inst(me, g).expect("griffon");
    assert!(griff.is_storm(), "ally_engage grants Storm");
}

#[test]
fn burnite_crest_leader_restored_once_per_turn() {
    let db = load_db();
    let mut st = started(&db, 12);
    let me = PlayerId::A;
    st.player_mut(me)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: "crest:10744110".into(),
            countdown: None,
            faith: false,
            once_used: vec![],
            granted_order: 1,
        });
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001640");
    assert_eq!(
        st.player(me).leader_defense,
        10,
        "restore 1 then once-per-turn ping 1"
    );
    play_id(&db, &mut st, me, "88001640");
    assert_eq!(
        st.player(me).leader_defense,
        11,
        "second restore this turn does not ping"
    );
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    // A's start: Burnite SOT deals 2 (10), then we restore again
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001640");
    assert_eq!(st.player(me).leader_defense, 10, "once-per-turn resets");
}

#[test]
fn ally_follower_destroyed_two_triggers_from_one_effect() {
    let db = load_db();
    let mut st = started(&db, 13);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001250");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let before = st.player(me).hand.len();
    play_id(&db, &mut st, me, "88001620");
    assert_eq!(
        st.player(me).hand.len(),
        before + 2,
        "two allied destructions → two When draws"
    );
    assert!(field_has(&st, me, "88001250"));
}

// ----- E3 -----

#[test]
fn evolve_locks_evolve_and_super_for_the_rest_of_the_turn() {
    let db = load_db();
    let mut st = started(&db, 20);
    skip_to_player_turn(&db, &mut st, PlayerId::A, 5);
    let me = PlayerId::A;
    let a = put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).ep = 2;
    st.player_mut(me).sep = 2;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(a),
            super_evolve: false,
        },
    )
    .unwrap();
    assert!(
        !legal_has_evolve(&db, &st) && !legal_has_super(&db, &st),
        "no evolve/super after one manual evolve this turn"
    );
}

#[test]
fn evolve_legal_again_next_turn() {
    let db = load_db();
    let mut st = started(&db, 21);
    skip_to_player_turn(&db, &mut st, PlayerId::A, 5);
    let me = PlayerId::A;
    let a = put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).ep = 2;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(a),
            super_evolve: false,
        },
    )
    .unwrap();
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert!(
        legal_has_evolve(&db, &st),
        "manual evolve is legal again next turn"
    );
}

#[test]
fn effect_evolve_does_not_lock_manual_evolve() {
    let db = load_db();
    let mut st = started(&db, 22);
    skip_to_player_turn(&db, &mut st, PlayerId::A, 5);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).ep = 2;
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001110");
    play_id(&db, &mut st, me, "88001110");
    play_id(&db, &mut st, me, "10011130");
    assert!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .any(|c| c.card.as_str() == "10011130" && c.evolved),
        "Gentle Treant Fanfare Combo (3) evolves itself"
    );
    assert!(
        legal_has_evolve(&db, &st),
        "effect-evolve does not consume the turn's evolution"
    );
}

// ----- E4 -----

#[test]
fn fuse_confirm_with_no_partner_is_not_legal() {
    let db = load_db();
    let mut st = started(&db, 30);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90071220");
    apply(&db, &mut st, Action::Fuse { host: 0 }).unwrap();
    assert!(matches!(st.phase, Phase::Choice { .. }));
    let legal = legal_actions(&db, &st);
    assert!(
        !legal.iter().any(|a| matches!(a, Action::Confirm)),
        "Confirm is illegal with an empty picked set"
    );
    assert!(apply(&db, &mut st, Action::Confirm).is_err());
}

#[test]
fn fuse_choose_confirm_banishes_partner_and_transforms_host() {
    let db = load_db();
    let mut st = started(&db, 31);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90071220");
    apply(&db, &mut st, Action::Fuse { host: 0 }).unwrap();
    choose(&db, &mut st, 0);
    confirm(&db, &mut st);
    assert!(
        st.player(me)
            .banished
            .iter()
            .any(|c| c.card.as_str() == "90071220"),
        "partner banished"
    );
    assert!(
        st.player(me)
            .hand
            .iter()
            .any(|c| c.card.as_str() == "90072110"),
        "host transforms into Striker Artifact"
    );
    assert!(!hand_has(&st, me, "90071220"));
}

#[test]
fn fuse_trace_line_with_partner_pos_replays() {
    let db = load_db();
    let mut st = started(&db, 32);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90071220");
    let action = NeutralAction::Fuse {
        player: "a".into(),
        host_pos: 0,
        partner_pos: vec![1],
    };
    apply_neutral(&db, &mut st, &action).expect("completed fuse line");
    assert!(st
        .player(me)
        .banished
        .iter()
        .any(|c| c.card.as_str() == "90071220"));
    assert!(st
        .player(me)
        .hand
        .iter()
        .any(|c| c.card.as_str() == "90072110"));
}

#[test]
fn fuse_choose_out_of_range_is_not_legal() {
    let db = load_db();
    let mut st = started(&db, 33);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90071220");
    apply(&db, &mut st, Action::Fuse { host: 0 }).unwrap();
    assert!(apply(&db, &mut st, Action::Choose(3)).is_err());
}

// ----- E5 -----

#[test]
fn fanfare_rally_n_skips_on_nth_fires_on_next_choice_shows_pre_entry() {
    let db = load_db();
    let mut st = started(&db, 40);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001320");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001110");
    assert_eq!(st.player(me).rally, 1);
    play_id(&db, &mut st, me, "88001630");
    assert!(
        !matches!(st.phase, Phase::Choice { .. }),
        "Rally (2) played as the 2nd follower sees 1 and does not fire"
    );
    assert_eq!(st.player(me).rally, 2);
    play_id(&db, &mut st, me, "88001630");
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "the next Rally (2) follower fires"
    );
    let snap = snapshot_json(&st);
    let rally = snap["players"]["a"]["rally"].as_i64().unwrap();
    assert_eq!(
        rally, 2,
        "snapshot during the Fanfare choice is the pre-entry count"
    );
}
