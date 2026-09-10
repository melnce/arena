//! R7 differential: Last Words wave order, draw-N before next op,
//! countdown destroy by id, fuse partners as `{card}`.

use arena_engine::event::{Event, EventTarget};
use arena_engine::trace::ChooseOptionJson;
use arena_engine::{
    apply, apply_neutral, from_neutral, legal_actions_neutral, replay_state_diff, snapshot_json,
    to_neutral, Action, CardInstance, GameRng, NeutralAction, Phase, Pick, PickChose, PickWhat,
    PlayerId,
};

mod common;
use common::*;

// ----- E27 -----

#[test]
fn simultaneous_last_words_active_side_draws_first() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10001120");
    put_field(&db, &mut st, opp, "10001120");
    let extra = db.card(cid("10032310")).expect("Arcane Eruption");
    let extra_id = st.alloc_id();
    st.player_mut(me)
        .deck
        .push(CardInstance::from_card(extra, extra_id));
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10032310");
    st.rng = GameRng::scripted(
        vec![
            Pick {
                what: PickWhat::Draw,
                among: None,
                chose: PickChose::Id("10032310".into()),
            },
            Pick {
                what: PickWhat::Draw,
                among: None,
                chose: PickChose::Id("88001110".into()),
            },
        ],
        1,
    );
    apply(&db, &mut st, Action::Play { hand: h }).expect("Arcane Eruption");
    assert!(!field_has(&st, me, "10001120"));
    assert!(!field_has(&st, opp, "10001120"));
    assert!(
        st.player(me)
            .hand
            .iter()
            .any(|c| c.card.as_str() == "10032310"),
        "active Leah draws first (10032310 was only in A's deck)"
    );
    assert!(
        st.player(opp)
            .hand
            .iter()
            .any(|c| c.card.as_str() == "88001110"),
        "non-active Leah draws second"
    );
}

// ----- E28 -----

#[test]
fn draw_count_two_then_random_destroy_replays() {
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
    apply(&db, &mut st, Action::Play { hand: h }).expect("draw, draw, random_target");
    assert_eq!(field_count(&st, opp), 2);
}

#[test]
fn ally_draw_reaction_fires_after_both_draws_before_destroy() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "88001730");
    let tank = put_field(&db, &mut st, opp, "88001320");
    if let Some(f) = st.field_inst_mut(opp, tank) {
        f.attack = 5;
        f.defense = 10;
        f.max_defense = 10;
    }
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10503310");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("Fate + ally_draw");
    let mut seq = Vec::new();
    for e in &events {
        match e {
            Event::Draw { player, .. } if *player == me => seq.push("draw"),
            Event::Damage {
                target: EventTarget::Leader(p),
                ..
            } if *p == opp => seq.push("ping"),
            Event::Destroy { .. } => seq.push("destroy"),
            _ => {}
        }
    }
    assert_eq!(
        seq,
        vec!["draw", "draw", "ping", "ping", "destroy"],
        "both draws, then both ally_draw pings, then destroy; {seq:?}"
    );
    assert_eq!(st.player(opp).leader_defense, 18);
    assert_eq!(field_count(&st, opp), 0);
}

// ----- E29 -----

#[test]
fn two_countdown_amulets_expire_without_hitting_the_follower() {
    let db = load_db();
    let mut st = started(&db, 4);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let a0 = put_field(&db, &mut st, opp, "10011210");
    let a1 = put_field(&db, &mut st, opp, "10011210");
    put_field(&db, &mut st, opp, "10011130");
    if let Some(c) = st.field_inst_mut(opp, a0) {
        c.countdown = Some(1);
    }
    if let Some(c) = st.field_inst_mut(opp, a1) {
        c.countdown = Some(1);
    }
    end_turn(&db, &mut st);
    assert_eq!(
        st.player(opp)
            .cemetery
            .iter()
            .filter(|c| c.card.as_str() == "10011210")
            .count(),
        2,
        "both Wild Profusions expire"
    );
    assert!(
        field_has(&st, opp, "10011130"),
        "Gentle Treant to the right is not destroyed"
    );
    assert!(!field_has(&st, opp, "10011210"));
    assert_eq!(st.active, opp);
    let _ = me;
}

// ----- E30 -----

#[test]
fn fuse_partner_legal_is_choose_card() {
    let db = load_db();
    let mut st = started(&db, 5);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90071220");
    apply(&db, &mut st, Action::Fuse { host: 0 }).unwrap();
    assert!(matches!(st.phase, Phase::Choice { .. }));
    let neu = legal_actions_neutral(&db, &st);
    let cards: Vec<_> = neu
        .iter()
        .filter_map(|a| match a {
            NeutralAction::Choose {
                option: ChooseOptionJson::Card { card },
                ..
            } => Some(card.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        cards,
        vec!["90071220"],
        "partner is {{card}}, not {{mode}}: {neu:?}"
    );
    assert!(
        !neu.iter().any(|a| matches!(
            a,
            NeutralAction::Choose {
                option: ChooseOptionJson::Mode { .. },
                ..
            }
        )),
        "no {{mode}} partner options: {neu:?}"
    );
    let mapped = from_neutral(
        &st,
        &NeutralAction::Choose {
            player: "a".into(),
            option: ChooseOptionJson::Card {
                card: "90071220".into(),
            },
        },
    )
    .expect("from_neutral card");
    assert_eq!(mapped, Action::Choose(0));
}

#[test]
fn fuse_game_arena_replays() {
    let db = load_db();
    let mut live = started(&db, 6);
    let me = PlayerId::A;
    live.player_mut(me).hand.clear();
    put_hand(&db, &mut live, me, "90071210");
    put_hand(&db, &mut live, me, "90071220");
    let mut recs = Vec::new();
    for action in [Action::Fuse { host: 0 }, Action::Choose(0), Action::Confirm] {
        let neu = to_neutral(&live, &action);
        apply(&db, &mut live, action).unwrap();
        recs.push((neu, live.picks.clone(), snapshot_json(&live)));
    }
    assert!(live
        .player(me)
        .hand
        .iter()
        .any(|c| c.card.as_str() == "90072110"));

    let mut replay = started(&db, 6);
    replay.player_mut(me).hand.clear();
    put_hand(&db, &mut replay, me, "90071210");
    put_hand(&db, &mut replay, me, "90071220");
    for (i, (neu, picks, snap)) in recs.iter().enumerate() {
        replay.rng = GameRng::scripted(picks.clone(), 6);
        apply_neutral(&db, &mut replay, neu)
            .unwrap_or_else(|e| panic!("replay i={i} {neu:?}: {e}"));
        let got = snapshot_json(&replay);
        if let Some((path, a, b)) = replay_state_diff(&got, snap) {
            panic!("fuse replay i={i} {path}: arena={a} trace={b}");
        }
    }
}
