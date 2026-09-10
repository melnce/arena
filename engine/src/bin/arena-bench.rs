//! Deterministic games/second. Prints one JSON line.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use arena_engine::{
    acting_player, apply, legal_actions, new_game, policy_rng, AnyPolicy, CardDb, CardId, First,
    GameConfig, Phase, PlayerId, Policy, MAX_ACTIONS, MAX_TURNS,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut seed: u64 = 1;
    let mut games: u32 = 100;
    let mut deck_a = PathBuf::from("engine/tests/fixtures/decks/basic-neutral-forest.json");
    let mut deck_b = deck_a.clone();
    let mut policy_a = "random".to_string();
    let mut policy_b: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                seed = args[i + 1].parse().unwrap();
                i += 2;
            }
            "--games" => {
                games = args[i + 1].parse().unwrap();
                i += 2;
            }
            "--deck-a" => {
                deck_a = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--deck-b" => {
                deck_b = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--policy" => {
                policy_a = args[i + 1].clone();
                i += 2;
            }
            "--vs" => {
                policy_b = Some(args[i + 1].clone());
                i += 2;
            }
            _ => i += 1,
        }
    }
    let mut db = CardDb::load(".").expect("db");
    let _ = db.load_extra_dir("engine/tests/fixtures/cards");
    let da = load_deck(&deck_a);
    let dbk = load_deck(&deck_b);
    if da.iter().any(|c| !db.has_card(*c)) || dbk.iter().any(|c| !db.has_card(*c)) {
        eprintln!("skip: missing card files for bench decks");
        println!(
            r#"{{"games":0,"actions":0,"seconds":0.0,"games_per_second":0.0,"actions_per_second":0.0,"mean_turns":0.0,"mean_actions":0.0,"terminal_by":{{"lethal":0,"deckout":0,"turn_cap":0,"action_cap":0}},"skipped":true}}"#
        );
        return;
    }
    let t0 = Instant::now();
    let mut actions = 0u64;
    let mut turns = 0u64;
    let mut lethal = 0u32;
    let mut deckout = 0u32;
    let mut turn_cap_n = 0u32;
    let mut action_cap_n = 0u32;
    let mut a_wins = 0u32;
    let mut b_wins = 0u32;
    let pol_b_name = policy_b.clone().unwrap_or_else(|| policy_a.clone());
    for g in 0..games {
        let s = seed.wrapping_add(g as u64);
        let mut state = new_game(
            &db,
            GameConfig {
                seed: s,
                deck_a: da.clone(),
                deck_b: dbk.clone(),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("game");
        let mut rng = policy_rng(s);
        let mut pol_a = AnyPolicy::parse(&policy_a);
        let mut pol_b = AnyPolicy::parse(&pol_b_name);
        let mut nact = 0u32;
        while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
            if state.turn > MAX_TURNS {
                turn_cap_n += 1;
                break;
            }
            if nact >= MAX_ACTIONS {
                action_cap_n += 1;
                break;
            }
            let legal = legal_actions(&db, &state);
            if legal.is_empty() {
                break;
            }
            let idx = match acting_player(&state) {
                PlayerId::A => pol_a.choose(&db, &state, &legal, &mut rng),
                PlayerId::B => pol_b.choose(&db, &state, &legal, &mut rng),
            };
            let idx = idx.min(legal.len().saturating_sub(1));
            if apply(&db, &mut state, legal[idx].clone()).is_err() {
                break;
            }
            nact += 1;
            actions += 1;
        }
        turns += u64::from(state.turn);
        if let Some(w) = state.winner {
            match w {
                PlayerId::A => a_wins += 1,
                PlayerId::B => b_wins += 1,
            }
            let p = state.player(w.opponent());
            if p.leader_defense <= 0 {
                lethal += 1;
            } else {
                deckout += 1;
            }
        }
    }
    let secs = t0.elapsed().as_secs_f64().max(1e-9);
    let out = serde_json::json!({
        "games": games,
        "actions": actions,
        "seconds": secs,
        "games_per_second": f64::from(games) / secs,
        "actions_per_second": actions as f64 / secs,
        "mean_turns": turns as f64 / f64::from(games.max(1)),
        "mean_actions": actions as f64 / f64::from(games.max(1)),
        "policy": policy_a,
        "vs": pol_b_name,
        "a_wins": a_wins,
        "b_wins": b_wins,
        "terminal_by": {
            "lethal": lethal,
            "deckout": deckout,
            "turn_cap": turn_cap_n,
            "action_cap": action_cap_n,
        }
    });
    println!("{out}");
}

fn load_deck(path: &PathBuf) -> Vec<CardId> {
    let text = fs::read_to_string(path).unwrap_or_else(|_| "{}".into());
    let map: BTreeMap<String, u32> = serde_json::from_str(&text).unwrap_or_default();
    let mut ids = Vec::new();
    for (k, n) in map {
        if let Some(id) = CardId::parse(&k) {
            for _ in 0..n {
                ids.push(id);
            }
        }
    }
    ids
}
