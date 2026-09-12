//! Shared drive loop for bench, Python matchup, and fixtures.
//!
//! This module must compile for `wasm32-unknown-unknown`: no `Instant` /
//! `SystemTime`, threads, or `std::fs`.

use crate::action::acting_player;
use crate::apply::{apply, legal_actions};
use crate::db::CardDb;
use crate::ids::PlayerId;
use crate::limits::{MAX_ACTIONS, MAX_TURNS};
use crate::policy::Policy;
use crate::rng::Xoshiro256ss;
use crate::state::{Phase, State};

/// Why a driven game stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    Lethal,
    Deckout,
    TurnCap,
    ActionCap,
    NoLegal,
    Illegal,
}

impl End {
    pub fn as_str(self) -> &'static str {
        match self {
            End::Lethal => "lethal",
            End::Deckout => "deckout",
            End::TurnCap => "turn_cap",
            End::ActionCap => "action_cap",
            End::NoLegal => "no_legal",
            End::Illegal => "illegal",
        }
    }
}

/// Result of [`play_game`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub winner: Option<PlayerId>,
    pub first: PlayerId,
    pub turns: u32,
    pub actions: u32,
    pub end: End,
}

/// Drives `state` to the end with `pol_a` acting for A and `pol_b` for B.
///
/// Same semantics as `arena-bench`: stop when `winner` is set or the phase is
/// `Terminal`; `state.turn > MAX_TURNS` → [`End::TurnCap`]; `actions >=
/// MAX_ACTIONS` → [`End::ActionCap`]; empty `legal_actions` → [`End::NoLegal`];
/// `apply` error → [`End::Illegal`] (never expected — counted so it is visible).
/// The chosen index is clamped with `.min(legal.len() - 1)`. Both seats draw
/// from the caller-supplied rng (as the bench does).
pub fn play_game(
    db: &CardDb,
    state: &mut State,
    pol_a: &mut dyn Policy,
    pol_b: &mut dyn Policy,
    rng: &mut Xoshiro256ss,
) -> Outcome {
    let first = state.first;
    let mut actions = 0u32;
    let mut end = End::NoLegal;
    while state.winner.is_none() && !matches!(state.phase, Phase::Terminal) {
        if state.turn > MAX_TURNS {
            end = End::TurnCap;
            break;
        }
        if actions >= MAX_ACTIONS {
            end = End::ActionCap;
            break;
        }
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            end = End::NoLegal;
            break;
        }
        let idx = match acting_player(state) {
            PlayerId::A => pol_a.choose(db, state, &legal, rng),
            PlayerId::B => pol_b.choose(db, state, &legal, rng),
        };
        let idx = idx.min(legal.len().saturating_sub(1));
        if apply(db, state, legal[idx].clone()).is_err() {
            end = End::Illegal;
            break;
        }
        actions += 1;
    }
    if let Some(w) = state.winner {
        let p = state.player(w.opponent());
        end = if p.leader_defense <= 0 {
            End::Lethal
        } else {
            End::Deckout
        };
    }
    Outcome {
        winner: state.winner,
        first,
        turns: state.turn,
        actions,
        end,
    }
}
