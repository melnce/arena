//! Set 10 Neutral cards + Divine Evidence: printed behaviour, Nacreheim Q&As,
//! Clue merge/destroy/untargetable, Scene of Deicide, Engage-after-merge.
//! Assert against card text and the Clue rules — never against incidental engine state.

use arena_engine::event::Event;
use arena_engine::{apply, legal_actions, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const NACREHEIM: &str = "11003210";
const EXIAS: &str = "11001110";
const DYNAMES: &str = "11002110";
const KYRIOS: &str = "11003110";
const SCENE: &str = "11002210";
const REZEAL: &str = "11004110";
const REZEAL_CREST: &str = "crest:11004110";
const TANK: &str = "89500001"; // 2/10
const STORM: &str = "88001120"; // 2/2 Storm
const CHOOSE_DESTROY_AMULET: &str = "89911001";
const RANDOM_BANISH_AMULET: &str = "89911002";

const SET10_NEUTRALS: &[&str] = &[EVIDENCE, NACREHEIM, EXIAS, DYNAMES, KYRIOS, SCENE, REZEAL];

fn clue_on_field(st: &arena_engine::State, who: PlayerId) -> Option<&arena_engine::CardInstance> {
    st.player(who).field.iter().flatten().find(|c| c.is_clue())
}

fn clue_count(st: &arena_engine::State, who: PlayerId) -> i32 {
    clue_on_field(st, who).map(|c| c.clue_count).unwrap_or(0)
}

fn clues_on_side(st: &arena_engine::State, who: PlayerId) -> usize {
    st.player(who)
        .field
        .iter()
        .flatten()
        .filter(|c| c.is_clue())
        .count()
}

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    give_pp(st, st.active, pp, pp.max(10));
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn engage_slot(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    give_pp(st, st.active, 10, 10);
    apply(db, st, Action::Engage { slot: Slot(slot) }).expect("engage");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

#[test]
fn set10_neutrals_require_supported() {
    let db = load_db();
    for id in SET10_NEUTRALS {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    db.crest(REZEAL_CREST)
        .unwrap_or_else(|e| panic!("{REZEAL_CREST}: {e}"));
}

#[test]
fn divine_evidence_engage_draw_consume() {
    let db = load_db();
    let mut st = started(&db, 10_010);
    let hand0 = st.player(PlayerId::A).hand.len();
    let slot = put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    assert_eq!(clue_count(&st, PlayerId::A), 1);
    let shadows0 = st.player(PlayerId::A).shadows;
    engage_slot(&db, &mut st, slot);
    // Drew 1; consumed last clue → destroyed (shadow +1), not banished.
    assert_eq!(st.player(PlayerId::A).hand.len(), hand0 + 1);
    assert_eq!(clues_on_side(&st, PlayerId::A), 0);
    assert_eq!(st.player(PlayerId::A).shadows, shadows0 + 1);
    assert!(
        st.player(PlayerId::A)
            .cemetery
            .iter()
            .any(|c| c.card.as_str() == EVIDENCE),
        "zero clues destroys into cemetery"
    );
    assert!(
        st.player(PlayerId::A)
            .banished
            .iter()
            .all(|c| c.card.as_str() != EVIDENCE),
        "negative: consume-to-0 must not banish"
    );
}

#[test]
fn nacreheim_banish_heals_destroy_at_zero_does_not() {
    // Q&A #1: merge-banish of Nacreheim fires "when banished" heal.
    let db = load_db();
    let mut st = started(&db, 10_011);
    put_field(&db, &mut st, PlayerId::A, NACREHEIM);
    st.player_mut(PlayerId::A).leader_defense = 10;
    play_pp(&db, &mut st, EVIDENCE, 1);
    assert_eq!(clue_count(&st, PlayerId::A), 2, "merge adds counts");
    assert_eq!(
        clues_on_side(&st, PlayerId::A),
        1,
        "only the new amulet remains"
    );
    assert_eq!(
        leader_def(&st, PlayerId::A),
        12,
        "Q&A #1: banished Nacreheim restores 2"
    );
    assert!(st
        .player(PlayerId::A)
        .banished
        .iter()
        .any(|c| c.card.as_str() == NACREHEIM));
    // Negative: no shadow from the merge banish.
    assert_eq!(st.player(PlayerId::A).shadows, 0);

    // Q&A #2: Engage at 1 clue destroys (no banish heal).
    let mut st = started(&db, 10_012);
    let slot = put_field(&db, &mut st, PlayerId::A, NACREHEIM);
    st.player_mut(PlayerId::A).leader_defense = 10;
    let shadows0 = st.player(PlayerId::A).shadows;
    engage_slot(&db, &mut st, slot);
    assert_eq!(
        leader_def(&st, PlayerId::A),
        10,
        "Q&A #2: destroy at 0 does not banish-heal"
    );
    assert_eq!(st.player(PlayerId::A).shadows, shadows0 + 1);
    assert!(
        st.player(PlayerId::A)
            .banished
            .iter()
            .all(|c| c.card.as_str() != NACREHEIM),
        "negative: destroy-at-0 is not a banish"
    );
}

#[test]
fn clue_merge_two_and_three() {
    let db = load_db();
    // Two: Evidence then Nacreheim → count 2; banished Evidence's self_banished is a no-op.
    let mut st = started(&db, 10_013);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    play_pp(&db, &mut st, NACREHEIM, 1);
    assert_eq!(clue_count(&st, PlayerId::A), 2);
    assert_eq!(clues_on_side(&st, PlayerId::A), 1);
    assert!(field_has(&st, PlayerId::A, NACREHEIM));
    assert!(!field_has(&st, PlayerId::A, EVIDENCE));
    assert_eq!(st.player(PlayerId::A).shadows, 0, "merge banish: no shadow");

    // Three: Evidence, Evidence (via put+play), Nacreheim.
    let mut st = started(&db, 10_014);
    put_field(&db, &mut st, PlayerId::A, EVIDENCE);
    // Second Evidence merges onto first → count 2 on the new Evidence.
    play_pp(&db, &mut st, EVIDENCE, 1);
    assert_eq!(clue_count(&st, PlayerId::A), 2);
    st.player_mut(PlayerId::A).leader_defense = 10;
    play_pp(&db, &mut st, NACREHEIM, 1);
    // Nacreheim enters at 1+2=3; previous Evidence banished (no self_banished heal on Evidence).
    assert_eq!(clue_count(&st, PlayerId::A), 3);
    assert_eq!(clues_on_side(&st, PlayerId::A), 1);
    assert_eq!(
        leader_def(&st, PlayerId::A),
        10,
        "Evidence has no banish heal"
    );
    assert_eq!(st.player(PlayerId::A).shadows, 0);
}

#[test]
fn clue_untargetable_by_enemy_choose_but_random_hits() {
    let db = load_db();
    let mut st = started(&db, 10_015);
    put_field(&db, &mut st, PlayerId::B, EVIDENCE);
    put_field(&db, &mut st, PlayerId::B, "89100020"); // countdown amulet stand-in
                                                      // Choose destroy: only the non-Clue amulet is offered.
    give_pp(&mut st, PlayerId::A, 1, 1);
    let h = put_hand(&db, &mut st, PlayerId::A, CHOOSE_DESTROY_AMULET);
    play(&db, &mut st, h);
    let Phase::Choice {
        node: arena_engine::ChoiceNode::Targets { options, .. },
        ..
    } = &st.phase
    else {
        panic!("expected choose targets for enemy amulet");
    };
    assert_eq!(options.len(), 1, "Clue amulet must not be selectable");
    choose(&db, &mut st, 0);
    assert!(
        field_has(&st, PlayerId::B, EVIDENCE),
        "negative: choose must not destroy the Clue"
    );
    assert!(!field_has(&st, PlayerId::B, "89100020"));

    // Random banish can hit the Clue.
    let mut st = started(&db, 10_016);
    put_field(&db, &mut st, PlayerId::B, EVIDENCE);
    play_pp(&db, &mut st, RANDOM_BANISH_AMULET, 0);
    assert_eq!(clues_on_side(&st, PlayerId::B), 0);
    assert!(st
        .player(PlayerId::B)
        .banished
        .iter()
        .any(|c| c.card.as_str() == EVIDENCE));
}

#[test]
fn engage_merged_amulet_same_turn() {
    let db = load_db();
    let mut st = started(&db, 10_017);
    // Real merge path: play Evidence, play Evidence (count 2).
    play_pp(&db, &mut st, EVIDENCE, 1);
    assert_eq!(clue_count(&st, PlayerId::A), 1);
    play_pp(&db, &mut st, EVIDENCE, 1);
    assert_eq!(clue_count(&st, PlayerId::A), 2, "two Evidence merge to count 2");
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| s.as_ref().filter(|c| c.is_clue()).map(|_| i as u8))
        .expect("clue");
    engage_slot(&db, &mut st, slot);
    assert_eq!(clue_count(&st, PlayerId::A), 1, "Engage consumes a clue");
    assert!(
        st.player(PlayerId::A)
            .field
            .iter()
            .flatten()
            .any(|c| c.is_clue() && c.flags.engaged_this_turn),
        "Evidence engaged this turn"
    );
    // Play Nacreheim → merge onto a new instance at count 2.
    play_pp(&db, &mut st, NACREHEIM, 1);
    assert_eq!(clue_count(&st, PlayerId::A), 2);
    let (slot, engaged) = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.is_clue())
                .map(|c| (i as u8, c.flags.engaged_this_turn))
        })
        .expect("merged clue");
    assert!(
        !engaged,
        "merge survivor is a new instance (engaged_this_turn does not carry)"
    );
    give_pp(&mut st, PlayerId::A, 10, 10);
    let legal = legal_actions(&db, &st);
    assert!(
        legal
            .iter()
            .any(|a| matches!(a, Action::Engage { slot: Slot(s) } if *s == slot)),
        "merged Clue amulet may Engage the same turn"
    );
    let hand0 = st.player(PlayerId::A).hand.len();
    engage_slot(&db, &mut st, slot);
    assert!(st.player(PlayerId::A).hand.len() > hand0, "Engage drew");
    assert_eq!(clue_count(&st, PlayerId::A), 1, "second Engage consumes to 1");
}

#[test]
fn exias_adds_evidence_rush() {
    let db = load_db();
    let mut st = started(&db, 10_020);
    play_pp(&db, &mut st, EXIAS, 2);
    assert!(field_has(&st, PlayerId::A, EXIAS));
    assert!(hand_has(&st, PlayerId::A, EVIDENCE));
    let f = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == EXIAS)
        .expect("exias");
    assert_eq!(f.traits.rush, Some(true));
    // Negative: no Clue on field yet.
    assert_eq!(clues_on_side(&st, PlayerId::A), 0);
}

#[test]
fn dynames_fanfare_and_evolve_replicate() {
    let db = load_db();
    let mut st = started(&db, 10_021);
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, DYNAMES, 3);
    assert!(hand_has(&st, PlayerId::A, EVIDENCE));
    assert!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .any(|c| c.card.as_str() == TANK && c.defense < c.max_defense),
        "Fanfare damaged the enemy follower"
    );
    // Evolve replicates Fanfare: need EP + another enemy.
    put_field(&db, &mut st, PlayerId::B, TANK);
    let slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == DYNAMES)
                .map(|_| i as u8)
        })
        .expect("dynames");
    st.player_mut(PlayerId::A).ep = 1;
    st.player_mut(PlayerId::A).turns_taken = 5; // first-player evolve unlock
    let hand_before = st
        .player(PlayerId::A)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == EVIDENCE)
        .count();
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    let hand_after = st
        .player(PlayerId::A)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == EVIDENCE)
        .count();
    assert_eq!(
        hand_after,
        hand_before + 1,
        "Evolve replicates add Evidence"
    );
}

#[test]
fn kyrios_aoe_and_two_evidence_ward() {
    let db = load_db();
    let mut st = started(&db, 10_022);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, KYRIOS, 5);
    assert_eq!(
        st.player(PlayerId::A)
            .hand
            .iter()
            .filter(|c| c.card.as_str() == EVIDENCE)
            .count(),
        2
    );
    assert!(st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .all(|c| c.defense <= c.max_defense - 2));
    let f = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == KYRIOS)
        .expect("kyrios");
    assert_eq!(f.traits.ward, Some(true));
}

#[test]
fn scene_of_deicide_with_and_without_enemy_and_full_field() {
    let db = load_db();
    // With an enemy follower: banish it and summon enemy Evidence.
    let mut st = started(&db, 10_030);
    put_field(&db, &mut st, PlayerId::A, SCENE);
    put_field(&db, &mut st, PlayerId::B, TANK);
    // Advance to end of A's turn with Scene already on field.
    end_turn(&db, &mut st); // A ends → Scene endOfTurn fires before B's turn?
                            // endOfTurn whose:own fires at end of controller's turn.
    assert!(
        !field_has(&st, PlayerId::B, TANK),
        "enemy follower banished"
    );
    assert!(
        field_has(&st, PlayerId::B, EVIDENCE),
        "enemy Divine Evidence summoned"
    );
    assert_eq!(clue_count(&st, PlayerId::B), 1);

    // Without enemy follower: still summons enemy Evidence (sentence independence).
    let mut st = started(&db, 10_031);
    put_field(&db, &mut st, PlayerId::A, SCENE);
    end_turn(&db, &mut st);
    assert!(
        field_has(&st, PlayerId::B, EVIDENCE),
        "summon still happens with no follower to banish"
    );

    // Full enemy field: summon does nothing (no merge); banish still removes one follower.
    let mut st = started(&db, 10_032);
    put_field(&db, &mut st, PlayerId::A, SCENE);
    for _ in 0..5 {
        put_field(&db, &mut st, PlayerId::B, TANK);
    }
    assert_eq!(st.player(PlayerId::B).field_count(), 5);
    end_turn(&db, &mut st);
    // Banish one follower (4 left) then summon Evidence into the free slot (5 again).
    assert!(
        field_has(&st, PlayerId::B, EVIDENCE),
        "after banish opens a slot, enemy Evidence enters"
    );
    assert_eq!(st.player(PlayerId::B).field_count(), 5);
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .filter(|c| c.card.as_str() == TANK)
            .count(),
        4,
        "one enemy follower was banished"
    );

    // True full-field summon failure: fill after the banish would need a board that
    // stays full — banish then summon on a board that had 5 followers means
    // banish→4 then summon→5. To test summon-noop, put 5 amulets that aren't
    // followers so banish of follower is a no-op and field stays full.
    let mut st = started(&db, 10_033);
    put_field(&db, &mut st, PlayerId::A, SCENE);
    for _ in 0..5 {
        put_field(&db, &mut st, PlayerId::B, "89100020");
    }
    assert_eq!(st.player(PlayerId::B).field_count(), 5);
    end_turn(&db, &mut st);
    assert_eq!(
        st.player(PlayerId::B).field_count(),
        5,
        "no follower to banish; full field → summon does nothing"
    );
    assert!(
        !field_has(&st, PlayerId::B, EVIDENCE),
        "negative: Evidence must not appear on a full field"
    );
}

#[test]
fn rezeal_fanfare_and_crest_play_reaction() {
    let db = load_db();
    // Fanfare: exactly 9 damage to the chosen tank; second enemy untouched.
    let mut st = started(&db, 10_040);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, REZEAL, 9);
    let defs: Vec<i32> = st
        .player(PlayerId::B)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == TANK)
        .map(|c| c.defense)
        .collect();
    assert_eq!(defs.len(), 2, "both tanks still on the field");
    assert!(
        defs.contains(&1) && defs.contains(&10),
        "chosen tank at exactly 1 (9 damage); other still 10; got {defs:?}"
    );

    // Crest via real Super-Evolve of Rezeal.
    let mut st = started(&db, 10_041);
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, REZEAL, 9);
    let rezeal_slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == REZEAL)
                .map(|_| i as u8)
        })
        .expect("rezeal");
    st.player_mut(PlayerId::A).turns_taken = 7;
    st.player_mut(PlayerId::A).sep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(rezeal_slot),
            super_evolve: true,
        },
    )
    .expect("super-evolve");
    assert!(
        st.player(PlayerId::A)
            .crests
            .iter()
            .any(|c| c.id == REZEAL_CREST),
        "Super-Evolve gained Crest: Rezeal"
    );

    // Play Dynames under the crest: Fanfare (Damage) before crest grants.
    put_field(&db, &mut st, PlayerId::B, TANK);
    give_pp(&mut st, PlayerId::A, 10, 10);
    let hand = put_hand(&db, &mut st, PlayerId::A, DYNAMES);
    let mut events = apply(&db, &mut st, Action::Play { hand }).expect("play dynames");
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "Fanfare target choice before crest grant"
    );
    let mid = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == DYNAMES)
        .expect("dynames");
    assert!(
        mid.granted.is_empty() && mid.traits.cant_attack_followers.is_none(),
        "crest grant must not land before Fanfare resolves"
    );
    while matches!(st.phase, Phase::Choice { .. }) {
        events.extend(apply(&db, &mut st, Action::Choose(0)).expect("choose"));
    }
    let play_i = events
        .iter()
        .position(|e| matches!(e, Event::Play { card, .. } if card.as_str() == DYNAMES))
        .expect("Play event");
    let dmg_i = events
        .iter()
        .position(|e| matches!(e, Event::Damage { .. }))
        .expect("Fanfare Damage event");
    assert!(
        play_i < dmg_i,
        "Play before Fanfare Damage in events ({play_i} < {dmg_i})"
    );
    let f = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == DYNAMES)
        .expect("dynames after");
    assert!(
        f.granted
            .iter()
            .any(|a| matches!(a, arena_engine::card::Ability::LastWords { .. })),
        "crest gave Last Words after Fanfare"
    );
    assert_eq!(f.traits.cant_attack_followers, Some(true));
    assert_eq!(f.traits.cant_attack_leader, Some(true));

    // Storm under crest: can't attack this turn; can next turn; LW summons a fresh copy.
    give_pp(&mut st, PlayerId::A, 10, 10);
    let hand = put_hand(&db, &mut st, PlayerId::A, STORM);
    apply(&db, &mut st, Action::Play { hand }).expect("play storm");
    let storm_slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == STORM)
                .map(|_| i as u8)
        })
        .expect("storm");
    let legal = legal_actions(&db, &st);
    assert!(
        !legal
            .iter()
            .any(|a| matches!(a, Action::Attack { attacker: Slot(s), .. } if *s == storm_slot)),
        "Storm under crest can't attack this turn"
    );
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    let storm_slot = st
        .player(PlayerId::A)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == STORM)
                .map(|_| i as u8)
        })
        .expect("storm still there");
    let legal = legal_actions(&db, &st);
    assert!(
        legal
            .iter()
            .any(|a| matches!(a, Action::Attack { attacker: Slot(s), .. } if *s == storm_slot)),
        "Storm can attack next turn after cant-attack expires"
    );
    // Trade Storm into a Tank (2 atk) → Storm dies → LW summons a fresh printed copy.
    let tank_slot = st
        .player(PlayerId::B)
        .field
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            s.as_ref()
                .filter(|c| c.card.as_str() == TANK)
                .map(|_| i as u8)
        })
        .expect("enemy tank");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(storm_slot),
            target: AttackTarget::Slot(Slot(tank_slot)),
        },
    )
    .expect("storm trades");
    let copies: Vec<_> = st
        .player(PlayerId::A)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == STORM)
        .collect();
    assert_eq!(copies.len(), 1, "LW summoned exactly one Storm copy");
    let copy = copies[0];
    assert_eq!(copy.attack, 2);
    assert_eq!(copy.defense, 2);
    assert!(
        copy.granted.is_empty(),
        "fresh copy has no granted Last Words"
    );
    assert!(
        copy.traits.cant_attack_followers.is_none() && copy.traits.cant_attack_leader.is_none(),
        "fresh copy has no cant-attack grant"
    );

    // Negative: crest must not fire on amulet play.
    play_pp(&db, &mut st, EVIDENCE, 1);
    let evidence = clue_on_field(&st, PlayerId::A).expect("evidence");
    assert!(
        evidence
            .granted
            .iter()
            .all(|a| !matches!(a, arena_engine::card::Ability::LastWords { .. })),
        "negative: crest does not grant LW to amulets"
    );
}
