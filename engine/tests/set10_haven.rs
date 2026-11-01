//! Set 10 Havencraft: Bunnies enter (empty hand / tied lowest), Destructive
//! Propagation Enhance, Parareligious Idol exact copy, Lapinette crest after SE.

use arena_engine::{apply, Action, AttackTarget, Phase, PlayerId, Slot};

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
const PAD: &str = "88001110";
const TANK: &str = "89500001";
const LAPINETTE_CREST: &str = "crest:11064110";

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
fn bunnies_empty_hand_still_evolves() {
    let db = load_db();
    let mut st = started(&db, 110_610);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    let slot = put_field(&db, &mut st, me, BUNNIES);
    // enter trigger fires on put_field? put_field may not fire enter — check.
    // put_field in common typically places without enter; use summon via spell.
    let mut st = started(&db, 110_611);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    play_pp(&db, &mut st, PROPAGATION, 5); // summons 2 Bunnies
    let b = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == BUNNIES)
        .expect("bunnies");
    assert!(
        b.evolved,
        "owner 2026-09-08: evolve is a separate sentence — evolves with empty hand"
    );
    let _ = slot;
}

#[test]
fn bunnies_tied_lowest_cost_banishes_one() {
    let db = load_db();
    let mut st = started(&db, 110_612);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    // Two cost-1 cards and one cost-3.
    put_hand(&db, &mut st, me, PAD);
    put_hand(&db, &mut st, me, PAD);
    put_hand(&db, &mut st, me, EVIDENCE); // cost 1 actually
    put_hand(&db, &mut st, me, TAMER); // cost 3
    let hand0 = st.player(me).hand.len();
    play_pp(&db, &mut st, "11062110", 3); // Tamer adds Bunnies to hand — messes hand
                                          // Cleaner: summon via Propagation with prepared hand.
    let mut st = started(&db, 110_613);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, PAD); // 1
    put_hand(&db, &mut st, me, PAD); // 1
    put_hand(&db, &mut st, me, TAMER); // 3
    let low = st.player(me).hand.iter().filter(|c| c.cost == 1).count();
    assert_eq!(low, 2);
    play_pp(&db, &mut st, PROPAGATION, 5); // 2 Bunnies enter one after another
                                           // Each Bunnies banishes one lowest-cost on entry.
    let ones = st.player(me).hand.iter().filter(|c| c.cost == 1).count();
    let threes = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == TAMER)
        .count();
    assert_eq!(ones, 0, "both lowest-cost pads banished across two entries");
    assert_eq!(threes, 1, "higher-cost Tamer remains");
    let _ = hand0;
}

#[test]
fn destructive_propagation_enhance_four_banishes_in_order() {
    let db = load_db();
    let mut st = started(&db, 110_614);
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
        "four Bunnies → four lowest-cost banishes"
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
fn idol_copies_banish_from_lapinette_and_own_evolve_not_on_opp_turn() {
    let db = load_db();
    // Lapinette banishes enemy → Idol copies.
    let mut st = started(&db, 110_615);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, IDOL);
    put_field(&db, &mut st, opp, TANK);
    // Buff the tank so exact copy is visible.
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.attack = 5;
        f.defense = 8;
    }
    play_pp(&db, &mut st, LAPINETTE, 6);
    assert!(
        st.player(me)
            .field
            .iter()
            .flatten()
            .any(|c| c.card.as_str() == TANK && c.attack == 5 && c.defense == 8),
        "Idol summons exact copy of Lapinette-banished tank"
    );

    // Idol Evolve banishes → copy.
    let mut st = started(&db, 110_616);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let idol = put_field(&db, &mut st, me, IDOL);
    put_field(&db, &mut st, opp, PAD);
    st.player_mut(me).turns_taken = 5;
    st.player_mut(me).ep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(idol),
            super_evolve: false,
        },
    )
    .expect("evolve");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert!(
        field_has(&st, me, PAD),
        "Idol Evolve banish → exact copy on our side"
    );

    // Negative: opponent's turn banish does not copy.
    let mut st = started(&db, 110_617);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, IDOL);
    put_field(&db, &mut st, me, PAD); // will be banished by B's Lapinette
    end_turn(&db, &mut st);
    play_pp(&db, &mut st, LAPINETTE, 6);
    assert!(
        !field_has(&st, me, PAD) || field_count(&st, me, PAD) == 0,
        "pad banished"
    );
    // Idol should not have copied onto A's field during B's turn — A may still have Idol only.
    assert_eq!(
        field_count(&st, me, PAD),
        0,
        "negative: Idol does not copy on opponent's turn"
    );
}

#[test]
fn lapinette_crest_after_super_evolve_two_summons() {
    let db = load_db();
    let mut st = started(&db, 110_618);
    let me = PlayerId::A;
    // Unlock SE.
    st.turn = 15;
    st.player_mut(me).turns_taken = 15;
    st.player_mut(me).sep = 1;
    put_field(&db, &mut st, PlayerId::B, PAD);
    play_pp(&db, &mut st, LAPINETTE, 6);
    let slot = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == LAPINETTE))
        .unwrap() as u8;
    // Evolve gains crest CD 1; Super-Evolve delays by 1 → CD 2.
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: true,
        },
    )
    .expect("super-evolve");
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(&db, &mut st, 0);
    }
    assert_eq!(
        crest_cd(&st, me, LAPINETTE_CREST),
        Some(2),
        "SE delays crest by 1 from the Evolve-granted CD 1"
    );
    // Clear Bunnies from Fanfare for a clean count.
    // Two start-of-turn summons as countdown ticks 2→1 then 1→0 (both fire).
    let before = field_count(&st, me, BUNNIES);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st); // A's next turn start: CD 2→1, startOfTurn summons
    let mid = field_count(&st, me, BUNNIES);
    assert!(mid > before, "first start-of-turn summon");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st); // second: CD 1→0, startOfTurn still resolves
    let after = field_count(&st, me, BUNNIES);
    assert!(
        after > mid,
        "Countdown-end turn still summons (rulebook §2); before={before} mid={mid} after={after}"
    );
    assert!(
        st.player(me).crests.iter().all(|c| c.id != LAPINETTE_CREST),
        "crest expired after CD hit 0"
    );
}

#[test]
fn combative_monk_strike_twice() {
    let db = load_db();
    let mut st = started(&db, 110_619);
    let me = PlayerId::A;
    let monk = put_field(&db, &mut st, me, MONK);
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
    // Strike grants Barrier before combat; Tank's 2 counter is absorbed → Barrier gone,
    // defense still at the post-Strike value (no net combat damage).
    assert_eq!(monk1.defense, def0 + 2, "Barrier absorbed combat damage");
    assert_ne!(
        monk1.traits.barrier,
        Some(true),
        "Barrier consumed by combat"
    );
    // Second attack same turn (attacksPerTurn: 2).
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
    assert_eq!(
        monk2.attack,
        atk0 + 4,
        "Strike fires on each of two attacks"
    );
    assert_eq!(
        monk2.defense,
        def0 + 4,
        "second Strike Barrier also absorbs combat"
    );
}

#[test]
fn haven_smoke_tamer_fruitless_rehab() {
    let db = load_db();
    let mut st = started(&db, 110_620);
    let me = PlayerId::A;
    play_pp(&db, &mut st, TAMER, 3);
    assert_eq!(hand_count(&st, me, BUNNIES), 1);

    play_pp(&db, &mut st, FRUITLESS, 3);
    // Mode 0 chosen by play_pp's choose(0): damage + crest, or Mischief-like.
    // choose(0) = mode 1 damage+crest OR we might get mode 2. Either is fine for smoke.
    assert!(
        st.player(me)
            .crests
            .iter()
            .any(|c| c.id == "crest:11063310")
            || field_has(&st, me, BUNNIES),
        "Fruitless activates a mode"
    );

    let mut st = started(&db, 110_621);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, PAD);
    play_pp(&db, &mut st, REHAB, 8);
    assert_eq!(field_count(&st, me, BUNNIES), 2);
}
