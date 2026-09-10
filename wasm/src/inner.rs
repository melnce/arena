//! JSON-neutral game driver. Shared by the wasm-bindgen wrapper and native tests.

use std::collections::BTreeMap;

use arena_engine::action::acting_player;
use arena_engine::{
    apply_neutral, hash, legal_actions_neutral, new_game, policy_rng, snapshot_json, CardDb,
    CardId, First, GameConfig, NeutralAction, State,
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

    pub fn acting(&self) -> String {
        acting_player(&self.state).as_str().to_string()
    }

    pub fn active(&self) -> String {
        self.state.active.as_str().to_string()
    }

    pub fn turn(&self) -> u32 {
        self.state.turn
    }

    pub fn winner(&self) -> Option<String> {
        self.state.winner.map(|p| p.as_str().to_string())
    }

    /// One NeutralAction for the acting player.
    ///
    /// Until `engine::policy` (M5a / RY) is on this tree, `"random"` and
    /// `"first-legal"` pick from `legal()` with `policy_rng(seed)`. When that
    /// module lands, route every name through it so `"h0"` works with no
    /// client change.
    pub fn bot_action(&self, policy: &str, seed: u64) -> Result<String, String> {
        let chosen = pick_bot_action(&self.state, policy, seed)?;
        serde_json::to_string(&chosen).map_err(|e| e.to_string())
    }

    fn legal_len(&self) -> usize {
        legal_actions_neutral(db(), &self.state).len()
    }
}

pub fn bot_policy_names() -> &'static [&'static str] {
    // After RY: read this list from engine::policy so `"h0"` appears here.
    &["random", "first-legal"]
}

pub fn bot_policies_json() -> String {
    serde_json::to_string(bot_policy_names()).expect("botPolicies")
}

fn pick_bot_action(state: &State, policy: &str, seed: u64) -> Result<NeutralAction, String> {
    let acts = legal_actions_neutral(db(), state);
    if acts.is_empty() {
        return Err("no legal actions".into());
    }
    match policy {
        "first-legal" => Ok(acts[0].clone()),
        "random" => {
            let mut rng = policy_rng(seed);
            let i = rng.gen_range(acts.len() as u32) as usize;
            Ok(acts[i].clone())
        }
        other => Err(format!(
            "unknown policy {other}; available {}",
            bot_policies_json()
        )),
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
