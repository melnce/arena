//! Random-deck soak gate: seeded class+Neutral 40-card decks, every class
//! vs every class, invariants after every action, then trace→replay on 10 decks.
//!
//! `ARENA_SOAK_GAMES` sets N (default 20; CI uses 100). Tokens are excluded
//! and `deck_enabled_num` is respected.

use arena_engine::card::Class;
use arena_engine::{
    apply, from_neutral, hash, legal_actions, new_game, policy_rng, snapshot_json, to_neutral,
    Action, CardDb, CardId, First, GameConfig, GameRng, OpeningHands, Phase, PlayerId,
};

mod common;
use common::*;

const CLASSES: [Class; 8] = [
    Class::Neutral,
    Class::Forestcraft,
    Class::Swordcraft,
    Class::Runecraft,
    Class::Dragoncraft,
    Class::Abysscraft,
    Class::Havencraft,
    Class::Portalcraft,
];

fn class_name(c: Class) -> &'static str {
    match c {
        Class::Neutral => "neutral",
        Class::Forestcraft => "forestcraft",
        Class::Swordcraft => "swordcraft",
        Class::Runecraft => "runecraft",
        Class::Dragoncraft => "dragoncraft",
        Class::Abysscraft => "abysscraft",
        Class::Havencraft => "havencraft",
        Class::Portalcraft => "portalcraft",
    }
}

fn collect_named(v: &serde_json::Value, out: &mut Vec<CardId>) {
    match v {
        serde_json::Value::Object(m) => {
            if let Some(serde_json::Value::String(s)) = m.get("named") {
                if let Some(id) = CardId::parse(s) {
                    out.push(id);
                }
            }
            for x in m.values() {
                collect_named(x, out);
            }
        }
        serde_json::Value::Array(a) => {
            for x in a {
                collect_named(x, out);
            }
        }
        _ => {}
    }
}

fn named_from_file(db: &CardDb, id: CardId) -> Vec<CardId> {
    let Some(path) = db.paths.get(&id.as_str()) else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str(&text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    collect_named(&v, &mut out);
    if let Some(rec) = db.catalog.get(&id.as_str()) {
        for r in &rec.related_card_ids {
            if let Some(cid) = CardId::parse(r) {
                out.push(cid);
            }
        }
    }
    out
}

fn soak_safe(db: &CardDb, id: CardId, seen: &mut std::collections::BTreeSet<CardId>) -> bool {
    if !seen.insert(id) {
        return true;
    }
    if db.require_supported(id).is_err() {
        return false;
    }
    named_from_file(db, id)
        .into_iter()
        .all(|n| soak_safe(db, n, seen))
}

fn authorable(db: &CardDb, class: Class) -> Vec<(CardId, i32)> {
    let mut out = Vec::new();
    for c in db.cards.values() {
        if c.token() {
            continue;
        }
        if c.class() != class && c.class() != Class::Neutral {
            continue;
        }
        let mut seen = std::collections::BTreeSet::new();
        if !soak_safe(db, c.id(), &mut seen) {
            continue;
        }
        let n = db
            .catalog
            .get(&c.id().as_str())
            .map(|r| r.deck_enabled_num)
            .unwrap_or(3)
            .clamp(1, 3);
        out.push((c.id(), n));
    }
    out.sort_by_key(|(id, _)| id.as_str());
    out
}

fn build_deck(pool: &[(CardId, i32)], seed: u64) -> Vec<CardId> {
    let mut rng = policy_rng(seed);
    let mut copies: Vec<(CardId, i32)> = pool.to_vec();
    let mut deck = Vec::new();
    while deck.len() < 40 {
        let live: Vec<usize> = copies
            .iter()
            .enumerate()
            .filter(|(_, (_, n))| *n > 0)
            .map(|(i, _)| i)
            .collect();
        assert!(!live.is_empty(), "pool exhausted before 40 cards");
        let i = rng.gen_range(live.len() as u32) as usize;
        let idx = live[i];
        deck.push(copies[idx].0);
        copies[idx].1 -= 1;
    }
    deck
}

fn assert_invariants(db: &CardDb, state: &arena_engine::State, terminal: bool) {
    for p in PlayerId::ALL {
        let pl = state.player(p);
        assert!(pl.hand.len() <= 9, "hand");
        assert!(pl.field_count() <= 5, "board");
        assert!(pl.pp <= pl.pp_max + 1, "pp");
        assert!(pl.leader_defense <= pl.leader_max, "leader defense");
        assert!(pl.faith >= 0 && pl.earth >= 0 && pl.shadows >= 0 && pl.combo >= 0);
        assert!(pl.ep >= 0 && pl.sep >= 0);
        for c in pl.field.iter().flatten() {
            assert!(c.defense <= c.max_defense, "follower defense");
            assert!(c.countdown.unwrap_or(0) >= 0);
        }
        for cr in &pl.crests {
            assert!(cr.countdown.unwrap_or(0) >= 0);
        }
    }
    if !matches!(state.phase, Phase::Choice { .. }) {
        assert!(state.bindings.is_empty(), "binding outlived its resolution");
    }
    if !terminal {
        let legal = legal_actions(db, state);
        assert!(!legal.is_empty(), "legal_actions empty before terminal");
    }
}

struct SoakOut {
    actions: u32,
    terminal: bool,
}

fn play_game(db: &CardDb, seed: u64, deck_a: &[CardId], deck_b: &[CardId], n_cap: u32) -> SoakOut {
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b: deck_b.to_vec(),
            first: First::A,
            opening_hands: None,
        },
    )
    .expect("new_game");
    let mut policy = policy_rng(seed);
    let mut actions = 0u32;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        assert!(
            actions < n_cap,
            "game did not reach terminal within {n_cap} actions"
        );
        let h = hash(&state);
        let phase_before = format!("{:?}", state.phase);
        let legal = legal_actions(db, &state);
        assert_eq!(hash(&state), h, "legal_actions mutated");
        assert!(!legal.is_empty(), "legal_actions empty before terminal");
        if matches!(state.phase, Phase::Main | Phase::Combat)
            && legal.iter().any(|a| matches!(a, Action::Choose(_)))
        {
            panic!("legal offered Choose in {phase_before}: {legal:?}");
        }
        let idx = policy.gen_range(legal.len() as u32) as usize;
        let act = legal[idx].clone();
        apply(db, &mut state, act.clone()).unwrap_or_else(|e| {
            panic!(
                "seed={seed} applied action was legal: {e:?} act={act:?} before={phase_before} after={:?} legal={legal:?}",
                state.phase
            );
        });
        actions += 1;
        let terminal = state.winner.is_some() || matches!(state.phase, Phase::Terminal);
        assert_invariants(db, &state, terminal);
    }
    assert!(
        state.winner.is_some() || matches!(state.phase, Phase::Terminal),
        "game did not reach terminal"
    );
    SoakOut {
        actions,
        terminal: true,
    }
}

fn emit_and_replay(db: &CardDb, seed: u64, deck_a: &[CardId], deck_b: &[CardId]) {
    let mut live = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b: deck_b.to_vec(),
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
    while live.winner.is_none() && !matches!(live.phase, Phase::Terminal) && i < 2000 {
        let legal = legal_actions(db, &live);
        if legal.is_empty() {
            break;
        }
        let idx = policy.gen_range(legal.len() as u32) as usize;
        let action = legal[idx].clone();
        let neu = to_neutral(&live, &action);
        apply(db, &mut live, action).unwrap();
        recs.push((neu, live.picks.clone(), snapshot_json(&live)));
        i += 1;
    }

    let mut replay = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b: deck_b.to_vec(),
            first: First::A,
            opening_hands: Some(opening),
        },
    )
    .unwrap();
    for (i, (neu, picks, snap)) in recs.iter().enumerate() {
        replay.rng = GameRng::scripted(picks.clone(), seed);
        let act = from_neutral(&replay, neu).expect("from_neutral");
        apply(db, &mut replay, act).unwrap_or_else(|e| panic!("replay seed={seed} i={i}: {e}"));
        let got = snapshot_json(&replay);
        if let Some((path, a, b)) = arena_engine::replay_state_diff(&got, snap) {
            panic!("seed {seed} i={i} {path}: arena={a} trace={b}");
        }
    }
}

#[test]
fn soak_random_class_matrix() {
    let n: u32 = std::env::var("ARENA_SOAK_GAMES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    let db = load_db();
    let mut pools = Vec::new();
    for &class in &CLASSES {
        let pool = authorable(&db, class);
        assert!(
            pool.iter().map(|(_, k)| *k as usize).sum::<usize>() >= 40,
            "{} pool too small for a 40-card deck",
            class_name(class)
        );
        pools.push((class, pool));
    }

    let mut terminals = 0u32;
    let mut actions = 0u64;
    let mut games = 0u32;
    let mut replay_decks: Vec<(Vec<CardId>, Vec<CardId>, u64)> = Vec::new();

    for (i, (ca, pa)) in pools.iter().enumerate() {
        for (j, (cb, pb)) in pools.iter().enumerate() {
            let deck_a = build_deck(pa, 10_000 + (i as u64) * 8 + j as u64);
            let deck_b = build_deck(pb, 20_000 + (i as u64) * 8 + j as u64);
            if replay_decks.len() < 10 {
                replay_decks.push((
                    deck_a.clone(),
                    deck_b.clone(),
                    30_000 + replay_decks.len() as u64,
                ));
            }
            for g in 0..n {
                let seed = 40_000 + (i as u64) * 1_000 + (j as u64) * 100 + u64::from(g);
                let o = play_game(&db, seed, &deck_a, &deck_b, 2000);
                actions += u64::from(o.actions);
                games += 1;
                if o.terminal {
                    terminals += 1;
                }
            }
            let _ = (ca, cb);
        }
    }

    eprintln!(
        "soak_random {games} games ({} classes, N={n}): terminals={terminals} mean_actions={}",
        CLASSES.len(),
        actions / u64::from(games.max(1)),
    );

    for (k, (da, dbk, seed)) in replay_decks.iter().enumerate() {
        emit_and_replay(&db, *seed, da, dbk);
        eprintln!("soak_random replay deck {k} ok");
    }
}
