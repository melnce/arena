//! JSON-neutral game driver. Shared by the wasm-bindgen wrapper and native tests.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use arena_engine::action::acting_player;
use arena_engine::{
    apply_neutral, board_info, by_name, hand_info, hash, legal_actions, legal_actions_neutral,
    names, new_game, policy_rng, snapshot_json, to_neutral, CardDb, CardId, First, GameConfig,
    NeutralAction, PlayerId, Policy, State,
};

use crate::bundle::card_db;
use crate::ser::{events_json, full_json, phase_str};

pub struct GameInner {
    state: State,
    /// Cached by policy name. Recreated empty on `clone` (H0 is stateless today).
    policies: RefCell<HashMap<String, Box<dyn Policy>>>,
}

impl Clone for GameInner {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            policies: RefCell::new(HashMap::new()),
        }
    }
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
        Ok(Self {
            state,
            policies: RefCell::new(HashMap::new()),
        })
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

    /// One NeutralAction for the acting player via `engine::policy::by_name`.
    pub fn bot_action(&self, policy: &str, seed: u64) -> Result<String, String> {
        let chosen = pick_bot_action(self, policy, seed)?;
        serde_json::to_string(&chosen).map_err(|e| e.to_string())
    }

    pub fn hand_info(&self, player: &str) -> Result<String, String> {
        let who = parse_player(player)?;
        serde_json::to_string(&hand_info(db(), &self.state, who)).map_err(|e| e.to_string())
    }

    pub fn board_info(&self, player: &str) -> Result<String, String> {
        let who = parse_player(player)?;
        serde_json::to_string(&board_info(db(), &self.state, who)).map_err(|e| e.to_string())
    }

    fn legal_len(&self) -> usize {
        legal_actions_neutral(db(), &self.state).len()
    }
}

fn parse_player(s: &str) -> Result<PlayerId, String> {
    match s {
        "a" => Ok(PlayerId::A),
        "b" => Ok(PlayerId::B),
        other => Err(format!("player must be a|b, got {other}")),
    }
}

pub fn bot_policy_names() -> &'static [&'static str] {
    names()
}

pub fn bot_policies_json() -> String {
    serde_json::to_string(bot_policy_names()).expect("botPolicies")
}

fn pick_bot_action(game: &GameInner, policy: &str, seed: u64) -> Result<NeutralAction, String> {
    let legal = legal_actions(db(), &game.state);
    if legal.is_empty() {
        return Err("no legal actions".into());
    }
    let mut policies = game.policies.borrow_mut();
    if !policies.contains_key(policy) {
        let boxed = by_name(policy, seed)
            .ok_or_else(|| format!("unknown policy {policy}; available {}", bot_policies_json()))?;
        policies.insert(policy.to_string(), boxed);
    }
    let p = policies.get_mut(policy).expect("policy inserted");
    let mut rng = policy_rng(seed);
    let idx = p.choose(db(), &game.state, &legal, &mut rng);
    if idx >= legal.len() {
        return Err(format!(
            "policy {policy} chose {idx} past legal_len={}",
            legal.len()
        ));
    }
    Ok(to_neutral(&game.state, &legal[idx]))
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
