//! Equivalence checks for the h0 speed refactor (snapshot hash, memo zones, pools, net, rng, db, play_form).
//!
//! Fast tests run in the `ci` job; the meta walk runs in `slow.yml` (`suite: h0_zb_equiv`).

use std::collections::HashSet;
use std::sync::Arc;

use arena_engine::apply::{play_form_effects_reference, play_form_for_equiv};
use arena_engine::card::{Card, Mode};
use arena_engine::determinize::{determinize_with, Info};
use arena_engine::shared::CanonicalBytes;
use arena_engine::snapshot::{hash, hash_via_value};
use arena_engine::trace::{Pick, PickChose, PickWhat};
use arena_engine::{
    apply, apply_known_legal, builtin_net, builtin_order, legal_actions, new_game, policy_rng,
    AnyPolicy, CardDb, CardId, First, GameConfig, GameRng, Observation, OraclePickNotLegal, Phase,
    PlayerId, Policy, ValueNet,
};

mod common;
use common::*;

const OPEN_WORLDS: u64 = 8;

fn meta_deck_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: serde_json::Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .collect()
}

fn meta_decks(db: &CardDb) -> Vec<Vec<CardId>> {
    meta_deck_stems()
        .iter()
        .map(|stem| load_deck_file(format!("oracle/decks/{stem}.json")))
        .filter(|d| deck_ready(db, d) && d.len() == 40)
        .collect()
}

fn equiv_nets() -> [Arc<ValueNet>; 3] {
    let tiny = fixtures_dir().join("net/tiny-linear.json");
    [
        builtin_net(),
        builtin_order(),
        ValueNet::load(tiny).expect("tiny-linear"),
    ]
}

fn assert_player_memo_zones(p: &arena_engine::PlayerState) {
    macro_rules! check {
        ($zone:expr) => {{
            let direct = CanonicalBytes::canonical_bytes(&*$zone);
            assert_eq!($zone.canonical(), direct.as_slice(), "memo canonical");
        }};
    }
    check!(p.leader_mods);
    check!(p.crests);
    check!(p.hand);
    check!(p.deck);
    check!(p.cemetery);
    check!(p.banished);
    check!(p.destroyed_history);
    check!(p.played_this_turn);
    check!(p.played_base_costs_this_match);
    check!(p.enter_counts);
    check!(p.starting_deck);
    check!(p.public_removals);
    check!(p.public_hand_additions);
}

fn check_pools(p: &arena_engine::PlayerState) {
    let strict: Vec<(CardId, u32)> = p.strict_remaining_pool().into_iter().collect();
    assert_eq!(
        strict,
        p.strict_remaining_pool_sorted(),
        "strict pool sorted"
    );
    let known: Vec<(CardId, u32)> = p.known_remaining_pool().into_iter().collect();
    assert_eq!(known, p.known_remaining_pool_sorted(), "known pool sorted");
}

fn pp_samples(card: &Card, inst: &arena_engine::CardInstance, usable: i32) -> Vec<i32> {
    let mut out = HashSet::new();
    out.insert(usable);
    out.insert(inst.cost);
    if inst.cost > 0 {
        out.insert(inst.cost - 1);
    }
    for m in card.modes() {
        let c = if let Mode::Enhance { cost, .. } = m {
            *cost
        } else if let Mode::Accelerate { cost, .. } = m {
            *cost
        } else if let Mode::Crystallize { cost, .. } = m {
            *cost
        } else {
            continue;
        };
        out.insert(c);
        if c > 0 {
            out.insert(c - 1);
        }
    }
    out.into_iter().collect()
}

fn check_play_form_hand(
    db: &CardDb,
    state: &arena_engine::State,
    me: PlayerId,
    static_in_play: bool,
) {
    let usable = state.player(me).usable_pp();
    for inst in state.player(me).hand.iter() {
        let Ok(card) = db.card(inst.card) else {
            continue;
        };
        for pp in pp_samples(card, inst, usable) {
            let got = play_form_for_equiv(db, state, me, card, inst, pp, static_in_play);
            let expect =
                play_form_effects_reference(db, state, me, card, inst, pp, static_in_play);
            assert_eq!(
                got, expect,
                "play_form {:?} pp={} static={}",
                inst.card, pp, static_in_play
            );
        }
    }
}

fn obs_with_neg_zero(obs: &Observation) -> Observation {
    let mut o = obs.clone();
    if let Some(i) = o.features.iter().position(|f| f.to_bits() == 0) {
        o.features[i] = -0.0;
    }
    o
}

fn check_value_nets(nets: &[Arc<ValueNet>], observations: &[Observation]) {
    let mut hit = [false; 3];
    for (ni, net) in nets.iter().enumerate() {
        for obs in observations {
            for clip in [0.0f32, 5.0, 2.5] {
                let fast = net.value_clipped(obs, clip);
                let reference = net.value_standardize_forward(obs, clip);
                assert_eq!(
                    fast.to_bits(),
                    reference.to_bits(),
                    "net {ni} clip={clip}"
                );
                hit[ni] = true;
            }
            let neg = obs_with_neg_zero(obs);
            for clip in [0.0f32, 5.0] {
                let fast = net.value_clipped(&neg, clip);
                let reference = net.value_standardize_forward(&neg, clip);
                assert_eq!(fast.to_bits(), reference.to_bits(), "neg0 net {ni} clip={clip}");
            }
        }
    }
    for (ni, h) in hit.iter().enumerate() {
        assert!(h, "net {ni} was not exercised");
    }
}

fn collect_observations(
    db: &CardDb,
    state: &arena_engine::State,
    buf: &mut Vec<Observation>,
) {
    for me in [PlayerId::A, PlayerId::B] {
        buf.push(arena_engine::encode_version(
            state,
            me,
            arena_engine::EncodingVersion::V3,
            Some(db),
        ));
        for w in 0..OPEN_WORLDS {
            let world = determinize_with(state, me, w + 1, Info::Open);
            for p in &world.players {
                check_pools(p);
            }
            buf.push(arena_engine::encode_version(
                &world,
                me,
                arena_engine::EncodingVersion::V3,
                Some(db),
            ));
        }
    }
}

fn check_state(
    db: &CardDb,
    state: &arena_engine::State,
    nets: &[Arc<ValueNet>],
    obs_buf: &mut Vec<Observation>,
    checked: &mut u64,
) {
    assert_eq!(hash(state), hash_via_value(state), "snapshot hash");
    *checked += 1;

    for p in &state.players {
        assert_player_memo_zones(p);
        check_pools(p);
    }
    for me in [PlayerId::A, PlayerId::B] {
        check_play_form_hand(db, state, me, false);
        check_play_form_hand(db, state, me, true);
    }

    for me in [PlayerId::A, PlayerId::B] {
        for w in 0..OPEN_WORLDS {
            let world = determinize_with(state, me, w + 1, Info::Open);
            assert_eq!(
                hash(&world),
                hash_via_value(&world),
                "open world {w} {me:?}"
            );
            *checked += 1;
            for p in &world.players {
                check_pools(p);
            }
        }
    }

    let start = obs_buf.len();
    collect_observations(db, state, obs_buf);
    check_value_nets(nets, &obs_buf[start..]);
}

fn walk_self_play(db: &CardDb, decks: &[Vec<CardId>], game_count: usize) -> u64 {
    let nets = equiv_nets();
    let net_refs: Vec<Arc<ValueNet>> = nets.to_vec();
    let mut checked = 0u64;
    let mut obs_buf = Vec::new();
    assert!(decks.len() >= 16, "need 16 meta decks, got {}", decks.len());

    for gi in 0..game_count {
        let da = &decks[gi % decks.len()];
        let dbk = &decks[(gi / 3 + 1) % decks.len()];
        let seed = 100 + gi as u64 * 17;
        let mut state = new_game(
            db,
            GameConfig {
                seed,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("game");
        let mut pol = AnyPolicy::parse("h0:nodes=200,k=2");
        let mut rng = policy_rng(seed + 7);
        let mut steps = 0;
        obs_buf.clear();

        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) && steps < 300 {
            check_state(db, &state, &net_refs, &mut obs_buf, &mut checked);

            let legal = legal_actions(db, &state);
            if legal.is_empty() {
                break;
            }
            for act in &legal {
                let mut s = state.clone();
                if apply(db, &mut s, act.clone()).is_ok() {
                    check_state(db, &s, &net_refs, &mut obs_buf, &mut checked);
                }
            }
            let i = pol
                .choose(db, &state, &legal, &mut rng)
                .min(legal.len() - 1);
            apply_known_legal(db, &mut state, legal[i].clone()).expect("apply");
            steps += 1;
        }

        if let Some(inst) = state.player(PlayerId::A).hand.first().cloned() {
            let mut s = state.clone();
            s.player_mut(PlayerId::A).hand.push(inst);
            assert_player_memo_zones(s.player(PlayerId::A));
        }
    }
    checked
}

fn assert_same_pick_err(
    a: Result<usize, OraclePickNotLegal>,
    b: Result<usize, OraclePickNotLegal>,
) {
    match (a, b) {
        (Ok(ia), Ok(ib)) => assert_eq!(ia, ib),
        (Err(ea), Err(eb)) => {
            assert_eq!(ea.what, eb.what);
            assert_eq!(ea.chose, eb.chose);
            assert_eq!(ea.candidates, eb.candidates);
        }
        (a, b) => panic!("mismatched ok/err: {a:?} vs {b:?}"),
    }
}

fn live_pick_equiv(
    seed: u64,
    among: Option<&str>,
    candidates: &[CardId],
) {
    let mut a = GameRng::live(seed);
    let mut b = GameRng::live(seed);
    let mut emit_a = Vec::new();
    let mut emit_b = Vec::new();
    let ia = a
        .pick_index_among_ids(PickWhat::Draw, among, candidates, &mut emit_a)
        .unwrap();
    let keys: Vec<String> = candidates.iter().map(|c| c.as_str()).collect();
    let ib = b
        .pick_index_among(PickWhat::Draw, among, &keys, &mut emit_b)
        .unwrap();
    assert_eq!(ia, ib, "live seed {seed}");
    assert_eq!(emit_a, emit_b);
    for _ in 0..3 {
        assert_eq!(
            a.gen_range(1_000_000),
            b.gen_range(1_000_000),
            "rng drift seed {seed}"
        );
    }
}

#[test]
fn pick_index_among_ids_matches_among_live_and_scripted() {
    let id_a = CardId::parse("88001110").unwrap();
    let id_b = CardId::parse("88001120").unwrap();
    let id_c = CardId::parse("88001130").unwrap();
    let triple = [id_a, id_b, id_c];
    let keys: Vec<String> = triple.iter().map(|c| c.as_str()).collect();

    for n in [1usize, 2, 40] {
        let mut candidates: Vec<CardId> = Vec::new();
        for i in 0..n {
            candidates.push(triple[i % 3]);
        }
        for seed in [1u64, 99, 4242] {
            live_pick_equiv(seed, Some("deck"), &candidates);
            live_pick_equiv(seed, None, &candidates);
        }
    }

    let picks = vec![Pick {
        what: PickWhat::Draw,
        among: Some("deck".into()),
        chose: PickChose::Id("88001120".into()),
    }];
    for among in [None, Some("deck")] {
        let mut sa = GameRng::scripted(picks.clone(), 0);
        let mut sb = GameRng::scripted(picks.clone(), 0);
        let mut emit_a = Vec::new();
        let mut emit_b = Vec::new();
        let ia = sa
            .pick_index_among_ids(PickWhat::Draw, among, &triple, &mut emit_a)
            .unwrap();
        let ib = sb
            .pick_index_among(PickWhat::Draw, among, &keys, &mut emit_b)
            .unwrap();
        assert_eq!(ia, ib);
        assert_eq!(ia, 1);
    }

    let bad = vec![Pick {
        what: PickWhat::Draw,
        among: None,
        chose: PickChose::Id("99999999".into()),
    }];
    for among in [None, Some("deck")] {
        let mut sa = GameRng::scripted(bad.clone(), 0);
        let mut sb = GameRng::scripted(bad.clone(), 0);
        let mut emit_a = Vec::new();
        let mut emit_b = Vec::new();
        assert_same_pick_err(
            sa.pick_index_among_ids(PickWhat::Draw, among, &triple, &mut emit_a),
            sb.pick_index_among(PickWhat::Draw, among, &keys, &mut emit_b),
        );
    }

    let mut sa = GameRng::scripted(vec![], 0);
    let mut sb = GameRng::scripted(vec![], 0);
    let mut emit_a = Vec::new();
    let mut emit_b = Vec::new();
    assert_same_pick_err(
        sa.pick_index_among_ids(PickWhat::Draw, Some("deck"), &triple, &mut emit_a),
        sb.pick_index_among(PickWhat::Draw, Some("deck"), &keys, &mut emit_b),
    );
}

#[test]
fn card_is_unsupported_matches_support_fn() {
    let db = load_db();
    for (id, card) in &db.cards {
        let table = db.card_is_unsupported(*id);
        let direct = arena_engine::support::card_unsupported(card).is_some();
        assert_eq!(table, direct, "{id:?}");
    }
}

#[test]
fn vocab_id_index_matches_binary_search() {
    let nets = equiv_nets();
    let mut extra: Vec<u32> = (0..10_000).collect();
    let mut rng = policy_rng(0x5eed);
    for _ in 0..2_000 {
        extra.push(rng.next_u64() as u32);
    }
    for net in &nets {
        for &id in net.vocab_ids() {
            let bs = net.vocab_ids().binary_search(&id).expect("in vocab");
            assert_eq!(net.vocab_id_index(id), bs);
        }
        for &id in &extra {
            let bs = net.vocab_ids().binary_search(&id).unwrap_or(0);
            assert_eq!(net.vocab_id_index(id), bs, "id {id}");
        }
    }
}

#[test]
#[ignore = "slow.yml suite h0_zb_equiv"]
fn zb_equiv_meta_walk_snapshot_pools_net_play_form() {
    let db = load_db();
    let decks = meta_decks(&db);
    let checked = walk_self_play(&db, &decks, 18);
    eprintln!("states checked: {checked}");
    assert!(checked > 10_000);
}

#[test]
#[ignore = "slow.yml suite h0_zb_equiv"]
fn sorted_pools_match_btreemap_on_meta_walk() {
    let db = load_db();
    let decks = meta_decks(&db);
    let mut n = 0u64;
    for gi in 0..8 {
        let da = &decks[gi % decks.len()];
        let dbk = &decks[(gi + 5) % decks.len()];
        let mut state = new_game(
            &db,
            GameConfig {
                seed: 50 + gi as u64,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("game");
        let mut rng = policy_rng(gi as u64);
        for _ in 0..40 {
            for p in &state.players {
                check_pools(p);
                n += 1;
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let i = rng.gen_range(legal.len() as u32) as usize;
            let _ = apply(&db, &mut state, legal[i].clone());
        }
    }
    assert!(n > 100);
}
