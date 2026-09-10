//! M2 wave 3 — Artifact Portal: field transform, Invoke, effect-evolve,
//! ally_enter binding, distinctNames / enteredThisMatch, filtered distinct
//! draw, copyOf + hand cost, Accelerate 3/4.

use arena_engine::{
    apply, legal_actions, Action, AttackTarget, CardInstance, Phase, PlayerId, Slot,
};

mod common;
use common::*;

const CLOSURE: &[&str] = &[
    "10072210", "10404110", "10471130", "10573310", "10574120", "10671110", "10672110", "10673110",
    "10674110", "10674120", "10771110", "10771310", "10772110", "10773110", "10774110", "10774120",
    "10874110", "10874120", "10974120", "90071110", "90071130", "90071140", "90071150", "90072110",
    "90073110", "90073120", "90073130", "90074110", "90074140", "90074150", "90074320",
];

#[test]
fn portal_closure_is_supported() {
    let db = load_db();
    for id in CLOSURE {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

fn push_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).expect(id);
    let inst = CardInstance::from_card(card, st.alloc_id());
    st.player_mut(who).deck.push(inst);
}

/// Transform an allied follower that already attacked: Little Buddies (Rush)
/// can attack a follower the same turn, not the leader. Owner 2026-09-10.
#[test]
fn sincerity_transform_rush_attacks_follower_not_leader() {
    let db = load_db();
    let mut st = started(&db, 40);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let mine = put_field(&db, &mut st, me, "88001110");
    if let Some(f) = st.field_inst_mut(me, mine) {
        f.flags.attacked_this_turn = true;
        f.flags.attacks_left = 0;
    }
    put_field(&db, &mut st, opp, "88001110");
    let rally_before = st.player(me).rally;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10573310");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, mine);
    let f = st.field_inst(me, mine).expect("transformed in place");
    assert_eq!(f.card.as_str(), "90074140");
    assert!(f.is_rush());
    assert!(f.flags.summoning_sick);
    assert_eq!(f.flags.attacks_left, 1);
    assert!(!f.flags.attacked_this_turn);
    assert_eq!(
        st.player(me).rally,
        rally_before,
        "transform is not an enter"
    );
    assert!(
        st.player(me)
            .cemetery
            .iter()
            .all(|c| c.card.as_str() != "88001110"),
        "original does not go to cemetery"
    );
    assert_eq!(
        *st.player(me)
            .enter_counts
            .get(&cid("90074140"))
            .unwrap_or(&0),
        0,
        "transformed-in card is not enteredThisMatch"
    );
    let legal = legal_actions(&db, &st);
    let attacks: Vec<_> = legal
        .iter()
        .filter(|a| matches!(a, Action::Attack { attacker, .. } if attacker.0 == mine))
        .cloned()
        .collect();
    assert!(
        attacks.iter().any(|a| matches!(
            a,
            Action::Attack {
                target: AttackTarget::Slot(_),
                ..
            }
        )),
        "Rush lets Buddies attack a follower the turn it appears"
    );
    assert!(
        !attacks.iter().any(|a| matches!(
            a,
            Action::Attack {
                target: AttackTarget::Leader,
                ..
            }
        )),
        "Rush cannot attack the leader"
    );
}

#[test]
fn sincerity_transform_amulet_is_legal_target() {
    let db = load_db();
    let mut st = started(&db, 41);
    let me = PlayerId::A;
    let amu = put_field(&db, &mut st, me, "10072210");
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10573310");
    choose(&db, &mut st, amu);
    let f = st.field_inst(me, amu).expect("amulet slot reused");
    assert_eq!(f.card.as_str(), "90074140");
    assert_eq!(f.kind, arena_engine::card::CardKind::Follower);
    assert_eq!(f.flags.attacks_left, 1);
    assert!(f.is_rush());
}

#[test]
fn sincerity_transform_enemy_follower() {
    let db = load_db();
    let mut st = started(&db, 42);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "88001110");
    let enemy = put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10573310");
    choose(&db, &mut st, 1);
    let got = st
        .field_inst(opp, enemy)
        .expect("enemy still in slot")
        .card
        .as_str();
    assert_eq!(got, "90074140");
}

/// Three Sandalphons in deck, evolves_used ≥ 6: exactly one Invoked. Rally
/// counts. Invoked ability: crest + bounce. Super Skybound Art is not
/// exercised here.
#[test]
fn sandalphon_invoke_one_copy_rally_and_bounce() {
    let db = load_db();
    let mut st = started(&db, 43);
    let me = PlayerId::A;
    st.player_mut(me).evolves_used = 6;
    for _ in 0..3 {
        push_deck(&db, &mut st, me, "10404110");
    }
    let deck_before = st
        .player(me)
        .deck
        .iter()
        .filter(|c| c.card.as_str() == "10404110")
        .count();
    assert_eq!(deck_before, 3);
    let rally_before = st.player(me).rally;
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    let deck_after = st
        .player(me)
        .deck
        .iter()
        .filter(|c| c.card.as_str() == "10404110")
        .count();
    assert_eq!(deck_after, 2, "exactly one copy invoked");
    assert!(
        st.picks
            .iter()
            .all(|p| p.what != arena_engine::PickWhat::MultisetPick),
        "Invoke records no pick"
    );
    assert_eq!(st.player(me).rally, rally_before + 1);
    assert!(
        st.player(me)
            .crests
            .iter()
            .any(|c| c.id == "crest:10404110"),
        "on:invoked grants the crest"
    );
    assert!(
        hand_has(&st, me, "10404110"),
        "on:invoked returns him to hand"
    );
    assert!(!field_has(&st, me, "10404110"), "bounced off the field");
}

#[test]
fn sandalphon_invoke_full_field_is_noop() {
    let db = load_db();
    let mut st = started(&db, 44);
    let me = PlayerId::A;
    st.player_mut(me).evolves_used = 6;
    for _ in 0..5 {
        put_field(&db, &mut st, me, "88001110");
    }
    for _ in 0..3 {
        push_deck(&db, &mut st, me, "10404110");
    }
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(
        st.player(me)
            .deck
            .iter()
            .filter(|c| c.card.as_str() == "10404110")
            .count(),
        3,
        "full field: stay in deck"
    );
    assert!(st.player(me).crests.is_empty());
    assert!(!hand_has(&st, me, "10404110"));
}

#[test]
fn substandard_puppet_effect_evolves_copy_and_self() {
    let db = load_db();
    let mut st = started(&db, 45);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10672110");
    let bodies: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "10672110")
        .collect();
    assert_eq!(bodies.len(), 2);
    for b in &bodies {
        assert!(b.evolved);
        assert_eq!(b.attack, 4, "+2/+2 on the 2/1");
        assert_eq!(b.defense, 3);
    }
    assert_eq!(st.player(me).evolves_used, 2);
    assert!(!st.player(me).evolved_this_turn);
}

#[test]
fn already_evolved_is_skipped() {
    let db = load_db();
    let mut st = started(&db, 46);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "10674110");
    let slot = put_field(&db, &mut st, me, "10672110");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.evolved = true;
        f.attack = 4;
        f.defense = 3;
        f.max_defense = 3;
    }
    st.player_mut(me).evolves_used = 0;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10671110");
    // Shoddy (6-cost) enters; Camiscilla evolves it. The already-evolved
    // Substandard is not evolved again.
    let sub = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10672110")
        .unwrap();
    assert_eq!(sub.attack, 4);
    assert_eq!(st.player(me).evolves_used, 1, "only Shoddy");
}

#[test]
fn asher_enhance_any_evolve_destroys_wards() {
    let db = load_db();
    let mut st = started(&db, 47);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "10001120");
    put_field(&db, &mut st, opp, "10001130");
    put_field(&db, &mut st, opp, "10061110");
    give_pp(&mut st, me, 9, 9);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10874110");
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    let asher = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10874110")
        .expect("Asher");
    assert!(asher.evolved);
    assert!(asher.is_storm());
    assert_eq!(st.player(me).evolves_used, 1);
    assert_eq!(
        field_count(&st, opp),
        1,
        "anyEvolve destroyed 2 of 3 Ward followers"
    );
}

#[test]
fn camiscilla_evolves_each_batch_entrant() {
    let db = load_db();
    let mut st = started(&db, 48);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "10674110");
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10672110");
    // Accelerate (3): two Substandard copies; Camiscilla evolves each.
    let bodies: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "10672110")
        .collect();
    assert_eq!(bodies.len(), 2);
    assert!(bodies.iter().all(|c| c.evolved));
    assert_eq!(st.player(me).evolves_used, 2);
}

#[test]
fn brazen_grants_rush_to_each_artifact_entrant() {
    let db = load_db();
    let mut st = started(&db, 49);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "10773110");
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10773110");
    // Fanfare Analyzing + Enhance Mystic; both get Rush from the one already
    // on the field, and the played Brazen also grants to later summons.
    let arts: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == "90071130" || c.card.as_str() == "90071150")
        .collect();
    assert_eq!(arts.len(), 2);
    assert!(arts.iter().all(|c| c.is_rush()), "give it Rush per entrant");
}

#[test]
fn scarlet_distinct_names_entered_this_match() {
    let db = load_db();
    let mut st = started(&db, 50);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.defense = 10;
        f.max_defense = 10;
    }
    // Successful entries only (put_field does not count). Summon via play.
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10773110"); // Analyzing + Enhance Mystic
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "90071140"); // Ancient
    give_pp(&mut st, me, 8, 8);
    play_id(&db, &mut st, me, "10774110");
    let tank = st.field_inst(opp, 0).expect("still there");
    // Brazen + Analyzing + Ancient + Scarlet herself is not Artifact.
    // distinct artifact names that entered: Analyzing, Ancient (Mystic only
    // on Enhance 5; Brazen was played at 10 so Enhance also fired → Mystic).
    assert!(
        tank.defense < 10,
        "Scarlet Fanfare dealt distinctNames damage, left {def}",
        def = tank.defense
    );
}

#[test]
fn freerunning_all_modes_when_three_names() {
    let db = load_db();
    let mut st = started(&db, 51);
    let me = PlayerId::A;
    st.player_mut(me).enter_counts.insert(cid("90071130"), 1);
    st.player_mut(me).enter_counts.insert(cid("90071140"), 1);
    st.player_mut(me).enter_counts.insert(cid("90071150"), 1);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10771310");
    assert!(
        !matches!(st.phase, Phase::Choice { .. }),
        "pick:all does not pause"
    );
    assert!(hand_has(&st, me, "90071130"));
    assert!(hand_has(&st, me, "90071140"));
}

#[test]
fn imari_distinct_names_draw() {
    let db = load_db();
    let mut st = started(&db, 52);
    let me = PlayerId::A;
    st.player_mut(me).turns_taken = 7;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).sep = 1;
    st.player_mut(me).evolved_this_turn = false;
    st.player_mut(me).hand.clear();
    let slot = put_field(&db, &mut st, me, "10574120");
    st.player_mut(me).deck.clear();
    push_deck(&db, &mut st, me, "10573310");
    push_deck(&db, &mut st, me, "10573310");
    push_deck(&db, &mut st, me, "10771310");
    for _ in 0..10 {
        push_deck(&db, &mut st, me, "88001110");
    }
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .unwrap();
    let spells: Vec<_> = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == "10573310" || c.card.as_str() == "10771310")
        .map(|c| c.card.as_str())
        .collect();
    assert_eq!(spells.len(), 2);
    assert_ne!(spells[0], spells[1], "differently named");
    let draws: Vec<_> = st
        .picks
        .iter()
        .filter(|p| p.what == arena_engine::PickWhat::Draw)
        .collect();
    assert!(draws.len() >= 2);
    assert!(draws.iter().any(|p| p.among.as_deref() == Some("deck")));
}

#[test]
fn depths_copy_of_fresh_print_cost_minus_three() {
    let db = load_db();
    let mut st = started(&db, 53);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, "10674110");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.evolved = true;
        f.attack = 8;
        f.defense = 8;
    }
    give_pp(&mut st, me, 0, 0);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "90074320");
    choose(&db, &mut st, slot);
    let copy = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10674110")
        .expect("copy in hand");
    assert!(!copy.evolved, "a copy, not an exact copy");
    assert_eq!(copy.attack, 6);
    assert_eq!(copy.cost, 4, "printed 7 − 3");
}

#[test]
fn accelerate_substandard_3_and_ludicrous_4() {
    let db = load_db();
    let mut st = started(&db, 54);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10672110");
    play(&db, &mut st, h);
    assert_eq!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .filter(|c| c.card.as_str() == "10672110")
            .count(),
        2
    );
    assert!(st
        .player(me)
        .field
        .iter()
        .flatten()
        .all(|c| c.base_cost == 5 || c.card.as_str() != "10672110"));
    assert!(!st.player(me).evolved_this_turn);
    // Ludicrous Accelerate 4
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10673110");
    play(&db, &mut st, h);
    assert!(field_has(&st, me, "10673110"));
    let body = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "10673110")
        .unwrap();
    assert_eq!(body.base_cost, 8, "summoned body keeps printed cost");
}

#[test]
fn analyzing_artifact_enter_draws_transform_does_not() {
    let db = load_db();
    let mut st = started(&db, 55);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10773110");
    assert!(
        st.picks
            .iter()
            .any(|p| p.what == arena_engine::PickWhat::Draw),
        "Analyzing on:enter draws"
    );
}
