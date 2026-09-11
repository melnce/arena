//! Headless match engine. Card JSON under `cards/` is data; this crate
//! interprets it. Must stay free of `web-sys`, `js-sys`, and `wasm-bindgen`.
#![forbid(unsafe_code)]

pub mod action;
pub mod action_id;
pub mod apply;
pub mod card;
pub mod db;
pub mod determinize;
pub mod encode;
pub mod error;
pub mod event;
pub mod ids;
pub mod info;
pub mod limits;
pub mod oracle;
pub mod policy;
pub mod rng;
pub mod search_key;
pub mod snapshot;
pub mod state;
pub mod support;
pub mod trace;

pub use action::{acting_player, from_neutral, to_neutral, Action};
pub use action_id::{legal_ids, legal_mask, ActionId};
pub use apply::{apply, apply_neutral, legal_actions, new_game, zone_count};
pub use card::{Card, CardId, CardOrCrest};
pub use db::CardDb;
pub use determinize::determinize;
pub use encode::{encode, Observation};
pub use error::{Illegal, LoadError, OraclePickNotLegal, ReplayError, Unsupported};
pub use ids::{AttackTarget, First, PlayerId, Slot};
pub use info::{
    board_info, hand_info, player_info, BoardCardInfo, GateInfo, HandCardInfo, PlayerInfo,
};
pub use limits::{MAX_ACTIONS, MAX_TURNS};
pub use policy::{by_name, names, AnyPolicy, FirstLegal, Policy, Random, H0};
pub use rng::{policy_rng, GameRng, Xoshiro256ss};
pub use search_key::search_key;
pub use snapshot::{hash, snapshot, snapshot_json};
pub use state::{
    CardInstance, ChoiceNode, GameConfig, OpeningHands, Phase, PlayForm, PlayerState, State,
};
pub use support::m1_unsupported_list;
pub use trace::{
    json_eq_first_diff, legal_divergence_parts, neutral_json, picks_from_trace_rng,
    replay_compare_legal, replay_state_diff, sort_json, ActionLine, NeutralAction,
    OpeningHandsJson, Pick, PickChose, PickWhat, TraceHeader,
};

/// Re-export used by bins that only need to list legal actions as NeutralAction.
/// `legal` is a set: identical NeutralActions appear once (hand copies of one
/// id collapse to one `choose {card}`).
pub fn legal_actions_neutral(db: &CardDb, state: &State) -> Vec<NeutralAction> {
    let mut out = Vec::new();
    for a in legal_actions(db, state) {
        let n = to_neutral(state, &a);
        if !out.contains(&n) {
            out.push(n);
        }
    }
    out
}

pub fn reseed(state: &mut State, seed: u64) {
    state.rng.reseed(seed);
}

const _: fn() = || {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_object_safe(_: &dyn Policy) {}
    assert_send::<State>();
    assert_sync::<State>();
    assert_sync::<CardDb>();
    let _: fn(&dyn Policy) = assert_object_safe;
};
