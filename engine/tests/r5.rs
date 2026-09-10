//! R5 differential: multi-target capture, delayed death settle, highest/lowest,
//! sequential choose, discarded trigger, pp cap, evolves_used, snapshot clamps.

use arena_engine::trace::ChooseOptionJson;
use arena_engine::{
    apply, apply_neutral, from_neutral, snapshot_json, Action, GameRng, Illegal, NeutralAction,
    Phase, Pick, PickChose, PickWhat, PlayerId,
};

mod common;
use common::*;

// ----- E14 -----

#[test]
fn all_followers_damage_hits_every_body_on_a_mixed_board() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10603210");
    put_field(&db, &mut st, opp, "88001110");
    let mid = put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, mid) {
        f.attack = 4;
        f.defense = 3;
        f.max_defense = 3;
    }
    end_turn(&db, &mut st);
    let bodies: Vec<(i32, i32)> = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .map(|c| (c.attack, c.defense))
        .collect();
    assert_eq!(
        bodies,
        vec![(4, 1)],
        "both 2/1s die; the middle 4/3 takes 2: {bodies:?}"
    );
}

#[test]
fn burnite_fanfare_damages_every_enemy_follower() {
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let a = put_field(&db, &mut st, opp, "88001110");
    let b = put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, a) {
        f.attack = 8;
        f.defense = 7;
        f.max_defense = 7;
    }
    if let Some(f) = st.field_inst_mut(opp, b) {
        f.attack = 9;
        f.defense = 9;
        f.max_defense = 9;
    }
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10744110");
    assert_eq!(field_count(&st, opp), 0, "9 damage kills 8/7 and 9/9");
}

#[test]
fn banish_all_other_followers_removes_every_other_body() {
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "10444120");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10804110");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert_eq!(field_count(&st, me), 1);
    assert!(field_has(&st, me, "10804110"));
    assert_eq!(field_count(&st, PlayerId::B), 0);
}

// ----- E15 -----

fn sloth_scripted(picks: Vec<Pick>) -> Result<arena_engine::State, Illegal> {
    let db = load_db();
    let mut st = started(&db, 4);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let z = put_field(&db, &mut st, opp, "88001110");
    let s = put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, z) {
        f.attack = 5;
        f.defense = 2;
        f.max_defense = 2;
    }
    if let Some(f) = st.field_inst_mut(opp, s) {
        f.attack = 7;
        f.defense = 2;
        f.max_defense = 2;
    }
    give_pp(&mut st, me, 2, 6);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10543310");
    st.rng = GameRng::scripted(picks, 4);
    apply(&db, &mut st, Action::Play { hand: h })?;
    Ok(st)
}

#[test]
fn sloth_scripted_slot0_then_slot0_replays() {
    // E36: after the first 2 damage, that follower is at 0 defense and is
    // skipped; the remaining body is survivor index 0 (not raw slot 1).
    let st = sloth_scripted(vec![
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
    ])
    .expect("slot0 then slot0");
    assert_eq!(field_count(&st, PlayerId::B), 0);
}

#[test]
fn sloth_scripted_slot0_then_slot1_replays_as_raw_alias() {
    // M1 traces numbered random_target by raw slot. After the first body
    // dies, survivor index 0 is raw slot 1; both labels map to that body.
    let st = sloth_scripted(vec![
        Pick {
            what: PickWhat::RandomTarget,
            among: None,
            chose: PickChose::Slot { slot: 0 },
        },
        Pick {
            what: PickWhat::RandomTarget,
            among: None,
            chose: PickChose::Slot { slot: 1 },
        },
    ])
    .expect("slot0 then raw slot1 alias");
    assert_eq!(field_count(&st, PlayerId::B), 0);
}

#[test]
fn sloth_scripted_slot0_then_slot2_is_oracle_failure() {
    let err = sloth_scripted(vec![
        Pick {
            what: PickWhat::RandomTarget,
            among: None,
            chose: PickChose::Slot { slot: 0 },
        },
        Pick {
            what: PickWhat::RandomTarget,
            among: None,
            chose: PickChose::Slot { slot: 2 },
        },
    ])
    .unwrap_err();
    match err {
        Illegal::OraclePickNotLegal(o) => {
            assert_eq!(o.what, PickWhat::RandomTarget);
            assert_eq!(o.chose, "slot:2");
            assert_eq!(o.candidates, vec!["slot:0".to_string()]);
        }
        other => panic!("{other}"),
    }
}

// ----- E16 -----

#[test]
fn pick_highest_attack_destroys_the_maximum() {
    let db = load_db();
    let mut st = started(&db, 5);
    let opp = PlayerId::B;
    let v = put_field(&db, &mut st, opp, "10644120");
    let d = put_field(&db, &mut st, opp, "10741110");
    let z = put_field(&db, &mut st, opp, "10444120");
    if let Some(f) = st.field_inst_mut(opp, v) {
        f.attack = 0;
        f.defense = 2;
        f.max_defense = 2;
    }
    if let Some(f) = st.field_inst_mut(opp, d) {
        f.attack = 4;
        f.defense = 3;
        f.max_defense = 3;
    }
    if let Some(f) = st.field_inst_mut(opp, z) {
        f.attack = 5;
        f.defense = 5;
        f.max_defense = 5;
    }
    give_pp(&mut st, PlayerId::A, 1, 1);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "88001710");
    assert!(
        !field_has(&st, opp, "10444120"),
        "Zooey 5-attack is highest"
    );
    assert!(field_has(&st, opp, "10644120"));
    assert!(field_has(&st, opp, "10741110"));
}

#[test]
fn pick_highest_tie_replays_either_recorded_slot() {
    for slot in [0u8, 1] {
        let db = load_db();
        let mut st = started(&db, 6);
        let opp = PlayerId::B;
        let mut ids = [0u32; 2];
        for id in &mut ids {
            let slot_i = put_field(&db, &mut st, opp, "88001110");
            if let Some(f) = st.field_inst_mut(opp, slot_i) {
                f.attack = 5;
                *id = f.id;
            }
        }
        give_pp(&mut st, PlayerId::A, 1, 1);
        st.player_mut(PlayerId::A).hand.clear();
        let h = put_hand(&db, &mut st, PlayerId::A, "88001710");
        st.rng = GameRng::scripted(
            vec![Pick {
                what: PickWhat::RandomTarget,
                among: None,
                chose: PickChose::Slot { slot },
            }],
            6,
        );
        apply(&db, &mut st, Action::Play { hand: h }).unwrap();
        assert_eq!(field_count(&st, opp), 1);
        let gone = ids[slot as usize];
        let stayed = ids[1 - slot as usize];
        assert!(st.find_field(opp, stayed).is_some());
        assert!(st.find_field(opp, gone).is_none());
    }
}

// ----- E17 -----

#[test]
fn choose_count_two_takes_two_picks_then_resolves() {
    let db = load_db();
    let mut st = started(&db, 7);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001320");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "88001120");
    put_hand(&db, &mut st, me, "10844120");
    play(&db, &mut st, 2);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "still choosing the second discard"
    );
    choose(&db, &mut st, 0);
    assert!(matches!(st.phase, Phase::Main));
    assert_eq!(st.player(me).hand.len(), 0, "both discarded");
    assert_eq!(st.player(PlayerId::B).leader_defense, 16);
}

#[test]
fn choose_count_two_with_one_card_completes_after_one_pick() {
    let db = load_db();
    let mut st = started(&db, 8);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "10844120");
    play(&db, &mut st, 1);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(
        matches!(st.phase, Phase::Main),
        "one remaining card clamps the select"
    );
    assert_eq!(st.player(PlayerId::B).leader_defense, 16);
}

// ----- E18 -----

#[test]
fn choose_card_takes_the_lowest_hand_position() {
    let db = load_db();
    let mut st = started(&db, 9);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "88001120");
    put_hand(&db, &mut st, me, "88001110");
    put_hand(&db, &mut st, me, "10642310");
    play(&db, &mut st, 3);
    let neu = NeutralAction::Choose {
        player: "a".into(),
        option: ChooseOptionJson::Card {
            card: "88001110".into(),
        },
    };
    let mapped = from_neutral(&st, &neu).expect("lowest copy");
    assert_eq!(mapped, Action::Choose(0));
    apply_neutral(&db, &mut st, &neu).unwrap();
    let ids: Vec<String> = st.player(me).hand.iter().map(|c| c.card.as_str()).collect();
    assert_eq!(ids, vec!["88001120".to_string(), "88001110".to_string()]);
}

// ----- E19 -----

#[test]
fn discarded_vorlalai_summons_after_the_spell_resolves() {
    let db = load_db();
    let mut st = started(&db, 10);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001320");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10644120");
    put_hand(&db, &mut st, me, "10642310");
    play(&db, &mut st, 1);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "destroy select is pending"
    );
    assert!(
        !field_has(&st, me, "10644120"),
        "on:discarded waits until the spell finishes"
    );
    choose(&db, &mut st, 0);
    assert!(
        field_has(&st, me, "10644120"),
        "on:discarded summons after the spell resolves"
    );
    assert_eq!(st.player(me).rally, 1);
}

// ----- E20 -----

#[test]
fn gain_max_pp_at_ten_stays_ten() {
    let db = load_db();
    let mut st = started(&db, 11);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10444120");
    assert_eq!(st.player(me).pp_max, 10);
}

// ----- E21 -----

#[test]
fn effect_evolve_counts_in_evolves_used() {
    let db = load_db();
    let mut st = started(&db, 12);
    skip_to_player_turn(&db, &mut st, PlayerId::A, 5);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).ep = 2;
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001110");
    play_id(&db, &mut st, me, "88001110");
    play_id(&db, &mut st, me, "10011130");
    assert!(st
        .player(me)
        .field
        .iter()
        .flatten()
        .any(|c| c.card.as_str() == "10011130" && c.evolved));
    assert_eq!(st.player(me).evolves_used, 1);
}

// ----- E22 -----

#[test]
fn leader_defense_is_clamped_at_zero() {
    let db = load_db();
    let mut st = started(&db, 13);
    st.player_mut(PlayerId::B).leader_defense = 1;
    give_pp(&mut st, PlayerId::A, 10, 10);
    st.player_mut(PlayerId::A).hand.clear();
    play_id(&db, &mut st, PlayerId::A, "88001310");
    assert_eq!(st.player(PlayerId::B).leader_defense, 0);
    let snap = snapshot_json(&st);
    assert_eq!(snap["players"]["b"]["leader_defense"], 0);
}

#[test]
fn crest_without_countdown_omits_the_key() {
    let db = load_db();
    let mut st = started(&db, 14);
    st.player_mut(PlayerId::A)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: "crest:10744110".into(),
            countdown: None,
            faith: false,
            once_used: vec![],
            granted_order: 1,
            granted: vec![],
            choose_used: Default::default(),
        });
    let snap = snapshot_json(&st);
    let crest = &snap["players"]["a"]["crests"][0];
    assert!(crest.get("countdown").is_none(), "{crest}");
    assert_eq!(crest["id"], "crest:10744110");
}

#[test]
fn field_slot_keeps_explicit_null_countdown() {
    let db = load_db();
    let mut st = started(&db, 15);
    put_field(&db, &mut st, PlayerId::A, "88001110");
    let snap = snapshot_json(&st);
    assert_eq!(
        snap["players"]["a"]["field"][0]["countdown"],
        serde_json::Value::Null
    );
}
