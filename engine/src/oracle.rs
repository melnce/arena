//! Replay a JSONL trace with `ScriptedRng` and diff every line.
//!
//! `replay_trace` is the library form of `arena-replay`. Exit-code mapping
//! stays in the bin: green → 0, divergence / illegal / header → 2,
//! unsupported / oracle-pick-not-legal → 3.

use serde::Deserialize;

use crate::apply::{apply_neutral, new_game};
use crate::card::CardId;
use crate::db::CardDb;
use crate::error::{Illegal, ReplayError};
use crate::ids::First;
use crate::legal_actions_neutral;
use crate::rng::GameRng;
use crate::snapshot::snapshot_json;
use crate::state::{GameConfig, OpeningHands, State};
use crate::trace::{
    legal_divergence_parts, neutral_json, picks_from_trace_rng, replay_compare_legal,
    replay_state_diff, NeutralAction, TraceHeader,
};

/// Result of a completed replay (parse / load / unsupported stay `ReplayError`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayOutcome {
    Green,
    Divergence(Divergence),
}

/// First line that does not match, with enough context to classify it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergence {
    pub i: u32,
    pub path: String,
    pub arena: String,
    pub trace: String,
    pub action: serde_json::Value,
    /// `Name (id)` for the hand/field slot the path points at, if any.
    pub cards: Vec<String>,
}

/// One allowlist row in `oracle/known-divergences.json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnownDivergence {
    pub trace: String,
    pub i: u32,
    pub path: String,
    pub class: DivergenceClass,
    pub reason: String,
    pub since: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DivergenceClass {
    OldData,
    OldEmitter,
    Engine,
    Convention,
    OldRule,
}

impl KnownDivergence {
    pub fn parse_list(text: &str) -> Result<Vec<Self>, String> {
        serde_json::from_str(text).map_err(|e| format!("known-divergences.json: {e}"))
    }

    pub fn matches(&self, trace: &str, i: u32, path: &str) -> bool {
        self.trace == trace && self.i == i && self.path == path
    }
}

impl Divergence {
    /// One-line report used by the oracle CI test.
    pub fn report_line(&self, trace: &str) -> String {
        format!(
            "{trace} i={} {} arena={} trace={} action={} cards={}",
            self.i,
            self.path,
            self.arena,
            self.trace,
            serde_json::to_string(&self.action).unwrap_or_else(|_| "null".into()),
            self.cards.join(", ")
        )
    }

    pub fn side(&self) -> &'static str {
        side_of_path(&self.path)
    }
}

/// Replay `text` (one JSONL document) against `db`.
pub fn replay_trace(db: &CardDb, text: &str) -> Result<ReplayOutcome, ReplayError> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| ReplayError::Header("empty".into()))?;
    let header: TraceHeader = serde_json::from_str(header_line)
        .map_err(|source| ReplayError::Parse { line: 1, source })?;
    let deck_a = parse_ids(&header.deck_a);
    let deck_b = parse_ids(&header.deck_b);
    let first = if header.first == "b" {
        First::B
    } else {
        First::A
    };
    let opening = OpeningHands {
        a: parse_ids(&header.opening_hands.a),
        b: parse_ids(&header.opening_hands.b),
    };
    let mut state = new_game(
        db,
        GameConfig {
            seed: header.seed,
            deck_a,
            deck_b,
            first,
            opening_hands: Some(opening),
        },
    )
    .map_err(|e| match e {
        crate::error::LoadError::Unsupported(u) => ReplayError::Unsupported(u),
        other => ReplayError::Header(other.to_string()),
    })?;
    for (ln, line) in lines.enumerate() {
        let rec: serde_json::Value =
            serde_json::from_str(line).map_err(|source| ReplayError::Parse {
                line: ln + 2,
                source,
            })?;
        let i = rec.get("i").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let action: NeutralAction = serde_json::from_value(
            rec.get("action").cloned().unwrap_or_default(),
        )
        .map_err(|source| ReplayError::Parse {
            line: ln + 2,
            source,
        })?;
        let action_json = serde_json::to_value(&action).unwrap_or(serde_json::Value::Null);
        let rng = rec.get("rng").map(picks_from_trace_rng).unwrap_or_default();
        state.rng = GameRng::scripted(rng, header.seed);
        if let Err(e) = apply_neutral(db, &mut state, &action) {
            return Err(match e {
                Illegal::Unsupported(u) => ReplayError::Unsupported(u),
                Illegal::OraclePickNotLegal(o) => ReplayError::OracleAt { i, err: o },
                other => illegal_at(i, &action, &legal_json(db, &state), other),
            });
        }
        let got = snapshot_json(&state);
        let want = rec.get("state").cloned().unwrap_or(serde_json::Value::Null);
        if let Some((path, a, b)) = replay_state_diff(&got, &want) {
            let cards = cards_at_path(db, &path, &got, &want);
            return Ok(ReplayOutcome::Divergence(Divergence {
                i,
                path,
                arena: a,
                trace: b,
                action: action_json,
                cards,
            }));
        }
        if replay_compare_legal(&got, &want) {
            if let Some(legal) = rec.get("legal") {
                let mut ours: Vec<NeutralAction> = legal_actions_neutral(db, &state);
                let mut theirs: Vec<NeutralAction> =
                    serde_json::from_value(legal.clone()).unwrap_or_default();
                ours.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
                theirs.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
                if ours != theirs {
                    let (arena, trace) = legal_divergence_parts(&ours, &theirs);
                    return Ok(ReplayOutcome::Divergence(Divergence {
                        i,
                        path: "legal".into(),
                        arena,
                        trace,
                        action: action_json,
                        cards: Vec::new(),
                    }));
                }
            }
        }
    }
    Ok(ReplayOutcome::Green)
}

fn legal_json(db: &CardDb, state: &State) -> String {
    let acts: Vec<String> = legal_actions_neutral(db, state)
        .iter()
        .map(neutral_json)
        .collect();
    format!("[{}]", acts.join(", "))
}

fn illegal_at(i: u32, action: &NeutralAction, legal: &str, source: Illegal) -> ReplayError {
    ReplayError::Illegal {
        i,
        action: neutral_json(action),
        legal: legal.to_string(),
        source,
    }
}

fn parse_ids(v: &[String]) -> Vec<CardId> {
    v.iter().filter_map(|s| CardId::parse(s)).collect()
}

fn card_label(db: &CardDb, id: &str) -> String {
    if let Some(cid) = CardId::parse(id) {
        if let Ok(card) = db.card(cid) {
            return format!("{} ({id})", card.name());
        }
    }
    if let Some(rec) = db.catalog.get(id) {
        return format!("{} ({id})", rec.name);
    }
    id.to_string()
}

/// `players.a.field[0].max_defense` → the card sitting in that slot.
fn slot_card_id(path: &str, state: &serde_json::Value) -> Option<String> {
    let rest = path.strip_prefix("players.")?;
    let (side, rest) = rest.split_once('.')?;
    if side != "a" && side != "b" {
        return None;
    }
    let zone = if rest.starts_with("field[") {
        "field"
    } else if rest.starts_with("hand[") {
        "hand"
    } else {
        return None;
    };
    let after = rest.strip_prefix(zone)?.strip_prefix('[')?;
    let (idx, _) = after.split_once(']')?;
    let i: usize = idx.parse().ok()?;
    state
        .get("players")?
        .get(side)?
        .get(zone)?
        .get(i)?
        .get("card")?
        .as_str()
        .map(str::to_string)
}

fn cards_at_path(
    db: &CardDb,
    path: &str,
    got: &serde_json::Value,
    want: &serde_json::Value,
) -> Vec<String> {
    let mut ids = Vec::new();
    for state in [got, want] {
        if let Some(id) = slot_card_id(path, state) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids.into_iter().map(|id| card_label(db, &id)).collect()
}

fn side_of_path(path: &str) -> &'static str {
    if path.starts_with("players.a.") {
        "a"
    } else if path.starts_with("players.b.") {
        "b"
    } else {
        "-"
    }
}
