//! H0 information regime (`info`): identity at the default (`fair`),
//! per-mode determinize unit tests, Fair seed-stability and variation,
//! spec round-trip, and play determinism. `info=draws` and `info=all`
//! remain available as explicit spec keys; `all` is hard-mode sparring.

use std::collections::BTreeMap;

use arena_engine::determinize::{
    determinize, determinize_with, determinize_with_stats, Info, OpenStats,
};
use arena_engine::{
    apply, legal_actions, new_game, play_game, policy_rng, search_key, Action, AnyPolicy, CardDb,
    CardId, CardInstance, First, GameConfig, Phase, PlayerId, Policy, State, H0, MAX_ACTIONS,
    MAX_TURNS,
};

mod common;
use common::*;

fn collect_states(db: &CardDb, n: usize, midgame_only: bool) -> Vec<arena_engine::State> {
    let decks = load_pinned_corpus_decks_sorted(db);
    let mut out = Vec::with_capacity(n);
    let mut seed = 1u64;
    while out.len() < n {
        let da = &decks[seed as usize % decks.len()];
        let dbk = &decks[(seed as usize / 3) % decks.len()];
        let Ok(mut state) = new_game(
            db,
            GameConfig {
                seed,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: First::A,
                opening_hands: None,
            },
        ) else {
            seed += 1;
            continue;
        };
        let mut rng = policy_rng(seed);
        let mut nact = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            let take = !midgame_only || (state.turn >= 3 && matches!(state.phase, Phase::Main));
            if take {
                out.push(state.clone());
                if out.len() >= n {
                    break;
                }
            }
            if state.turn > MAX_TURNS || nact >= MAX_ACTIONS {
                break;
            }
            let legal = legal_actions(db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = rng.gen_range(legal.len() as u32) as usize;
            if apply(db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
            nact += 1;
        }
        seed += 1;
        if seed > 50_000 {
            break;
        }
    }
    out
}

fn parse_h0(spec: &str) -> H0 {
    match AnyPolicy::parse_spec(spec).unwrap_or_else(|e| panic!("{spec}: {e}")) {
        AnyPolicy::H0(h) => h,
        other => panic!("{spec} parsed as {other:?}"),
    }
}

fn zone_ids(items: &[arena_engine::CardInstance]) -> Vec<u32> {
    items.iter().map(|c| c.id).collect()
}

fn zone_multiset(items: &[arena_engine::CardInstance]) -> BTreeMap<(u32, u32), u32> {
    let mut m = BTreeMap::new();
    for c in items {
        *m.entry((c.card.0, c.id)).or_insert(0) += 1;
    }
    m
}

/// Mixed hidden cards on both sides so shuffle / resample have something
/// to permute. Built on `started` so public-knowledge bookkeeping is real.
fn mixed_info_state(db: &CardDb) -> arena_engine::State {
    let mut st = started(db, 41);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    st.player_mut(PlayerId::A).deck.clear();
    st.player_mut(PlayerId::B).deck.clear();
    put_hand(db, &mut st, PlayerId::A, "10461110");
    put_hand(db, &mut st, PlayerId::A, "10061120");
    put_deck(db, &mut st, PlayerId::A, "88001110");
    put_deck(db, &mut st, PlayerId::A, "10461110");
    put_deck(db, &mut st, PlayerId::A, "10061120");
    put_deck(db, &mut st, PlayerId::A, "88001110");
    put_hand(db, &mut st, PlayerId::B, "88001110");
    put_hand(db, &mut st, PlayerId::B, "10461110");
    put_deck(db, &mut st, PlayerId::B, "10061120");
    put_deck(db, &mut st, PlayerId::B, "88001110");
    put_deck(db, &mut st, PlayerId::B, "10461110");
    assert!(matches!(st.phase, Phase::Main));
    st
}

fn check_open_identity(n: usize) {
    let db = load_db();
    let states = collect_states(&db, n, true);
    assert_eq!(states.len(), n, "could not reach {n} mid-game states");
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:info=open").unwrap().spec(), "h0");
    for (i, state) in states.iter().enumerate() {
        let legal = legal_actions(&db, state);
        if legal.is_empty() {
            continue;
        }
        let seed = 20260919u64.wrapping_add(i as u64);
        let mut main = H0::default();
        let mut named = parse_h0("h0");
        let mut open = parse_h0("h0:info=open");
        let mut rng_m = policy_rng(seed);
        let mut rng_n = policy_rng(seed);
        let mut rng_d = policy_rng(seed);
        let im = main.choose(&db, state, &legal, &mut rng_m);
        let inn = named.choose(&db, state, &legal, &mut rng_n);
        let idr = open.choose(&db, state, &legal, &mut rng_d);
        assert_eq!(im, inn, "h0 vs H0::default at state {i}");
        assert_eq!(im, idr, "h0 vs h0:info=open at state {i}");
        assert!(im < legal.len());
    }
}

#[test]
fn open_identity_smoke() {
    check_open_identity(8);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn open_identity_200_midgame() {
    check_open_identity(200);
}

#[test]
fn determinize_draws_leaves_own_deck() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let me = PlayerId::A;
    let got = determinize(&st, me, 7);
    let via = determinize_with(&st, me, 7, Info::Draws);
    assert_eq!(
        zone_ids(&got.player(me).deck),
        zone_ids(&st.player(me).deck)
    );
    assert_eq!(
        zone_ids(&got.player(me).hand),
        zone_ids(&st.player(me).hand)
    );
    assert_eq!(
        zone_ids(&got.player(me).deck),
        zone_ids(&via.player(me).deck)
    );
    assert_eq!(
        zone_ids(&got.player(me.opponent()).hand),
        zone_ids(&via.player(me.opponent()).hand)
    );
    assert_eq!(
        zone_ids(&got.player(me.opponent()).deck),
        zone_ids(&via.player(me.opponent()).deck)
    );
    assert_eq!(
        zone_multiset(&got.player(me.opponent()).hand)
            .into_iter()
            .chain(zone_multiset(&got.player(me.opponent()).deck))
            .collect::<BTreeMap<_, _>>(),
        zone_multiset(&st.player(me.opponent()).hand)
            .into_iter()
            .chain(zone_multiset(&st.player(me.opponent()).deck))
            .collect::<BTreeMap<_, _>>()
    );
}

#[test]
fn determinize_fair_keeps_own_hand_permutes_own_deck() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let me = PlayerId::A;
    let seed = 11u64;
    let fair = determinize_with(&st, me, seed, Info::Fair);
    let draws = determinize_with(&st, me, seed, Info::Draws);
    assert_eq!(
        zone_ids(&fair.player(me).hand),
        zone_ids(&st.player(me).hand),
        "Fair must not touch the perspective hand"
    );
    assert_eq!(
        zone_multiset(&fair.player(me).deck),
        zone_multiset(&st.player(me).deck),
        "Fair own deck is a permutation of the same multiset"
    );
    assert_eq!(
        zone_ids(&fair.player(me.opponent()).hand),
        zone_ids(&draws.player(me.opponent()).hand),
        "Fair opponent hand matches Draws (same seed)"
    );
    assert_eq!(
        zone_ids(&fair.player(me.opponent()).deck),
        zone_ids(&draws.player(me.opponent()).deck),
        "Fair opponent deck matches Draws (same seed)"
    );
}

#[test]
fn determinize_all_is_clone_plus_reseed() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let me = PlayerId::A;
    let seed = 13u64;
    let got = determinize_with(&st, me, seed, Info::All);
    assert_eq!(
        search_key(&got),
        search_key(&st),
        "All keeps the full state except rng"
    );
    assert_eq!(
        zone_ids(&got.player(me).hand),
        zone_ids(&st.player(me).hand)
    );
    assert_eq!(
        zone_ids(&got.player(me).deck),
        zone_ids(&st.player(me).deck)
    );
    assert_eq!(
        zone_ids(&got.player(me.opponent()).hand),
        zone_ids(&st.player(me.opponent()).hand)
    );
    assert_eq!(
        zone_ids(&got.player(me.opponent()).deck),
        zone_ids(&st.player(me.opponent()).deck)
    );
    let mut want = st.clone();
    want.rng.reseed(seed);
    assert_eq!(
        format!("{:?}", got.rng),
        format!("{:?}", want.rng),
        "All reseeds State.rng from seed"
    );
}

#[test]
fn fair_is_deterministic_given_seed() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let a = determinize_with(&st, PlayerId::A, 99, Info::Fair);
    let b = determinize_with(&st, PlayerId::A, 99, Info::Fair);
    assert_eq!(
        zone_ids(&a.player(PlayerId::A).deck),
        zone_ids(&b.player(PlayerId::A).deck)
    );
    assert_eq!(
        zone_ids(&a.player(PlayerId::B).hand),
        zone_ids(&b.player(PlayerId::B).hand)
    );
    assert_eq!(
        zone_ids(&a.player(PlayerId::B).deck),
        zone_ids(&b.player(PlayerId::B).deck)
    );
    assert_eq!(search_key(&a), search_key(&b));
}

#[test]
fn fair_varies_own_future_across_seeds() {
    let db = load_db();
    let st = mixed_info_state(&db);
    let me = PlayerId::A;
    let first = zone_ids(&determinize_with(&st, me, 1, Info::Fair).player(me).deck);
    let differs = (2u64..=16).any(|seed| {
        zone_ids(&determinize_with(&st, me, seed, Info::Fair).player(me).deck) != first
    });
    assert!(
        differs,
        "Fair must permute own deck order on at least one seed"
    );
}

#[test]
fn hidden_removals_logs_fuse_partners() {
    let db = load_db();
    let mut st = started(&db, 5);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    let partner_pos = put_hand(&db, &mut st, me, "90071220");
    let partner_id = st.player(me).hand[partner_pos as usize].id;
    apply(&db, &mut st, Action::Fuse { host: 0 }).unwrap();
    choose(&db, &mut st, 0);
    confirm(&db, &mut st);
    assert_eq!(st.player(me).hidden_removals, vec![partner_id]);
}

#[test]
fn hidden_removals_logs_deck_draw_overflow() {
    let db = load_db();
    let mut st = started(&db, 8);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    for _ in 0..9 {
        put_hand(&db, &mut st, me, "88001110");
    }
    put_deck(&db, &mut st, me, "10461110");
    let burned_id = st.player(me).deck[0].id;
    st.active = me;
    end_turn(&db, &mut st);
    end_turn(&db, &mut st);
    assert!(st.player(me).hidden_removals.contains(&burned_id));
}

#[test]
fn hidden_removals_skips_public_bounce_overflow() {
    let db = load_db();
    let mut st = started(&db, 9);
    let me = PlayerId::A;
    st.player_mut(me).hand.clear();
    for _ in 0..8 {
        put_hand(&db, &mut st, me, "88001110");
    }
    let _slot = put_field(&db, &mut st, me, "88001110");
    give_pp(&mut st, me, 10, 10);
    put_field(&db, &mut st, PlayerId::B, "88001320");
    let h = put_hand(&db, &mut st, me, "10012310");
    play(&db, &mut st, h);
    choose(&db, &mut st, 0);
    assert!(st.player(me).hidden_removals.is_empty());
}

#[test]
fn hidden_removals_discard_without_on_discard() {
    let db = load_db();
    let mut st = started(&db, 10);
    let me = PlayerId::A;
    let opp = me.opponent();
    st.player_mut(me).hand.clear();
    let silent_pos = put_hand(&db, &mut st, me, "88001110");
    let silent_id = st.player(me).hand[silent_pos as usize].id;
    let spell_pos = put_hand(&db, &mut st, me, "89200140");
    give_pp(&mut st, me, 10, 10);
    play(&db, &mut st, spell_pos);
    assert_eq!(st.player(me).hidden_removals, vec![silent_id]);

    let mut open_st = started(&db, 11);
    clear_hand(&mut open_st, opp);
    open_st.player_mut(opp).deck.clear();
    open_st.player_mut(opp).cemetery.clear();
    open_st.player_mut(opp).hidden_removals.clear();
    put_hand(&db, &mut open_st, opp, "10061120");
    put_deck(&db, &mut open_st, opp, "10461110");
    let hidden_id = open_st.alloc_id();
    let hidden_card = db.card(cid("88001110")).unwrap();
    let hidden = CardInstance::from_card(hidden_card, hidden_id);
    open_st.note_public_removal(opp, hidden.card);
    open_st.player_mut(opp).hidden_removals.push(hidden_id);
    open_st.player_mut(opp).cemetery.push(hidden);
    let mut seen = false;
    for seed in 1..=200u64 {
        let w = determinize_with(&open_st, me, seed, Info::Open);
        if w.player(opp)
            .hand
            .iter()
            .chain(w.player(opp).deck.iter())
            .any(|c| c.id == hidden_id)
        {
            seen = true;
            break;
        }
    }
    assert!(
        seen,
        "hidden discard must appear in opponent hand or deck on some seed"
    );
}

#[test]
fn hidden_removals_skip_discard_with_eld_blades() {
    let db = load_db();
    let eld = cid("10643310");
    assert!(db.has_card(eld), "Beheading Eld Blades must be in the db");
    let mut st = started(&db, 12);
    let me = PlayerId::A;
    let opp = me.opponent();
    st.player_mut(me).hand.clear();
    let eld_pos = put_hand(&db, &mut st, me, "10643310");
    let eld_id = st.player(me).hand[eld_pos as usize].id;
    let spell_pos = put_hand(&db, &mut st, me, "89200140");
    give_pp(&mut st, me, 10, 10);
    play(&db, &mut st, spell_pos);
    assert!(
        !st.player(me).hidden_removals.contains(&eld_id),
        "on-discard Eld Blades stays public"
    );

    let mut open_st = started(&db, 13);
    clear_hand(&mut open_st, opp);
    open_st.player_mut(opp).deck.clear();
    open_st.player_mut(opp).cemetery.clear();
    open_st.player_mut(opp).hidden_removals.clear();
    put_hand(&db, &mut open_st, opp, "10061120");
    put_deck(&db, &mut open_st, opp, "10461110");
    let public_id = open_st.alloc_id();
    let eld_card = db.card(eld).unwrap();
    let public_eld = CardInstance::from_card(eld_card, public_id);
    open_st.note_public_removal(opp, public_eld.card);
    open_st.player_mut(opp).cemetery.push(public_eld);
    for seed in 1..=200u64 {
        let w = determinize_with(&open_st, me, seed, Info::Open);
        assert!(
            !w.player(opp)
                .hand
                .iter()
                .chain(w.player(opp).deck.iter())
                .any(|c| c.id == public_id),
            "on-discard Eld Blades must not reappear in hand/deck at seed {seed}"
        );
    }
}

fn push_opp_cemetery(
    db: &CardDb,
    st: &mut State,
    opp: PlayerId,
    card: &str,
    hidden: bool,
) -> (usize, u32) {
    let index = st.player(opp).cemetery.len();
    let id = st.alloc_id();
    let inst = CardInstance::from_card(db.card(cid(card)).unwrap(), id);
    st.note_public_removal(opp, inst.card);
    if hidden {
        st.player_mut(opp).hidden_removals.push(id);
    }
    st.player_mut(opp).cemetery.push(inst);
    (index, id)
}

fn push_opp_banished(
    db: &CardDb,
    st: &mut State,
    opp: PlayerId,
    card: &str,
    hidden: bool,
) -> (usize, u32) {
    let index = st.player(opp).banished.len();
    let id = st.alloc_id();
    let inst = CardInstance::from_card(db.card(cid(card)).unwrap(), id);
    st.note_public_removal(opp, inst.card);
    if hidden {
        st.player_mut(opp).hidden_removals.push(id);
    }
    st.player_mut(opp).banished.push(inst);
    (index, id)
}

#[test]
fn open_hidden_slots_preserve_public_indices() {
    let db = load_db();
    let mut st = started(&db, 44);
    let me = PlayerId::A;
    let opp = me.opponent();
    clear_hand(&mut st, opp);
    st.player_mut(opp).deck.clear();
    st.player_mut(opp).cemetery.clear();
    st.player_mut(opp).banished.clear();
    st.player_mut(opp).hidden_removals.clear();
    put_hand(&db, &mut st, opp, "10061120");
    put_deck(&db, &mut st, opp, "10461110");
    put_deck(&db, &mut st, opp, "10061120");

    let (pub_cem0, pub_cem0_id) = push_opp_cemetery(&db, &mut st, opp, "88001110", false);
    push_opp_cemetery(&db, &mut st, opp, "10461110", true);
    let (pub_cem2, pub_cem2_id) = push_opp_cemetery(&db, &mut st, opp, "10061120", false);
    push_opp_cemetery(&db, &mut st, opp, "88001110", true);

    let (pub_ban0, pub_ban0_id) = push_opp_banished(&db, &mut st, opp, "10461110", false);
    push_opp_banished(&db, &mut st, opp, "10061120", true);

    let true_sizes = zone_sizes(&st, opp);
    assert_eq!(pub_cem0, 0);
    assert_eq!(pub_cem2, 2);
    assert_eq!(pub_ban0, 0);

    for seed in 1..=200u64 {
        let w = determinize_with(&st, me, seed, Info::Open);
        assert_eq!(zone_sizes(&w, opp), true_sizes, "seed {seed}");
        assert_eq!(
            w.player(opp).cemetery[pub_cem0].id,
            pub_cem0_id,
            "seed {seed}"
        );
        assert_eq!(
            w.player(opp).cemetery[pub_cem2].id,
            pub_cem2_id,
            "seed {seed}"
        );
        assert_eq!(
            w.player(opp).banished[pub_ban0].id,
            pub_ban0_id,
            "seed {seed}"
        );
    }
}

#[test]
fn open_hidden_removals_dedupes_and_skips_stale_ids() {
    let db = load_db();
    let mut st = started(&db, 45);
    let me = PlayerId::A;
    let opp = me.opponent();
    clear_hand(&mut st, opp);
    st.player_mut(opp).deck.clear();
    st.player_mut(opp).cemetery.clear();
    st.player_mut(opp).banished.clear();
    st.player_mut(opp).hidden_removals.clear();
    put_hand(&db, &mut st, opp, "10061120");
    put_deck(&db, &mut st, opp, "10461110");

    let (_, hidden_id) = push_opp_cemetery(&db, &mut st, opp, "88001110", true);
    st.player_mut(opp).hidden_removals.push(hidden_id);
    st.player_mut(opp).hidden_removals.push(hidden_id);
    st.player_mut(opp).hidden_removals.push(999_999);

    let true_sizes = zone_sizes(&st, opp);
    for seed in 1..=200u64 {
        let w = determinize_with(&st, me, seed, Info::Open);
        assert_eq!(zone_sizes(&w, opp), true_sizes, "seed {seed}");
    }
}

#[derive(Clone, Copy)]
enum OpenHiddenZone {
    Cemetery,
    Banished,
}

struct OpenFixture {
    me: PlayerId,
    host_id: u32,
    partner_id: u32,
    burned_id: u32,
    revealed_discard_id: u32,
    public_card: CardId,
    /// Recorded `(zone, index)` slots for privately removed cards.
    r_positions: Vec<(OpenHiddenZone, usize)>,
}

fn open_world_fixture(db: &CardDb) -> (arena_engine::State, OpenFixture) {
    let mut st = started(db, 42);
    let me = PlayerId::A;
    let opp = me.opponent();
    clear_hand(&mut st, opp);
    st.player_mut(opp).deck.clear();
    st.player_mut(opp).cemetery.clear();
    st.player_mut(opp).banished.clear();
    st.player_mut(opp).hidden_removals.clear();
    st.player_mut(opp).public_hand_additions.clear();
    st.player_mut(opp).public_removals.clear();

    let public_card = cid("88001110");
    let pub_inst = CardInstance::from_card(db.card(public_card).unwrap(), st.alloc_id());
    st.player_mut(opp).hand.push(pub_inst);
    st.note_public_addition(opp, public_card);

    let host_id = st.alloc_id();
    let host_card = db.card(cid("90071210")).unwrap();
    let mut host = CardInstance::from_card(host_card, host_id);
    host.flags.was_fused = true;
    st.player_mut(opp).hand.push(host);

    put_hand(db, &mut st, opp, "10061120");
    put_deck(db, &mut st, opp, "10461110");
    put_deck(db, &mut st, opp, "10061120");

    let partner_id = st.alloc_id();
    let partner_card = db.card(cid("90071220")).unwrap();
    let partner = CardInstance::from_card(partner_card, partner_id);
    st.note_public_removal(opp, partner.card);
    st.player_mut(opp).hidden_removals.push(partner_id);
    st.player_mut(opp).banished.push(partner);
    let partner_slot = (OpenHiddenZone::Banished, 0usize);

    let burned_id = st.alloc_id();
    let burned_card = db.card(cid("10461110")).unwrap();
    let burned = CardInstance::from_card(burned_card, burned_id);
    st.note_public_removal(opp, burned.card);
    st.player_mut(opp).hidden_removals.push(burned_id);
    st.player_mut(opp).cemetery.push(burned);
    let burned_slot = (OpenHiddenZone::Cemetery, 0usize);

    let revealed_discard_id = st.alloc_id();
    let revealed_card = db.card(cid("10643310")).unwrap();
    let revealed = CardInstance::from_card(revealed_card, revealed_discard_id);
    st.note_public_removal(opp, revealed.card);
    st.player_mut(opp).cemetery.push(revealed);

    (
        st,
        OpenFixture {
            me,
            host_id,
            partner_id,
            burned_id,
            revealed_discard_id,
            public_card,
            r_positions: vec![burned_slot, partner_slot],
        },
    )
}

fn zone_sizes(st: &State, who: PlayerId) -> (usize, usize, usize, usize) {
    let p = st.player(who);
    (
        p.hand.len(),
        p.deck.len(),
        p.cemetery.len(),
        p.banished.len(),
    )
}

fn opp_open_multiset(
    st: &State,
    opp: PlayerId,
    r_positions: &[(OpenHiddenZone, usize)],
) -> BTreeMap<(u32, u32), u32> {
    let mut m = zone_multiset(&st.player(opp).hand);
    for c in &st.player(opp).deck {
        *m.entry((c.card.0, c.id)).or_insert(0) += 1;
    }
    for &(zone, idx) in r_positions {
        let inst = match zone {
            OpenHiddenZone::Cemetery => &st.player(opp).cemetery[idx],
            OpenHiddenZone::Banished => &st.player(opp).banished[idx],
        };
        *m.entry((inst.card.0, inst.id)).or_insert(0) += 1;
    }
    m
}

#[test]
fn open_world_preserves_zones_and_multiset() {
    let db = load_db();
    let (st, fx) = open_world_fixture(&db);
    let opp = fx.me.opponent();
    let true_sizes = zone_sizes(&st, PlayerId::A);
    let true_opp = zone_sizes(&st, opp);
    let true_ms = opp_open_multiset(&st, opp, &fx.r_positions);
    let mut partner_seen = false;
    let mut burned_seen = false;
    for seed in 1..=200u64 {
        let w = determinize_with(&st, fx.me, seed, Info::Open);
        assert_eq!(zone_sizes(&w, PlayerId::A), true_sizes, "seed {seed}");
        assert_eq!(zone_sizes(&w, opp), true_opp, "seed {seed}");
        assert_eq!(
            opp_open_multiset(&w, opp, &fx.r_positions),
            true_ms,
            "seed {seed}"
        );
        assert!(
            w.player(opp)
                .hand
                .iter()
                .any(|c| c.id == fx.host_id && c.flags.was_fused),
            "host fixed in hand at seed {seed}"
        );
        assert!(
            w.player(opp).hand.iter().any(|c| c.card == fx.public_card),
            "public addition stays in hand at seed {seed}"
        );
        assert!(
            !w.player(opp)
                .hand
                .iter()
                .chain(w.player(opp).deck.iter())
                .any(|c| c.id == fx.revealed_discard_id),
            "on-discard card never in hand/deck at seed {seed}"
        );
        if w.player(opp)
            .hand
            .iter()
            .chain(w.player(opp).deck.iter())
            .any(|c| c.id == fx.partner_id)
        {
            partner_seen = true;
        }
        if w.player(opp)
            .hand
            .iter()
            .chain(w.player(opp).deck.iter())
            .any(|c| c.id == fx.burned_id)
        {
            burned_seen = true;
        }
        let again = determinize_with(&st, fx.me, seed, Info::Open);
        assert_eq!(
            zone_ids(&w.player(opp).hand),
            zone_ids(&again.player(opp).hand),
            "seed-stable hand at {seed}"
        );
        assert_eq!(
            zone_ids(&w.player(opp).deck),
            zone_ids(&again.player(opp).deck),
            "seed-stable deck at {seed}"
        );
    }
    assert!(
        partner_seen,
        "partner must appear in hand or deck on some seed"
    );
    assert!(
        burned_seen,
        "burned card must appear in hand or deck on some seed"
    );
}

#[test]
fn open_stats_count_hidden_and_hosts() {
    let db = load_db();
    let (st, fx) = open_world_fixture(&db);
    let mut stats = OpenStats::default();
    determinize_with_stats(&st, fx.me, 7, Info::Open, Some(&mut stats));
    assert_eq!(stats.hosts, 1);
    assert!(
        stats.hidden > 0,
        "hidden removals should sometimes deal into play"
    );
}

#[test]
fn spec_info_round_trips() {
    assert_eq!(AnyPolicy::parse_spec("h0").unwrap().spec(), "h0");
    assert_eq!(AnyPolicy::parse_spec("h0:info=open").unwrap().spec(), "h0");
    assert_eq!(
        AnyPolicy::parse_spec("h0:info=fair").unwrap().spec(),
        "h0:info=fair"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:info=draws").unwrap().spec(),
        "h0:info=draws"
    );
    assert_eq!(
        AnyPolicy::parse_spec("h0:info=all").unwrap().spec(),
        "h0:info=all"
    );
    assert_eq!(AnyPolicy::parse_spec("h0-fast").unwrap().spec(), "h0-fast");
    let again = AnyPolicy::parse_spec("h0:info=open").unwrap();
    assert_eq!(AnyPolicy::parse_spec(&again.spec()).unwrap(), again);
    let e = AnyPolicy::parse_spec("h0:info=bogus").unwrap_err();
    assert!(e.contains("info"), "info=bogus → {e}");
    assert!(
        e.contains("open") && e.contains("fair") && e.contains("draws") && e.contains("all"),
        "error must list the values: {e}"
    );
}

#[test]
fn all_builds_one_root() {
    let db = load_db();
    let st = started(&db, 1);
    let legal = legal_actions(&db, &st);
    assert!(legal.len() > 1, "need a searched decision");
    let mut all = parse_h0("h0:info=all");
    assert_eq!(all.determinizations, 4);
    let mut rng = policy_rng(1);
    all.choose(&db, &st, &legal, &mut rng);
    assert_eq!(all.stats.roots, 1, "info=all must build one root");
    let mut draws = parse_h0("h0:info=draws");
    let mut rng = policy_rng(1);
    draws.choose(&db, &st, &legal, &mut rng);
    assert_eq!(draws.stats.roots, 4, "info=draws still builds k roots");
}

fn play_pair_spec(
    db: &CardDb,
    spec_a: &str,
    spec_b: &str,
    n: u32,
    seed: u64,
) -> Vec<(Option<PlayerId>, u32, u32)> {
    let decks = load_deck_file("oracle/decks/basic-forest.json");
    assert!(deck_ready(db, &decks));
    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let first = if i % 2 == 0 { First::A } else { First::B };
        let mut state = new_game(
            db,
            GameConfig {
                seed: seed.wrapping_add(u64::from(i)),
                deck_a: decks.clone(),
                deck_b: decks.clone(),
                first,
                opening_hands: None,
            },
        )
        .unwrap();
        let mut rng = policy_rng(seed.wrapping_add(u64::from(i)));
        let mut a = parse_h0(spec_a);
        let mut b = parse_h0(spec_b);
        let o = play_game(db, &mut state, &mut a, &mut b, &mut rng);
        out.push((o.winner, o.turns, o.actions));
    }
    out
}

fn check_determinism(spec: &str, n: u32) {
    let db = load_db();
    const SEED: u64 = 20260919;
    let first = play_pair_spec(&db, spec, spec, n, SEED);
    let again = play_pair_spec(&db, spec, spec, n, SEED);
    assert_eq!(first, again, "{spec} must be deterministic");
}

#[test]
fn info_fair_determinism_smoke() {
    check_determinism("h0:info=fair,depth=2,nodes=200", 2);
}

#[test]
fn info_all_determinism_smoke() {
    check_determinism("h0:info=all,depth=2,nodes=200", 2);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn info_fair_determinism_20_games() {
    check_determinism("h0:info=fair", 20);
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn info_all_determinism_20_games() {
    check_determinism("h0:info=all", 20);
}
