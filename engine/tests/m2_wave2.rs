//! M2 wave 2: Skybound Art, Faith, hand/deck modifiers, World of Games,
//! Earth Sigil banish-merge, split leftover-as-one-hit, attacksPerTurn.

use arena_engine::{
    apply, legal_actions, snapshot, Action, AttackTarget, GameRng, Phase, Pick, PickChose,
    PickWhat, PlayerId, Slot,
};

mod common;
use common::*;

const FLARE: &str = "10433310";
const BREW: &str = "10031210";
const SEDIMENT: &str = "90031210";
const SUBJECT: &str = "10931110";
const FAIRY: &str = "90011110";
const SATHANID: &str = "10614120";
const DEPTHS: &str = "90014330";
const TETRA: &str = "10834110";
const WORLD: &str = "10503210";
const PACKING: &str = "90034350";
const LOVE: &str = "10932310";
const LEAF: &str = "10912110";
const TICO: &str = "10833110";
const MISSILE: &str = "90031310";
const MOELLE: &str = "10811130";
const HARK: &str = "10753310";
const RAVEN: &str = "10933310";
const LYRIA: &str = "10403120";

fn require_ok(db: &arena_engine::CardDb, ids: &[&str]) {
    for id in ids {
        db.require_supported(cid(id))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

#[test]
fn require_supported_rune_closure() {
    let db = load_db();
    require_ok(
        &db,
        &[
            "10031210", "10433310", "10503210", "10733110", "10734120", "10803310", "10833110",
            "10834110", "10931110", "10932110", "10933110", "10933310", "10934110", "10932310",
            "90031210", "90031310", "90034340", "90034350",
        ],
    );
}

#[test]
fn require_supported_elf_closure() {
    let db = load_db();
    require_ok(
        &db,
        &[
            "10403120", "10503210", "10513310", "10514120", "10614120", "10712110", "10714110",
            "10714120", "10811130", "10814110", "10911110", "10912110", "10913110", "10913310",
            "10914110", "90011110", "90011310", "90014330",
        ],
    );
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

fn drain_choice(db: &arena_engine::CardDb, st: &mut arena_engine::State) {
    while matches!(st.phase, Phase::Choice { .. }) {
        choose(db, st, 0);
    }
}

#[test]
fn skybound_turns_taken_equals_round_for_both_players() {
    let db = load_db();
    let mut st = started(&db, 80);
    assert_eq!(st.turn, st.player(PlayerId::A).turns_taken);
    end_turn(&db, &mut st);
    assert_eq!(st.turn, st.player(PlayerId::B).turns_taken);
    assert_eq!(st.turn, 1);
}

#[test]
fn alchemic_flare_skybound_fires_on_turn_8_with_two_evolves() {
    let db = load_db();
    let mut st = started(&db, 81);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    set_round(&mut st, me, 8);
    st.player_mut(me).ep = 2;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).hand.clear();
    let flare = put_hand(&db, &mut st, me, FLARE);
    assert_eq!(st.player(me).hand[flare as usize].skybound, 0);
    grant_evolve(&db, &mut st, 0);
    st.player_mut(me).evolved_this_turn = false;
    grant_evolve(&db, &mut st, 1);
    assert_eq!(
        st.player(me).hand[flare as usize].skybound,
        2,
        "both evolves increment the hand copy"
    );
    assert_eq!(st.turn, st.player(me).turns_taken);
    put_field(&db, &mut st, opp, "88001320");
    let before = st.player(opp).leader_defense;
    play(&db, &mut st, flare);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(opp).leader_defense,
        before - 2,
        "8 + 2 >= 10 fires Skybound Art"
    );
}

#[test]
fn alchemic_flare_skybound_does_not_fire_on_turn_7() {
    let db = load_db();
    let mut st = started(&db, 82);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    set_round(&mut st, me, 7);
    st.player_mut(me).ep = 2;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).hand.clear();
    let flare = put_hand(&db, &mut st, me, FLARE);
    grant_evolve(&db, &mut st, 0);
    st.player_mut(me).evolved_this_turn = false;
    grant_evolve(&db, &mut st, 1);
    put_field(&db, &mut st, opp, "88001320");
    let before = st.player(opp).leader_defense;
    play(&db, &mut st, flare);
    drain_choice(&db, &mut st);
    assert_eq!(
        st.player(opp).leader_defense,
        before,
        "7 + 2 < 10 does not fire Skybound Art"
    );
}

#[test]
fn alchemic_flare_drawn_after_evolves_does_not_count_them() {
    let db = load_db();
    let mut st = started(&db, 83);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    set_round(&mut st, me, 8);
    st.player_mut(me).ep = 2;
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, "88001110");
    grant_evolve(&db, &mut st, 0);
    st.player_mut(me).evolved_this_turn = false;
    grant_evolve(&db, &mut st, 1);
    st.player_mut(me).hand.clear();
    let flare = put_hand(&db, &mut st, me, FLARE);
    assert_eq!(
        st.player(me).hand[flare as usize].skybound,
        0,
        "a copy added after the evolves is a fresh instance"
    );
    put_field(&db, &mut st, opp, "88001320");
    let before = st.player(opp).leader_defense;
    play(&db, &mut st, flare);
    drain_choice(&db, &mut st);
    assert_eq!(st.player(opp).leader_defense, before);
}

#[test]
fn effect_evolve_increments_skybound_on_hand() {
    // Depths of the Eld Lance is an effect-granted evolve (ruling 2026-08-10).
    let db = load_db();
    let mut st = started(&db, 84);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, FLARE);
    give_pp(&mut st, me, 1, 1);
    play_id(&db, &mut st, me, DEPTHS);
    drain_choice(&db, &mut st);
    let sb = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == FLARE)
        .map(|c| c.skybound)
        .unwrap_or(0);
    assert_eq!(sb, 1, "effect-granted evolve still increments skybound");
}

#[test]
fn faith_crest_granted_at_match_start() {
    let db = load_db();
    let st = started_decks(&db, 90, &[SATHANID], &["88001110"]);
    let a = &st.player(PlayerId::A).crests;
    assert!(
        a.iter()
            .any(|c| c.id == "faith:10614120" && c.countdown.is_none()),
        "Faith: Sathanid from the starting deck, no countdown: {a:?}"
    );
    assert_eq!(st.player(PlayerId::A).faith, 0);
    let snap = snapshot(&st);
    assert_eq!(snap.players.a.faith, 0);
    assert!(snap
        .players
        .a
        .crests
        .iter()
        .any(|c| c.id == "faith:10614120"));
}

#[test]
fn faith_increments_on_allied_evolve() {
    let db = load_db();
    let mut st = started_decks(&db, 91, &[SATHANID], &["88001110"]);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).ep = 1;
    grant_evolve(&db, &mut st, 0);
    assert_eq!(st.player(me).faith, 1);
}

#[test]
fn evolve_faith_increments_before_evolve_ability_choice() {
    // E37 / elf-0: crests and Faith drain before the evolving follower's
    // Evolve: list. Miroku's replicate-Fanfare choice sees faith already +1.
    let db = load_db();
    let mut st = started_decks(&db, 94, &[SATHANID], &["88001110"]);
    let me = PlayerId::A;
    set_round(&mut st, me, 7);
    st.player_mut(me).sep = 1;
    st.player_mut(me).faith = 2;
    put_field(&db, &mut st, me, "10514120");
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(0),
            super_evolve: true,
        },
    )
    .expect("super-evolve Miroku");
    assert!(
        matches!(st.phase, Phase::Choice { .. }),
        "Evolve: replicate Fanfare opens a mode choice"
    );
    assert_eq!(
        st.player(me).faith,
        3,
        "Faith crest resolved before the Evolve: list"
    );
}

#[test]
fn obsidian_raven_second_random_skips_dead_survivor_index() {
    // E36 / rune-11: [Brew, Sephie 4/4, Subject 5/5]. First pick slot:1
    // (Sephie) dies; second pick slot:1 is the Subject among survivors
    // (Brew=0, Subject=1), not raw slot 2.
    let db = load_db();
    let mut st = started(&db, 95);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, BREW);
    let seph = put_field(&db, &mut st, opp, "10934110");
    let sub = put_field(&db, &mut st, opp, SUBJECT);
    if let Some(f) = st.field_inst_mut(opp, seph) {
        f.attack = 4;
        f.defense = 4;
        f.max_defense = 4;
    }
    if let Some(f) = st.field_inst_mut(opp, sub) {
        f.attack = 5;
        f.defense = 5;
        f.max_defense = 5;
    }
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, RAVEN);
    st.rng = GameRng::scripted(
        vec![
            Pick {
                what: PickWhat::RandomTarget,
                among: None,
                chose: PickChose::Slot { slot: 1 },
            },
            Pick {
                what: PickWhat::RandomTarget,
                among: None,
                chose: PickChose::Slot { slot: 1 },
            },
        ],
        95,
    );
    apply(&db, &mut st, Action::Play { hand: h }).expect("Raven 7+7");
    assert!(field_has(&st, opp, BREW), "Brew is not a follower target");
    assert!(
        !field_has(&st, opp, "10934110") && !field_has(&st, opp, SUBJECT),
        "both followers take 7 and leave"
    );
}

#[test]
fn pay_faith_10_adds_depths_and_grant() {
    let db = load_db();
    let mut st = started_decks(&db, 92, &[SATHANID], &["88001110"]);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    st.player_mut(me).faith = 10;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SATHANID);
    drain_choice(&db, &mut st);
    assert_eq!(st.player(me).faith, 0);
    assert!(
        hand_has(&st, me, DEPTHS),
        "pay 10 adds Depths of the Eld Lance"
    );
    put_field(&db, &mut st, me, "88001110");
    st.player_mut(me).ep = 1;
    let before = st.player(opp).leader_defense;
    grant_evolve(&db, &mut st, 1);
    assert_eq!(
        st.player(opp).leader_defense,
        before - 1,
        "granted faith ability deals 1 on allied evolve"
    );
}

#[test]
fn pay_faith_below_10_fizzles() {
    let db = load_db();
    let mut st = started_decks(&db, 93, &[SATHANID], &["88001110"]);
    let me = PlayerId::A;
    st.player_mut(me).faith = 9;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SATHANID);
    drain_choice(&db, &mut st);
    assert_eq!(st.player(me).faith, 9, "unpaid");
    assert!(!hand_has(&st, me, DEPTHS), "wrapped effect fizzles");
}

#[test]
fn humane_love_hand_buff_survives_play() {
    let db = load_db();
    let mut st = started(&db, 100);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, LOVE);
    drain_choice(&db, &mut st);
    let hand_copy = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == SUBJECT)
        .expect("hand copy");
    assert_eq!(hand_copy.attack, 3, "+1/+0 on the hand instance");
    let h = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == SUBJECT)
        .unwrap();
    give_pp(&mut st, me, 2, 2);
    play(&db, &mut st, h as u8);
    drain_choice(&db, &mut st);
    let field = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == SUBJECT && c.attack == 3)
        .or_else(|| {
            st.player(me)
                .field
                .iter()
                .flatten()
                .find(|c| c.card.as_str() == SUBJECT)
        });
    let f = field.expect("played copy");
    assert_eq!(f.attack, 3, "hand +1/+0 travels to the field");
}

#[test]
fn leafshadow_bane_on_fairy_survives_play() {
    let db = load_db();
    let mut st = started(&db, 101);
    let me = PlayerId::A;
    st.player_mut(me).combo = 3;
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, LEAF);
    drain_choice(&db, &mut st);
    let fairy = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == FAIRY)
        .expect("fairy");
    assert_eq!(fairy.traits.bane, Some(true));
    let h = st
        .player(me)
        .hand
        .iter()
        .position(|c| c.card.as_str() == FAIRY)
        .unwrap();
    give_pp(&mut st, me, 1, 1);
    play(&db, &mut st, h as u8);
    let f = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == FAIRY)
        .expect("played fairy");
    assert_eq!(f.traits.bane, Some(true), "Bane travels from hand to field");
}

#[test]
fn tico_cost_reduction_stays_on_hand_missiles() {
    let db = load_db();
    let mut st = started(&db, 102);
    let me = PlayerId::A;
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, TICO);
    drain_choice(&db, &mut st);
    st.player_mut(me).ep = 1;
    let slot = st
        .player(me)
        .field
        .iter()
        .position(|s| s.as_ref().is_some_and(|c| c.card.as_str() == TICO))
        .unwrap();
    grant_evolve(&db, &mut st, slot as u8);
    let missiles: Vec<_> = st
        .player(me)
        .hand
        .iter()
        .filter(|c| c.card.as_str() == MISSILE)
        .collect();
    assert_eq!(missiles.len(), 2);
    assert!(missiles.iter().all(|c| c.cost == 0));
}

#[test]
fn thestae_crest_deck_buff_survives_draw_and_play() {
    let db = load_db();
    let mut st = started_decks(&db, 103, &["88001110", "88001110"], &["88001110"]);
    let me = PlayerId::A;
    st.player_mut(me)
        .crests
        .push(arena_engine::state::CrestInstance {
            id: "crest:10714110".into(),
            countdown: Some(3),
            faith: false,
            once_used: vec![],
            granted_order: 1,
            granted: vec![],
        });
    st.player_mut(me).combo = 3;
    let deck_atk = st
        .player(me)
        .deck
        .iter()
        .find(|c| c.card.as_str() == "88001110")
        .map(|c| c.attack)
        .unwrap_or(0);
    end_turn(&db, &mut st);
    // B's turn; the buff applied at A's end.
    let after = st
        .player(me)
        .deck
        .iter()
        .find(|c| c.card.as_str() == "88001110")
        .map(|c| c.attack)
        .unwrap_or(0);
    assert_eq!(after, deck_atk + 1, "deck instance is +1/+1");
    let pos = st
        .player(me)
        .deck
        .iter()
        .position(|c| c.card.as_str() == "88001110")
        .unwrap();
    let inst = st.player_mut(me).deck.remove(pos);
    assert_eq!(inst.attack, deck_atk + 1);
    st.player_mut(me).hand.clear();
    st.player_mut(me).hand.push(inst);
    end_turn(&db, &mut st); // back to A
    st.player_mut(me).hand.retain(|c| c.attack == deck_atk + 1);
    give_pp(&mut st, me, 1, 1);
    play(&db, &mut st, 0);
    let f = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == "88001110")
        .expect("played");
    assert_eq!(f.attack, deck_atk + 1);
    assert_eq!(f.defense, f.max_defense);
}

#[test]
fn moelle_return_to_deck_then_draw() {
    let db = load_db();
    let mut st = started(&db, 104);
    let me = PlayerId::A;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "88001110");
    play_id(&db, &mut st, me, MOELLE);
    drain_choice(&db, &mut st);
    assert!(
        st.player(me)
            .deck
            .iter()
            .any(|c| c.card.as_str() == "88001110"),
        "selected hand card returned to deck"
    );
    assert_eq!(
        st.player(me).hand.len(),
        1,
        "independent draw still happens"
    );
}

#[test]
fn reanimate_counts_as_enter_for_obsessed_test_subject() {
    // rune-29 i=37: Wills United Reanimate (2) is an enter. Five prior
    // Subject entries (four trades + the reanimate) make the next play's
    // "5 other allied copies have entered this match" true.
    let db = load_db();
    let mut st = started(&db, 110);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    for _ in 0..4 {
        put_field(&db, &mut st, opp, "88001110");
        give_pp(&mut st, me, 2, 2);
        st.player_mut(me).hand.clear();
        play_id(&db, &mut st, me, SUBJECT);
        apply(
            &db,
            &mut st,
            Action::Attack {
                attacker: Slot(0),
                target: AttackTarget::Slot(Slot(0)),
            },
        )
        .expect("trade");
    }
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10803310");
    choose(&db, &mut st, 1); // Reanimate (2)
    drain_choice(&db, &mut st);
    assert!(
        field_has(&st, me, SUBJECT),
        "reanimate put a Subject on the field"
    );
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, SUBJECT);
    let played = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == SUBJECT && c.attack == 5);
    let f = played.expect("sixth Subject");
    assert_eq!(
        (f.attack, f.defense),
        (5, 5),
        "reanimate counted as an enter so 5 others have entered"
    );
}

#[test]
fn return_to_deck_then_draw_takes_oldest_copy() {
    // Two Great Harts: the deck copy is Thestae-buffed +1/+1; Moelle returns
    // a printed copy (appended). Moelle's draw of that id takes the oldest.
    let db = load_db();
    let mut st = started(&db, 104);
    let me = PlayerId::A;
    let card = db.card(cid("10714120")).expect("hart");
    let mut buffed = arena_engine::CardInstance::from_card(card, st.alloc_id());
    buffed.attack += 1;
    buffed.defense += 1;
    buffed.max_defense += 1;
    st.player_mut(me).deck.clear();
    st.player_mut(me).deck.push(buffed);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10714120");
    give_pp(&mut st, me, 1, 1);
    play_id(&db, &mut st, me, MOELLE);
    drain_choice(&db, &mut st);
    let drew = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == "10714120")
        .map(|c| c.attack);
    assert_eq!(drew, Some(6), "draw of duplicate id takes the oldest copy");
    let left = st
        .player(me)
        .deck
        .iter()
        .find(|c| c.card.as_str() == "10714120")
        .map(|c| c.attack);
    assert_eq!(left, Some(5), "returned printed copy stays in the deck");
}

#[test]
fn world_of_games_advances_on_same_base_cost() {
    let db = load_db();
    let mut st = started(&db, 105);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, WORLD);
    drain_choice(&db, &mut st);
    let cd = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
        .and_then(|c| c.countdown);
    assert_eq!(cd, Some(5));
    // Play a 1-cost follower; World of Games itself (base 1) counts.
    play_id(&db, &mut st, me, "88001110");
    drain_choice(&db, &mut st);
    let cd = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
        .and_then(|c| c.countdown);
    assert_eq!(
        cd,
        Some(4),
        "advance by 1: World of Games counts for a 1-cost play"
    );
}

#[test]
fn world_of_games_does_not_advance_on_unmatched_base_cost() {
    // elf-22: after a 1-cost tick, Magachiyo (3) with no other 3-cost card
    // must not advance. The text is "other than it with the same base cost".
    let db = load_db();
    let mut st = started(&db, 111);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, WORLD);
    drain_choice(&db, &mut st);
    play_id(&db, &mut st, me, "88001110");
    drain_choice(&db, &mut st);
    play_id(&db, &mut st, me, "10914110");
    drain_choice(&db, &mut st);
    let cd = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
        .and_then(|c| c.countdown);
    assert_eq!(cd, Some(4), "Magachiyo is 3; WoG and vanilla are 1");
}

#[test]
fn world_of_games_fifth_advance_destroys_mid_resolution() {
    let db = load_db();
    let mut st = started(&db, 106);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, WORLD);
    if let Some(f) = st
        .player_mut(me)
        .field
        .iter_mut()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
    {
        f.countdown = Some(1);
    }
    let hand_before = st.player(me).hand.len();
    play_id(&db, &mut st, me, "88001110");
    drain_choice(&db, &mut st);
    assert!(
        !field_has(&st, me, WORLD),
        "countdown 0 destroys immediately"
    );
    assert!(
        st.player(me).hand.len() > hand_before,
        "Last Words draw 2 queued from the advance"
    );
}

#[test]
fn send_em_packing_grants_attacks_left_after_one_attack() {
    let db = load_db();
    let mut st = started(&db, 107);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let slot = put_field(&db, &mut st, me, "88001110");
    if let Some(f) = st.field_inst_mut(me, slot) {
        f.flags.summoning_sick = false;
        f.flags.attacks_left = 1;
    }
    put_field(&db, &mut st, opp, "88001320");
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(slot),
            target: AttackTarget::Leader,
        },
    )
    .expect("attack");
    assert_eq!(st.field_inst(me, slot).unwrap().flags.attacks_left, 0);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, PACKING);
    drain_choice(&db, &mut st);
    let f = st.field_inst(me, slot).expect("still there");
    assert_eq!(f.traits.attacks_per_turn, Some(2));
    assert!(
        f.flags.attacks_left >= 1,
        "granted 2 after one attack: may attack again (attacks_left={})",
        f.flags.attacks_left
    );
    let legal = legal_actions(&db, &st);
    assert!(
        legal.iter().any(|a| matches!(
            a,
            Action::Attack {
                attacker,
                ..
            } if attacker.0 == slot
        )),
        "legal attack after the grant"
    );
}

#[test]
fn alchemic_flare_gains_sediment_without_holder() {
    let db = load_db();
    let mut st = started(&db, 108);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001320");
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, FLARE);
    drain_choice(&db, &mut st);
    assert!(field_has(&st, me, SEDIMENT));
    assert_eq!(st.player(me).earth, 1);
}

#[test]
fn alchemic_flare_adds_to_brew_no_sediment() {
    let db = load_db();
    let mut st = started(&db, 109);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, BREW);
    put_field(&db, &mut st, opp, "88001320");
    assert_eq!(st.player(me).earth, 1);
    give_pp(&mut st, me, 2, 2);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, FLARE);
    drain_choice(&db, &mut st);
    assert_eq!(st.player(me).earth, 2);
    assert!(!field_has(&st, me, SEDIMENT));
    assert!(field_has(&st, me, BREW));
}

#[test]
fn brew_onto_sediment_2_banishes_sediment() {
    let db = load_db();
    let mut st = started(&db, 110);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, SEDIMENT);
    st.player_mut(me).earth = 2;
    let shadows = st.player(me).shadows;
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, BREW);
    drain_choice(&db, &mut st);
    assert!(field_has(&st, me, BREW));
    assert!(!field_has(&st, me, SEDIMENT));
    assert_eq!(st.player(me).earth, 3);
    assert_eq!(st.player(me).shadows, shadows, "banish produces no shadow");
    assert!(st
        .player(me)
        .banished
        .iter()
        .any(|c| c.card.as_str() == SEDIMENT));
}

#[test]
fn brew_onto_brew_newest_survives_old_banished() {
    let db = load_db();
    let mut st = started(&db, 111);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, BREW);
    assert_eq!(st.player(me).earth, 1);
    give_pp(&mut st, me, 1, 1);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, BREW);
    drain_choice(&db, &mut st);
    let brews: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == BREW)
        .collect();
    assert_eq!(brews.len(), 1, "one Brew remains");
    assert_eq!(st.player(me).earth, 2);
    assert_eq!(
        st.player(me)
            .banished
            .iter()
            .filter(|c| c.card.as_str() == BREW)
            .count(),
        1
    );
}

#[test]
fn generated_sediment_merges_into_brew() {
    let db = load_db();
    let mut st = started(&db, 112);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, BREW);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "88001810");
    assert_eq!(st.player(me).earth, 2);
    assert!(!field_has(&st, me, SEDIMENT));
    assert!(field_has(&st, me, BREW));
}

#[test]
fn split_leftover_is_one_hit_on_last_follower() {
    let db = load_db();
    let mut st = started(&db, 113);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001320");
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.defense = 2;
        f.max_defense = 2;
        f.attack = 2;
    }
    if let Some(f) = st.field_inst_mut(opp, 1) {
        f.defense = 1;
        f.max_defense = 1;
        f.attack = 1;
    }
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, HARK);
    drain_choice(&db, &mut st);
    let oldest = st.field_inst(opp, 0);
    assert!(oldest.is_none(), "2/2 dies to a share of 2");
    let last = st.field_inst(opp, 0).or_else(|| st.field_inst(opp, 1));
    // After compact, the 1/1 that took 4 is gone.
    assert!(
        !field_has(&st, opp, "88001320") || last.is_some_and(|c| c.defense <= 0),
        "1/1 takes one hit of 4 and dies"
    );
}

#[test]
fn split_into_lone_barrier_1_1_survives() {
    let db = load_db();
    let mut st = started(&db, 114);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    if let Some(f) = st.field_inst_mut(opp, 0) {
        f.defense = 1;
        f.max_defense = 1;
        f.traits.barrier = Some(true);
    }
    give_pp(&mut st, me, 3, 3);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, HARK);
    drain_choice(&db, &mut st);
    let f = st.field_inst(opp, 0).expect("alive");
    assert_eq!(
        f.defense, 1,
        "Barrier reduced the whole leftover share to 0"
    );
    assert_ne!(f.traits.barrier, Some(true), "Barrier consumed");
}

#[test]
fn tetra_vars_increment_on_spellboost() {
    let db = load_db();
    let mut st = started(&db, 115);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, TETRA);
    give_pp(&mut st, me, 4, 4);
    play_id(&db, &mut st, me, MISSILE);
    drain_choice(&db, &mut st);
    play_id(&db, &mut st, me, MISSILE);
    drain_choice(&db, &mut st);
    let x = st
        .player(me)
        .hand
        .iter()
        .find(|c| c.card.as_str() == TETRA)
        .and_then(|c| c.vars.get(&arena_engine::card::VarKey::X).copied())
        .unwrap_or(0);
    assert_eq!(x, 2, "two spells Spellboost Tetra's X");
}

#[test]
fn sephie_fused_spends_pp_and_summons_subject() {
    let db = load_db();
    let mut st = started(&db, 120);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    let host = put_hand(&db, &mut st, me, "10934110");
    let _partner = put_hand(&db, &mut st, me, "88001110");
    apply(&db, &mut st, Action::Fuse { host }).expect("fuse start");
    choose(&db, &mut st, 0);
    confirm(&db, &mut st);
    assert!(
        field_has(&st, me, SUBJECT),
        "fused ability summons Obsessed Test Subject"
    );
    assert_eq!(st.player(me).pp, 3, "spent 2 PP");
}

#[test]
fn sephie_can_fuse_the_same_kind_across_turns() {
    let db = load_db();
    let mut st = started(&db, 126);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "10934110");
    put_hand(&db, &mut st, me, MISSILE);
    apply(&db, &mut st, Action::Fuse { host: 0 }).expect("fuse 1");
    choose(&db, &mut st, 0);
    confirm(&db, &mut st);
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    give_pp(&mut st, me, 5, 5);
    put_hand(&db, &mut st, me, MISSILE);
    let legal = legal_actions(&db, &st);
    assert!(
        legal.iter().any(|a| matches!(a, Action::Fuse { host: 0 })),
        "Fuse: Cards may take a Missile again next turn; kind-memory is α-only"
    );
}

#[test]
fn thestae_fanfare_minus_x_is_this_followers_attack() {
    let db = load_db();
    let mut st = started(&db, 121);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 4, 4);
    put_field(&db, &mut st, opp, "88001320");
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, "10714110");
    drain_choice(&db, &mut st);
    let f = st.field_inst(opp, 0).expect("alive");
    assert_eq!(f.defense, 7, "Thestae 3 attack → −0/−3");
    assert_eq!(f.max_defense, 7);
}

#[test]
fn great_hart_eot_split_uses_own_attack() {
    let db = load_db();
    let mut st = started(&db, 123);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10714120");
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001110");
    end_turn(&db, &mut st);
    assert_eq!(
        field_count(&st, opp),
        0,
        "Hart 5 attack split: 2 then leftover 3, both 2/2 die"
    );
}

#[test]
fn world_of_games_advances_on_enemy_same_base_cost() {
    let db = load_db();
    let mut st = started(&db, 122);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 5, 5);
    put_field(&db, &mut st, opp, "10712110");
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, WORLD);
    drain_choice(&db, &mut st);
    play_id(&db, &mut st, me, FLARE);
    drain_choice(&db, &mut st);
    let cd = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
        .and_then(|c| c.countdown);
    assert_eq!(
        cd,
        Some(4),
        "enemy 2-cost follower matches Flare's base cost 2"
    );
}

#[test]
fn drain_vs_barrier_restores_zero() {
    let db = load_db();
    let mut st = started(&db, 124);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let atk = put_field(&db, &mut st, me, SATHANID);
    put_field(&db, &mut st, opp, LYRIA);
    let before = st.player(me).leader_defense;
    apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: Slot(atk),
            target: AttackTarget::Slot(Slot(0)),
        },
    )
    .expect("attack");
    assert_eq!(
        st.player(me).leader_defense,
        before,
        "Barrier reduced the Drain hit to 0; restore nothing"
    );
    let lyria = st.field_inst(opp, 0).expect("Lyria lives");
    assert_ne!(lyria.traits.barrier, Some(true), "Barrier consumed");
    assert_eq!(lyria.defense, 1);
}

#[test]
fn obsidian_raven_with_no_enemy_follower_still_summons() {
    let db = load_db();
    let mut st = started(&db, 125);
    let me = PlayerId::A;
    give_pp(&mut st, me, 5, 5);
    put_field(&db, &mut st, me, BREW);
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, RAVEN);
    drain_choice(&db, &mut st);
    assert!(
        field_has(&st, me, SUBJECT),
        "summon still happens when the random damage has no target"
    );
}

#[test]
fn world_of_games_dies_on_play_before_enamored_enhance_summons() {
    // E39 / rune-2 i=68: WoG at 1 on a 3-card board (a 4-cost so the Enhance
    // matches, plus a filler). Play Enamored Enhance (summon ×3): WoG dies
    // on play, two Subjects land, Last Words draw before the summons.
    let db = load_db();
    let mut st = started(&db, 139);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    put_field(&db, &mut st, me, "10011130");
    put_field(&db, &mut st, me, "88001110");
    put_field(&db, &mut st, me, WORLD);
    if let Some(f) = st
        .player_mut(me)
        .field
        .iter_mut()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
    {
        f.countdown = Some(1);
    }
    st.player_mut(me).hand.clear();
    let hand_before = st.player(me).hand.len();
    play_id(&db, &mut st, me, "10932110");
    drain_choice(&db, &mut st);
    assert!(
        !field_has(&st, me, WORLD),
        "WoG dies at play time before the Enhance summons"
    );
    let subjects: Vec<_> = st
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.card.as_str() == SUBJECT)
        .collect();
    assert_eq!(
        subjects.len(),
        2,
        "two of three Enhance summons land after WoG frees the slot"
    );
    assert!(
        subjects.iter().all(|c| c.traits.ward == Some(true)),
        "Enhance grants Ward to the summons that land"
    );
    assert_eq!(
        st.player(me).hand.len(),
        hand_before + 2,
        "Last Words Draw 2 resolve before the summons"
    );
    assert_eq!(field_count(&st, me), 5);
}

#[test]
fn world_of_games_divine_thunder_qa_count_is_four() {
    // Official Q&A 10503210: only card on my field is WoG at 5; only enemy
    // card is Quake Goliath. Play Divine Thunder → count 4. The enemy
    // 4-cost counts, and the play reaction resolves before the spell
    // destroys Goliath.
    let db = load_db();
    let mut st = started(&db, 140);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    give_pp(&mut st, me, 4, 4);
    put_field(&db, &mut st, opp, "10001130");
    st.player_mut(me).hand.clear();
    play_id(&db, &mut st, me, WORLD);
    drain_choice(&db, &mut st);
    let cd = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
        .and_then(|c| c.countdown);
    assert_eq!(cd, Some(5));
    play_id(&db, &mut st, me, "10103310");
    drain_choice(&db, &mut st);
    let cd = st
        .player(me)
        .field
        .iter()
        .flatten()
        .find(|c| c.card.as_str() == WORLD)
        .and_then(|c| c.countdown);
    assert_eq!(cd, Some(4), "official Q&A: count will be 4");
    assert!(
        !field_has(&st, opp, "10001130"),
        "Divine Thunder still destroys Goliath"
    );
}

#[test]
fn evolved_follower_is_not_offered_super_evolve() {
    // Glossary Evolution / owner 2026-09-10: cannot EP-evolve then SEP later.
    let db = load_db();
    let mut st = started(&db, 141);
    let me = PlayerId::A;
    put_field(&db, &mut st, me, "88001110");
    grant_evolve(&db, &mut st, 0);
    st.player_mut(me).evolved_this_turn = false;
    set_round(&mut st, me, 7);
    st.player_mut(me).sep = 1;
    st.player_mut(me).ep = 1;
    let legal = legal_actions(&db, &st);
    assert!(
        !legal
            .iter()
            .any(|a| matches!(a, Action::Evolve { slot: Slot(0), .. })),
        "an evolved follower is offered neither evolve nor super-evolve"
    );
}

#[test]
fn effect_evolve_on_evolved_follower_is_noop() {
    // Remi & Rami Super-Evolve: evolve a Golem, then +3/+3. An already
    // evolved Golem is not evolved again; only the buff applies.
    let db = load_db();
    let mut st = started(&db, 142);
    let me = PlayerId::A;
    let golem = put_field(&db, &mut st, me, "90031120");
    put_field(&db, &mut st, me, "10032110");
    grant_evolve(&db, &mut st, golem);
    let g = st.field_inst(me, golem).expect("golem");
    assert_eq!(g.attack, 5);
    assert_eq!(g.defense, 5);
    assert!(g.evolved);
    assert!(!g.super_evolved);
    let used = st.player(me).evolves_used;
    set_round(&mut st, me, 7);
    st.player_mut(me).sep = 1;
    st.player_mut(me).evolved_this_turn = false;
    apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: Slot(1),
            super_evolve: true,
        },
    )
    .expect("super-evolve Remi");
    drain_choice(&db, &mut st);
    let g = st.field_inst(me, golem).expect("golem after effect-evolve");
    assert_eq!(
        g.attack, 8,
        "no-op evolve: +3/+3 only, not +2/+2 then +3/+3"
    );
    assert_eq!(g.defense, 8);
    assert_eq!(g.max_defense, 8);
    assert!(g.evolved);
    assert!(
        !g.super_evolved,
        "effect-evolve does not super-evolve the Golem"
    );
    assert_eq!(
        st.player(me).evolves_used,
        used + 1,
        "only Remi's SEP counts; the no-op does not increment evolves_used"
    );
}
