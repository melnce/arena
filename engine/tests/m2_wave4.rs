//! M2 wave 4: Royal (Swordcraft) and Ramp Dragon variants.
//! Silent constructs: Enhance add-vs-replace + bindings, Mars batch
//! `ally_follower_enter`, Barbaros countdown advance, Severed Ties `costEq`,
//! Beltezore `attacksPerTurn: 3`, Yidmetra Faith on Enhanced play, Roar X /
//! Lazing Overflow.

use arena_engine::{apply, legal_actions, snapshot, Action, AttackTarget, Phase, PlayerId, Slot};

mod common;
use common::*;

const YIDMETRA: &str = "10624120";
const DEPTHS_SWORD: &str = "90024320";
const ZETA: &str = "10424110";
const SPLENDOR: &str = "10523310";
const GOLD: &str = "90021350";
const RUTHLESS: &str = "10623310";
const LAGE: &str = "10923310";
const FLAG: &str = "90021210";
const MARS: &str = "10824120";
const KNIGHT: &str = "90021110";
const BARBAROS: &str = "10924110";
const TIES: &str = "10922310";
const BELTEZORE: &str = "10924120";
const ROAR: &str = "10542310";
const LAZING: &str = "10742310";
const FIGHTER: &str = "10001110";
const VANILLA: &str = "88001110";
const TANK: &str = "88001320";

fn require_ok(db: &arena_engine::CardDb, ids: &[&str]) {
    for id in ids {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

#[test]
fn require_supported_royal_closure() {
    let db = load_db();
    require_ok(
        &db,
        &[
            "10021110", "10424110", "10523310", "10524120", "10623310", "10624120", "10722310",
            "10823310", "10824120", "10921110", "10922110", "10922310", "10923110", "10923310",
            "10924110", "10924120", "90021110", "90021120", "90021210", "90021310", "90021320",
            "90021330", "90021340", "90021350", "90024320",
        ],
    );
}

#[test]
fn require_supported_ramp_claywies_closure() {
    let db = load_db();
    require_ok(
        &db,
        &[
            "10042310", "10403120", "10444120", "10503310", "10542310", "10543310", "10544110",
            "10644110", "10644120", "10741110", "10742310", "10744110", "10804110", "10842120",
            "10844120", "10944120",
        ],
    );
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
}

/// Yidmetra's Faith end to end: crest from the starting deck, +1 per
/// Enhanced play, the 5-point spend, the granted +1/+1 on Enhanced play.
#[test]
fn yidmetra_faith_enhanced_play_end_to_end() {
    let db = load_db();
    let mut st = started_decks(&db, 400, &[YIDMETRA], &[VANILLA]);
    let me = PlayerId::A;
    assert!(
        st.player(me)
            .crests
            .iter()
            .any(|c| c.id == "faith:10624120" && c.countdown.is_none()),
        "Faith: Yidmetra from the starting deck"
    );
    assert_eq!(st.player(me).faith, 0);

    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, FIGHTER);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(me).faith,
        1,
        "Enhance (4) Indomitable Fighter is an Enhanced play"
    );

    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SPLENDOR);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(me).faith,
        1,
        "Splendor at printed cost 3 is not Enhanced"
    );

    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SPLENDOR);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(me).faith,
        2,
        "Splendor Enhance (5) increments Faith"
    );

    // Evolve Yidmetra: pay 5 to grant the +1/+1-on-Enhanced-play ability.
    st.player_mut(me).faith = 5;
    st.player_mut(me).hand.clear();
    give_pp(&mut st, me, 2, 2);
    play_id(&db, &mut st, me, YIDMETRA);
    drain_choice(&db, &mut st);
    let slot = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == YIDMETRA))
        .expect("Yidmetra on field") as u8;
    grant_evolve(&db, &mut st, slot);
    assert!(
        hand_has(&st, me, DEPTHS_SWORD),
        "Fanfare still added Depths before evolve"
    );
    assert_eq!(
        st.player(me).faith,
        0,
        "Yidmetra Faith does not tick on evolve; pay 5 spends the value"
    );

    // Granted ability: next Enhanced play buffs allied followers +1/+1.
    let atk = st.field_inst(me, slot).unwrap().attack;
    let def = st.field_inst(me, slot).unwrap().defense;
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, FIGHTER);
    drain_choice(&db, &mut st);
    let y = st.field_inst(me, slot).expect("Yidmetra survives");
    assert_eq!(y.attack, atk + 1, "granted ability +1/+1 on Enhanced play");
    assert_eq!(y.defense, def + 1);
}

#[test]
fn yidmetra_faith_skips_accelerate_play() {
    // Accelerate/Crystallize are not Enhanced (rulebook Enhance).
    let db = load_db();
    let mut st = started_decks(&db, 401, &[YIDMETRA], &[VANILLA]);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    // Lumiore Accelerate (3): not an Enhance.
    play_id(&db, &mut st, me, "10844120");
    drain_choice(&db, &mut st);
    assert_eq!(st.player(me).faith, 0, "Accelerate is not an Enhanced play");
}

#[test]
fn zeta_enhance_adds_to_base_and_reads_binding() {
    let db = load_db();
    let mut st = started(&db, 410);
    let me = PlayerId::A;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, ZETA);
    drain_choice(&db, &mut st);
    let field: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == ZETA)
        .collect();
    assert_eq!(field.len(), 2, "Fanfare summons a copy");
    let played = field.iter().find(|c| c.traits.storm == Some(true));
    let summoned = field.iter().find(|c| c.traits.storm != Some(true));
    assert!(played.is_some(), "Enhance gives this follower Storm");
    let summoned = summoned.expect("summoned copy");
    assert_eq!(
        summoned.traits.bane,
        Some(true),
        "Enhance reads the Fanfare binding and gives the copy Bane"
    );
    assert_eq!(played.unwrap().traits.rush, Some(true));
}

#[test]
fn zeta_without_enhance_skips_bane_and_storm() {
    let db = load_db();
    let mut st = started(&db, 411);
    let me = PlayerId::A;
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, ZETA);
    drain_choice(&db, &mut st);
    let field: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == ZETA)
        .collect();
    assert_eq!(field.len(), 2);
    assert!(field.iter().all(|c| c.traits.storm != Some(true)));
    assert!(field.iter().all(|c| c.traits.bane != Some(true)));
}

#[test]
fn splendor_enhance_replaces_base() {
    let db = load_db();
    let mut st = started(&db, 412);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SPLENDOR);
    drain_choice(&db, &mut st);
    let n = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == GOLD)
        .count();
    assert_eq!(n, 2, "printed form adds 2");

    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SPLENDOR);
    drain_choice(&db, &mut st);
    let n = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == GOLD)
        .count();
    assert_eq!(n, 4, "Enhance replacesBase adds 4, not 2+4");
}

#[test]
fn ruthless_eld_enhance_activates_all_modes() {
    let db = load_db();
    let mut st = started(&db, 413);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    let before = st.player(opp).field[0].as_ref().unwrap().defense;
    let hand_before = st.player(me).hand.len();
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    let hb = st.player(me).hand.len();
    let _ = hand_before;
    play_id(&db, &mut st, me, RUTHLESS);
    assert!(
        matches!(st.phase, Phase::Main),
        "pick: all optionsFrom: fanfare does not pause for a mode choice"
    );
    assert_eq!(st.player(me).hand.len(), hb + 1, "mode 1 draws");
    assert_eq!(
        st.player(opp).field[0].as_ref().unwrap().defense,
        before - 3,
        "mode 2 deals 3 to a random enemy follower"
    );
}

#[test]
fn lage_dor_enhance_replaces_both_sentences() {
    let db = load_db();
    let mut st = started(&db, 414);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    let before = st.player(opp).field[0].as_ref().unwrap().defense;
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, LAGE);
    drain_choice(&db, &mut st);
    let flags = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == FLAG)
        .count();
    assert_eq!(flags, 2, "Enhance summons 2 Flags, not 1+2");
    assert_eq!(
        st.player(opp).field[0].as_ref().unwrap().defense,
        before - 4,
        "Enhance deals 4, not 2+4"
    );
}

#[test]
fn mars_batch_summon_fires_officer_enter_per_knight() {
    let db = load_db();
    let mut st = started(&db, 420);
    let me = PlayerId::A;
    give_pp(&mut st, me, 8, 8);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, MARS);
    drain_choice(&db, &mut st);
    let knights: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == KNIGHT)
        .collect();
    assert_eq!(knights.len(), 3);
    assert!(
        knights
            .iter()
            .all(|k| k.attack == 3 && k.traits.rush == Some(true)),
        "each Knight gets +2/+0 and Rush from Mars's when"
    );
    let mars = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == MARS)
        .expect("Mars");
    assert_eq!(
        mars.attack, 4,
        "three Officer enters: Mars 1 + 3 = 4 attack"
    );
}

#[test]
fn barbaros_countdown_delta_includes_summoned_flag() {
    let db = load_db();
    let mut st = started(&db, 430);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, FLAG);
    assert_eq!(st.player(me).field[0].as_ref().unwrap().countdown, Some(7));
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, BARBAROS);
    drain_choice(&db, &mut st);
    let flags: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == FLAG)
        .collect();
    assert_eq!(flags.len(), 2, "existing Flag + summoned Flag");
    assert!(
        flags.iter().all(|f| f.countdown == Some(2)),
        "both advanced 7 → 2: {flags:?}"
    );
}

#[test]
fn barbaros_advance_expires_flag_at_zero() {
    let db = load_db();
    let mut st = started(&db, 431);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let slot = put_field(&db, &mut st, me, FLAG);
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.countdown = Some(5);
    }
    let before = st.player(opp).leader_defense;
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, BARBAROS);
    drain_choice(&db, &mut st);
    assert!(
        !st.player(me)
            .field
            .iter()
            .flatten()
            .any(|c| c.card.as_str() == FLAG && c.countdown == Some(5)),
        "Flag at 5 reaches 0 and is destroyed"
    );
    let surviving = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == FLAG)
        .count();
    assert_eq!(surviving, 1, "the just-summoned Flag survives");
    let left = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == FLAG)
        .expect("summoned Flag");
    assert_eq!(
        left.countdown,
        Some(2),
        "new Flag 7 → 2 even when an older Flag expired and compacted"
    );
    assert_eq!(
        st.player(opp).leader_defense,
        before - 2,
        "expired Flag Last Words: 2 to the enemy leader"
    );
}

#[test]
fn barbaros_advance_expires_flag_at_four_and_keeps_new_at_two() {
    // Oracle shape: a Flag already at 4 expires (4 − 5), the summoned copy
    // must still be found after compact and land at 2.
    let db = load_db();
    let mut st = started(&db, 432);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let slot = put_field(&db, &mut st, me, FLAG);
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.countdown = Some(4);
    }
    let before = st.player(opp).leader_defense;
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, BARBAROS);
    drain_choice(&db, &mut st);
    let flags: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == FLAG)
        .collect();
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].countdown, Some(2));
    assert_eq!(st.player(opp).leader_defense, before - 2);
}

#[test]
fn severed_ties_costeq_on_played_instance() {
    let db = load_db();
    let mut st = started(&db, 440);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, TIES);
    drain_choice(&db, &mut st);
    let copies: Vec<_> = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == TIES)
        .collect();
    assert_eq!(copies.len(), 1, "cost 3 adds a copy");
    assert_eq!(copies[0].cost, 1, "chained copy cost set to 1");

    // The cost-1 copy does not chain.
    give_pp(&mut st, me, 1, 1);
    put_field(&db, &mut st, opp, TANK);
    let pos = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == TIES && c.cost == 1)
        .unwrap();
    play(&db, &mut st, pos as u8);
    drain_choice(&db, &mut st);
    let left = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == TIES)
        .count();
    assert_eq!(left, 0, "cost-1 copy does not add another");
}

#[test]
fn severed_ties_cost_modified_first_copy_does_not_chain() {
    let db = load_db();
    let mut st = started(&db, 441);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, TANK);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, TIES);
    st.player_mut(me).hand[h as usize].cost = 2;
    give_pp(&mut st, me, 2, 2);
    play(&db, &mut st, h);
    drain_choice(&db, &mut st);
    assert!(
        !st.player(me).hand.iter().any(|c| c.card.as_str() == TIES),
        "a cost-modified first copy (cost 2) does not chain"
    );
}

#[test]
fn beltezore_attacks_per_turn_three_with_storm() {
    let db = load_db();
    let mut st = started(&db, 450);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, BELTEZORE);
    drain_choice(&db, &mut st);
    let slot = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == BELTEZORE))
        .unwrap() as u8;
    let f = st.field_inst(me, slot).unwrap();
    assert_eq!(f.flags.attacks_left, 3, "printed attacksPerTurn: 3");
    assert_eq!(f.traits.storm, Some(true));
    let legal = legal_actions(&db, &st);
    let attacks = legal
        .iter()
        .filter(|a| matches!(a, Action::Attack { attacker, .. } if attacker.0 == slot))
        .count();
    assert!(attacks > 0, "Storm lets it attack the turn it is played");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .expect("attack 1");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .expect("attack 2");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .expect("attack 3");
    assert_eq!(st.field_inst(me, slot).unwrap().flags.attacks_left, 0);
}

#[test]
fn roar_of_prominence_x_read_once_before_damage() {
    let db = load_db();
    let mut st = started(&db, 460);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, VANILLA);
    put_field(&db, &mut st, opp, TANK);
    put_field(&db, &mut st, opp, VANILLA);
    // 3 followers on the field. X = 3, read once; all take 3 (not a shrinking X).
    let tank_def = st.player(opp).field[0].as_ref().unwrap().defense;
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, ROAR);
    drain_choice(&db, &mut st);
    assert!(
        !field_has(&st, me, VANILLA),
        "allied 1-def vanilla dies to X=3"
    );
    assert!(!field_has(&st, opp, VANILLA), "enemy vanilla dies to X=3");
    let tank = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == TANK)
        .expect("tank survives");
    assert_eq!(
        tank.defense,
        tank_def - 3,
        "X is 3 for every follower, not re-counted after deaths"
    );
}

#[test]
fn lazing_flame_overflow_draws() {
    let db = load_db();
    let mut st = started(&db, 470);
    let me = PlayerId::A;
    st.player_mut(me).pp_max = 6;
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 3, 6);
    st.player_mut(me).hand.clear();
    let before = st.player(me).hand.len();
    play_id(&db, &mut st, me, LAZING);
    drain_choice(&db, &mut st);
    assert_eq!(st.player(me).leader_defense, 13, "restore 3 to the leader");
    assert_eq!(st.player(me).hand.len(), before, "max PP 6 is not Overflow");

    st.player_mut(me).pp_max = 7;
    st.player_mut(me).leader_defense = 10;
    give_pp(&mut st, me, 3, 7);
    st.player_mut(me).hand.clear();
    let before = st.player(me).hand.len();
    play_id(&db, &mut st, me, LAZING);
    drain_choice(&db, &mut st);
    assert_eq!(st.player(me).leader_defense, 13);
    assert_eq!(
        st.player(me).hand.len(),
        before + 1,
        "max PP ≥ 7 is Overflow: draw 1"
    );
}

#[test]
fn snapshot_yidmetra_faith_present() {
    let db = load_db();
    let st = started_decks(&db, 480, &[YIDMETRA], &[VANILLA]);
    let snap = snapshot(&st);
    assert!(snap
        .players
        .a
        .crests
        .iter()
        .any(|c| c.id == "faith:10624120"));
}
