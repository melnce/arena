//! Mulligan keep tables and deck fingerprints for learned mulligan policy.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use crate::card::CardId;
use crate::ids::PlayerId;
use crate::state::PlayerState;

/// Canonical deck fingerprint: distinct ids sorted ascending as `<id>x<count>`, joined with `,`.
pub fn deck_fingerprint_counts(counts: &BTreeMap<CardId, u32>) -> String {
    counts
        .iter()
        .map(|(id, n)| format!("{}x{}", id, n))
        .collect::<Vec<_>>()
        .join(",")
}

/// Multiset of card ids in `hand` plus `deck` (opening mulligan: 40 cards total).
pub fn deck_fingerprint_player(p: &PlayerState) -> String {
    let mut counts: BTreeMap<CardId, u32> = BTreeMap::new();
    for inst in p.hand.iter().chain(p.deck.iter()) {
        counts.entry(inst.card).and_modify(|n| *n += 1).or_insert(1);
    }
    deck_fingerprint_counts(&counts)
}

/// Per-seat keep map: card id string → keep (`true`) or send back (`false`).
pub type SeatTable = BTreeMap<String, bool>;

#[derive(Debug, Clone)]
pub struct DeckMulligan {
    pub first: SeatTable,
    pub second: SeatTable,
}

/// Version-1 mulligan keep table (JSON). Only `version` and per-deck `first` / `second` are read.
#[derive(Debug, Clone)]
pub struct MulliganTable {
    pub path: String,
    pub decks: BTreeMap<String, DeckMulligan>,
}

impl MulliganTable {
    /// Load a keep table from disk. Errors name `path` and the field, like [`ValueNet::load`].
    pub fn load(path: impl AsRef<Path>) -> Result<Arc<MulliganTable>, String> {
        let path = path.as_ref();
        let name = path.display().to_string();
        let text = std::fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
        Self::from_json_named(&name, &text).map(Arc::new)
    }

    pub fn from_json_named(name: &str, text: &str) -> Result<MulliganTable, String> {
        let root: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("{name}: {e}"))?;
        let version = root
            .get("version")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| format!("{name}: missing version"))?;
        if version != 1 {
            return Err(format!("{name}: unsupported version {version}"));
        }
        let decks_obj = root
            .get("decks")
            .and_then(|v| v.as_object())
            .ok_or_else(|| format!("{name}: missing decks"))?;
        let mut decks = BTreeMap::new();
        for (fp, entry) in decks_obj {
            let first = parse_seat(entry.get("first"), name, fp, "first")?;
            let second = parse_seat(entry.get("second"), name, fp, "second")?;
            decks.insert(fp.clone(), DeckMulligan { first, second });
        }
        Ok(MulliganTable {
            path: name.to_string(),
            decks,
        })
    }

    pub fn lookup(&self, fingerprint: &str) -> Option<&DeckMulligan> {
        self.decks.get(fingerprint)
    }
}

fn parse_seat(
    v: Option<&serde_json::Value>,
    path: &str,
    fp: &str,
    seat: &str,
) -> Result<SeatTable, String> {
    let obj = v
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("{path}: decks.{fp}.{seat} missing"))?;
    let mut out = BTreeMap::new();
    for (k, v) in obj {
        let keep = v
            .as_bool()
            .ok_or_else(|| format!("{path}: decks.{fp}.{seat}.{k} must be bool"))?;
        out.insert(k.clone(), keep);
    }
    Ok(out)
}

/// Seat label for table lookup: `first` when the player went first, else `second`.
pub fn mulligan_seat(first: PlayerId, me: PlayerId) -> &'static str {
    if first == me {
        "first"
    } else {
        "second"
    }
}
