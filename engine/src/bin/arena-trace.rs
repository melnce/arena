//! Seeded random-legal self-play → JSONL trace (`docs/trace-format.md`).
//!
//! The policy's rolls use a **separate** xoshiro256** stream (`policy_rng`)
//! so the trace's `rng` array contains only the game's rolls.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use arena_engine::{
    apply, legal_actions, new_game, policy_rng, snapshot_json, to_neutral, Action, CardDb, CardId,
    First, GameConfig, NeutralAction, Phase, PlayerId, TraceHeader,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut seed: u64 = 1;
    let mut games: u32 = 1;
    let mut deck_a = PathBuf::from("engine/tests/fixtures/decks/basic-neutral-forest.json");
    let mut deck_b = deck_a.clone();
    let mut out = PathBuf::from("traces");
    let mut first = First::Coin;
    let mut turn_cap: u32 = 60;
    let mut action_cap: u32 = 800;
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
            "--out" => {
                out = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--first" => {
                first = match args[i + 1].as_str() {
                    "a" => First::A,
                    "b" => First::B,
                    _ => First::Coin,
                };
                i += 2;
            }
            "--turn-cap" => {
                turn_cap = args[i + 1].parse().unwrap();
                i += 2;
            }
            "--action-cap" => {
                action_cap = args[i + 1].parse().unwrap();
                i += 2;
            }
            _ => i += 1,
        }
    }
    let mut db = CardDb::load(".").expect("CardDb");
    let _ = db.load_extra_dir("engine/tests/fixtures/cards");
    fs::create_dir_all(&out).ok();
    let da = load_deck(&deck_a);
    let dbk = load_deck(&deck_b);
    for g in 0..games {
        let s = seed.wrapping_add(g as u64);
        let path = out.join(format!("{s}.jsonl"));
        if let Err(e) = play_one(&db, s, &da, &dbk, first, turn_cap, action_cap, &path) {
            eprintln!("game {s}: {e}");
        }
    }
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

#[allow(clippy::too_many_arguments)]
fn play_one(
    db: &CardDb,
    seed: u64,
    deck_a: &[CardId],
    deck_b: &[CardId],
    first: First,
    turn_cap: u32,
    action_cap: u32,
    path: &PathBuf,
) -> Result<(), String> {
    let mut state = new_game(
        db,
        GameConfig {
            seed,
            deck_a: deck_a.to_vec(),
            deck_b: deck_b.to_vec(),
            first,
        },
    )
    .map_err(|e| e.to_string())?;
    let header = TraceHeader {
        v: 1,
        engine: "arena".into(),
        seed,
        first: state.first.as_str().into(),
        deck_a: {
            let mut v: Vec<String> = deck_a.iter().map(|c| c.as_str()).collect();
            v.sort();
            v
        },
        deck_b: {
            let mut v: Vec<String> = deck_b.iter().map(|c| c.as_str()).collect();
            v.sort();
            v
        },
    };
    let mut lines = vec![serde_json::to_string(&header).unwrap()];
    let mut policy = policy_rng(seed);
    let mut i = 0u32;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        if state.turn > turn_cap || i >= action_cap {
            break;
        }
        let legal = legal_actions(db, &state);
        if legal.is_empty() {
            break;
        }
        let choice = policy.gen_range(legal.len() as u32) as usize;
        let action = legal[choice].clone();
        let recorded = to_neutral(&state, &action);
        let _ = apply(db, &mut state, action).map_err(|e| e.to_string())?;
        let line = serde_json::json!({
            "i": i,
            "action": recorded,
            "rng": state.picks,
            "state": snapshot_json(&state),
            "legal": legal_actions(db, &state).iter().map(|a| to_neutral(&state, a)).collect::<Vec<NeutralAction>>(),
        });
        lines.push(serde_json::to_string(&line).unwrap());
        i += 1;
        let _ = PlayerId::A;
        let _ = Action::EndTurn;
    }
    fs::write(path, lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(())
}
