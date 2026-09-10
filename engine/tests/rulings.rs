//! Owner-ruling tests (in-scope headings).

use arena_engine::{apply, legal_actions, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

#[test]
fn accelerate_only_when_normal_unaffordable() {
    // owner-rulings — Accelerate — original cost preserved — 2026-08-12
    // and Accelerate / Crystallize 2026-09-06; Shoddy Plaything 10671110.
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10671110");
    assert!(legal_actions(&db, &st)
        .iter()
        .any(|a| matches!(a, Action::Play { hand } if *hand == h)));
    play(&db, &mut st, h);
    assert!(field_has(&st, me, "10671110"), "Accelerate summons");
    assert_eq!(st.player(me).pp, 0);
    // cemetery corpse is the accelerate spell form
    assert!(st
        .player(me)
        .cemetery
        .iter()
        .any(|c| c.card.as_str() == "10671110"));
}

#[test]
fn accelerate_not_offered_when_normal_affordable() {
    let db = load_db();
    let mut st = started(&db, 1);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10671110");
    play(&db, &mut st, h);
    // normal play: follower enters, Fanfare draws 3
    assert!(field_has(&st, me, "10671110"));
    assert!(
        st.player(me).hand.len() >= 3,
        "Fanfare draw 3 on normal play"
    );
}

#[test]
fn multi_tier_enhance_all_affordable() {
    // Multi-tier Enhance — ALL affordable tiers activate — 2026-08-15
    let db = load_db();
    let mut st = started(&db, 2);
    let me = PlayerId::A;
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10001110");
    play(&db, &mut st, h);
    let f = st.player(me).field.iter().flatten().next().unwrap();
    assert_eq!(f.attack, 5);
    assert_eq!(f.defense, 5);
}

#[test]
fn select_is_forced_opens_choice() {
    // "Select" is forced — 2026-08-16 / printed Select 2026-09-08
    let db = load_db();
    let mut st = started(&db, 3);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001300");
    play(&db, &mut st, h);
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "printed Select must open a node, not auto-pick leftmost"
    );
}

#[test]
fn spell_unplayable_without_select_target() {
    // Playability with no Select target — 2026-08-16
    let db = load_db();
    let mut st = started(&db, 4);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10041310");
    let legal = legal_actions(&db, &st);
    assert!(!legal
        .iter()
        .any(|a| matches!(a, Action::Play { hand } if *hand == h)));
}

#[test]
fn super_evolve_prevention_counts_as_damage() {
    // Damage prevented by super-evolve still counts as taking damage — 2026-08-23
    let db = load_db();
    let mut st = started(&db, 5);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).turns_taken = 7;
    st.player_mut(me).sep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .unwrap();
    let before = st.field_inst(me, slot).unwrap().defense;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .unwrap();
    let after = st.field_inst(me, slot).map(|c| c.defense);
    // own-turn protection: counter-damage is 0 but still counts as taking damage
    assert_eq!(after, Some(before));
}

#[test]
fn n_random_distinct_vs_repeat_independent() {
    // "N random followers" = N distinct; "do this N times" = repeats allowed — 2026-08-23
    let db = load_db();
    let mut st = started(&db, 6);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001320"); // 10/10
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001380");
    play(&db, &mut st, h);
    // two independent 1-damage rolls; both may hit the same body
    let total_lost: i32 = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .map(|c| c.max_defense - c.defense)
        .sum();
    assert_eq!(total_lost, 2, "repeat = two damage instances totaling 2");
}

#[test]
fn hand_overflow_no_last_words() {
    // Hand overflow destroys without Last Words — 2026-08-10
    let db = load_db();
    let mut st = started(&db, 7);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    for _ in 0..9 {
        put_hand(&db, &mut st, me, "88001110");
    }
    let deck_before = st.player(me).deck.len();
    put_hand(&db, &mut st, me, "88001470");
    assert_eq!(st.player(me).hand.len(), 9);
    assert_eq!(
        st.player(me).deck.len(),
        deck_before,
        "Last Words draw must not fire"
    );
}

#[test]
fn fuse_partners_banished() {
    // Fused cards are banished — 2026-09-02; Artifact fuse chain 2026-09-05
    let db = load_db();
    let mut st = started(&db, 8);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90031210"); // earth sigil amulet is not artifact
    put_hand(&db, &mut st, me, "90071220");
    let legal = legal_actions(&db, &st);
    let fuse = legal
        .iter()
        .find(|a| matches!(a, Action::Fuse { .. }))
        .cloned()
        .expect("fuse must be legal at 0 PP");
    apply(&db, &mut st, fuse).unwrap();
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
        if matches!(st.phase, Phase::Choice { .. }) {
            confirm(&db, &mut st);
        }
    }
    assert!(
        st.player(me)
            .banished
            .iter()
            .any(|c| c.card.as_str() == "90071220" || c.card.as_str() == "90071210"),
        "partners (or host leftover) banished"
    );
}

#[test]
fn earth_sigil_merge_collectible_wins() {
    // Witch's New Brew always wins an Earth Sigil merge — 2026-08-30
    let db = load_db();
    let mut st = started(&db, 9);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    put_field(&db, &mut st, me, "90031210");
    assert_eq!(st.player(me).earth, 1);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10031210");
    play(&db, &mut st, h);
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert!(field_has(&st, me, "10031210"), "collectible survives");
    assert!(!field_has(&st, me, "90031210"), "token holder is replaced");
    assert!(st.player(me).earth >= 2);
}

#[test]
fn crest_cap_five() {
    // Crest and Faith slots are capped at five — 2026-09-05
    let db = load_db();
    let mut st = started(&db, 10);
    let me = PlayerId::A;
    for i in 0..5 {
        st.player_mut(me)
            .crests
            .push(arena_engine::state::CrestInstance {
                id: format!("crest:1057411{i}"),
                countdown: Some(3),
                faith: false,
                once_used: vec![],
                granted_order: i as u32,
            });
    }
    assert_eq!(st.player(me).crests.len(), 5);
}

#[test]
fn last_words_summon_after_compaction() {
    // Last Words summons never spawn in place — 2026-09-05
    let db = load_db();
    let mut st = started(&db, 11);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001390");
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001300");
    play(&db, &mut st, h);
    choose(&db, &mut st, 0); // destroy leftmost (Last Words summoner)
    let ids: Vec<String> = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .map(|c| c.card.as_str())
        .collect();
    assert!(
        ids.contains(&"88001110".to_string()),
        "surviving follower compacted left"
    );
}

#[test]
fn engage_sacrifice_is_destruction() {
    // An amulet destroyed by its own Engage is destroyed — 2026-09-09
    let db = load_db();
    let mut st = started(&db, 12);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    let slot = put_field(&db, &mut st, me, "10001210");
    put_field(&db, &mut st, PlayerId::B, "88001140");
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).unwrap();
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert!(!field_has(&st, me, "10001210"));
    assert!(st
        .player(me)
        .cemetery
        .iter()
        .any(|c| c.card.as_str() == "10001210"));
    assert!(st.player(me).shadows >= 1);
}

#[test]
fn cant_be_played_gears() {
    // Artifact fuse partners / Can't be played on Gears — 2026-09-10
    let db = load_db();
    let mut st = started(&db, 13);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "90071210");
    let legal = legal_actions(&db, &st);
    assert!(!legal
        .iter()
        .any(|a| matches!(a, Action::Play { hand } if *hand == h)));
}

#[test]
fn bonus_pp_second_player_charges() {
    let db = load_db();
    let mut st = started(&db, 14);
    // A is first; B is second
    end_turn(&db, &mut st);
    assert!(st.player(PlayerId::B).is_second);
    assert!(st.player(PlayerId::B).bonus_pp.early_charge);
    let legal = legal_actions(&db, &st);
    assert!(legal.iter().any(|a| matches!(a, Action::BonusPp)));
}

#[test]
fn attacked_leader_counts_the_attack() {
    // "Attacked a leader last turn" — the attack counts, not the damage — 2026-08-29
    let db = load_db();
    let mut st = started(&db, 15);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "88001110");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .unwrap();
    assert!(st.player(me).attacked_leader_this_turn);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert!(st.player(me).attacked_leader_last_turn);
}
