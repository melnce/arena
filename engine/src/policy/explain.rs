//! Per-decision explain record for [`H0::choose`]. When no sink is attached
//! (`explain` is `None`), recording is a no-op and search is unchanged.

use serde::Serialize;

use crate::action::Action;
use crate::apply::apply;
use crate::db::CardDb;
use crate::state::{Phase, State};
use crate::trace::{player_str, NeutralAction};

/// Which branch of [`H0::choose`] decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoosePath {
    SingleLegal,
    Mulligan,
    OnePly,
    ConsensusLethal,
    Search,
}

/// How a principal-variation line ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PvEnd {
    Depth,
    Cap,
    Terminal,
    OppReply,
    OppLethal,
}

/// Leaf snapshot at the end of a principal-variation line.
#[derive(Debug, Clone, Serialize)]
pub struct PvLeaf {
    pub value: f32,
    pub phase: String,
    pub turn: u32,
    pub active: String,
}

/// One world determinization's search result for a root candidate.
#[derive(Debug, Clone, Serialize)]
pub struct WorldRecord {
    pub raw: f32,
    pub clamped: f32,
    pub node_cap: u32,
    pub nodes: u32,
    pub hit_cap: bool,
    pub skipped: bool,
    pub pv: Vec<NeutralAction>,
    pub end: Option<PvEnd>,
    pub leaf: Option<PvLeaf>,
}

/// One root candidate's aggregated scores.
#[derive(Debug, Clone, Serialize)]
pub struct CandidateRecord {
    pub legal_index: usize,
    pub action: NeutralAction,
    pub worlds: Vec<WorldRecord>,
    pub root_agg: f32,
    pub worst: f32,
    pub n: u32,
}

/// Full explain record for one `choose` call.
#[derive(Debug, Clone, Serialize)]
pub struct ExplainRecord {
    pub path: ChoosePath,
    pub k: u32,
    pub node_cap: u32,
    pub alloc: String,
    pub nodes: u32,
    pub candidates: Vec<CandidateRecord>,
    pub chosen_index: usize,
    pub tie_set: Vec<usize>,
}

impl ExplainRecord {
    pub(crate) fn new(path: ChoosePath, k: u32, node_cap: u32, alloc: &str) -> Self {
        Self {
            path,
            k,
            node_cap,
            alloc: alloc.to_string(),
            nodes: 0,
            candidates: Vec::new(),
            chosen_index: 0,
            tie_set: Vec::new(),
        }
    }
}

fn phase_str(p: &Phase) -> String {
    match p {
        Phase::Mulligan { .. } => "mulligan".into(),
        Phase::Main | Phase::Combat => "main".into(),
        Phase::Choice { .. } => "choice".into(),
        Phase::End => "end".into(),
        Phase::Terminal => "terminal".into(),
    }
}

/// Tracks the principal variation for one `(root, candidate)` pair.
pub(crate) struct PvTracker {
    path: Vec<Action>,
    best_actions: Vec<Action>,
    best_end: Option<PvEnd>,
    best_leaf_value: f32,
    best_phase: String,
    best_turn: u32,
    best_active: String,
    cap: u32,
    start_nodes: u32,
    hit_cap: bool,
}

impl PvTracker {
    pub(crate) fn new(start_nodes: u32, cap: u32) -> Self {
        Self {
            path: Vec::new(),
            best_actions: Vec::new(),
            best_end: None,
            best_leaf_value: f32::NAN,
            best_phase: String::new(),
            best_turn: 0,
            best_active: String::new(),
            cap,
            start_nodes,
            hit_cap: false,
        }
    }

    pub(crate) fn push(&mut self, a: Action) {
        if self.path.len() < 12 {
            self.path.push(a);
        }
    }

    pub(crate) fn pop_to(&mut self, len: usize) {
        self.path.truncate(len);
    }

    pub(crate) fn path_len(&self) -> usize {
        self.path.len()
    }

    pub(crate) fn best_actions(&self) -> &[Action] {
        &self.best_actions
    }

    pub(crate) fn note_nodes(&mut self, nodes: u32) {
        if nodes >= self.cap {
            self.hit_cap = true;
        }
    }

    pub(crate) fn note_leaf(&mut self, v: f32, end: PvEnd, state: &State) {
        self.note_end(v, end, state);
        self.capture_path(v);
    }

    pub(crate) fn note_end(&mut self, v: f32, end: PvEnd, state: &State) {
        self.best_end = Some(end);
        self.best_leaf_value = v;
        self.best_phase = phase_str(&state.phase);
        self.best_turn = state.turn;
        self.best_active = player_str(state.active);
    }

    pub(crate) fn commit_if_better(&mut self, v: f32) {
        if self.best_actions.is_empty() || v > self.best_leaf_value {
            self.capture_path(v);
        }
    }

    pub(crate) fn commit_if_worse(&mut self, v: f32) {
        if self.best_actions.is_empty() || v < self.best_leaf_value {
            self.capture_path(v);
        }
    }

    fn capture_path(&mut self, v: f32) {
        self.best_actions = self.path.clone();
        if self.best_actions.len() > 12 {
            self.best_actions.truncate(12);
        }
        self.best_leaf_value = v;
    }

    pub(crate) fn into_world(
        self,
        raw: f32,
        clamped: f32,
        skipped: bool,
        db: &CardDb,
        root: &State,
    ) -> WorldRecord {
        let nodes = self.start_nodes; // filled by caller
        let pv = actions_to_neutral(db, root, &self.best_actions);
        let leaf = if self.best_end.is_some() {
            Some(PvLeaf {
                value: self.best_leaf_value,
                phase: self.best_phase.clone(),
                turn: self.best_turn,
                active: self.best_active.clone(),
            })
        } else {
            None
        };
        WorldRecord {
            raw,
            clamped,
            node_cap: self.cap,
            nodes,
            hit_cap: self.hit_cap,
            skipped,
            pv,
            end: self.best_end,
            leaf,
        }
    }
}

pub(crate) fn finish_world(
    tracker: PvTracker,
    raw: f32,
    clamped: f32,
    skipped: bool,
    nodes_spent: u32,
    db: &CardDb,
    root: &State,
) -> WorldRecord {
    let mut w = tracker.into_world(raw, clamped, skipped, db, root);
    w.nodes = nodes_spent;
    w
}

pub(crate) fn leaf_after_pv(
    db: &CardDb,
    root: &State,
    actions: &[Action],
    value: impl Fn(&State) -> f32,
) -> Option<(f32, String, u32, String)> {
    if actions.is_empty() {
        return None;
    }
    let mut s = root.clone();
    for a in actions {
        if apply(db, &mut s, a.clone()).is_err() {
            return None;
        }
    }
    Some((value(&s), phase_str(&s.phase), s.turn, player_str(s.active)))
}

fn actions_to_neutral(db: &CardDb, root: &State, actions: &[Action]) -> Vec<NeutralAction> {
    let mut s = root.clone();
    let mut out = Vec::with_capacity(actions.len());
    for a in actions {
        out.push(crate::action::to_neutral(&s, a));
        if apply(db, &mut s, a.clone()).is_err() {
            break;
        }
    }
    out
}
