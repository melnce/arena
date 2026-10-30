//! H0 decision job: one `choose` split into steps for multi-instance wasm workers.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::action::{acting_player, Action};
use crate::apply::legal_actions;
use crate::db::CardDb;
use crate::determinize::{determinize_block, OpenStats};
use crate::ids::PlayerId;
use crate::policy::AnyPolicy;
use crate::rng::{policy_rng, Xoshiro256ss};
use crate::state::{Phase, State};

use crate::policy::explain::{
    CandidateRecord, ChoosePath, ExplainRecord, LostRerankRecord, PvEnd, PvTracker,
};
use crate::policy::fuse_guard::filter_cand_fuseguard;

#[cfg(not(target_arch = "wasm32"))]
use super::try_take_kill_parallel;
use super::{
    all_cands_lost, consensus_lethal, finite, kill_det_on_roots, kill_roll_on_roots, line_key,
    one_ply, own_fuse_partners, root_agg, search_own, search_world, tie_set_search, try_apply,
    try_holdback_resample, try_holdback_trade, try_lost_rerank, try_take_kill, useful_action,
    Alloc, Deal, Info, SearchStats, WorldSearchResult, H0,
};

/// Run one full [`H0::choose`] (in-process).
pub(super) fn choose_sync(
    h: &mut H0,
    db: &CardDb,
    state: &State,
    legal: &[Action],
    rng: &mut Xoshiro256ss,
) -> usize {
    run_h0_decision_mode(h, db, state, legal, rng, JobExecMode::Direct, None)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JobExecMode {
    /// Search only (kill already handled by the job driver).
    ViaJobPostKill,
    /// Native `choose` parallelism when configured.
    Direct,
}

struct PreparedRoots {
    me: PlayerId,
    cand: Vec<usize>,
    subset: Vec<Action>,
    roots: Vec<State>,
    k: u32,
    fuse_dropped: u32,
    root_world_seeds: Vec<u64>,
    wbase_used: Option<u64>,
    wbase_reused: bool,
    deal_seed: Option<u64>,
}

#[allow(clippy::too_many_arguments)]
fn merge_world_results(
    world_results: &[WorldSearchResult],
    subset_len: usize,
    acc: &mut [f32],
    n: &mut [u32],
    worst: &mut [f32],
    dec_stats: &mut SearchStats,
    explain_cands: &mut [CandidateRecord],
    recording: bool,
    attempted: &mut u64,
) -> u32 {
    let mut world_nodes_sum = 0u32;
    for wr in world_results {
        *attempted += wr.attempted;
        dec_stats.accum(&wr.stats);
        dec_stats.skipped_worlds += wr.skipped_worlds;
        world_nodes_sum += wr.nodes_spent;
        if recording {
            for (j, worlds) in wr.explain.iter().enumerate() {
                for w in worlds {
                    explain_cands[j].worlds.push(w.clone());
                }
            }
        }
        for j in 0..subset_len {
            acc[j] += wr.acc[j];
            if wr.n[j] > 0 && wr.worst[j] < worst[j] {
                worst[j] = wr.worst[j];
            }
            n[j] += wr.n[j];
        }
    }
    world_nodes_sum
}

#[allow(clippy::too_many_arguments)]
fn pick_after_world_merge(
    h: &mut H0,
    db: &CardDb,
    state: &State,
    roots: &[State],
    subset: &[Action],
    cand: &[usize],
    me: PlayerId,
    odepth: u32,
    obeam: usize,
    k: u32,
    wbase_used: Option<u64>,
    root_world_seeds: &[u64],
    acc: &[f32],
    n: &[u32],
    worst: &[f32],
    pairs_skipped_add: u64,
    recording: bool,
    explain_rec: &mut Option<ExplainRecord>,
    explain_cands: &mut [CandidateRecord],
    dec_stats: &mut SearchStats,
) -> usize {
    let root_vocab = h.root_vocab(state);
    let eval = h.evaluator(db, &root_vocab);
    let mut best_i = 0usize;
    let mut best_v = f32::NEG_INFINITY;
    let mut any_scored = false;
    for (j, &c) in n.iter().enumerate() {
        if c == 0 {
            continue;
        }
        any_scored = true;
        let v = root_agg(acc[j], c, worst[j], h.pess);
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
            if worst[j] <= -h.wv + LETHAL_EPS {
                dec_stats.cands_with_lethal_root += 1;
                if j == best_i {
                    dec_stats.chose_with_lethal_root += 1;
                }
            }
        }
    }
    let pick_last_value = finite(best_v, h.wv);
    let unscored_inc = if !any_scored { 1 } else { 0 };
    let mut lost_rerank_rec: Option<LostRerankRecord> = None;
    let (pick_i, pick_last_value) =
        if h.lostrank > 0 && any_scored && all_cands_lost(n, acc, worst, h.pess, h.wv) {
            if let Some((j, _, spent, aggregates)) = try_lost_rerank(
                h.lostrank,
                h.pess,
                h.horizon,
                h.hres,
                db,
                roots,
                subset,
                me,
                eval,
                odepth,
                obeam,
                h.fusemacro,
                h.depth,
                h.beam,
                dec_stats,
            ) {
                lost_rerank_rec = Some(LostRerankRecord {
                    aggregates,
                    nodes: spent,
                    chosen_index: cand[j],
                });
                (j, pick_last_value)
            } else {
                (best_i, pick_last_value)
            }
        } else {
            (best_i, pick_last_value)
        };
    let (chosen, override_path, pick_last_value) = if lost_rerank_rec.is_some() {
        (cand[pick_i], None, pick_last_value)
    } else if any_scored && matches!(subset[pick_i], Action::EndTurn) {
        let mut chosen_idx = cand[pick_i];
        let mut lv = pick_last_value;
        let mut path = None;
        if h.hbk > 0 {
            if let Some((idx, new_lv)) = try_holdback_resample(
                h.hbk,
                h.pess,
                h.node_cap,
                k,
                h.depth,
                h.beam,
                h.tt,
                h.info,
                h.deal,
                h.turn_stable_worlds(),
                wbase_used,
                root_world_seeds,
                h.hread_deal(),
                db,
                roots,
                state,
                me,
                subset,
                cand,
                acc,
                n,
                worst,
                pick_i,
                eval,
                odepth,
                obeam,
                h.fusemacro,
                h.horizon,
                h.hres,
                explain_rec,
                dec_stats,
            ) {
                chosen_idx = idx;
                lv = new_lv;
                path = Some(ChoosePath::HoldbackResample);
            }
        }
        if path.is_none() && h.hbcheck > 0 {
            if let Some((idx, new_lv)) = try_holdback_trade(
                h.hbcheck,
                h.pess,
                h.horizon,
                h.hres,
                h.osteps,
                db,
                roots,
                subset,
                cand,
                n,
                pick_i,
                me,
                eval,
                odepth,
                obeam,
                explain_rec,
                dec_stats,
            ) {
                chosen_idx = idx;
                lv = new_lv;
                path = Some(ChoosePath::HoldbackTrade);
            }
        }
        (chosen_idx, path, lv)
    } else {
        (cand[pick_i], None, pick_last_value)
    };
    h.stats.pairs_skipped += pairs_skipped_add;
    h.stats.unscored += unscored_inc;
    h.last_value = Some(pick_last_value);
    if let Some(rec) = explain_rec {
        rec.chosen_index = chosen;
        let did_lost_rerank = lost_rerank_rec.is_some();
        rec.lost_rerank = lost_rerank_rec;
        if any_scored {
            if override_path.is_some() || did_lost_rerank {
                rec.tie_set = vec![chosen];
            } else {
                rec.tie_set = tie_set_search(n, acc, worst, h.pess, best_v, cand);
            }
            if let Some(p) = override_path {
                rec.path = p;
            }
        } else {
            rec.path = ChoosePath::Unscored;
            rec.tie_set = cand.to_vec();
        }
        rec.candidates = explain_cands.to_vec();
        h.last_path = Some(rec.path);
    } else {
        let path = if !any_scored {
            ChoosePath::Unscored
        } else if let Some(p) = override_path {
            p
        } else {
            ChoosePath::Search
        };
        h.last_path = Some(path);
    }
    chosen
}

fn run_h0_decision_mode(
    h: &mut H0,
    db: &CardDb,
    state: &State,
    legal: &[Action],
    rng: &mut Xoshiro256ss,
    mode: JobExecMode,
    prepared: Option<&PreparedRoots>,
) -> usize {
    h.last_value = None;
    h.last_path = None;
    if mode != JobExecMode::ViaJobPostKill {
        h.stats.decisions += 1;
    }
    let recording = h.explain_armed;
    if recording {
        h.explain = None;
    }
    let mut shared_table = if h.info != Info::All {
        h.tt.then(HashMap::new)
    } else {
        None
    };
    if legal.len() <= 1 {
        if recording {
            let mut rec =
                ExplainRecord::new(ChoosePath::SingleLegal, 0, h.node_cap, h.alloc_label());
            rec.chosen_index = 0;
            rec.tie_set = vec![0];
            h.stamp_explain_deal(&mut rec, None);
            h.explain = Some(rec);
        }
        h.last_path = Some(ChoosePath::SingleLegal);
        return 0;
    }
    if matches!(state.phase, Phase::Mulligan { .. }) {
        let idx = h.choose_mulligan(db, state, legal, rng);
        if recording {
            let mut rec = ExplainRecord::new(ChoosePath::Mulligan, 0, h.node_cap, h.alloc_label());
            rec.chosen_index = idx;
            rec.tie_set = vec![idx];
            rec.mull = h.take_pending_mull_explain();
            h.stamp_explain_deal(&mut rec, None);
            h.explain = Some(rec);
        }
        h.last_path = Some(ChoosePath::Mulligan);
        return idx;
    }

    let (
        me,
        cand,
        subset,
        roots,
        k,
        fuse_dropped,
        root_world_seeds,
        wbase_used,
        deal_seed,
        wbase_reused,
    ) = if let Some(p) = prepared {
        (
            p.me,
            p.cand.clone(),
            p.subset.clone(),
            p.roots.clone(),
            p.k,
            p.fuse_dropped,
            p.root_world_seeds.clone(),
            p.wbase_used,
            p.deal_seed,
            false,
        )
    } else {
        match prepare_roots(h, db, state, legal, rng) {
            Ok(p) => (
                p.me,
                p.cand,
                p.subset,
                p.roots,
                p.k,
                p.fuse_dropped,
                p.root_world_seeds,
                p.wbase_used,
                p.deal_seed,
                p.wbase_reused,
            ),
            Err(JobStep::Done { index, value }) => {
                h.last_value = value;
                h.last_path = Some(ChoosePath::SingleLegal);
                return index as usize;
            }
            Err(_) => {
                h.last_path = Some(ChoosePath::SingleLegal);
                return 0;
            }
        }
    };
    if cand.is_empty() {
        if recording {
            let mut rec =
                ExplainRecord::new(ChoosePath::SingleLegal, 0, h.node_cap, h.alloc_label());
            rec.chosen_index = 0;
            rec.tie_set = vec![0];
            rec.fuse_dropped = fuse_dropped;
            h.stamp_explain_deal(&mut rec, None);
            h.explain = Some(rec);
        }
        h.last_path = Some(ChoosePath::SingleLegal);
        return 0;
    }
    if cand.len() == 1 {
        if recording {
            let mut rec =
                ExplainRecord::new(ChoosePath::SingleLegal, 0, h.node_cap, h.alloc_label());
            rec.chosen_index = cand[0];
            rec.tie_set = vec![cand[0]];
            rec.fuse_dropped = fuse_dropped;
            h.stamp_explain_deal(&mut rec, None);
            h.explain = Some(rec);
        }
        h.last_path = Some(ChoosePath::SingleLegal);
        return cand[0];
    }
    let mut nodes = 0u32;
    let mut dec_stats = SearchStats::default();
    let mut explain_rec = if recording {
        let mut rec = ExplainRecord::new(ChoosePath::Search, k, h.node_cap, h.alloc_label());
        rec.wbase = wbase_used;
        rec.wbase_reused = wbase_reused;
        rec.fuse_dropped = fuse_dropped;
        h.stamp_explain_deal(&mut rec, deal_seed);
        Some(rec)
    } else {
        None
    };
    if mode != JobExecMode::ViaJobPostKill
        && h.tkill > 0
        && matches!(
            state.phase,
            Phase::Main | Phase::Combat | Phase::Choice { .. }
        )
    {
        let kill = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if mode == JobExecMode::Direct && h.threads > 1 && h.tkroll > 0 {
                    try_take_kill_parallel(
                        db,
                        &roots,
                        legal,
                        &cand,
                        me,
                        h.tkill,
                        h.tkroll,
                        &mut dec_stats,
                    )
                } else {
                    try_take_kill(
                        db,
                        &roots,
                        legal,
                        &cand,
                        me,
                        h.tkill,
                        h.tkroll,
                        &mut dec_stats,
                    )
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                try_take_kill(
                    db,
                    &roots,
                    legal,
                    &cand,
                    me,
                    h.tkill,
                    h.tkroll,
                    &mut dec_stats,
                )
            }
        };
        if let Some((idx, path)) = kill {
            h.last_value = Some(h.wv);
            h.last_path = Some(path);
            if let Some(rec) = &mut explain_rec {
                rec.path = path;
                rec.chosen_index = idx;
                rec.tie_set = vec![idx];
            }
            h.stats.accum(&dec_stats);
            if let Some(rec) = explain_rec {
                h.explain = Some(rec);
            }
            return idx;
        }
    }
    let root_vocab = h.root_vocab(state);
    let eval = h.evaluator(db, &root_vocab);
    let odepth = h.odepth;
    let obeam = h.obeam;
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
    let pick = if h.depth <= 2 {
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
            h.node_cap,
            eval,
            h.pess,
            if recording {
                Some((&cand, &mut tie_set))
            } else {
                None
            },
        );
        h.last_value = Some(v);
        h.last_path = Some(ChoosePath::OnePly);
        if let Some(rec) = &mut explain_rec {
            rec.chosen_index = cand[j];
            rec.tie_set = tie_set;
        }
        cand[j]
    } else {
        let nodes_before_lethal = nodes;
        let lethal_cap = h.lethal_cap(nodes);
        let lethal = consensus_lethal(db, &roots, &subset, me, 2, &mut nodes, lethal_cap);
        lethal_spent = Some(nodes - nodes_before_lethal);
        if let Some(j) = lethal {
            if let Some(rec) = &mut explain_rec {
                rec.path = ChoosePath::ConsensusLethal;
                rec.nodes_lethal = nodes - nodes_before_lethal;
                rec.chosen_index = cand[j];
                rec.tie_set = vec![cand[j]];
            }
            h.last_value = Some(h.wv);
            h.last_path = Some(ChoosePath::ConsensusLethal);
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
            if h.alloc == Alloc::World {
                let leftover = h.node_cap.saturating_sub(nodes_after_lethal);
                let base = leftover / k;
                let extra = leftover % k;
                let mut world_results: Vec<WorldSearchResult> = Vec::with_capacity(roots.len());
                let run_world = |r: usize| {
                    let world_budget = base + if (r as u32) < extra { 1 } else { 0 };
                    search_world(
                        db,
                        &roots[r],
                        r,
                        world_budget,
                        &subset,
                        me,
                        h.depth,
                        h.beam,
                        h.tt,
                        h.info,
                        eval,
                        odepth,
                        obeam,
                        h.fusemacro,
                        h.horizon,
                        h.hres,
                        recording,
                    )
                };
                #[cfg(not(target_arch = "wasm32"))]
                if mode == JobExecMode::Direct && h.threads > 1 {
                    use std::sync::atomic::{AtomicUsize, Ordering};
                    let slots: Vec<std::sync::Mutex<Option<WorldSearchResult>>> = (0..roots.len())
                        .map(|_| std::sync::Mutex::new(None))
                        .collect();
                    let next = AtomicUsize::new(0);
                    let workers = h.threads.min(k).max(1) as usize;
                    std::thread::scope(|s| {
                        for _ in 0..workers {
                            s.spawn(|| loop {
                                let r = next.fetch_add(1, Ordering::Relaxed);
                                if r >= roots.len() {
                                    break;
                                }
                                let wr = run_world(r);
                                *slots[r].lock().unwrap() = Some(wr);
                            });
                        }
                    });
                    for slot in slots {
                        world_results.push(slot.lock().unwrap().take().expect("world"));
                    }
                } else {
                    for r in 0..roots.len() {
                        world_results.push(run_world(r));
                    }
                }
                #[cfg(target_arch = "wasm32")]
                for r in 0..roots.len() {
                    world_results.push(run_world(r));
                }
                let world_nodes_sum = merge_world_results(
                    &world_results,
                    subset.len(),
                    &mut acc,
                    &mut n,
                    &mut worst,
                    &mut dec_stats,
                    &mut explain_cands,
                    recording,
                    &mut attempted,
                );
                nodes = nodes_after_lethal + world_nodes_sum;
            } else {
                for (r, root) in roots.iter().enumerate() {
                    let mut root_table = if h.info == Info::All {
                        h.tt.then(HashMap::new)
                    } else {
                        None
                    };
                    let root_key = line_key(root);
                    for (j, a) in subset.iter().enumerate() {
                        if nodes >= h.node_cap {
                            let skipped = (subset.len() - j) as u64
                                + (roots.len() - r - 1) as u64 * subset.len() as u64;
                            dec_stats.skipped_worlds += skipped;
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
                        let cap = if h.alloc == Alloc::Root {
                            h.node_cap
                        } else {
                            const MIN_SHARE: u32 = 24;
                            let pairs_left = (k as usize - r) * subset.len() - j;
                            let remaining = h.node_cap - nodes;
                            let even = remaining / pairs_left as u32;
                            let share = if remaining >= MIN_SHARE.saturating_mul(pairs_left as u32)
                            {
                                even.max(MIN_SHARE)
                            } else {
                                even.max(1)
                            };
                            nodes.saturating_add(share).min(h.node_cap)
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
                            let line = vec![root_key, line_key(&s)];
                            let at_fuse_choice = h.fusemacro && own_fuse_partners(&s, me).is_some();
                            let search_depth = if at_fuse_choice {
                                h.depth
                            } else {
                                h.depth.saturating_sub(1)
                            };
                            search_own(
                                db,
                                &s,
                                me,
                                search_depth,
                                h.beam,
                                &mut nodes,
                                cap,
                                &line,
                                eval,
                                odepth,
                                obeam,
                                &mut dec_stats,
                                if h.info == Info::All {
                                    root_table.as_mut()
                                } else {
                                    shared_table.as_mut()
                                },
                                tracker.as_mut(),
                                h.fusemacro,
                                h.horizon,
                                h.hres,
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
            }
            let pairs_skipped_add = if nodes_after_lethal < h.node_cap {
                total_pairs - attempted
            } else {
                0
            };

            pick_after_world_merge(
                h,
                db,
                state,
                &roots,
                &subset,
                &cand,
                me,
                odepth,
                obeam,
                k,
                wbase_used,
                &root_world_seeds,
                &acc,
                &n,
                &worst,
                pairs_skipped_add,
                recording,
                &mut explain_rec,
                &mut explain_cands,
                &mut dec_stats,
            )
        }
    };
    h.stats.nodes += u64::from(nodes);
    if nodes >= h.node_cap {
        h.stats.cap_hits += 1;
    }
    if let Some(spent) = lethal_spent {
        h.stats.lethal_nodes += u64::from(spent);
    }
    h.stats.accum(&dec_stats);
    if let Some(mut rec) = explain_rec {
        rec.nodes = nodes;
        rec.horizon_nodes = dec_stats.horizon_nodes as u32;
        if h.last_path.is_none() {
            h.last_path = Some(rec.path);
        }
        h.explain = Some(rec);
    }
    pick
}

fn f32_bits(v: f32) -> u32 {
    v.to_bits()
}
fn f32_from_bits(b: u32) -> f32 {
    f32::from_bits(b)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillDetOutcome {
    pub taken_index: Option<u32>,
    pub nodes: u32,
    pub stats: SearchStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillRollOutcome {
    pub taken_index: Option<u32>,
    pub nodes: u32,
    pub stats: SearchStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldOutcome {
    pub r: u32,
    pub acc_bits: Vec<u32>,
    pub n: Vec<u32>,
    pub worst_bits: Vec<u32>,
    pub nodes_spent: u32,
    pub attempted: u64,
    pub skipped_worlds: u64,
    pub stats: SearchStats,
}

impl WorldOutcome {
    fn from_wr(r: u32, wr: &WorldSearchResult) -> Self {
        Self {
            r,
            acc_bits: wr.acc.iter().map(|v| f32_bits(*v)).collect(),
            n: wr.n.clone(),
            worst_bits: wr.worst.iter().map(|v| f32_bits(*v)).collect(),
            nodes_spent: wr.nodes_spent,
            attempted: wr.attempted,
            skipped_worlds: wr.skipped_worlds,
            stats: wr.stats.clone(),
        }
    }

    fn to_search_result(&self) -> WorldSearchResult {
        WorldSearchResult {
            acc: self.acc_bits.iter().map(|b| f32_from_bits(*b)).collect(),
            n: self.n.clone(),
            worst: self.worst_bits.iter().map(|b| f32_from_bits(*b)).collect(),
            nodes_spent: self.nodes_spent,
            stats: self.stats.clone(),
            attempted: self.attempted,
            skipped_worlds: self.skipped_worlds,
            explain: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStep {
    Done { index: u32, value: Option<f32> },
    Kill { roll: bool },
    AwaitRoll,
    Worlds { budgets: Vec<u32> },
}

impl JobStep {
    fn done(index: usize, value: Option<f32>) -> Self {
        JobStep::Done {
            index: index as u32,
            value,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JobDrive {
    KillDet,
    AwaitRoll,
    Worlds,
    Done,
}

pub struct H0Job {
    state: State,
    legal: Vec<Action>,
    drive: JobDrive,
    owned: Option<H0>,
    seed: u64,
    roots: Vec<State>,
    cand: Vec<usize>,
    subset: Vec<Action>,
    me: PlayerId,
    k: u32,
    fuse_dropped: u32,
    root_world_seeds: Vec<u64>,
    wbase_used: Option<u64>,
    wbase_reused: bool,
    deal_seed: Option<u64>,
    nodes_after_lethal: u32,
    lethal_spent: Option<u32>,
    world_budgets: Vec<u32>,
    dec_stats: SearchStats,
    done_index: Option<usize>,
    done_value: Option<f32>,
    det_kill_attempted: bool,
    pending_kill_roll: Option<KillRollOutcome>,
}

fn parse_h0_spec(spec: &str) -> Result<H0, String> {
    match AnyPolicy::parse_spec(spec)? {
        AnyPolicy::H0(h) => Ok(h),
        _ => Err(format!("h0 job requires an h0 policy spec, got '{spec}'")),
    }
}

fn job_needs_kill(h: &H0, state: &State) -> bool {
    h.tkill > 0
        && matches!(
            state.phase,
            Phase::Main | Phase::Combat | Phase::Choice { .. }
        )
}

fn prepare_roots(
    h: &mut H0,
    db: &CardDb,
    state: &State,
    legal: &[Action],
    rng: &mut Xoshiro256ss,
) -> Result<PreparedRoots, JobStep> {
    let me = acting_player(state);
    let cand: Vec<usize> = legal
        .iter()
        .enumerate()
        .filter(|(_, a)| useful_action(state, me, a, h.bpp1, h.bpp2))
        .map(|(i, _)| i)
        .collect();
    let (cand, fuse_dropped) = filter_cand_fuseguard(db, state, me, h.fuseguard, legal, cand);
    h.stats.candidates += cand.len() as u64;
    if cand.is_empty() {
        if h.explain_armed {
            let mut rec =
                ExplainRecord::new(ChoosePath::SingleLegal, 0, h.node_cap, h.alloc_label());
            rec.chosen_index = 0;
            rec.tie_set = vec![0];
            rec.fuse_dropped = fuse_dropped;
            h.stamp_explain_deal(&mut rec, None);
            h.explain = Some(rec);
        }
        return Err(JobStep::done(0, None));
    }
    if cand.len() == 1 {
        if h.explain_armed {
            let mut rec =
                ExplainRecord::new(ChoosePath::SingleLegal, 0, h.node_cap, h.alloc_label());
            rec.chosen_index = cand[0];
            rec.tie_set = vec![cand[0]];
            rec.fuse_dropped = fuse_dropped;
            h.stamp_explain_deal(&mut rec, None);
            h.explain = Some(rec);
        }
        return Err(JobStep::done(cand[0], None));
    }
    let subset: Vec<Action> = cand.iter().map(|&i| legal[i].clone()).collect();
    let k = h.k();
    let (world_seeds, wbase_used, wbase_reused) = h.root_world_seeds(state, me, k, rng);
    let _ = wbase_reused;
    let root_world_seeds = world_seeds.clone();
    let block = h.deal == Deal::Block && h.hread.is_none();
    let deal_seed = if block {
        if h.turn_stable_worlds() {
            let base = wbase_used.expect("turn-stable worlds require a base");
            let mut seed_rng = Xoshiro256ss::from_seed(base);
            for _ in 0..k {
                seed_rng.next_u64();
            }
            Some(seed_rng.next_u64())
        } else {
            Some(rng.next_u64())
        }
    } else {
        None
    };
    let mut roots = Vec::with_capacity(k as usize);
    for (world, seed) in world_seeds.into_iter().enumerate() {
        if h.info == Info::Open {
            let mut open = OpenStats::default();
            roots.push(determinize_block(
                state,
                me,
                seed,
                deal_seed.unwrap_or(0),
                world as u32,
                h.info,
                Some(&mut open),
                block,
                h.hread_deal(),
            ));
            h.stats.open_hidden += u64::from(open.hidden);
            h.stats.open_hosts += u64::from(open.hosts);
        } else {
            roots.push(determinize_block(
                state,
                me,
                seed,
                deal_seed.unwrap_or(0),
                world as u32,
                h.info,
                None,
                block,
                h.hread_deal(),
            ));
        }
    }
    h.stats.roots += u64::from(k);
    Ok(PreparedRoots {
        me,
        cand,
        subset,
        roots,
        k,
        fuse_dropped,
        root_world_seeds,
        wbase_used,
        wbase_reused,
        deal_seed,
    })
}

type PostKillShortcut = (usize, Option<f32>, ChoosePath, u32, Option<u32>);

/// After kill solvers: `one_ply` or `consensus_lethal`, matching [`run_h0_decision_mode`].
fn post_kill_shortcut(
    h: &mut H0,
    db: &CardDb,
    state: &State,
    prep: &PreparedRoots,
) -> Option<PostKillShortcut> {
    let mut nodes = 0u32;
    if h.depth <= 2 {
        let root_vocab = h.root_vocab(state);
        let eval = h.evaluator(db, &root_vocab);
        let (j, v) = one_ply(
            &prep.roots,
            db,
            &prep.subset,
            prep.me,
            &mut nodes,
            h.node_cap,
            eval,
            h.pess,
            None,
        );
        return Some((prep.cand[j], Some(v), ChoosePath::OnePly, nodes, None));
    }
    let nodes_before_lethal = nodes;
    let lethal_cap = h.lethal_cap(nodes);
    if let Some(j) = consensus_lethal(
        db,
        &prep.roots,
        &prep.subset,
        prep.me,
        2,
        &mut nodes,
        lethal_cap,
    ) {
        let spent = nodes - nodes_before_lethal;
        return Some((
            prep.cand[j],
            Some(h.wv),
            ChoosePath::ConsensusLethal,
            nodes,
            Some(spent),
        ));
    }
    None
}

fn job_explain_shortcut(
    h: &mut H0,
    prep: &PreparedRoots,
    path: ChoosePath,
    chosen: usize,
    nodes: u32,
    lethal_spent: Option<u32>,
) {
    if !h.explain_armed {
        return;
    }
    let mut rec = ExplainRecord::new(path, prep.k, h.node_cap, h.alloc_label());
    rec.wbase = prep.wbase_used;
    rec.wbase_reused = prep.wbase_reused;
    rec.fuse_dropped = prep.fuse_dropped;
    h.stamp_explain_deal(&mut rec, prep.deal_seed);
    rec.chosen_index = chosen;
    rec.tie_set = vec![chosen];
    rec.nodes = nodes;
    if let Some(spent) = lethal_spent {
        rec.nodes_lethal = spent;
    }
    h.explain = Some(rec);
}

fn nodes_after_consensus_lethal(h: &H0, db: &CardDb, prep: &PreparedRoots) -> (u32, Option<u32>) {
    let mut nodes = 0u32;
    let nodes_before = nodes;
    let lethal_cap = h.lethal_cap(nodes);
    let _ = consensus_lethal(
        db,
        &prep.roots,
        &prep.subset,
        prep.me,
        2,
        &mut nodes,
        lethal_cap,
    );
    (nodes, Some(nodes - nodes_before))
}

fn apply_decision_tail(
    h: &mut H0,
    nodes: u32,
    lethal_spent: Option<u32>,
    dec_stats: &mut SearchStats,
    explain_rec: Option<ExplainRecord>,
) {
    h.stats.nodes += u64::from(nodes);
    if nodes >= h.node_cap {
        h.stats.cap_hits += 1;
    }
    if let Some(spent) = lethal_spent {
        h.stats.lethal_nodes += u64::from(spent);
    }
    h.stats.accum(dec_stats);
    if let Some(mut rec) = explain_rec {
        rec.nodes = nodes;
        rec.horizon_nodes = dec_stats.horizon_nodes as u32;
        h.explain = Some(rec);
    }
}

pub fn kill_det(
    db: &CardDb,
    state: &State,
    spec: &str,
    seed: u64,
) -> Result<KillDetOutcome, String> {
    let mut h = parse_h0_spec(spec)?;
    let legal = legal_actions(db, state);
    let mut rng = policy_rng(seed);
    let prep = match prepare_roots(&mut h, db, state, &legal, &mut rng) {
        Ok(p) => p,
        Err(JobStep::Done { .. }) => {
            return Ok(KillDetOutcome {
                taken_index: None,
                nodes: 0,
                stats: SearchStats::default(),
            });
        }
        Err(_) => return Err("unexpected early job step".into()),
    };
    let mut stats = SearchStats::default();
    let (taken, nodes) = kill_det_on_roots(
        db,
        &prep.roots,
        &legal,
        &prep.cand,
        prep.me,
        h.tkill,
        &mut stats,
    );
    Ok(KillDetOutcome {
        taken_index: taken.map(|i| i as u32),
        nodes,
        stats,
    })
}

pub fn kill_roll(
    db: &CardDb,
    state: &State,
    spec: &str,
    seed: u64,
) -> Result<KillRollOutcome, String> {
    let mut h = parse_h0_spec(spec)?;
    let legal = legal_actions(db, state);
    let mut rng = policy_rng(seed);
    let prep = match prepare_roots(&mut h, db, state, &legal, &mut rng) {
        Ok(p) => p,
        Err(_) => {
            return Ok(KillRollOutcome {
                taken_index: None,
                nodes: 0,
                stats: SearchStats::default(),
            });
        }
    };
    let mut stats = SearchStats::default();
    let (taken, nodes) = kill_roll_on_roots(
        db,
        &prep.roots,
        &legal,
        &prep.cand,
        prep.me,
        h.tkill,
        h.tkroll,
        &mut stats,
        None,
    );
    Ok(KillRollOutcome {
        taken_index: taken.map(|i| i as u32),
        nodes,
        stats,
    })
}

pub fn world(
    db: &CardDb,
    state: &State,
    spec: &str,
    seed: u64,
    r: usize,
    budget: u32,
) -> Result<WorldOutcome, String> {
    let mut h = parse_h0_spec(spec)?;
    let legal = legal_actions(db, state);
    let mut rng = policy_rng(seed);
    let prep = prepare_roots(&mut h, db, state, &legal, &mut rng).map_err(|e| {
        if let JobStep::Done { index, .. } = e {
            format!("early decision index {index}")
        } else {
            "early decision".into()
        }
    })?;
    if r >= prep.roots.len() {
        return Err(format!("world {r} out of range (k={})", prep.roots.len()));
    }
    let root_vocab = h.root_vocab(state);
    let eval = h.evaluator(db, &root_vocab);
    let wr = search_world(
        db,
        &prep.roots[r],
        r,
        budget,
        &prep.subset,
        prep.me,
        h.depth,
        h.beam,
        h.tt,
        h.info,
        eval,
        h.odepth,
        h.obeam,
        h.fusemacro,
        h.horizon,
        h.hres,
        h.explain_armed,
    );
    Ok(WorldOutcome::from_wr(r as u32, &wr))
}

fn world_budgets_for(h: &H0, nodes_after_lethal: u32) -> Vec<u32> {
    let leftover = h.node_cap.saturating_sub(nodes_after_lethal);
    let k = h.k();
    let base = leftover / k;
    let extra = leftover % k;
    (0..k)
        .map(|r| base + if r < extra { 1 } else { 0 })
        .collect()
}

impl H0Job {
    fn prepared_roots(&self) -> PreparedRoots {
        PreparedRoots {
            me: self.me,
            cand: self.cand.clone(),
            subset: self.subset.clone(),
            roots: self.roots.clone(),
            k: self.k,
            fuse_dropped: self.fuse_dropped,
            root_world_seeds: self.root_world_seeds.clone(),
            wbase_used: self.wbase_used,
            wbase_reused: self.wbase_reused,
            deal_seed: self.deal_seed,
        }
    }

    fn refresh_consensus_measurement(&mut self, db: &CardDb) {
        let h = self.owned.as_ref().expect("consensus measurement needs h0");
        if h.depth <= 2 {
            self.nodes_after_lethal = 0;
            self.lethal_spent = None;
            return;
        }
        let prep = self.prepared_roots();
        let (nodes, spent) = nodes_after_consensus_lethal(h, db, &prep);
        self.nodes_after_lethal = nodes;
        self.lethal_spent = spent;
    }

    fn try_shortcut_after_kill(&mut self, db: &CardDb) -> Option<JobStep> {
        let prep = self.prepared_roots();
        let h = self.owned.as_mut().expect("shortcut needs h0");
        let (idx, lv, path, nodes, lethal_spent) = post_kill_shortcut(h, db, &self.state, &prep)?;
        h.last_value = lv;
        job_explain_shortcut(h, &prep, path, idx, nodes, lethal_spent);
        apply_decision_tail(h, nodes, lethal_spent, &mut self.dec_stats, None);
        self.done_index = Some(idx);
        self.done_value = lv;
        self.drive = JobDrive::Done;
        Some(JobStep::done(idx, lv))
    }

    fn step_after_kill(&mut self, db: &CardDb) -> Result<JobStep, String> {
        if let Some(step) = self.try_shortcut_after_kill(db) {
            return Ok(step);
        }
        self.refresh_consensus_measurement(db);
        let alloc_world = self
            .owned
            .as_ref()
            .map(|h| h.alloc == Alloc::World && h.depth > 2)
            .unwrap_or(false);
        if alloc_world {
            let h = self.owned.as_ref().unwrap();
            self.world_budgets = world_budgets_for(h, self.nodes_after_lethal);
            self.drive = JobDrive::Worlds;
            return Ok(JobStep::Worlds {
                budgets: self.world_budgets.clone(),
            });
        }
        self.schedule_after_kill(db)
    }

    pub fn choose_stats(&self) -> SearchStats {
        self.owned
            .as_ref()
            .map(|h| h.stats.clone())
            .unwrap_or_else(|| self.dec_stats.clone())
    }

    pub fn start(
        db: &CardDb,
        state: &State,
        spec: &str,
        seed: u64,
    ) -> Result<(H0Job, JobStep), String> {
        let mut h = parse_h0_spec(spec.trim())?;
        h.threads = 1;
        let legal = legal_actions(db, state);
        let mut rng = policy_rng(seed);
        h.last_value = None;
        h.stats.decisions += 1;
        if legal.len() <= 1 {
            if h.explain_armed {
                let mut rec =
                    ExplainRecord::new(ChoosePath::SingleLegal, 0, h.node_cap, h.alloc_label());
                rec.chosen_index = 0;
                rec.tie_set = vec![0];
                h.stamp_explain_deal(&mut rec, None);
                h.explain = Some(rec);
            }
            return Ok((
                H0Job::terminal(h, state, legal, seed, 0, None),
                JobStep::done(0, None),
            ));
        }
        if matches!(state.phase, Phase::Mulligan { .. }) {
            let idx = h.choose_mulligan(db, state, &legal, &mut rng);
            if h.explain_armed {
                let mut rec =
                    ExplainRecord::new(ChoosePath::Mulligan, 0, h.node_cap, h.alloc_label());
                rec.chosen_index = idx;
                rec.tie_set = vec![idx];
                rec.mull = h.take_pending_mull_explain();
                h.stamp_explain_deal(&mut rec, None);
                h.explain = Some(rec);
            }
            return Ok((
                H0Job::terminal(h, state, legal, seed, idx, None),
                JobStep::done(idx, None),
            ));
        }
        let needs_kill = job_needs_kill(&h, state);
        let prep = match prepare_roots(&mut h, db, state, &legal, &mut rng) {
            Ok(p) => p,
            Err(step) => {
                let JobStep::Done { index, value } = step else {
                    return Err("unexpected step".into());
                };
                return Ok((
                    H0Job::terminal(h, state, legal, seed, index as usize, value),
                    JobStep::done(index as usize, value),
                ));
            }
        };
        let kill_roll_first = h.tkroll > 0;
        let mut job = H0Job {
            state: state.clone(),
            legal,
            drive: if needs_kill {
                JobDrive::KillDet
            } else {
                JobDrive::Done
            },
            owned: Some(h),
            seed,
            roots: prep.roots,
            cand: prep.cand,
            subset: prep.subset,
            me: prep.me,
            k: prep.k,
            fuse_dropped: prep.fuse_dropped,
            root_world_seeds: prep.root_world_seeds,
            wbase_used: prep.wbase_used,
            wbase_reused: prep.wbase_reused,
            deal_seed: prep.deal_seed,
            nodes_after_lethal: 0,
            lethal_spent: None,
            world_budgets: Vec::new(),
            dec_stats: SearchStats::default(),
            done_index: None,
            done_value: None,
            det_kill_attempted: false,
            pending_kill_roll: None,
        };
        let step = if needs_kill {
            JobStep::Kill {
                roll: kill_roll_first,
            }
        } else {
            job.step_after_kill(db)?
        };
        Ok((job, step))
    }

    fn terminal(
        mut h: H0,
        state: &State,
        legal: Vec<Action>,
        seed: u64,
        index: usize,
        value: Option<f32>,
    ) -> H0Job {
        h.last_value = value;
        H0Job {
            state: state.clone(),
            legal,
            drive: JobDrive::Done,
            owned: Some(h),
            seed,
            roots: Vec::new(),
            cand: Vec::new(),
            subset: Vec::new(),
            me: acting_player(state),
            k: 0,
            fuse_dropped: 0u32,
            root_world_seeds: Vec::new(),
            wbase_used: None,
            wbase_reused: false,
            deal_seed: None,
            nodes_after_lethal: 0,
            lethal_spent: None,
            world_budgets: Vec::new(),
            dec_stats: SearchStats::default(),
            done_index: Some(index),
            done_value: value,
            det_kill_attempted: false,
            pending_kill_roll: None,
        }
    }

    pub fn stats(&self) -> &SearchStats {
        &self.dec_stats
    }

    fn finish_kill_taken(&mut self, idx: u32) -> JobStep {
        let wv = self.owned.as_ref().map(|h| h.wv).unwrap_or(0.0);
        if let Some(h) = self.owned.as_mut() {
            h.last_value = Some(wv);
            h.stats.accum(&self.dec_stats);
        }
        self.done_index = Some(idx as usize);
        self.done_value = Some(wv);
        self.drive = JobDrive::Done;
        JobStep::done(idx as usize, Some(wv))
    }

    pub fn provide_kill_det(
        &mut self,
        db: &CardDb,
        outcome: KillDetOutcome,
    ) -> Result<JobStep, String> {
        if self.owned.is_none() {
            return Err("provide_kill_det requires start() job".into());
        }
        self.det_kill_attempted = true;
        self.dec_stats.accum(&outcome.stats);
        if let Some(idx) = outcome.taken_index {
            return Ok(self.finish_kill_taken(idx));
        }
        if let Some(roll) = self.pending_kill_roll.take() {
            return self.apply_kill_roll(db, roll);
        }
        let (tkroll, needs_kill) = {
            let h = self.owned.as_ref().unwrap();
            (h.tkroll, job_needs_kill(h, &self.state))
        };
        if tkroll > 0 && needs_kill {
            self.drive = JobDrive::AwaitRoll;
            return Ok(JobStep::AwaitRoll);
        }
        self.step_after_kill(db)
    }

    fn apply_kill_roll(
        &mut self,
        db: &CardDb,
        outcome: KillRollOutcome,
    ) -> Result<JobStep, String> {
        self.dec_stats.accum(&outcome.stats);
        if let Some(idx) = outcome.taken_index {
            return Ok(self.finish_kill_taken(idx));
        }
        let (tkill, needs_kill) = {
            let h = self.owned.as_ref().unwrap();
            (h.tkill, job_needs_kill(h, &self.state))
        };
        if !self.det_kill_attempted && tkill > 0 && needs_kill {
            return Ok(JobStep::Kill { roll: false });
        }
        self.step_after_kill(db)
    }

    pub fn provide_kill_roll(
        &mut self,
        db: &CardDb,
        outcome: KillRollOutcome,
    ) -> Result<JobStep, String> {
        if self.owned.is_none() {
            return Err("provide_kill_roll requires start() job".into());
        }
        if !self.det_kill_attempted && self.drive == JobDrive::KillDet {
            self.pending_kill_roll = Some(outcome);
            return Ok(JobStep::Kill { roll: false });
        }
        self.apply_kill_roll(db, outcome)
    }

    fn schedule_after_kill(&mut self, db: &CardDb) -> Result<JobStep, String> {
        let prep = self.prepared_roots();
        let h = self.owned.as_mut().unwrap();
        let mut rng = policy_rng(self.seed);
        let idx = run_h0_decision_mode(
            h,
            db,
            &self.state,
            &self.legal,
            &mut rng,
            JobExecMode::ViaJobPostKill,
            Some(&prep),
        );
        let lv = h.last_value;
        h.stats.accum(&self.dec_stats);
        self.dec_stats = SearchStats::default();
        self.done_index = Some(idx);
        self.done_value = lv;
        self.drive = JobDrive::Done;
        Ok(JobStep::done(idx, lv))
    }

    pub fn finish(&mut self, db: &CardDb, worlds: &[WorldOutcome]) -> Result<JobStep, String> {
        if worlds.len() != self.k as usize {
            return Err(format!(
                "expected {} world outcomes, got {}",
                self.k,
                worlds.len()
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for w in worlds {
            if w.r >= self.k {
                return Err(format!("world r={} out of range (k={})", w.r, self.k));
            }
            if !seen.insert(w.r) {
                return Err(format!("duplicate world r={}", w.r));
            }
        }
        for r in 0..self.k {
            if !seen.contains(&r) {
                return Err(format!("missing world r={}", r));
            }
        }
        let h = self.owned.as_mut().ok_or("finish requires start() job")?;
        let recording = h.explain_armed;
        let subset_len = self.subset.len();
        let mut acc = vec![0.0f32; subset_len];
        let mut n = vec![0u32; subset_len];
        let mut worst = vec![f32::INFINITY; subset_len];
        let mut explain_cands: Vec<CandidateRecord> = if recording {
            self.subset
                .iter()
                .enumerate()
                .map(|(j, a)| CandidateRecord {
                    legal_index: self.cand[j],
                    action: crate::action::to_neutral(&self.state, a),
                    worlds: Vec::new(),
                    root_agg: 0.0,
                    worst: f32::INFINITY,
                    n: 0,
                })
                .collect()
        } else {
            Vec::new()
        };
        let mut explain_rec = if recording {
            let mut rec =
                ExplainRecord::new(ChoosePath::Search, self.k, h.node_cap, h.alloc_label());
            rec.wbase = self.wbase_used;
            rec.wbase_reused = self.wbase_reused;
            rec.fuse_dropped = self.fuse_dropped;
            h.stamp_explain_deal(&mut rec, self.deal_seed);
            rec.path = ChoosePath::Search;
            rec.nodes_lethal = self.lethal_spent.unwrap_or(0);
            Some(rec)
        } else {
            None
        };
        let world_results: Vec<WorldSearchResult> = (0..self.k)
            .map(|r| {
                worlds
                    .iter()
                    .find(|w| w.r == r)
                    .expect("missing world")
                    .to_search_result()
            })
            .collect();
        let mut attempted = 0u64;
        let world_nodes_sum = merge_world_results(
            &world_results,
            subset_len,
            &mut acc,
            &mut n,
            &mut worst,
            &mut self.dec_stats,
            &mut explain_cands,
            recording,
            &mut attempted,
        );
        let nodes = self.nodes_after_lethal + world_nodes_sum;
        let total_pairs = u64::from(self.k) * subset_len as u64;
        let pairs_skipped_add = if self.nodes_after_lethal < h.node_cap {
            total_pairs - attempted
        } else {
            0
        };
        let odepth = h.odepth;
        let obeam = h.obeam;
        let idx = pick_after_world_merge(
            h,
            db,
            &self.state,
            &self.roots,
            &self.subset,
            &self.cand,
            self.me,
            odepth,
            obeam,
            self.k,
            self.wbase_used,
            &self.root_world_seeds,
            &acc,
            &n,
            &worst,
            pairs_skipped_add,
            recording,
            &mut explain_rec,
            &mut explain_cands,
            &mut self.dec_stats,
        );
        apply_decision_tail(
            h,
            nodes,
            self.lethal_spent,
            &mut self.dec_stats,
            explain_rec,
        );
        self.done_index = Some(idx);
        self.done_value = h.last_value;
        self.drive = JobDrive::Done;
        Ok(JobStep::done(idx, h.last_value))
    }
}
