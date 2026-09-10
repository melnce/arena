//! Headless match engine. Card JSON under `cards/` is data; this crate
//! interprets it. Must stay free of `web-sys`, `js-sys`, and `wasm-bindgen`.
#![forbid(unsafe_code)]

pub mod action;
pub mod apply;
pub mod card;
pub mod db;
pub mod error;
pub mod event;
pub mod ids;
pub mod rng;
pub mod snapshot;
pub mod state;
pub mod support;
pub mod trace;

pub use action::{from_neutral, to_neutral, Action};
pub use apply::{apply, legal_actions, new_game, zone_count};
pub use card::{Card, CardId, CardOrCrest};
pub use db::CardDb;
pub use error::{Illegal, LoadError, OraclePickNotLegal, ReplayError, Unsupported};
pub use ids::{AttackTarget, First, PlayerId, Slot};
pub use rng::{policy_rng, GameRng, Xoshiro256ss};
pub use snapshot::{hash, snapshot, snapshot_json};
pub use state::{
    CardInstance, ChoiceNode, GameConfig, OpeningHands, Phase, PlayForm, PlayerState, State,
};
pub use support::m1_unsupported_list;
pub use trace::{
    json_eq_first_diff, legal_divergence_parts, neutral_json, picks_from_trace_rng, sort_json,
    ActionLine, NeutralAction, OpeningHandsJson, Pick, PickChose, PickWhat, TraceHeader,
};

/// Re-export used by bins that only need to list legal actions as NeutralAction.
pub fn legal_actions_neutral(db: &CardDb, state: &State) -> Vec<NeutralAction> {
    legal_actions(db, state)
        .iter()
        .map(|a| to_neutral(state, a))
        .collect()
}

pub fn reseed(state: &mut State, seed: u64) {
    state.rng.reseed(seed);
}
