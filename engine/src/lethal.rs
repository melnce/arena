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
//! deliberately cheap and incomplete. Search may call this solver only
//! through H0's `olsolve` key, with a budget charged to the node cap.
//! It does not reuse the sweep glance.
//!
//! [`forced_lethal_det`] is the same search with RNG-consuming actions
//! skipped: a proof of absence there is only for deterministic lines.
//!
//! [`forced_lethal_accepting`] is roll search with an acceptance callback
//! invoked on each candidate kill line before returning; rejected lines
//! continue searching within the same budget and transposition is disabled.
//!
//! # Two passes and two keys
//!
//! Pass 1 is the historical search: cycle detection and transposition use
//! `(hash(state), rng fingerprint)` where `hash` is the public snapshot hash
//! (choice nodes appear only as `"choice"`). Pass 1 also tracks whether it
//! was **exact**: every cycle cut and every transposition hit would have
//! happened the same way under [`crate::lethal_key`] plus the fingerprint.
//!
//! [`LethalVerdict::Lethal`] and [`LethalVerdict::Unknown`] from pass 1 are
//! returned unchanged (same line, same node count). [`LethalVerdict::None`]
//! from an exact pass 1 is final. When pass 1 returns [`LethalVerdict::None`]
//! but was inexact — it pruned a branch that the in-flight key would have
//! kept — pass 2 reruns from the root keyed on `(lethal_key(state), fingerprint)`
//! with a fresh table and path. One budget covers both passes; the reported
//! node count is the total. A [`LethalVerdict::None`] no longer hides a cut
//! fuse partner or other in-flight choice.
//!
//! Transposition is allowed only for positions already proven `None`
//! within budget. `Unknown` is never memoised. Pass 1 stores the snapshot
//! key with the [`lethal_key`] at proof time; a hit with a different
//! in-flight key makes pass 1 inexact. Pass 2 memoises on the in-flight key.

use std::collections::HashMap;

use crate::action::{acting_player, Action};
use crate::apply::{apply, legal_actions};
use crate::db::CardDb;
use crate::ids::{AttackTarget, PlayerId};
use crate::rng::GameRng;
use crate::search_key::lethal_key_with;
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
    forced_lethal_inner::<fn(&[Action]) -> bool>(db, state, budget, false, None, None)
}

/// Exhaustive within-turn search for a **deterministic** kill only.
///
/// Actions whose `apply` advances `state.rng` are not expanded and cannot
/// complete a kill. Every [`LethalVerdict::Lethal`] has `rng_dependent ==
/// false`. [`LethalVerdict::None`] is a proof that no deterministic line
/// exists within budget — not a proof that no kill exists at all.
pub fn forced_lethal_det(db: &CardDb, state: &State, budget: u32) -> LethalVerdict {
    forced_lethal_inner::<fn(&[Action]) -> bool>(db, state, budget, true, None, None)
}

/// Like [`forced_lethal`], but when a kill is found the search calls
/// `accept(&line)` before returning [`LethalVerdict::Lethal`]. If `accept`
/// returns `false`, that line is treated as a dead end and the search
/// continues within the same budget. Transposition is disabled — acceptance
/// depends on the whole line, not just the position.
pub fn forced_lethal_accepting<F>(
    db: &CardDb,
    state: &State,
    budget: u32,
    mut accept: F,
) -> LethalVerdict
where
    F: FnMut(&[Action]) -> bool,
{
    forced_lethal_inner(db, state, budget, false, Some(&mut accept), None)
}

/// Like [`forced_lethal_accepting`], but returns [`LethalVerdict::Unknown`] promptly
/// when `cancel` is set (checked at each `apply`).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn forced_lethal_accepting_cancel<F>(
    db: &CardDb,
    state: &State,
    budget: u32,
    mut accept: F,
    cancel: &std::sync::atomic::AtomicBool,
) -> LethalVerdict
where
    F: FnMut(&[Action]) -> bool,
{
    forced_lethal_inner(db, state, budget, false, Some(&mut accept), Some(cancel))
}

fn forced_lethal_inner<F>(
    db: &CardDb,
    state: &State,
    budget: u32,
    deterministic_only: bool,
    accept: Option<&mut F>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> LethalVerdict
where
    F: FnMut(&[Action]) -> bool,
{
    if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
        return LethalVerdict::Unknown { nodes: 0 };
    }
    let perspective = acting_player(state);
    let mut nodes = 0u32;
    let use_tt = accept.is_none();
    let mut accept_holder = accept;
    let mut inexact = false;
    let public = hash(state);
    let mut tt: HashMap<(u64, u64), u64> = HashMap::new();
    let mut path_keys: Vec<(u64, u64)> = vec![position_key(state, public, false)];
    let mut path_lethal: Vec<(u64, u64)> =
        vec![(lethal_key_with(state, public), state.rng.fingerprint())];
    let mut line: Vec<Action> = Vec::new();
    match search(
        db,
        state,
        perspective,
        budget,
        &mut nodes,
        &mut tt,
        use_tt,
        false,
        &mut path_keys,
        &mut path_lethal,
        &mut inexact,
        &mut line,
        false,
        0,
        deterministic_only,
        &mut accept_holder,
        cancel,
    ) {
        Outcome::Lethal { rng_dependent } => {
            return LethalVerdict::Lethal {
                line,
                nodes,
                rng_dependent: if deterministic_only {
                    false
                } else {
                    rng_dependent
                },
            };
        }
        Outcome::Unknown => return LethalVerdict::Unknown { nodes },
        Outcome::None if !inexact => return LethalVerdict::None { nodes },
        Outcome::None => {}
    }

    let mut tt2: HashMap<(u64, u64), u64> = HashMap::new();
    let public = hash(state);
    path_keys = vec![position_key(state, public, true)];
    path_lethal.clear();
    line.clear();
    match search(
        db,
        state,
        perspective,
        budget,
        &mut nodes,
        &mut tt2,
        use_tt,
        true,
        &mut path_keys,
        &mut path_lethal,
        &mut inexact,
        &mut line,
        false,
        0,
        deterministic_only,
        &mut accept_holder,
        cancel,
    ) {
        Outcome::Lethal { rng_dependent } => LethalVerdict::Lethal {
            line,
            nodes,
            rng_dependent: if deterministic_only {
                false
            } else {
                rng_dependent
            },
        },
        Outcome::None => LethalVerdict::None { nodes },
        Outcome::Unknown => LethalVerdict::Unknown { nodes },
    }
}

/// Replay `line` on `state`; every action must apply, consume no RNG, and
/// the last position must leave `winner == Some(me)`.
pub fn confirm_det_lethal_line(db: &CardDb, state: &State, me: PlayerId, line: &[Action]) -> bool {
    let mut s = state.clone();
    for a in line {
        let rng_before = s.rng.clone();
        if apply(db, &mut s, a.clone()).is_err() {
            return false;
        }
        if rng_consumed(&rng_before, &s.rng) {
            return false;
        }
    }
    s.winner == Some(me)
}

/// Seed for one rerolled dice replay during roll-confirmed lethal checks.
/// Derived only from `(pos_key, reroll_index)` — not from the policy rng.
pub fn roll_confirm_seed(pos_key: u64, reroll: u32) -> u64 {
    pos_key
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(u64::from(reroll))
}

/// Replay `line` on `state` under each `seeds` entry (reseed before the
/// line). Every action must apply and the last position must leave
/// `winner == Some(me)`.
pub fn confirm_lethal_line_rerolled(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    line: &[Action],
    seeds: &[u64],
) -> bool {
    for &seed in seeds {
        let mut s = state.clone();
        s.reseed(seed);
        for a in line {
            if apply(db, &mut s, a.clone()).is_err() {
                return false;
            }
        }
        if s.winner != Some(me) {
            return false;
        }
    }
    true
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

fn position_key(state: &State, public: u64, use_lethal: bool) -> (u64, u64) {
    let h = if use_lethal {
        lethal_key_with(state, public)
    } else {
        public
    };
    (h, state.rng.fingerprint())
}

fn rng_consumed(before: &GameRng, after: &GameRng) -> bool {
    before.fingerprint() != after.fingerprint()
}

fn lethal_accepted<F>(line: &[Action], accept: &mut Option<&mut F>) -> bool
where
    F: FnMut(&[Action]) -> bool,
{
    match accept.as_deref_mut() {
        Some(f) => f(line),
        None => true,
    }
}

#[allow(clippy::too_many_arguments)]
fn search<F>(
    db: &CardDb,
    state: &State,
    perspective: PlayerId,
    budget: u32,
    nodes: &mut u32,
    tt: &mut HashMap<(u64, u64), u64>,
    use_tt: bool,
    use_lethal_key: bool,
    path_keys: &mut Vec<(u64, u64)>,
    path_lethal: &mut Vec<(u64, u64)>,
    inexact: &mut bool,
    line: &mut Vec<Action>,
    rng_so_far: bool,
    ply: u32,
    deterministic_only: bool,
    accept: &mut Option<&mut F>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Outcome
where
    F: FnMut(&[Action]) -> bool,
{
    if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
        return Outcome::Unknown;
    }
    if state.winner == Some(perspective) {
        if lethal_accepted(line, accept) {
            return Outcome::Lethal {
                rng_dependent: rng_so_far,
            };
        }
        return Outcome::None;
    }
    if !within_turn(state, perspective) {
        return Outcome::None;
    }
    if ply >= MAX_PLY {
        return Outcome::Unknown;
    }
    let public = hash(state);
    let lethal = lethal_key_with(state, public);
    let key = position_key(state, public, use_lethal_key);
    if use_tt {
        if let Some(&stored_lethal) = tt.get(&key) {
            if stored_lethal != lethal && !use_lethal_key {
                *inexact = true;
            }
            return Outcome::None;
        }
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
        if use_tt {
            tt.insert(key, lethal);
        }
        return Outcome::None;
    }

    let mut saw_unknown = false;
    for (_, _, a) in acts {
        if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            return Outcome::Unknown;
        }
        if *nodes >= budget {
            return Outcome::Unknown;
        }
        let mut s = Box::new(state.clone());
        let rng_before = s.rng.clone();
        *nodes += 1;
        if apply(db, &mut s, a.clone()).is_err() {
            continue;
        }
        let consumed = rng_consumed(&rng_before, &s.rng);
        let child_public = hash(&s);
        let child_lethal = lethal_key_with(&s, child_public);
        let child_key = position_key(&s, child_public, use_lethal_key);
        let child_lethal_key = (child_lethal, s.rng.fingerprint());
        if path_keys.contains(&child_key) {
            if !use_lethal_key
                && !path_lethal.contains(&child_lethal_key)
                && !(deterministic_only && consumed)
            {
                *inexact = true;
            }
            continue;
        }
        if deterministic_only && consumed {
            continue;
        }
        if s.winner == Some(perspective) {
            line.push(a);
            if lethal_accepted(line, accept) {
                return Outcome::Lethal {
                    rng_dependent: rng_so_far || consumed,
                };
            }
            line.pop();
            continue;
        }
        path_keys.push(child_key);
        if !use_lethal_key {
            path_lethal.push(child_lethal_key);
        }
        line.push(a);
        let out = search(
            db,
            &s,
            perspective,
            budget,
            nodes,
            tt,
            use_tt,
            use_lethal_key,
            path_keys,
            path_lethal,
            inexact,
            line,
            rng_so_far || consumed,
            ply + 1,
            deterministic_only,
            accept,
            cancel,
        );
        match out {
            Outcome::Lethal { rng_dependent } => return Outcome::Lethal { rng_dependent },
            Outcome::Unknown => {
                line.pop();
                path_keys.pop();
                if !use_lethal_key {
                    path_lethal.pop();
                }
                saw_unknown = true;
            }
            Outcome::None => {
                line.pop();
                path_keys.pop();
                if !use_lethal_key {
                    path_lethal.pop();
                }
            }
        }
    }
    if saw_unknown {
        Outcome::Unknown
    } else {
        if use_tt {
            tt.insert(key, lethal);
        }
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
