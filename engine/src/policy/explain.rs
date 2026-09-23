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
    /// Search entered but no `(root, candidate)` pair scored (e.g. the
    /// consensus-lethal check spent the entire node cap).
    Unscored,
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
    OppSearch,
    Tt,
}

/// Leaf snapshot at the end of a principal-variation line.
#[derive(Debug, Clone, Serialize)]
pub struct PvLeaf {
    pub value: f32,
    pub phase: String,
    pub turn: u32,
    pub active: String,
}

/// Principal variation returned from a search subtree.
#[derive(Debug, Clone)]
pub(crate) struct Line {
    pub actions: Vec<Action>,
    pub len: usize,
    pub end: PvEnd,
    pub value: f32,
    pub phase: String,
    pub turn: u32,
    pub active: String,
}

/// One world determinization's search result for a root candidate.
#[derive(Debug, Clone, Serialize)]
pub struct WorldRecord {
    pub r: u32,
    pub raw: f32,
    pub clamped: f32,
    pub node_cap: u32,
    pub nodes: u32,
    pub hit_cap: bool,
    pub skipped: bool,
    pub pv: Vec<NeutralAction>,
    pub pv_len: u32,
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
    pub nodes_lethal: u32,
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
            nodes_lethal: 0,
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
    path_len: usize,
    last: Option<Line>,
    cap: u32,
    hit_cap: bool,
}

impl PvTracker {
    pub(crate) fn new(cap: u32) -> Self {
        Self {
            path: Vec::new(),
            path_len: 0,
            last: None,
            cap,
            hit_cap: false,
        }
    }

    pub(crate) fn push(&mut self, a: Action) {
        self.path_len += 1;
        if self.path.len() < 12 {
            self.path.push(a);
        }
    }

    pub(crate) fn pop_to(&mut self, len: usize) {
        self.path_len = len;
        if self.path.len() > len {
            self.path.truncate(len);
        }
    }

    pub(crate) fn path_len(&self) -> usize {
        self.path_len
    }

    pub(crate) fn note_nodes(&mut self, nodes: u32) {
        if nodes >= self.cap {
            self.hit_cap = true;
        }
    }

    pub(crate) fn set_leaf(&mut self, v: f32, end: PvEnd, state: &State) {
        self.last = Some(self.make_line(v, end, state));
    }

    pub(crate) fn set_tt(&mut self, v: f32, state: &State) {
        self.last = Some(self.make_line(v, PvEnd::Tt, state));
    }

    pub(crate) fn take_last(&mut self) -> Option<Line> {
        self.last.take()
    }

    pub(crate) fn restore_last(&mut self, line: Option<Line>) {
        self.last = line;
    }

    fn make_line(&self, v: f32, end: PvEnd, state: &State) -> Line {
        Line {
            actions: self.path.clone(),
            len: self.path_len,
            end,
            value: v,
            phase: phase_str(&state.phase),
            turn: state.turn,
            active: player_str(state.active),
        }
    }

    pub(crate) fn skipped_world(r: u32, cap: u32) -> WorldRecord {
        WorldRecord {
            r,
            raw: 0.0,
            clamped: 0.0,
            node_cap: cap,
            nodes: 0,
            hit_cap: false,
            skipped: true,
            pv: Vec::new(),
            pv_len: 0,
            end: None,
            leaf: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn into_world(
        self,
        r: u32,
        raw: f32,
        clamped: f32,
        skipped: bool,
        nodes_spent: u32,
        db: &CardDb,
        root: &State,
    ) -> WorldRecord {
        if skipped {
            return Self::skipped_world(r, self.cap);
        }
        let line = self.last;
        let (pv, pv_len, end, leaf) = if let Some(line) = line {
            let pv = actions_to_neutral(db, root, &line.actions);
            let leaf = PvLeaf {
                value: line.value,
                phase: line.phase,
                turn: line.turn,
                active: line.active,
            };
            (pv, line.len as u32, Some(line.end), Some(leaf))
        } else {
            (Vec::new(), 0, None, None)
        };
        WorldRecord {
            r,
            raw,
            clamped,
            node_cap: self.cap,
            nodes: nodes_spent,
            hit_cap: self.hit_cap,
            skipped: false,
            pv,
            pv_len,
            end,
            leaf,
        }
    }
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
