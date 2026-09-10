//! Pinned 2026-09-10 trace conventions (`docs/trace-format.md`).

use arena_engine::trace::ChooseOptionJson;
use arena_engine::{
    apply, from_neutral, legal_actions, legal_actions_neutral, legal_divergence_parts, snapshot,
    to_neutral, Action, GameRng, Illegal, NeutralAction, Phase, Pick, PickChose, PickWhat,
    PlayerId, ReplayError, TraceHeader,
};

mod common;
use common::*;

#[test]
fn turn_is_round_number_zero_in_mulligan() {
    let db = load_db();
    let st = new_pre_mulligan(&db, 3);
    assert_eq!(st.turn, 0);
    assert!(matches!(st.phase, Phase::Mulligan { .. }));
    let mut st = started(&db, 3);
    assert_eq!(st.turn, 1);
    end_turn(&db, &mut st);
    assert_eq!(st.turn, 1, "second player's first turn is still turn 1");
    end_turn(&db, &mut st);
    assert_eq!(st.turn, 2);
}

#[test]
fn pp_zero_until_first_turn() {
    let db = load_db();
    let st = new_pre_mulligan(&db, 3);
    for p in [PlayerId::A, PlayerId::B] {
        let pl = st.player(p);
        assert_eq!(pl.pp, 0);
        assert_eq!(pl.pp_max, 0);
        assert!(!pl.bonus_pp.active);
    }
    let st = started(&db, 3);
    assert_eq!(st.player(PlayerId::A).pp_max, 1);
    assert_eq!(st.player(PlayerId::B).pp, 0);
    assert_eq!(st.player(PlayerId::B).pp_max, 0);
}

#[test]
fn mulligan_legal_is_sixteen_masks() {
    let db = load_db();
    let st = new_pre_mulligan(&db, 3);
    let legal = legal_actions(&db, &st);
    assert_eq!(legal.len(), 16);
    assert!(legal
        .iter()
        .all(|a| matches!(a, Action::MulliganConfirm { .. })));
    let n = to_neutral(&st, &legal[0]);
    match n {
        NeutralAction::Mulligan { player, .. } => assert_eq!(player, st.first.as_str()),
        other => panic!("{other:?}"),
    }
}

#[test]
fn choose_actor_is_the_player_who_owns_the_choice() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001300");
    play(&db, &mut st, h);
    assert!(matches!(st.phase, Phase::Choice { player, .. } if player == me));
    let legal = legal_actions(&db, &st);
    let choose = legal
        .iter()
        .find(|a| matches!(a, Action::Choose(_)))
        .expect("choose");
    match to_neutral(&st, choose) {
        NeutralAction::Choose { player, .. } => assert_eq!(player, "a"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn granted_is_instance_tags_minus_printed() {
    let db = load_db();
    let mut st = started(&db, 5);
    let me = PlayerId::A;
    let vanilla = put_field(&db, &mut st, me, "88001110");
    let printed_lw = put_field(&db, &mut st, me, "88001200");
    let grant = arena_engine::card::Ability::LastWords {
        printed: "granted".into(),
        effects: vec![],
        zone: None,
        once_per_turn: None,
        when: None,
        replaces: None,
    };
    if let Some(c) = st.player_mut(me).field[vanilla as usize].as_mut() {
        c.granted.push(grant.clone());
    }
    if let Some(c) = st.player_mut(me).field[printed_lw as usize].as_mut() {
        c.granted.push(grant);
    }
    let snap = snapshot(&st);
    let vanilla_g = snap.players.a.field[vanilla as usize]
        .as_ref()
        .and_then(|f| f.granted.clone());
    assert_eq!(
        vanilla_g.as_deref(),
        Some(["lastWords".to_string()].as_slice())
    );
    let printed_g = snap.players.a.field[printed_lw as usize]
        .as_ref()
        .and_then(|f| f.granted.clone());
    assert_eq!(printed_g, None, "grant of a printed tag is invisible");
}

#[test]
fn search_from_deck_emits_multiset_pick() {
    let db = load_db();
    let mut st = started(&db, 8);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    // deck already has pad 88001110 followers
    let h = put_hand(&db, &mut st, me, "88001610");
    play(&db, &mut st, h);
    assert!(
        st.picks.iter().any(|p| {
            p.what == PickWhat::MultisetPick
                && p.among.as_deref() == Some("deck")
                && matches!(p.chose, PickChose::Id(_))
        }),
        "search leaves the deck as multiset_pick among deck; picks={:?}",
        st.picks
    );
    assert!(
        st.player(me)
            .hand
            .iter()
            .any(|c| c.card.as_str() == "88001110"),
        "searched follower entered hand"
    );
}

#[test]
fn scripted_multiset_pick_not_candidate_is_oracle_error() {
    let db = load_db();
    let mut st = started(&db, 8);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001610");
    st.rng = GameRng::scripted(
        vec![Pick {
            what: PickWhat::MultisetPick,
            among: Some("deck".into()),
            chose: PickChose::Id("99999999".into()),
        }],
        8,
    );
    let err = apply(&db, &mut st, Action::Play { hand: h }).unwrap_err();
    match err {
        Illegal::OraclePickNotLegal(o) => {
            assert_eq!(o.what, PickWhat::MultisetPick);
            assert_eq!(o.chose, "99999999");
            assert!(!o.candidates.is_empty());
        }
        other => panic!("{other}"),
    }
}

/// Old-engine emitter bug: `{"play":{"card":"uid_24","hand_pos":0}}` must
/// not play whatever sits at position 0 — `from_neutral` is `None` / NotLegal.
#[test]
fn play_mismatched_card_is_not_legal() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001110");
    let actual = st.player(me).hand[h as usize].card.as_str();
    assert_eq!(actual, "88001110");
    let neu = NeutralAction::Play {
        player: "a".into(),
        hand_pos: h,
        card: "uid_24".into(),
    };
    assert!(
        from_neutral(&st, &neu).is_none(),
        "mismatched play.card must not map to Play {{ hand }}"
    );
    let ok = NeutralAction::Play {
        player: "a".into(),
        hand_pos: h,
        card: actual,
    };
    assert_eq!(from_neutral(&st, &ok), Some(Action::Play { hand: h }));
}

#[test]
fn choose_card_not_in_offered_set_is_not_legal() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001300");
    play(&db, &mut st, h);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    let neu = NeutralAction::Choose {
        player: "a".into(),
        option: arena_engine::trace::ChooseOptionJson::Card {
            card: "uid_24".into(),
        },
    };
    assert!(
        from_neutral(&st, &neu).is_none(),
        "choose.option.card must be in the offered set"
    );
}

#[test]
fn header_x_keys_are_ignored() {
    let raw = r#"{
        "v": 1,
        "engine": "practice-tool",
        "seed": 1,
        "first": "a",
        "deck_a": [],
        "deck_b": [],
        "opening_hands": { "a": [], "b": [] },
        "x_final_hash": "deadbeef"
    }"#;
    let h: TraceHeader = serde_json::from_str(raw).expect("x_ keys ignored");
    assert_eq!(h.seed, 1);
}

#[test]
fn legal_divergence_prints_symmetric_difference() {
    let arena = vec![NeutralAction::BonusPp { player: "b".into() }];
    let trace = vec![NeutralAction::EndTurn { player: "b".into() }];
    let (a, t) = legal_divergence_parts(&arena, &trace);
    assert!(a.starts_with("1 actions; only arena: "), "{a}");
    assert!(a.contains("\"bonus_pp\""), "{a}");
    assert!(t.starts_with("1 actions; only trace: "), "{t}");
    assert!(t.contains("\"end_turn\""), "{t}");
}

/// Sincerity: pool spans both boards. Unique slot numbers still carry `player`
/// (Practice-Tool PR #392 / b3bd5473), not only colliding slot numbers.
#[test]
fn choose_slot_carries_player_when_pool_spans_both_boards() {
    let db = load_db();
    let mut st = started(&db, 392);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10573310");
    let slots: Vec<_> = legal_actions_neutral(&db, &st)
        .into_iter()
        .filter_map(|a| match a {
            NeutralAction::Choose {
                option: ChooseOptionJson::Slot { slot, player },
                ..
            } => Some((slot, player)),
            _ => None,
        })
        .collect();
    assert!(
        slots.iter().all(|(_, p)| p.is_some()),
        "every both-board option names the board: {slots:?}"
    );
    assert!(
        slots
            .iter()
            .any(|(s, p)| *s == 2 && p.as_deref() == Some("a")),
        "unique allied slot 2 still carries player: {slots:?}"
    );
    assert!(
        slots
            .iter()
            .any(|(s, p)| *s == 0 && p.as_deref() == Some("b")),
        "enemy slot 0 carries player: {slots:?}"
    );
}

/// Asher Fanfare: enemy-only Ward grant. `{slot}` omits `player`.
#[test]
fn choose_slot_omits_player_when_pool_is_one_board() {
    let db = load_db();
    let mut st = started(&db, 393);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10874110");
    let slots: Vec<_> = legal_actions_neutral(&db, &st)
        .into_iter()
        .filter_map(|a| match a {
            NeutralAction::Choose {
                option: ChooseOptionJson::Slot { slot, player },
                ..
            } => Some((slot, player)),
            _ => None,
        })
        .collect();
    assert_eq!(
        slots,
        vec![(0, None), (1, None)],
        "one-board pool is {{slot}} only"
    );
}

#[test]
fn replay_illegal_prints_action_and_legal() {
    let err = ReplayError::Illegal {
        i: 4,
        action: r#"{"bonus_pp":{"player":"b"}}"#.into(),
        legal: r#"[{"end_turn":{"player":"b"}}]"#.into(),
        source: Illegal::NotLegal,
    };
    let s = err.to_string();
    assert!(s.contains("i=4"), "{s}");
    assert!(s.contains("bonus_pp"), "{s}");
    assert!(s.contains("end_turn"), "{s}");
}

fn new_pre_mulligan(db: &arena_engine::CardDb, seed: u64) -> arena_engine::State {
    arena_engine::new_game(
        db,
        arena_engine::GameConfig {
            seed,
            deck_a: pad_deck(&["88001110"], 40),
            deck_b: pad_deck(&["88001110"], 40),
            first: arena_engine::First::A,
            opening_hands: None,
        },
    )
    .expect("new_game")
}
