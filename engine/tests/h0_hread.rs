//! Hand-reading (`hread`): turn-end history, key isolation, SIR weights.

use arena_engine::determinize::{
    determinize_block, HreadDeal, Info, HREAD_DELTA, HREAD_EPS_FA, HREAD_EPS_S,
};
use arena_engine::{
    apply, hash, search_key, Action, AnyPolicy, CardDb, CardInstance, PlayerId, State, TurnEnd, H0,
};

mod common;
use common::*;

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn zone_ids(items: &[CardInstance]) -> Vec<u32> {
    items.iter().map(|c| c.id).collect()
}

#[test]
fn hread_history_turn_ends_and_hand_since() {
    let db = load_db();
    let mut st = started(&db, 77);
    give_pp(&mut st, PlayerId::A, 3, 5);
    give_pp(&mut st, PlayerId::B, 2, 4);
    st.active = PlayerId::A;

    assert_eq!(st.player(PlayerId::A).turn_ends.len(), 0);
    for inst in &st.player(PlayerId::A).hand {
        assert_eq!(inst.hand_since, 0);
    }

    apply(&db, &mut st, Action::EndTurn).expect("A end");
    assert_eq!(st.player(PlayerId::A).turn_ends.len(), 1);
    assert_eq!(st.player(PlayerId::A).turn_ends[0].unspent, 3);
    assert_eq!(st.player(PlayerId::A).turn_ends[0].pp_max, 5);
    assert!(!st.player(PlayerId::A).turn_ends[0].board_full);

    let a_hand_before_b_end = st.player(PlayerId::A).hand.len();
    apply(&db, &mut st, Action::EndTurn).expect("B end");
    assert_eq!(st.player(PlayerId::B).turn_ends.len(), 1);
    assert!(st.player(PlayerId::B).turn_ends[0].unspent > 0);
    assert!(st.player(PlayerId::B).turn_ends[0].pp_max >= 4);
    assert_eq!(
        st.player(PlayerId::A).hand.len(),
        a_hand_before_b_end + 1,
        "A draws at the start of their turn after B ends"
    );
    assert_eq!(
        st.player(PlayerId::A).hand.last().unwrap().hand_since,
        1,
        "draw after B's first End Turn"
    );

    apply(&db, &mut st, Action::EndTurn).expect("A end 2");
    assert_eq!(st.player(PlayerId::A).turn_ends.len(), 2);
    apply(&db, &mut st, Action::EndTurn).expect("B end 2");
    assert_eq!(
        st.player(PlayerId::A).hand.last().unwrap().hand_since,
        2,
        "draw after two more End Turns"
    );
}

#[test]
fn hread_not_in_hash_or_search_key() {
    let db = load_db();
    let st = started(&db, 88);
    let h0 = hash(&st);
    let k0 = search_key(&st);

    let mut alt = st.clone();
    alt.player_mut(PlayerId::A).turn_ends.push(TurnEnd {
        unspent: 9,
        pp_max: 10,
        board_full: true,
    });
    alt.player_mut(PlayerId::B).hand[0].hand_since = 99;

    assert_eq!(hash(&alt), h0);
    assert_eq!(search_key(&alt), k0);
}

fn hread_weight_fixture(db: &CardDb) -> (State, PlayerId) {
    let mut st = started(db, 123);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    clear_hand(&mut st, opp);
    st.player_mut(opp).deck.clear();
    st.player_mut(opp).cemetery.clear();
    st.player_mut(opp).banished.clear();
    st.player_mut(opp).hidden_removals.clear();
    st.player_mut(opp).hidden_removal_cards.clear();
    st.player_mut(opp).public_hand_additions.clear();

    let cheap = db.card(cid("10061120")).expect("cheap");
    let pricey = db.card(cid("10954120")).expect("pricey");
    assert_eq!(cheap.cost(), 2);
    assert_eq!(pricey.cost(), 8);

    let mut slot_inst = CardInstance::from_card(cheap, st.alloc_id());
    slot_inst.hand_since = 0;
    st.player_mut(opp).hand.push(slot_inst);

    st.player_mut(opp).turn_ends = vec![
        TurnEnd {
            unspent: 5,
            pp_max: 5,
            board_full: false,
        },
        TurnEnd {
            unspent: 5,
            pp_max: 5,
            board_full: false,
        },
        TurnEnd {
            unspent: 5,
            pp_max: 5,
            board_full: false,
        },
    ];

    let pool_pricey = CardInstance::from_card(pricey, st.alloc_id());
    st.player_mut(opp).deck.push(pool_pricey);

    st.active = me;
    assert_eq!(st.player(opp).turn_ends.len(), 3);
    assert_eq!(st.player(opp).hand[0].hand_since, 0);
    (st, me)
}

fn hread_hand_since_fixture(db: &CardDb) -> (State, PlayerId) {
    let mut st = started(db, 456);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    clear_hand(&mut st, opp);
    st.player_mut(opp).deck.clear();
    st.player_mut(opp).hidden_removals.clear();
    st.player_mut(opp).hidden_removal_cards.clear();
    st.player_mut(opp).public_hand_additions.clear();

    let cheap = db.card(cid("10061120")).expect("cheap");
    let pricey = db.card(cid("10954120")).expect("pricey");

    let mut slot_inst = CardInstance::from_card(cheap, st.alloc_id());
    slot_inst.hand_since = 2;
    st.player_mut(opp).hand.push(slot_inst);

    st.player_mut(opp).turn_ends = vec![
        TurnEnd {
            unspent: 5,
            pp_max: 5,
            board_full: false,
        },
        TurnEnd {
            unspent: 5,
            pp_max: 5,
            board_full: false,
        },
        TurnEnd {
            unspent: 5,
            pp_max: 5,
            board_full: false,
        },
    ];

    let pricey_id = st.alloc_id();
    st.player_mut(opp)
        .deck
        .push(CardInstance::from_card(pricey, pricey_id));
    st.active = me;
    (st, me)
}

fn hread_empty_hand_fixture(db: &CardDb) -> (State, PlayerId) {
    let mut st = started(db, 789);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    clear_hand(&mut st, opp);
    st.player_mut(opp).hidden_removals.clear();
    st.player_mut(opp).hidden_removal_cards.clear();
    st.player_mut(opp).public_hand_additions.clear();
    assert!(st.player(opp).hand.is_empty());
    assert!(st.player(opp).deck.len() >= 3);
    st.active = me;
    (st, me)
}

#[test]
fn hread_weights_favor_expensive_card() {
    let db = load_db();
    let (st, me) = hread_weight_fixture(&db);
    let hread = HreadDeal {
        eps_fa: HREAD_EPS_FA,
        eps_s: HREAD_EPS_S,
        delta: HREAD_DELTA,
        m: 256,
    };

    let mut cheap_count = 0u32;
    let trials = 2000u64;
    for seed in 1..=trials {
        let w = determinize_block(&st, me, seed, 0, 0, Info::Open, None, false, Some(hread));
        let card = w.player(me.opponent()).hand[0].card;
        if card == cid("10061120") {
            cheap_count += 1;
        }
    }
    let cheap_frac = cheap_count as f64 / trials as f64;
    assert!(
        cheap_frac < 0.05,
        "cheap card dealt {cheap_frac:.3} of the time with hread=on"
    );

    let mut cheap_off = 0u32;
    for seed in 1..=trials {
        let w = determinize_block(&st, me, seed, 0, 0, Info::Open, None, false, None);
        let card = w.player(me.opponent()).hand[0].card;
        if card == cid("10061120") {
            cheap_off += 1;
        }
    }
    let off_frac = cheap_off as f64 / trials as f64;
    assert!(
        off_frac > 0.35 && off_frac < 0.65,
        "cheap card dealt {off_frac:.3} of the time with hread=off"
    );
}

#[test]
fn hread_dealt_card_carries_slot_hand_since() {
    let db = load_db();
    let (st, me) = hread_hand_since_fixture(&db);
    let hread = HreadDeal {
        eps_fa: HREAD_EPS_FA,
        eps_s: HREAD_EPS_S,
        delta: HREAD_DELTA,
        m: 256,
    };

    for seed in [11u64, 22, 33] {
        let on = determinize_block(&st, me, seed, 0, 0, Info::Open, None, false, Some(hread));
        assert_eq!(on.player(me.opponent()).hand[0].hand_since, 2);
        let off = determinize_block(&st, me, seed, 0, 0, Info::Open, None, false, None);
        assert_eq!(off.player(me.opponent()).hand[0].hand_since, 2);
    }
}

#[test]
fn hread_empty_hand_matches_off_and_shuffles_deck() {
    let db = load_db();
    let (st, me) = hread_empty_hand_fixture(&db);
    let hread = HreadDeal {
        eps_fa: HREAD_EPS_FA,
        eps_s: HREAD_EPS_S,
        delta: HREAD_DELTA,
        m: 256,
    };

    for seed in [11u64, 22, 33] {
        let off = determinize_block(&st, me, seed, 0, 0, Info::Open, None, false, None);
        let on = determinize_block(&st, me, seed, 0, 0, Info::Open, None, false, Some(hread));
        assert_eq!(
            zone_ids(&off.player(me.opponent()).hand),
            zone_ids(&on.player(me.opponent()).hand)
        );
        assert_eq!(
            zone_ids(&off.player(me.opponent()).deck),
            zone_ids(&on.player(me.opponent()).deck)
        );
    }

    let d11 = zone_ids(
        &determinize_block(&st, me, 11, 0, 0, Info::Open, None, false, Some(hread))
            .player(me.opponent())
            .deck,
    );
    let d22 = zone_ids(
        &determinize_block(&st, me, 22, 0, 0, Info::Open, None, false, Some(hread))
            .player(me.opponent())
            .deck,
    );
    assert_ne!(d11, d22, "different root seeds must shuffle the deck");
}

#[test]
fn hread_deterministic_for_seed() {
    let db = load_db();
    let (st, me) = hread_weight_fixture(&db);
    let hread = HreadDeal {
        eps_fa: HREAD_EPS_FA,
        eps_s: HREAD_EPS_S,
        delta: HREAD_DELTA,
        m: 256,
    };
    let a = determinize_block(&st, me, 4242, 0, 0, Info::Open, None, false, Some(hread));
    let b = determinize_block(&st, me, 4242, 0, 0, Info::Open, None, false, Some(hread));
    assert_eq!(
        a.player(me.opponent()).hand[0].card,
        b.player(me.opponent()).hand[0].card
    );
}

#[test]
fn hread_spec_round_trip() {
    let h = parse_h0("h0:hread=on,hreadm=512");
    let spec = AnyPolicy::H0(h).spec();
    assert!(spec.contains("hread=on"));
    assert!(spec.contains("hreadm=512"));
    let h2 = parse_h0(&spec);
    assert_eq!(h2.hread, Some((HREAD_EPS_FA, HREAD_EPS_S, HREAD_DELTA)));
    assert_eq!(h2.hreadm, 512);
}

#[test]
fn hread_parse_rejections() {
    for bad in [
        "h0:hread=0/0.5/0.8",
        "h0:hread=1.5/0.5/0.8",
        "h0:hread=0.15/0.5",
        "h0:hread=x",
        "h0:hreadm=0",
        "h0:hreadm=4097",
    ] {
        assert!(
            AnyPolicy::parse_spec(bad).is_err(),
            "{bad} should be rejected"
        );
    }
    let off = parse_h0("h0:hread=off");
    assert!(off.hread.is_none());
    assert_eq!(AnyPolicy::H0(off).spec(), "h0");
}
