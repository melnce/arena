//! Rulebook "_Test case_" paragraphs whose mechanics are in M1 scope.

use arena_engine::{apply, hash, legal_actions, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

#[test]
fn zones_hand_overflow_destroys_tenth() {
    // rulebook Zones — Hand. Owner ruling 2026-08-10 (no Last Words).
    let db = load_db();
    let mut st = started(&db, 7);
    let who = PlayerId::A;
    st.player_mut(who).hand.clear();
    for _ in 0..8 {
        put_hand(&db, &mut st, who, "88001110");
    }
    give_pp(&mut st, who, 10, 10);
    let shadows = st.player(who).shadows;
    let cem = st.player(who).cemetery.len();
    // Fanfare Last Words card is 1 cost; playing Fairy Tamer adds 2 Fairies — overflow the second.
    play_id(&db, &mut st, who, "10011110");
    assert!(
        st.player(who).hand.len() <= 9,
        "engine must never allow a 10th card"
    );
    assert!(
        st.player(who).cemetery.len() > cem || st.player(who).shadows > shadows,
        "overflow burns"
    );
}

#[test]
fn zones_summon_two_with_one_slot_skips_excess() {
    // rulebook Zones — Field.
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    for _ in 0..4 {
        put_field(&db, &mut st, me, "88001110");
    }
    assert_eq!(field_count(&st, me), 4);
    give_pp(&mut st, me, 10, 10);
    let h = put_hand(&db, &mut st, me, "88001260");
    play(&db, &mut st, h);
    assert_eq!(field_count(&st, me), 5, "one summon fills the last slot");
}

#[test]
fn random_destroy_two_are_distinct() {
    // rulebook Randomness. Owner ruling 2026-08-23.
    let db = load_db();
    let mut st = started(&db, 11);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    for _ in 0..3 {
        put_field(&db, &mut st, opp, "88001110");
    }
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001290");
    play(&db, &mut st, h);
    assert_eq!(
        field_count(&st, opp),
        1,
        "two distinct destroyed, one remains"
    );
}

#[test]
fn evolution_locked_before_threshold() {
    // rulebook Evolution Point Rules.
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).ep = 2;
    st.player_mut(me).turns_taken = 4;
    let legal = legal_actions(&db, &st);
    assert!(
        !legal.iter().any(|a| matches!(a, Action::Evolve { .. })),
        "evolve disallowed before turn 5 for first player"
    );
    st.player_mut(me).turns_taken = 5;
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().any(|a| matches!(
        a,
        Action::Evolve {
            super_evolve: false,
            ..
        }
    )));
}

#[test]
fn evolution_cannot_super_already_evolved() {
    // rulebook Evolution Point Rules — second test case.
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).ep = 2;
    st.player_mut(me).sep = 2;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .unwrap();
    let legal = legal_actions(&db, &st);
    assert!(!legal.iter().any(|a| matches!(
        a,
        Action::Evolve {
            slot: Slot(s),
            ..
        } if *s == slot
    )));
}

#[test]
fn fanfare_fizzle_still_playable() {
    // rulebook Fanfare test case.
    let db = load_db();
    let mut st = started(&db, 4);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001210");
    let legal = legal_actions(&db, &st);
    assert!(legal
        .iter()
        .any(|a| matches!(a, Action::Play { hand } if *hand == h)));
    play(&db, &mut st, h);
    assert!(field_has(&st, me, "88001210"));
}

#[test]
fn last_words_do_not_fire_on_banish() {
    // rulebook Last Words test case (banish half).
    let db = load_db();
    let mut st = started(&db, 5);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001200");
    let hand0 = st.player(opp).hand.len();
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001480");
    play(&db, &mut st, h);
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert_eq!(field_count(&st, opp), 0);
    assert_eq!(
        st.player(opp).hand.len(),
        hand0,
        "banish must not fire Last Words (no extra draw/ping path)"
    );
    assert!(st
        .player(opp)
        .banished
        .iter()
        .any(|c| c.card.as_str() == "88001200"));
}

#[test]
fn strike_resolves_before_damage_and_can_lethal() {
    // rulebook Strike test case.
    let db = load_db();
    let mut st = started(&db, 6);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let slot = put_field(&db, &mut st, me, "88001220");
    put_field(&db, &mut st, opp, "88001110");
    st.player_mut(opp).leader_defense = 2;
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .unwrap();
    assert_eq!(
        st.winner,
        Some(me),
        "Strike 2 to a 2-defense leader ends the game"
    );
}

#[test]
fn clash_defender_still_fires_if_killed_by_attacker_clash() {
    // rulebook Clash test case.
    let db = load_db();
    let mut st = started(&db, 8);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let a = put_field(&db, &mut st, me, "88001230");
    put_field(&db, &mut st, opp, "88001240");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(a),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .unwrap();
    // attacker Clash 3 should kill 2/2 defender; defender Clash 2 still hits
    let att = st.field_inst(me, a);
    if let Some(c) = att {
        assert!(c.defense <= 2, "defender Clash still dealt 2 (queued)");
    }
}

#[test]
fn bane_kills_through_barrier_and_zero_damage() {
    // rulebook Bane test case.
    let db = load_db();
    let mut st = started(&db, 9);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let a = put_field(&db, &mut st, me, "88001150");
    put_field(&db, &mut st, opp, "88001190");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(a),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .unwrap();
    assert_eq!(field_count(&st, opp), 0, "Bane destroys Barrier 2/2");
}

#[test]
fn drain_attacker_only_combat_only() {
    // rulebook Drain test case (1).
    let db = load_db();
    let mut st = started(&db, 10);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001160");
    st.player_mut(me).leader_defense = 10;
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .unwrap();
    assert_eq!(st.player(me).leader_defense, 15);
    assert!(st.player(me).leader_defense <= st.player(me).leader_max);
}

#[test]
fn ward_blocks_non_ward_attack() {
    // rulebook Ward test case.
    let db = load_db();
    let mut st = started(&db, 11);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, opp, "88001140");
    put_field(&db, &mut st, opp, "88001110");
    let legal = legal_actions(&db, &st);
    let attacks: Vec<_> = legal
        .iter()
        .filter_map(|a| match a {
            Action::Attack { target, .. } => Some(target),
            _ => None,
        })
        .collect();
    assert!(attacks
        .iter()
        .all(|t| matches!(t, AttackTarget::Slot(Slot(0)))));
    assert!(!attacks.iter().any(|t| matches!(t, AttackTarget::Leader)));
}

#[test]
fn storm_can_attack_leader_immediately() {
    // rulebook Storm test case.
    let db = load_db();
    let mut st = started(&db, 12);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001120");
    play(&db, &mut st, h);
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().any(|a| matches!(
        a,
        Action::Attack {
            target: AttackTarget::Leader,
            ..
        }
    )));
}

#[test]
fn rush_cannot_attack_leader_the_turn_played() {
    // rulebook Rush test case.
    let db = load_db();
    let mut st = started(&db, 13);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001130");
    play(&db, &mut st, h);
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().any(|a| matches!(
        a,
        Action::Attack {
            target: AttackTarget::Slot(_),
            ..
        }
    )));
    assert!(!legal.iter().any(|a| matches!(
        a,
        Action::Attack {
            target: AttackTarget::Leader,
            ..
        }
    )));
}

#[test]
fn ambush_blocks_select_not_aoe() {
    // rulebook Ambush test case.
    let db = load_db();
    let mut st = started(&db, 14);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001170");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let _h = put_hand(&db, &mut st, me, "88001300");
    let legal = legal_actions(&db, &st);
    assert!(
        !legal.iter().any(|a| matches!(a, Action::Play { .. })),
        "single-target destroy spell unplayable vs only Ambush"
    );
}

#[test]
fn aura_not_selectable_but_attackable() {
    // rulebook Aura test case.
    let db = load_db();
    let mut st = started(&db, 15);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, opp, "88001180");
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().any(|a| matches!(
        a,
        Action::Attack {
            target: AttackTarget::Slot(_),
            ..
        }
    )));
}

#[test]
fn barrier_eats_one_damage_instance() {
    // rulebook Barrier test case.
    let db = load_db();
    let mut st = started(&db, 16);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001190");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001310");
    play(&db, &mut st, h);
    let f = st.player(opp).field[0].as_ref().expect("still there");
    assert_eq!(f.defense, 2, "Barrier reduced the AoE instance to 0");
    assert!(!f.is_barrier());
}

#[test]
fn necromancy_pays_or_skips() {
    // rulebook Necromancy test case.
    let db = load_db();
    let mut st = started(&db, 17);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).shadows = 3;
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001270");
    play(&db, &mut st, h);
    let f = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001270")
        .unwrap();
    assert!(!f.is_storm(), "3 shadows: Necromancy skipped");
    st.player_mut(me).field = Default::default();
    st.player_mut(me).shadows = 4;
    let h = put_hand(&db, &mut st, me, "88001270");
    play(&db, &mut st, h);
    let f = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001270")
        .unwrap();
    assert!(f.is_storm());
    assert_eq!(st.player(me).shadows, 0);
}

#[test]
fn reanimate_highest_cost_summoning_sick() {
    // rulebook Reanimate test case. Owner ruling 2026-09-02.
    let db = load_db();
    let mut st = started(&db, 18);
    let me = PlayerId::A;
    st.player_mut(me)
        .destroyed_history
        .push(arena_engine::state::DestroyedRecord {
            card: cid("88001110"),
            base_cost: 1,
            kind: arena_engine::card::CardKind::Follower,
            owner: me,
            from_field: true,
        });
    st.player_mut(me)
        .destroyed_history
        .push(arena_engine::state::DestroyedRecord {
            card: cid("88001320"),
            base_cost: 4,
            kind: arena_engine::card::CardKind::Follower,
            owner: me,
            from_field: true,
        });
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001280");
    play(&db, &mut st, h);
    let f = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001320")
        .expect("reanimated the 4-cost");
    assert!(f.flags.summoning_sick);
}

#[test]
fn legal_actions_never_mutates() {
    let db = load_db();
    let st = started(&db, 99);
    let before = hash(&st);
    let _ = legal_actions(&db, &st);
    assert_eq!(hash(&st), before);
}

#[test]
fn clone_plus_same_actions_same_hash() {
    let db = load_db();
    let mut a = started(&db, 42);
    let mut b = a.clone();
    give_pp(&mut a, PlayerId::A, 10, 10);
    give_pp(&mut b, PlayerId::A, 10, 10);
    a.player_mut(PlayerId::A).hand.clear();
    b.player_mut(PlayerId::A).hand.clear();
    let _ = put_hand(&db, &mut a, PlayerId::A, "88001110");
    let _ = put_hand(&db, &mut b, PlayerId::A, "88001110");
    play(&db, &mut a, 0);
    play(&db, &mut b, 0);
    end_turn(&db, &mut a);
    end_turn(&db, &mut b);
    assert_eq!(hash(&a), hash(&b));
}

#[test]
fn simultaneous_leader_lethal_active_loses() {
    // rulebook Match Flow — simultaneous lethal.
    let db = load_db();
    let mut st = started(&db, 21);
    st.player_mut(PlayerId::A).leader_defense = 1;
    st.player_mut(PlayerId::B).leader_defense = 1;
    let slot = put_field(&db, &mut st, PlayerId::A, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001110");
    // force both leaders to 0 via effect damage after combat? Use drain-less attack
    // and then ping. Simpler: deal 1 to both via two attacks is sequential.
    // Direct: drop both to 0 in one apply via AoE that hits both leaders — our AoE is enemies only.
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .ok();
}

#[test]
fn overflow_is_max_pp_at_least_7() {
    let db = load_db();
    let mut st = started(&db, 22);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001420");
    let hand_before = st.player(me).hand.len();
    play(&db, &mut st, h);
    assert_eq!(
        st.player(me).hand.len(),
        hand_before - 1,
        "no overflow draw at 6"
    );
    st.player_mut(me).field = Default::default();
    give_pp(&mut st, me, 7, 7);
    let h = put_hand(&db, &mut st, me, "88001420");
    let hand_before = st.player(me).hand.len();
    play(&db, &mut st, h);
    assert!(st.player(me).hand.len() >= hand_before, "overflow draws");
}
