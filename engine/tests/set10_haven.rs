//! Set 10 Havencraft: Bunnies enter (empty hand / tied lowest / cost-modified),
//! Destructive Propagation Enhance, Parareligious Idol exact copy, Lapinette
//! crest, Fruitless Testimony modes, Monk / Rehab / Tamer / Refiner.

use arena_engine::event::{Event, ZoneLabel};
use arena_engine::{apply, legal_actions, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

const EVIDENCE: &str = "90001210";
const BUNNIES: &str = "90061140";
const REFINER: &str = "11061110";
const CULTIVATION: &str = "11061310";
const TAMER: &str = "11062110";
const FRUITLESS: &str = "11063310";
const ACCORDIONIST: &str = "11062120";
const PROPAGATION: &str = "11062310";
const LAPINETTE: &str = "11064110";
const IDOL: &str = "11064120";
const MONK: &str = "11061120";
const REHAB: &str = "11063110";
const FALCON: &str = "90061110";
const GHOST: &str = "90051130";
const PAD: &str = "88001110"; // 2/2
const TANK: &str = "89500001"; // 2/10
const LAPINETTE_CREST: &str = "crest:11064110";
const FRUITLESS_CREST: &str = "crest:11063310";

const SET10_HAVEN: &[&str] = &[
    REFINER,
    CULTIVATION,
    TAMER,
    FRUITLESS,
    ACCORDIONIST,
    PROPAGATION,
    LAPINETTE,
    IDOL,
    MONK,
    REHAB,
];

fn play_pp(db: &arena_engine::CardDb, st: &mut arena_engine::State, id: &str, pp: i32) {
    let max = st.player(st.active).pp_max.max(pp);
    give_pp(st, st.active, pp, max);
    let h = put_hand(db, st, st.active, id);
    play(db, st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn field_count(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    st.player(who)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == id)
        .count()
}

fn field_has(st: &arena_engine::State, who: PlayerId, id: &str) -> bool {
    field_count(st, who, id) > 0
}

fn hand_count(st: &arena_engine::State, who: PlayerId, id: &str) -> usize {
    st.player(who)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == id)
        .count()
}

fn crest_cd(st: &arena_engine::State, who: PlayerId, id: &str) -> Option<i32> {
    st.player(who)
        .crests
        .iter()
        .find(|c| c.id == id)
        .and_then(|c| c.countdown)
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
fn set10_haven_require_supported() {
    let db = load_db();
    for id in SET10_HAVEN {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
    db.require_supported(cid(BUNNIES))
        .unwrap_or_else(|e| panic!("{BUNNIES}: {e}"));
    db.crest(LAPINETTE_CREST)
        .unwrap_or_else(|e| panic!("{LAPINETTE_CREST}: {e}"));
}

#[test]
fn desolationist_refiner_last_words() {
    let db = load_db();
    let mut st = started(&db, 110_601);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let r = put_field(&db, &mut st, me, REFINER);
    let h0 = hand_count(&st, me, EVIDENCE);
    end_turn(&db, &mut st);
    let atk = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, atk) {
        f.flags.summoning_sick = false;
        f.attack = 5;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(r)),
        },
    )
    .expect("kill refiner");
    assert_eq!(
        hand_count(&st, me, EVIDENCE),
        h0 + 2,
        "Last Words: Add 2 Divine Evidence"
    );

    // Negative: banish does not fire LW.
    let mut st = started(&db, 110_602);
    put_field(&db, &mut st, PlayerId::B, REFINER);
    let h0 = hand_count(&st, PlayerId::B, EVIDENCE);
    play_pp(&db, &mut st, LAPINETTE, 6);
    assert_eq!(
        hand_count(&st, PlayerId::B, EVIDENCE),
        h0,
        "negative: banish does not fire Refiner Last Words"
    );
}

#[test]
fn solitary_cultivation_buff_and_barrier() {
    let db = load_db();
    let mut st = started(&db, 110_603);
    let me = PlayerId::A;
    let pad = put_field(&db, &mut st, me, PAD);
    let (a0, d0) = {
        let f = st.field_inst(me, pad).unwrap();
        (f.attack, f.defense)
    };
    play_pp(&db, &mut st, CULTIVATION, 3);
    let f = st.field_inst(me, pad).unwrap();
    assert_eq!((f.attack, f.defense), (a0 + 2, d0 + 2), "+2/+2");
    assert_eq!(f.traits.barrier, Some(true), "Barrier");

    // Negative: only the selected ally is buffed.
    let mut st = started(&db, 110_604);
    let me = PlayerId::A;
    let a = put_field(&db, &mut st, me, PAD);
    let b = put_field(&db, &mut st, me, TANK);
    let (ba0, bd0) = {
        let f = st.field_inst(me, b).unwrap();
        (f.attack, f.defense)
    };
    give_pp(&mut st, me, 3, 3);
    let h = put_hand(&db, &mut st, me, CULTIVATION);
    play(&db, &mut st, h);
    choose(&db, &mut st, 0); // select leftmost pad
    assert_eq!(
        st.field_inst(me, a).unwrap().traits.barrier,
        Some(true),
        "selected pad gets Barrier"
    );
    let f = st.field_inst(me, b).unwrap();
    assert_eq!(
        (f.attack, f.defense),
        (ba0, bd0),
        "negative: non-selected tank unchanged"
    );
    assert_ne!(f.traits.barrier, Some(true));
}

#[test]
fn winged_accordionist_fanfare_and_evolve() {
    let db = load_db();
    let mut st = started(&db, 110_605);
    let me = PlayerId::A;
    play_pp(&db, &mut st, ACCORDIONIST, 4);
    assert!(field_has(&st, me, FALCON), "Fanfare: Summon a Holy Falcon");
    assert!(field_has(&st, me, ACCORDIONIST));

    let mut st = started(&db, 110_606);
    let me = PlayerId::A;
    let acc = put_field(&db, &mut st, me, ACCORDIONIST);
    let ally = put_field(&db, &mut st, me, PAD);
    let (a0, d0) = {
        let f = st.field_inst(me, ally).unwrap();
        (f.attack, f.defense)
    };
    evolve_now(&db, &mut st, acc, false);
    let f = st.field_inst(me, ally).unwrap();
    assert_eq!(
        (f.attack, f.defense),
        (a0 + 1, d0 + 1),
        "Evolve: other allies +1/+1"
    );
    // Negative: Accordionist itself is not buffed by "other".
    let self_f = st.field_inst(me, acc).unwrap();
    assert_eq!(
        (self_f.attack, self_f.defense),
        (1 + 2, 1 + 2),
        "evolved stats only — no +1/+1 from own Evolve text"
    );
}

#[test]
fn fruitless_testimony_modes_crest_two_turns_and_with_lapinette() {
    let db = load_db();
    // Mode 1: Deal 4 + crest.
    let mut st = started(&db, 110_607);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, TANK);
    give_pp(&mut st, me, 3, 3);
    let h = put_hand(&db, &mut st, me, FRUITLESS);
    play(&db, &mut st, h);
    choose(&db, &mut st, 0); // mode 1
    assert_eq!(
        st.player(PlayerId::B)
            .field
            .iter()
            .flatten()
            .next()
            .unwrap()
            .defense,
        6,
        "mode 1: Deal 4 → tank 10−4=6"
    );
    assert_eq!(crest_cd(&st, me, FRUITLESS_CREST), Some(2));

    // Crest: restore 1 + Evidence on TWO turn starts (CD 2).
    st.player_mut(me).leader_defense = 15;
    let def0 = st.player(me).leader_defense;
    let ev0 = hand_count(&st, me, EVIDENCE);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st); // A's turn start 1: CD 2→1
    assert_eq!(st.player(me).leader_defense, def0 + 1);
    assert_eq!(hand_count(&st, me, EVIDENCE), ev0 + 1);
    assert_eq!(crest_cd(&st, me, FRUITLESS_CREST), Some(1));
    end_turn(&db, &mut st);
    end_turn(&db, &mut st); // A's turn start 2: CD 1→0, still fires
    assert_eq!(st.player(me).leader_defense, def0 + 2);
    assert_eq!(hand_count(&st, me, EVIDENCE), ev0 + 2);
    assert!(
        st.player(me)
            .crests
            .iter()
            .all(|c| c.id != FRUITLESS_CREST),
        "crest expired after two starts"
    );

    // Mode 2: Summon Bunnies.
    let mut st = started(&db, 110_608);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 3, 3);
    let h = put_hand(&db, &mut st, me, FRUITLESS);
    play(&db, &mut st, h);
    choose(&db, &mut st, 1); // mode 2
    assert_eq!(
        field_count(&st, me, BUNNIES),
        1,
        "mode 2: Summon a Bunnies of Desolation"
    );
    assert!(
        st.player(me)
            .crests
            .iter()
            .all(|c| c.id != FRUITLESS_CREST),
        "mode 2 does not gain crest"
    );

    // Lapinette crest + Fruitless crest both active together.
    let mut st = started(&db, 110_609);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_field(&db, &mut st, PlayerId::B, PAD);
    play_pp(&db, &mut st, LAPINETTE, 6);
    let lap = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == LAPINETTE))
        .unwrap() as u8;
    // Clear fanfare Bunnies for a clean crest-summon count.
    st.player_mut(me).field.iter_mut().for_each(|s| {
        if s.as_ref().is_some_and(|c| c.card.as_str() == BUNNIES) {
            *s = None;
        }
    });
    evolve_now(&db, &mut st, lap, false); // Lapinette crest CD1
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 3, 3);
    put_field(&db, &mut st, PlayerId::B, TANK);
    let h = put_hand(&db, &mut st, me, FRUITLESS);
    play(&db, &mut st, h);
    choose(&db, &mut st, 0); // Fruitless crest CD2
    assert!(
        st.player(me)
            .crests
            .iter()
            .any(|c| c.id == LAPINETTE_CREST)
            && st
                .player(me)
                .crests
                .iter()
                .any(|c| c.id == FRUITLESS_CREST),
        "both crests active together"
    );
    // Dual-crest SoT: Lapinette summons; Fruitless restores. Evidence add is
    // covered above (mode 1 CD2); Bunnies enter may banish the just-added Clue.
    st.player_mut(me).hand.clear();
    st.player_mut(me).leader_defense = 14;
    let def0 = st.player(me).leader_defense;
    let b0 = field_count(&st, me, BUNNIES);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st); // both start-of-turn crests fire
    assert_eq!(
        field_count(&st, me, BUNNIES),
        b0 + 1,
        "Lapinette crest summons Bunnies"
    );
    assert_eq!(
        st.player(me).leader_defense,
        def0 + 1,
        "Fruitless crest restores 1 alongside Lapinette"
    );
    assert!(
        st.player(me)
            .crests
            .iter()
            .all(|c| c.id != LAPINETTE_CREST),
        "Lapinette crest CD1 gone"
    );
    assert_eq!(crest_cd(&st, me, FRUITLESS_CREST), Some(1));
}

#[test]
fn parareligious_idol_copy_full_field_and_opponents_turn_negative() {
    let db = load_db();
    // Lapinette banishes enemy → Idol copies; copy keeps granted buffs.
    let mut st = started(&db, 110_615);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, IDOL);
    put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.attack = 5;
        f.defense = 8;
        f.max_defense = 8;
        f.traits.barrier = Some(true);
        f.traits.ward = Some(true);
    }
    play_pp(&db, &mut st, LAPINETTE, 6);
    let copy = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == TANK)
        .expect("Idol exact copy");
    assert_eq!((copy.attack, copy.defense), (5, 8), "copy keeps buffed stats");
    assert_eq!(
        copy.traits.barrier,
        Some(true),
        "copy keeps granted Barrier"
    );
    assert_eq!(copy.traits.ward, Some(true), "copy keeps granted Ward");

    // Full field after Lapinette lands: Idol cannot summon the copy.
    let mut st = started(&db, 110_6151);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, IDOL);
    for _ in 0..3 {
        put_field(&db, &mut st, me, PAD);
    }
    assert_eq!(st.player(me).field.iter().flatten().count(), 4);
    put_field(&db, &mut st, opp, TANK);
    play_pp(&db, &mut st, LAPINETTE, 6); // fills 5th slot; copy + Bunnies have no room
    assert_eq!(st.player(me).field.iter().flatten().count(), 5);
    assert_eq!(
        field_count(&st, me, TANK),
        0,
        "full field: exact copy summon fails"
    );

    // Idol Evolve banishes → copy.
    let mut st = started(&db, 110_616);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let idol = put_field(&db, &mut st, me, IDOL);
    put_field(&db, &mut st, opp, PAD);
    evolve_now(&db, &mut st, idol, false);
    assert!(
        field_has(&st, me, PAD),
        "Idol Evolve banish → exact copy on our side"
    );

    // Negative: B's Ghost dies (banished) on B's turn — Idol does not copy.
    let mut st = started(&db, 110_617);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, IDOL);
    end_turn(&db, &mut st); // B
    put_field(&db, &mut st, opp, GHOST);
    end_turn(&db, &mut st); // B's EoT banishes Ghost
    assert_eq!(
        field_count(&st, me, GHOST),
        0,
        "negative: B's Ghost banished on B's turn — Idol does not copy"
    );
}

#[test]
fn bunnies_empty_hand_still_evolves() {
    let db = load_db();
    let mut st = started(&db, 110_611);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    let ep0 = st.player(me).ep;
    play_pp(&db, &mut st, PROPAGATION, 5); // summons 2 Bunnies
    let bunnies: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == BUNNIES)
        .collect();
    assert_eq!(bunnies.len(), 2);
    assert!(
        bunnies.iter().all(|b| b.evolved),
        "owner 2026-09-08: evolve is a separate sentence — evolves with empty hand"
    );
    assert_eq!(st.player(me).ep, ep0, "enter-evolve spends no EP");

    // Negative: put_field does not fire enter — stays unevolved.
    let mut st = started(&db, 110_6111);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, BUNNIES);
    assert!(
        !st.field_inst(me, slot).unwrap().evolved,
        "negative: put_field skips enter — no evolve"
    );
}

#[test]
fn bunnies_lowest_current_cost_and_last_words() {
    let db = load_db();
    // Unique lowest each entry: PAD (1) then TAMER (3).
    let mut st = started(&db, 110_613);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, PAD);
    put_hand(&db, &mut st, me, TAMER);
    play_pp(&db, &mut st, PROPAGATION, 5);
    assert_eq!(hand_count(&st, me, PAD), 0, "first entry banishes PAD");
    assert_eq!(hand_count(&st, me, TAMER), 0, "second entry banishes TAMER");
    assert_eq!(st.player(me).hand.len(), 0);

    // Lowest by *current* cost: high-base card with cost reduced to 0 is banished.
    let mut st = started(&db, 110_6131);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, ACCORDIONIST); // base 4
    put_hand(&db, &mut st, me, PAD); // cost 1
    if let Some(c) = st
        .player_mut(me)
        .hand
        .iter_mut()
        .find(|c| c.card.as_str() == ACCORDIONIST)
    {
        c.cost = 0;
    }
    play_pp(&db, &mut st, PROPAGATION, 5); // 2 Bunnies
    assert_eq!(
        hand_count(&st, me, ACCORDIONIST),
        0,
        "cost-modified Accordionist (current 0) is lowest"
    );
    // Second bunny banishes the remaining PAD.
    assert_eq!(hand_count(&st, me, PAD), 0);

    // Last Words: Deal 2 to enemy leader.
    let mut st = started(&db, 110_6132);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).hand.clear();
    play_pp(&db, &mut st, PROPAGATION, 5);
    let b = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == BUNNIES))
        .unwrap() as u8;
    let def0 = st.player(opp).leader_defense;
    end_turn(&db, &mut st);
    let atk = put_field(&db, &mut st, opp, TANK);
    if let Some(f) = st.field_inst_mut(opp, atk) {
        f.flags.summoning_sick = false;
        f.attack = 5;
    }
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(b)),
        },
    )
    .expect("kill bunnies");
    assert_eq!(
        st.player(opp).leader_defense,
        def0 - 2,
        "Bunnies Last Words: Deal 2 to enemy leader"
    );
}

#[test]
fn destructive_propagation_enter_model_two_and_enhance_four() {
    let db = load_db();
    // Base: summon 2 — known hand, exact banishes in entry order.
    let mut st = started(&db, 110_614);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, EVIDENCE); // cost 1 — first banish
    put_hand(&db, &mut st, me, TAMER); // cost 3 — second banish
    let mut events = Vec::new();
    give_pp(&mut st, me, 5, 5);
    let h = put_hand(&db, &mut st, me, PROPAGATION);
    events.extend(apply(&db, &mut st, Action::Play { hand: h }).expect("play"));
    while matches!(st.phase, Phase::Choice { .. }) {
        events.extend(apply(&db, &mut st, Action::Choose(0)).expect("choose"));
    }
    let banished: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Banish {
                card,
                from: ZoneLabel::Hand,
            } => Some(card.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        banished,
        vec![EVIDENCE, TAMER],
        "entry order: first Bunnies banishes Evidence, second banishes Tamer"
    );
    assert_eq!(field_count(&st, me, BUNNIES), 2);

    // Enhance (8): summon 4 — four pads banished, fifth Tamer stays.
    let mut st = started(&db, 110_6141);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    for _ in 0..4 {
        put_hand(&db, &mut st, me, PAD);
    }
    put_hand(&db, &mut st, me, TAMER);
    give_pp(&mut st, me, 8, 8);
    let h = put_hand(&db, &mut st, me, PROPAGATION);
    play(&db, &mut st, h);
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert_eq!(field_count(&st, me, BUNNIES), 4);
    assert_eq!(
        st.player(me)
            .hand
            .iter()
            .filter(|c| c.card.as_str() == PAD)
            .count(),
        0,
        "four Bunnies → four lowest-cost pads banished"
    );
    assert_eq!(
        hand_count(&st, me, TAMER),
        1,
        "Enhance 4 banishes exactly 4 — fifth hand card (Tamer) stays"
    );
    assert!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .filter(|c| c.card.as_str() == BUNNIES)
            .all(|c| c.evolved),
        "each Bunnies evolved"
    );
}

#[test]
fn lapinette_normal_evolve_exactly_one_summon() {
    let db = load_db();
    let mut st = started(&db, 110_618);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, PAD);
    play_pp(&db, &mut st, LAPINETTE, 6);
    let slot = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == LAPINETTE))
        .unwrap() as u8;
    // Remove Fanfare Bunnies so crest summon is exact.
    st.player_mut(me).field.iter_mut().for_each(|s| {
        if s.as_ref().is_some_and(|c| c.card.as_str() == BUNNIES) {
            *s = None;
        }
    });
    evolve_now(&db, &mut st, slot, false);
    assert_eq!(
        crest_cd(&st, me, LAPINETTE_CREST),
        Some(1),
        "normal Evolve: crest CD 1"
    );
    let before = field_count(&st, me, BUNNIES);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st); // A's next turn start
    assert_eq!(
        field_count(&st, me, BUNNIES),
        before + 1,
        "exactly one summon on one turn start"
    );
    assert!(
        st.player(me)
            .crests
            .iter()
            .all(|c| c.id != LAPINETTE_CREST),
        "crest gone after CD1 turn start"
    );

    // Negative control: without Evolve, no crest summon next turn.
    let mut st = started(&db, 110_6181);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, PAD);
    play_pp(&db, &mut st, LAPINETTE, 6);
    st.player_mut(me).field.iter_mut().for_each(|s| {
        if s.as_ref().is_some_and(|c| c.card.as_str() == BUNNIES) {
            *s = None;
        }
    });
    let before = field_count(&st, me, BUNNIES);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert_eq!(
        field_count(&st, me, BUNNIES),
        before,
        "negative: no Evolve → no crest summon"
    );
}

#[test]
fn combative_monk_two_attacks_third_rejected_rush() {
    let db = load_db();
    let mut st = started(&db, 110_619);
    let me = PlayerId::A;
    let monk = put_field(&db, &mut st, me, MONK);
    assert_eq!(
        st.field_inst(me, monk).unwrap().traits.rush,
        Some(true),
        "Rush"
    );
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    put_field(&db, &mut st, PlayerId::B, TANK);
    if let Some(f) = st.field_inst_mut(me, monk) {
        f.flags.summoning_sick = false;
    }
    let atk0 = st.field_inst(me, monk).unwrap().attack;
    let def0 = st.field_inst(me, monk).unwrap().defense;
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(monk),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .expect("attack 1");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    let monk1 = st.field_inst(me, monk).unwrap();
    assert_eq!(monk1.attack, atk0 + 2, "Strike on first attack");
    assert_eq!(monk1.defense, def0 + 2, "Barrier absorbed combat damage");
    assert_ne!(
        monk1.traits.barrier,
        Some(true),
        "Barrier consumed by combat"
    );
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(monk),
            target: AttackTarget::Slot(Slot(1)),
        },
    )
    .expect("attack 2");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    let monk2 = st.field_inst(me, monk).unwrap();
    assert_eq!(monk2.attack, atk0 + 4, "Strike fires on each of two attacks");
    assert_eq!(monk2.defense, def0 + 4, "second Strike Barrier absorbs");

    // Third attack rejected.
    let third = apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(monk),
            target: AttackTarget::Slot(Slot(2)),
        },
    );
    assert!(third.is_err(), "third attack rejected");
    let legal = legal_actions(&db, &st);
    assert!(
        !legal.iter().any(|a| matches!(
            a,
            Action::Attack {
                attacker: Slot(s),
                ..
            } if *s == monk
        )),
        "Monk not in legal attackers after two attacks"
    );
}

#[test]
fn desolationist_rehabilitator_banish_not_destroy_and_super_evolve() {
    let db = load_db();
    let mut st = started(&db, 110_620);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    // Give tank Last Words-like tracking: cemetery shadows would rise on destroy.
    let sh0 = st.player(opp).shadows;
    st.player_mut(me).hand.clear();
    play_pp(&db, &mut st, REHAB, 8);
    assert_eq!(
        st.player(opp).field.iter().flatten().count(),
        0,
        "enemy removed"
    );
    assert_eq!(
        st.player(opp).shadows,
        sh0,
        "banished not destroyed — no shadow for opponent"
    );
    assert_eq!(field_count(&st, me, BUNNIES), 2);

    // Super-Evolve: Draw 2 + Restore 2.
    let mut st = started(&db, 110_621);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    for _ in 0..3 {
        let card = db.card(cid(PAD)).unwrap();
        let id = st.alloc_id();
        st.player_mut(me)
            .deck
            .push(arena_engine::CardInstance::from_card(card, id));
    }
    let r = put_field(&db, &mut st, me, REHAB);
    st.player_mut(me).leader_defense = 15;
    let def0 = st.player(me).leader_defense;
    let hand0 = st.player(me).hand.len();
    evolve_now(&db, &mut st, r, true);
    assert_eq!(st.player(me).hand.len(), hand0 + 2, "SE: Draw 2");
    assert_eq!(st.player(me).leader_defense, def0 + 2, "SE: Restore 2");
}

#[test]
fn desolationist_tamer_evolve_recovers_pp() {
    let db = load_db();
    let mut st = started(&db, 110_622);
    let me = PlayerId::A;
    play_pp(&db, &mut st, TAMER, 3);
    assert_eq!(hand_count(&st, me, BUNNIES), 1, "Fanfare: Add Bunnies");

    let mut st = started(&db, 110_623);
    let me = PlayerId::A;
    let t = put_field(&db, &mut st, me, TAMER);
    give_pp(&mut st, me, 0, 3);
    evolve_now(&db, &mut st, t, false);
    assert_eq!(st.player(me).pp, 1, "Evolve: Recover 1 play point");

    // Negative: without Evolve, PP stays 0.
    let mut st = started(&db, 110_624);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, TAMER);
    give_pp(&mut st, me, 0, 3);
    assert_eq!(st.player(me).pp, 0, "negative: no Evolve → no recover");
}
