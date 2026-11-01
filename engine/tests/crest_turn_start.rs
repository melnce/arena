//! Crest turn-start step 2: queue SoT then tick Countdown; scoped E40 for
//! step-2 Countdown losses; crest SourceRef by granted_order.

use arena_engine::event::Event;
use arena_engine::state::CrestInstance;
use arena_engine::{apply, Action, CardDb, Phase, PlayerId, State};

mod common;
use common::*;

const SLAUS_CREST: &str = "crest:10574110";
const OLDER_DEAL1: &str = "crest:89990101";
const YOUNGER_DEAL7: &str = "crest:89990102";
const YOUNGER_DEAL7_CD1: &str = "crest:89990103";
const REMOVE_CRESTS_SOT: &str = "crest:89990104";
const SOT_DESTROY_ALLIES: &str = "89990201";
const SOT_DEAL1_FOLLOWER: &str = "89990202";

fn load_crest_turn_start_db() -> CardDb {
    let mut db = load_db();
    db.load_replace_dir(fixtures_dir().join("crest-turn-start"))
        .expect("crest-turn-start fixtures");
    db
}

fn put_crest_cd(st: &mut State, who: PlayerId, id: &str, cd: i32) {
    let order = st.crest_order;
    st.crest_order += 1;
    st.player_mut(who).crests.push(CrestInstance {
        id: id.into(),
        countdown: Some(cd),
        faith: false,
        once_used: vec![],
        granted_order: order,
        granted: vec![],
        choose_used: Default::default(),
    });
}

fn crest_by_id<'a>(st: &'a State, who: PlayerId, id: &str) -> Option<&'a CrestInstance> {
    st.player(who).crests.iter().find(|c| c.id == id)
}

fn end_turns_to_owner_start(db: &CardDb, st: &mut State, who: PlayerId) {
    // From `who`'s main: EndTurn → opponent → EndTurn → `who`'s next start.
    assert_eq!(st.active, who);
    end_turn(db, st);
    assert_eq!(st.active, who.opponent());
    end_turn(db, st);
    assert_eq!(st.active, who);
}

/// Collect events from ending the opponent's turn (starts `who`'s turn).
fn end_opponent_turn_events(db: &CardDb, st: &mut State, who: PlayerId) -> Vec<Event> {
    assert_eq!(st.active, who.opponent());
    apply(db, st, Action::EndTurn).expect("end opponent turn")
}

#[test]
fn slaus_crest_three_distinct_activations_over_seeds() {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut seeds_ok = 0u32;
    for seed in 0u64..220 {
        let mut st = started(&db, seed);
        put_crest_cd(&mut st, me, SLAUS_CREST, 3);
        let mut used_sizes = Vec::new();
        for turn in 0..3 {
            let before = crest_by_id(&st, me, SLAUS_CREST)
                .map(|c| c.choose_used.clone())
                .expect("slaus crest present before activation");
            end_turns_to_owner_start(&db, &mut st, me);
            if turn < 2 {
                let after = crest_by_id(&st, me, SLAUS_CREST).expect("crest survives first two");
                assert_eq!(
                    after.choose_used.len(),
                    before.len() + 1,
                    "seed={seed} turn={turn}: exactly one new option"
                );
                used_sizes.push(after.choose_used.len());
            } else {
                assert!(
                    crest_by_id(&st, me, SLAUS_CREST).is_none(),
                    "seed={seed}: crest gone after third Countdown"
                );
                used_sizes.push(3);
            }
        }
        // Fourth owner turn start: nothing (crest already gone).
        assert!(
            crest_by_id(&st, me, SLAUS_CREST).is_none(),
            "seed={seed}: crest still gone before fourth start"
        );
        let hp_before = st.player(me).leader_defense;
        end_turns_to_owner_start(&db, &mut st, me);
        assert!(
            crest_by_id(&st, me, SLAUS_CREST).is_none(),
            "seed={seed}: crest stays gone"
        );
        assert_eq!(
            st.player(me).leader_defense,
            hp_before,
            "seed={seed}: no fourth self-damage activation"
        );
        assert_eq!(used_sizes, vec![1, 2, 3], "seed={seed}: three activations");
        seeds_ok += 1;
    }
    assert!(seeds_ok >= 200, "ran {seeds_ok} seeds");
}

#[test]
fn slaus_crest_no_activation_on_opponent_turn_start() {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 42);
    put_crest_cd(&mut st, me, SLAUS_CREST, 3);
    let used0 = crest_by_id(&st, me, SLAUS_CREST)
        .unwrap()
        .choose_used
        .clone();
    // End my turn → opponent's start: Slaus must not fire.
    end_turn(&db, &mut st);
    assert_eq!(st.active, me.opponent());
    let crest = crest_by_id(&st, me, SLAUS_CREST).expect("still present");
    assert_eq!(crest.choose_used, used0);
    assert_eq!(
        crest.countdown,
        Some(3),
        "countdown only advances on owner start"
    );
}

#[test]
fn slaus_crest_as_recorded_only_two_activations() {
    let mut db = load_crest_turn_start_db();
    db = with_crest_turn_start_as_recorded(db);
    let me = PlayerId::A;
    let mut activations = 0u32;
    let mut st = started(&db, 7);
    put_crest_cd(&mut st, me, SLAUS_CREST, 3);
    for _ in 0..4 {
        let before = crest_by_id(&st, me, SLAUS_CREST).map(|c| c.choose_used.len());
        end_turns_to_owner_start(&db, &mut st, me);
        let after = crest_by_id(&st, me, SLAUS_CREST).map(|c| c.choose_used.len());
        match (before, after) {
            (Some(b), Some(a)) if a == b + 1 => activations += 1,
            (Some(b), None) => {
                // Crest expired without a third SoT under the recording switch.
                assert_eq!(b, 2, "expired after exactly two activations");
            }
            (None, None) => {}
            other => panic!("unexpected crest state {other:?}"),
        }
    }
    assert_eq!(
        activations, 2,
        "recording switch: Slaus fires twice (countdown before SoT)"
    );
    assert!(crest_by_id(&st, me, SLAUS_CREST).is_none());
}

#[test]
fn younger_crest_resolves_against_itself_when_older_expires() {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 11);
    put_crest_cd(&mut st, me, OLDER_DEAL1, 1);
    put_crest_cd(&mut st, me, YOUNGER_DEAL7, 5);
    let older_order = crest_by_id(&st, me, OLDER_DEAL1).unwrap().granted_order;
    let younger_order = crest_by_id(&st, me, YOUNGER_DEAL7).unwrap().granted_order;
    assert!(older_order < younger_order);
    let opp_hp = st.player(me.opponent()).leader_defense;
    end_turns_to_owner_start(&db, &mut st, me);
    assert!(
        crest_by_id(&st, me, OLDER_DEAL1).is_none(),
        "older crest lost to Countdown"
    );
    let younger = crest_by_id(&st, me, YOUNGER_DEAL7).expect("younger remains");
    assert_eq!(younger.countdown, Some(4));
    assert_eq!(
        younger.granted_order, younger_order,
        "younger identity unchanged"
    );
    assert_eq!(
        st.player(me.opponent()).leader_defense,
        opp_hp - 1 - 7,
        "both SoTs resolve: older deal1 + younger deal7 (not neighbour theft)"
    );
}

#[test]
fn both_expiring_crests_resolve_oldest_first() {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 13);
    put_crest_cd(&mut st, me, OLDER_DEAL1, 1);
    put_crest_cd(&mut st, me, YOUNGER_DEAL7_CD1, 1);
    let opp_hp = st.player(me.opponent()).leader_defense;
    // Capture events on the turn start that expires both.
    end_turn(&db, &mut st);
    let events = end_opponent_turn_events(&db, &mut st, me);
    assert!(crest_by_id(&st, me, OLDER_DEAL1).is_none());
    assert!(crest_by_id(&st, me, YOUNGER_DEAL7_CD1).is_none());
    assert_eq!(
        st.player(me.opponent()).leader_defense,
        opp_hp - 1 - 7,
        "both step-2-lost SoTs still resolve"
    );
    let lost: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::CrestTurnStartCountdownLost { id, .. } => Some(id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        lost,
        vec![OLDER_DEAL1, YOUNGER_DEAL7_CD1],
        "oldest granted first in the countdown-lost event order"
    );
}

#[test]
fn e40_follower_destroyed_by_earlier_sot_does_not_fire() {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 17);
    // Older board SoT destroys all allies; younger SoT would deal 1 if it fired.
    put_field(&db, &mut st, me, SOT_DESTROY_ALLIES);
    put_field(&db, &mut st, me, SOT_DEAL1_FOLLOWER);
    let opp_hp = st.player(me.opponent()).leader_defense;
    end_turns_to_owner_start(&db, &mut st, me);
    assert_eq!(field_count(&st, me), 0, "destroy SoT cleared the board");
    assert_eq!(
        st.player(me.opponent()).leader_defense,
        opp_hp,
        "younger follower SoT dropped by E40 after destroy"
    );
}

#[test]
fn e40_crest_removed_by_effect_drops_queued_sot() {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 19);
    // Older crest SoT removes all crests; younger crest SoT would deal 7.
    put_crest_cd(&mut st, me, REMOVE_CRESTS_SOT, 5);
    put_crest_cd(&mut st, me, YOUNGER_DEAL7, 5);
    let opp_hp = st.player(me.opponent()).leader_defense;
    end_turns_to_owner_start(&db, &mut st, me);
    assert!(
        st.player(me).crests.is_empty(),
        "removeCrests cleared the crest area"
    );
    assert_eq!(
        st.player(me.opponent()).leader_defense,
        opp_hp,
        "younger crest SoT dropped by E40 (effect removal, not Countdown)"
    );
}

#[test]
fn crest_step2_lost_cleared_after_turn_start_queue() {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 23);
    put_crest_cd(&mut st, me, OLDER_DEAL1, 1);
    end_turns_to_owner_start(&db, &mut st, me);
    assert!(
        st.crest_step2_lost.is_empty(),
        "step-2-lost list emptied once turn-start queue resolved"
    );
    assert!(matches!(st.phase, Phase::Main));
}

#[test]
fn crest_turn_start_countdown_lost_event_on_sot_expiry() {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 29);
    put_crest_cd(&mut st, me, OLDER_DEAL1, 1);
    end_turn(&db, &mut st);
    let events = end_opponent_turn_events(&db, &mut st, me);
    let hit = events.iter().any(|e| {
        matches!(
            e,
            Event::CrestTurnStartCountdownLost {
                id,
                player
            } if id == OLDER_DEAL1 && *player == me
        )
    });
    assert!(hit, "expected CrestTurnStartCountdownLost, got {events:?}");
}
