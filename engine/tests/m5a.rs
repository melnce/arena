//! M5a fixtures: ActionId, encode, search_key, determinize, H0.

use std::collections::BTreeSet;
use std::path::PathBuf;

use arena_engine::{
    apply, determinize, encode, legal_actions, legal_ids, legal_mask, new_game, policy_rng,
    search_key, Action, ActionId, AttackTarget, CardId, First, GameConfig, Observation, Phase,
    PlayerId, Policy, Random, Slot, H0, MAX_ACTIONS, MAX_TURNS,
};

mod common;
use common::*;

fn counts(ids: &[CardId]) -> std::collections::BTreeMap<CardId, u32> {
    let mut m = std::collections::BTreeMap::new();
    for id in ids {
        *m.entry(*id).or_insert(0) += 1;
    }
    m
}

fn oracle_decks() -> Vec<PathBuf> {
    let dir = repo_root().join("oracle/decks");
    let mut v: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("oracle/decks")
        .filter_map(|e| {
            let p = e.ok()?.path();
            (p.extension()? == "json").then_some(p)
        })
        .collect();
    v.sort();
    v
}

fn collect_states(
    db: &arena_engine::CardDb,
    n: usize,
    midgame_only: bool,
) -> (Vec<arena_engine::State>, usize) {
    let decks: Vec<Vec<CardId>> = oracle_decks()
        .into_iter()
        .map(load_deck_file)
        .filter(|d| deck_ready(db, d) && d.len() == 40)
        .collect();
    assert!(
        !decks.is_empty(),
        "need at least one oracle deck of 40 cards"
    );
    let mut out = Vec::with_capacity(n);
    let mut max_choice = 0usize;
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
            max_choice = max_choice.max(arena_engine::encode::choice_option_len(&state));
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
    (out, max_choice)
}

#[test]
fn action_id_table_and_legal_mask() {
    let db = load_db();
    let (states, max_choice) = collect_states(&db, 20_000, false);
    assert_eq!(states.len(), 20_000, "could not reach 20k states");
    eprintln!("m5a max ChoiceNode options in 20k-state sample: {max_choice}");
    assert!(
        max_choice <= ActionId::CHOOSE_N,
        "choice options {max_choice} exceed Choose cap {}",
        ActionId::CHOOSE_N
    );
    assert_eq!(ActionId::COUNT, 114);
    for state in &states {
        let legal = legal_actions(&db, state);
        let mask = legal_mask(&db, state);
        let ids = legal_ids(&db, state);
        let mut true_bits = 0usize;
        for (i, bit) in mask.iter().enumerate() {
            if *bit {
                true_bits += 1;
                let a = ActionId(i as u8).to_action();
                assert!(
                    legal.contains(&a),
                    "mask bit {i} set but {a:?} not in legal_actions"
                );
            }
        }
        assert_eq!(true_bits, legal.len());
        assert_eq!(ids.len(), legal.len());
        for (a, id) in legal.iter().zip(ids.iter()) {
            assert_eq!(ActionId::from_action(a), *id);
            assert_eq!(&id.to_action(), a, "from_action ∘ to_action");
        }
        for a in &legal {
            assert_eq!(&ActionId::from_action(a).to_action(), a);
        }
    }
}

#[test]
fn observation_layout_and_masking() {
    let db = load_db();
    let (states, _) = collect_states(&db, 20_000, false);
    let mut last_off = 0usize;
    for f in Observation::LAYOUT {
        assert_eq!(f.offset, last_off, "{}", f.name);
        last_off += f.width;
    }
    assert_eq!(last_off, Observation::LEN);

    for state in &states {
        for who in PlayerId::ALL {
            let a = encode(state, who);
            let b = encode(state, who);
            assert_eq!(a, b, "encode is deterministic");
            assert_eq!(a.features.len(), Observation::LEN);
            assert_eq!(a.ids.len(), Observation::IDS_LEN);
            let hand = Observation::IDS_OPP_HAND;
            for id in &a.ids[hand..hand + 9] {
                assert_eq!(*id, 0, "opponent hand id leaked");
            }
            let opp = state.player(who.opponent());
            let pool: u32 = opp.known_remaining_pool().values().sum();
            let hidden = opp.hand.len() + opp.deck.len();
            let mut start = counts(&opp.starting_deck);
            for id in &opp.public_hand_additions {
                *start.entry(*id).or_insert(0) += 1;
            }
            let mut subtracted = 0usize;
            for id in &opp.public_removals {
                if let Some(n) = start.get_mut(id) {
                    if *n > 0 {
                        *n -= 1;
                        subtracted += 1;
                    }
                }
            }
            assert_eq!(
                pool as usize,
                opp.starting_deck.len() + opp.public_hand_additions.len() - subtracted,
                "known pool == decklist + additions − public removals"
            );
            let _ = hidden;
            let hist: f32 = a.features[353 + arena_engine::encode::HIST_WIDTH..Observation::LEN]
                .iter()
                .sum();
            assert!(
                (hist - pool as f32).abs() < 1e-3
                    || opp.known_remaining_pool().len() > arena_engine::encode::HIST_WIDTH,
                "histogram sum {hist} vs pool {pool}"
            );
        }
    }
}

#[test]
fn search_key_sees_hidden_and_ignores_rng() {
    let db = load_db();
    let mut slaus = started_decks(&db, 1, &["10574110"], &["88001110"]);
    clear_hand(&mut slaus, PlayerId::A);
    let slot = put_field(&db, &mut slaus, PlayerId::A, "10574110");
    let k0 = search_key(&slaus);
    slaus.player_mut(PlayerId::A).field[slot as usize]
        .as_mut()
        .unwrap()
        .choose_used
        .insert(0);
    let k1 = search_key(&slaus);
    assert_ne!(k0, k1, "choose_used must change search_key");

    let mut once = started(&db, 2);
    let slot = put_field(&db, &mut once, PlayerId::A, "10001110");
    let k0 = search_key(&once);
    once.player_mut(PlayerId::A).field[slot as usize]
        .as_mut()
        .unwrap()
        .once_used
        .push(arena_engine::card::TriggerTag::Fanfare);
    let k1 = search_key(&once);
    assert_ne!(k0, k1, "once_used must change search_key");

    let mut hand = started(&db, 3);
    put_hand(&db, &mut hand, PlayerId::A, "10001110");
    let k0 = search_key(&hand);
    hand.player_mut(PlayerId::A)
        .hand
        .last_mut()
        .unwrap()
        .once_used
        .push(arena_engine::card::TriggerTag::Spellboost);
    let k1 = search_key(&hand);
    assert_ne!(k0, k1, "hand-zone oncePerTurn flag must change search_key");

    let mut mulla = new_game(
        &db,
        GameConfig {
            seed: 4,
            deck_a: pad_deck(&["88001110"], 40),
            deck_b: pad_deck(&["88001110"], 40),
            first: First::A,
            opening_hands: None,
        },
    )
    .unwrap();
    let k0 = search_key(&mulla);
    apply(
        &db,
        &mut mulla,
        Action::MulliganConfirm { swap: [false; 4] },
    )
    .unwrap();
    let k1 = search_key(&mulla);
    assert_ne!(k0, k1, "mulligan actor must change search_key");
    assert!(matches!(
        mulla.phase,
        Phase::Mulligan {
            player: PlayerId::B
        }
    ));

    let s = started(&db, 5);
    assert_eq!(search_key(&s), search_key(&s.clone()));

    let mut a = started(&db, 6);
    let mut b = a.clone();
    arena_engine::reseed(&mut a, 99);
    arena_engine::reseed(&mut b, 100);
    assert_eq!(
        search_key(&a),
        search_key(&b),
        "RNG-only differences must not change search_key"
    );
}

#[test]
fn determinize_preserves_observation() {
    let db = load_db();
    let (states, _) = collect_states(&db, 5_000, true);
    assert!(
        states.len() >= 100,
        "need mid-game states, got {}",
        states.len()
    );
    let mut saw_diff_hand = false;
    for (i, state) in states.iter().enumerate() {
        let p = PlayerId::A;
        let e0 = encode(state, p);
        let d1 = determinize(state, p, 1000 + i as u64);
        let d2 = determinize(state, p, 2000 + i as u64);
        assert_eq!(encode(&d1, p), e0, "determinize must preserve encode");
        assert_eq!(encode(&d2, p), e0);
        assert_eq!(
            d1.player(p).hand.iter().map(|c| c.id).collect::<Vec<_>>(),
            state
                .player(p)
                .hand
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            d1.player(p).deck.iter().map(|c| c.id).collect::<Vec<_>>(),
            state
                .player(p)
                .deck
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>()
        );
        let opp = p.opponent();
        assert_eq!(d1.player(opp).hand.len(), state.player(opp).hand.len());
        let _pool: u32 = state.player(opp).known_remaining_pool().values().sum();
        let h1: BTreeSet<u32> = d1.player(opp).hand.iter().map(|c| c.id).collect();
        let h2: BTreeSet<u32> = d2.player(opp).hand.iter().map(|c| c.id).collect();
        if h1 != h2 {
            saw_diff_hand = true;
        }
    }
    assert!(
        saw_diff_hand,
        "two seeds should differ in opponent hand at least once"
    );
}

fn h0_pick(db: &arena_engine::CardDb, state: &arena_engine::State) -> Action {
    let legal = legal_actions(db, state);
    assert!(!legal.is_empty());
    let mut h0 = H0::default();
    let mut rng = policy_rng(1);
    let i = h0.choose(db, state, &legal, &mut rng);
    legal[i].clone()
}

#[test]
fn h0_takes_storm_lethal() {
    let db = load_db();
    let mut st = started_decks(&db, 11, &["10461110"], &["88001110"]);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let slot = put_field(&db, &mut st, PlayerId::A, "10461110");
    {
        let f = st.field_inst_mut(PlayerId::A, slot).unwrap();
        f.flags.summoning_sick = false;
        f.flags.attacks_left = 1;
    }
    st.player_mut(PlayerId::B).leader_defense = 2;
    let a = h0_pick(&db, &st);
    assert!(
        matches!(
            a,
            Action::Attack {
                target: AttackTarget::Leader,
                ..
            }
        ),
        "expected face attack, got {a:?}"
    );
}

#[test]
fn h0_evolve_then_attack_lethal() {
    let db = load_db();
    let mut st = started_decks(&db, 12, &["10001110"], &["88001110"]);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let slot = put_field(&db, &mut st, PlayerId::A, "10001110");
    {
        let f = st.field_inst_mut(PlayerId::A, slot).unwrap();
        f.flags.summoning_sick = false;
        f.flags.attacks_left = 1;
        f.attack = 2;
        f.defense = 2;
        f.max_defense = 2;
        f.evolved = false;
    }
    set_round(&mut st, PlayerId::A, 5);
    st.player_mut(PlayerId::A).ep = 1;
    st.player_mut(PlayerId::A).evolved_this_turn = false;
    st.player_mut(PlayerId::B).leader_defense = 4;
    let first = h0_pick(&db, &st);
    assert!(
        matches!(
            first,
            Action::Evolve {
                super_evolve: false,
                ..
            }
        ),
        "expected evolve, got {first:?}"
    );
    apply(&db, &mut st, first).unwrap();
    let second = h0_pick(&db, &st);
    assert!(
        matches!(
            second,
            Action::Attack {
                target: AttackTarget::Leader,
                ..
            }
        ),
        "expected face after evolve, got {second:?}"
    );
}

#[test]
fn h0_takes_burn_spell_lethal() {
    let db = load_db();
    let mut st = started_decks(&db, 13, &["10743310"], &["88001110"]);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    give_pp(&mut st, PlayerId::A, 10, 10);
    put_hand(&db, &mut st, PlayerId::A, "10743310");
    st.player_mut(PlayerId::B).leader_defense = 3;
    let a = h0_pick(&db, &st);
    assert!(matches!(a, Action::Play { .. }), "expected play, got {a:?}");
}

#[test]
fn h0_does_not_attack_into_ward_when_face_is_safe() {
    let db = load_db();
    let mut st = started(&db, 14);
    clear_hand(&mut st, PlayerId::A);
    clear_hand(&mut st, PlayerId::B);
    let mine = put_field(&db, &mut st, PlayerId::A, "10001110");
    {
        let f = st.field_inst_mut(PlayerId::A, mine).unwrap();
        f.attack = 1;
        f.defense = 1;
        f.max_defense = 1;
        f.flags.summoning_sick = false;
        f.flags.attacks_left = 1;
        f.traits.ignores_ward = Some(true);
    }
    let theirs = put_field(&db, &mut st, PlayerId::B, "10001110");
    {
        let f = st.field_inst_mut(PlayerId::B, theirs).unwrap();
        f.attack = 5;
        f.defense = 5;
        f.max_defense = 5;
        f.traits.ward = Some(true);
        f.flags.summoning_sick = false;
    }
    let a = h0_pick(&db, &st);
    assert!(
        matches!(
            a,
            Action::Attack {
                attacker: Slot(0),
                target: AttackTarget::Leader
            }
        ),
        "expected safe face attack, got {a:?}"
    );
}

struct PlayOut {
    winner: Option<PlayerId>,
    turns: u32,
    actions: u32,
}

fn play_pair(
    db: &arena_engine::CardDb,
    seed: u64,
    decks: &[CardId],
    first: First,
    mut a: H0,
    mut b: impl Policy,
) -> PlayOut {
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: decks.to_vec(),
            deck_b: decks.to_vec(),
            first,
            opening_hands: None,
        },
    )
    .unwrap();
    let mut rng = policy_rng(seed);
    let mut nact = 0u32;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        if state.turn > MAX_TURNS || nact >= MAX_ACTIONS {
            break;
        }
        let legal = legal_actions(db, &state);
        if legal.is_empty() {
            break;
        }
        let who = arena_engine::acting_player(&state);
        let idx = match who {
            PlayerId::A => a.choose(db, &state, &legal, &mut rng),
            PlayerId::B => b.choose(db, &state, &legal, &mut rng),
        }
        .min(legal.len() - 1);
        if apply(db, &mut state, legal[idx].clone()).is_err() {
            break;
        }
        nact += 1;
    }
    PlayOut {
        winner: state.winner,
        turns: state.turn,
        actions: nact,
    }
}

#[test]
fn h0_beats_random_basic_forest_mirror() {
    let db = load_db();
    let decks = load_deck_file("oracle/decks/basic-forest.json");
    assert!(deck_ready(&db, &decks));
    const N: u32 = 200;
    const SEED: u64 = 20260910;
    let mut wins = 0u32;
    let mut losses = 0u32;
    let mut draws = 0u32;
    let mut draw_turns = 0u32;
    let mut draw_acts = 0u32;
    let mut results = Vec::with_capacity(N as usize);
    for i in 0..N {
        let first = if i % 2 == 0 { First::A } else { First::B };
        let out = play_pair(
            &db,
            SEED.wrapping_add(u64::from(i)),
            &decks,
            first,
            H0::fast(),
            Random,
        );
        results.push(out.winner);
        match out.winner {
            Some(PlayerId::A) => wins += 1,
            Some(PlayerId::B) => losses += 1,
            None => {
                draws += 1;
                draw_turns += out.turns;
                draw_acts += out.actions;
            }
        }
    }
    let rate = f64::from(wins) / f64::from(N);
    eprintln!(
        "H0 vs Random basic-forest-mirror: {wins}/{N} ({:.1}%) losses={losses} draws={draws} draw_mean_turns={} draw_mean_acts={}",
        rate * 100.0,
        if draws == 0 { 0 } else { draw_turns / draws },
        if draws == 0 { 0 } else { draw_acts / draws },
    );
    assert!(
        rate >= 0.90,
        "H0 win rate {:.1}% below 90% on basic-forest-mirror",
        rate * 100.0
    );

    let mut again = Vec::with_capacity(N as usize);
    for i in 0..N {
        let first = if i % 2 == 0 { First::A } else { First::B };
        again.push(
            play_pair(
                &db,
                SEED.wrapping_add(u64::from(i)),
                &decks,
                first,
                H0::fast(),
                Random,
            )
            .winner,
        );
    }
    assert_eq!(
        results, again,
        "H0 vs Random must be deterministic for a seed"
    );
}
