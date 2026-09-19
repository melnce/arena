//! Deterministic games/second. Prints one JSON line.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use arena_engine::policy::SearchStats;
use arena_engine::{
    new_game, play_game, policy_rng, AnyPolicy, CardDb, CardId, End, First, GameConfig, PlayerId,
};

fn parse_or_exit(s: &str) -> AnyPolicy {
    match AnyPolicy::parse_spec(s) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut seed: u64 = 1;
    let mut games: u32 = 100;
    let mut deck_a = PathBuf::from("engine/tests/fixtures/decks/basic-neutral-forest.json");
    let mut deck_b = deck_a.clone();
    let mut policy_a = "random".to_string();
    let mut policy_b: Option<String> = None;
    let mut print_stats = false;
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
            "--stats" => {
                print_stats = true;
                i += 1;
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
    let mut stats_a = SearchStats::default();
    let mut stats_b = SearchStats::default();
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
        let mut pol_a = parse_or_exit(&policy_a);
        let mut pol_b = parse_or_exit(&pol_b_name);
        let out = play_game(&db, &mut state, &mut pol_a, &mut pol_b, &mut rng);
        if let AnyPolicy::H0(h) = &pol_a {
            stats_a.add(&h.stats);
        }
        if let AnyPolicy::H0(h) = &pol_b {
            stats_b.add(&h.stats);
        }
        actions += u64::from(out.actions);
        turns += u64::from(out.turns);
        match out.end {
            End::Lethal => lethal += 1,
            End::Deckout => deckout += 1,
            End::TurnCap => turn_cap_n += 1,
            End::ActionCap => action_cap_n += 1,
            End::NoLegal | End::Illegal => {}
        }
        if let Some(w) = out.winner {
            match w {
                PlayerId::A => a_wins += 1,
                PlayerId::B => b_wins += 1,
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
    if print_stats {
        print_search_stats("A", &policy_a, &stats_a);
        print_search_stats("B", &pol_b_name, &stats_b);
    }
}

fn print_search_stats(seat: &str, spec: &str, s: &SearchStats) {
    if s.decisions == 0 {
        println!("search-stats {seat} {spec}: decisions=0");
        return;
    }
    let d = s.decisions as f64;
    println!(
        "search-stats {seat} {spec}: decisions={} nodes/decision={:.2} cap_hit_rate={:.4} candidates/decision={:.2} pairs_skipped/decision={:.2} opp_leaves/decision={:.2} opp_cap_hit_rate={:.4} tt_hits/decision={:.2} tt_stores/decision={:.2} opp_lethal_checks/decision={:.2} opp_lethal_found/decision={:.2} opp_lethal_evo_found/decision={:.2} opp_lethal_nodes/decision={:.2} chose_with_lethal_root/decision={:.2} cands_with_lethal_root/decision={:.2}",
        s.decisions,
        s.nodes as f64 / d,
        s.cap_hits as f64 / d,
        s.candidates as f64 / d,
        s.pairs_skipped as f64 / d,
        s.opp_leaves as f64 / d,
        s.opp_cap_hits as f64 / d,
        s.tt_hits as f64 / d,
        s.tt_stores as f64 / d,
        s.opp_lethal_checks as f64 / d,
        s.opp_lethal_found as f64 / d,
        s.opp_lethal_evo_found as f64 / d,
        s.opp_lethal_nodes as f64 / d,
        s.chose_with_lethal_root as f64 / d,
        s.cands_with_lethal_root as f64 / d,
    );
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
