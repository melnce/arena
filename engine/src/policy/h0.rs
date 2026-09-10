//! H0: depth-limited beam search with a hand-written value and
//! determinized opponent replies.

use crate::action::{acting_player, Action};
use crate::apply::{apply, legal_actions};
use crate::db::CardDb;
use crate::determinize::determinize;
use crate::ids::PlayerId;
use crate::limits::MAX_TURNS;
use crate::rng::Xoshiro256ss;
use crate::state::{Phase, State};

use super::Policy;

const INF: f32 = 1.0e9;

/// Determinized search bot. Opponent replies use a greedy value maximiser
/// (not a nested H0): four determinizations × a depth-2 H0 would spend the
/// 2 000-`apply` node cap on the opponent and starve own-turn lethal search.
#[derive(Debug, Clone)]
pub struct H0 {
    pub depth: u32,
    pub beam: usize,
    pub determinizations: u32,
    pub node_cap: u32,
}

impl Default for H0 {
    fn default() -> Self {
        Self {
            depth: 6,
            beam: 4,
            determinizations: 4,
            node_cap: 2000,
        }
    }
}

impl H0 {
    /// Shallow lethal-aware search for bulk fixtures.
    pub fn fast() -> Self {
        Self {
            depth: 2,
            beam: 2,
            determinizations: 0,
            node_cap: 80,
        }
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
        if legal.len() <= 1 {
            return 0;
        }
        // Bonus PP is a toggle; picking it forever hits the action cap.
        let cand: Vec<usize> = if legal.iter().any(|a| !matches!(a, Action::BonusPp)) {
            legal
                .iter()
                .enumerate()
                .filter(|(_, a)| !matches!(a, Action::BonusPp))
                .map(|(i, _)| i)
                .collect()
        } else {
            (0..legal.len()).collect()
        };
        if cand.len() <= 1 {
            return cand.first().copied().unwrap_or(0);
        }
        let subset: Vec<Action> = cand.iter().map(|&i| legal[i].clone()).collect();
        let me = acting_player(state);
        let mut nodes = 0u32;

        if matches!(state.phase, Phase::Mulligan { .. }) {
            return 0;
        }

        if self.determinizations == 0 && self.depth <= 2 {
            return cand[greedy_play(db, state, &subset, me, &mut nodes, self.node_cap.min(80))
                .min(cand.len() - 1)];
        }

        if let Some(i) = find_lethal(db, state, &subset, me, 2, &mut nodes, self.node_cap) {
            return cand[i];
        }

        let mut best_i = 0usize;
        let mut best_v = f32::NEG_INFINITY;
        for (j, a) in subset.iter().enumerate() {
            if nodes >= self.node_cap {
                break;
            }
            let mut s = state.clone();
            nodes += 1;
            if apply(db, &mut s, a.clone()).is_err() {
                continue;
            }
            if s.winner == Some(me) {
                return cand[j];
            }
            let v = search_own(
                db,
                &s,
                me,
                self.depth.saturating_sub(1),
                self.beam,
                self.determinizations,
                &mut nodes,
                self.node_cap,
                rng,
            );
            if v > best_v {
                best_v = v;
                best_i = j;
            }
        }
        cand[best_i]
    }
}

fn find_lethal(
    db: &CardDb,
    state: &State,
    legal: &[Action],
    me: PlayerId,
    depth: u32,
    nodes: &mut u32,
    cap: u32,
) -> Option<usize> {
    for (i, a) in legal.iter().enumerate() {
        if *nodes >= cap {
            return None;
        }
        if matches!(a, Action::EndTurn | Action::BonusPp | Action::Confirm) {
            continue;
        }
        let mut s = state.clone();
        *nodes += 1;
        if apply(db, &mut s, a.clone()).is_err() {
            continue;
        }
        if s.winner == Some(me) {
            return Some(i);
        }
        if depth == 0 || acting_player(&s) != me {
            continue;
        }
        let next = legal_actions(db, &s);
        if find_lethal(db, &s, &next, me, depth.saturating_sub(1), nodes, cap).is_some() {
            return Some(i);
        }
    }
    None
}

fn greedy_play(
    db: &CardDb,
    state: &State,
    legal: &[Action],
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
) -> usize {
    let can_spend = legal.iter().any(|a| {
        matches!(
            a,
            Action::Play { .. } | Action::Attack { .. } | Action::Evolve { .. } | Action::Choose(_)
        )
    });
    let mut best_i = 0usize;
    let mut best_v = f32::NEG_INFINITY;
    for (i, a) in legal.iter().enumerate() {
        if *nodes >= cap {
            break;
        }
        let mut s = state.clone();
        *nodes += 1;
        if apply(db, &mut s, a.clone()).is_err() {
            continue;
        }
        if s.winner == Some(me) {
            return i;
        }
        let mut v = value(&s, me);
        if matches!(a, Action::EndTurn) && can_spend {
            v -= 4.0;
        }
        if matches!(a, Action::BonusPp) {
            v -= 20.0;
        }
        if matches!(
            a,
            Action::Attack {
                target: crate::ids::AttackTarget::Leader,
                ..
            }
        ) {
            v += 3.0;
        }
        if v > best_v {
            best_v = v;
            best_i = i;
        }
    }
    best_i
}

fn greedy_index(
    db: &CardDb,
    state: &State,
    legal: &[Action],
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
) -> usize {
    let mut best_i = 0usize;
    let mut best_v = f32::NEG_INFINITY;
    for (i, a) in legal.iter().enumerate() {
        if *nodes >= cap {
            break;
        }
        let mut s = state.clone();
        *nodes += 1;
        if apply(db, &mut s, a.clone()).is_err() {
            continue;
        }
        let v = value(&s, me);
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
    dets: u32,
    nodes: &mut u32,
    cap: u32,
    rng: &mut Xoshiro256ss,
) -> f32 {
    if state.winner == Some(me) {
        return INF;
    }
    if state.winner == Some(me.opponent()) {
        return -INF;
    }
    if *nodes >= cap {
        return value(state, me);
    }
    if acting_player(state) != me || matches!(state.phase, Phase::Terminal) {
        return opponent_replies(db, state, me, dets, nodes, cap, rng);
    }
    if depth == 0 {
        return value(state, me);
    }
    let legal = legal_actions(db, state);
    if legal.is_empty() {
        return value(state, me);
    }

    let mut scored: Vec<(f32, State)> = Vec::with_capacity(legal.len());
    for a in &legal {
        if *nodes >= cap {
            break;
        }
        let mut s = state.clone();
        *nodes += 1;
        if apply(db, &mut s, a.clone()).is_err() {
            continue;
        }
        if s.winner == Some(me) {
            return INF;
        }
        let mut v = value(&s, me);
        if matches!(a, Action::BonusPp) {
            v -= 20.0;
        }
        scored.push((v, s));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(beam.max(1));

    let mut best = f32::NEG_INFINITY;
    for (_, s) in scored {
        let v = search_own(
            db,
            &s,
            me,
            depth.saturating_sub(1),
            beam,
            dets,
            nodes,
            cap,
            rng,
        );
        if v > best {
            best = v;
        }
    }
    best
}

fn opponent_replies(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    dets: u32,
    nodes: &mut u32,
    cap: u32,
    rng: &mut Xoshiro256ss,
) -> f32 {
    if state.winner == Some(me) {
        return INF;
    }
    if state.winner == Some(me.opponent()) {
        return -INF;
    }
    if dets == 0 {
        return value(state, me);
    }
    let n = dets;
    let mut acc = 0.0f32;
    let mut used = 0u32;
    for _ in 0..n {
        if *nodes >= cap {
            break;
        }
        let seed = rng.next_u64();
        let mut s = determinize(state, me, seed);
        greedy_until_end(db, &mut s, me.opponent(), nodes, cap);
        acc += value(&s, me);
        used += 1;
    }
    if used == 0 {
        value(state, me)
    } else {
        acc / used as f32
    }
}

fn greedy_until_end(db: &CardDb, state: &mut State, who: PlayerId, nodes: &mut u32, cap: u32) {
    let mut steps = 0u32;
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
                *nodes += 1;
                let _ = apply(db, state, legal[end].clone());
                break;
            }
        }
        let i = greedy_index(db, state, &legal, who, nodes, cap);
        if apply(db, state, legal[i].clone()).is_err() {
            break;
        }
        *nodes += 1;
        steps += 1;
    }
}

pub fn value(state: &State, me: PlayerId) -> f32 {
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

fn next_pp(p: &crate::state::PlayerState) -> f32 {
    let mut m = p.pp_max;
    if m < 10 {
        m += 1;
    }
    m as f32
}

fn board_score(p: &crate::state::PlayerState) -> f32 {
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
