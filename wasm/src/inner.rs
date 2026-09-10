//! JSON-neutral game driver. Shared by the wasm-bindgen wrapper and native tests.

use std::collections::BTreeMap;

use arena_engine::{
    apply_neutral, hash, legal_actions_neutral, new_game, snapshot_json, CardDb, CardId, First,
    GameConfig, NeutralAction, State,
};

use crate::bundle::card_db;
use crate::ser::{events_json, full_json, phase_str};

#[derive(Clone)]
pub struct GameInner {
    state: State,
}

impl GameInner {
    pub fn new(seed: u64, deck_a: &str, deck_b: &str, first: &str) -> Result<Self, String> {
        let first = parse_first(first)?;
        let deck_a = parse_deck(deck_a)?;
        let deck_b = parse_deck(deck_b)?;
        let state = new_game(
            db(),
            GameConfig {
                seed,
                deck_a,
                deck_b,
                first,
                opening_hands: None,
            },
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { state })
    }

    pub fn legal(&self) -> Result<String, String> {
        let acts = legal_actions_neutral(db(), &self.state);
        serde_json::to_string(&acts).map_err(|e| e.to_string())
    }

    pub fn apply(&mut self, action: &str) -> Result<String, String> {
        let parsed: NeutralAction = serde_json::from_str(action)
            .map_err(|e| format!("parse NeutralAction: {e}; legal_len={}", self.legal_len()))?;
        match apply_neutral(db(), &mut self.state, &parsed) {
            Ok(events) => events_json(&events),
            Err(err) => Err(format!(
                "illegal: {}; action={}; legal_len={}",
                err,
                action.trim(),
                self.legal_len()
            )),
        }
    }

    pub fn snapshot(&self) -> Result<String, String> {
        serde_json::to_string(&snapshot_json(&self.state)).map_err(|e| e.to_string())
    }

    pub fn full(&self) -> Result<String, String> {
        full_json(&self.state)
    }

    pub fn hash(&self) -> String {
        hash(&self.state).to_string()
    }

    pub fn phase(&self) -> String {
        phase_str(&self.state.phase).to_string()
    }

    fn legal_len(&self) -> usize {
        legal_actions_neutral(db(), &self.state).len()
    }
}

fn db() -> &'static CardDb {
    card_db()
}

fn parse_first(s: &str) -> Result<First, String> {
    match s {
        "coin" => Ok(First::Coin),
        "a" => Ok(First::A),
        "b" => Ok(First::B),
        other => Err(format!("first must be coin|a|b, got {other}")),
    }
}

fn parse_deck(text: &str) -> Result<Vec<CardId>, String> {
    let map: BTreeMap<String, u32> =
        serde_json::from_str(text).map_err(|e| format!("deck JSON: {e}"))?;
    let mut ids = Vec::new();
    for (k, n) in map {
        let id = CardId::parse(&k).ok_or_else(|| format!("invalid card id {k}"))?;
        for _ in 0..n {
            ids.push(id);
        }
    }
    Ok(ids)
}
