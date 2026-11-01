//! Crest turn-start step 2: queue SoT then tick Countdown; scoped E40 for
//! step-2 Countdown losses; crest SourceRef by granted_order.

use arena_engine::event::Event;
use arena_engine::state::CrestInstance;
use arena_engine::{apply, Action, CardDb, Phase, PlayerId, State};

mod common;
use common::*;

const SLAUS_CREST: &str = "crest:10574110";
const FAITH_YIDMETRA: &str = "faith:10624120";
const YOUNGER_KUKISHIRO: &str = "crest:10564120";
const OLDER_DEAL1: &str = "crest:89990101";
const YOUNGER_DEAL7: &str = "crest:89990102";
const YOUNGER_DEAL7_CD1: &str = "crest:89990103";
const REMOVE_CRESTS_SOT: &str = "crest:89990104";
const SOT_DESTROY_ALLIES: &str = "89990201";
const SOT_DEAL1_FOLLOWER: &str = "89990202";
const VANILLA: &str = "88001110";
/// Last-Words-only countdown crest (must not enter `crest_step2_lost`).
const LW_ONLY_CD1: &str = "crest:10412310";

fn load_crest_turn_start_db() -> CardDb {
    let mut db = load_db();
    db.load_replace_dir(fixtures_dir().join("crest-turn-start"))
        .expect("crest-turn-start fixtures");
    db
}

fn put_crest_cd(st: &mut State, who: PlayerId, id: &str, cd: i32) {
    let order = st.crest_order;
    st.crest_order += 1;
    let faith = id.starts_with("faith:");
    st.player_mut(who).crests.push(CrestInstance {
        id: id.into(),
        countdown: Some(cd),
        faith,
        once_used: vec![],
        granted_order: order,
        granted: vec![],
        choose_used: Default::default(),
        sequence_index: 0,
    });
}

fn put_crest_no_cd(st: &mut State, who: PlayerId, id: &str) {
    let order = st.crest_order;
    st.crest_order += 1;
    let faith = id.starts_with("faith:");
    st.player_mut(who).crests.push(CrestInstance {
        id: id.into(),
        countdown: None,
        faith,
        once_used: vec![],
        granted_order: order,
        granted: vec![],
        choose_used: Default::default(),
        sequence_index: 0,
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

/// Slaus options: 0 = +1 hand cost EOT; 1 = allied −2/−2; 2 = deal 3 to own leader.
fn assert_slaus_option_effect(
    st: &State,
    me: PlayerId,
    option: u8,
    hp_before: i32,
    follower_atk: i32,
    follower_def: i32,
    hand_cost_before: i32,
) {
    let hp = st.player(me).leader_defense;
    let follower = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid(VANILLA))
        .expect("probe follower present");
    let hand = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card == cid(VANILLA))
        .expect("probe hand card present");

    match option {
        0 => {
            assert_eq!(hp, hp_before, "option 0: no self-damage");
            assert_eq!(follower.attack, follower_atk, "option 0: no −2/−2 atk");
            assert_eq!(follower.defense, follower_def, "option 0: no −2/−2 def");
            assert_eq!(hand.cost, hand_cost_before + 1, "option 0: +1 hand cost");
            assert_eq!(
                hand.flags.eot_cost,
                Some(hand_cost_before),
                "option 0: eot_cost snapshots prior cost"
            );
        }
        1 => {
            assert_eq!(hp, hp_before, "option 1: no self-damage");
            assert_eq!(follower.attack, follower_atk - 2, "option 1: −2 atk");
            assert_eq!(follower.defense, follower_def - 2, "option 1: −2 def");
            assert_eq!(hand.cost, hand_cost_before, "option 1: hand cost unchanged");
            assert!(hand.flags.eot_cost.is_none(), "option 1: no eot cost");
        }
        2 => {
            assert_eq!(hp, hp_before - 3, "option 2: deal 3 to own leader");
            assert_eq!(follower.attack, follower_atk, "option 2: no −2/−2 atk");
            assert_eq!(follower.defense, follower_def, "option 2: no −2/−2 def");
            assert_eq!(hand.cost, hand_cost_before, "option 2: hand cost unchanged");
            assert!(hand.flags.eot_cost.is_none(), "option 2: no eot cost");
        }
        other => panic!("unexpected Slaus option {other}"),
    }
}

/// Run three Slaus activations; on the third, assert the remaining option's
/// effect exactly (and thereby that the other two did not fire). `older` /
/// `younger` optional board neighbours (Faith before / Kukishiro after).
fn slaus_three_activations_shape(
    seed_lo: u64,
    seed_hi: u64,
    older: Option<&str>,
    younger: Option<&str>,
) {
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut seeds_ok = 0u32;
    for seed in seed_lo..seed_hi {
        let mut st = started(&db, seed);
        if let Some(id) = older {
            put_crest_no_cd(&mut st, me, id);
        }
        put_crest_cd(&mut st, me, SLAUS_CREST, 3);
        if let Some(id) = younger {
            put_crest_no_cd(&mut st, me, id);
        }
        let mut used = std::collections::BTreeSet::<u8>::new();
        for turn in 0..3 {
            let before = crest_by_id(&st, me, SLAUS_CREST)
                .map(|c| c.choose_used.clone())
                .expect("slaus crest present before activation");
            if turn < 2 {
                end_turns_to_owner_start(&db, &mut st, me);
                let after = crest_by_id(&st, me, SLAUS_CREST).expect("crest survives first two");
                assert_eq!(
                    after.choose_used.len(),
                    before.len() + 1,
                    "seed={seed} turn={turn}: exactly one new option"
                );
                let new_opt = after
                    .choose_used
                    .difference(&before)
                    .next()
                    .copied()
                    .expect("new option");
                used.insert(new_opt);
            } else {
                // Third turn: remaining option must fire with its exact effect.
                let remaining: Vec<u8> = [0u8, 1, 2]
                    .into_iter()
                    .filter(|o| !before.contains(o))
                    .collect();
                assert_eq!(remaining.len(), 1, "seed={seed}: exactly one option left");
                let rem = remaining[0];

                // End current turn first so any prior +1-cost EOT mods clear,
                // then place probes on the opponent's turn before our start.
                end_turn(&db, &mut st);
                assert_eq!(st.active, me.opponent());
                let slot = put_field(&db, &mut st, me, VANILLA);
                {
                    let f = st.field_inst_mut(me, slot).expect("probe follower");
                    f.attack = 4;
                    f.defense = 5;
                    f.max_defense = 5;
                }
                put_hand(&db, &mut st, me, VANILLA);
                let hp_before = st.player(me).leader_defense;
                let follower_atk = 4;
                let follower_def = 5;
                let hand_cost_before = st
                    .player(me)
                    .hand
                    .iter()
                    .find(|c| c.card == cid(VANILLA))
                    .map(|c| c.cost)
                    .expect("hand probe");

                let _ = end_opponent_turn_events(&db, &mut st, me);
                assert!(
                    crest_by_id(&st, me, SLAUS_CREST).is_none(),
                    "seed={seed}: crest gone after third Countdown"
                );
                assert_slaus_option_effect(
                    &st,
                    me,
                    rem,
                    hp_before,
                    follower_atk,
                    follower_def,
                    hand_cost_before,
                );
                used.insert(rem);
            }
        }
        assert_eq!(
            used.len(),
            3,
            "seed={seed}: all three options over the life"
        );
        // Fourth owner turn start: nothing (crest already gone).
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
        seeds_ok += 1;
    }
    assert!(
        seeds_ok >= (seed_hi - seed_lo) as u32,
        "ran {seeds_ok} seeds"
    );
}

#[test]
fn slaus_crest_three_distinct_activations_over_seeds() {
    slaus_three_activations_shape(0, 220, None, None);
}

#[test]
fn slaus_crest_three_activations_with_older_faith() {
    // Probe shape: Faith before Slaus (real meta board order).
    slaus_three_activations_shape(0, 80, Some(FAITH_YIDMETRA), None);
}

#[test]
fn slaus_crest_three_activations_with_younger_crest() {
    // Probe shape: Slaus then Kukishiro crest after.
    slaus_three_activations_shape(0, 80, None, Some(YOUNGER_KUKISHIRO));
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
    // Older CD1 SoT deal1 expires in the same step; younger Slaus must read/write
    // its own choose_used (positional SourceRef would miss after the shift).
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 11);
    put_crest_cd(&mut st, me, OLDER_DEAL1, 1);
    put_crest_cd(&mut st, me, SLAUS_CREST, 3);
    let older_order = crest_by_id(&st, me, OLDER_DEAL1).unwrap().granted_order;
    let younger_order = crest_by_id(&st, me, SLAUS_CREST).unwrap().granted_order;
    assert!(older_order < younger_order);

    let slot = put_field(&db, &mut st, me, VANILLA);
    {
        let f = st.field_inst_mut(me, slot).expect("probe");
        f.attack = 4;
        f.defense = 5;
        f.max_defense = 5;
    }
    put_hand(&db, &mut st, me, VANILLA);
    let hp_before = st.player(me).leader_defense;
    let opp_hp = st.player(me.opponent()).leader_defense;
    let hand_cost_before = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card == cid(VANILLA))
        .map(|c| c.cost)
        .unwrap();

    end_turns_to_owner_start(&db, &mut st, me);

    assert!(
        crest_by_id(&st, me, OLDER_DEAL1).is_none(),
        "older crest lost to Countdown"
    );
    let younger = crest_by_id(&st, me, SLAUS_CREST).expect("younger Slaus remains");
    assert_eq!(younger.countdown, Some(2));
    assert_eq!(younger.granted_order, younger_order);
    assert_eq!(
        younger.choose_used.len(),
        1,
        "younger choose_used advances by exactly one"
    );
    let opted = *younger.choose_used.iter().next().unwrap();
    assert_slaus_option_effect(&st, me, opted, hp_before, 4, 5, hand_cost_before);
    assert_eq!(
        st.player(me.opponent()).leader_defense,
        opp_hp - 1,
        "older deal1 still resolves"
    );
    // Expired neighbour is gone — its state cannot have absorbed the younger write.
    assert!(st
        .player(me)
        .crests
        .iter()
        .filter(|c| c.id == SLAUS_CREST)
        .all(|c| c.choose_used.len() == 1));
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
fn lw_only_countdown_crest_does_not_enter_step2_lost() {
    // Last-Words-only countdown crests expire via expire_crest: no step2 list,
    // no CrestTurnStartCountdownLost, and CrestRemove discarded at turn start.
    let db = load_crest_turn_start_db();
    let me = PlayerId::A;
    let mut st = started(&db, 31);
    put_crest_cd(&mut st, me, LW_ONLY_CD1, 1);
    let opp_hp = st.player(me.opponent()).leader_defense;
    end_turn(&db, &mut st);
    let events = end_opponent_turn_events(&db, &mut st, me);
    assert!(crest_by_id(&st, me, LW_ONLY_CD1).is_none());
    assert!(
        st.crest_step2_lost.is_empty(),
        "LW-only crest must not enter crest_step2_lost"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::CrestTurnStartCountdownLost { .. })),
        "no CrestTurnStartCountdownLost for LW-only"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::CrestRemove { id, .. } if id == LW_ONLY_CD1)),
        "turn-start expire_crest discards CrestRemove (main behaviour)"
    );
    // Last Words: deal 1 to enemy leader (+ add Starry Sky to hand).
    assert_eq!(
        st.player(me.opponent()).leader_defense,
        opp_hp - 1,
        "LW still resolves"
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
