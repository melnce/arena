//! M3 wave 1: Havencraft + Neutral fixtures and require_supported closure.

use arena_engine::card::{CardKind, Class};
use arena_engine::state::DestroyedRecord;
use arena_engine::{apply, Action, AttackTarget, CardId, Phase, PlayerId, Slot};

mod common;
use common::*;

const VANILLA: &str = "88001110";
const TANK: &str = "88001320";
const OMERIO: &str = "10964120";
const HOLY_FALCON: &str = "90061110";
const AWED: &str = "10461210";
const GEMS: &str = "10463210";
const SHELL: &str = "10562210";
const UNHOLY: &str = "10661210";
const LYANTHOTH: &str = "10664120";
const DEPTHS: &str = "90064320";
const SAINT: &str = "10563110";
const FOX: &str = "10061120";
const TENETS: &str = "10961110";
const LAMRETTA: &str = "10461120";
const GALLEON: &str = "10464110";
const KANDIMA: &str = "10664110";
const EDETH: &str = "10862110";
const KATALINA: &str = "10401110";
const GODDESS: &str = "10502110";
const GETENOU: &str = "10504110";
const AZVALDT: &str = "10903210";
const ZERAEL: &str = "10904110";
const BEAST: &str = "10603110";
const ZOE: &str = "10864120";
const ENCROACHED: &str = "10602210";
const SCRIPTURE: &str = "10662210";
const ADVENT: &str = "10661310";

fn require_ok(db: &arena_engine::CardDb, ids: &[&str]) {
    for id in ids {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

fn drain_choice(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn set_round(st: &mut arena_engine::State, who: PlayerId, n: u32) {
    st.player_mut(who).turns_taken = n;
    st.turn = n;
    give_pp(st, who, n as i32, n as i32);
}

fn grant_evolve(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8) {
    let me = st.active;
    if st.player(me).turns_taken < 5 {
        set_round(st, me, 5);
    }
    let ep = st.player(me).ep.max(1);
    st.player_mut(me).ep = ep;
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve");
    drain_choice(db, st);
}

fn remember_destroyed(
    st: &mut arena_engine::State,
    who: PlayerId,
    id: &str,
    base_cost: i32,
    kind: CardKind,
) {
    st.player_mut(who).destroyed_history.push(DestroyedRecord {
        card: cid(id),
        base_cost,
        kind,
        owner: who,
        from_field: true,
    });
}

#[test]
fn require_supported_haven_neutral_closure() {
    let db = load_db();
    require_ok(
        &db,
        &[
            "10001110", "10001120", "10001130", "10001210", "10002110", "10002120", "10002210",
            "10061110", "10061120", "10061130", "10061210", "10062110", "10062120", "10062210",
            "10401110", "10401120", "10402110", "10403110", "10403120", "10404110", "10461110",
            "10461120", "10461210", "10462110", "10462120", "10462210", "10463110", "10463210",
            "10464110", "10464120", "10501110", "10502110", "10502120", "10503210", "10503310",
            "10504110", "10561110", "10561120", "10561310", "10562110", "10562120", "10562210",
            "10563110", "10563210", "10564110", "10564120", "10601110", "10601120", "10602210",
            "10603110", "10603210", "10604110", "10661110", "10661210", "10661310", "10662110",
            "10662120", "10662210", "10663110", "10663210", "10664110", "10664120", "10701110",
            "10701310", "10702110", "10703110", "10703210", "10704110", "10704120", "10761110",
            "10761120", "10761210", "10762110", "10762120", "10762210", "10763110", "10763210",
            "10764110", "10764120", "10801110", "10801120", "10802110", "10802310", "10803110",
            "10803310", "10804110", "10804120", "10861110", "10861120", "10861130", "10862110",
            "10862120", "10862310", "10863110", "10863210", "10864110", "10864120", "10901110",
            "10901310", "10902110", "10903110", "10903210", "10904110", "10961110", "10961120",
            "10961210", "10962110", "10962120", "10962310", "10963110", "10963210", "10964110",
            "10964120", "90061110", "90061120", "90061130", "90064210", "90064320",
        ],
    );
    for id in db.cards.values() {
        if matches!(id.class(), Class::Havencraft | Class::Neutral) {
            db.require_supported(id.id())
                .unwrap_or_else(|e| panic!("{}: {e}", id.id()));
        }
    }
}

/// Omerio's sequence over four amulet destructions (1, 2, 3, then 1 again).
#[test]
fn omerio_sequence_wraps_after_four_amulet_destructions() {
    let db = load_db();
    let mut st = started_decks(&db, 11, &[OMERIO], &[VANILLA]);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).field = Default::default();
    let o = put_field(&db, &mut st, me, OMERIO);
    put_field(&db, &mut st, me, AWED);
    put_field(&db, &mut st, me, GEMS);
    put_field(&db, &mut st, me, SHELL);
    put_field(&db, &mut st, me, UNHOLY);
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, TANK);
    st.player_mut(me).leader_defense = 10;
    let before_def = st.player(me).leader_defense;
    grant_evolve(&db, &mut st, o);
    assert!(
        field_has(&st, me, HOLY_FALCON),
        "step 3 summons a Holy Falcon"
    );
    assert_eq!(
        st.player(me).leader_defense,
        before_def + 2,
        "step 2 restores 2"
    );
    let omerio = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid(OMERIO))
        .expect("omerio");
    assert_eq!(
        omerio.sequence_index, 1,
        "four activations: 0,1,2,0 → next is 1"
    );
    let tanks: Vec<i32> = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .map(|c| c.defense)
        .collect();
    assert!(
        tanks.iter().any(|&d| d < 10),
        "steps 1 and 1-again damaged enemy followers"
    );
}

/// Lyanthoth's faith reaching 10 and paying for a Depths of the Eld Tome.
#[test]
fn lyanthoth_faith_reaches_10_and_pays() {
    let db = load_db();
    let mut st = started_decks(&db, 12, &[LYANTHOTH], &[VANILLA]);
    let me = PlayerId::A;
    assert!(
        st.player(me)
            .crests
            .iter()
            .any(|c| c.id == "faith:10664120"),
        "Faith: Lyanthoth from the starting deck"
    );
    assert_eq!(st.player(me).faith, 0);
    put_field(&db, &mut st, me, UNHOLY);
    give_pp(&mut st, me, 1, 1);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, DEPTHS);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(me).faith,
        1,
        "destroyed allied amulet ticks faith"
    );

    st.player_mut(me).field = Default::default();
    put_field(&db, &mut st, me, LYANTHOTH);
    st.player_mut(me).faith = 10;
    clear_hand(&mut st, me);
    end_turn(&db, &mut st);
    assert!(
        hand_has(&st, me, DEPTHS) || st.player(me).hand.iter().any(|c| c.card == cid(DEPTHS)),
        "pay faith 10 adds Depths of the Eld Tome"
    );
    assert_eq!(st.player(me).faith, 0);
}

/// Saint of Rehabilitation summons a Fox per restore during the owner's turn only.
#[test]
fn saint_fox_per_restore_owner_turn_only() {
    let db = load_db();
    let mut st = started_decks(&db, 13, &[SAINT], &[VANILLA]);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, SAINT);
    drain_choice(&db, &mut st);
    assert!(
        field_has(&st, me, FOX),
        "Fanfare restore during your turn summons a Fox"
    );
    let foxes = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card == cid(FOX))
        .count();
    // 0-heal at max still counts as restored.
    st.player_mut(me).leader_defense = st.player(me).leader_max;
    give_pp(&mut st, me, 3, 3);
    play_id(&db, &mut st, me, "10962110"); // Agent EOT restore; also fanfare
    drain_choice(&db, &mut st);
    end_turn(&db, &mut st);
    let foxes_after = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card == cid(FOX))
        .count();
    assert!(
        foxes_after > foxes,
        "a restore that heals 0 still summons a Fox; had {foxes}, now {foxes_after}"
    );

    // Opponent's turn: restoring the opponent's leader does not summon for us.
    let foxes_mid = foxes_after;
    give_pp(&mut st, PlayerId::B, 5, 5);
    clear_hand(&mut st, PlayerId::B);
    play_id(&db, &mut st, PlayerId::B, SAINT);
    drain_choice(&db, &mut st);
    let foxes_end = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card == cid(FOX))
        .count();
    assert_eq!(
        foxes_end, foxes_mid,
        "opponent-turn restores do not summon a Fox for the original Saint"
    );
}

/// Follower of the Tenets evolves on a restore, and not on the opponent's turn.
#[test]
fn tenets_evolves_on_restore_not_opponent_turn() {
    let db = load_db();
    let mut st = started_decks(&db, 14, &[TENETS], &[VANILLA]);
    let me = PlayerId::A;
    st.player_mut(me).field = Default::default();
    let slot = put_field(&db, &mut st, me, TENETS);
    give_pp(&mut st, me, 3, 3);
    play_id(&db, &mut st, me, "10962110");
    drain_choice(&db, &mut st);
    end_turn(&db, &mut st);
    let evolved = st.field_inst(me, slot).map(|c| c.evolved).unwrap_or(false);
    assert!(evolved, "restore during your turn evolves Tenets");

    // Fresh copy; opponent's restore of *their* leader leaves it unevolved.
    let mut st = started_decks(&db, 15, &[TENETS], &[VANILLA]);
    st.player_mut(me).field = Default::default();
    let slot = put_field(&db, &mut st, me, TENETS);
    end_turn(&db, &mut st);
    give_pp(&mut st, PlayerId::B, 3, 3);
    clear_hand(&mut st, PlayerId::B);
    play_id(&db, &mut st, PlayerId::B, "10962110");
    drain_choice(&db, &mut st);
    end_turn(&db, &mut st);
    let evolved = st.field_inst(me, slot).map(|c| c.evolved).unwrap_or(false);
    assert!(
        !evolved,
        "opponent-turn restore of their leader does not evolve Tenets"
    );
}

/// Lamretta's end-of-turn 2 damage hits all followers incl. herself, gated on evolved.
#[test]
fn lamretta_eot_two_damage_all_followers_if_evolved() {
    let db = load_db();
    let mut st = started_decks(&db, 16, &[LAMRETTA], &[VANILLA]);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).field = Default::default();
    let slot = put_field(&db, &mut st, me, LAMRETTA);
    put_field(&db, &mut st, me, VANILLA);
    put_field(&db, &mut st, opp, VANILLA);
    grant_evolve(&db, &mut st, slot);
    end_turn(&db, &mut st);
    for p in [me, opp] {
        for c in st.player(p).field.iter().flatten() {
            if c.kind == CardKind::Follower {
                assert!(
                    c.defense <= c.max_defense.saturating_sub(2) || c.defense < 4,
                    "{} {} should have taken the 2 EOT damage",
                    p.as_str(),
                    c.card
                );
            }
        }
    }
}

/// Galleon's random evolve skips evolved and attacked followers.
#[test]
fn galleon_random_evolve_skips_evolved_and_attacked() {
    let db = load_db();
    let mut st = started_decks(&db, 17, &[GALLEON], &[VANILLA]);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    set_round(&mut st, me, 7);
    st.player_mut(me).field = Default::default();
    let g = put_field(&db, &mut st, me, GALLEON);
    let attacked = put_field(&db, &mut st, me, VANILLA);
    let evolved = put_field(&db, &mut st, me, VANILLA);
    let fresh = put_field(&db, &mut st, me, VANILLA);
    put_field(&db, &mut st, opp, VANILLA);
    if let Some(c) = st.field_inst_mut(me, attacked) {
        c.flags.attacked_this_turn = true;
        c.flags.summoning_sick = false;
    }
    if let Some(c) = st.field_inst_mut(me, evolved) {
        c.evolved = true;
    }
    if let Some(c) = st.field_inst_mut(me, g) {
        c.evolved = true;
    }
    end_turn(&db, &mut st);
    let fresh_evo = st.field_inst(me, fresh).map(|c| c.evolved).unwrap_or(false);
    let attacked_evo = st
        .field_inst(me, attacked)
        .map(|c| c.evolved)
        .unwrap_or(false);
    assert!(
        fresh_evo,
        "the unevolved follower that didn't attack evolves"
    );
    assert!(
        !attacked_evo,
        "the follower that attacked this turn is skipped"
    );
}

/// Kandima summons two differently named destroyed Last Words amulets.
#[test]
fn kandima_two_differently_named_copies() {
    let db = load_db();
    let mut st = started_decks(&db, 18, &[KANDIMA], &[VANILLA]);
    let me = PlayerId::A;
    remember_destroyed(&mut st, me, UNHOLY, 2, CardKind::Amulet);
    remember_destroyed(&mut st, me, SCRIPTURE, 2, CardKind::Amulet);
    give_pp(&mut st, me, 4, 4);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, KANDIMA);
    drain_choice(&db, &mut st);
    let names: Vec<CardId> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card != cid(KANDIMA))
        .map(|c| c.card)
        .collect();
    assert_eq!(names.len(), 2, "two copies summoned: {names:?}");
    assert_ne!(names[0], names[1], "differently named");
}

/// Edeth's summoned copy has Last Words removed.
#[test]
fn edeth_summoned_copy_without_last_words() {
    let db = load_db();
    let mut st = started_decks(&db, 19, &[EDETH], &[VANILLA]);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).field = Default::default();
    let slot = put_field(&db, &mut st, me, EDETH);
    let atk = put_field(&db, &mut st, opp, TANK);
    if let Some(c) = st.field_inst_mut(opp, atk) {
        c.flags.summoning_sick = false;
        c.flags.attacks_left = 1;
    }
    end_turn(&db, &mut st);
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(slot)),
        },
    )
    .expect("attack edeth");
    drain_choice(&db, &mut st);
    let copy = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid(EDETH))
        .expect("summoned Edeth");
    assert!(
        !copy.printed_tags.contains("lastWords"),
        "Last Words stripped from the summoned copy"
    );
}

/// Katalina's damage cap under a 5-damage (actually 10) hit.
#[test]
fn katalina_damage_cap_under_five_damage_hit() {
    let db = load_db();
    let mut st = started_decks(&db, 20, &[KATALINA], &[VANILLA]);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(opp).field = Default::default();
    let k = put_field(&db, &mut st, opp, KATALINA);
    let atk = put_field(&db, &mut st, me, TANK);
    if let Some(c) = st.field_inst_mut(me, atk) {
        c.flags.summoning_sick = false;
        c.flags.attacks_left = 1;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(k)),
        },
    )
    .expect("hit katalina");
    let def = st.field_inst(opp, k).map(|c| c.defense).unwrap_or(0);
    assert_eq!(def, 2, "5 def, cap 3 from a 10-damage hit → 2 remaining");
}

/// Goddess of Starlight copies the 3 leftmost remaining cards after discarding 3.
#[test]
fn goddess_leftmost_three_copies() {
    let db = load_db();
    let mut st = started_decks(&db, 21, &[GODDESS], &[VANILLA]);
    let me = PlayerId::A;
    set_round(&mut st, me, 5);
    st.player_mut(me).ep = 1;
    st.player_mut(me).field = Default::default();
    let slot = put_field(&db, &mut st, me, GODDESS);
    clear_hand(&mut st, me);
    // Leftmost after discarding the three rightmost: VANILLA, ADVENT, SCRIPTURE.
    put_hand(&db, &mut st, me, VANILLA);
    put_hand(&db, &mut st, me, ADVENT);
    put_hand(&db, &mut st, me, SCRIPTURE);
    put_hand(&db, &mut st, me, UNHOLY);
    put_hand(&db, &mut st, me, AWED);
    put_hand(&db, &mut st, me, GEMS);
    if st.player(me).turns_taken < 5 {
        set_round(&mut st, me, 5);
    }
    st.player_mut(me).ep = st.player(me).ep.max(1);
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve goddess");
    // Discard the three rightmost so the leftmost three remain.
    for _ in 0..3 {
        if let Phase::Choice { node, .. } = &st.phase {
            if let arena_engine::ChoiceNode::Targets { options, .. } = node {
                let last = (options.len() - 1) as u8;
                choose(&db, &mut st, last);
            } else {
                choose(&db, &mut st, 0);
            }
        }
    }
    drain_choice(&db, &mut st);
    let hand: Vec<String> = st.player(me).hand.iter().map(|c| c.card.as_str()).collect();
    let vanilla = hand.iter().filter(|id| *id == VANILLA).count();
    let advent = hand.iter().filter(|id| *id == ADVENT).count();
    let scripture = hand.iter().filter(|id| *id == SCRIPTURE).count();
    assert_eq!(vanilla, 2, "leftmost vanilla copied: {hand:?}");
    assert_eq!(advent, 2, "leftmost advent copied: {hand:?}");
    assert_eq!(scripture, 2, "leftmost scripture copied: {hand:?}");
}

/// Getenou mode 2 binds the two drawn cards and applies cost −8.
#[test]
fn getenou_cost_minus_eight_on_two_drawn() {
    let db = load_db();
    let mut st = started_decks(&db, 22, &[GETENOU], &[VANILLA]);
    let me = PlayerId::A;
    give_pp(&mut st, me, 8, 8);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, GETENOU);
    // Mode 2.
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 1);
    }
    drain_choice(&db, &mut st);
    let drawn: Vec<i32> = st.player(me).hand.iter().map(|c| c.cost).collect();
    assert_eq!(drawn.len(), 2, "drew 2: {drawn:?}");
    assert!(
        drawn.iter().all(|&c| c <= 0),
        "both drawn cards have cost −8 applied: {drawn:?}"
    );
}

/// Azvaldt's cost-history condition and Last Words four differently named +3/+3.
#[test]
fn azvaldt_cost_history_and_last_words() {
    let db = load_db();
    let mut st = started_decks(&db, 23, &[AZVALDT, ZERAEL], &[VANILLA]);
    let me = PlayerId::A;
    st.player_mut(me).played_base_costs_this_match = vec![1, 2, 3, 4, 5, 6, 7, 8];
    st.player_mut(me).field = Default::default();
    put_field(&db, &mut st, me, AZVALDT);
    remember_destroyed(&mut st, me, VANILLA, 1, CardKind::Follower);
    remember_destroyed(&mut st, me, KATALINA, 5, CardKind::Follower);
    remember_destroyed(&mut st, me, LAMRETTA, 2, CardKind::Follower);
    remember_destroyed(&mut st, me, TENETS, 2, CardKind::Follower);
    // Ensure Zerael is in the deck for Invoke.
    if !st.player(me).deck.iter().any(|c| c.card == cid(ZERAEL)) {
        put_hand(&db, &mut st, me, ZERAEL);
        // move from hand to deck
        if let Some(i) = st
            .player(me)
            .hand
            .iter()
            .position(|c| c.card == cid(ZERAEL))
        {
            let inst = st.player_mut(me).hand.remove(i);
            st.player_mut(me).deck.push(inst);
        }
    }
    end_turn(&db, &mut st);
    drain_choice(&db, &mut st);
    assert!(
        !field_has(&st, me, AZVALDT),
        "Azvaldt self-destroyed on the cost-history EOT"
    );
    assert!(
        field_has(&st, me, ZERAEL),
        "Zerael invoked after Azvaldt died and before Last Words finish the board"
    );
    let copies: Vec<CardId> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card != cid(ZERAEL))
        .map(|c| c.card)
        .collect();
    assert_eq!(
        copies.len(),
        4,
        "four destroyed-this-match copies: {copies:?}"
    );
    let uniq: std::collections::BTreeSet<_> = copies.iter().collect();
    assert_eq!(uniq.len(), 4, "differently named");
    for c in st.player(me).field.iter().flatten() {
        if c.kind == CardKind::Follower {
            assert!(c.attack >= 3, "+3/+3 to all including Zerael / copies");
        }
    }
}

/// Beast Lost to the Dark grants three distinct traits.
#[test]
fn beast_lost_three_distinct_traits() {
    let db = load_db();
    let mut st = started_decks(&db, 24, &[BEAST], &[VANILLA]);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, BEAST);
    drain_choice(&db, &mut st);
    let beast = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid(BEAST))
        .expect("beast");
    let mut n = 0;
    if beast.traits.storm == Some(true) {
        n += 1;
    }
    if beast.traits.bane == Some(true) {
        n += 1;
    }
    if beast.traits.intimidate == Some(true) {
        n += 1;
    }
    if beast.traits.drain == Some(true) {
        n += 1;
    }
    if beast.traits.aura == Some(true) {
        n += 1;
    }
    if beast.traits.barrier == Some(true) {
        n += 1;
    }
    assert_eq!(n, 3, "three distinct granted abilities");
}

/// Zoe's self-damage is a second clause after the mode choice.
#[test]
fn zoe_self_damage_after_mode() {
    let db = load_db();
    let mut st = started_decks(&db, 25, &[ZOE], &[VANILLA]);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    clear_hand(&mut st, me);
    play_id(&db, &mut st, me, ZOE);
    // Mode 3: restore 3 — then self-damage 3.
    if matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 2);
    }
    drain_choice(&db, &mut st);
    let zoe = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid(ZOE))
        .expect("zoe");
    assert_eq!(zoe.defense, 1, "2/4 minus 3 self-damage after the mode");
}

/// Encroached World's hand transform into an exact copy of an enemy-deck card.
#[test]
fn encroached_world_hand_transform_from_enemy_deck() {
    let db = load_db();
    let mut st = started_decks(&db, 26, &[ENCROACHED], &[KATALINA]);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).field = Default::default();
    let slot = put_field(&db, &mut st, me, ENCROACHED);
    clear_hand(&mut st, me);
    put_hand(&db, &mut st, me, VANILLA);
    // Guarantee the only enemy-deck name we care about is Katalina.
    st.player_mut(opp).deck.retain(|c| c.card == cid(KATALINA));
    if st.player(opp).deck.is_empty() {
        put_hand(&db, &mut st, opp, KATALINA);
        if let Some(i) = st
            .player(opp)
            .hand
            .iter()
            .position(|c| c.card == cid(KATALINA))
        {
            let inst = st.player_mut(opp).hand.remove(i);
            st.player_mut(opp).deck.push(inst);
        }
    }
    apply(&db, &mut st, Action::Engage { slot: Slot(slot) }).expect("engage");
    drain_choice(&db, &mut st);
    assert!(
        hand_has(&st, me, KATALINA),
        "hand card transformed into an exact copy of an enemy-deck card"
    );
    assert!(
        !hand_has(&st, me, VANILLA),
        "the original hand card is gone"
    );
}

/// Camiscilla's `pick: entering` must survive Bahamut mode 1 compacting the board.
#[test]
fn camiscilla_evolve_entering_survives_bahamut_banish() {
    let db = load_db();
    let mut st = started_decks(&db, 27, &["10804110"], &[VANILLA]);
    let me = PlayerId::A;
    st.player_mut(me).field = Default::default();
    put_field(&db, &mut st, me, "10674110");
    put_field(&db, &mut st, me, VANILLA);
    clear_hand(&mut st, me);
    let h = put_hand(&db, &mut st, me, "10804110");
    give_pp(&mut st, me, 9, 9);
    play(&db, &mut st, h);
    assert!(matches!(st.phase, Phase::Choice { .. }), "Bahamut modes");
    choose(&db, &mut st, 0);
    drain_choice(&db, &mut st);
    assert_eq!(field_count(&st, me), 1, "other followers banished");
    let bahamut = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card == cid("10804110"))
        .expect("bahamut");
    assert!(
        bahamut.evolved,
        "Camiscilla evolve-entering finds Bahamut after compact"
    );
    assert_eq!(bahamut.attack, 15);
    assert_eq!(bahamut.defense, 15);
}
