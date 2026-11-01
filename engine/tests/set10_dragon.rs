//! Set 10 Dragoncraft: printed behaviour, Jerva Q&A (X=0), Clue-in-hand
//! discard/draw, Leizaut X∈{0,1,3}, Bird's-Eye modes, Dragonewt Scout Strike.

use arena_engine::{apply, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const MARAUDER: &str = "11041110";
const MANIFEST: &str = "11042310";
const ICENEWT: &str = "11041120";
const DANCE: &str = "11041310";
const SCOUT: &str = "11042120";
const BIRDS: &str = "11043310";
const MUGGER: &str = "11043110";
const JERVA: &str = "11044120";
const SMUGGLER: &str = "11042110";
const LEIZAUT: &str = "11044110";
const MISCHIEF: &str = "11012110";
const TANK: &str = "89500001"; // 2/10
const PAD: &str = "88001110"; // 2/2

const SET10_DRAGON: &[&str] = &[
    MARAUDER, MANIFEST, ICENEWT, DANCE, SCOUT, BIRDS, MUGGER, JERVA, SMUGGLER, LEIZAUT,
];

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    // Preserve an already-raised max (Overflow setups); never clamp it up to 10.
    let max = st.player(st.active).pp_max.max(pp);
    give_pp(st, st.active, pp, max);
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn hand_count(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    st.player(who)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == id)
        .count()
}

fn push_deck(db: &arena_engine::CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).unwrap();
    let id_alloc = st.alloc_id();
    st.player_mut(who)
        .deck
        .push(arena_engine::CardInstance::from_card(card, id_alloc));
}

fn evolve_now(db: &arena_engine::CardDb, st: &mut arena_engine::State, slot: u8, super_evo: bool) {
    let me = st.active;
    st.player_mut(me).turns_taken = st.player(me).turns_taken.max(5);
    if super_evo {
        st.player_mut(me).turns_taken = st.player(me).turns_taken.max(7);
        st.player_mut(me).sep = 1;
    } else {
        st.player_mut(me).ep = 1;
    }
    apply(
        db,
        st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: super_evo,
        },
    )
    .expect("evolve");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

#[test]
fn set10_dragon_require_supported() {
    let db = load_db();
    for id in SET10_DRAGON {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    db.require_supported(cid(MISCHIEF))
        .unwrap_or_else(|e| panic!("{MISCHIEF}: {e}"));
}

#[test]
fn marauder_rush_last_words_and_banish_negative() {
    let db = load_db();
    let mut st = started(&db, 110_418);
    let me = PlayerId::A;
    let m = put_field(&db, &mut st, me, MARAUDER);
    assert_eq!(
        st.field_inst(me, m).unwrap().traits.rush,
        Some(true),
        "Rush"
    );

    // Last Words: destroy → +1 Divine Evidence.
    let mut st = started(&db, 110_419);
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, MARAUDER);
    put_field(&db, &mut st, opp, PAD);
    let hand0 = hand_count(&st, opp, EVIDENCE);
    play_pp(&db, &mut st, SMUGGLER, 7);
    assert_eq!(
        hand_count(&st, opp, EVIDENCE),
        hand0 + 1,
        "Marauder LW: Add a Divine Evidence"
    );

    // Negative: banish does not fire Last Words (left the field via banish).
    let mut st = started(&db, 110_420);
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, MARAUDER);
    let hand0 = hand_count(&st, opp, EVIDENCE);
    play_pp(&db, &mut st, "11064110", 6); // Lapinette Fanfare banishes
    assert_eq!(
        hand_count(&st, opp, EVIDENCE),
        hand0,
        "negative: banish does not fire Marauder Last Words"
    );
}

#[test]
fn collection_manifest_exact_damage_draw_and_negatives() {
    let db = load_db();
    // Deck where only LEIZAUT (cost 10) is eligible → must draw it; tank → def 6.
    let mut st = started(&db, 110_421);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(me).deck.clear();
    for id in [PAD, PAD, LEIZAUT] {
        push_deck(&db, &mut st, me, id);
    }
    put_hand(&db, &mut st, me, EVIDENCE);
    let hand_ev = hand_count(&st, me, EVIDENCE);
    play_pp(&db, &mut st, MANIFEST, 2);
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        6,
        "Deal 4 → tank 10−4=6"
    );
    assert_eq!(hand_count(&st, me, EVIDENCE), hand_ev - 1, "discarded Clue");
    assert_eq!(
        hand_count(&st, me, LEIZAUT),
        1,
        "only eligible base-cost≥5 in deck is Leizaut"
    );

    // Negative: Clue in hand but none eligible in deck → discard, no draw.
    let mut st = started(&db, 110_422);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(me).deck.clear();
    for _ in 0..3 {
        push_deck(&db, &mut st, me, PAD);
    }
    put_hand(&db, &mut st, me, EVIDENCE);
    let hand0 = st.player(me).hand.len();
    play_pp(&db, &mut st, MANIFEST, 2);
    assert_eq!(
        st.player(me).hand.len(),
        hand0 - 1,
        "none-in-deck: discarded Clue, drew nothing"
    );
    assert_eq!(hand_count(&st, me, EVIDENCE), 0);

    // Negative: no Clue in hand → damage only, no draw.
    let mut st = started(&db, 110_4220);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    st.player_mut(me).deck.clear();
    push_deck(&db, &mut st, me, LEIZAUT);
    let hand0 = st.player(me).hand.len();
    play_pp(&db, &mut st, MANIFEST, 2);
    assert_eq!(
        st.player(me).hand.len(),
        hand0,
        "negative: no Clue in hand → no draw"
    );
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        6
    );
}

#[test]
fn freezing_icenewt_enhance_three_not_four() {
    let db = load_db();
    // One tank: every random pick hits it. Enhance = 3×3 = 9 → def 1 (not 4×3=12).
    let mut st = started(&db, 110_423);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    give_pp(&mut st, me, 7, 7);
    let h = put_hand(&db, &mut st, me, ICENEWT);
    play(&db, &mut st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    let ice = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ICENEWT)
        .unwrap();
    assert_eq!((ice.attack, ice.defense), (7, 7), "+5/+5 on Enhance");
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        1,
        "3 times instead: 10−9=1 (4 times would kill)"
    );

    // Negative: base Fanfare is 1×3 → def 7, no buff.
    let mut st = started(&db, 110_424);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    play_pp(&db, &mut st, ICENEWT, 3);
    let ice = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == ICENEWT)
        .unwrap();
    assert_eq!(
        (ice.attack, ice.defense),
        (2, 2),
        "negative: no enhance buff"
    );
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        7,
        "one hit of 3"
    );
}

#[test]
fn dance_draws_restores_and_overflow() {
    let db = load_db();
    let mut st = started(&db, 110_425);
    let me = PlayerId::A;
    st.player_mut(me).pp_max = 7;
    st.player_mut(me).leader_defense = 19;
    st.player_mut(me).deck.clear();
    push_deck(&db, &mut st, me, PAD);
    let hand0 = st.player(me).hand.len();
    let def0 = st.player(me).leader_defense;
    play_pp(&db, &mut st, DANCE, 3);
    assert_eq!(st.player(me).hand.len(), hand0 + 1, "Draw a card");
    assert_eq!(st.player(me).leader_defense, def0 + 1, "Restore 1");
    assert_eq!(st.player(me).pp_max, 8, "Overflow → +1 max PP");

    // Negative: not Overflow → no max PP (draw + restore still fire).
    let mut st = started(&db, 110_426);
    let me = PlayerId::A;
    st.player_mut(me).pp_max = 6;
    st.player_mut(me).leader_defense = 19;
    st.player_mut(me).deck.clear();
    push_deck(&db, &mut st, me, PAD);
    let hand0 = st.player(me).hand.len();
    play_pp(&db, &mut st, DANCE, 3);
    assert_eq!(st.player(me).hand.len(), hand0 + 1, "still draws");
    assert_eq!(
        st.player(me).pp_max,
        6,
        "negative: not Overflow → no max PP"
    );
}

#[test]
fn dragonewt_scout_strike_evolves_other() {
    let db = load_db();
    let mut st = started(&db, 110_427);
    let me = PlayerId::A;
    let scout = put_field(&db, &mut st, me, SCOUT);
    let ally = put_field(&db, &mut st, me, PAD);
    put_field(&db, &mut st, PlayerId::B, PAD);
    if let Some(f) = st.field_inst_mut(me, scout) {
        f.flags.summoning_sick = false;
        f.traits.ambush = None;
        f.traits.storm = Some(true);
    }
    let ep0 = st.player(me).ep;
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(scout),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .expect("attack");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert!(
        st.field_inst(me, ally).unwrap().evolved,
        "Strike evolves another unevolved ally"
    );
    assert_eq!(st.player(me).ep, ep0, "effect evolve spends no EP");
    assert!(
        !st.field_inst(me, scout).unwrap().evolved,
        "negative: does not evolve self"
    );
}

#[test]
fn birds_eye_mode1_mischief_and_mode4_damage() {
    let db = load_db();
    // Modes: 0 Mischief, 1 leftmost, 2 Evidence×3, 3 damage
    let mut st = started(&db, 110_429);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    give_pp(&mut st, me, 3, 3);
    let h = put_hand(&db, &mut st, me, BIRDS);
    play(&db, &mut st, h);
    choose(&db, &mut st, 0); // mode 1: Summon Inquiring Mischief
    choose(&db, &mut st, 2); // mode 4 (index shifts after removing 0): Deal 4
    assert!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .any(|c| c.card.as_str() == MISCHIEF),
        "mode 1 summons Inquiring Mischief 11012110"
    );
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        6,
        "mode 4: Deal 4 → tank 10−4=6"
    );

    // Negative: mode 4 with no enemy follower → Mischief still summons, no crash.
    let mut st = started(&db, 110_430);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    let h = put_hand(&db, &mut st, me, BIRDS);
    play(&db, &mut st, h);
    choose(&db, &mut st, 0); // Mischief
    choose(&db, &mut st, 2); // damage with empty enemy field
    assert_eq!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .filter(|c| c.card.as_str() == MISCHIEF)
            .count(),
        1,
        "negative: no enemy → mode 4 fizzles, mode 1 still summons"
    );
}

#[test]
fn birds_eye_leftmost_buff_unchanged() {
    let db = load_db();
    let mut st = started(&db, 110_4291);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, PAD);
    put_field(&db, &mut st, me, PAD);
    put_field(&db, &mut st, me, TANK);
    give_pp(&mut st, me, 3, 3);
    let h = put_hand(&db, &mut st, me, BIRDS);
    play(&db, &mut st, h);
    choose(&db, &mut st, 1); // leftmost
    choose(&db, &mut st, 1); // Evidence (old 2 → 1 after removal)
    let atks: Vec<i32> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() != MISCHIEF)
        .map(|c| c.attack)
        .collect();
    assert_eq!(atks[0], 3, "leftmost pad +1");
    assert_eq!(atks[1], 3, "second leftmost pad +1");
    assert_eq!(atks[2], 2, "rightmost tank unchanged");
    assert_eq!(hand_count(&st, me, EVIDENCE), 3);
}

#[test]
fn mugger_clue_ramp_and_evolve_damage() {
    let db = load_db();
    let mut st = started(&db, 110_431);
    let me = PlayerId::A;
    st.player_mut(me).pp_max = 4;
    put_hand(&db, &mut st, me, EVIDENCE);
    let max0 = st.player(me).pp_max;
    play_pp(&db, &mut st, MUGGER, 4);
    assert_eq!(st.player(me).pp_max, max0 + 1);
    assert_eq!(hand_count(&st, me, EVIDENCE), 0);

    // Evolve: Select an enemy follower and deal it 3 damage.
    let mut st = started(&db, 110_4311);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    let mug = put_field(&db, &mut st, me, MUGGER);
    evolve_now(&db, &mut st, mug, false);
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        7,
        "Evolve: Deal 3 → tank 10−3=7"
    );

    // Negative: no Clue → no ramp.
    let mut st = started(&db, 110_432);
    let me = PlayerId::A;
    st.player_mut(me).pp_max = 4;
    let max0 = st.player(me).pp_max;
    play_pp(&db, &mut st, MUGGER, 4);
    assert_eq!(st.player(me).pp_max, max0, "negative: no Clue → no ramp");
}

#[test]
fn smuggler_destroys_second_selected_and_adds_two_evidence() {
    let db = load_db();
    let mut st = started(&db, 110_428);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, PAD); // slot 0 — left alive
    put_field(&db, &mut st, opp, TANK); // slot 1 — first pick
    put_field(&db, &mut st, opp, TANK); // slot 2 — second pick
    give_pp(&mut st, me, 7, 7);
    let h = put_hand(&db, &mut st, me, SMUGGLER);
    play(&db, &mut st, h);
    // Select slot 1 then the remaining right tank (index 1 among remaining choices).
    choose(&db, &mut st, 1);
    choose(&db, &mut st, 1);
    let alive: Vec<String> = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .map(|c| c.card.as_str())
        .collect();
    assert_eq!(
        alive,
        vec![PAD.to_string()],
        "both selected tanks destroyed; pad remains"
    );
    assert_eq!(
        hand_count(&st, me, EVIDENCE),
        2,
        "Add 2 copies of Divine Evidence"
    );

    // Negative: fewer than 2 enemies — destroys the one present, still +2 Evidence.
    let mut st = started(&db, 110_4281);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, PAD);
    play_pp(&db, &mut st, SMUGGLER, 7);
    assert!(
        st.player(PlayerId::B).field.iter().flatten().count() == 0,
        "single target still destroyed"
    );
    assert_eq!(hand_count(&st, me, EVIDENCE), 2);
}

#[test]
fn jerva_x_zero_barrier_x_five_split_and_super_evolve() {
    let db = load_db();
    // X=0 with Barrier: Barrier stays, defense unchanged.
    let mut st = started(&db, 110_433);
    let opp = PlayerId::B;
    let t = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, t) {
        f.traits.barrier = Some(true);
    }
    assert_eq!(st.player(opp).leader_defense, st.player(opp).leader_max);
    play_pp(&db, &mut st, JERVA, 6);
    let f = st.field_inst(opp, t).unwrap();
    assert_eq!(f.defense, 10, "X=0: no defense change");
    assert_eq!(
        f.traits.barrier,
        Some(true),
        "X=0: Barrier stays (0 damage does not consume)"
    );

    // X=5 vs two tanks: oldest takes 5 → def 5; youngest leftover 0 → def 10.
    let mut st = started(&db, 110_434);
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, TANK);
    st.player_mut(opp).leader_defense = 15; // X = 20−15 = 5
    play_pp(&db, &mut st, JERVA, 6);
    let defs: Vec<i32> = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .map(|c| c.defense)
        .collect();
    assert_eq!(defs, vec![5, 10], "X=5 split: exact defenses oldest-first");

    // Super-Evolve: Storm + Ward.
    let mut st = started(&db, 110_4341);
    let me = PlayerId::A;
    let j = put_field(&db, &mut st, me, JERVA);
    evolve_now(&db, &mut st, j, true);
    let f = st.field_inst(me, j).unwrap();
    assert_eq!(f.traits.storm, Some(true), "Super-Evolve Storm");
    assert_eq!(f.traits.ward, Some(true), "Super-Evolve Ward");
}

#[test]
fn leizaut_x_zero_one_three_and_storm() {
    let db = load_db();
    // Storm keyword.
    let mut st = started(&db, 110_4349);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, LEIZAUT);
    assert_eq!(
        st.field_inst(me, slot).unwrap().traits.storm,
        Some(true),
        "Storm"
    );

    // X=0
    let mut st = started(&db, 110_435);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    st.player_mut(me).deck.clear();
    for id in [PAD, PAD, PAD, LEIZAUT] {
        push_deck(&db, &mut st, me, id);
    }
    play_pp(&db, &mut st, LEIZAUT, 10);
    assert_eq!(
        st.player(opp)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        10,
        "X=0: no damage loops"
    );
    assert!(st.player(me).deck.iter().all(|c| c.base_cost > 4));

    // X=1 — one tank, every random pick identical.
    let mut st = started(&db, 110_436);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    put_hand(&db, &mut st, me, EVIDENCE);
    st.player_mut(me).deck.clear();
    for _ in 0..3 {
        push_deck(&db, &mut st, me, PAD);
    }
    play_pp(&db, &mut st, LEIZAUT, 10);
    assert_eq!(
        st.player(opp)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        2,
        "X=1: one deal of 8 → 10−8=2"
    );

    // X=3 — three pads (2/2); each 8-damage hit kills one. All three die.
    let mut st = started(&db, 110_437);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    for _ in 0..3 {
        put_field(&db, &mut st, opp, PAD);
    }
    for _ in 0..3 {
        put_hand(&db, &mut st, me, EVIDENCE);
    }
    st.player_mut(me).deck.clear();
    play_pp(&db, &mut st, LEIZAUT, 10);
    assert_eq!(
        st.player(opp).field.iter().flatten().count(),
        0,
        "X=3: three deals of 8 clear three 2/2 pads"
    );
}
