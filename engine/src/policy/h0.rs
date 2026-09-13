//! H0: depth-limited beam search with a hand-written value.
//!
//! Search starts from `K = max(1, determinizations)` roots produced by
//! `determinize(state, me, seed)` (which reseeds the game RNG). Own-turn
//! search, lethal, and the opponent reply all run on those roots — the
//! true hidden hand and live game RNG are never consulted. A lethal is
//! taken only when every root agrees. The node cap is global.

use std::collections::HashMap;

use crate::action::{acting_player, Action};
use crate::apply::{apply, legal_actions};
use crate::card::CardKind;
use crate::db::CardDb;
use crate::determinize::determinize;
use crate::ids::PlayerId;
use crate::limits::MAX_TURNS;
use crate::rng::Xoshiro256ss;
use crate::search_key::search_key;
use crate::state::{Phase, PlayerState, State};

use super::needs::NeedsTable;
use super::Policy;

/// Per-decision transposition table: (`search_key`, remaining depth, side-to-move-is-me).
type Tt = HashMap<(u64, u8, bool), f32>;

/// Economy-term weights for [`ValueVersion::V1`]. A weight of `0` drops that term.
#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub shadows: f32,
    pub earth: f32,
    pub faith: f32,
    pub rally: f32,
    pub boost: f32,
    pub need: f32,
    pub last_words: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            shadows: 0.12,
            earth: 0.35,
            faith: 0.15,
            rally: 0.05,
            boost: 0.10,
            need: 0.60,
            last_words: 0.80,
        }
    }
}

/// Which leaf value `H0` uses. Default stays [`ValueVersion::V0`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValueVersion {
    #[default]
    V0,
    V1,
}

/// (`NeedsTable`, version, weights) — the only thing search functions evaluate.
#[derive(Clone, Copy)]
struct Evaluator<'a> {
    needs: &'a NeedsTable,
    version: ValueVersion,
    weights: &'a Weights,
}

impl Evaluator<'_> {
    fn value(self, state: &State, me: PlayerId) -> f32 {
        match self.version {
            ValueVersion::V0 => value(state, me),
            ValueVersion::V1 => value_v1(state, me, self.needs, self.weights),
        }
    }
}

const INF: f32 = 1.0e9;
/// Finite stand-in for a terminal when averaging across roots so a lucky
/// lethal does not look like consensus.
const FINITE_WIN: f32 = 80.0;

/// Per-decision search counters, accumulated across [`H0::choose`] calls.
#[derive(Default, Clone, Debug)]
pub struct SearchStats {
    /// `choose` invocations.
    pub decisions: u64,
    /// `apply`s (`try_apply` / one-ply applies).
    pub nodes: u64,
    /// Decisions that exhausted `node_cap`.
    pub cap_hits: u64,
    /// Legal actions kept after the Bonus-PP useful-action filter.
    pub candidates: u64,
    /// Determinized roots built.
    pub roots: u64,
    /// Opponent-model leaf evaluations.
    pub opp_leaves: u64,
    /// Opponent searches truncated by the node cap.
    pub opp_cap_hits: u64,
    /// TT lookups that returned a value (`tt=1` only).
    pub tt_hits: u64,
    /// Values written to the per-decision table (`tt=1` only).
    pub tt_stores: u64,
}

impl SearchStats {
    pub fn add(&mut self, other: &SearchStats) {
        self.accum(other);
    }

    fn accum(&mut self, other: &SearchStats) {
        self.decisions += other.decisions;
        self.nodes += other.nodes;
        self.cap_hits += other.cap_hits;
        self.candidates += other.candidates;
        self.roots += other.roots;
        self.opp_leaves += other.opp_leaves;
        self.opp_cap_hits += other.opp_cap_hits;
        self.tt_hits += other.tt_hits;
        self.tt_stores += other.tt_stores;
    }
}

/// Determinized search bot. With `odepth = 0` (default) the opponent reply is
/// the historical greedy line; `odepth ≥ 1` replaces that with a beam search.
#[derive(Debug, Clone)]
pub struct H0 {
    pub depth: u32,
    pub beam: usize,
    pub determinizations: u32,
    pub node_cap: u32,
    pub value: ValueVersion,
    pub weights: Weights,
    pub odepth: u32,
    pub obeam: usize,
    /// Per-decision transposition table. Off by default (`tt=0`).
    pub tt: bool,
    pub stats: SearchStats,
}

impl Default for H0 {
    fn default() -> Self {
        Self {
            depth: 6,
            beam: 4,
            determinizations: 4,
            node_cap: 2000,
            value: ValueVersion::V0,
            weights: Weights::default(),
            odepth: 0,
            obeam: 3,
            tt: false,
            stats: SearchStats::default(),
        }
    }
}

impl H0 {
    /// Shallow lethal-aware search for bulk fixtures. `K = 1`.
    pub fn fast() -> Self {
        Self {
            depth: 2,
            beam: 2,
            determinizations: 0,
            node_cap: 80,
            ..Self::default()
        }
    }

    fn k(&self) -> u32 {
        self.determinizations.max(1)
    }

    fn evaluator<'a>(&'a self, db: &'a CardDb) -> Evaluator<'a> {
        Evaluator {
            needs: db.needs(),
            version: self.value,
            weights: &self.weights,
        }
    }

    /// Leaf value under this spec (`v0` is the historical arithmetic).
    pub fn evaluate(&self, db: &CardDb, state: &State, me: PlayerId) -> f32 {
        self.evaluator(db).value(state, me)
    }

    /// Run only the opponent model from a state where it is the opponent's
    /// turn. `odepth = 0` is the greedy line's value.
    pub fn opponent_value(&self, db: &CardDb, state: &State, me: PlayerId) -> f32 {
        let mut nodes = 0u32;
        let eval = self.evaluator(db);
        let line = [search_key(state)];
        let mut stats = SearchStats::default();
        opponent_reply(
            db,
            state,
            me,
            &mut nodes,
            self.node_cap,
            &line,
            eval,
            self.odepth,
            self.obeam,
            &mut stats,
            None,
        )
    }

    pub fn reset_stats(&mut self) {
        self.stats = SearchStats::default();
    }
}

fn tt_get(
    tt: Option<&Tt>,
    state: &State,
    depth: u32,
    me_to_move: bool,
    stats: &mut SearchStats,
) -> Option<f32> {
    let v = *tt?.get(&(search_key(state), depth as u8, me_to_move))?;
    stats.tt_hits += 1;
    Some(v)
}

fn tt_put(
    tt: Option<&mut Tt>,
    state: &State,
    depth: u32,
    me_to_move: bool,
    v: f32,
    stats: &mut SearchStats,
) {
    if let Some(tt) = tt {
        tt.insert((search_key(state), depth as u8, me_to_move), v);
        stats.tt_stores += 1;
    }
}

impl Policy for H0 {
    fn choose(
        &mut self,
        db: &CardDb,
        state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize {
        self.stats.decisions += 1;
        let mut table = self.tt.then(HashMap::new);
        if legal.len() <= 1 {
            return 0;
        }
        if matches!(state.phase, Phase::Mulligan { .. }) {
            return mulligan_index(state, legal);
        }

        let me = acting_player(state);
        let cand: Vec<usize> = legal
            .iter()
            .enumerate()
            .filter(|(_, a)| useful_action(state, me, a))
            .map(|(i, _)| i)
            .collect();
        self.stats.candidates += cand.len() as u64;
        if cand.is_empty() {
            return 0;
        }
        if cand.len() == 1 {
            return cand[0];
        }
        let subset: Vec<Action> = cand.iter().map(|&i| legal[i].clone()).collect();
        let mut nodes = 0u32;
        let k = self.k();
        let mut roots = Vec::with_capacity(k as usize);
        for _ in 0..k {
            roots.push(determinize(state, me, rng.next_u64()));
        }
        self.stats.roots += u64::from(k);
        let eval = self.evaluator(db);
        let odepth = self.odepth;
        let obeam = self.obeam;
        let mut dec_stats = SearchStats::default();

        // `fast()` is 1-ply on the determinized root: a depth-2 consensus
        // lethal walk (every legal × every reply, plus `search_key` on each
        // apply) was the 8× regression vs pre-R2 greedy. Immediate wins are
        // still taken; constructed lethals use `H0::default()`.
        let pick = if self.depth <= 2 {
            cand[one_ply(&roots, db, &subset, me, &mut nodes, self.node_cap, eval)]
        } else if let Some(j) =
            consensus_lethal(db, &roots, &subset, me, 2, &mut nodes, self.node_cap)
        {
            cand[j]
        } else {
            let mut acc = vec![0.0f32; subset.len()];
            let mut n = vec![0u32; subset.len()];
            for root in &roots {
                let root_key = search_key(root);
                for (j, a) in subset.iter().enumerate() {
                    if nodes >= self.node_cap {
                        break;
                    }
                    let Some(s) = try_apply(db, root, a, &mut nodes, self.node_cap, &[root_key])
                    else {
                        continue;
                    };
                    let v = if s.winner == Some(me) {
                        FINITE_WIN
                    } else {
                        let line = vec![root_key, search_key(&s)];
                        search_own(
                            db,
                            &s,
                            me,
                            self.depth.saturating_sub(1),
                            self.beam,
                            &mut nodes,
                            self.node_cap,
                            &line,
                            eval,
                            odepth,
                            obeam,
                            &mut dec_stats,
                            table.as_mut(),
                        )
                    };
                    acc[j] += finite(v);
                    n[j] += 1;
                }
            }

            let mut best_i = 0usize;
            let mut best_v = f32::NEG_INFINITY;
            for (j, &c) in n.iter().enumerate() {
                if c == 0 {
                    continue;
                }
                let v = acc[j] / c as f32;
                if v > best_v {
                    best_v = v;
                    best_i = j;
                }
            }
            cand[best_i]
        };
        self.stats.nodes += u64::from(nodes);
        if nodes >= self.node_cap {
            self.stats.cap_hits += 1;
        }
        self.stats.accum(&dec_stats);
        pick
    }
}

fn mulligan_index(state: &State, legal: &[Action]) -> usize {
    let me = acting_player(state);
    let hand = &state.player(me).hand;
    let mut want = [false; 4];
    for (i, slot) in want.iter_mut().enumerate() {
        if let Some(c) = hand.get(i) {
            *slot = c.cost >= 4;
        }
    }
    legal
        .iter()
        .position(|a| matches!(a, Action::MulliganConfirm { swap } if *swap == want))
        .unwrap_or(0)
}

fn useful_action(state: &State, me: PlayerId, a: &Action) -> bool {
    match a {
        // Activate only — never cancel an unspent orb.
        Action::BonusPp => !state.player(me).bonus_pp.active,
        _ => true,
    }
}

fn finite(v: f32) -> f32 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-FINITE_WIN, FINITE_WIN)
    }
}

fn try_apply(
    db: &CardDb,
    state: &State,
    a: &Action,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> Option<State> {
    if *nodes >= cap {
        return None;
    }
    let mut s = state.clone();
    *nodes += 1;
    if apply(db, &mut s, a.clone()).is_err() {
        return None;
    }
    let k = search_key(&s);
    if line.contains(&k) {
        return None;
    }
    Some(s)
}

fn consensus_lethal(
    db: &CardDb,
    roots: &[State],
    legal: &[Action],
    me: PlayerId,
    depth: u32,
    nodes: &mut u32,
    cap: u32,
) -> Option<usize> {
    let mut ok = vec![true; legal.len()];
    let mut any = false;
    for root in roots {
        let line = [search_key(root)];
        for (j, a) in legal.iter().enumerate() {
            if !ok[j] {
                continue;
            }
            if !is_lethal(db, root, a, me, depth, nodes, cap, &line) {
                ok[j] = false;
            } else {
                any = true;
            }
        }
    }
    if !any {
        return None;
    }
    ok.iter().position(|&b| b)
}

#[allow(clippy::too_many_arguments)]
fn is_lethal(
    db: &CardDb,
    state: &State,
    a: &Action,
    me: PlayerId,
    depth: u32,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> bool {
    if matches!(a, Action::EndTurn | Action::Confirm) {
        return false;
    }
    let Some(s) = try_apply(db, state, a, nodes, cap, line) else {
        return false;
    };
    if s.winner == Some(me) {
        return true;
    }
    if depth == 0 || acting_player(&s) != me {
        return false;
    }
    let next = legal_actions(db, &s);
    let mut next_line = line.to_vec();
    next_line.push(search_key(&s));
    next.iter().any(|b| {
        useful_action(&s, me, b)
            && is_lethal(
                db,
                &s,
                b,
                me,
                depth.saturating_sub(1),
                nodes,
                cap,
                &next_line,
            )
    })
}

fn one_ply(
    roots: &[State],
    db: &CardDb,
    subset: &[Action],
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    eval: Evaluator<'_>,
) -> usize {
    let mut acc = vec![0.0f32; subset.len()];
    let mut n = vec![0u32; subset.len()];
    for root in roots {
        for (j, a) in subset.iter().enumerate() {
            if *nodes >= cap {
                break;
            }
            *nodes += 1;
            let mut s = root.clone();
            if apply(db, &mut s, a.clone()).is_err() {
                continue;
            }
            let mut v = if s.winner == Some(me) {
                FINITE_WIN
            } else {
                eval.value(&s, me)
            };
            if matches!(
                a,
                Action::Attack {
                    target: crate::ids::AttackTarget::Leader,
                    ..
                }
            ) {
                v += 3.0;
            }
            acc[j] += finite(v);
            n[j] += 1;
        }
    }
    let mut best_i = 0usize;
    let mut best_v = f32::NEG_INFINITY;
    for (j, &c) in n.iter().enumerate() {
        if c == 0 {
            continue;
        }
        let v = acc[j] / c as f32;
        if v > best_v {
            best_v = v;
            best_i = j;
        }
    }
    best_i
}

#[allow(clippy::too_many_arguments)]
fn greedy_index(
    db: &CardDb,
    state: &State,
    legal: &[Action],
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
) -> usize {
    let mut best_i = 0usize;
    let mut best_v = f32::NEG_INFINITY;
    for (i, a) in legal.iter().enumerate() {
        if *nodes >= cap {
            break;
        }
        if !useful_action(state, me, a) {
            continue;
        }
        let Some(s) = try_apply(db, state, a, nodes, cap, line) else {
            continue;
        };
        let v = eval.value(&s, me);
        if v > best_v {
            best_v = v;
            best_i = i;
        }
    }
    best_i
}

#[allow(clippy::too_many_arguments)]
fn search_own(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    depth: u32,
    beam: usize,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    odepth: u32,
    obeam: usize,
    stats: &mut SearchStats,
    mut tt: Option<&mut Tt>,
) -> f32 {
    if state.winner == Some(me) {
        return INF;
    }
    if state.winner == Some(me.opponent()) {
        return -INF;
    }
    if *nodes >= cap {
        return eval.value(state, me);
    }
    if acting_player(state) != me || matches!(state.phase, Phase::Terminal) {
        if let Some(v) = tt_get(tt.as_deref(), state, depth, false, stats) {
            return v;
        }
        let v = opponent_reply(
            db,
            state,
            me,
            nodes,
            cap,
            line,
            eval,
            odepth,
            obeam,
            stats,
            tt.as_deref_mut(),
        );
        tt_put(tt, state, depth, false, v, stats);
        return v;
    }
    if depth == 0 {
        return eval.value(state, me);
    }
    if let Some(v) = tt_get(tt.as_deref(), state, depth, true, stats) {
        return v;
    }
    let legal = legal_actions(db, state);
    if legal.is_empty() {
        return eval.value(state, me);
    }

    let mut scored: Vec<(f32, State, u64)> = Vec::with_capacity(legal.len());
    for a in &legal {
        if *nodes >= cap {
            break;
        }
        if !useful_action(state, me, a) {
            continue;
        }
        let Some(s) = try_apply(db, state, a, nodes, cap, line) else {
            continue;
        };
        if s.winner == Some(me) {
            tt_put(tt, state, depth, true, INF, stats);
            return INF;
        }
        let k = search_key(&s);
        scored.push((eval.value(&s, me), s, k));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(beam.max(1));

    let mut best = f32::NEG_INFINITY;
    for (_, s, k) in scored {
        let mut next_line = line.to_vec();
        next_line.push(k);
        let v = search_own(
            db,
            &s,
            me,
            depth.saturating_sub(1),
            beam,
            nodes,
            cap,
            &next_line,
            eval,
            odepth,
            obeam,
            stats,
            tt.as_deref_mut(),
        );
        if v > best {
            best = v;
        }
    }
    tt_put(tt, state, depth, true, best, stats);
    best
}

#[allow(clippy::too_many_arguments)]
fn opponent_reply(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    odepth: u32,
    obeam: usize,
    stats: &mut SearchStats,
    tt: Option<&mut Tt>,
) -> f32 {
    if state.winner == Some(me) {
        return INF;
    }
    if state.winner == Some(me.opponent()) {
        return -INF;
    }
    if odepth == 0 {
        let mut s = state.clone();
        greedy_until_end(db, &mut s, me.opponent(), nodes, cap, line, eval);
        if *nodes >= cap {
            stats.opp_cap_hits += 1;
        }
        stats.opp_leaves += 1;
        return eval.value(&s, me);
    }
    let mut truncated = false;
    let v = search_opp(
        db,
        state,
        me,
        odepth,
        obeam,
        nodes,
        cap,
        line,
        eval,
        stats,
        &mut truncated,
        tt,
    );
    if truncated {
        stats.opp_cap_hits += 1;
    }
    v
}

/// Maximising `eval.value(s, opp)` is minimising `eval.value(s, me)`: the
/// leaf value is antisymmetric term by term (`value(s, A) == -value(s, B)`).
#[allow(clippy::too_many_arguments)]
fn search_opp(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    depth: u32,
    beam: usize,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    stats: &mut SearchStats,
    truncated: &mut bool,
    mut tt: Option<&mut Tt>,
) -> f32 {
    let opp = me.opponent();
    if state.winner == Some(me) {
        return INF;
    }
    if state.winner == Some(opp) {
        return -FINITE_WIN;
    }
    if matches!(state.phase, Phase::Terminal) || state.turn > MAX_TURNS {
        stats.opp_leaves += 1;
        return eval.value(state, me);
    }
    if *nodes >= cap {
        *truncated = true;
        stats.opp_leaves += 1;
        return eval.value(state, me);
    }

    let me_to_move = acting_player(state) == me;
    if let Some(v) = tt_get(tt.as_deref(), state, depth, me_to_move, stats) {
        return v;
    }

    let v = search_opp_expand(
        db,
        state,
        me,
        depth,
        beam,
        nodes,
        cap,
        line,
        eval,
        stats,
        truncated,
        tt.as_deref_mut(),
    );
    tt_put(tt, state, depth, me_to_move, v, stats);
    v
}

#[allow(clippy::too_many_arguments)]
fn search_opp_expand(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    depth: u32,
    beam: usize,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    stats: &mut SearchStats,
    truncated: &mut bool,
    mut tt: Option<&mut Tt>,
) -> f32 {
    let opp = me.opponent();

    // Effect handed a choice to `me` while it is still the opponent's turn.
    if acting_player(state) == me && state.active == opp {
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            stats.opp_leaves += 1;
            return eval.value(state, me);
        }
        let i = greedy_index(db, state, &legal, me, nodes, cap, line, eval);
        let Some(s) = try_apply(db, state, &legal[i], nodes, cap, line) else {
            if *nodes >= cap {
                *truncated = true;
            }
            stats.opp_leaves += 1;
            return eval.value(state, me);
        };
        let mut next_line = line.to_vec();
        next_line.push(search_key(&s));
        return search_opp(
            db, &s, me, depth, beam, nodes, cap, &next_line, eval, stats, truncated, tt,
        );
    }

    if acting_player(state) != opp {
        stats.opp_leaves += 1;
        return eval.value(state, me);
    }
    if depth == 0 {
        stats.opp_leaves += 1;
        return eval.value(state, me);
    }

    let legal = legal_actions(db, state);
    if legal.is_empty() {
        stats.opp_leaves += 1;
        return eval.value(state, me);
    }

    let mut scored: Vec<(f32, State, u64)> = Vec::new();
    let mut end_turn: Option<(State, u64)> = None;
    for a in &legal {
        if *nodes >= cap {
            *truncated = true;
            break;
        }
        if !useful_action(state, opp, a) {
            continue;
        }
        let Some(s) = try_apply(db, state, a, nodes, cap, line) else {
            if *nodes >= cap {
                *truncated = true;
            }
            continue;
        };
        if s.winner == Some(opp) {
            return -FINITE_WIN;
        }
        let k = search_key(&s);
        if matches!(a, Action::EndTurn) {
            end_turn = Some((s, k));
        } else {
            scored.push((eval.value(&s, opp), s, k));
        }
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(beam.max(1));
    // EndTurn is always kept so "do nothing more" is a considered line — that
    // is what removes the greedy 3-step cutoff honestly.
    if let Some((s, k)) = end_turn {
        scored.push((0.0, s, k));
    }
    if scored.is_empty() {
        stats.opp_leaves += 1;
        return eval.value(state, me);
    }

    let mut worst = f32::INFINITY;
    for (_, s, k) in scored {
        let mut next_line = line.to_vec();
        next_line.push(k);
        let v = search_opp(
            db,
            &s,
            me,
            depth.saturating_sub(1),
            beam,
            nodes,
            cap,
            &next_line,
            eval,
            stats,
            truncated,
            tt.as_deref_mut(),
        );
        if v == -FINITE_WIN {
            return -FINITE_WIN;
        }
        if v < worst {
            worst = v;
        }
    }
    if worst.is_finite() {
        worst
    } else {
        stats.opp_leaves += 1;
        eval.value(state, me)
    }
}

#[allow(clippy::too_many_arguments)]
fn greedy_until_end(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
) {
    let mut steps = 0u32;
    let mut line = line.to_vec();
    while state.winner.is_none()
        && !matches!(state.phase, Phase::Terminal)
        && acting_player(state) == who
        && *nodes < cap
        && steps < 6
        && state.turn <= MAX_TURNS
    {
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            break;
        }
        if let Some(end) = legal.iter().position(|a| matches!(a, Action::EndTurn)) {
            if steps >= 3 {
                if let Some(next) = try_apply(db, state, &legal[end], nodes, cap, &line) {
                    *state = next;
                }
                break;
            }
        }
        let i = greedy_index(db, state, &legal, who, nodes, cap, &line, eval);
        let Some(next) = try_apply(db, state, &legal[i], nodes, cap, &line) else {
            break;
        };
        line.push(search_key(&next));
        *state = next;
        steps += 1;
    }
}

/// Historical H0 leaf value (`value=v0`). Byte-for-byte the pre-v1 arithmetic.
pub fn value(state: &State, me: PlayerId) -> f32 {
    value_v0(state, me)
}

pub fn value_v0(state: &State, me: PlayerId) -> f32 {
    if state.winner == Some(me) {
        return INF;
    }
    if let Some(w) = state.winner {
        if w != me {
            return -INF;
        }
    }
    let opp = me.opponent();
    let p = state.player(me);
    let o = state.player(opp);
    let mut v = 4.5 * (p.leader_defense - o.leader_defense) as f32;
    v += board_score(p) - board_score(o);
    v += 0.55 * (p.hand.len() as f32 - o.hand.len() as f32);
    v += 0.18 * (next_pp(p) - next_pp(o));
    v += 0.35 * (p.ep - o.ep) as f32;
    v += 0.45 * (p.sep - o.sep) as f32;
    v += 0.25 * (p.crests.len() as f32 - o.crests.len() as f32);
    for c in &p.crests {
        if c.countdown.is_some() {
            v += 0.15;
        }
    }
    for c in &o.crests {
        if c.countdown.is_some() {
            v -= 0.15;
        }
    }
    v
}

fn value_v1(state: &State, me: PlayerId, needs: &NeedsTable, w: &Weights) -> f32 {
    let v = value_v0(state, me);
    if !v.is_finite() {
        return v;
    }
    let p = state.player(me);
    let o = state.player(me.opponent());
    v + w.shadows * (sat(p.shadows, 10) - sat(o.shadows, 10))
        + w.earth * (sat(p.earth, 6) - sat(o.earth, 6))
        + w.faith * (sat(p.faith, 10) - sat(o.faith, 10))
        + w.rally * (sat(p.rally, 15) - sat(o.rally, 15))
        + w.boost * (boost(p, needs) - boost(o, needs))
        + w.need * (live(p, needs) - live(o, needs))
        + w.last_words * (lw(p, needs) - lw(o, needs))
}

fn sat(x: i32, cap: i32) -> f32 {
    x.max(0).min(cap) as f32
}

fn need_frac(have: i32, n: i32) -> f32 {
    if n <= 0 {
        return 1.0;
    }
    let have = have.max(0) as f32;
    let n = n as f32;
    if have >= n {
        1.0
    } else {
        let r = have / n;
        r * r
    }
}

fn live(p: &PlayerState, needs: &NeedsTable) -> f32 {
    let mut sum = 0.0f32;
    for c in &p.hand {
        let Some(n) = needs.get(c.card) else {
            continue;
        };
        if !n.has_threshold() {
            continue;
        }
        let mut acc = 0.0f32;
        let mut k = 0u32;
        for &need in &n.shadows {
            acc += need_frac(p.shadows, need);
            k += 1;
        }
        for &need in &n.earth {
            acc += need_frac(p.earth, need);
            k += 1;
        }
        for &need in &n.faith {
            acc += need_frac(p.faith, need);
            k += 1;
        }
        for &need in &n.rally {
            acc += need_frac(p.rally, need);
            k += 1;
        }
        if k > 0 {
            sum += acc / k as f32;
        }
    }
    sum
}

fn boost(p: &PlayerState, needs: &NeedsTable) -> f32 {
    let mut s = 0.0f32;
    for c in &p.hand {
        if needs.get(c.card).is_some_and(|n| n.spellboost) {
            s += sat(c.spellboost_count, 10);
        }
    }
    s
}

fn lw(p: &PlayerState, needs: &NeedsTable) -> f32 {
    p.field
        .iter()
        .flatten()
        .filter(|c| c.kind == CardKind::Follower && needs.get(c.card).is_some_and(|n| n.last_words))
        .count() as f32
}

fn next_pp(p: &PlayerState) -> f32 {
    let mut m = p.pp_max;
    if m < 10 {
        m += 1;
    }
    m as f32
}

fn board_score(p: &PlayerState) -> f32 {
    let mut s = 0.0f32;
    for c in p.field.iter().flatten() {
        s += (c.attack + c.defense) as f32;
        if c.is_ward() {
            s += 2.2;
        }
        if c.is_storm() {
            s += 1.6;
        }
        if c.evolved {
            s += 1.1;
        }
    }
    s
}
