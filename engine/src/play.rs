//! Shared drive loop for bench, Python matchup, and fixtures.
//!
//! This module must compile for `wasm32-unknown-unknown`: no `Instant` /
//! `SystemTime`, threads, or `std::fs`.

use crate::action::{acting_player, Action};
use crate::apply::{apply, legal_actions};
use crate::card::CardId;
use crate::db::CardDb;
use crate::ids::PlayerId;
use crate::limits::{MAX_ACTIONS, MAX_TURNS};
use crate::policy::{choose_path_code, Policy};
use crate::rng::Xoshiro256ss;
use crate::state::{Phase, State};

/// Opening hand and swap mask captured immediately before a mulligan confirm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulliganRecord {
    pub hand: Vec<CardId>,
    pub swap: [bool; 4],
}

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
#[derive(Debug, Clone)]
pub struct Outcome {
    pub winner: Option<PlayerId>,
    pub first: PlayerId,
    pub turns: u32,
    pub actions: u32,
    pub end: End,
    /// Per-player mulligan, indexed by [`PlayerId`] (`A` = 0, `B` = 1).
    pub mulligans: [Option<MulliganRecord>; 2],
    /// Legal index applied at each step (after clamp), in order. Empty unless
    /// [`play_game_record`] was used with `record = true`.
    pub chosen: Vec<u16>,
    /// Per-step [`Policy::last_path`] code (`255` = none). Empty unless recorded.
    pub paths: Vec<u8>,
    /// Per-step [`Policy::last_value`] (`NaN` = none). Empty unless recorded.
    pub values: Vec<f32>,
    /// [`crate::hash`] at game end. `0` unless recorded.
    pub final_hash: u64,
}

impl PartialEq for Outcome {
    fn eq(&self, other: &Self) -> bool {
        self.winner == other.winner
            && self.first == other.first
            && self.turns == other.turns
            && self.actions == other.actions
            && self.end == other.end
            && self.mulligans == other.mulligans
            && self.chosen == other.chosen
            && self.paths == other.paths
            && values_eq(&self.values, &other.values)
            && self.final_hash == other.final_hash
    }
}

impl Eq for Outcome {}

fn values_eq(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
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
    play_game_record(db, state, pol_a, pol_b, rng, false)
}

/// Like [`play_game`], but when `record` is true fills `chosen`, `paths`,
/// `values`, and `final_hash` on the outcome (pre-reserves up to
/// [`MAX_ACTIONS`] steps).
pub fn play_game_record(
    db: &CardDb,
    state: &mut State,
    pol_a: &mut dyn Policy,
    pol_b: &mut dyn Policy,
    rng: &mut Xoshiro256ss,
    record: bool,
) -> Outcome {
    let first = state.first;
    let mut actions = 0u32;
    let mut end = End::NoLegal;
    let mut mulligans: [Option<MulliganRecord>; 2] = [None, None];
    let mut chosen = if record {
        Vec::with_capacity(MAX_ACTIONS as usize)
    } else {
        Vec::new()
    };
    let mut paths = if record {
        Vec::with_capacity(MAX_ACTIONS as usize)
    } else {
        Vec::new()
    };
    let mut values = if record {
        Vec::with_capacity(MAX_ACTIONS as usize)
    } else {
        Vec::new()
    };
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
        let pol: &mut dyn Policy = match acting_player(state) {
            PlayerId::A => pol_a,
            PlayerId::B => pol_b,
        };
        let idx = pol.choose(db, state, &legal, rng);
        let idx = idx.min(legal.len().saturating_sub(1));
        if record {
            chosen.push(idx as u16);
            paths.push(choose_path_code(pol.last_path()));
            values.push(pol.last_value().unwrap_or(f32::NAN));
        }
        if let Action::MulliganConfirm { swap } = &legal[idx] {
            let me = acting_player(state);
            let hand: Vec<CardId> = state
                .player(me)
                .hand
                .iter()
                .take(4)
                .map(|c| c.card)
                .collect();
            mulligans[me.idx()] = Some(MulliganRecord { hand, swap: *swap });
        }
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
    let final_hash = if record { crate::hash(state) } else { 0 };
    Outcome {
        winner: state.winner,
        first,
        turns: state.turn,
        actions,
        end,
        mulligans,
        chosen,
        paths,
        values,
        final_hash,
    }
}
