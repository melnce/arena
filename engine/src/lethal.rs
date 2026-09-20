//! Exhaustive within-turn forced-lethal search.
//!
//! [`forced_lethal`] is a **proof procedure**, not a glance. It walks every
//! legal action except [`Action::EndTurn`] (and mulligan), depth-first, until
//! the turn-holder is shown to have a line that leaves `winner ==
//! Some(perspective)`, every such branch is exhausted, or the node budget
//! runs out. Those three outcomes are different types: [`LethalVerdict::None`]
//! is a proof of absence; [`LethalVerdict::Unknown`] is not. There is no
//! boolean wrapper and no `Default` — collapsing the last two into "no
//! lethal" is the bug this module exists to make unrepresentable.
//!
//! # What "forced" means here
//!
//! Some card effects consume `state.rng`. A line that kills under one roll
//! may miss under another. Strict "forced" — *kills under every outcome* —
//! is a different, much more expensive search, and it is **not** what this
//! metric asks.
//!
//! This search uses the game's own RNG state. A [`LethalVerdict::Lethal`]
//! verdict means *the bot could actually have executed this kill in this
//! game*, which is the question "did it miss a kill it had?" A line that
//! needed a coin flip is still a real miss, but a weaker indictment:
//! [`LethalVerdict::Lethal::rng_dependent`] is set when any action on the
//! winning line advanced the generator, and the offline runner reports
//! those separately.
//!
//! A later reader who assumes "forced" means "under all outcomes" will
//! draw the wrong conclusion from the number. It does not.
//!
//! # What this is not
//!
//! [`crate::policy::H0`]'s `opp_lethal_sweep` is a capped heuristic glance
//! (one `Play` deep, at most one evolve, 40 or 120 applies). It is
//! deliberately cheap and incomplete. This solver must never be called
//! from search — it is far too slow — and it does not reuse that glance.
//!
//! Transposition is allowed only for positions already proven `None`
//! within budget. `Unknown` is never memoised. The table key is
//! [`crate::search_key`] plus [`crate::GameRng::fingerprint`], so two
//! boards that differ only in the next roll do not share a `None`.

use std::collections::HashSet;

use crate::action::{acting_player, Action};
use crate::apply::{apply, legal_actions};
use crate::db::CardDb;
use crate::ids::{AttackTarget, PlayerId};
use crate::rng::GameRng;
use crate::snapshot::hash;
use crate::state::{Phase, State};

/// A real within-turn kill is far shorter than this. Hitting the cap is
/// [`Outcome::Unknown`], not a proof — the turn space may still hide a line.
const MAX_PLY: u32 = 64;

/// Three-way verdict. There is no `Default` and no `is_lethal()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LethalVerdict {
    /// A line was found that ends the turn-holder's turn with the
    /// opponent's leader dead.
    Lethal {
        line: Vec<Action>,
        nodes: u32,
        rng_dependent: bool,
    },
    /// The search completed with every branch exhausted and found none.
    /// This is a proof.
    None { nodes: u32 },
    /// The node budget ran out first. This is not a proof of absence.
    Unknown { nodes: u32 },
}

/// Action kinds the solver expands. The match in [`consider`] is
/// exhaustive: a new [`Action`] variant is a compile error here, so
/// dropping a kind cannot silently turn a proof into a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LethalActionKind {
    Play,
    Attack,
    Evolve,
    Engage,
    Fuse,
    BonusPp,
    Choose,
    Confirm,
}

/// Exhaustive within-turn search from `state`'s acting player.
///
/// `budget` is a cap on `apply` calls. `budget == 0` with any expandable
/// action yields [`LethalVerdict::Unknown`], not [`LethalVerdict::None`].
pub fn forced_lethal(db: &CardDb, state: &State, budget: u32) -> LethalVerdict {
    let perspective = acting_player(state);
    let mut nodes = 0u32;
    let mut tt: HashSet<(u64, u64)> = HashSet::new();
    let mut path_keys: Vec<(u64, u64)> = vec![pos_key(state)];
    let mut line: Vec<Action> = Vec::new();
    match search(
        db,
        state,
        perspective,
        budget,
        &mut nodes,
        &mut tt,
        &mut path_keys,
        &mut line,
        false,
        0,
    ) {
        Outcome::Lethal { rng_dependent } => LethalVerdict::Lethal {
            line,
            nodes,
            rng_dependent,
        },
        Outcome::None => LethalVerdict::None { nodes },
        Outcome::Unknown => LethalVerdict::Unknown { nodes },
    }
}

/// Kind the solver will expand, or `None` for [`Action::EndTurn`] /
/// mulligan.
pub fn lethal_action_kind(a: &Action) -> Option<LethalActionKind> {
    match a {
        Action::Play { .. } => Some(LethalActionKind::Play),
        Action::Attack { .. } => Some(LethalActionKind::Attack),
        Action::Evolve { .. } => Some(LethalActionKind::Evolve),
        Action::Engage { .. } => Some(LethalActionKind::Engage),
        Action::Fuse { .. } => Some(LethalActionKind::Fuse),
        Action::BonusPp => Some(LethalActionKind::BonusPp),
        Action::Choose(_) => Some(LethalActionKind::Choose),
        Action::Confirm => Some(LethalActionKind::Confirm),
        Action::EndTurn | Action::MulliganConfirm { .. } => None,
    }
}

#[derive(Clone, Copy)]
enum Outcome {
    Lethal { rng_dependent: bool },
    None,
    Unknown,
}

fn consider(a: &Action) -> bool {
    lethal_action_kind(a).is_some()
}

fn within_turn(state: &State, perspective: PlayerId) -> bool {
    state.winner.is_none()
        && state.active == perspective
        && !matches!(
            state.phase,
            Phase::Terminal | Phase::Mulligan { .. } | Phase::End
        )
}

fn action_rank(a: &Action) -> u8 {
    match a {
        Action::Attack {
            target: AttackTarget::Leader,
            ..
        } => 0,
        Action::Attack { .. } => 1,
        Action::Evolve { .. } => 2,
        Action::Engage { .. } => 3,
        Action::Play { .. } => 4,
        Action::Fuse { .. } => 5,
        Action::BonusPp => 6,
        Action::Choose(_) => 7,
        Action::Confirm => 8,
        Action::MulliganConfirm { .. } => 9,
        Action::EndTurn => 10,
    }
}

fn pos_key(state: &State) -> (u64, u64) {
    // `hash` is the public snapshot hash: no RNG, no step_counter. Using
    // `search_key` here would treat every apply as a new node (it folds in
    // `step_counter`) and a Bonus-PP toggle would recurse forever.
    (hash(state), state.rng.fingerprint())
}

fn rng_consumed(before: &GameRng, after: &GameRng) -> bool {
    before.fingerprint() != after.fingerprint()
}

#[allow(clippy::too_many_arguments)]
fn search(
    db: &CardDb,
    state: &State,
    perspective: PlayerId,
    budget: u32,
    nodes: &mut u32,
    tt: &mut HashSet<(u64, u64)>,
    path_keys: &mut Vec<(u64, u64)>,
    line: &mut Vec<Action>,
    rng_so_far: bool,
    ply: u32,
) -> Outcome {
    if state.winner == Some(perspective) {
        return Outcome::Lethal {
            rng_dependent: rng_so_far,
        };
    }
    if !within_turn(state, perspective) {
        return Outcome::None;
    }
    if ply >= MAX_PLY {
        return Outcome::Unknown;
    }
    let key = pos_key(state);
    if tt.contains(&key) {
        return Outcome::None;
    }

    let legal = legal_actions(db, state);
    let mut acts: Vec<(u8, usize, Action)> = legal
        .into_iter()
        .enumerate()
        .filter(|(_, a)| consider(a))
        .map(|(i, a)| (action_rank(&a), i, a))
        .collect();
    acts.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    if acts.is_empty() {
        tt.insert(key);
        return Outcome::None;
    }

    let mut saw_unknown = false;
    for (_, _, a) in acts {
        if *nodes >= budget {
            return Outcome::Unknown;
        }
        let mut s = Box::new(state.clone());
        let rng_before = s.rng.clone();
        *nodes += 1;
        if apply(db, &mut s, a.clone()).is_err() {
            continue;
        }
        let child_key = pos_key(&s);
        if path_keys.contains(&child_key) {
            continue;
        }
        let consumed = rng_consumed(&rng_before, &s.rng);
        if s.winner == Some(perspective) {
            line.push(a);
            return Outcome::Lethal {
                rng_dependent: rng_so_far || consumed,
            };
        }
        path_keys.push(child_key);
        line.push(a);
        let out = search(
            db,
            &s,
            perspective,
            budget,
            nodes,
            tt,
            path_keys,
            line,
            rng_so_far || consumed,
            ply + 1,
        );
        match out {
            Outcome::Lethal { rng_dependent } => return Outcome::Lethal { rng_dependent },
            Outcome::Unknown => {
                line.pop();
                path_keys.pop();
                saw_unknown = true;
            }
            Outcome::None => {
                line.pop();
                path_keys.pop();
            }
        }
    }
    if saw_unknown {
        Outcome::Unknown
    } else {
        tt.insert(key);
        Outcome::None
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn consider_is_exhaustive_allowlist() {
        let kinds = [
            Action::Play { hand: 0 },
            Action::Attack {
                attacker: crate::ids::Slot(0),
                target: AttackTarget::Leader,
            },
            Action::Evolve {
                slot: crate::ids::Slot(0),
                super_evolve: false,
            },
            Action::Engage {
                slot: crate::ids::Slot(0),
            },
            Action::Fuse { host: 0 },
            Action::BonusPp,
            Action::Choose(0),
            Action::Confirm,
        ];
        for a in &kinds {
            assert!(consider(a), "{a:?} must be searched");
            assert!(lethal_action_kind(a).is_some(), "{a:?}");
        }
        assert!(!consider(&Action::EndTurn));
        assert!(!consider(&Action::MulliganConfirm { swap: [false; 4] }));
        assert!(lethal_action_kind(&Action::EndTurn).is_none());
    }
}
