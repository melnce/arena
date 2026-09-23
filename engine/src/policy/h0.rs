//! H0: depth-limited beam search. The default leaf is the built-in
//! `h0-linear-v1` value net; `value=v0` is the historical hand-written
//! arithmetic. [`H0::fast`] keeps `value=v0` and `tt=0`.
//!
//! Search starts from `K = max(1, determinizations)` roots produced by
//! `determinize_with(state, me, seed, info)` (which reseeds the game RNG).
//! Under [`Info::All`] every root would be identical, so K is 1 regardless
//! of `determinizations`. Own-turn search, lethal, and the opponent reply
//! all run on those roots — the true hidden hand and live game RNG are
//! never consulted (except under `info=all`, where the opponent hand *is*
//! the true hand). A lethal is taken only when every root agrees. The
//! node cap is global. `encode` still masks the opponent's hand at the
//! leaf; `info` is a search-time knob only.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::action::{acting_player, Action};
use crate::apply::{apply, legal_actions};
use crate::card::{CardId, CardKind};
use crate::db::CardDb;
use crate::determinize::determinize_with;

pub use crate::determinize::Info;
use crate::encode::{encode_with_vocab, vocab};
use crate::ids::{AttackTarget, PlayerId};
use crate::limits::MAX_TURNS;
use crate::rng::Xoshiro256ss;
use crate::search_key::search_key;
use crate::state::{Phase, PlayerState, State};

use super::explain::{CandidateRecord, ChoosePath, ExplainRecord, Line, PvEnd, PvTracker};
use super::needs::NeedsTable;
use super::net::ValueNet;
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

/// Which leaf value `H0` uses. Default is [`ValueVersion::Net`] (the
/// built-in `h0-linear-v1` model). [`ValueVersion::V0`] is the
/// hand-written leaf the bot used before the net became the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValueVersion {
    V0,
    V1,
    /// Learned leaf. Search, determinization, and the opponent model
    /// are unchanged; only [`Evaluator::value`] dispatches. Without
    /// `net=<path>` this is [`builtin_net`].
    #[default]
    Net,
}

const BUILTIN_NET_JSON: &str = include_str!("../../models/h0-linear-v1.json");
/// Committed name of the built-in value model (`engine/models/h0-linear-v1.json`).
pub const BUILTIN_NET_NAME: &str = "h0-linear-v1";
static BUILTIN_NET: OnceLock<Arc<ValueNet>> = OnceLock::new();

/// The built-in `h0-linear-v1` model, parsed once. A parse failure is a
/// build defect.
pub fn builtin_net() -> Arc<ValueNet> {
    BUILTIN_NET
        .get_or_init(|| {
            ValueNet::from_json_named(BUILTIN_NET_NAME, BUILTIN_NET_JSON)
                .expect("built-in value model parses")
        })
        .clone()
}

/// Leaf evaluator plus the root-level terminal stand-in (`wv`) and the
/// cheap opponent-model knobs (`olethal`, `oevo`, `osteps`). `wv` is carried
/// here (not as a sibling of `odepth`/`obeam`) so `finite`, `one_ply`, and
/// the opponent lethal short-circuit share one value.
#[derive(Clone, Copy)]
struct Evaluator<'a> {
    needs: &'a NeedsTable,
    version: ValueVersion,
    weights: &'a Weights,
    wv: f32,
    olethal: bool,
    oevo: bool,
    osteps: u32,
    net: Option<&'a ValueNet>,
    vocab: &'a [CardId],
    clip: f32,
}

impl Evaluator<'_> {
    fn value(self, state: &State, me: PlayerId) -> f32 {
        match self.version {
            ValueVersion::V0 => value(state, me),
            ValueVersion::V1 => value_v1(state, me, self.needs, self.weights),
            ValueVersion::Net => {
                let obs = encode_with_vocab(state, me, self.vocab);
                let net = self.net.expect("value=net requires a loaded net");
                if self.clip > 0.0 {
                    net.value_clipped(&obs, self.clip)
                } else {
                    net.value(&obs)
                }
            }
        }
    }
}

const INF: f32 = 1.0e9;
/// Default saturation bound (`wv`) on every accumulated value. 80 is
/// today's clamp; it sits inside the reachable live range of [`value_v0`].
const DEFAULT_WV: f32 = 80.0;

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
    /// Lethal sweeps run (`olethal=1`, `odepth=0`).
    pub opp_lethal_checks: u64,
    /// Sweeps that found a glance-level opponent lethal.
    pub opp_lethal_found: u64,
    /// Sweeps that found lethal only through an evolve branch (`oevo=1`):
    /// attack-only and play-then-attack both missed, then play-then-evolve
    /// or standalone-evolve succeeded.
    pub opp_lethal_evo_found: u64,
    /// `apply`s spent inside lethal sweeps.
    pub opp_lethal_nodes: u64,
    /// Searched decisions whose chosen candidate had a determinization
    /// pinned at the clamp floor (`-wv`).
    pub chose_with_lethal_root: u64,
    /// Same test, summed over every candidate offered at that decision.
    pub cands_with_lethal_root: u64,
    /// TT lookups that returned a value (`tt=1` only).
    pub tt_hits: u64,
    /// Values written to the per-decision table (`tt=1` only).
    pub tt_stores: u64,
    /// `(root, candidate)` pairs never given a search after
    /// `consensus_lethal` left leftover budget:
    /// `k × |subset| − attempted` per searched decision.
    pub pairs_skipped: u64,
    /// `apply`s spent in the consensus-lethal check before search.
    pub lethal_nodes: u64,
    /// Search-path decisions where no candidate was scored.
    pub unscored: u64,
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
        self.opp_lethal_checks += other.opp_lethal_checks;
        self.opp_lethal_found += other.opp_lethal_found;
        self.opp_lethal_evo_found += other.opp_lethal_evo_found;
        self.opp_lethal_nodes += other.opp_lethal_nodes;
        self.chose_with_lethal_root += other.chose_with_lethal_root;
        self.cands_with_lethal_root += other.cands_with_lethal_root;
        self.tt_hits += other.tt_hits;
        self.tt_stores += other.tt_stores;
        self.pairs_skipped += other.pairs_skipped;
        self.lethal_nodes += other.lethal_nodes;
        self.unscored += other.unscored;
    }
}

/// How [`H0::choose`] spends `node_cap` across `(root, candidate)` pairs.
/// [`Alloc::Root`] is today's root-major spend (later pairs skipped when
/// the cap binds). [`Alloc::Fair`] gives each remaining pair a share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alloc {
    /// Budget spent root-major: later candidates / roots are skipped
    /// when `node_cap` binds. Today's behaviour.
    #[default]
    Root,
    /// Per-pair share of the remaining budget so every candidate is
    /// scored on every determinization (depth is what the share affords).
    Fair,
}

/// Determinized search bot. With `odepth = 0` (default) the opponent reply is
/// the historical greedy line; `odepth ≥ 1` replaces that with a beam search.
#[derive(Debug, Clone)]
pub struct H0 {
    pub depth: u32,
    pub beam: usize,
    pub determinizations: u32,
    pub node_cap: u32,
    /// How the node cap is split across `(root, candidate)` pairs.
    /// Default [`Alloc::Fair`] is today's per-pair budget share (`c42163b`).
    pub alloc: Alloc,
    /// What the search is allowed to know. Default [`Info::Fair`] is the
    /// sweep-8b flip: own deck order is resampled (hand untouched),
    /// opponent is resampled — the same information a human with open
    /// decklists has. `info=draws` restores the pre-flip path.
    pub info: Info,
    pub value: ValueVersion,
    pub weights: Weights,
    pub odepth: u32,
    pub obeam: usize,
    /// Saturation bound on every accumulated value (`wv`). Default `80`
    /// is today's clamp — the same floor a detected opponent lethal
    /// returns (`-wv`).
    pub wv: f32,
    /// Pessimism weight on the root aggregation, in `[0, 1]`. `0` is
    /// today's mean over determinizations; `1` is the worst sample.
    /// Default `0` — the blend is not taken on that path.
    pub pess: f32,
    /// Per-decision transposition table. On by default (`tt=1`).
    pub tt: bool,
    /// Bounded opponent-lethal sweep before the greedy line (`odepth=0` only).
    /// Default `true` is the sweep-5 flip; `olethal=0` restores the pre-flip greedy path.
    pub olethal: bool,
    /// Extend that sweep with at most one `Evolve` / `super_evolve` per
    /// leaf (`olethal=1`, `odepth=0` only). Default `true` is the sweep-8b
    /// flip; `oevo=0` restores the pre-flip glance path.
    pub oevo: bool,
    /// Greedy-line steps before a forced `EndTurn`. Default `6` is the sweep-5
    /// flip; the hard stop is `osteps + 3` (today: 9).
    pub osteps: u32,
    /// Learned leaf (`value=net`). `None` when the leaf is v0/v1.
    pub net: Option<Arc<ValueNet>>,
    /// Consensus-lethal node budget as a fraction of `node_cap`, in `(0, 1]`.
    /// Default `1.0` is today's behaviour (the whole cap).
    pub lcap: f32,
    /// Standardised-input clamp for the learned leaf; `0` is off. Ignored
    /// with `value=v0` / `value=v1`.
    pub clip: f32,
    /// Spec path compared by `h0_fields_eq` and printed by `spec()`.
    pub net_path: Option<String>,
    pub stats: SearchStats,
    /// Root value of the most recent [`Policy::choose`]. Reset every
    /// `choose`; ignored by `h0_fields_eq` and `spec()`.
    pub last_value: Option<f32>,
    /// When set before [`Policy::choose`], the next decision is recorded
    /// into [`explain`]. Ignored by `h0_fields_eq` and `spec()`.
    pub explain_armed: bool,
    /// Per-decision explain record. Populated when [`explain_armed`] was
    /// set; take with [`take_explain`].
    pub explain: Option<ExplainRecord>,
}

impl Default for H0 {
    fn default() -> Self {
        Self {
            depth: 6,
            beam: 4,
            determinizations: 4,
            node_cap: 2000,
            alloc: Alloc::Fair,
            info: Info::Fair,
            value: ValueVersion::Net,
            weights: Weights::default(),
            odepth: 0,
            obeam: 3,
            wv: DEFAULT_WV,
            pess: 0.0,
            tt: true,
            olethal: true,
            oevo: true,
            osteps: 6,
            net: Some(builtin_net()),
            lcap: 1.0,
            clip: 0.0,
            net_path: None,
            stats: SearchStats::default(),
            last_value: None,
            explain_armed: false,
            explain: None,
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
            tt: false,
            value: ValueVersion::V0,
            net: None,
            // Fixed test baseline: olethal/osteps are reachable at depth 2
            // (unlike alloc); inheriting would spend the 80-node cap on the sweep.
            olethal: false,
            oevo: false,
            osteps: 3,
            pess: 0.0,
            info: Info::Draws,
            ..Self::default()
        }
    }

    fn k(&self) -> u32 {
        self.determinizations.max(1)
    }

    fn evaluator<'a>(&'a self, db: &'a CardDb, root_vocab: &'a [CardId]) -> Evaluator<'a> {
        Evaluator {
            needs: db.needs(),
            version: self.value,
            weights: &self.weights,
            wv: self.wv,
            olethal: self.olethal,
            oevo: self.oevo,
            osteps: self.osteps,
            net: self.net.as_deref(),
            vocab: root_vocab,
            clip: self.clip,
        }
    }

    fn lethal_cap(&self, nodes: u32) -> u32 {
        let budget = (self.lcap * self.node_cap as f32).floor() as u32;
        nodes.saturating_add(budget).min(self.node_cap)
    }

    fn root_vocab(&self, state: &State) -> Vec<CardId> {
        if self.value == ValueVersion::Net {
            vocab(state)
        } else {
            Vec::new()
        }
    }

    /// Arm the explain sink for the next [`Policy::choose`]. When not armed,
    /// search behaviour and node accounting are unchanged.
    pub fn arm_explain(&mut self) {
        self.explain_armed = true;
    }

    /// Take the explain record from the most recent armed `choose`.
    pub fn take_explain(&mut self) -> Option<ExplainRecord> {
        self.explain_armed = false;
        self.explain.take()
    }

    fn alloc_label(&self) -> &'static str {
        match self.alloc {
            Alloc::Root => "root",
            Alloc::Fair => "fair",
        }
    }

    /// Leaf value under this spec (`v0` is the historical arithmetic).
    pub fn evaluate(&self, db: &CardDb, state: &State, me: PlayerId) -> f32 {
        let root_vocab = self.root_vocab(state);
        self.evaluator(db, &root_vocab).value(state, me)
    }

    /// Run only the opponent model from a state where it is the opponent's
    /// turn. `odepth = 0` is the greedy line's value. Records search
    /// counters into `self.stats` (including `opp_lethal_*` when the sweep runs).
    pub fn opponent_value(&mut self, db: &CardDb, state: &State, me: PlayerId) -> f32 {
        let mut nodes = 0u32;
        let root_vocab = self.root_vocab(state);
        let eval = self.evaluator(db, &root_vocab);
        let line = [search_key(state)];
        let mut stats = SearchStats::default();
        let v = opponent_reply(
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
            None,
        );
        self.stats.nodes += u64::from(nodes);
        self.stats.accum(&stats);
        v
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
        self.last_value = None;
        self.stats.decisions += 1;
        let recording = self.explain_armed;
        if recording {
            self.explain = None;
        }
        let mut table = self.tt.then(HashMap::new);
        if legal.len() <= 1 {
            if recording {
                let mut rec = ExplainRecord::new(
                    ChoosePath::SingleLegal,
                    0,
                    self.node_cap,
                    self.alloc_label(),
                );
                rec.chosen_index = 0;
                rec.tie_set = vec![0];
                self.explain = Some(rec);
            }
            return 0;
        }
        if matches!(state.phase, Phase::Mulligan { .. }) {
            let idx = mulligan_index(state, legal);
            if recording {
                let mut rec =
                    ExplainRecord::new(ChoosePath::Mulligan, 0, self.node_cap, self.alloc_label());
                rec.chosen_index = idx;
                rec.tie_set = vec![idx];
                self.explain = Some(rec);
            }
            return idx;
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
            if recording {
                let mut rec = ExplainRecord::new(
                    ChoosePath::SingleLegal,
                    0,
                    self.node_cap,
                    self.alloc_label(),
                );
                rec.chosen_index = 0;
                rec.tie_set = vec![0];
                self.explain = Some(rec);
            }
            return 0;
        }
        if cand.len() == 1 {
            if recording {
                let mut rec = ExplainRecord::new(
                    ChoosePath::SingleLegal,
                    0,
                    self.node_cap,
                    self.alloc_label(),
                );
                rec.chosen_index = cand[0];
                rec.tie_set = vec![cand[0]];
                self.explain = Some(rec);
            }
            return cand[0];
        }
        let subset: Vec<Action> = cand.iter().map(|&i| legal[i].clone()).collect();
        let mut nodes = 0u32;
        // `info=all` makes every determinization identical; do not spend
        // `k` copies of the same tree.
        let k = if self.info == Info::All { 1 } else { self.k() };
        let mut roots = Vec::with_capacity(k as usize);
        for _ in 0..k {
            roots.push(determinize_with(state, me, rng.next_u64(), self.info));
        }
        self.stats.roots += u64::from(k);
        let root_vocab = self.root_vocab(state);
        let eval = self.evaluator(db, &root_vocab);
        let odepth = self.odepth;
        let obeam = self.obeam;
        let mut dec_stats = SearchStats::default();
        let mut explain_rec = if recording {
            Some(ExplainRecord::new(
                ChoosePath::Search,
                k,
                self.node_cap,
                self.alloc_label(),
            ))
        } else {
            None
        };
        let mut explain_cands: Vec<CandidateRecord> = if recording {
            subset
                .iter()
                .enumerate()
                .map(|(j, a)| CandidateRecord {
                    legal_index: cand[j],
                    action: crate::action::to_neutral(state, a),
                    worlds: Vec::new(),
                    root_agg: 0.0,
                    worst: f32::INFINITY,
                    n: 0,
                })
                .collect()
        } else {
            Vec::new()
        };

        // `fast()` is 1-ply on the determinized root: a depth-2 consensus
        // lethal walk (every legal × every reply, plus `search_key` on each
        // apply) was the 8× regression vs pre-R2 greedy. Immediate wins are
        // still taken; constructed lethals use `H0::default()`.
        let mut lethal_spent = None;
        let pick = if self.depth <= 2 {
            if let Some(rec) = &mut explain_rec {
                rec.path = ChoosePath::OnePly;
            }
            let mut tie_set = Vec::new();
            let (j, v) = one_ply(
                &roots,
                db,
                &subset,
                me,
                &mut nodes,
                self.node_cap,
                eval,
                self.pess,
                if recording {
                    Some((&cand, &mut tie_set))
                } else {
                    None
                },
            );
            self.last_value = Some(v);
            if let Some(rec) = &mut explain_rec {
                rec.chosen_index = cand[j];
                rec.tie_set = tie_set;
            }
            cand[j]
        } else {
            let nodes_before_lethal = nodes;
            let lethal_cap = self.lethal_cap(nodes);
            let lethal = consensus_lethal(db, &roots, &subset, me, 2, &mut nodes, lethal_cap);
            lethal_spent = Some(nodes - nodes_before_lethal);
            if let Some(j) = lethal {
                if let Some(rec) = &mut explain_rec {
                    rec.path = ChoosePath::ConsensusLethal;
                    rec.nodes_lethal = nodes - nodes_before_lethal;
                    rec.chosen_index = cand[j];
                    rec.tie_set = vec![cand[j]];
                }
                self.last_value = Some(self.wv);
                cand[j]
            } else {
                if let Some(rec) = &mut explain_rec {
                    rec.path = ChoosePath::Search;
                    rec.nodes_lethal = nodes - nodes_before_lethal;
                }
                let mut acc = vec![0.0f32; subset.len()];
                let mut n = vec![0u32; subset.len()];
                let mut worst = vec![f32::INFINITY; subset.len()];
                let total_pairs = u64::from(k) * subset.len() as u64;
                let mut attempted = 0u64;
                let nodes_after_lethal = nodes;
                let mut cap_exhausted = false;
                for (r, root) in roots.iter().enumerate() {
                    let root_key = search_key(root);
                    for (j, a) in subset.iter().enumerate() {
                        if nodes >= self.node_cap {
                            if recording {
                                for explain_cand in explain_cands.iter_mut().skip(j) {
                                    explain_cand
                                        .worlds
                                        .push(PvTracker::skipped_world(r as u32, 0));
                                }
                                for rr in (r + 1)..roots.len() {
                                    for explain_cand in explain_cands.iter_mut() {
                                        explain_cand
                                            .worlds
                                            .push(PvTracker::skipped_world(rr as u32, 0));
                                    }
                                }
                            }
                            cap_exhausted = true;
                            break;
                        }
                        attempted += 1;
                        let cap = match self.alloc {
                            Alloc::Root => self.node_cap,
                            Alloc::Fair => {
                                const MIN_SHARE: u32 = 24;
                                let pairs_left = (k as usize - r) * subset.len() - j;
                                let remaining = self.node_cap - nodes;
                                let even = remaining / pairs_left as u32;
                                let share =
                                    if remaining >= MIN_SHARE.saturating_mul(pairs_left as u32) {
                                        even.max(MIN_SHARE)
                                    } else {
                                        even.max(1)
                                    };
                                nodes.saturating_add(share).min(self.node_cap)
                            }
                        };
                        let pair_nodes_start = nodes;
                        let mut tracker = if recording {
                            Some(PvTracker::new(cap))
                        } else {
                            None
                        };
                        let Some(s) = try_apply(db, root, a, &mut nodes, cap, &[root_key]) else {
                            if recording {
                                explain_cands[j]
                                    .worlds
                                    .push(PvTracker::skipped_world(r as u32, cap));
                            }
                            continue;
                        };
                        if let Some(t) = tracker.as_mut() {
                            t.push(a.clone());
                        }
                        let v = if s.winner == Some(me) {
                            if let Some(t) = tracker.as_mut() {
                                t.set_leaf(eval.wv, PvEnd::Terminal, &s);
                            }
                            eval.wv
                        } else {
                            let line = vec![root_key, search_key(&s)];
                            search_own(
                                db,
                                &s,
                                me,
                                self.depth.saturating_sub(1),
                                self.beam,
                                &mut nodes,
                                cap,
                                &line,
                                eval,
                                odepth,
                                obeam,
                                &mut dec_stats,
                                table.as_mut(),
                                tracker.as_mut(),
                            )
                        };
                        if let Some(t) = tracker.as_mut() {
                            t.note_nodes(nodes);
                        }
                        let fv = finite(v, eval.wv);
                        if recording {
                            let tracker = tracker.unwrap_or_else(|| PvTracker::new(cap));
                            explain_cands[j].worlds.push(tracker.into_world(
                                r as u32,
                                v,
                                fv,
                                false,
                                nodes - pair_nodes_start,
                                db,
                                root,
                            ));
                        }
                        acc[j] += fv;
                        if fv < worst[j] {
                            worst[j] = fv;
                        }
                        n[j] += 1;
                    }
                    if cap_exhausted {
                        break;
                    }
                }
                if nodes_after_lethal < self.node_cap {
                    self.stats.pairs_skipped += total_pairs - attempted;
                }

                let mut best_i = 0usize;
                let mut best_v = f32::NEG_INFINITY;
                let mut any_scored = false;
                for (j, &c) in n.iter().enumerate() {
                    if c == 0 {
                        continue;
                    }
                    any_scored = true;
                    let v = root_agg(acc[j], c, worst[j], self.pess);
                    if recording {
                        explain_cands[j].root_agg = v;
                        explain_cands[j].worst = worst[j];
                        explain_cands[j].n = c;
                    }
                    if v > best_v {
                        best_v = v;
                        best_i = j;
                    }
                }
                if any_scored {
                    const LETHAL_EPS: f32 = 1e-3;
                    for (j, &c) in n.iter().enumerate() {
                        if c == 0 {
                            continue;
                        }
                        if worst[j] <= -self.wv + LETHAL_EPS {
                            dec_stats.cands_with_lethal_root += 1;
                            if j == best_i {
                                dec_stats.chose_with_lethal_root += 1;
                            }
                        }
                    }
                }
                self.last_value = Some(finite(best_v, self.wv));
                if !any_scored {
                    self.stats.unscored += 1;
                }
                if let Some(rec) = &mut explain_rec {
                    rec.chosen_index = cand[best_i];
                    if any_scored {
                        rec.tie_set = tie_set_search(&n, &acc, &worst, self.pess, best_v, &cand);
                    } else {
                        rec.path = ChoosePath::Unscored;
                        rec.tie_set = cand.clone();
                    }
                    rec.candidates = explain_cands;
                }
                cand[best_i]
            }
        };
        self.stats.nodes += u64::from(nodes);
        if nodes >= self.node_cap {
            self.stats.cap_hits += 1;
        }
        if let Some(spent) = lethal_spent {
            self.stats.lethal_nodes += u64::from(spent);
        }
        self.stats.accum(&dec_stats);
        if let Some(mut rec) = explain_rec {
            rec.nodes = nodes;
            self.explain = Some(rec);
        }
        pick
    }

    fn last_value(&self) -> Option<f32> {
        self.last_value
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

fn finite(v: f32, wv: f32) -> f32 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-wv, wv)
    }
}

/// Root aggregation over the determinizations that scored candidate `j`.
/// `pess == 0.0` is today's mean, written as the existing expression so
/// the default path cannot drift.
fn root_agg(acc: f32, n: u32, worst: f32, pess: f32) -> f32 {
    if pess == 0.0 {
        acc / n as f32
    } else {
        let mean = acc / n as f32;
        (1.0 - pess) * mean + pess * worst
    }
}

fn tie_set_search(
    n: &[u32],
    acc: &[f32],
    worst: &[f32],
    pess: f32,
    best_v: f32,
    cand: &[usize],
) -> Vec<usize> {
    let mut out = Vec::new();
    for (j, &c) in n.iter().enumerate() {
        if c == 0 {
            continue;
        }
        if root_agg(acc[j], c, worst[j], pess) == best_v {
            out.push(cand[j]);
        }
    }
    out
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

#[allow(clippy::too_many_arguments)]
fn one_ply(
    roots: &[State],
    db: &CardDb,
    subset: &[Action],
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    eval: Evaluator<'_>,
    pess: f32,
    tie_out: Option<(&[usize], &mut Vec<usize>)>,
) -> (usize, f32) {
    let mut acc = vec![0.0f32; subset.len()];
    let mut n = vec![0u32; subset.len()];
    let mut worst = vec![f32::INFINITY; subset.len()];
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
                eval.wv
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
            let fv = finite(v, eval.wv);
            acc[j] += fv;
            if fv < worst[j] {
                worst[j] = fv;
            }
            n[j] += 1;
        }
    }
    let mut best_i = 0usize;
    let mut best_v = f32::NEG_INFINITY;
    for (j, &c) in n.iter().enumerate() {
        if c == 0 {
            continue;
        }
        let v = root_agg(acc[j], c, worst[j], pess);
        if v > best_v {
            best_v = v;
            best_i = j;
        }
    }
    let best_v = finite(best_v, eval.wv);
    if let Some((cand, out)) = tie_out {
        *out = tie_set_search(&n, &acc, &worst, pess, best_v, cand);
    }
    (best_i, best_v)
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

#[allow(clippy::too_many_arguments, clippy::needless_option_as_deref)]
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
    mut track: Option<&mut PvTracker>,
) -> f32 {
    if state.winner == Some(me) {
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(INF, PvEnd::Terminal, state);
        }
        return INF;
    }
    if state.winner == Some(me.opponent()) {
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(-INF, PvEnd::Terminal, state);
        }
        return -INF;
    }
    if *nodes >= cap {
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::Cap, state);
        }
        return v;
    }
    if acting_player(state) != me || matches!(state.phase, Phase::Terminal) {
        if let Some(v) = tt_get(tt.as_deref(), state, depth, false, stats) {
            if let Some(t) = track.as_deref_mut() {
                t.set_tt(v, state);
            }
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
            track.as_deref_mut(),
        );
        tt_put(tt, state, depth, false, v, stats);
        return v;
    }
    if depth == 0 {
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::Depth, state);
        }
        return v;
    }
    if let Some(v) = tt_get(tt.as_deref(), state, depth, true, stats) {
        if let Some(t) = track.as_deref_mut() {
            t.set_tt(v, state);
        }
        return v;
    }
    let legal = legal_actions(db, state);
    if legal.is_empty() {
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::Terminal, state);
        }
        return v;
    }

    let mut scored: Vec<(f32, State, u64, Action)> = Vec::with_capacity(legal.len());
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
            if let Some(t) = track.as_deref_mut() {
                t.push(a.clone());
                t.set_leaf(INF, PvEnd::Terminal, &s);
            }
            tt_put(tt, state, depth, true, INF, stats);
            return INF;
        }
        let k = search_key(&s);
        scored.push((eval.value(&s, me), s, k, a.clone()));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(beam.max(1));

    let mut best = f32::NEG_INFINITY;
    let mut best_line: Option<Line> = None;
    for (_, s, k, a) in scored {
        let plen = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
        if let Some(t) = track.as_deref_mut() {
            t.push(a);
        }
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
            track.as_deref_mut(),
        );
        if v > best {
            best = v;
            if let Some(t) = track.as_deref_mut() {
                best_line = t.take_last();
            }
        }
        if let Some(t) = track.as_deref_mut() {
            t.pop_to(plen);
        }
    }
    if let Some(t) = track.as_deref_mut() {
        t.restore_last(best_line);
    }
    tt_put(tt, state, depth, true, best, stats);
    best
}

#[allow(clippy::too_many_arguments, clippy::needless_option_as_deref)]
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
    mut track: Option<&mut PvTracker>,
) -> f32 {
    if state.winner == Some(me) {
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(INF, PvEnd::Terminal, state);
        }
        return INF;
    }
    if state.winner == Some(me.opponent()) {
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(-eval.wv, PvEnd::Terminal, state);
        }
        return -eval.wv;
    }
    if odepth == 0 {
        if eval.olethal
            && acting_player(state) == me.opponent()
            && opp_lethal_sweep(db, state, me, nodes, cap, line, eval.oevo, stats)
        {
            stats.opp_leaves += 1;
            if let Some(t) = track.as_deref_mut() {
                t.set_leaf(-eval.wv, PvEnd::OppLethal, state);
            }
            return -eval.wv;
        }
        let mut s = state.clone();
        greedy_until_end(
            db,
            &mut s,
            me.opponent(),
            nodes,
            cap,
            line,
            eval,
            track.as_deref_mut(),
        );
        if *nodes >= cap {
            stats.opp_cap_hits += 1;
        }
        stats.opp_leaves += 1;
        let v = eval.value(&s, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::OppReply, &s);
        }
        return v;
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
        track.as_deref_mut(),
    );
    if truncated {
        stats.opp_cap_hits += 1;
    }
    v
}

/// Maximising `eval.value(s, opp)` is minimising `eval.value(s, me)`: the
/// leaf value is antisymmetric term by term (`value(s, A) == -value(s, B)`).
#[allow(clippy::too_many_arguments, clippy::needless_option_as_deref)]
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
    mut track: Option<&mut PvTracker>,
) -> f32 {
    let opp = me.opponent();
    if state.winner == Some(me) {
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(INF, PvEnd::Terminal, state);
        }
        return INF;
    }
    if state.winner == Some(opp) {
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(-eval.wv, PvEnd::Terminal, state);
        }
        return -eval.wv;
    }
    if matches!(state.phase, Phase::Terminal) || state.turn > MAX_TURNS {
        stats.opp_leaves += 1;
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::OppSearch, state);
        }
        return v;
    }
    if *nodes >= cap {
        *truncated = true;
        stats.opp_leaves += 1;
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::Cap, state);
        }
        return v;
    }

    let me_to_move = acting_player(state) == me;
    if let Some(v) = tt_get(tt.as_deref(), state, depth, me_to_move, stats) {
        if let Some(t) = track.as_deref_mut() {
            t.set_tt(v, state);
        }
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
        track.as_deref_mut(),
    );
    tt_put(tt, state, depth, me_to_move, v, stats);
    v
}

#[allow(clippy::too_many_arguments, clippy::needless_option_as_deref)]
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
    mut track: Option<&mut PvTracker>,
) -> f32 {
    let opp = me.opponent();

    // Effect handed a choice to `me` while it is still the opponent's turn.
    if acting_player(state) == me && state.active == opp {
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            stats.opp_leaves += 1;
            let v = eval.value(state, me);
            if let Some(t) = track.as_deref_mut() {
                t.set_leaf(v, PvEnd::OppSearch, state);
            }
            return v;
        }
        let i = greedy_index(db, state, &legal, me, nodes, cap, line, eval);
        let plen = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
        if let Some(t) = track.as_deref_mut() {
            t.push(legal[i].clone());
        }
        let Some(s) = try_apply(db, state, &legal[i], nodes, cap, line) else {
            if let Some(t) = track.as_deref_mut() {
                t.pop_to(plen);
            }
            if *nodes >= cap {
                *truncated = true;
            }
            stats.opp_leaves += 1;
            let v = eval.value(state, me);
            if let Some(t) = track.as_deref_mut() {
                t.set_leaf(v, PvEnd::OppSearch, state);
            }
            return v;
        };
        let mut next_line = line.to_vec();
        next_line.push(search_key(&s));
        let v = search_opp(
            db,
            &s,
            me,
            depth,
            beam,
            nodes,
            cap,
            &next_line,
            eval,
            stats,
            truncated,
            tt.as_deref_mut(),
            track.as_deref_mut(),
        );
        if let Some(t) = track.as_deref_mut() {
            t.pop_to(plen);
        }
        return v;
    }

    if acting_player(state) != opp {
        stats.opp_leaves += 1;
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::OppSearch, state);
        }
        return v;
    }
    if depth == 0 {
        stats.opp_leaves += 1;
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::OppSearch, state);
        }
        return v;
    }

    let legal = legal_actions(db, state);
    if legal.is_empty() {
        stats.opp_leaves += 1;
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::OppSearch, state);
        }
        return v;
    }

    let mut scored: Vec<(f32, State, u64, Action)> = Vec::new();
    let mut end_turn: Option<(State, u64, Action)> = None;
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
            if let Some(t) = track.as_deref_mut() {
                let plen = t.path_len();
                t.push(a.clone());
                t.set_leaf(-eval.wv, PvEnd::Terminal, &s);
                t.pop_to(plen);
            }
            return -eval.wv;
        }
        let k = search_key(&s);
        if matches!(a, Action::EndTurn) {
            end_turn = Some((s, k, a.clone()));
        } else {
            scored.push((eval.value(&s, opp), s, k, a.clone()));
        }
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(beam.max(1));
    // EndTurn is always kept so "do nothing more" is a considered line — that
    // is what removes the greedy 3-step cutoff honestly.
    if let Some((s, k, a)) = end_turn {
        scored.push((0.0, s, k, a));
    }
    if scored.is_empty() {
        stats.opp_leaves += 1;
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::OppSearch, state);
        }
        return v;
    }

    let mut worst = f32::INFINITY;
    let mut worst_line: Option<Line> = None;
    for (_, s, k, a) in scored {
        let plen = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
        if let Some(t) = track.as_deref_mut() {
            t.push(a);
        }
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
            track.as_deref_mut(),
        );
        if let Some(t) = track.as_deref_mut() {
            t.pop_to(plen);
        }
        if v == -eval.wv {
            return -eval.wv;
        }
        if v < worst {
            worst = v;
            if let Some(t) = track.as_deref_mut() {
                worst_line = t.take_last();
            }
        }
    }
    if worst.is_finite() {
        if let Some(t) = track.as_deref_mut() {
            t.restore_last(worst_line);
        }
        worst
    } else {
        stats.opp_leaves += 1;
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::OppSearch, state);
        }
        v
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
    mut track: Option<&mut PvTracker>,
) {
    let mut steps = 0u32;
    let mut line = line.to_vec();
    while state.winner.is_none()
        && !matches!(state.phase, Phase::Terminal)
        && acting_player(state) == who
        && *nodes < cap
        && steps < eval.osteps + 3
        && state.turn <= MAX_TURNS
    {
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            break;
        }
        if let Some(end) = legal.iter().position(|a| matches!(a, Action::EndTurn)) {
            if steps >= eval.osteps {
                if let Some(next) = try_apply(db, state, &legal[end], nodes, cap, &line) {
                    if let Some(t) = track.as_deref_mut() {
                        t.push(legal[end].clone());
                    }
                    *state = next;
                }
                break;
            }
        }
        let i = greedy_index(db, state, &legal, who, nodes, cap, &line, eval);
        let Some(next) = try_apply(db, state, &legal[i], nodes, cap, &line) else {
            break;
        };
        if let Some(t) = track.as_deref_mut() {
            t.push(legal[i].clone());
        }
        line.push(search_key(&next));
        *state = next;
        steps += 1;
    }
}

/// Applies spent by one opponent-lethal sweep, charged through [`try_apply`].
/// Tight for the attack-only + play-then-attack loop; do not raise.
const OPP_LETHAL_APPLY_CAP: u32 = 40;
/// Same sweep when `oevo=1` (the default). Play loop plus at most one
/// evolve per leaf (and the face line after it). 120 is the worst-case
/// envelope — a wide hand of opening plays, each trying several evolve
/// slots × a short face line, plus the standalone pass — not the mean.
/// The 50-game abyss-p8rfn mirror measured 8.2 applies/sweep (`oevo=1`)
/// vs 6.1 (`oevo=0`); the play-then-evolve fixture spends 7.
const OPP_LETHAL_EVO_APPLY_CAP: u32 = 120;

fn leader_attackers(db: &CardDb, state: &State) -> Vec<u8> {
    legal_actions(db, state)
        .into_iter()
        .filter_map(|a| match a {
            Action::Attack {
                attacker,
                target: AttackTarget::Leader,
            } => Some(attacker.0),
            _ => None,
        })
        .collect()
}

/// First legal face attack, then the next, until none remain. Attacks to the
/// leader commute, so one fixed order suffices.
fn face_line_kills(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> bool {
    let opp = me.opponent();
    let mut s = state.clone();
    let mut line = line.to_vec();
    loop {
        if s.winner == Some(opp) {
            return true;
        }
        if s.winner.is_some() || matches!(s.phase, Phase::Terminal) {
            return false;
        }
        if acting_player(&s) != opp || *nodes >= cap {
            return false;
        }
        let legal = legal_actions(db, &s);
        let Some(a) = legal.iter().find(|a| {
            matches!(
                a,
                Action::Attack {
                    target: AttackTarget::Leader,
                    ..
                }
            )
        }) else {
            return false;
        };
        let Some(next) = try_apply(db, &s, a, nodes, cap, &line) else {
            return false;
        };
        line.push(search_key(&next));
        s = next;
    }
}

/// Depth-first resolve of a play: pending `Choose` in index order, then any
/// required `Confirm`. Then a face line only if the play added a leader
/// attack or lowered my defense. With `oevo`, a miss on that face line
/// tries each legal evolve before giving up.
#[allow(clippy::too_many_arguments)]
fn resolve_play_line(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    origin_def: i32,
    origin_faces: &[u8],
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    oevo: bool,
    evo_found: &mut bool,
) -> bool {
    let opp = me.opponent();
    if state.winner == Some(opp) {
        return true;
    }
    if state.winner.is_some() || matches!(state.phase, Phase::Terminal) || *nodes >= cap {
        return false;
    }

    if matches!(state.phase, Phase::Choice { .. }) {
        let legal = legal_actions(db, state);
        let chooses: Vec<Action> = legal
            .iter()
            .filter(|a| matches!(a, Action::Choose(_)))
            .cloned()
            .collect();
        for a in &chooses {
            if *nodes >= cap {
                return false;
            }
            let Some(s) = try_apply(db, state, a, nodes, cap, line) else {
                continue;
            };
            let mut next_line = line.to_vec();
            next_line.push(search_key(&s));
            if resolve_play_line(
                db,
                &s,
                me,
                origin_def,
                origin_faces,
                nodes,
                cap,
                &next_line,
                oevo,
                evo_found,
            ) {
                return true;
            }
        }
        if chooses.is_empty() {
            if let Some(a) = legal.iter().find(|a| matches!(a, Action::Confirm)) {
                let Some(s) = try_apply(db, state, a, nodes, cap, line) else {
                    return false;
                };
                let mut next_line = line.to_vec();
                next_line.push(search_key(&s));
                return resolve_play_line(
                    db,
                    &s,
                    me,
                    origin_def,
                    origin_faces,
                    nodes,
                    cap,
                    &next_line,
                    oevo,
                    evo_found,
                );
            }
        }
        return false;
    }

    if acting_player(state) != opp {
        return false;
    }
    let lowered = state.player(me).leader_defense < origin_def;
    let now_faces = leader_attackers(db, state);
    let new_face = now_faces.iter().any(|slot| !origin_faces.contains(slot));
    if !(new_face || lowered) {
        return false;
    }
    if face_line_kills(db, state, me, nodes, cap, line) {
        return true;
    }
    if oevo {
        let just_played: Vec<u8> = now_faces
            .iter()
            .copied()
            .filter(|slot| !origin_faces.contains(slot))
            .collect();
        if evolve_then_face(db, state, me, nodes, cap, line, &just_played, &now_faces) {
            *evo_found = true;
            return true;
        }
    }
    false
}

/// Legal `Evolve` / `super_evolve` actions, cheapest-and-highest-yield first:
/// the just-played slot, then other current face attackers, then the rest.
/// `super_evolve` variants stay in the list; they are not special-cased.
fn ordered_evolves(db: &CardDb, state: &State, prefer: &[u8], faces: &[u8]) -> Vec<Action> {
    let mut evos: Vec<Action> = legal_actions(db, state)
        .into_iter()
        .filter(|a| matches!(a, Action::Evolve { .. }))
        .collect();
    evos.sort_by_key(|a| match a {
        Action::Evolve { slot, super_evolve } => {
            let s = slot.0;
            let pri = if prefer.contains(&s) {
                0u8
            } else if faces.contains(&s) {
                1
            } else {
                2
            };
            (pri, s, *super_evolve)
        }
        _ => (3, 0, false),
    });
    evos
}

#[allow(clippy::too_many_arguments)]
fn evolve_then_face(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    prefer: &[u8],
    faces: &[u8],
) -> bool {
    for a in ordered_evolves(db, state, prefer, faces) {
        if *nodes >= cap {
            return false;
        }
        let Some(s) = try_apply(db, state, &a, nodes, cap, line) else {
            continue;
        };
        let mut next_line = line.to_vec();
        next_line.push(search_key(&s));
        if face_line_kills(db, &s, me, nodes, cap, &next_line) {
            return true;
        }
    }
    false
}

/// Standalone evolve — no play. Covers "+2 face from an on-board attacker".
fn standalone_evolve_kills(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> bool {
    let legal = legal_actions(db, state);
    for a in &legal {
        if *nodes >= cap {
            return false;
        }
        if !matches!(a, Action::Evolve { .. }) {
            continue;
        }
        let Some(s) = try_apply(db, state, a, nodes, cap, line) else {
            continue;
        };
        let mut next_line = line.to_vec();
        next_line.push(search_key(&s));
        if face_line_kills(db, &s, me, nodes, cap, &next_line) {
            return true;
        }
    }
    false
}

/// Glance-level opponent lethal: face attacks, then each `Play` (plus its
/// `Choose`/`Confirm`) and a face line when that play opened one. With
/// `oevo=1` (the default), a play that opened a line but missed face then
/// tries each legal evolve (just-played slot first), and a standalone
/// evolve pass runs if those also miss. Caps at [`OPP_LETHAL_APPLY_CAP`]
/// (`oevo=0`) or [`OPP_LETHAL_EVO_APPLY_CAP`] (`oevo=1`), all charged to
/// `nodes`.
#[allow(clippy::too_many_arguments)]
fn opp_lethal_sweep(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    oevo: bool,
    stats: &mut SearchStats,
) -> bool {
    stats.opp_lethal_checks += 1;
    let start = *nodes;
    let apply_limit = if oevo {
        OPP_LETHAL_EVO_APPLY_CAP
    } else {
        OPP_LETHAL_APPLY_CAP
    };
    let sweep_cap = nodes.saturating_add(apply_limit).min(cap);
    let mut evo_found = false;
    let found = face_line_kills(db, state, me, nodes, sweep_cap, line) || {
        let origin_def = state.player(me).leader_defense;
        let origin_faces = leader_attackers(db, state);
        let legal = legal_actions(db, state);
        let mut hit = false;
        for a in &legal {
            if *nodes >= sweep_cap {
                break;
            }
            if !matches!(a, Action::Play { .. }) {
                continue;
            }
            let Some(s) = try_apply(db, state, a, nodes, sweep_cap, line) else {
                continue;
            };
            let mut next_line = line.to_vec();
            next_line.push(search_key(&s));
            if resolve_play_line(
                db,
                &s,
                me,
                origin_def,
                &origin_faces,
                nodes,
                sweep_cap,
                &next_line,
                oevo,
                &mut evo_found,
            ) {
                hit = true;
                break;
            }
        }
        if hit {
            true
        } else if oevo && standalone_evolve_kills(db, state, me, nodes, sweep_cap, line) {
            evo_found = true;
            true
        } else {
            false
        }
    };
    stats.opp_lethal_nodes += u64::from(*nodes - start);
    if found {
        stats.opp_lethal_found += 1;
        if evo_found {
            stats.opp_lethal_evo_found += 1;
        }
    }
    found
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
