//! Typed engine errors. A load/play path that would reach unimplemented
//! behaviour fails here — never as a silent no-op.

use std::fmt;

use crate::card::CardId;
use crate::ids::PlayerId;
use crate::trace::PickWhat;

/// Failures that abort loading a card or starting a game.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("parse {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("catalog {path}: {source}")]
    Catalog {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("duplicate card id {id} ({first} and {second})")]
    Duplicate {
        id: String,
        first: String,
        second: String,
    },
    #[error("missing card file for id {0}")]
    MissingCard(CardId),
    #[error("missing crest file for id {0}")]
    MissingCrest(String),
    #[error("{0}")]
    Unsupported(Unsupported),
    #[error("card {card} has bound ref {name} that no `as` on the same card can produce")]
    UnboundRef { card: String, name: String },
    #[error("opening hand card {card} is not in player {player}'s deck")]
    OpeningHandNotInDeck { player: String, card: String },
}

/// A card uses a schema construct the M1 engine does not implement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsupported {
    pub card: String,
    pub construct: String,
}

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unsupported {{ card: {}, construct: {} }}",
            self.card, self.construct
        )
    }
}

impl std::error::Error for Unsupported {}

/// An action was not among `legal_actions` (or a scripted pick was illegal).
#[derive(Debug, thiserror::Error)]
pub enum Illegal {
    #[error("action is not legal in this state")]
    NotLegal,
    #[error("game is already terminal")]
    Terminal,
    #[error("oracle pick not legal: {0}")]
    OraclePickNotLegal(OraclePickNotLegal),
    #[error("resolution step ceiling exceeded")]
    StepCeiling,
    #[error("{0}")]
    Unsupported(Unsupported),
}

/// Recorded `Pick.chose` is not among the current candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OraclePickNotLegal {
    pub what: PickWhat,
    pub chose: String,
    pub candidates: Vec<String>,
}

impl fmt::Display for OraclePickNotLegal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OraclePickNotLegal {{ what: {}, chose: {}, candidates: [{}] }}",
            self.what.as_str(),
            self.chose,
            self.candidates.join(", ")
        )
    }
}

/// Failures from the replay / diff harness.
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse line {line}: {source}")]
    Parse {
        line: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error("header mismatch: {0}")]
    Header(String),
    #[error("divergence at i={i} path {path}: arena={arena} trace={trace}")]
    Diverge {
        i: u32,
        path: String,
        arena: String,
        trace: String,
    },
    #[error("{0}")]
    Unsupported(Unsupported),
    #[error("{0}")]
    Oracle(OraclePickNotLegal),
    #[error("oracle at i={i}: {err}")]
    OracleAt { i: u32, err: OraclePickNotLegal },
    #[error("illegal action at i={i}: {action}; legal={legal} ({source})")]
    Illegal {
        i: u32,
        action: String,
        legal: String,
        #[source]
        source: Illegal,
    },
}

impl ReplayError {
    pub fn exit_code(&self) -> i32 {
        match self {
            ReplayError::Diverge { .. } | ReplayError::Header(_) | ReplayError::Illegal { .. } => 2,
            ReplayError::Unsupported(_) | ReplayError::Oracle(_) | ReplayError::OracleAt { .. } => {
                3
            }
            ReplayError::Io(_) | ReplayError::Parse { .. } => 1,
        }
    }
}

/// Winner side used by a few constructors.
#[allow(dead_code)]
pub fn winner_label(p: PlayerId) -> &'static str {
    match p {
        PlayerId::A => "a",
        PlayerId::B => "b",
    }
}
