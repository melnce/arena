//! H0: depth-limited beam search. The default leaf is the built-in
//! `h0-linear-v4` value net; `value=v0` is the historical hand-written
//! arithmetic. [`H0::fast`] keeps `value=v0` and `tt=0`.
//!
//! Search starts from `K = max(1, determinizations)` roots produced by
//! `determinize_with(state, me, seed, info)` (which reseeds the game RNG).
//! Under [`Info::All`] every root clones the true hidden state (opponent
//! hand and both decks' contents) and only the RNG seed differs, so K
//! roots average over future draws and random effects. Own-turn search,
//! lethal, and the opponent reply all run on those roots — the live game
//! RNG is never read directly (except under `info=all`, where the opponent
//! hand *is* the true hand). A lethal is taken only when every root agrees. The
//! node cap is global. `encode` still masks the opponent's hand at the
//! leaf; `info` is a search-time knob only.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::action::{acting_player, to_neutral, Action};
use crate::apply::{apply, legal_actions};
use crate::card::{CardId, CardKind};
use crate::db::CardDb;
use crate::determinize::{determinize_block, HreadDeal};

pub use crate::determinize::Info;
use crate::encode::{encode_with_vocab, vocab, EncodingVersion};
use crate::ids::{AttackTarget, PlayerId, Slot};
#[cfg(not(target_arch = "wasm32"))]
use crate::lethal::forced_lethal_accepting_cancel;
use crate::lethal::{
    confirm_det_lethal_line, confirm_lethal_line_rerolled, forced_lethal, forced_lethal_accepting,
    forced_lethal_det, roll_confirm_seed, LethalVerdict,
};
use crate::limits::MAX_TURNS;
use crate::rng::Xoshiro256ss;
use crate::search_key::{search_key, search_key_id};
use crate::state::FIELD_SIZE;
use crate::state::{ChoiceNode, Phase, PlayerState, State};
use crate::trace::{ChooseOptionJson, NeutralAction};

use super::explain::{
    ChoosePath, ExplainRecord, HoldbackAttackRecord, HoldbackBranchWorldRecord, HoldbackRecord,
    HoldbackResampleCandidate, HoldbackResampleRecord, Line, PvEnd, PvTracker, WorldRecord,
};
use super::fuse_guard::fuse_action_is_noop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HorizonCutoff {
    Depth,
    Cap,
}
use super::mulligan::{deck_fingerprint_player, mulligan_seat, MulliganTable};
use super::needs::NeedsTable;
use super::net::ValueNet;
use super::Policy;

/// Opening mulligan mode for [`H0`]. Default [`MullMode::Table`] uses the
/// built-in `mulligan-v3` keep table; [`MullMode::Rule`] is cost ≥ 4 send back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MullMode {
    Rule,
    /// One `next_u64()` from the rng passed to [`Policy::choose`]; slot `i` is
    /// sent back iff bit `i` is set (`i < hand length`, at most 4). Deterministic
    /// for a seed.
    Random,
    #[default]
    Table,
}

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
/// built-in `h0-linear-v4` model). [`ValueVersion::V0`] is the
/// hand-written leaf the bot used before the net became the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValueVersion {
    V0,
    V1,
    /// Learned leaf. Search, determinization, and the opponent model
    /// are unchanged; only [`Evaluator::value`] dispatches. Without
    /// `net=<name|path>` this is [`builtin_net`].
    #[default]
    Net,
}

const BUILTIN_NET_JSON: &str = include_str!("../../models/h0-linear-v4.json");
/// Committed name of the built-in value model (`engine/models/h0-linear-v4.json`).
pub const BUILTIN_NET_NAME: &str = "h0-linear-v4";
static BUILTIN_NET: OnceLock<Arc<ValueNet>> = OnceLock::new();

const BUILTIN_MULLIGAN_JSON: &str = include_str!("../../models/mulligan-v3.json");
/// Committed name of the built-in mulligan table (`engine/models/mulligan-v3.json`).
pub const BUILTIN_MULLIGAN_NAME: &str = "mulligan-v3";
static BUILTIN_MULLIGAN: OnceLock<Arc<MulliganTable>> = OnceLock::new();

/// The built-in `h0-linear-v4` model, parsed once. A parse failure is a
/// build defect.
pub fn builtin_net() -> Arc<ValueNet> {
    BUILTIN_NET
        .get_or_init(|| {
            ValueNet::from_json_named(BUILTIN_NET_NAME, BUILTIN_NET_JSON)
                .expect("built-in value model parses")
        })
        .clone()
}

const BUILTIN_VALUE_V64_JSON: &str = include_str!("../../models/h0-value-v64.json");
/// Committed name of the served-bot residual value leaf (`engine/models/h0-value-v64.json`).
pub const BUILTIN_VALUE_V64_NAME: &str = "h0-value-v64";
static BUILTIN_VALUE_V64: OnceLock<Arc<ValueNet>> = OnceLock::new();

/// The built-in `h0-value-v64` residual leaf, parsed once. A parse failure is a
/// build defect.
pub fn builtin_value_v64() -> Arc<ValueNet> {
    BUILTIN_VALUE_V64
        .get_or_init(|| {
            ValueNet::from_json_named(BUILTIN_VALUE_V64_NAME, BUILTIN_VALUE_V64_JSON)
                .expect("built-in h0-value-v64 parses")
        })
        .clone()
}

const BUILTIN_VALUE_V65_JSON: &str = include_str!("../../models/h0-value-v65.json");
/// Committed name of the served-bot residual value leaf (`engine/models/h0-value-v65.json`).
pub const BUILTIN_VALUE_V65_NAME: &str = "h0-value-v65";
static BUILTIN_VALUE_V65: OnceLock<Arc<ValueNet>> = OnceLock::new();

/// The built-in `h0-value-v65` residual leaf, parsed once. A parse failure is a
/// build defect.
pub fn builtin_value_v65() -> Arc<ValueNet> {
    BUILTIN_VALUE_V65
        .get_or_init(|| {
            ValueNet::from_json_named(BUILTIN_VALUE_V65_NAME, BUILTIN_VALUE_V65_JSON)
                .expect("built-in h0-value-v65 parses")
        })
        .clone()
}

const BUILTIN_VALUE_V66_JSON: &str = include_str!("../../models/h0-value-v66.json");
/// Committed name of the served-bot residual value leaf (`engine/models/h0-value-v66.json`).
pub const BUILTIN_VALUE_V66_NAME: &str = "h0-value-v66";
static BUILTIN_VALUE_V66: OnceLock<Arc<ValueNet>> = OnceLock::new();

/// The built-in `h0-value-v66` residual leaf, parsed once. A parse failure is a
/// build defect.
pub fn builtin_value_v66() -> Arc<ValueNet> {
    BUILTIN_VALUE_V66
        .get_or_init(|| {
            ValueNet::from_json_named(BUILTIN_VALUE_V66_NAME, BUILTIN_VALUE_V66_JSON)
                .expect("built-in h0-value-v66 parses")
        })
        .clone()
}

const BUILTIN_VALUE_V69_JSON: &str = include_str!("../../models/h0-value-v69.json");
/// Committed name of the served-bot residual value leaf (`engine/models/h0-value-v69.json`).
pub const BUILTIN_VALUE_V69_NAME: &str = "h0-value-v69";
static BUILTIN_VALUE_V69: OnceLock<Arc<ValueNet>> = OnceLock::new();

/// The built-in `h0-value-v69` residual leaf, parsed once. A parse failure is a
/// build defect.
pub fn builtin_value_v69() -> Arc<ValueNet> {
    BUILTIN_VALUE_V69
        .get_or_init(|| {
            ValueNet::from_json_named(BUILTIN_VALUE_V69_NAME, BUILTIN_VALUE_V69_JSON)
                .expect("built-in h0-value-v69 parses")
        })
        .clone()
}

/// Resolve a built-in value leaf by committed name (`builtin`, `h0-linear-v4`, `h0-value-v64`, `h0-value-v65`, `h0-value-v66`, `h0-value-v69`).
pub fn builtin_net_by_name(name: &str) -> Option<Arc<ValueNet>> {
    match name {
        "builtin" | BUILTIN_NET_NAME => Some(builtin_net()),
        BUILTIN_VALUE_V64_NAME => Some(builtin_value_v64()),
        BUILTIN_VALUE_V65_NAME => Some(builtin_value_v65()),
        BUILTIN_VALUE_V66_NAME => Some(builtin_value_v66()),
        BUILTIN_VALUE_V69_NAME => Some(builtin_value_v69()),
        _ => None,
    }
}

const BUILTIN_ORDER_JSON: &str = include_str!("../../models/h0-order-v53.json");
/// Committed name of the built-in move-ordering model (`engine/models/h0-order-v53.json`).
pub const BUILTIN_ORDER_NAME: &str = "h0-order-v53";
static BUILTIN_ORDER: OnceLock<Arc<ValueNet>> = OnceLock::new();

/// The built-in `h0-order-v53` move-ordering model, parsed once. A parse failure is a
/// build defect.
pub fn builtin_order() -> Arc<ValueNet> {
    BUILTIN_ORDER
        .get_or_init(|| {
            ValueNet::from_json_named(BUILTIN_ORDER_NAME, BUILTIN_ORDER_JSON)
                .expect("built-in order model parses")
        })
        .clone()
}

const BUILTIN_ORDER_V59_JSON: &str = include_str!("../../models/h0-order-v59.json");
/// Committed name of the served bot's own-turn ordering model (`engine/models/h0-order-v59.json`).
pub const BUILTIN_ORDER_V59_NAME: &str = "h0-order-v59";
static BUILTIN_ORDER_V59: OnceLock<Arc<ValueNet>> = OnceLock::new();

/// The built-in `h0-order-v59` move-ordering model, parsed once. A parse failure is a
/// build defect.
pub fn builtin_order_v59() -> Arc<ValueNet> {
    BUILTIN_ORDER_V59
        .get_or_init(|| {
            ValueNet::from_json_named(BUILTIN_ORDER_V59_NAME, BUILTIN_ORDER_V59_JSON)
                .expect("built-in order v59 model parses")
        })
        .clone()
}

/// Resolve a built-in move-ordering model by committed name (`builtin` is `h0-order-v53`).
pub fn builtin_order_by_name(name: &str) -> Option<Arc<ValueNet>> {
    match name {
        "builtin" | BUILTIN_ORDER_NAME => Some(builtin_order()),
        BUILTIN_ORDER_V59_NAME => Some(builtin_order_v59()),
        _ => None,
    }
}

/// The built-in `mulligan-v3` keep table, parsed once. A parse failure is a
/// build defect.
pub fn builtin_mulligan() -> Arc<MulliganTable> {
    BUILTIN_MULLIGAN
        .get_or_init(|| {
            Arc::new(
                MulliganTable::from_json_named(BUILTIN_MULLIGAN_NAME, BUILTIN_MULLIGAN_JSON)
                    .expect("built-in mulligan table parses"),
            )
        })
        .clone()
}

/// Leaf evaluator plus the root-level terminal stand-in (`wv`) and the
/// cheap opponent-model knobs (`olethal`, `oevo`, `osteps`). `wv` is carried
/// here (not as a sibling of `odepth`/`obeam`) so `finite`, `one_ply`, and
/// the opponent lethal short-circuit share one value.
#[derive(Clone, Copy)]
struct Evaluator<'a> {
    db: &'a CardDb,
    needs: &'a NeedsTable,
    version: ValueVersion,
    weights: &'a Weights,
    wv: f32,
    olethal: bool,
    oevo: bool,
    okill: u32,
    omacro: bool,
    olsolve: u32,
    osteps: u32,
    /// When `false`, search scores the leaf at the bot's turn end instead of
    /// running the opponent reply (lost-turn re-rank only).
    opp_reply: bool,
    net: Option<&'a ValueNet>,
    vocab: &'a [CardId],
    encoding: EncodingVersion,
    clip: f32,
    bpp1: u32,
    bpp2: u32,
    bppv: f32,
    fuseguard: bool,
    /// Bot-side move-ordering net (`order=`).
    order_net: Option<&'a ValueNet>,
    /// Opponent-model move-ordering net (`oorder=1` or `oorder=<model>`).
    opp_order_net: Option<&'a ValueNet>,
    oopen: u32,
    onext: u32,
    onextwin: u32,
    onextleaf: bool,
}

impl Evaluator<'_> {
    /// One-step value for move ordering (`order=` / `oorder=` when set; else leaf).
    fn pick_value(self, state: &State, val_for: PlayerId, search_me: PlayerId) -> f32 {
        let order_for_pick = if val_for == search_me {
            self.order_net
        } else {
            self.opp_order_net
        };
        if order_for_pick.is_none() {
            return self.value(state, val_for);
        }
        if matches!((order_for_pick, self.net), (Some(a), Some(b)) if std::ptr::eq(a, b)) {
            return self.value(state, val_for);
        }
        self.order_value(state, val_for, order_for_pick.expect("checked"))
    }

    /// Raw leaf value (unclipped). Matches main's `Evaluator::value` before bppv.
    fn leaf(self, state: &State, me: PlayerId) -> f32 {
        match self.version {
            ValueVersion::V0 => value(state, me),
            ValueVersion::V1 => value_v1(state, me, self.needs, self.weights),
            ValueVersion::Net => {
                let obs = encode_with_vocab(state, me, self.vocab, self.encoding, Some(self.db));
                let net = self.net.expect("value=net requires a loaded net");
                if self.clip > 0.0 {
                    net.value_clipped(&obs, self.clip)
                } else {
                    net.value(&obs)
                }
            }
        }
    }

    fn bpp_prior(self, state: &State, me: PlayerId) -> f32 {
        if self.bppv == 0.0 {
            return 0.0;
        }
        let opp = me.opponent();
        let me_own = state.active == me;
        let opp_own = state.active == opp;
        self.bppv
            * (state.player(me).usable_bonus_charges(me_own)
                - state.player(opp).usable_bonus_charges(opp_own)) as f32
    }

    /// Leaf value plus `bppv` prior (unclipped; `finite` applies at aggregation).
    fn value(self, state: &State, me: PlayerId) -> f32 {
        let v = self.leaf(state, me);
        if !v.is_finite() {
            return v;
        }
        v + self.bpp_prior(state, me)
    }

    fn order_leaf(self, state: &State, me: PlayerId, order_net: &ValueNet) -> f32 {
        let obs = encode_with_vocab(state, me, self.vocab, order_net.encoding, Some(self.db));
        if self.clip > 0.0 {
            order_net.value_clipped(&obs, self.clip)
        } else {
            order_net.value(&obs)
        }
    }

    /// Order net value plus `bppv` prior (unclipped). Without `order=`, matches [`value`].
    fn order_value(self, state: &State, me: PlayerId, order_net: &ValueNet) -> f32 {
        let v = self.order_leaf(state, me, order_net);
        if !v.is_finite() {
            return v;
        }
        v + self.bpp_prior(state, me)
    }
}

const INF: f32 = 1.0e9;
/// Default saturation bound (`wv`) on every accumulated value. 80 is
/// today's clamp; it sits inside the reachable live range of [`value_v0`].
const DEFAULT_WV: f32 = 80.0;
const DEFAULT_HRES: u32 = 200;

/// Per-decision search counters, accumulated across [`H0::choose`] calls.
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
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
    /// Most `apply`s charged in a single lethal sweep (okill budgets).
    pub opp_lethal_applies_max: u64,
    /// Sweeps that found lethal through the Ward-break prefix (`okill` bit 1).
    pub opp_lethal_ward_found: u64,
    /// Sweeps that found lethal through slot-freeing trades (`okill` bit 2).
    pub opp_lethal_slot_found: u64,
    /// Sweeps that found lethal through play-through (`okill` bit 4).
    pub opp_lethal_through_found: u64,
    /// Sweeps that found lethal through two plays (`okill` bit 8).
    pub opp_lethal_two_found: u64,
    /// Bounded [`forced_lethal`] calls after a sweep miss (`olsolve>0`,
    /// `olethal=1`, `odepth=0`).
    pub opp_solver_calls: u64,
    /// Solver calls that found a kill within budget.
    pub opp_solver_found: u64,
    /// Solver calls that returned [`LethalVerdict::Unknown`] (budget
    /// exhausted, not proof of absence).
    pub opp_solver_unknown: u64,
    /// `apply`s spent inside solver calls (charged to the node cap).
    pub opp_solver_nodes: u64,
    /// Root [`forced_lethal_det`] calls (`tkill>0`, own turn).
    pub own_solver_calls: u64,
    /// Solver calls that found a kill on the first determinization.
    pub own_solver_found: u64,
    /// Found kill confirmed on every root and played.
    pub own_solver_taken: u64,
    /// A root did not confirm, or `line[0]` was not legal.
    pub own_solver_rejected: u64,
    /// Solver calls that returned [`LethalVerdict::Unknown`].
    pub own_solver_unknown: u64,
    /// `apply`s spent inside own-turn solver calls (outside `node_cap`).
    pub own_solver_nodes: u64,
    /// Most `apply`s in a single own-turn solver call.
    pub own_solver_nodes_max: u64,
    /// Root [`forced_lethal_accepting`] calls after a deterministic miss
    /// (`tkill>0`, `tkroll>0`).
    pub own_roll_calls: u64,
    /// Roll solver calls that returned a line accepted on every root under
    /// rerolled dice.
    pub own_roll_found: u64,
    /// Roll kill confirmed on every root under rerolled dice and played.
    pub own_roll_taken: u64,
    /// Decisions where the roll check took nothing — no accepted line
    /// ([`LethalVerdict::None`] / [`LethalVerdict::Unknown`]), or the accepted
    /// line's `line[0]` was not a legal candidate.
    pub own_roll_rejected: u64,
    /// Candidate kill lines rejected by the reroll acceptance check during search.
    pub own_roll_lines_rejected: u64,
    /// Roll solver calls that returned [`LethalVerdict::Unknown`].
    pub own_roll_unknown: u64,
    /// `apply`s spent inside roll-confirmed solver calls (outside `node_cap`).
    pub own_roll_nodes: u64,
    /// Most `apply`s in a single roll-confirmed solver call.
    pub own_roll_nodes_max: u64,
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
    /// Uncharged `apply`s used to finish a fuse partner choice at a leaf
    /// (`fusemacro=1` only).
    pub fuse_overshoot: u64,
    /// Leaves scored through `horizon` finish-and-reply.
    pub horizon_leaves: u64,
    /// Reserve nodes spent on those leaves (not charged to the pair budget).
    pub horizon_nodes: u64,
    /// Mid-turn horizon finish could not end the turn — bare value used.
    pub horizon_fallback: u64,
    /// Root worlds skipped because `node_cap` bound before the pair ran.
    pub skipped_worlds: u64,
    /// Mulligans decided from a found deck entry (`mull=<table>`).
    pub mull_table: u64,
    /// Table mode, deck fingerprint not found — rule used for the whole hand.
    pub mull_fallback: u64,
    /// Under `info=open`, privately removed cards dealt into hand or deck.
    pub open_hidden: u64,
    /// Under `info=open`, revealed fuse hosts held fixed in the opponent hand.
    pub open_hosts: u64,
    /// Decisions where the held-back check ran (`hbcheck>0`, search chose
    /// `EndTurn` with a kill attack available).
    pub hb_checks: u64,
    /// Determinizations with a sure opponent removal line for a held-back
    /// attacker.
    pub hb_worlds_removable: u64,
    /// Kill attacks played instead of `EndTurn` after the held-back check.
    pub hb_overrides: u64,
    /// `apply`s spent inside held-back removal searches (outside `node_cap`).
    pub hb_nodes: u64,
    /// Most `apply`s in a single held-back removal search (one world).
    pub hb_nodes_max: u64,
    /// Worlds where the held-back budget ran out before any sure removal line.
    pub hb_unknown: u64,
    /// Decisions where the holdback resample check ran (`hbk>0`, search chose
    /// `EndTurn` with a kill attack available).
    pub hbk_checks: u64,
    /// Kill attacks played instead of `EndTurn` after the holdback resample check.
    pub hbk_flips: u64,
    /// `apply`s spent inside holdback resample searches (outside `node_cap`).
    pub hbk_nodes: u64,
    /// Opponent replies where opening search ran (`oopen>0`, `odepth=0`).
    pub opp_open_replies: u64,
    /// Opening lines completed and scored.
    pub opp_open_lines: u64,
    /// Replies whose value came from an opening line (strictly below greedy).
    pub opp_open_lower: u64,
    /// `apply`s spent in opening search (actions and greedy completions).
    pub opp_open_nodes: u64,
    /// Reply leaves that reached the post-bot-turn lethal glance (`onext>0`).
    pub opp_next_checks: u64,
    /// Leaves whose value was capped by `onext`.
    pub opp_next_found: u64,
    /// `apply`s spent in the bot-turn finish and next-turn lethal glance.
    pub opp_next_nodes: u64,
    /// Greedy next bot turns that won (`onextwin` / `onextleaf`, with `onext>0`).
    pub opp_next_own_wins: u64,
    /// Reply lines scored after the greedy next bot turn (`onextleaf`, with `onext>0`).
    pub opp_next_leaf: u64,
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
        self.opp_lethal_applies_max = self
            .opp_lethal_applies_max
            .max(other.opp_lethal_applies_max);
        self.opp_lethal_ward_found += other.opp_lethal_ward_found;
        self.opp_lethal_slot_found += other.opp_lethal_slot_found;
        self.opp_lethal_through_found += other.opp_lethal_through_found;
        self.opp_lethal_two_found += other.opp_lethal_two_found;
        self.opp_solver_calls += other.opp_solver_calls;
        self.opp_solver_found += other.opp_solver_found;
        self.opp_solver_unknown += other.opp_solver_unknown;
        self.opp_solver_nodes += other.opp_solver_nodes;
        self.own_solver_calls += other.own_solver_calls;
        self.own_solver_found += other.own_solver_found;
        self.own_solver_taken += other.own_solver_taken;
        self.own_solver_rejected += other.own_solver_rejected;
        self.own_solver_unknown += other.own_solver_unknown;
        self.own_solver_nodes += other.own_solver_nodes;
        self.own_solver_nodes_max = self.own_solver_nodes_max.max(other.own_solver_nodes_max);
        self.own_roll_calls += other.own_roll_calls;
        self.own_roll_found += other.own_roll_found;
        self.own_roll_taken += other.own_roll_taken;
        self.own_roll_rejected += other.own_roll_rejected;
        self.own_roll_lines_rejected += other.own_roll_lines_rejected;
        self.own_roll_unknown += other.own_roll_unknown;
        self.own_roll_nodes += other.own_roll_nodes;
        self.own_roll_nodes_max = self.own_roll_nodes_max.max(other.own_roll_nodes_max);
        self.chose_with_lethal_root += other.chose_with_lethal_root;
        self.cands_with_lethal_root += other.cands_with_lethal_root;
        self.tt_hits += other.tt_hits;
        self.tt_stores += other.tt_stores;
        self.pairs_skipped += other.pairs_skipped;
        self.lethal_nodes += other.lethal_nodes;
        self.unscored += other.unscored;
        self.fuse_overshoot += other.fuse_overshoot;
        self.horizon_leaves += other.horizon_leaves;
        self.horizon_nodes += other.horizon_nodes;
        self.horizon_fallback += other.horizon_fallback;
        self.skipped_worlds += other.skipped_worlds;
        self.mull_table += other.mull_table;
        self.mull_fallback += other.mull_fallback;
        self.open_hidden += other.open_hidden;
        self.open_hosts += other.open_hosts;
        self.hb_checks += other.hb_checks;
        self.hb_worlds_removable += other.hb_worlds_removable;
        self.hb_overrides += other.hb_overrides;
        self.hb_nodes += other.hb_nodes;
        self.hb_nodes_max = self.hb_nodes_max.max(other.hb_nodes_max);
        self.hb_unknown += other.hb_unknown;
        self.hbk_checks += other.hbk_checks;
        self.hbk_flips += other.hbk_flips;
        self.hbk_nodes += other.hbk_nodes;
        self.opp_open_replies += other.opp_open_replies;
        self.opp_open_lines += other.opp_open_lines;
        self.opp_open_lower += other.opp_open_lower;
        self.opp_open_nodes += other.opp_open_nodes;
        self.opp_next_checks += other.opp_next_checks;
        self.opp_next_found += other.opp_next_found;
        self.opp_next_nodes += other.opp_next_nodes;
        self.opp_next_own_wins += other.opp_next_own_wins;
        self.opp_next_leaf += other.opp_next_leaf;
    }
}

/// Turn-stable world seeding for [`H0`]. Default [`Wseed::Off`] draws one
/// seed per root from the caller rng; [`Wseed::Turn`] caches a per-turn base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Wseed {
    #[default]
    Off,
    Turn,
}

/// How opponent unknown hands are dealt across determinizations.
/// Default [`Deal::Indep`] shuffles independently per root; [`Deal::Block`]
/// deals consecutive blocks from one shared shuffle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Deal {
    #[default]
    Indep,
    Block,
}

#[derive(Debug, Clone)]
struct TurnWorldCache {
    turn: u32,
    active: PlayerId,
    me: PlayerId,
    base: u64,
    step_counter: u64,
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
    /// After consensus lethal, split leftover budget evenly across worlds;
    /// within each world, [`Alloc::Fair`]-style per-candidate shares.
    World,
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
    /// Worker threads for native parallel search (`1` = sequential).
    pub threads: u32,
    /// What the search is allowed to know. Default [`Info::Open`] deals the
    /// opponent only what the bot cannot rule out. `info=fair` restores the
    /// sweep-8b flip (own deck resampled, hand untouched; opponent
    /// resampled). `info=draws` restores the pre-flip path.
    pub info: Info,
    pub value: ValueVersion,
    pub weights: Weights,
    pub odepth: u32,
    pub obeam: usize,
    /// Opening actions explored on the greedy opponent reply (`odepth=0` only).
    pub oopen: u32,
    /// After a greedy-reply leaf, play the bot's turn greedily and glance for
    /// opponent lethal on the following turn (`odepth=0` only).
    pub onext: u32,
    /// With `onext>0`: a reply line whose greedy next bot turn wins scores at
    /// least this value (`0` = off).
    pub onextwin: u32,
    /// With `onext>0`: score the line after the bot's greedy next turn (minus
    /// the leaf from the opponent's side at his turn start) instead of at the
    /// bot's turn start.
    pub onextleaf: bool,
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
    /// Extra kill shapes in the opponent-lethal sweep (`okill` bitmask).
    /// Bit 1 = Ward break, 2 = slot-freeing, 4 = play-through. Default `0`.
    pub okill: u32,
    /// In the greedy opponent reply, credit a `Play` with an evolve on the
    /// just-played slot when that line scores higher. Default `false`.
    pub omacro: bool,
    /// After the opponent-lethal sweep misses, run [`forced_lethal`] with
    /// this node budget (charged to the pair cap). `0` = off (today).
    pub olsolve: u32,
    /// On each own-turn decision (Main / Combat / Choice), run
    /// [`forced_lethal_det`] on the first determinization before search.
    /// `0` = off (today).
    pub tkill: u32,
    /// After a deterministic miss, run [`forced_lethal`] on the first
    /// determinization and confirm the line under rerolled dice on every
    /// root. Only when `tkill>0`; `0` = off (today).
    pub tkroll: u32,
    /// After search chooses `EndTurn` with a kill attack available, re-score
    /// `EndTurn` per root with a bounded opponent removal search. Apply budget
    /// per root; all applies are outside `node_cap`. `0` = off (today).
    pub hbcheck: u32,
    /// After search chooses `EndTurn` with a kill attack available, re-score
    /// `EndTurn` and each kill on fresh determinizations before `hbcheck`.
    /// Applies are outside `node_cap`. `0` = off (today).
    pub hbk: u32,
    /// Greedy-line steps before a forced `EndTurn`. Default `6` is the sweep-5
    /// flip; the hard stop is `osteps + 3` (today: 9).
    pub osteps: u32,
    /// Treat a fuse as one search ply by expanding partner-choice completions
    /// (`fusemacro=1`). Default `true` is the sweep-10 flip; `fusemacro=0`
    /// restores step-by-step fuse search.
    pub fusemacro: bool,
    /// Learned leaf (`value=net`). `None` when the leaf is v0/v1.
    pub net: Option<Arc<ValueNet>>,
    /// Consensus-lethal node budget as a fraction of `node_cap`, in `(0, 1]`.
    /// Default `0.5` is the sweep-10 flip; `lcap=1` restores the whole cap.
    pub lcap: f32,
    /// Standardised-input clamp for the learned leaf; `0` is off. Default `5`
    /// is the sweep-10 flip. Ignored with `value=v0` / `value=v1`.
    pub clip: f32,
    /// Spec path compared by `h0_fields_eq` and printed by `spec()`.
    pub net_path: Option<String>,
    /// Move-ordering net (`order=`). Used only where search ranks one-step values.
    pub order: Option<Arc<ValueNet>>,
    /// Spec path compared by `h0_fields_eq` and printed by `spec()`.
    pub order_path: Option<String>,
    /// Opponent-model move-ordering net (`oorder=<model>`). `None` when only `oorder=0|1`.
    pub oorder_net: Option<Arc<ValueNet>>,
    /// Spec path compared by `h0_fields_eq` and printed by `spec()`.
    pub oorder_path: Option<String>,
    /// With `order=`, `1` also orders the opponent model's greedy picks (`oorder=1`).
    pub oorder: bool,
    /// Opening mulligan mode. Default [`MullMode::Table`] with [`builtin_mulligan`].
    pub mull: MullMode,
    pub mull_table: Option<Arc<MulliganTable>>,
    /// Table path compared by `h0_fields_eq` and printed by `spec()`.
    pub mull_path: Option<String>,
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
    /// Do not activate the early Bonus PP charge before this turn (1–6;
    /// default `1` = today; `6` = never).
    pub bpp1: u32,
    /// Do not activate the late Bonus PP charge before this turn (≥ 6;
    /// default `6` = today).
    pub bpp2: u32,
    /// Value prior: add `bppv` per usable Bonus PP charge for the evaluated
    /// player, subtract for the opponent (default `0` = today).
    pub bppv: f32,
    /// Score leaves through the opponent reply after the bot's turn ends.
    /// `0` = today; `1` = finished turns always answered; `2` = mid-turn
    /// cut-offs end the turn there; `3` = greedy finish before `EndTurn`.
    pub horizon: u32,
    /// Reserve node budget for horizon finish-and-reply (default `200`).
    pub hres: u32,
    /// Turn-stable world seeding. `off` = one seed per root (today); `turn` =
    /// one base per `(turn, active, me)` cached across decisions in the turn.
    pub wseed: Wseed,
    /// Explicit turn base (implies turn-stable worlds; skips the cache).
    pub wbase: Option<u64>,
    /// Lost-turn tie-break budget. `0` = off (today). When every scored
    /// candidate is a sure loss in every world, re-rank by the position at
    /// the end of the bot's own turn with the opponent reply off.
    pub lostrank: u32,
    /// Drop no-op fuses from the bot's own root candidates and own-turn search.
    pub fuseguard: bool,
    /// Opponent hand dealing across roots. Default [`Deal::Indep`].
    pub deal: Deal,
    /// Hand-reading deal weights for [`Info::Open`]. `None` = off (uniform).
    /// Tuple is `(eps_fa, eps_s, delta)`.
    pub hread: Option<(f32, f32, f32)>,
    /// SIR candidates per world when `hread` is on. Default `256`.
    pub hreadm: u32,
    /// Per-turn world base cache (`wseed=turn` only; not in `spec()`).
    turn_world_cache: Option<TurnWorldCache>,
}

impl Default for H0 {
    fn default() -> Self {
        Self {
            depth: 6,
            beam: 4,
            determinizations: 4,
            node_cap: 2000,
            alloc: Alloc::Fair,
            threads: 1,
            info: Info::Open,
            value: ValueVersion::Net,
            weights: Weights::default(),
            odepth: 0,
            obeam: 3,
            oopen: 0,
            onext: 0,
            onextwin: 0,
            onextleaf: false,
            wv: DEFAULT_WV,
            pess: 0.0,
            tt: true,
            olethal: true,
            oevo: true,
            okill: 0,
            omacro: false,
            olsolve: 0,
            tkill: 0,
            tkroll: 0,
            hbcheck: 0,
            hbk: 0,
            osteps: 6,
            fusemacro: true,
            net: Some(builtin_net()),
            lcap: 0.5,
            clip: 5.0,
            net_path: None,
            order: None,
            order_path: None,
            oorder_net: None,
            oorder_path: None,
            oorder: false,
            mull: MullMode::Table,
            mull_table: Some(builtin_mulligan()),
            mull_path: None,
            stats: SearchStats::default(),
            last_value: None,
            explain_armed: false,
            explain: None,
            bpp1: 1,
            bpp2: 6,
            bppv: 0.0,
            horizon: 0,
            hres: DEFAULT_HRES,
            wseed: Wseed::Off,
            wbase: None,
            lostrank: 0,
            fuseguard: false,
            deal: Deal::Indep,
            hread: None,
            hreadm: 256,
            turn_world_cache: None,
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
            // Fixed test baseline: lcap/clip/fusemacro are inert at depth 2
            // (one_ply never runs consensus_lethal or search_own; v0 ignores
            // clip) but pin the pre-flip defaults so fast() stays field-identical.
            lcap: 1.0,
            clip: 0.0,
            fusemacro: false,
            // Fixed test baseline: mulligan table is inert when tests use
            // opening_hands or non-mulligan positions; pin rule mulligan so
            // fast() stays field-identical.
            mull: MullMode::Rule,
            mull_table: None,
            mull_path: None,
            ..Self::default()
        }
    }

    fn k(&self) -> u32 {
        self.determinizations.max(1)
    }

    fn evaluator<'a>(&'a self, db: &'a CardDb, root_vocab: &'a [CardId]) -> Evaluator<'a> {
        Evaluator {
            db,
            needs: db.needs(),
            version: self.value,
            weights: &self.weights,
            oevo: self.oevo,
            okill: self.okill,
            omacro: self.omacro,
            olsolve: self.olsolve,
            osteps: self.osteps,
            opp_reply: true,
            net: self.net.as_deref(),
            vocab: root_vocab,
            encoding: self
                .net
                .as_deref()
                .map(|n| n.encoding)
                .unwrap_or(EncodingVersion::V1),
            clip: self.clip,
            wv: self.wv,
            olethal: self.olethal,
            bpp1: self.bpp1,
            bpp2: self.bpp2,
            bppv: self.bppv,
            fuseguard: self.fuseguard,
            order_net: match (self.order.as_deref(), self.net.as_deref()) {
                (Some(a), Some(b)) if std::ptr::eq(a, b) => None,
                (Some(a), _) => Some(a),
                _ => None,
            },
            opp_order_net: {
                let raw = self.oorder_net.as_deref().or(if self.oorder {
                    self.order.as_deref()
                } else {
                    None
                });
                match (raw, self.net.as_deref()) {
                    (Some(a), Some(b)) if std::ptr::eq(a, b) => None,
                    (Some(a), _) => Some(a),
                    _ => None,
                }
            },
            oopen: self.oopen,
            onext: self.onext,
            onextwin: self.onextwin,
            onextleaf: self.onextleaf,
        }
    }

    fn lethal_cap(&self, nodes: u32) -> u32 {
        let budget = (self.lcap * self.node_cap as f32).floor() as u32;
        nodes.saturating_add(budget).min(self.node_cap)
    }

    fn root_vocab(&self, state: &State) -> Vec<CardId> {
        if self.value == ValueVersion::Net || self.order.is_some() || self.oorder_net.is_some() {
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
            Alloc::World => "world",
        }
    }

    fn turn_stable_worlds(&self) -> bool {
        self.wseed == Wseed::Turn || self.wbase.is_some()
    }

    fn stamp_explain_deal(&self, rec: &mut ExplainRecord, deal_seed: Option<u64>) {
        rec.deal = match self.deal {
            Deal::Indep => "indep".to_string(),
            Deal::Block => "block".to_string(),
        };
        rec.deal_seed = deal_seed;
        rec.hread = self
            .hread
            .map(|(eps_fa, eps_s, delta)| super::explain::HreadExplain {
                eps_fa,
                eps_s,
                delta,
                m: self.hreadm,
            });
    }

    fn hread_deal(&self) -> Option<HreadDeal> {
        self.hread.map(|(eps_fa, eps_s, delta)| HreadDeal {
            eps_fa,
            eps_s,
            delta,
            m: self.hreadm,
        })
    }

    fn root_world_seeds(
        &mut self,
        state: &State,
        me: PlayerId,
        k: u32,
        rng: &mut Xoshiro256ss,
    ) -> (Vec<u64>, Option<u64>, bool) {
        if !self.turn_stable_worlds() {
            let seeds = (0..k).map(|_| rng.next_u64()).collect();
            return (seeds, None, false);
        }
        let (base, reused) = if let Some(base) = self.wbase {
            (base, false)
        } else {
            let cache_ok = self.turn_world_cache.as_ref().is_some_and(|c| {
                c.turn == state.turn
                    && c.active == state.active
                    && c.me == me
                    && state.step_counter >= c.step_counter
            });
            if cache_ok {
                (self.turn_world_cache.as_ref().unwrap().base, true)
            } else {
                let base = rng.next_u64();
                self.turn_world_cache = Some(TurnWorldCache {
                    turn: state.turn,
                    active: state.active,
                    me,
                    base,
                    step_counter: state.step_counter,
                });
                (base, false)
            }
        };
        let mut seed_rng = Xoshiro256ss::from_seed(base);
        let seeds = (0..k).map(|_| seed_rng.next_u64()).collect();
        (seeds, Some(base), reused)
    }

    /// Sample `n` opponent hands for the side to move's opponent under this
    /// spec's `info`, `deal`, and `hread` settings. Root seeds are drawn as
    /// [`Policy::choose`] would for `k = n`.
    pub fn sample_opponent_hands(&mut self, state: &State, seed: u64, n: u32) -> Vec<Vec<CardId>> {
        let me = state.active;
        let opp = me.opponent();
        let mut rng = crate::rng::policy_rng(seed);
        let (world_seeds, wbase_used, _) = self.root_world_seeds(state, me, n, &mut rng);
        let block = self.deal == Deal::Block && self.hread.is_none();
        let deal_seed = if block {
            if self.turn_stable_worlds() {
                let base = wbase_used.expect("turn-stable worlds require a base");
                let mut seed_rng = Xoshiro256ss::from_seed(base);
                for _ in 0..n {
                    seed_rng.next_u64();
                }
                Some(seed_rng.next_u64())
            } else {
                Some(rng.next_u64())
            }
        } else {
            None
        };
        let mut hands = Vec::with_capacity(world_seeds.len());
        for (world, wseed) in world_seeds.into_iter().enumerate() {
            let d = determinize_block(
                state,
                me,
                wseed,
                deal_seed.unwrap_or(0),
                world as u32,
                self.info,
                None,
                block,
                self.hread_deal(),
            );
            hands.push(d.player(opp).hand.iter().map(|c| c.card).collect());
        }
        hands
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
        let line = [line_key(state)];
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
    let v = *tt?.get(&(search_key_id(state), depth as u8, me_to_move))?;
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
        tt.insert((search_key_id(state), depth as u8, me_to_move), v);
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
        self::job::choose_sync(self, db, state, legal, rng)
    }

    fn last_value(&self) -> Option<f32> {
        self.last_value
    }
}

#[path = "job.rs"]
pub mod job;

impl H0 {
    fn choose_mulligan(
        &mut self,
        db: &CardDb,
        state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize {
        match self.mull {
            MullMode::Rule => mulligan_index(state, legal),
            MullMode::Random => mulligan_random(state, legal, rng),
            MullMode::Table => mulligan_table(self, db, state, legal),
        }
    }
}

fn mulligan_rule_mask(state: &State) -> [bool; 4] {
    let me = acting_player(state);
    let hand = &state.player(me).hand;
    let mut want = [false; 4];
    for (i, slot) in want.iter_mut().enumerate() {
        if let Some(c) = hand.get(i) {
            *slot = c.cost >= 4;
        }
    }
    want
}

fn mulligan_index(state: &State, legal: &[Action]) -> usize {
    let want = mulligan_rule_mask(state);
    legal
        .iter()
        .position(|a| matches!(a, Action::MulliganConfirm { swap } if *swap == want))
        .unwrap_or(0)
}

fn mulligan_random(state: &State, legal: &[Action], rng: &mut Xoshiro256ss) -> usize {
    let me = acting_player(state);
    let n = state.player(me).hand.len().min(4);
    let bits = rng.next_u64();
    let mut want = [false; 4];
    for (i, slot) in want.iter_mut().enumerate().take(n) {
        *slot = (bits >> i) & 1 == 1;
    }
    legal
        .iter()
        .position(|a| matches!(a, Action::MulliganConfirm { swap } if *swap == want))
        .unwrap_or_else(|| mulligan_index(state, legal))
}

fn mulligan_table(h0: &mut H0, _db: &CardDb, state: &State, legal: &[Action]) -> usize {
    let me = acting_player(state);
    let fp = deck_fingerprint_player(state.player(me));
    let table = h0.mull_table.get_or_insert_with(builtin_mulligan);
    let Some(entry) = table.lookup(&fp) else {
        h0.stats.mull_fallback += 1;
        return mulligan_index(state, legal);
    };
    h0.stats.mull_table += 1;
    let seat = mulligan_seat(state.first, me);
    let seat_map = if seat == "first" {
        &entry.first
    } else {
        &entry.second
    };
    let hand = &state.player(me).hand;
    let mut want = [false; 4];
    for (i, slot) in want.iter_mut().enumerate() {
        if let Some(c) = hand.get(i) {
            let id = c.card.to_string();
            let keep = match seat_map.get(&id) {
                Some(k) => *k,
                None => c.cost < 4,
            };
            *slot = !keep;
        }
    }
    legal
        .iter()
        .position(|a| matches!(a, Action::MulliganConfirm { swap } if *swap == want))
        .unwrap_or_else(|| mulligan_index(state, legal))
}

fn useful_action(state: &State, me: PlayerId, a: &Action, bpp1: u32, bpp2: u32) -> bool {
    match a {
        // Activate only — never cancel an unspent orb.
        Action::BonusPp => {
            let p = state.player(me);
            if p.bonus_pp.active {
                return false;
            }
            let turns = p.turns_taken;
            if turns < 6 {
                turns >= bpp1 && p.bonus_pp.early_charge
            } else {
                turns >= bpp2 && p.bonus_pp.late_charge
            }
        }
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

/// Only a Choice or Mulligan state can repeat an earlier state of its line: every other `apply` bumps
/// `step_counter` (see `drain_until_quiet`), and the key includes it.
#[inline]
pub(super) fn can_repeat(s: &State) -> bool {
    matches!(s.phase, Phase::Choice { .. } | Phase::Mulligan { .. })
}

/// Line entry for the repetition check: the state's key where a repeat is possible, else a per-step tag that no
/// state of the same line can produce.
#[inline]
pub(super) fn line_key(s: &State) -> u64 {
    if can_repeat(s) {
        search_key_id(s)
    } else {
        0x6c69_6e65_0000_0000 ^ s.step_counter
    }
}

/// [`try_apply`] for `a` taken from `legal_actions(db, state)` of this very state (see [`apply_known_legal`]).
fn try_apply_legal(
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
    if crate::apply::apply_known_legal(db, &mut s, a.clone()).is_err() {
        return None;
    }
    if can_repeat(&s) && line.contains(&line_key(&s)) {
        return None;
    }
    Some(s)
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
    if can_repeat(&s) && line.contains(&line_key(&s)) {
        return None;
    }
    Some(s)
}

struct WorldSearchResult {
    acc: Vec<f32>,
    n: Vec<u32>,
    worst: Vec<f32>,
    nodes_spent: u32,
    stats: SearchStats,
    attempted: u64,
    skipped_worlds: u64,
    explain: Vec<Vec<WorldRecord>>,
}

fn fair_world_pair_cap(remaining: u32, pairs_left: usize, nodes_so_far: u32, hard_cap: u32) -> u32 {
    const MIN_SHARE: u32 = 24;
    let even = remaining / pairs_left as u32;
    let share = if remaining >= MIN_SHARE.saturating_mul(pairs_left as u32) {
        even.max(MIN_SHARE)
    } else {
        even.max(1)
    };
    nodes_so_far.saturating_add(share).min(hard_cap)
}

#[allow(clippy::too_many_arguments)]
fn search_world(
    db: &CardDb,
    root: &State,
    r: usize,
    world_budget: u32,
    subset: &[Action],
    me: PlayerId,
    depth: u32,
    beam: usize,
    tt_on: bool,
    info: Info,
    eval: Evaluator<'_>,
    odepth: u32,
    obeam: usize,
    fusemacro: bool,
    horizon: u32,
    hres: u32,
    recording: bool,
) -> WorldSearchResult {
    let n_cand = subset.len();
    let mut acc = vec![0.0f32; n_cand];
    let mut n = vec![0u32; n_cand];
    let mut worst = vec![f32::INFINITY; n_cand];
    let mut explain = if recording {
        vec![Vec::new(); n_cand]
    } else {
        Vec::new()
    };
    let mut world_nodes = 0u32;
    let mut stats = SearchStats::default();
    let mut attempted = 0u64;
    let mut skipped_worlds = 0u64;
    let mut shared_table = if info != Info::All {
        tt_on.then(HashMap::new)
    } else {
        None
    };
    let mut root_table = if info == Info::All {
        tt_on.then(HashMap::new)
    } else {
        None
    };
    let root_key = line_key(root);
    for (j, a) in subset.iter().enumerate() {
        if world_nodes >= world_budget {
            skipped_worlds += (subset.len() - j) as u64;
            if recording {
                for ej in explain.iter_mut().skip(j) {
                    ej.push(PvTracker::skipped_world(r as u32, 0));
                }
            }
            break;
        }
        attempted += 1;
        let pairs_left = subset.len() - j;
        let remaining = world_budget - world_nodes;
        let cap = fair_world_pair_cap(remaining, pairs_left, world_nodes, world_budget);
        let pair_nodes_start = world_nodes;
        let mut tracker = if recording {
            Some(PvTracker::new(cap))
        } else {
            None
        };
        let Some(s) = try_apply(db, root, a, &mut world_nodes, cap, &[root_key]) else {
            if recording {
                explain[j].push(PvTracker::skipped_world(r as u32, cap));
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
            let at_fuse_choice = fusemacro && own_fuse_partners(&s, me).is_some();
            let search_depth = if at_fuse_choice {
                depth
            } else {
                depth.saturating_sub(1)
            };
            search_own(
                db,
                &s,
                me,
                search_depth,
                beam,
                &mut world_nodes,
                cap,
                &line,
                eval,
                odepth,
                obeam,
                &mut stats,
                if info == Info::All {
                    root_table.as_mut()
                } else {
                    shared_table.as_mut()
                },
                tracker.as_mut(),
                fusemacro,
                horizon,
                hres,
            )
        };
        if let Some(t) = tracker.as_mut() {
            t.note_nodes(world_nodes);
        }
        let fv = finite(v, eval.wv);
        if recording {
            let tracker = tracker.unwrap_or_else(|| PvTracker::new(cap));
            explain[j].push(tracker.into_world(
                r as u32,
                v,
                fv,
                false,
                world_nodes - pair_nodes_start,
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
    WorldSearchResult {
        acc,
        n,
        worst,
        nodes_spent: world_nodes,
        stats,
        attempted,
        skipped_worlds,
        explain,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn kill_det_on_roots(
    db: &CardDb,
    roots: &[State],
    legal: &[Action],
    cand: &[usize],
    me: PlayerId,
    budget: u32,
    stats: &mut SearchStats,
) -> (Option<usize>, u32) {
    if roots.is_empty() {
        return (None, 0);
    }
    stats.own_solver_calls += 1;
    let verdict = forced_lethal_det(db, &roots[0], budget);
    let nodes = match &verdict {
        LethalVerdict::Lethal { nodes, .. }
        | LethalVerdict::None { nodes }
        | LethalVerdict::Unknown { nodes } => *nodes,
    };
    stats.own_solver_nodes += u64::from(nodes);
    stats.own_solver_nodes_max = stats.own_solver_nodes_max.max(u64::from(nodes));
    match verdict {
        LethalVerdict::Lethal { line, .. } => {
            stats.own_solver_found += 1;
            let mut all_roots_confirmed = true;
            for root in roots.iter().skip(1) {
                if !confirm_det_lethal_line(db, root, me, &line) {
                    all_roots_confirmed = false;
                    break;
                }
            }
            if all_roots_confirmed {
                let first = &line[0];
                if let Some(idx) = legal.iter().position(|a| a == first) {
                    if cand.contains(&idx) {
                        stats.own_solver_taken += 1;
                        return (Some(idx), nodes);
                    }
                }
                stats.own_solver_rejected += 1;
            } else {
                stats.own_solver_rejected += 1;
            }
        }
        LethalVerdict::Unknown { .. } => {
            stats.own_solver_unknown += 1;
        }
        LethalVerdict::None { .. } => {}
    }
    (None, nodes)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn kill_roll_on_roots(
    db: &CardDb,
    roots: &[State],
    legal: &[Action],
    cand: &[usize],
    me: PlayerId,
    budget: u32,
    tkroll: u32,
    stats: &mut SearchStats,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> (Option<usize>, u32) {
    if tkroll == 0 {
        return (None, 0);
    }
    if roots.is_empty() {
        return (None, 0);
    }
    stats.own_roll_calls += 1;
    let confirm_line = |line: &[Action]| -> bool {
        for root in roots {
            let pos_key = search_key(root);
            let seeds: Vec<u64> = (0..tkroll).map(|i| roll_confirm_seed(pos_key, i)).collect();
            if !confirm_lethal_line_rerolled(db, root, me, line, &seeds) {
                stats.own_roll_lines_rejected += 1;
                return false;
            }
        }
        true
    };
    let roll_verdict = match cancel {
        Some(cancel) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                forced_lethal_accepting_cancel(db, &roots[0], budget, confirm_line, cancel)
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = cancel;
                forced_lethal_accepting(db, &roots[0], budget, confirm_line)
            }
        }
        None => forced_lethal_accepting(db, &roots[0], budget, confirm_line),
    };
    let roll_nodes = match &roll_verdict {
        LethalVerdict::Lethal { nodes, .. }
        | LethalVerdict::None { nodes }
        | LethalVerdict::Unknown { nodes } => *nodes,
    };
    stats.own_roll_nodes += u64::from(roll_nodes);
    stats.own_roll_nodes_max = stats.own_roll_nodes_max.max(u64::from(roll_nodes));
    match roll_verdict {
        LethalVerdict::Lethal { line, .. } => {
            stats.own_roll_found += 1;
            let first = &line[0];
            let Some(idx) = legal.iter().position(|a| a == first) else {
                stats.own_roll_rejected += 1;
                return (None, roll_nodes);
            };
            if !cand.contains(&idx) {
                stats.own_roll_rejected += 1;
                return (None, roll_nodes);
            }
            stats.own_roll_taken += 1;
            (Some(idx), roll_nodes)
        }
        LethalVerdict::Unknown { .. } => {
            stats.own_roll_unknown += 1;
            stats.own_roll_rejected += 1;
            (None, roll_nodes)
        }
        LethalVerdict::None { .. } => {
            stats.own_roll_rejected += 1;
            (None, roll_nodes)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn try_take_kill_parallel(
    db: &CardDb,
    roots: &[State],
    legal: &[Action],
    cand: &[usize],
    me: PlayerId,
    budget: u32,
    tkroll: u32,
    stats: &mut SearchStats,
) -> Option<(usize, ChoosePath)> {
    use std::sync::atomic::{AtomicBool, Ordering};

    let cancel = AtomicBool::new(false);
    std::thread::scope(|s| {
        let roll_handle = s.spawn(|| {
            let mut roll_stats = SearchStats::default();
            let (taken, _) = kill_roll_on_roots(
                db,
                roots,
                legal,
                cand,
                me,
                budget,
                tkroll,
                &mut roll_stats,
                Some(&cancel),
            );
            (taken, roll_stats)
        });

        let (taken, nodes) = kill_det_on_roots(db, roots, legal, cand, me, budget, stats);
        if let Some(idx) = taken {
            cancel.store(true, Ordering::Relaxed);
            return Some((idx, ChoosePath::TakeKill));
        }
        let _ = nodes;

        let (roll_taken, roll_stats) = roll_handle.join().expect("roll thread join");
        stats.accum(&roll_stats);
        roll_taken.map(|idx| (idx, ChoosePath::TakeKillRoll))
    })
}

#[allow(clippy::too_many_arguments)]
fn try_take_kill(
    db: &CardDb,
    roots: &[State],
    legal: &[Action],
    cand: &[usize],
    me: PlayerId,
    budget: u32,
    tkroll: u32,
    stats: &mut SearchStats,
) -> Option<(usize, ChoosePath)> {
    let (taken, _) = kill_det_on_roots(db, roots, legal, cand, me, budget, stats);
    if let Some(idx) = taken {
        return Some((idx, ChoosePath::TakeKill));
    }
    let (roll_taken, _) =
        kill_roll_on_roots(db, roots, legal, cand, me, budget, tkroll, stats, None);
    roll_taken.map(|idx| (idx, ChoosePath::TakeKillRoll))
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
        let line = [line_key(root)];
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
    next_line.push(line_key(&s));
    next.iter().any(|b| {
        useful_action(&s, me, b, 1, 6)
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

#[derive(Clone, PartialEq, Eq)]
enum GreedyPick {
    Plain(usize),
    PlayMacro { index: usize, evolve: Action },
}

fn greedy_action_indices(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    legal: &[Action],
    bpp1: u32,
    bpp2: u32,
    skip_noop_fuse: bool,
) -> Vec<usize> {
    let useful: Vec<usize> = legal
        .iter()
        .enumerate()
        .filter(|(_, a)| useful_action(state, me, a, bpp1, bpp2))
        .map(|(i, _)| i)
        .collect();
    if !skip_noop_fuse {
        return useful;
    }
    let mut dropped = 0u32;
    let mut kept = Vec::with_capacity(useful.len());
    for &i in &useful {
        if fuse_action_is_noop(db, state, me, &legal[i]) {
            dropped += 1;
        } else {
            kept.push(i);
        }
    }
    if dropped > 0 && !kept.is_empty() {
        kept
    } else {
        useful
    }
}

#[allow(clippy::too_many_arguments)]
fn greedy_index(
    db: &CardDb,
    state: &State,
    legal: &[Action],
    choose_for: PlayerId,
    search_me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    skip_noop_fuse: bool,
) -> (usize, Option<State>) {
    let indices = greedy_action_indices(
        db,
        state,
        choose_for,
        legal,
        eval.bpp1,
        eval.bpp2,
        skip_noop_fuse,
    );
    let mut best_i = indices.first().copied().unwrap_or(0);
    let mut best_v = f32::NEG_INFINITY;
    let mut best_s: Option<State> = None;
    for i in indices {
        if *nodes >= cap {
            break;
        }
        let a = &legal[i];
        let Some(s) = try_apply_legal(db, state, a, nodes, cap, line) else {
            continue;
        };
        let v = eval.pick_value(&s, choose_for, search_me);
        if v > best_v {
            best_v = v;
            best_i = i;
            best_s = Some(s);
        }
    }
    (best_i, best_s)
}

/// The re-apply of greedy_index's pick without redoing it: the same cap check and node charge as `try_apply`;
/// the kept afterstate is what that apply would produce (and it already passed the same line check).
#[allow(clippy::too_many_arguments)]
fn take_or_apply(
    db: &CardDb,
    state: &State,
    a: &Action,
    kept: Option<State>,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> Option<State> {
    match kept {
        Some(s) => {
            if *nodes >= cap {
                return None;
            }
            *nodes += 1;
            Some(s)
        }
        None => try_apply(db, state, a, nodes, cap, line),
    }
}

#[allow(clippy::too_many_arguments)]
fn greedy_pick(
    db: &CardDb,
    state: &State,
    legal: &[Action],
    choose_for: PlayerId,
    search_me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    macro_evolve: bool,
) -> GreedyPick {
    let mut best = GreedyPick::Plain(0);
    let mut best_v = f32::NEG_INFINITY;
    for (i, a) in legal.iter().enumerate() {
        if *nodes >= cap {
            break;
        }
        if !useful_action(state, choose_for, a, eval.bpp1, eval.bpp2) {
            continue;
        }
        let (v, evo) = if macro_evolve && matches!(a, Action::Play { .. }) {
            score_play_with_macro(db, state, a, choose_for, search_me, nodes, cap, line, eval)
        } else {
            let Some(s) = try_apply(db, state, a, nodes, cap, line) else {
                continue;
            };
            (eval.pick_value(&s, choose_for, search_me), None)
        };
        if v > best_v {
            best_v = v;
            best = match evo {
                Some(e) => GreedyPick::PlayMacro {
                    index: i,
                    evolve: e,
                },
                None => GreedyPick::Plain(i),
            };
        }
    }
    best
}

#[allow(clippy::too_many_arguments)]
fn score_play_with_macro(
    db: &CardDb,
    state: &State,
    play: &Action,
    choose_for: PlayerId,
    search_me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
) -> (f32, Option<Action>) {
    let Some(mut s) = try_apply(db, state, play, nodes, cap, line) else {
        return (f32::NEG_INFINITY, None);
    };
    let mut line2 = line.to_vec();
    line2.push(line_key(&s));
    if !greedy_resolve_choices(
        db, &mut s, choose_for, search_me, nodes, cap, &mut line2, eval,
    ) {
        return (f32::NEG_INFINITY, None);
    }
    let play_v = eval.pick_value(&s, choose_for, search_me);
    let faces = leader_attackers(db, &s);
    let prefer = new_follower_slots(state, &s, choose_for);
    let mut best_v = play_v;
    let mut best_evo = None;
    if !prefer.is_empty() {
        for slot in &prefer {
            let mut per_slot = 0usize;
            for a in ordered_evolves(db, &s, &prefer, &faces) {
                let Action::Evolve { slot: sl, .. } = &a else {
                    continue;
                };
                if sl.0 != *slot {
                    continue;
                }
                if per_slot >= 2 {
                    break;
                }
                if *nodes >= cap {
                    break;
                }
                let Some(s2) = try_apply(db, &s, &a, nodes, cap, &line2) else {
                    continue;
                };
                per_slot += 1;
                let v2 = eval.pick_value(&s2, choose_for, search_me);
                if v2 > best_v {
                    best_v = v2;
                    best_evo = Some(a);
                }
            }
        }
    }
    (best_v, best_evo)
}

fn new_follower_slots(before: &State, after: &State, who: PlayerId) -> Vec<u8> {
    let mut out = Vec::new();
    for i in 0..FIELD_SIZE {
        let now = after.player(who).field[i].as_ref();
        if now.is_none_or(|c| c.kind != CardKind::Follower) {
            continue;
        }
        let was = before.player(who).field[i].as_ref();
        if was.is_none() || was.is_some_and(|c| c.id != now.unwrap().id) {
            out.push(i as u8);
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn greedy_resolve_choices(
    db: &CardDb,
    state: &mut State,
    choose_for: PlayerId,
    search_me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &mut Vec<u64>,
    eval: Evaluator<'_>,
) -> bool {
    while matches!(state.phase, Phase::Choice { player, .. } if player == choose_for) {
        if *nodes >= cap {
            return false;
        }
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            return false;
        }
        let (i, kept) = greedy_index(
            db, state, &legal, choose_for, search_me, nodes, cap, line, eval, false,
        );
        let Some(next) = take_or_apply(db, state, &legal[i], kept, nodes, cap, line) else {
            return false;
        };
        line.push(line_key(&next));
        *state = next;
    }
    true
}

/// Partner cards to pick for one fuse completion (`fusemacro=1`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct FuseCompletion {
    partners: Vec<CardId>,
}

fn own_fuse_partners(state: &State, me: PlayerId) -> Option<(u8, Vec<u8>, Vec<u8>)> {
    match &state.phase {
        Phase::Choice {
            player,
            node:
                ChoiceNode::FusePartners {
                    host,
                    options,
                    picked,
                },
        } if *player == me => Some((*host, options.clone(), picked.clone())),
        _ => None,
    }
}

fn fuse_host_has_recipes(db: &CardDb, state: &State, me: PlayerId, host: u8) -> bool {
    state
        .player(me)
        .hand
        .get(host as usize)
        .and_then(|inst| db.card(inst.card).ok())
        .and_then(|card| card.fuse())
        .is_some_and(|f| f.recipes.is_some())
}

fn distinct_unpicked_partner_cards(
    state: &State,
    me: PlayerId,
    options: &[u8],
    picked: &[u8],
) -> Vec<CardId> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for &pos in options {
        if picked.contains(&pos) {
            continue;
        }
        let Some(inst) = state.player(me).hand.get(pos as usize) else {
            continue;
        };
        if !seen.contains(&inst.card) {
            seen.push(inst.card);
            out.push(inst.card);
        }
    }
    out
}

/// Partner-card sets for each fuse completion (`fusemacro=1`). Exposed for tests.
#[doc(hidden)]
pub fn fuse_completion_partner_sets(db: &CardDb, state: &State, me: PlayerId) -> Vec<Vec<CardId>> {
    fuse_completions(db, state, me)
        .into_iter()
        .map(|c| c.partners)
        .collect()
}

fn fuse_completions(db: &CardDb, state: &State, me: PlayerId) -> Vec<FuseCompletion> {
    let Some((host, options, picked)) = own_fuse_partners(state, me) else {
        return Vec::new();
    };
    let has_recipes = fuse_host_has_recipes(db, state, me, host);
    let distinct = distinct_unpicked_partner_cards(state, me, &options, &picked);
    let mut out = Vec::new();
    if !picked.is_empty() {
        out.push(FuseCompletion {
            partners: Vec::new(),
        });
        if has_recipes {
            for card in distinct {
                out.push(FuseCompletion {
                    partners: vec![card],
                });
            }
        }
    } else {
        for card in &distinct {
            out.push(FuseCompletion {
                partners: vec![*card],
            });
        }
        if has_recipes {
            for i in 0..distinct.len() {
                for j in (i + 1)..distinct.len() {
                    out.push(FuseCompletion {
                        partners: vec![distinct[i], distinct[j]],
                    });
                }
            }
        }
    }
    out
}

fn fuse_single_partner_completions(state: &State, me: PlayerId) -> Vec<FuseCompletion> {
    let Some((_, options, picked)) = own_fuse_partners(state, me) else {
        return Vec::new();
    };
    if !picked.is_empty() {
        return vec![FuseCompletion {
            partners: Vec::new(),
        }];
    }
    distinct_unpicked_partner_cards(state, me, &options, &picked)
        .into_iter()
        .map(|card| FuseCompletion {
            partners: vec![card],
        })
        .collect()
}

fn choose_targets_card(state: &State, action: &Action, card: CardId) -> bool {
    if !matches!(action, Action::Choose(_)) {
        return false;
    }
    match &to_neutral(state, action) {
        NeutralAction::Choose {
            option: ChooseOptionJson::Card { card: id },
            ..
        } => CardId::parse(id) == Some(card),
        _ => false,
    }
}

fn find_choose_for_card(db: &CardDb, state: &State, card: CardId) -> Option<Action> {
    legal_actions(db, state)
        .into_iter()
        .find(|a| choose_targets_card(state, a, card))
}

fn try_apply_uncharged(
    db: &CardDb,
    state: &State,
    action: &Action,
    line: &[u64],
    stats: &mut SearchStats,
) -> Option<State> {
    let mut s = state.clone();
    stats.fuse_overshoot += 1;
    if apply(db, &mut s, action.clone()).is_err() {
        return None;
    }
    if can_repeat(&s) && line.contains(&line_key(&s)) {
        return None;
    }
    Some(s)
}

#[allow(clippy::too_many_arguments)]
fn apply_fuse_completion(
    db: &CardDb,
    state: &State,
    completion: &FuseCompletion,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    charge: bool,
    stats: &mut SearchStats,
) -> Option<(State, Vec<Action>)> {
    let mut s = state.clone();
    let mut actions = Vec::new();
    let mut cur_line = line.to_vec();
    for &card in &completion.partners {
        let action = find_choose_for_card(db, &s, card)?;
        actions.push(action.clone());
        s = if charge {
            try_apply(db, &s, &action, nodes, cap, &cur_line)?
        } else {
            try_apply_uncharged(db, &s, &action, &cur_line, stats)?
        };
        cur_line.push(line_key(&s));
    }
    let confirm = Action::Confirm;
    actions.push(confirm.clone());
    s = if charge {
        try_apply(db, &s, &confirm, nodes, cap, &cur_line)?
    } else {
        try_apply_uncharged(db, &s, &confirm, &cur_line, stats)?
    };
    Some((s, actions))
}

#[allow(clippy::too_many_arguments)]
fn fuse_overshoot_score(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    line: &[u64],
    eval: Evaluator<'_>,
    stats: &mut SearchStats,
    track: Option<&mut PvTracker>,
    end: PvEnd,
) -> f32 {
    let singles = fuse_single_partner_completions(state, me);
    let mut best_order = f32::NEG_INFINITY;
    let mut best_leaf = f32::NEG_INFINITY;
    let mut best_state = state.clone();
    let mut best_actions = Vec::new();
    for completion in singles {
        if let Some((s, actions)) =
            apply_fuse_completion(db, state, &completion, &mut 0, u32::MAX, line, false, stats)
        {
            let order_v = eval.pick_value(&s, me, me);
            if order_v > best_order {
                best_order = order_v;
                best_leaf = if eval.order_net.is_none() {
                    order_v
                } else {
                    eval.value(&s, me)
                };
                best_state = s;
                best_actions = actions;
            }
        }
    }
    if !best_leaf.is_finite() {
        best_leaf = eval.value(state, me);
        best_state = state.clone();
        best_actions.clear();
    }
    if let Some(t) = track {
        let plen = t.path_len();
        for a in &best_actions {
            t.push(a.clone());
        }
        t.set_leaf(best_leaf, end, &best_state);
        t.pop_to(plen);
    }
    best_leaf
}

/// At a Main node, score every fuse completion but return only the best by
/// immediate leaf value (ties keep completion order).
#[allow(clippy::too_many_arguments)]
fn best_fuse_main_child(
    db: &CardDb,
    state: &State,
    fuse_action: &Action,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    stats: &mut SearchStats,
) -> Option<(f32, State, u64, Vec<Action>)> {
    let after_fuse = try_apply(db, state, fuse_action, nodes, cap, line)?;
    let mut best: Option<(f32, State, u64, Vec<Action>)> = None;
    for completion in fuse_completions(db, &after_fuse, me) {
        if *nodes >= cap {
            break;
        }
        let Some((s, tail)) =
            apply_fuse_completion(db, &after_fuse, &completion, nodes, cap, line, true, stats)
        else {
            continue;
        };
        let order_v = eval.pick_value(&s, me, me);
        if best.as_ref().is_none_or(|(bv, _, _, _)| order_v > *bv) {
            let mut prefix = vec![fuse_action.clone()];
            prefix.extend(tail);
            let key = line_key(&s);
            best = Some((order_v, s, key, prefix));
        }
    }
    best
}

fn horizon_work_cap(nodes: u32, cap: u32, hres: u32) -> u32 {
    nodes + cap.saturating_sub(nodes).max(hres)
}

fn charge_horizon_work(
    nodes_before: u32,
    nodes_after: u32,
    cap: u32,
    stats: &mut SearchStats,
    track: Option<&mut PvTracker>,
) {
    let charged = nodes_after.min(cap).saturating_sub(nodes_before.min(cap));
    let total = nodes_after.saturating_sub(nodes_before);
    let uncharged = total - charged;
    if uncharged > 0 {
        stats.horizon_nodes += u64::from(uncharged);
        if let Some(t) = track {
            t.add_horizon_nodes(uncharged);
        }
    }
}

/// Reserve applies past the pair cap must not shrink later pairs' fair shares.
fn clamp_horizon_charged_nodes(nodes: &mut u32, nodes_before: u32, cap: u32) {
    *nodes = (*nodes).min(cap.max(nodes_before));
}

fn horizon_reply_end(cutoff: HorizonCutoff) -> PvEnd {
    match cutoff {
        HorizonCutoff::Depth => PvEnd::DepthReply,
        HorizonCutoff::Cap => PvEnd::CapReply,
    }
}

fn fix_horizon_pv_end(track: Option<&mut PvTracker>, cutoff: HorizonCutoff) {
    if let Some(t) = track {
        if let Some(mut line) = t.take_last() {
            if line.end != PvEnd::OppLethal
                && line.end != PvEnd::OppSolver
                && line.end != PvEnd::OppOpen
                && line.end != PvEnd::OppNext
                && line.end != PvEnd::OwnNext
                && line.end != PvEnd::NextLeaf
            {
                line.end = horizon_reply_end(cutoff);
            }
            t.restore_last(Some(line));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_bot_choices(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    nodes: &mut u32,
    work_cap: u32,
    line: &mut Vec<u64>,
    eval: Evaluator<'_>,
    mut track: Option<&mut PvTracker>,
) -> bool {
    while matches!(state.phase, Phase::Choice { player, .. } if player == me) {
        if *nodes >= work_cap {
            return false;
        }
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            return false;
        }
        let (i, kept) = greedy_index(
            db, state, &legal, me, me, nodes, work_cap, line, eval, false,
        );
        let Some(next) = take_or_apply(db, state, &legal[i], kept, nodes, work_cap, line) else {
            return false;
        };
        if let Some(t) = track.as_deref_mut() {
            t.push(legal[i].clone());
        }
        line.push(line_key(&next));
        *state = next;
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn try_bot_end_turn(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    nodes: &mut u32,
    work_cap: u32,
    line: &mut Vec<u64>,
    mut track: Option<&mut PvTracker>,
) -> bool {
    if acting_player(state) != me {
        return true;
    }
    if !matches!(state.phase, Phase::Main | Phase::Combat) {
        return false;
    }
    let legal = legal_actions(db, state);
    let Some(et) = legal.iter().find(|a| matches!(a, Action::EndTurn)) else {
        return false;
    };
    if *nodes >= work_cap {
        return false;
    }
    let Some(next) = try_apply(db, state, et, nodes, work_cap, line) else {
        return false;
    };
    if let Some(t) = track.as_mut() {
        t.push(et.clone());
    }
    line.push(line_key(&next));
    *state = next;
    true
}

fn bot_turn_still_active(state: &State, me: PlayerId) -> bool {
    acting_player(state) == me
        && !matches!(state.phase, Phase::Terminal)
        && matches!(
            state.phase,
            Phase::Main | Phase::Combat | Phase::Choice { .. }
        )
}

#[allow(clippy::too_many_arguments)]
fn finish_bot_turn_horizon(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    horizon: u32,
    nodes: &mut u32,
    work_cap: u32,
    line: &mut Vec<u64>,
    eval: Evaluator<'_>,
    mut track: Option<&mut PvTracker>,
) -> bool {
    if horizon >= 3 {
        greedy_until_end(
            db,
            state,
            me,
            me,
            nodes,
            work_cap,
            line,
            eval,
            track.as_deref_mut(),
            eval.fuseguard,
            0,
        );
    } else if !resolve_bot_choices(
        db,
        state,
        me,
        nodes,
        work_cap,
        line,
        eval,
        track.as_deref_mut(),
    ) {
        return false;
    }
    if bot_turn_still_active(state, me)
        && !try_bot_end_turn(db, state, me, nodes, work_cap, line, track)
    {
        return false;
    }
    !bot_turn_still_active(state, me)
}

#[allow(clippy::too_many_arguments)]
fn horizon_score_leaf(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    hres: u32,
    horizon: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    odepth: u32,
    obeam: usize,
    stats: &mut SearchStats,
    mut track: Option<&mut PvTracker>,
    cutoff: HorizonCutoff,
) -> f32 {
    let nodes_before = *nodes;
    let track_len = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
    let work_cap = horizon_work_cap(*nodes, cap, hres);
    let mut reply_state = state.clone();
    let mut reply_line = line.to_vec();

    if horizon >= 2
        && bot_turn_still_active(&reply_state, me)
        && !finish_bot_turn_horizon(
            db,
            &mut reply_state,
            me,
            horizon,
            nodes,
            work_cap,
            &mut reply_line,
            eval,
            track.as_deref_mut(),
        )
    {
        stats.horizon_fallback += 1;
        let nodes_after_work = *nodes;
        charge_horizon_work(
            nodes_before,
            nodes_after_work,
            cap,
            stats,
            track.as_deref_mut(),
        );
        clamp_horizon_charged_nodes(nodes, nodes_before, cap);
        let v = eval.value(state, me);
        let end = match cutoff {
            HorizonCutoff::Depth => PvEnd::Depth,
            HorizonCutoff::Cap => PvEnd::Cap,
        };
        if let Some(t) = track.as_mut() {
            t.pop_to(track_len);
            t.set_leaf(v, end, state);
        }
        return v;
    }

    if !eval.opp_reply {
        let v = eval.value(&reply_state, me);
        let end = match cutoff {
            HorizonCutoff::Depth => PvEnd::Depth,
            HorizonCutoff::Cap => PvEnd::Cap,
        };
        if let Some(t) = track.as_mut() {
            t.pop_to(track_len);
            t.set_leaf(v, end, &reply_state);
        }
        return v;
    }

    let v = opponent_reply(
        db,
        &reply_state,
        me,
        nodes,
        work_cap,
        &reply_line,
        eval,
        odepth,
        obeam,
        stats,
        None,
        track.as_deref_mut(),
    );
    let nodes_after_work = *nodes;
    charge_horizon_work(
        nodes_before,
        nodes_after_work,
        cap,
        stats,
        track.as_deref_mut(),
    );
    clamp_horizon_charged_nodes(nodes, nodes_before, cap);
    stats.horizon_leaves += 1;
    if matches!(cutoff, HorizonCutoff::Depth) || nodes_before >= cap {
        fix_horizon_pv_end(track, cutoff);
    }
    v
}

#[allow(clippy::too_many_arguments, clippy::needless_option_as_deref)]
fn search_own_fuse_choice(
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
    fusemacro: bool,
    horizon: u32,
    hres: u32,
) -> f32 {
    if depth == 0 || *nodes >= cap {
        return fuse_overshoot_score(
            db,
            state,
            me,
            line,
            eval,
            stats,
            track.as_deref_mut(),
            if depth == 0 { PvEnd::Depth } else { PvEnd::Cap },
        );
    }
    if let Some(v) = tt_get(tt.as_deref(), state, depth, true, stats) {
        if let Some(t) = track.as_deref_mut() {
            t.set_tt(v, state);
        }
        return v;
    }
    let completions = fuse_completions(db, state, me);
    if completions.is_empty() {
        return fuse_overshoot_score(
            db,
            state,
            me,
            line,
            eval,
            stats,
            track.as_deref_mut(),
            PvEnd::Cap,
        );
    }
    let mut best = f32::NEG_INFINITY;
    let mut best_line: Option<Line> = None;
    for completion in completions {
        if *nodes >= cap {
            break;
        }
        let plen = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
        let Some((s, tail)) =
            apply_fuse_completion(db, state, &completion, nodes, cap, line, true, stats)
        else {
            continue;
        };
        if let Some(t) = track.as_deref_mut() {
            for a in tail {
                t.push(a);
            }
        }
        let mut next_line = line.to_vec();
        next_line.push(line_key(&s));
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
            fusemacro,
            horizon,
            hres,
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
    if !best.is_finite() {
        let v = fuse_overshoot_score(
            db,
            state,
            me,
            line,
            eval,
            stats,
            track.as_deref_mut(),
            PvEnd::Cap,
        );
        tt_put(tt, state, depth, true, v, stats);
        return v;
    }
    if let Some(t) = track.as_deref_mut() {
        t.restore_last(best_line);
    }
    tt_put(tt, state, depth, true, best, stats);
    best
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
    fusemacro: bool,
    horizon: u32,
    hres: u32,
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
    let turn_over = acting_player(state) != me || matches!(state.phase, Phase::Terminal);
    if horizon >= 1 && turn_over && cap.saturating_sub(*nodes) < hres {
        return horizon_score_leaf(
            db,
            state,
            me,
            nodes,
            cap,
            hres,
            horizon,
            line,
            eval,
            odepth,
            obeam,
            stats,
            track.as_deref_mut(),
            HorizonCutoff::Cap,
        );
    }
    if *nodes >= cap {
        if fusemacro && own_fuse_partners(state, me).is_some() {
            return fuse_overshoot_score(
                db,
                state,
                me,
                line,
                eval,
                stats,
                track.as_deref_mut(),
                PvEnd::Cap,
            );
        }
        if horizon >= 2 && bot_turn_still_active(state, me) {
            return horizon_score_leaf(
                db,
                state,
                me,
                nodes,
                cap,
                hres,
                horizon,
                line,
                eval,
                odepth,
                obeam,
                stats,
                track.as_deref_mut(),
                HorizonCutoff::Cap,
            );
        }
        let v = eval.value(state, me);
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(v, PvEnd::Cap, state);
        }
        return v;
    }
    if acting_player(state) != me || matches!(state.phase, Phase::Terminal) {
        if !eval.opp_reply {
            let v = eval.value(state, me);
            if let Some(t) = track.as_deref_mut() {
                t.set_leaf(v, PvEnd::Depth, state);
            }
            return v;
        }
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
    if fusemacro && own_fuse_partners(state, me).is_some() {
        return search_own_fuse_choice(
            db, state, me, depth, beam, nodes, cap, line, eval, odepth, obeam, stats, tt, track,
            fusemacro, horizon, hres,
        );
    }
    if depth == 0 {
        if horizon >= 2 && bot_turn_still_active(state, me) {
            return horizon_score_leaf(
                db,
                state,
                me,
                nodes,
                cap,
                hres,
                horizon,
                line,
                eval,
                odepth,
                obeam,
                stats,
                track.as_deref_mut(),
                HorizonCutoff::Depth,
            );
        }
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

    let useful_indices: Vec<usize> = legal
        .iter()
        .enumerate()
        .filter(|(_, a)| useful_action(state, me, a, eval.bpp1, eval.bpp2))
        .map(|(i, _)| i)
        .collect();
    let action_indices = if eval.fuseguard {
        let mut dropped = 0u32;
        let mut kept = Vec::with_capacity(useful_indices.len());
        for &i in &useful_indices {
            if fuse_action_is_noop(db, state, me, &legal[i]) {
                dropped += 1;
            } else {
                kept.push(i);
            }
        }
        if dropped > 0 && !kept.is_empty() {
            kept
        } else {
            useful_indices
        }
    } else {
        useful_indices
    };

    let mut scored: Vec<(f32, State, u64, Vec<Action>)> = Vec::with_capacity(action_indices.len());
    for i in action_indices {
        if *nodes >= cap {
            break;
        }
        let a = &legal[i];
        if fusemacro && matches!(a, Action::Fuse { .. }) {
            let Some((v, s, k, prefix)) =
                best_fuse_main_child(db, state, a, me, nodes, cap, line, eval, stats)
            else {
                continue;
            };
            if s.winner == Some(me) {
                if let Some(t) = track.as_deref_mut() {
                    for act in prefix {
                        t.push(act);
                    }
                    t.set_leaf(INF, PvEnd::Terminal, &s);
                }
                tt_put(tt, state, depth, true, INF, stats);
                return INF;
            }
            scored.push((v, s, k, prefix));
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
        let k = line_key(&s);
        scored.push((eval.pick_value(&s, me, me), s, k, vec![a.clone()]));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(beam.max(1));

    let mut best = f32::NEG_INFINITY;
    let mut best_line: Option<Line> = None;
    for (_, s, k, prefix) in scored {
        let plen = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
        if let Some(t) = track.as_deref_mut() {
            for a in prefix {
                t.push(a);
            }
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
            fusemacro,
            horizon,
            hres,
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

/// After a greedy-reply leaf (`odepth=0`), finish the bot's turn greedily and
/// glance for opponent lethal on the following turn. Caps the line at
/// `-onext` when the sweep finds a kill.
#[allow(clippy::too_many_arguments)]
fn opp_next_cap_value(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    v: f32,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    onext: u32,
    stats: &mut SearchStats,
    default_end: PvEnd,
) -> (f32, PvEnd) {
    if onext == 0
        || state.winner.is_some()
        || matches!(state.phase, Phase::Terminal)
        || acting_player(state) != me
        || state.active != me
        || *nodes >= cap
    {
        return (v, default_end);
    }
    let nodes_start = *nodes;
    let mut p = state.clone();
    let work_line = line.to_vec();
    greedy_until_end(
        db,
        &mut p,
        me,
        me,
        nodes,
        cap,
        &work_line,
        eval,
        None,
        eval.fuseguard,
        0,
    );
    if p.winner == Some(me) {
        stats.opp_next_nodes += u64::from(*nodes - nodes_start);
        if eval.onextleaf || eval.onextwin > 0 {
            stats.opp_next_own_wins += 1;
        }
        if eval.onextleaf {
            return (eval.wv, PvEnd::OwnNext);
        }
        if eval.onextwin > 0 {
            let raised = v.max(eval.onextwin as f32);
            if raised > v {
                return (raised, PvEnd::OwnNext);
            }
        }
        return (v, default_end);
    }
    if acting_player(&p) != me.opponent() || *nodes >= cap {
        stats.opp_next_nodes += u64::from(*nodes - nodes_start);
        return (v, default_end);
    }
    stats.opp_next_checks += 1;
    let mut sweep_line = work_line;
    sweep_line.push(line_key(&p));
    let mut scratch = SearchStats::default();
    let found = opp_lethal_sweep(db, &p, me, nodes, cap, &sweep_line, eval, &mut scratch);
    stats.opp_next_nodes += u64::from(*nodes - nodes_start);
    let base = if eval.onextleaf {
        -eval.value(&p, me.opponent())
    } else {
        v
    };
    if found {
        stats.opp_next_found += 1;
        return (base.min(-(onext as f32)), PvEnd::OppNext);
    }
    if eval.onextleaf {
        stats.opp_next_leaf += 1;
        return (base, PvEnd::NextLeaf);
    }
    (v, default_end)
}

fn opening_node(
    state: &State,
    me: PlayerId,
    d: u32,
    oopen: u32,
    eval: Evaluator<'_>,
    nodes: u32,
    cap: u32,
) -> bool {
    d < oopen
        && d < eval.osteps
        && state.winner.is_none()
        && !matches!(state.phase, Phase::Terminal)
        && state.turn <= MAX_TURNS
        && acting_player(state) == me.opponent()
        && nodes < cap
}

#[allow(clippy::too_many_arguments, clippy::needless_option_as_deref)]
fn opponent_open_search(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    obeam: usize,
    oopen: u32,
    d: u32,
    stats: &mut SearchStats,
    mut track: Option<&mut PvTracker>,
) -> Option<(f32, Option<Line>)> {
    if !opening_node(state, me, d, oopen, eval, *nodes, cap) {
        return None;
    }
    let opp = me.opponent();
    let nodes_start = *nodes;
    let legal = legal_actions(db, state);
    if legal.is_empty() {
        return None;
    }

    let mut scored: Vec<(f32, State, u64, Action, usize)> = Vec::new();
    let mut end_turn_score: Option<f32> = None;
    let mut legal_idx = 0usize;
    for a in &legal {
        if *nodes >= cap {
            break;
        }
        if !useful_action(state, opp, a, eval.bpp1, eval.bpp2) {
            continue;
        }
        let Some(s) = try_apply_legal(db, state, a, nodes, cap, line) else {
            continue;
        };
        if s.winner == Some(opp) {
            let win_line = if let Some(t) = track.as_deref_mut() {
                let plen = t.path_len();
                t.push(a.clone());
                t.set_leaf(-eval.wv, PvEnd::Terminal, &s);
                t.pop_to(plen);
                t.take_last()
            } else {
                None
            };
            stats.opp_open_nodes += u64::from(*nodes - nodes_start);
            return Some((-eval.wv, win_line));
        }
        let k = line_key(&s);
        if matches!(a, Action::EndTurn) {
            end_turn_score = Some(eval.pick_value(&s, opp, me));
        } else {
            scored.push((eval.pick_value(&s, opp, me), s, k, a.clone(), legal_idx));
            legal_idx += 1;
        }
    }
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.4.cmp(&b.4))
    });
    scored.truncate(obeam.max(1));

    let mut best = f32::INFINITY;
    let mut best_line: Option<Line> = None;

    for (i, (score, s, k, a, _)) in scored.into_iter().enumerate() {
        if *nodes >= cap {
            break;
        }
        let skip_complete = i == 0 && end_turn_score.is_none_or(|e| score > e);

        if !skip_complete {
            let plen = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
            if let Some(t) = track.as_deref_mut() {
                t.push(a.clone());
            }
            let mut next_line = line.to_vec();
            next_line.push(k);
            let mut s_work = s.clone();
            if eval.omacro {
                greedy_until_end_omacro(
                    db,
                    &mut s_work,
                    opp,
                    me,
                    nodes,
                    cap,
                    &next_line,
                    eval,
                    track.as_deref_mut(),
                    d + 1,
                );
            } else {
                greedy_until_end(
                    db,
                    &mut s_work,
                    opp,
                    me,
                    nodes,
                    cap,
                    &next_line,
                    eval,
                    track.as_deref_mut(),
                    false,
                    d + 1,
                );
            }
            let (v, leaf_end) = if s_work.winner == Some(opp) {
                (-eval.wv, PvEnd::OppOpen)
            } else {
                let raw = eval.value(&s_work, me);
                opp_next_cap_value(
                    db,
                    &s_work,
                    me,
                    raw,
                    nodes,
                    cap,
                    &next_line,
                    eval,
                    eval.onext,
                    stats,
                    PvEnd::OppOpen,
                )
            };
            stats.opp_open_lines += 1;
            if v == -eval.wv {
                if let Some(t) = track.as_deref_mut() {
                    t.set_leaf(v, leaf_end, &s_work);
                    best_line = t.take_last();
                }
                stats.opp_open_nodes += u64::from(*nodes - nodes_start);
                return Some((v, best_line));
            }
            if v < best {
                best = v;
                if let Some(t) = track.as_deref_mut() {
                    t.set_leaf(v, leaf_end, &s_work);
                    best_line = t.take_last();
                }
            }
            if let Some(t) = track.as_deref_mut() {
                t.pop_to(plen);
            }
        }

        if d + 1 < oopen && opening_node(&s, me, d + 1, oopen, eval, *nodes, cap) {
            let plen = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
            if let Some(t) = track.as_deref_mut() {
                t.push(a.clone());
            }
            let mut next_line = line.to_vec();
            next_line.push(k);
            if let Some((rv, rline)) = opponent_open_search(
                db,
                &s,
                me,
                nodes,
                cap,
                &next_line,
                eval,
                obeam,
                oopen,
                d + 1,
                stats,
                track.as_deref_mut(),
            ) {
                if rv == -eval.wv {
                    stats.opp_open_nodes += u64::from(*nodes - nodes_start);
                    return Some((rv, rline));
                }
                if rv < best {
                    best = rv;
                    best_line = rline;
                }
            }
            if let Some(t) = track.as_deref_mut() {
                t.pop_to(plen);
            }
        }
    }

    stats.opp_open_nodes += u64::from(*nodes - nodes_start);
    if best.is_finite() {
        Some((best, best_line))
    } else {
        None
    }
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
        if eval.olethal && acting_player(state) == me.opponent() {
            if opp_lethal_sweep(db, state, me, nodes, cap, line, eval, stats) {
                stats.opp_leaves += 1;
                if let Some(t) = track.as_deref_mut() {
                    t.set_leaf(-eval.wv, PvEnd::OppLethal, state);
                }
                return -eval.wv;
            }
            if eval.olsolve > 0 && *nodes < cap {
                let budget = eval.olsolve.min(cap - *nodes);
                stats.opp_solver_calls += 1;
                let start = *nodes;
                match forced_lethal(db, state, budget) {
                    LethalVerdict::Lethal { nodes: spent, .. } => {
                        *nodes = nodes.saturating_add(spent).min(cap);
                        stats.opp_solver_found += 1;
                        stats.opp_solver_nodes += u64::from(*nodes - start);
                        stats.opp_leaves += 1;
                        if let Some(t) = track.as_deref_mut() {
                            t.set_leaf(-eval.wv, PvEnd::OppSolver, state);
                        }
                        return -eval.wv;
                    }
                    LethalVerdict::Unknown { nodes: spent } => {
                        *nodes = nodes.saturating_add(spent).min(cap);
                        stats.opp_solver_unknown += 1;
                        stats.opp_solver_nodes += u64::from(*nodes - start);
                    }
                    LethalVerdict::None { nodes: spent } => {
                        *nodes = nodes.saturating_add(spent).min(cap);
                        stats.opp_solver_nodes += u64::from(*nodes - start);
                    }
                }
            }
        }
        let plen0 = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
        let mut s = state.clone();
        if eval.omacro {
            greedy_until_end_omacro(
                db,
                &mut s,
                me.opponent(),
                me,
                nodes,
                cap,
                line,
                eval,
                track.as_deref_mut(),
                0,
            );
        } else {
            greedy_until_end(
                db,
                &mut s,
                me.opponent(),
                me,
                nodes,
                cap,
                line,
                eval,
                track.as_deref_mut(),
                false,
                0,
            );
        }
        let raw_g = eval.value(&s, me);
        let (g, g_end) = opp_next_cap_value(
            db,
            &s,
            me,
            raw_g,
            nodes,
            cap,
            line,
            eval,
            eval.onext,
            stats,
            PvEnd::OppReply,
        );
        let mut best_pv = None;
        if let Some(t) = track.as_deref_mut() {
            t.set_leaf(g, g_end, &s);
            best_pv = t.take_last();
        }
        let mut best = g;
        if eval.oopen > 0 && acting_player(state) == me.opponent() && *nodes < cap {
            stats.opp_open_replies += 1;
            if let Some(t) = track.as_deref_mut() {
                t.pop_to(plen0);
            }
            if let Some((ov, oline)) = opponent_open_search(
                db,
                state,
                me,
                nodes,
                cap,
                line,
                eval,
                obeam,
                eval.oopen,
                0,
                stats,
                track.as_deref_mut(),
            ) {
                if ov < g - 1e-5 {
                    stats.opp_open_lower += 1;
                }
                if ov < best {
                    best = ov;
                    if oline.is_some() {
                        best_pv = oline;
                    }
                }
            }
        }
        if best == -eval.wv {
            if let Some(t) = track.as_deref_mut() {
                t.restore_last(best_pv);
            }
            stats.opp_leaves += 1;
            return -eval.wv;
        }
        if *nodes >= cap {
            stats.opp_cap_hits += 1;
        }
        stats.opp_leaves += 1;
        if let Some(t) = track.as_deref_mut() {
            t.restore_last(best_pv);
        }
        return best;
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
        let (i, kept) = greedy_index(db, state, &legal, me, me, nodes, cap, line, eval, false);
        let plen = track.as_ref().map(|t| t.path_len()).unwrap_or(0);
        if let Some(t) = track.as_deref_mut() {
            t.push(legal[i].clone());
        }
        let Some(s) = take_or_apply(db, state, &legal[i], kept, nodes, cap, line) else {
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
        next_line.push(line_key(&s));
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
        if !useful_action(state, opp, a, eval.bpp1, eval.bpp2) {
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
        let k = line_key(&s);
        if matches!(a, Action::EndTurn) {
            end_turn = Some((s, k, a.clone()));
        } else {
            scored.push((eval.pick_value(&s, opp, me), s, k, a.clone()));
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
    search_me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    mut track: Option<&mut PvTracker>,
    skip_noop_fuse: bool,
    start_steps: u32,
) {
    let mut steps = start_steps;
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
        let (i, kept) = greedy_index(
            db,
            state,
            &legal,
            who,
            search_me,
            nodes,
            cap,
            &line,
            eval,
            skip_noop_fuse,
        );
        let Some(next) = take_or_apply(db, state, &legal[i], kept, nodes, cap, &line) else {
            break;
        };
        if let Some(t) = track.as_deref_mut() {
            t.push(legal[i].clone());
        }
        line.push(line_key(&next));
        *state = next;
        steps += 1;
    }
}

/// Greedy opponent reply with optional play→evolve (`omacro=1`).
#[allow(clippy::too_many_arguments)]
fn greedy_until_end_omacro(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    search_me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    mut track: Option<&mut PvTracker>,
    start_steps: u32,
) {
    let mut steps = start_steps;
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
        let pick = greedy_pick(
            db, state, &legal, who, search_me, nodes, cap, &line, eval, true,
        );
        let (i, macro_evo) = match pick {
            GreedyPick::Plain(i) => (i, None),
            GreedyPick::PlayMacro { index, evolve } => (index, Some(evolve)),
        };
        let Some(next) = try_apply(db, state, &legal[i], nodes, cap, &line) else {
            break;
        };
        if let Some(t) = track.as_deref_mut() {
            t.push(legal[i].clone());
        }
        line.push(line_key(&next));
        *state = next;
        if matches!(legal[i], Action::Play { .. }) {
            greedy_resolve_choices(db, state, who, search_me, nodes, cap, &mut line, eval);
        }
        if let Some(evo) = macro_evo {
            if let Some(next2) = try_apply(db, state, &evo, nodes, cap, &line) {
                if let Some(t) = track.as_deref_mut() {
                    t.push(evo);
                }
                line.push(line_key(&next2));
                *state = next2;
            }
        }
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
/// One shared apply budget per opponent-lethal sweep when any `okill` bit is
/// set. 300 is the worst-case envelope for the whole sweep (Ward prefix, core,
/// slot prefixes) — not per phase. Four meta-deck mirrors (2 games each, seed
/// 42) measured 21.1 applies/sweep mean (`okill=7`, max 278) vs 14.1 (`oevo=1`
/// only); bounded Ward unlock, core reserve, and two slot prefixes keep the
/// max at 300.
const OPP_LETHAL_KILL_APPLY_CAP: u32 = 300;
/// Standalone evolves tried to unlock a Ward attack in the prefix.
const WARD_PREFIX_UNLOCK_EVOS: usize = 2;
/// Plays tried (cheap first) when unlocking Ward attacks in the prefix.
const WARD_PREFIX_UNLOCK_PLAYS: usize = 4;
/// Slot-free prefix states tried after face chip (spec: at most two trades).
const SLOT_PREFIX_STATES: usize = 2;
/// Applies held back from the core sweep when slot-freeing may follow.
const SLOT_PREFIX_APPLY_RESERVE: u32 = 48;
const OKILL_WARD: u32 = 1;
const OKILL_SLOT: u32 = 2;
const OKILL_THROUGH: u32 = 4;
const OKILL_TWO: u32 = 8;
/// Apply budget for the two-play pass (`okill` bit 8), charged after the
/// existing sweep shapes miss. 200 is the worst-case envelope — not the mean.
const OPP_LETHAL_TWO_APPLY_CAP: u32 = 200;

#[derive(Clone, Copy, PartialEq, Eq)]
enum KillShape {
    Base,
    Ward,
    Slot,
    Through,
    Two,
}

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

fn face_potential(db: &CardDb, state: &State) -> i32 {
    let who = acting_player(state);
    leader_attackers(db, state)
        .iter()
        .map(|slot| state.field_inst(who, *slot).map(|f| f.attack).unwrap_or(0))
        .sum()
}

fn play_relevant(db: &CardDb, before: &State, after: &State, me: PlayerId) -> bool {
    face_potential(db, after) > face_potential(db, before)
        || after.player(me).leader_defense < before.player(me).leader_defense
}

fn has_leader_attack(db: &CardDb, state: &State) -> bool {
    legal_actions(db, state).iter().any(|a| {
        matches!(
            a,
            Action::Attack {
                target: AttackTarget::Leader,
                ..
            }
        )
    })
}

fn defender_ward_slots(state: &State, defender: PlayerId) -> Vec<u8> {
    state
        .player(defender)
        .field
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            f.as_ref()
                .filter(|c| c.kind == CardKind::Follower && c.defense > 0 && c.is_ward())
                .map(|_| i as u8)
        })
        .collect()
}

fn attacker_can_face_leader(state: &State, who: PlayerId, slot: u8) -> bool {
    let Some(inst) = state.field_inst(who, slot) else {
        return false;
    };
    if inst.traits.cant_attack_leader == Some(true) {
        return false;
    }
    inst.is_storm() || !inst.flags.summoning_sick
}

fn ward_attack_priority(state: &State, who: PlayerId, attack: &Action) -> (u8, i32, u8) {
    let Action::Attack { attacker, target } = attack else {
        return (9, 0, 0);
    };
    let Some(inst) = state.field_inst(who, attacker.0) else {
        return (9, 0, 0);
    };
    let cant_face = !attacker_can_face_leader(state, who, attacker.0);
    let tier = if cant_face { 0 } else { 1 };
    let ward_def = match target {
        AttackTarget::Slot(s) => state
            .field_inst(who.opponent(), s.0)
            .map(|f| f.defense)
            .unwrap_or(0),
        AttackTarget::Leader => 0,
    };
    let kills = inst.attack >= ward_def;
    let atk_key = if kills { inst.attack } else { i32::MAX };
    (tier, atk_key, attacker.0)
}

/// Apply every legal leader attack in slot order; stop only on a kill.
fn chip_face_line_state(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> State {
    let opp = me.opponent();
    let mut s = state.clone();
    let mut line = line.to_vec();
    loop {
        if s.winner == Some(opp) {
            return s;
        }
        if s.winner.is_some() || acting_player(&s) != opp || *nodes >= cap {
            return s;
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
            return s;
        };
        let Some(next) = try_apply(db, &s, a, nodes, cap, &line) else {
            return s;
        };
        line.push(line_key(&next));
        s = next;
    }
}

fn resolve_pending_choices(
    db: &CardDb,
    state: &State,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> Option<State> {
    let mut s = state.clone();
    let mut line = line.to_vec();
    loop {
        if !matches!(s.phase, Phase::Choice { .. }) {
            return Some(s);
        }
        if *nodes >= cap {
            return None;
        }
        let legal = legal_actions(db, &s);
        let chooses: Vec<Action> = legal
            .iter()
            .filter(|a| matches!(a, Action::Choose(_)))
            .cloned()
            .collect();
        if !chooses.is_empty() {
            let next = try_apply(db, &s, &chooses[0], nodes, cap, &line)?;
            line.push(line_key(&next));
            s = next;
            continue;
        }
        if let Some(a) = legal.iter().find(|a| matches!(a, Action::Confirm)) {
            let next = try_apply(db, &s, a, nodes, cap, &line)?;
            line.push(line_key(&next));
            s = next;
            continue;
        }
        return None;
    }
}

fn ordered_plays(db: &CardDb, state: &State) -> Vec<Action> {
    let mut plays: Vec<Action> = legal_actions(db, state)
        .into_iter()
        .filter(|a| matches!(a, Action::Play { .. }))
        .collect();
    plays.sort_by_key(|a| match a {
        Action::Play { hand } => {
            let cost = state
                .player(acting_player(state))
                .hand
                .get(*hand as usize)
                .and_then(|c| db.card(c.card).ok())
                .map(|card| card.cost())
                .unwrap_or(99);
            (cost, *hand)
        }
        _ => (99, 0),
    });
    plays
}

fn legal_ward_attacks(db: &CardDb, state: &State, ward_slot: u8) -> Vec<Action> {
    let target = AttackTarget::Slot(Slot(ward_slot));
    legal_actions(db, state)
        .into_iter()
        .filter(|a| matches!(a, Action::Attack { target: t, .. } if *t == target))
        .collect()
}

/// First legal face attack, then the next, until none remain. Attacks to the
/// leader commute, so one fixed order suffices. With `play_through`, once no
/// leader attack is legal each remaining follower attack is tried once (slot
/// order), re-checking for a kill after each.
#[allow(clippy::too_many_arguments)]
fn face_line_kills(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    play_through: bool,
    through_used: &mut bool,
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
            break;
        };
        let Some(next) = try_apply(db, &s, a, nodes, cap, &line) else {
            return false;
        };
        line.push(line_key(&next));
        s = next;
    }
    if !play_through {
        return false;
    }
    for slot in 0..FIELD_SIZE as u8 {
        if s.winner == Some(opp) {
            *through_used = true;
            return true;
        }
        if s.winner.is_some() || matches!(s.phase, Phase::Terminal) {
            return false;
        }
        if acting_player(&s) != opp || *nodes >= cap {
            return false;
        }
        let legal = legal_actions(db, &s);
        let target = AttackTarget::Slot(Slot(slot));
        let Some(a) = legal.iter().find(|a| {
            matches!(
                a,
                Action::Attack {
                    attacker: _,
                    target: t,
                } if *t == target
            )
        }) else {
            continue;
        };
        let Some(next) = try_apply(db, &s, a, nodes, cap, &line) else {
            continue;
        };
        line.push(line_key(&next));
        s = next;
        if face_line_kills(db, &s, me, nodes, cap, &line, false, through_used) {
            *through_used = true;
            return true;
        }
        if s.winner == Some(opp) {
            *through_used = true;
            return true;
        }
    }
    false
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
    play_through: bool,
    evo_found: &mut bool,
    through_used: &mut bool,
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
            let Some(s) = try_apply_legal(db, state, a, nodes, cap, line) else {
                continue;
            };
            let mut next_line = line.to_vec();
            next_line.push(line_key(&s));
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
                play_through,
                evo_found,
                through_used,
            ) {
                return true;
            }
        }
        if chooses.is_empty() {
            if let Some(a) = legal.iter().find(|a| matches!(a, Action::Confirm)) {
                let Some(s) = try_apply_legal(db, state, a, nodes, cap, line) else {
                    return false;
                };
                let mut next_line = line.to_vec();
                next_line.push(line_key(&s));
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
                    play_through,
                    evo_found,
                    through_used,
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
        if play_through && oevo {
            let just_played: Vec<u8> = now_faces
                .iter()
                .copied()
                .filter(|slot| !origin_faces.contains(slot))
                .collect();
            if evolve_then_face(
                db,
                state,
                me,
                nodes,
                cap,
                line,
                &just_played,
                &now_faces,
                play_through,
                through_used,
            ) {
                *evo_found = true;
                *through_used = true;
                return true;
            }
        }
        return false;
    }
    if face_line_kills(db, state, me, nodes, cap, line, play_through, through_used) {
        return true;
    }
    if oevo {
        let just_played: Vec<u8> = now_faces
            .iter()
            .copied()
            .filter(|slot| !origin_faces.contains(slot))
            .collect();
        if evolve_then_face(
            db,
            state,
            me,
            nodes,
            cap,
            line,
            &just_played,
            &now_faces,
            play_through,
            through_used,
        ) {
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
    play_through: bool,
    through_used: &mut bool,
) -> bool {
    for a in ordered_evolves(db, state, prefer, faces) {
        if *nodes >= cap {
            return false;
        }
        let Some(s) = try_apply(db, state, &a, nodes, cap, line) else {
            continue;
        };
        let mut next_line = line.to_vec();
        next_line.push(line_key(&s));
        if face_line_kills(
            db,
            &s,
            me,
            nodes,
            cap,
            &next_line,
            play_through,
            through_used,
        ) {
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
        next_line.push(line_key(&s));
        let mut dummy_through = false;
        if face_line_kills(
            db,
            &s,
            me,
            nodes,
            cap,
            &next_line,
            false,
            &mut dummy_through,
        ) {
            return true;
        }
    }
    false
}

/// Resolve the first play's pending choices, then try relevant second plays.
#[allow(clippy::too_many_arguments)]
fn opp_lethal_two_after_first(
    db: &CardDb,
    before_first: &State,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    two_end: u32,
    line: &[u64],
    oevo: bool,
    play_through: bool,
    evo_found: &mut bool,
    through_used: &mut bool,
) -> bool {
    if matches!(state.phase, Phase::Choice { .. }) {
        let legal = legal_actions(db, state);
        let chooses: Vec<Action> = legal
            .iter()
            .filter(|a| matches!(a, Action::Choose(_)))
            .cloned()
            .collect();
        for a in &chooses {
            if *nodes >= two_end {
                return false;
            }
            let Some(s) = try_apply_legal(db, state, a, nodes, two_end, line) else {
                continue;
            };
            let mut next_line = line.to_vec();
            next_line.push(line_key(&s));
            if opp_lethal_two_after_first(
                db,
                before_first,
                &s,
                me,
                nodes,
                two_end,
                &next_line,
                oevo,
                play_through,
                evo_found,
                through_used,
            ) {
                return true;
            }
        }
        if chooses.is_empty() {
            if let Some(a) = legal.iter().find(|a| matches!(a, Action::Confirm)) {
                let Some(s) = try_apply_legal(db, state, a, nodes, two_end, line) else {
                    return false;
                };
                let mut next_line = line.to_vec();
                next_line.push(line_key(&s));
                return opp_lethal_two_after_first(
                    db,
                    before_first,
                    &s,
                    me,
                    nodes,
                    two_end,
                    &next_line,
                    oevo,
                    play_through,
                    evo_found,
                    through_used,
                );
            }
        }
        return false;
    }
    if !play_relevant(db, before_first, state, me) {
        return false;
    }
    for second in ordered_plays(db, state) {
        if *nodes >= two_end {
            return false;
        }
        let Some(after_second) = try_apply_legal(db, state, &second, nodes, two_end, line) else {
            continue;
        };
        // Second plays that open a choice are not "relevant" until the choice resolves
        // (e.g. mode that deals face damage). resolve_play_line handles that path.
        if !matches!(after_second.phase, Phase::Choice { .. })
            && !play_relevant(db, state, &after_second, me)
        {
            continue;
        }
        let mut line2 = line.to_vec();
        line2.push(line_key(&after_second));
        let origin_def = state.player(me).leader_defense;
        let origin_faces = leader_attackers(db, state);
        if resolve_play_line(
            db,
            &after_second,
            me,
            origin_def,
            &origin_faces,
            nodes,
            two_end,
            &line2,
            oevo,
            play_through,
            evo_found,
            through_used,
        ) {
            return true;
        }
    }
    false
}

/// After the existing sweep shapes miss: try each relevant first `Play` (with
/// its `Choose`/`Confirm` resolved), then each relevant second `Play`, then
/// [`resolve_play_line`] on the face line.
#[allow(clippy::too_many_arguments)]
fn opp_lethal_two_pass(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    oevo: bool,
    play_through: bool,
    evo_found: &mut bool,
    through_used: &mut bool,
) -> bool {
    let two_end = (*nodes).saturating_add(OPP_LETHAL_TWO_APPLY_CAP).min(cap);
    for first in ordered_plays(db, state) {
        if *nodes >= two_end {
            break;
        }
        let Some(after_first) = try_apply_legal(db, state, &first, nodes, two_end, line) else {
            continue;
        };
        let mut line1 = line.to_vec();
        line1.push(line_key(&after_first));
        if opp_lethal_two_after_first(
            db,
            state,
            &after_first,
            me,
            nodes,
            two_end,
            &line1,
            oevo,
            play_through,
            evo_found,
            through_used,
        ) {
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn opp_lethal_sweep_core(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    sweep_cap: u32,
    line: &[u64],
    oevo: bool,
    play_through: bool,
    evo_found: &mut bool,
    through_used: &mut bool,
) -> bool {
    if face_line_kills(
        db,
        state,
        me,
        nodes,
        sweep_cap,
        line,
        play_through,
        through_used,
    ) {
        return true;
    }
    let origin_def = state.player(me).leader_defense;
    let origin_faces = leader_attackers(db, state);
    let legal = legal_actions(db, state);
    for a in &legal {
        if *nodes >= sweep_cap {
            break;
        }
        if !matches!(a, Action::Play { .. }) {
            continue;
        }
        let Some(s) = try_apply_legal(db, state, a, nodes, sweep_cap, line) else {
            continue;
        };
        let mut next_line = line.to_vec();
        next_line.push(line_key(&s));
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
            play_through,
            evo_found,
            through_used,
        ) {
            return true;
        }
    }
    if oevo && standalone_evolve_kills(db, state, me, nodes, sweep_cap, line) {
        *evo_found = true;
        return true;
    }
    false
}

fn try_ward_prefix(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> Option<State> {
    let defender = me;
    let attacker = me.opponent();
    if defender_ward_slots(state, defender).is_empty() || has_leader_attack(db, state) {
        return None;
    }
    let mut s = state.clone();
    let mut line = line.to_vec();
    let mut alt_budget = 3usize;
    while !defender_ward_slots(&s, defender).is_empty() {
        if *nodes >= cap {
            return None;
        }
        if acting_player(&s) != attacker {
            return None;
        }
        let ward_slot = defender_ward_slots(&s, defender)[0];
        let mut attacks = legal_ward_attacks(db, &s, ward_slot);
        if attacks.is_empty() {
            let faces = leader_attackers(db, &s);
            let mut unlocked = false;
            for a in ordered_evolves(db, &s, &[], &faces)
                .into_iter()
                .take(WARD_PREFIX_UNLOCK_EVOS)
            {
                if *nodes >= cap {
                    return None;
                }
                let Some(next) = try_apply(db, &s, &a, nodes, cap, &line) else {
                    continue;
                };
                let mut nl = line.clone();
                nl.push(line_key(&next));
                attacks = legal_ward_attacks(db, &next, ward_slot);
                if !attacks.is_empty() {
                    s = next;
                    line = nl;
                    unlocked = true;
                    break;
                }
            }
            if !unlocked {
                for play in ordered_plays(db, &s)
                    .into_iter()
                    .take(WARD_PREFIX_UNLOCK_PLAYS)
                {
                    if *nodes >= cap {
                        break;
                    }
                    let Some(next) = try_apply(db, &s, &play, nodes, cap, &line) else {
                        continue;
                    };
                    let mut nl = line.clone();
                    nl.push(line_key(&next));
                    let Some(played) = resolve_pending_choices(db, &next, nodes, cap, &nl) else {
                        continue;
                    };
                    nl.push(line_key(&played));
                    if !legal_ward_attacks(db, &played, ward_slot).is_empty() {
                        s = played;
                        line = nl;
                        unlocked = true;
                        break;
                    }
                    let prefer = new_follower_slots(&s, &played, attacker);
                    for evo in ordered_evolves(db, &played, &prefer, &leader_attackers(db, &played))
                    {
                        let Action::Evolve { slot, .. } = &evo else {
                            continue;
                        };
                        if !prefer.contains(&slot.0) {
                            continue;
                        }
                        if *nodes >= cap {
                            break;
                        }
                        let Some(after) = try_apply(db, &played, &evo, nodes, cap, &nl) else {
                            continue;
                        };
                        let mut nl2 = nl.clone();
                        nl2.push(line_key(&after));
                        attacks = legal_ward_attacks(db, &after, ward_slot);
                        if !attacks.is_empty() {
                            s = after;
                            line = nl2;
                            unlocked = true;
                            break;
                        }
                        break;
                    }
                    if unlocked {
                        break;
                    }
                }
            }
            if !unlocked {
                return None;
            }
        }
        attacks.sort_by_key(|a| ward_attack_priority(&s, attacker, a));
        let mut cleared = false;
        for a in attacks.iter().take(alt_budget.max(1)) {
            let mut trial = s.clone();
            let mut trial_line = line.clone();
            let Some(next) = try_apply(db, &trial, a, nodes, cap, &trial_line) else {
                continue;
            };
            trial_line.push(line_key(&next));
            trial = next;
            let ward_gone = defender_ward_slots(&trial, defender)
                .iter()
                .all(|slot| *slot != ward_slot);
            s = trial;
            line = trial_line;
            cleared = true;
            alt_budget = alt_budget.saturating_sub(1);
            if ward_gone {
                break;
            }
        }
        if !cleared {
            return None;
        }
    }
    Some(s)
}

fn follower_attacks_from_slot(db: &CardDb, state: &State, from_slot: u8) -> Vec<Action> {
    legal_actions(db, state)
        .into_iter()
        .filter(|a| {
            matches!(
                a,
                Action::Attack {
                    attacker: Slot(s),
                    target: AttackTarget::Slot(_),
                } if *s == from_slot
            )
        })
        .collect()
}

fn slot_free_prefix_states(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
) -> Vec<(State, Vec<u64>)> {
    let attacker = me.opponent();
    if state.player(attacker).field_count() < FIELD_SIZE {
        return vec![(state.clone(), line.to_vec())];
    }
    let mut out = Vec::new();
    let chipped = chip_face_line_state(db, state, me, nodes, cap, line);
    for base in [state, &chipped] {
        if base.player(attacker).field_count() < FIELD_SIZE {
            out.push((base.clone(), line.to_vec()));
            continue;
        }
        let trades = sacrifice_trades(db, base, attacker, me);
        for trade in trades.iter().take(4) {
            if *nodes >= cap {
                break;
            }
            let Action::Attack {
                attacker: Slot(slot),
                ..
            } = trade
            else {
                continue;
            };
            let Some(s1) = try_apply(db, base, trade, nodes, cap, line) else {
                continue;
            };
            let mut l1 = line.to_vec();
            l1.push(line_key(&s1));
            if s1.player(attacker).field_count() < FIELD_SIZE {
                out.push((s1, l1));
                continue;
            }
            for a2 in follower_attacks_from_slot(db, &s1, *slot) {
                if *nodes >= cap {
                    break;
                }
                let Some(s2) = try_apply(db, &s1, &a2, nodes, cap, &l1) else {
                    continue;
                };
                let mut l2 = l1.clone();
                l2.push(line_key(&s2));
                if s2.player(attacker).field_count() < FIELD_SIZE {
                    out.push((s2, l2));
                }
            }
        }
    }
    out.truncate(SLOT_PREFIX_STATES);
    out
}

fn sacrifice_trades(
    db: &CardDb,
    state: &State,
    attacker: PlayerId,
    defender: PlayerId,
) -> Vec<Action> {
    let mut out = Vec::new();
    for a in legal_actions(db, state) {
        let Action::Attack {
            attacker: slot,
            target: AttackTarget::Slot(def_slot),
        } = a
        else {
            continue;
        };
        let Some(att) = state.field_inst(attacker, slot.0) else {
            continue;
        };
        let Some(def) = state.field_inst(defender, def_slot.0) else {
            continue;
        };
        if def.attack < att.defense {
            continue;
        }
        out.push(a);
    }
    out.sort_by_key(|a| {
        let Action::Attack { attacker: slot, .. } = a else {
            return (i32::MAX, 0u8);
        };
        let inst = state.field_inst(attacker, slot.0).unwrap();
        (inst.attack + inst.defense, slot.0)
    });
    out
}

/// Glance-level opponent lethal: face attacks, then each `Play` (plus its
/// `Choose`/`Confirm`) and a face line when that play opened one. With
/// `oevo=1` (the default), a play that opened a line but missed face then
/// tries each legal evolve (just-played slot first), and a standalone
/// evolve pass runs if those also miss. With `okill` bits, Ward-break
/// prefixes, slot-freeing trades, and play-through follower lines are
/// tried before giving up. Caps at [`OPP_LETHAL_APPLY_CAP`] (`oevo=0`),
/// [`OPP_LETHAL_EVO_APPLY_CAP`] (`oevo=1`), or [`OPP_LETHAL_KILL_APPLY_CAP`]
/// (any `okill` bit), all charged to `nodes`.
#[allow(clippy::too_many_arguments)]
fn opp_lethal_sweep(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    nodes: &mut u32,
    cap: u32,
    line: &[u64],
    eval: Evaluator<'_>,
    stats: &mut SearchStats,
) -> bool {
    stats.opp_lethal_checks += 1;
    let start = *nodes;
    let okill = eval.okill;
    let oevo = eval.oevo;
    let play_through = okill & OKILL_THROUGH != 0;
    let apply_limit = if okill != 0 {
        OPP_LETHAL_KILL_APPLY_CAP
    } else if oevo {
        OPP_LETHAL_EVO_APPLY_CAP
    } else {
        OPP_LETHAL_APPLY_CAP
    };
    let mut evo_found = false;
    let mut through_used = false;
    let mut shape = KillShape::Base;
    let mut found = false;

    let sweep_end = (*nodes).saturating_add(apply_limit).min(cap);
    let slot_pending =
        okill & OKILL_SLOT != 0 && state.player(me.opponent()).field_count() >= FIELD_SIZE;
    let core_end = if slot_pending {
        sweep_end.saturating_sub(SLOT_PREFIX_APPLY_RESERVE.min(apply_limit))
    } else {
        sweep_end
    };

    if okill & OKILL_WARD != 0
        && !defender_ward_slots(state, me).is_empty()
        && !has_leader_attack(db, state)
    {
        if let Some(cleared) = try_ward_prefix(db, state, me, nodes, core_end, line) {
            let mut next_line = line.to_vec();
            next_line.push(line_key(&cleared));
            if opp_lethal_sweep_core(
                db,
                &cleared,
                me,
                nodes,
                core_end,
                &next_line,
                oevo,
                play_through,
                &mut evo_found,
                &mut through_used,
            ) {
                found = true;
                shape = KillShape::Ward;
            }
        }
    }

    if !found {
        found = opp_lethal_sweep_core(
            db,
            state,
            me,
            nodes,
            core_end,
            line,
            oevo,
            play_through,
            &mut evo_found,
            &mut through_used,
        );
    }

    if !found && slot_pending {
        for (s, next_line) in slot_free_prefix_states(db, state, me, nodes, sweep_end, line) {
            if opp_lethal_sweep_core(
                db,
                &s,
                me,
                nodes,
                sweep_end,
                &next_line,
                oevo,
                play_through,
                &mut evo_found,
                &mut through_used,
            ) {
                found = true;
                shape = KillShape::Slot;
                break;
            }
        }
    }

    if found && play_through && shape == KillShape::Base && through_used {
        shape = KillShape::Through;
    }

    if !found
        && okill & OKILL_TWO != 0
        && opp_lethal_two_pass(
            db,
            state,
            me,
            nodes,
            cap,
            line,
            oevo,
            play_through,
            &mut evo_found,
            &mut through_used,
        )
    {
        found = true;
        shape = KillShape::Two;
    }

    let sweep_applies = *nodes - start;
    stats.opp_lethal_nodes += u64::from(sweep_applies);
    stats.opp_lethal_applies_max = stats.opp_lethal_applies_max.max(u64::from(sweep_applies));
    if found {
        stats.opp_lethal_found += 1;
        if evo_found {
            stats.opp_lethal_evo_found += 1;
        }
        match shape {
            KillShape::Ward => stats.opp_lethal_ward_found += 1,
            KillShape::Slot => stats.opp_lethal_slot_found += 1,
            KillShape::Through => stats.opp_lethal_through_found += 1,
            KillShape::Two => stats.opp_lethal_two_found += 1,
            KillShape::Base => {}
        }
    }
    found
}

const HB_MAX_DEPTH: u32 = 3;
const HB_REROLLS: u32 = 4;
const HB_REMOVAL_EPS: f32 = 1e-4;

#[derive(Clone, Debug)]
struct HoldbackBranch {
    plain: f32,
    removal: f32,
    value: f32,
    removable: bool,
    line_len: u32,
}

#[derive(Clone)]
struct RemovalCandidate {
    line: Vec<Action>,
    opp_value: f32,
    kills_leader: bool,
}

fn is_kill_attack(db: &CardDb, state: &State, a: &Action) -> bool {
    let Action::Attack {
        target: AttackTarget::Slot(target),
        ..
    } = a
    else {
        return false;
    };
    let me = acting_player(state);
    let opp = me.opponent();
    let Some(target_inst) = state.field_inst(opp, target.0) else {
        return false;
    };
    let target_id = target_inst.id;
    let mut s = state.clone();
    if apply(db, &mut s, a.clone()).is_err() {
        return false;
    }
    s.find_field(opp, target_id).is_none()
}

fn attack_attacker_id(state: &State, me: PlayerId, a: &Action) -> Option<u32> {
    let Action::Attack { attacker, .. } = a else {
        return None;
    };
    state.field_inst(me, attacker.0).map(|f| f.id)
}

fn collect_attacker_ids(state: &State, me: PlayerId, attacks: &[&Action]) -> Vec<u32> {
    let mut ids = Vec::new();
    for a in attacks {
        if let Some(id) = attack_attacker_id(state, me, a) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids
}

fn filter_x_ids_on_field(state: &State, me: PlayerId, x_ids: &[u32]) -> Vec<u32> {
    x_ids
        .iter()
        .copied()
        .filter(|&id| state.find_field(me, id).is_some())
        .collect()
}

fn hb_x_removed(state: &State, me: PlayerId, x_ids: &[u32]) -> bool {
    x_ids.iter().any(|id| state.find_field(me, *id).is_none())
}

fn confirm_holdback_removal(
    db: &CardDb,
    turn_start: &State,
    me: PlayerId,
    x_ids: &[u32],
    line: &[Action],
    nodes: &mut u32,
    budget: u32,
) -> bool {
    let pos_key = search_key(turn_start);
    let seeds: Vec<u64> = (0..HB_REROLLS)
        .map(|i| roll_confirm_seed(pos_key, i))
        .collect();
    for &seed in &seeds {
        let mut s = turn_start.clone();
        s.reseed(seed);
        for a in line {
            if *nodes >= budget {
                return false;
            }
            *nodes += 1;
            if apply(db, &mut s, a.clone()).is_err() {
                return false;
            }
        }
        if !hb_x_removed(&s, me, x_ids) {
            return false;
        }
    }
    true
}

fn try_apply_hb(
    db: &CardDb,
    state: &State,
    a: &Action,
    nodes: &mut u32,
    budget: u32,
    line: &[u64],
) -> Option<State> {
    if *nodes >= budget {
        return None;
    }
    let mut s = state.clone();
    *nodes += 1;
    if apply(db, &mut s, a.clone()).is_err() {
        return None;
    }
    if can_repeat(&s) && line.contains(&line_key(&s)) {
        return None;
    }
    Some(s)
}

fn removal_better(a: &RemovalCandidate, b: &RemovalCandidate) -> bool {
    if a.kills_leader != b.kills_leader {
        return a.kills_leader;
    }
    a.opp_value > b.opp_value
}

fn holdback_removal_search(
    db: &CardDb,
    turn_start: &State,
    me: PlayerId,
    x_ids: &[u32],
    budget: u32,
    eval: Evaluator<'_>,
) -> (Option<RemovalCandidate>, u32, bool) {
    let mut best: Option<RemovalCandidate> = None;
    let mut nodes = 0u32;
    let mut tt: HashMap<u64, u32> = HashMap::new();
    for max_depth in 1..=HB_MAX_DEPTH {
        if nodes >= budget {
            break;
        }
        let mut path_keys: Vec<u64> = vec![line_key(turn_start)];
        let mut line: Vec<Action> = Vec::new();
        search_holdback_removal(
            db,
            turn_start,
            turn_start,
            me,
            x_ids,
            budget,
            max_depth,
            &mut nodes,
            &mut tt,
            &mut path_keys,
            &mut line,
            eval,
            &mut best,
        );
    }
    let unknown = nodes >= budget && best.is_none();
    (best, nodes, unknown)
}

#[allow(clippy::too_many_arguments)]
fn search_holdback_removal(
    db: &CardDb,
    turn_start: &State,
    state: &State,
    me: PlayerId,
    x_ids: &[u32],
    budget: u32,
    max_depth: u32,
    nodes: &mut u32,
    tt: &mut HashMap<u64, u32>,
    path_keys: &mut Vec<u64>,
    line: &mut Vec<Action>,
    eval: Evaluator<'_>,
    best: &mut Option<RemovalCandidate>,
) {
    let opp = me.opponent();
    if hb_x_removed(state, me, x_ids) {
        if confirm_holdback_removal(db, turn_start, me, x_ids, line, nodes, budget) {
            let kills_leader = state.winner == Some(opp);
            let opp_value = if kills_leader {
                eval.wv
            } else {
                eval.value(state, opp)
            };
            let cand = RemovalCandidate {
                line: line.clone(),
                opp_value,
                kills_leader,
            };
            match best {
                None => *best = Some(cand),
                Some(b) if removal_better(&cand, b) => *best = Some(cand),
                _ => {}
            }
        }
        return;
    }
    if line.len() as u32 >= max_depth {
        return;
    }
    if *nodes >= budget {
        return;
    }
    let remaining = max_depth - line.len() as u32;
    let key = search_key_id(state);
    if tt.get(&key).is_some_and(|&d| d >= remaining) {
        return;
    }
    tt.insert(key, remaining);

    if acting_player(state) != opp
        || matches!(
            state.phase,
            Phase::Terminal | Phase::Mulligan { .. } | Phase::End
        )
    {
        return;
    }

    let legal = legal_actions(db, state);
    for a in legal {
        if matches!(a, Action::EndTurn | Action::MulliganConfirm { .. }) {
            continue;
        }
        let Some(next) = try_apply_hb(db, state, &a, nodes, budget, path_keys) else {
            continue;
        };
        line.push(a.clone());
        path_keys.push(line_key(&next));
        search_holdback_removal(
            db, turn_start, &next, me, x_ids, budget, max_depth, nodes, tt, path_keys, line, eval,
            best,
        );
        line.pop();
        path_keys.pop();
        if *nodes >= budget {
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn holdback_leaf_value(
    db: &CardDb,
    root: &State,
    turn_start: &State,
    me: PlayerId,
    eval: Evaluator<'_>,
    horizon: u32,
    hres: u32,
    odepth: u32,
    obeam: usize,
    dec_stats: &mut SearchStats,
) -> f32 {
    if turn_start.winner == Some(me.opponent()) {
        return -eval.wv;
    }
    if turn_start.winner == Some(me) {
        return eval.wv;
    }
    let root_key = line_key(root);
    let mut nodes = 0u32;
    let cap = u32::MAX;
    let line = vec![root_key, line_key(turn_start)];
    let mut hb_stats = SearchStats::default();
    let v = if horizon >= 1 {
        horizon_score_leaf(
            db,
            turn_start,
            me,
            &mut nodes,
            cap,
            hres,
            horizon,
            &line,
            eval,
            odepth,
            obeam,
            &mut hb_stats,
            None,
            HorizonCutoff::Depth,
        )
    } else {
        opponent_reply(
            db,
            turn_start,
            me,
            &mut nodes,
            cap,
            &line,
            eval,
            odepth,
            obeam,
            &mut hb_stats,
            None,
            None,
        )
    };
    dec_stats.horizon_nodes += hb_stats.horizon_nodes;
    dec_stats.horizon_leaves += hb_stats.horizon_leaves;
    finite(v, eval.wv)
}

fn all_cands_lost(n: &[u32], acc: &[f32], worst: &[f32], pess: f32, wv: f32) -> bool {
    const LETHAL_EPS: f32 = 1e-3;
    let threshold = -wv + LETHAL_EPS;
    let mut any = false;
    for (j, &c) in n.iter().enumerate() {
        if c == 0 {
            continue;
        }
        any = true;
        if root_agg(acc[j], c, worst[j], pess) > threshold {
            return false;
        }
    }
    any
}

#[allow(clippy::too_many_arguments)]
fn try_lost_rerank(
    lostrank: u32,
    pess: f32,
    horizon: u32,
    hres: u32,
    db: &CardDb,
    roots: &[State],
    subset: &[Action],
    me: PlayerId,
    eval: Evaluator<'_>,
    odepth: u32,
    obeam: usize,
    fusemacro: bool,
    depth: u32,
    beam: usize,
    stats: &mut SearchStats,
) -> Option<(usize, f32, u32, Vec<f32>)> {
    let eval_lost = Evaluator {
        olethal: false,
        olsolve: 0,
        opp_reply: false,
        ..eval
    };
    let mut lost_nodes = 0u32;
    let mut acc = vec![0.0f32; subset.len()];
    let mut n_acc = vec![0u32; subset.len()];
    let mut worst = vec![f32::INFINITY; subset.len()];
    for (r, root) in roots.iter().enumerate() {
        let root_key = line_key(root);
        for (j, a) in subset.iter().enumerate() {
            if lost_nodes >= lostrank {
                break;
            }
            let pairs_left = (roots.len() - r) * subset.len() - j;
            let remaining = lostrank - lost_nodes;
            let even = remaining / pairs_left as u32;
            const MIN_SHARE: u32 = 24;
            let share = if remaining >= MIN_SHARE.saturating_mul(pairs_left as u32) {
                even.max(MIN_SHARE)
            } else {
                even.max(1)
            };
            let cap = lost_nodes.saturating_add(share).min(lostrank);
            let pair_start = lost_nodes;
            let Some(s) = try_apply(db, root, a, &mut lost_nodes, cap, &[root_key]) else {
                continue;
            };
            let at_fuse_choice = fusemacro && own_fuse_partners(&s, me).is_some();
            let search_depth = if at_fuse_choice {
                depth
            } else {
                depth.saturating_sub(1)
            };
            let line = vec![root_key, line_key(&s)];
            let v = if s.winner == Some(me) {
                eval_lost.wv
            } else {
                search_own(
                    db,
                    &s,
                    me,
                    search_depth,
                    beam,
                    &mut lost_nodes,
                    cap,
                    &line,
                    eval_lost,
                    odepth,
                    obeam,
                    stats,
                    None,
                    None,
                    fusemacro,
                    horizon,
                    hres,
                )
            };
            let _ = pair_start;
            let fv = finite(v, eval_lost.wv);
            acc[j] += fv;
            if fv < worst[j] {
                worst[j] = fv;
            }
            n_acc[j] += 1;
        }
    }
    let mut best_i = 0usize;
    let mut best_v = f32::NEG_INFINITY;
    let mut aggregates = vec![f32::NEG_INFINITY; subset.len()];
    let mut any = false;
    for (j, &c) in n_acc.iter().enumerate() {
        if c == 0 {
            continue;
        }
        any = true;
        let v = root_agg(acc[j], c, worst[j], pess);
        aggregates[j] = v;
        if v > best_v {
            best_v = v;
            best_i = j;
        }
    }
    if !any {
        return None;
    }
    Some((best_i, finite(best_v, eval.wv), lost_nodes, aggregates))
}

fn holdback_branch(plain: f32, removal_plain: f32, line_len: u32) -> HoldbackBranch {
    let removable = removal_plain + HB_REMOVAL_EPS < plain;
    let removal = if removable { removal_plain } else { plain };
    HoldbackBranch {
        plain,
        removal,
        value: plain.min(removal),
        removable,
        line_len,
    }
}

#[allow(clippy::too_many_arguments)]
fn holdback_symmetric_v(
    db: &CardDb,
    root: &State,
    turn_start: &State,
    me: PlayerId,
    x_ids: &[u32],
    hbcheck: u32,
    eval: Evaluator<'_>,
    horizon: u32,
    hres: u32,
    odepth: u32,
    obeam: usize,
    dec_stats: &mut SearchStats,
) -> HoldbackBranch {
    let plain = holdback_leaf_value(
        db, root, turn_start, me, eval, horizon, hres, odepth, obeam, dec_stats,
    );
    if x_ids.is_empty() {
        return holdback_branch(plain, plain, 0);
    }
    let (removal, nodes, unknown) =
        holdback_removal_search(db, turn_start, me, x_ids, hbcheck, eval);
    dec_stats.hb_nodes += u64::from(nodes);
    dec_stats.hb_nodes_max = dec_stats.hb_nodes_max.max(u64::from(nodes));
    if unknown {
        dec_stats.hb_unknown += 1;
    }
    let (removal_plain, line_len) = match removal {
        Some(rem) => {
            let mut after = turn_start.clone();
            for a in &rem.line {
                if apply(db, &mut after, a.clone()).is_err() {
                    return holdback_branch(plain, plain, 0);
                }
            }
            if after.winner == Some(me.opponent()) {
                (-eval.wv, rem.line.len() as u32)
            } else {
                (
                    holdback_leaf_value(
                        db, root, &after, me, eval, horizon, hres, odepth, obeam, dec_stats,
                    ),
                    rem.line.len() as u32,
                )
            }
        }
        None => (plain, 0),
    };
    holdback_branch(plain, removal_plain, line_len)
}

fn holdback_apply_action(
    db: &CardDb,
    state: &State,
    action: &Action,
    me: PlayerId,
    eval: Evaluator<'_>,
) -> Option<State> {
    let mut s = state.clone();
    if apply(db, &mut s, action.clone()).is_err() {
        return None;
    }
    let mut nodes = 0u32;
    let cap = u32::MAX;
    let mut line = vec![line_key(state), line_key(&s)];
    if !greedy_resolve_choices(db, &mut s, me, me, &mut nodes, cap, &mut line, eval) {
        return None;
    }
    Some(s)
}

fn holdback_bot_finish(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    osteps: u32,
    eval: Evaluator<'_>,
) -> bool {
    let mut nodes = 0u32;
    let cap = u32::MAX;
    let mut line = vec![line_key(state)];
    let mut steps = 0u32;
    while bot_turn_still_active(state, me) && steps < osteps {
        if !greedy_resolve_choices(db, state, me, me, &mut nodes, cap, &mut line, eval) {
            return false;
        }
        if !matches!(state.phase, Phase::Main | Phase::Combat) {
            break;
        }
        let legal = legal_actions(db, state);
        if legal.is_empty() {
            break;
        }
        let (i, kept) = greedy_index(
            db,
            state,
            &legal,
            me,
            me,
            &mut nodes,
            cap,
            &line,
            eval,
            eval.fuseguard,
        );
        let a = legal[i].clone();
        let ended = matches!(a, Action::EndTurn);
        match kept {
            Some(s) => *state = s,
            None => {
                if apply(db, state, a).is_err() {
                    return false;
                }
            }
        }
        line.push(line_key(state));
        if ended {
            return true;
        }
        steps += 1;
    }
    if !bot_turn_still_active(state, me) {
        return true;
    }
    if !matches!(state.phase, Phase::Main | Phase::Combat) {
        return false;
    }
    apply(db, state, Action::EndTurn).is_ok()
}

fn holdback_after_attack_finish_end(
    db: &CardDb,
    root: &State,
    me: PlayerId,
    attack: &Action,
    osteps: u32,
    eval: Evaluator<'_>,
) -> Option<State> {
    let mut s = holdback_apply_action(db, root, attack, me, eval)?;
    if !holdback_bot_finish(db, &mut s, me, osteps, eval) {
        return None;
    }
    if acting_player(&s) == me {
        return None;
    }
    Some(s)
}

fn holdback_world_record(r: u32, branch: &HoldbackBranch) -> HoldbackBranchWorldRecord {
    HoldbackBranchWorldRecord {
        r,
        plain: branch.plain,
        removal: branch.removal,
        value: branch.value,
        removable: branch.removable,
        line_len: branch.line_len,
    }
}

fn holdback_aggregate(branches: &[HoldbackBranch], pess: f32) -> Option<f32> {
    if branches.is_empty() {
        return None;
    }
    let n = branches.len() as u32;
    let acc = branches.iter().map(|b| b.value).sum();
    let worst = branches
        .iter()
        .map(|b| b.value)
        .fold(f32::INFINITY, f32::min);
    Some(root_agg(acc, n, worst, pess))
}

const HBK_WORLD_XOR: u64 = 0x485b_4b20_5e5e_4b4b;
const HBK_DEAL_XOR: u64 = 0x7f4a_7c15_9e37_79b9;

fn hbk_pair_share(node_cap: u32, k: u32, candidates: usize) -> u32 {
    const MIN_SHARE: u32 = 24;
    let total_pairs = (k as usize) * candidates;
    if total_pairs == 0 {
        return 1;
    }
    let even = node_cap / total_pairs as u32;
    if node_cap >= MIN_SHARE.saturating_mul(total_pairs as u32) {
        even.max(MIN_SHARE)
    } else {
        even.max(1)
    }
}

fn hbk_fresh_seeds_and_deal(
    root_world_seeds: &[u64],
    hbk: u32,
    block: bool,
    turn_stable: bool,
    wbase: Option<u64>,
) -> (Vec<u64>, Option<u64>) {
    let fresh: Vec<u64> = (0..hbk)
        .map(|i| {
            let base = root_world_seeds
                .get(i as usize % root_world_seeds.len())
                .copied()
                .unwrap_or(0);
            let mut rng = Xoshiro256ss::from_seed(base ^ HBK_WORLD_XOR ^ i as u64);
            rng.next_u64()
        })
        .collect();
    let deal = if block {
        if turn_stable {
            let base = wbase.expect("turn-stable block deal") ^ HBK_DEAL_XOR;
            let mut deal_rng = Xoshiro256ss::from_seed(base);
            for _ in 0..hbk {
                deal_rng.next_u64();
            }
            Some(deal_rng.next_u64())
        } else {
            let mut deal_rng = Xoshiro256ss::from_seed(
                root_world_seeds.first().copied().unwrap_or(0) ^ HBK_DEAL_XOR,
            );
            for _ in 0..hbk {
                deal_rng.next_u64();
            }
            Some(deal_rng.next_u64())
        }
    } else {
        None
    };
    (fresh, deal)
}

#[allow(clippy::too_many_arguments)]
fn score_search_candidate_pair(
    db: &CardDb,
    root: &State,
    a: &Action,
    me: PlayerId,
    pair_cap: u32,
    eval: Evaluator<'_>,
    depth: u32,
    beam: usize,
    info: Info,
    odepth: u32,
    obeam: usize,
    fusemacro: bool,
    horizon: u32,
    hres: u32,
    hbk_nodes: &mut u64,
    shared_table: &mut Option<HashMap<(u64, u8, bool), f32>>,
    root_table: &mut Option<HashMap<(u64, u8, bool), f32>>,
) -> Option<f32> {
    let mut nodes = 0u32;
    let root_key = line_key(root);
    let cap = pair_cap;
    let pair_nodes_start = nodes;
    let s = try_apply(db, root, a, &mut nodes, cap, &[root_key])?;
    let at_fuse_choice = fusemacro && own_fuse_partners(&s, me).is_some();
    let search_depth = if at_fuse_choice {
        depth
    } else {
        depth.saturating_sub(1)
    };
    let v = if s.winner == Some(me) {
        eval.wv
    } else {
        let line = vec![root_key, line_key(&s)];
        let mut pair_stats = SearchStats::default();
        search_own(
            db,
            &s,
            me,
            search_depth,
            beam,
            &mut nodes,
            cap,
            &line,
            eval,
            odepth,
            obeam,
            &mut pair_stats,
            if info == Info::All {
                root_table.as_mut()
            } else {
                shared_table.as_mut()
            },
            None,
            fusemacro,
            horizon,
            hres,
        )
    };
    *hbk_nodes += u64::from(nodes - pair_nodes_start);
    Some(finite(v, eval.wv))
}

#[allow(clippy::too_many_arguments)]
fn try_holdback_resample(
    hbk: u32,
    pess: f32,
    node_cap: u32,
    k: u32,
    depth: u32,
    beam: usize,
    tt: bool,
    info: Info,
    deal: Deal,
    turn_stable: bool,
    wbase: Option<u64>,
    root_world_seeds: &[u64],
    hread_deal: Option<HreadDeal>,
    db: &CardDb,
    roots: &[State],
    visible: &State,
    me: PlayerId,
    subset: &[Action],
    cand: &[usize],
    acc: &[f32],
    n: &[u32],
    worst: &[f32],
    end_j: usize,
    eval: Evaluator<'_>,
    odepth: u32,
    obeam: usize,
    fusemacro: bool,
    horizon: u32,
    hres: u32,
    explain_rec: &mut Option<ExplainRecord>,
    dec_stats: &mut SearchStats,
) -> Option<(usize, f32)> {
    if !matches!(subset[end_j], Action::EndTurn) {
        return None;
    }

    let mut kill_js: Vec<usize> = Vec::new();
    for (j, a) in subset.iter().enumerate() {
        if n[j] == 0 {
            continue;
        }
        if is_kill_attack(db, &roots[0], a) {
            kill_js.push(j);
        }
    }
    if kill_js.is_empty() {
        return None;
    }

    dec_stats.hbk_checks += 1;

    let block = deal == Deal::Block && hread_deal.is_none();
    let (fresh_seeds, fresh_deal_seed) =
        hbk_fresh_seeds_and_deal(root_world_seeds, hbk, block, turn_stable, wbase);
    let pair_share = hbk_pair_share(node_cap, k, subset.len());
    let fresh_deal = fresh_deal_seed.unwrap_or(0);

    let mut fresh_acc = vec![0.0f32; subset.len()];
    let mut fresh_n = vec![0u32; subset.len()];
    let mut fresh_worst = vec![f32::INFINITY; subset.len()];

    let mut shared_table = if info != Info::All {
        tt.then(HashMap::new)
    } else {
        None
    };

    for (world, wseed) in fresh_seeds.iter().enumerate() {
        let mut root_table = if info == Info::All {
            tt.then(HashMap::new)
        } else {
            None
        };
        let root = determinize_block(
            visible,
            me,
            *wseed,
            fresh_deal,
            world as u32,
            info,
            None,
            block,
            hread_deal,
        );
        for j in std::iter::once(end_j).chain(kill_js.iter().copied()) {
            if let Some(fv) = score_search_candidate_pair(
                db,
                &root,
                &subset[j],
                me,
                pair_share,
                eval,
                depth,
                beam,
                info,
                odepth,
                obeam,
                fusemacro,
                horizon,
                hres,
                &mut dec_stats.hbk_nodes,
                &mut shared_table,
                &mut root_table,
            ) {
                fresh_acc[j] += fv;
                fresh_n[j] += 1;
                if fv < fresh_worst[j] {
                    fresh_worst[j] = fv;
                }
            }
        }
    }

    let end_root_only = root_agg(acc[end_j], n[end_j], worst[end_j], pess);
    let end_total_n = n[end_j] + fresh_n[end_j];
    let end_combined = if end_total_n == 0 {
        f32::NEG_INFINITY
    } else {
        root_agg(
            acc[end_j] + fresh_acc[end_j],
            end_total_n,
            worst[end_j].min(fresh_worst[end_j]),
            pess,
        )
    };

    let mut resample_cands = vec![HoldbackResampleCandidate {
        legal_index: cand[end_j],
        root_only: end_root_only,
        combined: end_combined,
    }];

    let mut best_kill: Option<(usize, f32)> = None;
    for j in kill_js {
        let root_only = root_agg(acc[j], n[j], worst[j], pess);
        let total_n = n[j] + fresh_n[j];
        let combined = if total_n == 0 {
            f32::NEG_INFINITY
        } else {
            root_agg(
                acc[j] + fresh_acc[j],
                total_n,
                worst[j].min(fresh_worst[j]),
                pess,
            )
        };
        resample_cands.push(HoldbackResampleCandidate {
            legal_index: cand[j],
            root_only,
            combined,
        });
        if best_kill.as_ref().is_none_or(|(_, v)| combined > *v) {
            best_kill = Some((cand[j], combined));
        }
    }

    let flipped = best_kill.as_ref().is_some_and(|(_, v)| *v > end_combined);
    if let Some(rec) = explain_rec {
        rec.holdback_resample = Some(HoldbackResampleRecord {
            fresh_k: hbk,
            candidates: resample_cands,
            flipped,
        });
    }

    if let Some((idx, agg)) = best_kill {
        if agg > end_combined {
            dec_stats.hbk_flips += 1;
            return Some((idx, agg));
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn try_holdback_trade(
    hbcheck: u32,
    pess: f32,
    horizon: u32,
    hres: u32,
    osteps: u32,
    db: &CardDb,
    roots: &[State],
    subset: &[Action],
    cand: &[usize],
    n: &[u32],
    end_j: usize,
    me: PlayerId,
    eval: Evaluator<'_>,
    odepth: u32,
    obeam: usize,
    explain_rec: &mut Option<ExplainRecord>,
    dec_stats: &mut SearchStats,
) -> Option<(usize, f32)> {
    if !matches!(subset[end_j], Action::EndTurn) {
        return None;
    }

    let mut kill_js: Vec<usize> = Vec::new();
    for (j, a) in subset.iter().enumerate() {
        if n[j] == 0 {
            continue;
        }
        if is_kill_attack(db, &roots[0], a) {
            kill_js.push(j);
        }
    }
    if kill_js.is_empty() {
        return None;
    }
    let kill_actions: Vec<&Action> = kill_js.iter().map(|j| &subset[*j]).collect();

    dec_stats.hb_checks += 1;

    let mut end_branches: Vec<HoldbackBranch> = Vec::new();
    let mut attack_worlds: Vec<Vec<HoldbackBranch>> = vec![Vec::new(); kill_js.len()];

    for root in roots.iter() {
        let x_ids_end = collect_attacker_ids(root, me, &kill_actions);
        let mut after_et = root.clone();
        if apply(db, &mut after_et, Action::EndTurn).is_err() {
            continue;
        }
        let end_branch = holdback_symmetric_v(
            db, root, &after_et, me, &x_ids_end, hbcheck, eval, horizon, hres, odepth, obeam,
            dec_stats,
        );
        if end_branch.removable {
            dec_stats.hb_worlds_removable += 1;
        }
        end_branches.push(end_branch.clone());

        for (ki, j) in kill_js.iter().enumerate() {
            let attack = &subset[*j];
            let branch = if let Some(after) =
                holdback_after_attack_finish_end(db, root, me, attack, osteps, eval)
            {
                let x_ids_atk = filter_x_ids_on_field(&after, me, &x_ids_end);
                holdback_symmetric_v(
                    db, root, &after, me, &x_ids_atk, hbcheck, eval, horizon, hres, odepth, obeam,
                    dec_stats,
                )
            } else {
                end_branch.clone()
            };
            attack_worlds[ki].push(branch);
        }
    }

    let end_prime = holdback_aggregate(&end_branches, pess)?;
    let mut attack_records = Vec::new();
    let mut best_kill: Option<(usize, f32)> = None;
    for (ki, j) in kill_js.iter().enumerate() {
        let worlds = &attack_worlds[ki];
        let agg = holdback_aggregate(worlds, pess).unwrap_or(f32::NEG_INFINITY);
        attack_records.push(HoldbackAttackRecord {
            legal_index: cand[*j],
            aggregate: agg,
            worlds: worlds
                .iter()
                .enumerate()
                .map(|(r, b)| holdback_world_record(r as u32, b))
                .collect(),
        });
        if best_kill.as_ref().is_none_or(|(_, v)| agg > *v) {
            best_kill = Some((cand[*j], agg));
        }
    }

    if let Some(rec) = explain_rec {
        rec.holdback = Some(HoldbackRecord {
            end_prime,
            end_worlds: end_branches
                .iter()
                .enumerate()
                .map(|(r, b)| holdback_world_record(r as u32, b))
                .collect(),
            attacks: attack_records,
        });
    }

    if let Some((idx, agg)) = best_kill {
        if agg > end_prime {
            dec_stats.hb_overrides += 1;
            if let Some(rec) = explain_rec {
                rec.path = ChoosePath::HoldbackTrade;
            }
            return Some((idx, agg));
        }
    }
    None
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

#[cfg(test)]
mod fuse_guard_greedy_tests {
    use std::path::Path;

    use super::*;
    use crate::apply::{apply, legal_actions, new_game};
    use crate::db::CardDb;
    use crate::ids::{First, PlayerId};
    use crate::state::GameConfig;
    use crate::CardInstance;

    fn test_db() -> CardDb {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest.join("..");
        let mut db = CardDb::load(&root).expect("cards");
        db.load_extra_dir(manifest.join("tests/fixtures/cards"))
            .expect("fixture cards");
        db
    }

    fn pad_deck(ids: &[&str]) -> Vec<CardId> {
        let mut v: Vec<CardId> = ids.iter().map(|s| CardId::parse(s).expect("id")).collect();
        let pad = CardId::parse("88001110").expect("pad");
        while v.len() < 40 {
            v.push(pad);
        }
        v
    }

    fn started(db: &CardDb, seed: u64) -> State {
        let mut state = new_game(
            db,
            GameConfig {
                seed,
                deck_a: pad_deck(&["10931110"]),
                deck_b: pad_deck(&["10931110"]),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).expect("mull A");
        apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).expect("mull B");
        state
    }

    fn clear_hand(state: &mut State, who: PlayerId) {
        state.player_mut(who).hand.clear();
    }

    fn put_hand(db: &CardDb, state: &mut State, who: PlayerId, id: &str) -> u8 {
        let card = db.card(CardId::parse(id).expect("id")).expect("card");
        let inst = CardInstance::from_card(card, state.alloc_id());
        let pos = state.player(who).hand.len() as u8;
        state.player_mut(who).hand.push(inst);
        pos
    }

    fn give_pp(state: &mut State, who: PlayerId, pp: i32, pp_max: i32) {
        let p = state.player_mut(who);
        p.pp_max = pp_max;
        p.pp = pp;
    }

    /// Sephie at 0 PP with a fuse partner in hand: the v0 leaf scores a smaller hand.
    fn sephie_noop_greedy_state(db: &CardDb) -> (State, PlayerId) {
        let me = PlayerId::A;
        let mut state = started(db, 77_001);
        clear_hand(&mut state, me);
        put_hand(db, &mut state, me, "10934110");
        put_hand(db, &mut state, me, "88001110");
        give_pp(&mut state, me, 0, 10);
        assert!(state.active == me);
        assert!(matches!(state.phase, Phase::Main));
        (state, me)
    }

    fn greedy_h0(fuseguard: bool) -> H0 {
        H0 {
            fuseguard,
            value: ValueVersion::V0,
            osteps: 1,
            olethal: false,
            oevo: false,
            net: None,
            ..H0::default()
        }
    }

    fn in_fuse_partners(state: &State, me: PlayerId) -> bool {
        matches!(
            state.phase,
            Phase::Choice {
                player,
                node: ChoiceNode::FusePartners { .. },
                ..
            } if player == me
        )
    }

    fn greedy_took_noop_fuse(before: &State, after: &State, me: PlayerId) -> bool {
        in_fuse_partners(after, me) || after.player(me).hand.len() < before.player(me).hand.len()
    }

    #[test]
    fn greedy_until_end_skips_noop_fuse_when_guard_on() {
        let db = test_db();
        let (state, me) = sephie_noop_greedy_state(&db);
        let legal = legal_actions(&db, &state);
        assert!(
            legal.iter().any(|a| matches!(a, Action::Fuse { .. })),
            "fuse must be legal"
        );
        let mut nodes = 0u32;
        let line = vec![line_key(&state)];

        let h0_on = greedy_h0(true);
        let mut on = state.clone();
        greedy_until_end(
            &db,
            &mut on,
            me,
            me,
            &mut nodes,
            u32::MAX,
            &line,
            h0_on.evaluator(&db, &[]),
            None,
            true,
            0,
        );
        assert!(
            !greedy_took_noop_fuse(&state, &on, me),
            "fuseguard on must not greedily fuse"
        );

        let h0_off = greedy_h0(false);
        let mut off = state.clone();
        nodes = 0;
        greedy_until_end(
            &db,
            &mut off,
            me,
            me,
            &mut nodes,
            u32::MAX,
            &line,
            h0_off.evaluator(&db, &[]),
            None,
            false,
            0,
        );
        assert!(
            greedy_took_noop_fuse(&state, &off, me),
            "fuseguard off must greedily fuse"
        );
    }

    #[test]
    fn finish_bot_turn_horizon_skips_noop_fuse_when_guard_on() {
        let db = test_db();
        let (state, me) = sephie_noop_greedy_state(&db);
        let mut nodes = 0u32;
        let mut line = vec![line_key(&state)];

        let h0_on = greedy_h0(true);
        let mut on = state.clone();
        assert!(
            finish_bot_turn_horizon(
                &db,
                &mut on,
                me,
                3,
                &mut nodes,
                u32::MAX,
                &mut line,
                h0_on.evaluator(&db, &[]),
                None,
            ),
            "horizon finish"
        );
        assert!(
            !greedy_took_noop_fuse(&state, &on, me),
            "finish_bot_turn_horizon must not greedily fuse with fuseguard on"
        );

        let h0_off = greedy_h0(false);
        let mut off = state.clone();
        nodes = 0;
        line = vec![line_key(&state)];
        assert!(
            finish_bot_turn_horizon(
                &db,
                &mut off,
                me,
                3,
                &mut nodes,
                u32::MAX,
                &mut line,
                h0_off.evaluator(&db, &[]),
                None,
            ),
            "horizon finish"
        );
        assert!(
            greedy_took_noop_fuse(&state, &off, me),
            "finish_bot_turn_horizon must greedily fuse with fuseguard off"
        );
    }

    #[test]
    fn holdback_bot_finish_skips_noop_fuse_when_guard_on() {
        let db = test_db();
        let (state, me) = sephie_noop_greedy_state(&db);

        let h0_on = greedy_h0(true);
        let mut on = state.clone();
        assert!(holdback_bot_finish(
            &db,
            &mut on,
            me,
            1,
            h0_on.evaluator(&db, &[])
        ));
        assert!(
            !greedy_took_noop_fuse(&state, &on, me),
            "holdback_bot_finish must not greedily fuse with fuseguard on"
        );

        let h0_off = greedy_h0(false);
        let mut off = state.clone();
        let _ = holdback_bot_finish(&db, &mut off, me, 1, h0_off.evaluator(&db, &[]));
        assert!(
            greedy_took_noop_fuse(&state, &off, me),
            "holdback_bot_finish must greedily fuse with fuseguard off"
        );
    }
}

#[cfg(test)]
mod okill_two_choice_second_tests {
    use std::path::Path;

    use super::*;
    use crate::apply::{apply, new_game};
    use crate::db::CardDb;
    use crate::ids::{First, PlayerId};
    use crate::state::{GameConfig, Phase};
    use crate::CardInstance;

    const STORM: &str = "10461110";
    const BITTERSWEET: &str = "10852310";
    const VANILLA: &str = "88001110";

    fn test_db() -> CardDb {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest.join("..");
        let mut db = CardDb::load(&root).expect("cards");
        db.load_extra_dir(manifest.join("tests/fixtures/cards"))
            .expect("fixture cards");
        db
    }

    fn pad_deck(ids: &[&str]) -> Vec<CardId> {
        let mut v: Vec<CardId> = ids.iter().map(|s| CardId::parse(s).expect("id")).collect();
        let pad = CardId::parse(VANILLA).expect("pad");
        while v.len() < 40 {
            v.push(pad);
        }
        v
    }

    fn started(db: &CardDb, seed: u64) -> State {
        let mut state = new_game(
            db,
            GameConfig {
                seed,
                deck_a: pad_deck(&[VANILLA]),
                deck_b: pad_deck(&[VANILLA]),
                first: First::A,
                opening_hands: None,
            },
        )
        .expect("new_game");
        apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).expect("mull A");
        apply(db, &mut state, Action::MulliganConfirm { swap: [false; 4] }).expect("mull B");
        state
    }

    fn clear_hand(state: &mut State, who: PlayerId) {
        state.player_mut(who).hand.clear();
    }

    fn put_hand(db: &CardDb, state: &mut State, who: PlayerId, id: &str) -> u8 {
        let card = db.card(CardId::parse(id).expect("id")).expect("card");
        let inst = CardInstance::from_card(card, state.alloc_id());
        let pos = state.player(who).hand.len() as u8;
        state.player_mut(who).hand.push(inst);
        pos
    }

    fn give_pp(state: &mut State, who: PlayerId, pp: i32, pp_max: i32) {
        let p = state.player_mut(who);
        p.pp_max = pp_max;
        p.pp = pp;
    }

    fn skip_to_player_turn(db: &CardDb, state: &mut State, who: PlayerId, turns: u32) {
        while state.player(who).turns_taken < turns || state.active != who {
            let legal = legal_actions(db, state);
            if legal.is_empty() {
                break;
            }
            if legal.iter().any(|a| matches!(a, Action::EndTurn)) {
                apply(db, state, Action::EndTurn).expect("end turn");
            } else if matches!(state.phase, Phase::Choice { .. }) {
                apply(db, state, Action::Choose(0)).expect("choose");
            } else {
                apply(db, state, legal[0].clone()).ok();
            }
        }
    }

    /// Storm then Bittersweet Departures (mode 1 + filler): second play opens a mode
    /// choice before any face damage lands.
    fn storm_then_bittersweet_root(db: &CardDb) -> State {
        let mut st = started(db, 43);
        skip_to_player_turn(db, &mut st, PlayerId::B, 1);
        clear_hand(&mut st, PlayerId::A);
        clear_hand(&mut st, PlayerId::B);
        put_hand(db, &mut st, PlayerId::B, STORM);
        put_hand(db, &mut st, PlayerId::B, BITTERSWEET);
        st.player_mut(PlayerId::B).turns_taken = 6;
        st.turn = 6;
        give_pp(&mut st, PlayerId::B, 6, 6);
        st.player_mut(PlayerId::B).ep = 0;
        st.player_mut(PlayerId::B).sep = 0;
        st.player_mut(PlayerId::B).evolved_this_turn = false;
        let storm_attack = st
            .player(PlayerId::B)
            .hand
            .iter()
            .find(|c| c.card == CardId::parse(STORM).unwrap())
            .map(|c| c.attack)
            .expect("Storm in hand");
        st.player_mut(PlayerId::A).leader_defense = storm_attack + 1;
        st.player_mut(PlayerId::B).leader_defense = 20;
        assert_eq!(st.active, PlayerId::B);
        st
    }

    #[test]
    fn opp_lethal_two_after_storm_sees_bittersweet_choice_kill() {
        let db = test_db();
        let root = storm_then_bittersweet_root(&db);
        let me = PlayerId::A;
        let storm_hand = root
            .player(PlayerId::B)
            .hand
            .iter()
            .position(|c| c.card == CardId::parse(STORM).unwrap())
            .expect("storm") as u8;
        let mut after_storm = root.clone();
        apply(&db, &mut after_storm, Action::Play { hand: storm_hand }).expect("play storm");
        assert!(
            face_potential(&db, &after_storm) > face_potential(&db, &root),
            "storm must add face potential for the follow-up line"
        );
        let mut nodes = 0u32;
        let line = vec![line_key(&root), line_key(&after_storm)];
        let mut evo_found = false;
        let mut through_used = false;
        assert!(
            opp_lethal_two_after_first(
                &db,
                &root,
                &after_storm,
                me,
                &mut nodes,
                500,
                &line,
                false,
                false,
                &mut evo_found,
                &mut through_used,
            ),
            "second play (Bittersweet mode choice) must complete lethal after Storm"
        );
    }
}

#[cfg(test)]
mod oorder_pick_side_tests {
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    use super::*;
    use crate::apply::new_game;
    use crate::db::CardDb;
    use crate::encode::vocab;
    use crate::ids::First;
    use crate::state::GameConfig;
    use serde_json::Value;

    fn unique_temp(stem: &str, ext: &str) -> std::path::PathBuf {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!("{stem}-{}-{n}.{ext}", std::process::id()))
    }

    fn recorded_db() -> CardDb {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest.join("..");
        let mut db = CardDb::load(&root).expect("cards");
        db.load_extra_dir(manifest.join("tests/fixtures/cards"))
            .expect("fixture cards");
        db.load_replace_dir(root.join("oracle/cards-as-recorded"))
            .expect("recorded overlay");
        db.load_replace_dir(manifest.join("tests/fixtures/as-recorded/earrings-engage-noop"))
            .expect("earrings");
        db.load_replace_dir(manifest.join("tests/fixtures/as-recorded/kukishiro-crest-from-deck"))
            .expect("kukishiro");
        db
    }

    fn zero_order_net_path() -> std::path::PathBuf {
        let v4 = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("models/h0-linear-v4.json"),
        )
        .expect("v4");
        let mut j: Value = serde_json::from_str(&v4).expect("json");
        if let Some(w) = j
            .get_mut("linear")
            .and_then(|l| l.get_mut("w"))
            .and_then(|v| v.as_array_mut())
        {
            for x in w.iter_mut() {
                *x = Value::from(0.0);
            }
        }
        if let Some(linear) = j.get_mut("linear").and_then(|v| v.as_object_mut()) {
            if let Some(zw) = linear.get_mut("zone_w").and_then(|v| v.as_array_mut()) {
                for row in zw.iter_mut() {
                    if let Some(row) = row.as_array_mut() {
                        for x in row.iter_mut() {
                            *x = Value::from(0.0);
                        }
                    }
                }
            }
            linear.insert("b".to_string(), Value::from(0.0));
        }
        let path = unique_temp("h0-oorder-zero-net-unit", "json");
        let mut f = fs::File::create(&path).expect("create");
        f.write_all(serde_json::to_string(&j).unwrap().as_bytes())
            .expect("write");
        path
    }

    fn review_state(db: &CardDb) -> State {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../py/tests/fixtures");
        let path = root.join("review13/games/4984932781931433298-b4973563.json");
        let cap: Value = serde_json::from_str(&fs::read_to_string(path).expect("review")).unwrap();
        let seed = cap["seed"].as_u64().unwrap();
        let mut deck = Vec::new();
        for (id, n) in cap["deckA"].as_object().unwrap() {
            let cid = CardId::parse(id).unwrap();
            deck.extend(std::iter::repeat_n(cid, n.as_u64().unwrap() as usize));
        }
        let mut deck_b = Vec::new();
        for (id, n) in cap["deckB"].as_object().unwrap() {
            let cid = CardId::parse(id).unwrap();
            deck_b.extend(std::iter::repeat_n(cid, n.as_u64().unwrap() as usize));
        }
        let first = match cap.get("first").and_then(|v| v.as_str()) {
            Some("b") | Some("B") => First::B,
            Some("a") | Some("A") => First::A,
            _ => First::Coin,
        };
        let mut st = new_game(
            db,
            GameConfig {
                seed,
                deck_a: deck,
                deck_b,
                first,
                opening_hands: None,
            },
        )
        .expect("game");
        let ply = 27usize;
        for step in cap["actions"].as_array().unwrap().iter().take(ply) {
            if step.get("reseed").is_some() {
                st.reseed(step["reseed"].as_u64().unwrap());
                continue;
            }
            let mut body = serde_json::Map::new();
            for (k, v) in step.as_object().expect("action object") {
                if k == "value" || k == "bot_value" {
                    continue;
                }
                body.insert(k.clone(), v.clone());
            }
            let neu: crate::NeutralAction =
                serde_json::from_value(serde_json::Value::Object(body)).unwrap();
            crate::apply::apply_neutral(db, &mut st, &neu).unwrap();
        }
        st
    }

    #[test]
    fn pick_value_uses_order_and_oorder_on_separate_sides() {
        let db = recorded_db();
        let state = review_state(&db);
        let me = state.active;
        let opp = me.opponent();
        let v53_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("models/h0-order-v53.json")
            .display()
            .to_string();
        let zero_path = zero_order_net_path().display().to_string();
        let v53 = ValueNet::load(&v53_path).expect("v53");
        let zero = ValueNet::load(&zero_path).expect("zero");
        let h = H0 {
            order: Some(v53.clone()),
            order_path: Some(v53_path),
            oorder_net: Some(zero.clone()),
            oorder_path: Some(zero_path),
            ..H0::default()
        };
        let root_vocab = vocab(&state);
        let eval = h.evaluator(&db, &root_vocab);
        let own = eval.pick_value(&state, me, me);
        let opp_v = eval.pick_value(&state, opp, me);
        let leaf_own = eval.value(&state, me);
        let leaf_opp = eval.value(&state, opp);
        assert_eq!(own, eval.order_value(&state, me, &v53));
        assert_eq!(opp_v, eval.order_value(&state, opp, &zero));
        assert_eq!(opp_v, 0.0, "zero order net → scale×tanh(0), bppv=0");
        assert_eq!(h.bppv, 0.0);
        assert_ne!(own, leaf_own, "bot side should use order=v53");
        assert_ne!(opp_v, leaf_opp, "opponent side should use oorder=zero");
    }

    #[test]
    fn pick_value_oorder_alone_orders_opponent_not_self() {
        let db = recorded_db();
        let state = review_state(&db);
        let me = state.active;
        let opp = me.opponent();
        let zero_path = zero_order_net_path().display().to_string();
        let zero = ValueNet::load(&zero_path).expect("zero");
        let h = H0 {
            oorder_net: Some(zero.clone()),
            oorder_path: Some(zero_path),
            ..H0::default()
        };
        let root_vocab = vocab(&state);
        let eval = h.evaluator(&db, &root_vocab);
        let own = eval.pick_value(&state, me, me);
        let opp_v = eval.pick_value(&state, opp, me);
        let leaf_own = eval.value(&state, me);
        let leaf_opp = eval.value(&state, opp);
        assert_eq!(own, leaf_own, "no order= → bot uses leaf");
        assert_eq!(opp_v, eval.order_value(&state, opp, &zero));
        assert_eq!(opp_v, 0.0, "zero order net → scale×tanh(0), bppv=0");
        assert_ne!(opp_v, leaf_opp, "oorder alone → opponent uses model");
    }
}

#[cfg(test)]
mod speed_reuse_tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::apply::{apply, apply_neutral, legal_actions, new_game};
    use crate::db::CardDb;
    use crate::ids::First;
    use crate::state::GameConfig;
    use serde_json::Value;

    fn recorded_db() -> CardDb {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest.join("..");
        let mut db = CardDb::load(&root).expect("cards");
        db.load_extra_dir(manifest.join("tests/fixtures/cards"))
            .expect("fixture cards");
        db.load_replace_dir(root.join("oracle/cards-as-recorded"))
            .expect("recorded overlay");
        db.load_replace_dir(manifest.join("tests/fixtures/as-recorded/earrings-engage-noop"))
            .expect("earrings");
        db.load_replace_dir(manifest.join("tests/fixtures/as-recorded/kukishiro-crest-from-deck"))
            .expect("kukishiro");
        db
    }

    fn review_at(db: &CardDb, review: &str, game: &str, ply: usize) -> State {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../py/tests/fixtures")
            .join(review)
            .join("games")
            .join(game);
        let cap: Value = serde_json::from_str(&fs::read_to_string(path).expect("review")).unwrap();
        let seed = cap["seed"].as_u64().unwrap();
        let mut deck_a = Vec::new();
        for (id, n) in cap["deckA"].as_object().unwrap() {
            let cid = CardId::parse(id).unwrap();
            deck_a.extend(std::iter::repeat_n(cid, n.as_u64().unwrap() as usize));
        }
        let mut deck_b = Vec::new();
        for (id, n) in cap["deckB"].as_object().unwrap() {
            let cid = CardId::parse(id).unwrap();
            deck_b.extend(std::iter::repeat_n(cid, n.as_u64().unwrap() as usize));
        }
        let first = match cap.get("first").and_then(|v| v.as_str()) {
            Some("b") | Some("B") => First::B,
            Some("a") | Some("A") => First::A,
            _ => First::Coin,
        };
        let mut st = new_game(
            db,
            GameConfig {
                seed,
                deck_a,
                deck_b,
                first,
                opening_hands: None,
            },
        )
        .expect("game");
        for step in cap["actions"].as_array().unwrap().iter().take(ply) {
            if step.get("reseed").is_some() {
                st.reseed(step["reseed"].as_u64().unwrap());
                continue;
            }
            let mut body = serde_json::Map::new();
            for (k, v) in step.as_object().expect("action object") {
                if k == "value" || k == "bot_value" {
                    continue;
                }
                body.insert(k.clone(), v.clone());
            }
            let neu: crate::NeutralAction =
                serde_json::from_value(serde_json::Value::Object(body)).unwrap();
            apply_neutral(db, &mut st, &neu).unwrap();
        }
        st
    }

    fn v0() -> H0 {
        H0 {
            value: ValueVersion::V0,
            net: None,
            olethal: false,
            oevo: false,
            ..H0::default()
        }
    }

    #[test]
    fn take_or_apply_charges_one_node_and_refuses_at_cap() {
        let db = recorded_db();
        let st = review_at(&db, "review13", "4984932781931433298-b4973563.json", 27);
        let mut nodes = 3u32;
        let kept = st.clone();
        let got = take_or_apply(
            &db,
            &st,
            &Action::EndTurn,
            Some(kept.clone()),
            &mut nodes,
            10,
            &[],
        )
        .expect("kept");
        assert_eq!(nodes, 4, "kept afterstate charges one node");
        assert_eq!(search_key(&got), search_key(&kept));
        let mut nodes = 10u32;
        let got = take_or_apply(
            &db,
            &st,
            &Action::EndTurn,
            Some(st.clone()),
            &mut nodes,
            10,
            &[],
        );
        assert!(got.is_none(), "at the cap the kept state is not taken");
        assert_eq!(nodes, 10, "a cap refusal does not charge");
    }

    #[test]
    fn greedy_index_kept_afterstate_matches_reapply() {
        let db = recorded_db();
        let moments = [
            ("review13", "4984932781931433298-b4973563.json", 27usize),
            ("review14", "12644260486301391828-d90f8eb2.json", 35),
            ("review14", "6598261483642061665-d90f8eb2.json", 91),
        ];
        let h = v0();
        for (review, game, ply) in moments {
            let state = review_at(&db, review, game, ply);
            let legal = legal_actions(&db, &state);
            assert!(legal.len() >= 2, "{review} {ply}");
            let me = acting_player(&state);
            let eval = h.evaluator(&db, &[]);
            let line = vec![line_key(&state)];
            let mut nodes = 0u32;
            let (i, kept) = greedy_index(
                &db,
                &state,
                &legal,
                me,
                me,
                &mut nodes,
                u32::MAX,
                &line,
                eval,
                false,
            );
            let kept = kept.expect("a candidate applied");
            let mut again_nodes = 0u32;
            let again = try_apply(&db, &state, &legal[i], &mut again_nodes, u32::MAX, &line)
                .expect("re-apply");
            assert_eq!(
                search_key(&kept),
                search_key(&again),
                "{review} ply {ply} kept afterstate"
            );
        }
    }

    #[test]
    fn line_key_is_id_on_choice_and_changes_across_main() {
        let db = recorded_db();
        let choice = review_at(&db, "review13", "4984932781931433298-b4973563.json", 45);
        assert!(matches!(choice.phase, Phase::Choice { .. }));
        assert_eq!(line_key(&choice), search_key_id(&choice));
        let main = review_at(&db, "review14", "12644260486301391828-d90f8eb2.json", 35);
        assert!(matches!(main.phase, Phase::Main));
        let legal = legal_actions(&db, &main);
        let next = legal.iter().find_map(|a| {
            let mut s = main.clone();
            if apply(&db, &mut s, a.clone()).is_ok()
                && s.winner.is_none()
                && matches!(s.phase, Phase::Main)
            {
                Some(s)
            } else {
                None
            }
        });
        let next = next.expect("Main successor");
        assert_ne!(line_key(&main), line_key(&next));
    }
}
