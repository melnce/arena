//! NeutralAction, Pick, and the JSONL header — `docs/trace-format.md`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::card::CardId;
use crate::ids::{PlayerId, Slot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickWhat {
    Draw,
    RandomTarget,
    RandomCard,
    RandomSplit,
    Coin,
    RandomUnused,
    Reanimate,
    MultisetPick,
}

impl PickWhat {
    pub fn as_str(self) -> &'static str {
        match self {
            PickWhat::Draw => "draw",
            PickWhat::RandomTarget => "random_target",
            PickWhat::RandomCard => "random_card",
            PickWhat::RandomSplit => "random_split",
            PickWhat::Coin => "coin",
            PickWhat::RandomUnused => "random_unused",
            PickWhat::Reanimate => "reanimate",
            PickWhat::MultisetPick => "multiset_pick",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PickChose {
    Id(String),
    Slot { slot: u8 },
    Mode { mode: u8 },
    Counts(Vec<i32>),
}

impl PickChose {
    pub fn as_key(&self) -> String {
        match self {
            PickChose::Id(s) => s.clone(),
            PickChose::Slot { slot } => format!("slot:{slot}"),
            PickChose::Mode { mode } => format!("mode:{mode}"),
            PickChose::Counts(c) => format!("{c:?}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pick {
    pub what: PickWhat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub among: Option<String>,
    pub chose: PickChose,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpeningHandsJson {
    pub a: Vec<String>,
    pub b: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceHeader {
    pub v: u32,
    pub engine: String,
    pub seed: u64,
    pub first: String,
    pub deck_a: Vec<String>,
    pub deck_b: Vec<String>,
    pub opening_hands: OpeningHandsJson,
}

/// Parse a trace line's `rng` array, ignoring every `{"what":"raw",…}` pick
/// (old-engine shuffles). Other objects must be `Pick`s.
pub fn picks_from_trace_rng(value: &serde_json::Value) -> Vec<Pick> {
    let Some(arr) = value.as_array() else {
        return Vec::new();
    };
    let filtered: Vec<serde_json::Value> = arr
        .iter()
        .filter(|item| item.get("what").and_then(|w| w.as_str()) != Some("raw"))
        .cloned()
        .collect();
    serde_json::from_value(serde_json::Value::Array(filtered)).unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionLine {
    pub i: u32,
    pub action: NeutralAction,
    #[serde(default)]
    pub rng: Vec<Pick>,
    pub state: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal: Option<Vec<NeutralAction>>,
}

/// Closed discriminator = the single key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeutralAction {
    Mulligan {
        player: String,
        swap: [bool; 4],
    },
    Play {
        player: String,
        hand_pos: u8,
        card: String,
    },
    Attack {
        player: String,
        attacker_slot: u8,
        target: AttackTargetJson,
    },
    Evolve {
        player: String,
        slot: u8,
        #[serde(rename = "super")]
        super_evolve: bool,
    },
    Engage {
        player: String,
        slot: u8,
    },
    Fuse {
        player: String,
        host_pos: u8,
        partner_pos: Vec<u8>,
    },
    BonusPp {
        player: String,
    },
    Choose {
        player: String,
        option: ChooseOptionJson,
    },
    Confirm {
        player: String,
    },
    EndTurn {
        player: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttackTargetJson {
    Slot { slot: u8 },
    Leader(LeaderWord),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LeaderWord {
    Leader,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChooseOptionJson {
    Card { card: String },
    Slot { slot: u8 },
    Leader(LeaderWord),
    Mode { mode: u8 },
}

pub fn player_str(p: PlayerId) -> String {
    p.as_str().to_string()
}

pub fn parse_player(s: &str) -> Option<PlayerId> {
    match s {
        "a" => Some(PlayerId::A),
        "b" => Some(PlayerId::B),
        _ => None,
    }
}

pub fn sorted_deck_ids(ids: &[CardId]) -> Vec<String> {
    let mut v: Vec<String> = ids.iter().map(|c| c.as_str()).collect();
    v.sort();
    v
}

pub fn slot_of(s: Slot) -> u8 {
    s.0
}

/// Recursively sort object keys so snapshot JSON is comparable.
pub fn sort_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            for k in keys {
                if let Some(v) = map.get(&k) {
                    out.insert(k, sort_json(v.clone()));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sort_json).collect())
        }
        other => other,
    }
}

pub fn json_eq_first_diff<'a>(
    a: &'a serde_json::Value,
    b: &'a serde_json::Value,
    path: &str,
) -> Option<(String, String, String)> {
    match (a, b) {
        (serde_json::Value::Object(am), serde_json::Value::Object(bm)) => {
            let mut keys: BTreeMap<&str, ()> = BTreeMap::new();
            for k in am.keys() {
                keys.insert(k, ());
            }
            for k in bm.keys() {
                keys.insert(k, ());
            }
            for k in keys.keys() {
                let p = if path.is_empty() {
                    (*k).to_string()
                } else {
                    format!("{path}.{k}")
                };
                match (am.get(*k), bm.get(*k)) {
                    (Some(av), Some(bv)) => {
                        if let Some(d) = json_eq_first_diff(av, bv, &p) {
                            return Some(d);
                        }
                    }
                    (Some(av), None) => {
                        return Some((p, av.to_string(), "null".into()));
                    }
                    (None, Some(bv)) => {
                        return Some((p, "null".into(), bv.to_string()));
                    }
                    (None, None) => {}
                }
            }
            None
        }
        (serde_json::Value::Array(aa), serde_json::Value::Array(ba)) => {
            if aa.len() != ba.len() {
                return Some((
                    format!("{path}.len"),
                    aa.len().to_string(),
                    ba.len().to_string(),
                ));
            }
            for (i, (av, bv)) in aa.iter().zip(ba.iter()).enumerate() {
                if let Some(d) = json_eq_first_diff(av, bv, &format!("{path}[{i}]")) {
                    return Some(d);
                }
            }
            None
        }
        (av, bv) if av == bv => None,
        (av, bv) => Some((path.to_string(), av.to_string(), bv.to_string())),
    }
}

fn phase_is_terminal(v: &serde_json::Value) -> bool {
    v.get("phase").and_then(|p| p.as_str()) == Some("terminal")
}

fn winner_value(v: &serde_json::Value) -> serde_json::Value {
    v.get("winner").cloned().unwrap_or(serde_json::Value::Null)
}

/// Replay contract for one line's `state`.
///
/// At a line whose `phase` is `terminal`, only `phase` and `winner` are
/// compared; the rest of the state is post-mortem and engine-private
/// (`docs/trace-format.md` Conventions).
pub fn replay_state_diff(
    got: &serde_json::Value,
    want: &serde_json::Value,
) -> Option<(String, String, String)> {
    if phase_is_terminal(got) && phase_is_terminal(want) {
        let gw = winner_value(got);
        let ww = winner_value(want);
        if gw != ww {
            return Some(("winner".into(), gw.to_string(), ww.to_string()));
        }
        return None;
    }
    json_eq_first_diff(got, want, "")
}

/// `legal` is not compared when both sides are already `phase: terminal`.
pub fn replay_compare_legal(got: &serde_json::Value, want: &serde_json::Value) -> bool {
    !(phase_is_terminal(got) && phase_is_terminal(want))
}

/// Compact one-line JSON of a NeutralAction (replay diagnostics).
pub fn neutral_json(a: &NeutralAction) -> String {
    serde_json::to_string(a).unwrap_or_else(|_| format!("{a:?}"))
}

/// Symmetric-difference message for a `legal` divergence.
/// `arena=N actions; only arena: {…} trace=M actions; only trace: {…}`
pub fn legal_divergence_parts(
    ours: &[NeutralAction],
    theirs: &[NeutralAction],
) -> (String, String) {
    let only_arena: Vec<String> = ours
        .iter()
        .filter(|a| !theirs.contains(a))
        .map(neutral_json)
        .collect();
    let only_trace: Vec<String> = theirs
        .iter()
        .filter(|a| !ours.contains(a))
        .map(neutral_json)
        .collect();
    (
        format!(
            "{} actions; only arena: {}",
            ours.len(),
            only_arena.join(" ")
        ),
        format!(
            "{} actions; only trace: {}",
            theirs.len(),
            only_trace.join(" ")
        ),
    )
}

/// FNV-1a 64 of the canonical JSON bytes (sorted keys, compact).
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}
