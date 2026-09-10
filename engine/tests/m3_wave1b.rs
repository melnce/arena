//! M3 wave 1b: remaining Swordcraft + Forestcraft constructs.
//! Silent no-ops are defects. Each named fixture is listed in the PR body.

use arena_engine::state::CrestInstance;
use arena_engine::{
    apply, legal_actions, new_game, policy_rng, snapshot_json, Action, AttackTarget, CardDb,
    CardInstance, First, GameConfig, GameRng, OpeningHands, Phase, PlayerId, Slot,
};

mod common;
use common::*;

const OKITA: &str = "10823110";
const KOU: &str = "10411110";
const STAFF: &str = "10522110";
const OLUON: &str = "10524110";
const GOLDEN: &str = "10423110";
const SEOFON: &str = "10424120";
const MAJESTIC: &str = "10622310";
const SHARED: &str = "10822310";
const CHLOE: &str = "10412110";
const WOLFRAUD: &str = "10514110";
const MARLONE: &str = "10811110";
const TIA: &str = "10814120";
const EVE: &str = "90014110";
const TRAP: &str = "10911210";
const MANAMEL: &str = "10411120";
const CURIOSITY: &str = "10813310";
const MEASURED: &str = "10721310";
const ALFHEIMR: &str = "10413310";
const TAMER: &str = "10011110";
const FAIRY: &str = "90011110";
const SPRING: &str = "90011120";
const VANILLA: &str = "88001110";
const TANK: &str = "88001320";
const COPY_HAND: &str = "88001830";
const RUFLET: &str = "10812110";
const KNIGHT: &str = "90021110";
const SUMMON3: &str = "88001860";
const SKIPPER: &str = "10513110";
const WATCHER: &str = "88001840";
const BUFF_ENEMY: &str = "88001850";
const SELECT_DESTROY: &str = "88001300";

fn require_ok(db: &CardDb, ids: &[&str]) {
    for id in ids {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

fn drain_choice(db: &CardDb, st: &mut arena_engine::State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

fn set_round(st: &mut arena_engine::State, who: PlayerId, n: u32) {
    st.player_mut(who).turns_taken = n;
    st.turn = n;
    give_pp(st, who, n as i32, n as i32);
}

fn put_deck(db: &CardDb, st: &mut arena_engine::State, who: PlayerId, id: &str) {
    let card = db.card(cid(id)).expect("card");
    let inst = CardInstance::from_card(card, st.alloc_id());
    st.player_mut(who).deck.push(inst);
}

fn attack(db: &CardDb, st: &mut arena_engine::State, attacker: u8, target: AttackTarget) {
    apply(
        db,
        st,
        Action::Attack {
            attacker: Slot(attacker),
            target,
        },
    )
    .expect("attack");
}

fn emit_and_replay_deck(db: &CardDb, seed: u64, deck_path: &str) {
    let decks = load_deck_file(deck_path);
    let mut live = new_game(
        db,
        GameConfig {
            seed,
            deck_a: decks.clone(),
            deck_b: decks.clone(),
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let opening = OpeningHands {
        a: live
            .player(PlayerId::A)
            .hand
            .iter()
            .map(|c| c.card)
            .collect(),
        b: live
            .player(PlayerId::B)
            .hand
            .iter()
            .map(|c| c.card)
            .collect(),
    };
    let mut policy = policy_rng(seed);
    let mut recs = Vec::new();
    let mut i = 0u32;
    while live.winner.is_none() && !matches!(live.phase, Phase::Terminal) && i < 800 {
        let legal = legal_actions(db, &live);
        if legal.is_empty() {
            break;
        }
        let idx = policy.gen_range(legal.len() as u32) as usize;
        let action = legal[idx].clone();
        let neu = arena_engine::to_neutral(&live, &action);
        apply(db, &mut live, action).unwrap();
        recs.push((neu, live.picks.clone(), snapshot_json(&live)));
        i += 1;
    }

    let mut replay = new_game(
        db,
        GameConfig {
            seed,
            deck_a: decks.clone(),
            deck_b: decks,
            first: First::A,
            opening_hands: Some(opening),
        },
    )
    .unwrap();
    for (i, (neu, picks, snap)) in recs.iter().enumerate() {
        replay.rng = GameRng::scripted(picks.clone(), seed);
        let act = arena_engine::from_neutral(&replay, neu).expect("from_neutral");
        apply(db, &mut replay, act).unwrap_or_else(|e| panic!("replay seed={seed} i={i}: {e}"));
        let got = snapshot_json(&replay);
        if let Some((path, a, b)) = arena_engine::replay_state_diff(&got, snap) {
            panic!("seed {seed} i={i} {path}: arena={a} trace={b}");
        }
    }
}

#[test]
fn require_supported_sword_forest_wave1b() {
    let db = load_db();
    require_ok(
        &db,
        &[
            // Forestcraft (new + named tokens)
            "10411110", "10411120", "10411310", "10412110", "10412120", "10412310", "10413110",
            "10413310", "10414110", "10414120", "10511110", "10511120", "10511310", "10512110",
            "10512120", "10512310", "10513110", "10514110", "10611110", "10611120", "10611310",
            "10612110", "10612120", "10612310", "10613110", "10613310", "10614110", "10711110",
            "10711120", "10711310", "10712120", "10712310", "10713110", "10713310", "10811110",
            "10811120", "10812120", "10812310", "10813110", "10813310", "10814120", "10911120",
            "10911210", "10912120", "10912310", "10914120", "90011110", "90011120", "90011310",
            "90014110", // Swordcraft (new + named tokens)
            "10421110", "10421120", "10421130", "10422110", "10422120", "10422130", "10423110",
            "10423310", "10424120", "10521110", "10521120", "10521310", "10522110", "10522120",
            "10522310", "10523110", "10524110", "10621110", "10621120", "10621310", "10622110",
            "10622120", "10622310", "10623110", "10721110", "10721120", "10721310", "10722110",
            "10722120", "10723110", "10723310", "10724120", "10821110", "10821120", "10821130",
            "10822120", "10822310", "10823110", "10824110", "10921120", "10921310", "10922120",
            "90021110", "90021120", "90021130", "90022110", "90024330",
        ],
    );
}

/// Okita: one Follower Strike unevolved, three when evolved; both before combat.
#[test]
fn okita_strikes_once_unevolved_thrice_evolved_before_combat() {
    let db = load_db();
    let mut st = started(&db, 500);
    let me = PlayerId::A;
    let opp = PlayerId::B;

    // Unevolved 2/1 vs 3/3: Strike 3 kills before combat, Okita takes no return.
    let att = put_field(&db, &mut st, me, OKITA);
    let def = put_field(&db, &mut st, opp, VANILLA);
    if let Some(f) = st.field_inst_mut(opp, def) {
        f.attack = 3;
        f.defense = 3;
        f.max_defense = 3;
    }
    attack(&db, &mut st, att, AttackTarget::Slot(Slot(def)));
    assert!(
        field_has(&st, me, OKITA),
        "unevolved Strike kills the 3-def before combat, Okita lives"
    );
    assert!(
        !field_has(&st, opp, VANILLA),
        "one Strike of 3 destroys the 3-def defender"
    );

    // Evolved: three Strikes of 3 then combat 4 into a 10/10.
    st.player_mut(me).field = Default::default();
    st.player_mut(opp).field = Default::default();
    let att = put_field(&db, &mut st, me, OKITA);
    if let Some(f) = st.field_inst_mut(me, att) {
        f.evolved = true;
        f.attack = 4;
        f.defense = 3;
        f.max_defense = 3;
        f.flags.summoning_sick = false;
    }
    let def = put_field(&db, &mut st, opp, TANK);
    attack(&db, &mut st, att, AttackTarget::Slot(Slot(def)));
    let survived = st.field_inst(opp, def).map(|c| c.defense);
    assert!(
        survived.is_none() || survived == Some(10 - 9 - 4),
        "evolved does 3+3+3 then combat 4, not a single 3; got {survived:?}"
    );
}

#[test]
fn kou_and_you_strike_restores_all_allies() {
    let db = load_db();
    let mut st = started(&db, 501);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let att = put_field(&db, &mut st, me, KOU);
    let ally = put_field(&db, &mut st, me, VANILLA);
    if let Some(f) = st.field_inst_mut(me, ally) {
        f.defense = 1;
        f.max_defense = 4;
    }
    st.player_mut(me).leader_defense = 10;
    st.player_mut(me).leader_max = 20;
    put_field(&db, &mut st, opp, VANILLA);
    attack(&db, &mut st, att, AttackTarget::Slot(Slot(0)));
    assert_eq!(
        st.player(me).leader_defense,
        13,
        "Strike restores the leader"
    );
    let ad = st.field_inst(me, ally).expect("ally").defense;
    assert_eq!(ad, 4, "Strike restores allied followers");
}

#[test]
fn swift_staffmaster_cost_3_expires_eot() {
    let db = load_db();
    let mut st = started(&db, 502);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, STAFF);
    give_pp(&mut st, me, 6, 6);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, STAFF);
    let drawn = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == STAFF)
        .expect("Fanfare drew the deck copy");
    assert_eq!(drawn.cost, 3, "when you draw this card, set cost to 3");
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    let after = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == STAFF)
        .expect("copy still in hand");
    assert_eq!(after.cost, 6, "untilEndOfTurn cost reverts at EOT");
}

#[test]
fn oluon_evolved_hits_three_random_excluding_self() {
    let db = load_db();
    let mut st = started(&db, 503);
    let me = PlayerId::A;
    let slot = put_field(&db, &mut st, me, OLUON);
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.evolved = true;
        f.attack = 9;
        f.defense = 9;
        f.max_defense = 9;
    }
    let before_a = st.player(me).leader_defense;
    let before_b = st.player(PlayerId::B).leader_defense;
    end_turn(&db, &mut st);
    let oluon = st.field_inst(me, slot).expect("Oluon");
    assert_eq!(oluon.defense, 9, "Oluon is excluded from the random pool");
    let dealt = (before_a - st.player(me).leader_defense)
        + (before_b - st.player(PlayerId::B).leader_defense);
    assert_eq!(
        dealt, 21,
        "three 7-damage hits, leaders included, self excluded"
    );
}

#[test]
fn golden_knight_mode1_super_evolve_spends_no_sep() {
    let db = load_db();
    let mut st = started(&db, 504);
    let me = PlayerId::A;
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    let sep = st.player(me).sep;
    play_id(&db, &mut st, me, GOLDEN);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    let g = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == GOLDEN)
        .expect("Golden Knight");
    assert!(g.super_evolved, "mode 1 super-evolves");
    assert!(g.evolved);
    assert_eq!(st.player(me).sep, sep, "effect-granted SE spends no SEP");
    assert!(
        !st.player(me).evolved_this_turn,
        "granted evolve leaves evolved_this_turn untouched"
    );
}

#[test]
fn seofon_skybound_and_super_skybound_tiers() {
    let db = load_db();
    let me = PlayerId::A;

    let mut st = started(&db, 505);
    set_round(&mut st, me, 10);
    put_field(&db, &mut st, me, VANILLA);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SEOFON);
    drain_choice(&db, &mut st);
    let evolved: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.evolved)
        .collect();
    assert_eq!(
        evolved.len(),
        2,
        "Skybound (10) evolves all unevolved allies"
    );
    assert!(evolved.iter().all(|c| !c.super_evolved));

    let mut st = started(&db, 506);
    set_round(&mut st, me, 15);
    put_field(&db, &mut st, me, VANILLA);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SEOFON);
    drain_choice(&db, &mut st);
    let supers: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.super_evolved)
        .collect();
    assert_eq!(
        supers.len(),
        2,
        "Super Skybound (15) super-evolves them instead"
    );
}

#[test]
fn majestic_conquest_delays_own_crest() {
    let db = load_db();
    let mut st = started(&db, 507);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, MAJESTIC);
    drain_choice(&db, &mut st);
    let crest = st
        .player(me)
        .crests
        .iter()
        .find(|c| c.id == "crest:10622310")
        .expect("crest gained then delayed");
    assert_eq!(
        crest.countdown,
        Some(4),
        "Enhance is additive: gain (2) then delay +2"
    );
}

#[test]
fn shared_existence_highest_attack_tie() {
    let db = load_db();
    let mut st = started(&db, 508);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, VANILLA);
    put_field(&db, &mut st, opp, VANILLA);
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.attack = 5;
        f.defense = 5;
        f.max_defense = 5;
    }
    if let Some(f) = st.field_inst_mut(opp, 1) {
        f.attack = 5;
        f.defense = 5;
        f.max_defense = 5;
    }
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SHARED);
    drain_choice(&db, &mut st);
    let alive: Vec<_> = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .filter(|c| c.defense > 0)
        .collect();
    assert_eq!(
        alive.len(),
        1,
        "exactly one of the tied highest-attack followers is hit"
    );
    assert_eq!(alive[0].attack, 5);
}

#[test]
fn chloe_summon_from_hand_then_returns() {
    let db = load_db();
    let mut st = started(&db, 509);
    let me = PlayerId::A;
    give_pp(&mut st, me, 8, 8);
    st.player_mut(me).hand.clear();
    let pos = put_hand(&db, &mut st, me, VANILLA);
    if let Some(h) = st.player_mut(me).hand.get_mut(pos as usize) {
        h.attack = 9;
        h.defense = 7;
        h.max_defense = 7;
    }
    play_id(&db, &mut st, me, CHLOE);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    let v = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == VANILLA)
        .expect("summoned from hand");
    assert_eq!(v.attack, 9, "the hand instance moved, not a reprint");
    assert!(
        hand_has(&st, me, CHLOE),
        "this card returns to hand after the summon"
    );
    assert_eq!(
        st.player(me)
            .hand
            .iter()
            .filter(|c| c.card.as_str() == VANILLA)
            .count(),
        0,
        "the selected follower left the hand"
    );
}

#[test]
fn summon_copy_of_hand_leaves_the_hand_instance() {
    let db = load_db();
    let mut st = started(&db, 519);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    let pos = put_hand(&db, &mut st, me, VANILLA);
    if let Some(h) = st.player_mut(me).hand.get_mut(pos as usize) {
        h.attack = 9;
        h.defense = 7;
        h.max_defense = 7;
    }
    let hand_id = st.player(me).hand[pos as usize].id;
    play_id(&db, &mut st, me, COPY_HAND);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    let field = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == VANILLA)
        .expect("exact copy summoned");
    assert_eq!(field.attack, 9);
    assert_eq!(field.defense, 7);
    assert_ne!(field.id, hand_id, "the copy is a new instance");
    let still = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == VANILLA)
        .expect("hand instance stays (Cartographer)");
    assert_eq!(still.id, hand_id);
    assert_eq!(still.attack, 9);
    assert_eq!(still.defense, 7);
}

#[test]
fn wolfraud_five_exact_copies_from_enemy_deck() {
    let db = load_db();
    let mut st = started(&db, 510);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(opp).deck.clear();
    for id in [VANILLA, TANK, OKITA, KOU, TIA] {
        put_deck(&db, &mut st, opp, id);
    }
    let deck_before = st.player(opp).deck.len();
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, WOLFRAUD);
    drain_choice(&db, &mut st);
    let slot = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == WOLFRAUD))
        .expect("Wolfraud") as u8;
    set_round(&mut st, me, 5);
    st.player_mut(me).ep = 1;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve");
    assert_eq!(
        st.player(opp).deck.len(),
        deck_before,
        "exact copies leave the originals in the enemy deck"
    );
    assert_eq!(
        st.player(me).hand.len(),
        5,
        "discard hand, then five exact copies"
    );
    let mut names: Vec<_> = st.player(me).hand.iter().map(|c| c.card.as_str()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 5, "five distinct deck instances");
}

#[test]
fn marlone_x_floors_at_zero() {
    let db = load_db();
    let mut st = started(&db, 511);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, VANILLA);
    put_field(&db, &mut st, me, VANILLA);
    put_field(&db, &mut st, opp, TANK);
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, MARLONE);
    drain_choice(&db, &mut st);
    assert!(
        field_has(&st, opp, TANK),
        "X = 1 enemy − 3 allies floors at 0"
    );

    let mut st = started(&db, 512);
    for _ in 0..5 {
        put_field(&db, &mut st, opp, VANILLA);
    }
    give_pp(&mut st, me, 7, 7);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, MARLONE);
    drain_choice(&db, &mut st);
    let left = st.player(opp).field.iter().flatten().count();
    assert_eq!(left, 1, "X = 5 − 1 (Marlone) destroys 4");
}

#[test]
fn tia_once_per_turn_eve_on_own_enhance_buff() {
    let db = load_db();
    let mut st = started(&db, 513);
    let me = PlayerId::A;
    give_pp(&mut st, me, 4, 4);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, TIA);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(me)
            .hand
            .iter()
            .filter(|c| c.card.as_str() == EVE)
            .count(),
        1,
        "Enhance (4) buffs Tia and adds one Eve"
    );
    give_pp(&mut st, me, 2, 2);
    play_id(&db, &mut st, me, ALFHEIMR);
    // mode 2: +1/+0 all allies — second buff this turn
    choose(&db, &mut st, 1);
    assert_eq!(
        st.player(me)
            .hand
            .iter()
            .filter(|c| c.card.as_str() == EVE)
            .count(),
        1,
        "oncePerTurn: a second +atk this turn does not add another Eve"
    );
}

#[test]
fn tia_opponent_turn_buff_adds_no_eve() {
    let db = load_db();
    let mut st = started(&db, 523);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, TIA);
    drain_choice(&db, &mut st);
    assert!(!hand_has(&st, me, EVE), "no Enhance, no Eve yet");
    end_turn(&db, &mut st);
    give_pp(&mut st, opp, 1, 1);
    st.player_mut(opp).hand.clear();
    play_id(&db, &mut st, opp, BUFF_ENEMY);
    assert!(
        !hand_has(&st, me, EVE),
        "buff on the opponent's turn does not add Eve"
    );
    let tia = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == TIA)
        .expect("Tia");
    assert!(tia.attack > 2, "the opponent's buff landed");
}

#[test]
fn trap_in_the_woods_e31_super_evolved_entrant() {
    let db = load_db();
    let mut st = started(&db, 514);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, TRAP);
    drain_choice(&db, &mut st);
    assert!(field_has(&st, me, TRAP));
    end_turn(&db, &mut st);
    give_pp(&mut st, opp, 7, 7);
    st.player_mut(opp).hand.clear();
    play_id(&db, &mut st, opp, GOLDEN);
    choose(&db, &mut st, 0);
    assert!(
        field_has(&st, opp, GOLDEN),
        "SE on the owner's turn cannot be destroyed by the trap (E31)"
    );
    assert!(!field_has(&st, me, TRAP), "the trap still destroys itself");
}

#[test]
fn trap_in_the_woods_kills_only_first_of_three_knights() {
    let db = load_db();
    let mut st = started(&db, 524);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, TRAP);
    drain_choice(&db, &mut st);
    end_turn(&db, &mut st);
    give_pp(&mut st, opp, 1, 1);
    st.player_mut(opp).hand.clear();
    play_id(&db, &mut st, opp, SUMMON3);
    drain_choice(&db, &mut st);
    let knights: Vec<_> = st
        .player(opp)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == KNIGHT)
        .collect();
    assert_eq!(
        knights.len(),
        2,
        "E40: trap resolves once, two Knights stay"
    );
    assert!(!field_has(&st, me, TRAP), "the trap destroyed itself");
}

#[test]
fn e40_destroyed_enter_watcher_skips_later_entrants() {
    let db = load_db();
    let mut st = started(&db, 525);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, WATCHER);
    let hp = st.player(me).leader_defense;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SKIPPER);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(me).leader_defense,
        hp - 1,
        "first allied enter deals 1; later queued copies skip (E40)"
    );
    assert!(
        !field_has(&st, me, WATCHER),
        "the watcher destroyed itself on the first enter"
    );
    let fairies = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == FAIRY)
        .count();
    assert_eq!(fairies, 3);
}

#[test]
fn last_words_still_resolves_after_source_destroyed() {
    let db = load_db();
    let mut st = started(&db, 526);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, RUFLET);
    end_turn(&db, &mut st);
    give_pp(&mut st, opp, 2, 2);
    st.player_mut(opp).hand.clear();
    play_id(&db, &mut st, opp, SELECT_DESTROY);
    assert!(matches!(st.phase, Phase::Choice { .. }));
    choose(&db, &mut st, 0);
    assert!(!field_has(&st, me, RUFLET), "Ruflet was destroyed");
    assert!(
        hand_has(&st, me, FAIRY),
        "Last Words still adds a Fairy after the source left"
    );
}

#[test]
fn yuel_crest_evolves_played_follower_before_fanfare_once_per_turn() {
    let db = load_db();
    let mut st = started(&db, 515);
    let me = PlayerId::A;
    let order = st.crest_order;
    st.crest_order += 1;
    st.player_mut(me).crests.push(CrestInstance {
        id: "crest:10414120".into(),
        countdown: Some(4),
        faith: false,
        once_used: vec![],
        granted_order: order,
        granted: vec![],
    });
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, TAMER);
    drain_choice(&db, &mut st);
    let tamer = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == TAMER)
        .expect("Tamer");
    assert!(tamer.evolved, "crest evolves the played follower (E39)");
    assert!(
        hand_has(&st, me, FAIRY),
        "Fanfare still ran after the crest evolve"
    );
    give_pp(&mut st, me, 2, 2);
    play_id(&db, &mut st, me, VANILLA);
    drain_choice(&db, &mut st);
    let v = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == VANILLA)
        .expect("second play");
    assert!(
        !v.evolved,
        "crest oncePerTurn: a second follower this turn stays unevolved"
    );
}

#[test]
fn manamel_eot_evolve_then_own_damage_after_crests() {
    let db = load_db();
    let mut st = started(&db, 516);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, MANAMEL);
    put_field(&db, &mut st, opp, TANK);
    let before = st.field_inst(opp, 0).unwrap().defense;
    end_turn(&db, &mut st);
    let m = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == MANAMEL)
        .expect("Manamel");
    assert!(m.evolved, "end of turn evolves this follower");
    assert_eq!(
        st.field_inst(opp, 0).unwrap().defense,
        before - 1,
        "on:anyEvolve deals 1 after the evolve (E37: after crests)"
    );
}

#[test]
fn curiosity_abounds_two_differently_named() {
    let db = load_db();
    let mut st = started(&db, 517);
    let me = PlayerId::A;
    st.player_mut(me).deck.clear();
    put_deck(&db, &mut st, me, SPRING);
    put_deck(&db, &mut st, me, SPRING);
    put_deck(&db, &mut st, me, CHLOE);
    put_deck(&db, &mut st, me, TIA);
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, CURIOSITY);
    drain_choice(&db, &mut st);
    let names: std::collections::BTreeSet<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .map(|c| c.card.as_str())
        .collect();
    assert_eq!(names.len(), 2, "two differently named summons");
    assert!(names
        .iter()
        .all(|n| *n == SPRING || *n == CHLOE || *n == TIA));
}

#[test]
fn measured_attunement_restriction_lasts_through_opponent_turn() {
    let db = load_db();
    let mut st = started(&db, 518);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).rally = 10;
    put_field(&db, &mut st, opp, TANK);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, MEASURED);
    choose(&db, &mut st, 0);
    let t = st.field_inst(opp, 0).expect("tank survived 3");
    assert_eq!(t.traits.cant_attack_followers, Some(true));
    assert_eq!(t.traits.cant_attack_leader, Some(true));
    end_turn(&db, &mut st);
    let t = st.field_inst(opp, 0).expect("still restricted on opp turn");
    assert_eq!(t.traits.cant_attack_followers, Some(true));
    end_turn(&db, &mut st);
    let t = st.field_inst(opp, 0).expect("tank");
    assert_ne!(
        t.traits.cant_attack_followers,
        Some(true),
        "expires at the end of the opponent's turn"
    );
    assert_ne!(t.traits.cant_attack_leader, Some(true));
}

#[test]
fn self_consistency_sword_pool_20_seeds() {
    let db = load_db();
    let decks = load_deck_file("oracle/decks/sword-pool.json");
    assert_eq!(decks.len(), 40);
    assert!(deck_ready(&db, &decks), "sword-pool must load");
    for g in 0u64..20 {
        emit_and_replay_deck(&db, 20260910 + g, "oracle/decks/sword-pool.json");
    }
}

#[test]
fn self_consistency_forest_pool_20_seeds() {
    let db = load_db();
    let decks = load_deck_file("oracle/decks/forest-pool.json");
    assert_eq!(decks.len(), 40);
    assert!(deck_ready(&db, &decks), "forest-pool must load");
    for g in 0u64..20 {
        emit_and_replay_deck(&db, 20260910 + g, "oracle/decks/forest-pool.json");
    }
}

#[test]
fn sword_forest_pool_throughput_floor() {
    let floor: f64 = std::env::var("ARENA_BENCH_MIN_GAMES_PER_SEC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    if floor <= 0.0 {
        eprintln!("throughput skipped (set ARENA_BENCH_MIN_GAMES_PER_SEC)");
        return;
    }
    let db = load_db();
    for (path, label) in [
        ("oracle/decks/sword-pool.json", "sword-pool"),
        ("oracle/decks/forest-pool.json", "forest-pool"),
    ] {
        let decks = load_deck_file(path);
        const GAMES: u32 = 80;
        let t0 = std::time::Instant::now();
        let mut actions = 0u64;
        for i in 0..GAMES {
            let mut state = new_game(
                &db,
                GameConfig {
                    seed: 7 + u64::from(i),
                    deck_a: decks.clone(),
                    deck_b: decks.clone(),
                    first: First::A,
                    opening_hands: None,
                },
            )
            .expect("new_game");
            let mut policy = policy_rng(7 + u64::from(i));
            let mut n = 0u32;
            while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
                if state.turn > 60 || n >= 800 {
                    break;
                }
                let legal = legal_actions(&db, &state);
                if legal.is_empty() {
                    break;
                }
                let idx = policy.gen_range(legal.len() as u32) as usize;
                if apply(&db, &mut state, legal[idx].clone()).is_err() {
                    break;
                }
                n += 1;
            }
            actions += u64::from(n);
        }
        let secs = t0.elapsed().as_secs_f64().max(1e-9);
        let gps = f64::from(GAMES) / secs;
        eprintln!(
            "throughput {GAMES} {label} games: {gps:.1} games/s, {:.0} actions/s (floor {floor})",
            actions as f64 / secs
        );
        if cfg!(debug_assertions) {
            continue;
        }
        assert!(gps >= floor, "{label} {gps:.1} games/s below floor {floor}");
    }
}
