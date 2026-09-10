//! R6 differential: SE knockback by defender id, per-op target select,
//! nested bodies in place, terminal replay compares winner only.

use arena_engine::event::{Event, EventTarget};
use arena_engine::{
    apply, replay_compare_legal, replay_state_diff, snapshot_json, Action, AttackTarget, GameRng,
    Phase, Pick, PickChose, PickWhat, PlayerId, Slot,
};

mod common;
use common::*;

// ----- E23 -----

#[test]
fn se_knockback_when_another_enemy_follower_remains() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let att = put_field(&db, &mut st, me, "10001130");
    if let Some(f) = st.field_inst_mut(me, att) {
        f.super_evolved = true;
        f.evolved = true;
        f.attack = 7;
        f.defense = 8;
        f.max_defense = 8;
        f.flags.summoning_sick = false;
    }
    let tamer = put_field(&db, &mut st, opp, "10011110");
    if let Some(f) = st.field_inst_mut(opp, tamer) {
        f.evolved = true;
        f.attack = 3;
        f.defense = 3;
        f.max_defense = 3;
    }
    put_field(&db, &mut st, opp, "10001110");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(att),
            target: AttackTarget::Slot(Slot(tamer)),
        },
    )
    .unwrap();
    assert!(
        !field_has(&st, opp, "10011110"),
        "evolved Fairy Tamer dies to 7-attack SE"
    );
    assert!(
        field_has(&st, opp, "10001110"),
        "Indomitable Fighter stays on the field"
    );
    assert_eq!(
        st.player(opp).leader_defense,
        19,
        "knockback 1 even when another follower remains"
    );
}

// ----- E24 -----

#[test]
fn fate_of_the_world_picks_draw_draw_then_random_target() {
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10503310");
    st.rng = GameRng::scripted(
        vec![
            Pick {
                what: PickWhat::Draw,
                among: None,
                chose: PickChose::Id("88001110".into()),
            },
            Pick {
                what: PickWhat::Draw,
                among: None,
                chose: PickChose::Id("88001110".into()),
            },
            Pick {
                what: PickWhat::RandomTarget,
                among: None,
                chose: PickChose::Slot { slot: 0 },
            },
        ],
        2,
    );
    apply(&db, &mut st, Action::Play { hand: h }).expect("Fate printed form");
    let whats: Vec<PickWhat> = st.picks.iter().map(|p| p.what).collect();
    assert_eq!(
        whats,
        vec![PickWhat::Draw, PickWhat::Draw, PickWhat::RandomTarget],
        "targets for destroy are selected after the draws; picks={:?}",
        st.picks
    );
    assert_eq!(
        field_count(&st, opp),
        2,
        "one highest-attack body destroyed"
    );
}

#[test]
fn random_damage_then_draw_picks_random_target_then_draw() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let body = put_field(&db, &mut st, opp, "88001320");
    if let Some(f) = st.field_inst_mut(opp, body) {
        f.defense = 4;
        f.max_defense = 4;
    }
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001720");
    st.rng = GameRng::scripted(
        vec![
            Pick {
                what: PickWhat::RandomTarget,
                among: None,
                chose: PickChose::Slot { slot: 0 },
            },
            Pick {
                what: PickWhat::Draw,
                among: None,
                chose: PickChose::Id("88001110".into()),
            },
        ],
        3,
    );
    apply(&db, &mut st, Action::Play { hand: h }).expect("damage then draw");
    let whats: Vec<PickWhat> = st.picks.iter().map(|p| p.what).collect();
    assert_eq!(
        whats,
        vec![PickWhat::RandomTarget, PickWhat::Draw],
        "picks={:?}",
        st.picks
    );
    let left = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001320")
        .expect("tank survives 2");
    assert_eq!(left.defense, 2);
}

// ----- E25 -----

fn sloth_overflow_events(leader_def: i32, follower_def: i32) -> (arena_engine::State, Vec<Event>) {
    let db = load_db();
    let mut st = started(&db, 4);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let z = put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, z) {
        f.defense = follower_def;
        f.max_defense = follower_def;
    }
    st.player_mut(opp).leader_defense = leader_def;
    give_pp(&mut st, me, 2, 7);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10543310");
    st.rng = GameRng::scripted(
        vec![
            Pick {
                what: PickWhat::RandomTarget,
                among: None,
                chose: PickChose::Slot { slot: 0 },
            },
            Pick {
                what: PickWhat::RandomTarget,
                among: None,
                chose: PickChose::Slot { slot: 0 },
            },
        ],
        4,
    );
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("Sloth overflow");
    (st, events)
}

fn damage_targets(events: &[Event]) -> Vec<EventTarget> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Damage { target, .. } => Some(*target),
            _ => None,
        })
        .collect()
}

#[test]
fn sloth_overflow_hits_follower_twice_then_leader() {
    let (st, events) = sloth_overflow_events(20, 12);
    let hits = damage_targets(&events);
    assert_eq!(
        hits,
        vec![
            EventTarget::Slot(PlayerId::B, Slot(0)),
            EventTarget::Slot(PlayerId::B, Slot(0)),
            EventTarget::Leader(PlayerId::B),
        ],
        "repeat body finishes before the overflow if; hits={hits:?}"
    );
    let body = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .next()
        .expect("follower survives 4");
    assert_eq!(body.defense, 8);
    assert_eq!(st.player(PlayerId::B).leader_defense, 18);
    assert!(st.winner.is_none());
}

#[test]
fn sloth_overflow_lethal_still_hits_follower_first() {
    let (st, events) = sloth_overflow_events(1, 12);
    let hits = damage_targets(&events);
    assert_eq!(
        hits,
        vec![
            EventTarget::Slot(PlayerId::B, Slot(0)),
            EventTarget::Slot(PlayerId::B, Slot(0)),
            EventTarget::Leader(PlayerId::B),
        ],
        "follower takes both hits before the leader dies; hits={hits:?}"
    );
    let body = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .next()
        .expect("follower still present at 8");
    assert_eq!(body.defense, 8);
    assert_eq!(st.player(PlayerId::B).leader_defense, 0);
    assert_eq!(st.winner, Some(PlayerId::A));
    assert!(matches!(st.phase, Phase::Terminal));
}

// ----- E26 -----

#[test]
fn terminal_replay_compares_winner_only() {
    let same = serde_json::json!({
        "phase": "terminal",
        "winner": "a",
        "turn": 6,
        "players": { "a": { "leader_defense": 12 }, "b": { "hand": [] } }
    });
    let postmortem = serde_json::json!({
        "phase": "terminal",
        "winner": "a",
        "turn": 7,
        "players": { "a": { "leader_defense": 11 }, "b": { "hand": ["88001110"] } }
    });
    assert_eq!(replay_state_diff(&same, &postmortem), None);
    assert!(
        !replay_compare_legal(&same, &postmortem),
        "legal is skipped at terminal"
    );

    let other_winner = serde_json::json!({
        "phase": "terminal",
        "winner": "b",
        "turn": 6
    });
    let diff = replay_state_diff(&same, &other_winner).expect("winner mismatch");
    assert_eq!(diff.0, "winner");

    let mid = serde_json::json!({
        "phase": "main",
        "winner": null,
        "turn": 6,
        "players": { "a": { "leader_defense": 12 } }
    });
    let still_main = serde_json::json!({
        "phase": "main",
        "winner": null,
        "turn": 6,
        "players": { "a": { "leader_defense": 11 } }
    });
    let mid_diff = replay_state_diff(&mid, &still_main).expect("non-terminal full compare");
    assert!(mid_diff.0.contains("leader_defense"), "{mid_diff:?}");
}

#[test]
fn snapshot_json_terminal_ignores_postmortem_fields() {
    let db = load_db();
    let mut st = started(&db, 5);
    st.player_mut(PlayerId::B).leader_defense = 0;
    st.winner = Some(PlayerId::A);
    st.phase = Phase::Terminal;
    let got = snapshot_json(&st);
    let mut want = got.clone();
    want["turn"] = serde_json::json!(99);
    want["players"]["a"]["pp"] = serde_json::json!(0);
    want["players"]["b"]["hand"] = serde_json::json!([]);
    assert_eq!(replay_state_diff(&got, &want), None);
}
