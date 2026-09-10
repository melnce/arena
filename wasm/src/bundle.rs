//! Baked card bundle (`build.rs` → `OUT_DIR/bundle.json`).

use std::sync::OnceLock;

use arena_engine::{CardDb, CardId};
use serde_json::json;

const BUNDLE_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/bundle.json"));
const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/version.txt"));

pub fn bundle_bytes() -> usize {
    BUNDLE_JSON.len()
}

pub fn version() -> &'static str {
    VERSION.trim()
}

pub fn card_db() -> &'static CardDb {
    static DB: OnceLock<CardDb> = OnceLock::new();
    DB.get_or_init(|| {
        let map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(BUNDLE_JSON).expect("card bundle json");
        let entries: Vec<(String, String)> = map
            .into_iter()
            .map(|(path, value)| {
                (
                    path,
                    serde_json::to_string(&value).expect("re-serialize card"),
                )
            })
            .collect();
        CardDb::from_json(&entries).expect("card bundle")
    })
}

pub fn card_text(id: &str) -> Result<String, String> {
    let db = card_db();
    if let Some(cid) = CardId::parse(id) {
        if let Ok(card) = db.card(cid) {
            return serde_json::to_string(&json!({
                "id": id,
                "name": card.name(),
                "text": card.text(),
                "kind": kind_str(card.kind()),
                "cost": card.cost(),
            }))
            .map_err(|e| e.to_string());
        }
    }
    if let Ok(crest) = db.crest(id) {
        return serde_json::to_string(&json!({
            "id": id,
            "name": crest.name,
            "text": crest.text,
            "kind": if crest.faith { "faith" } else { "crest" },
            "cost": serde_json::Value::Null,
        }))
        .map_err(|e| e.to_string());
    }
    Err(format!("unknown card id {id}"))
}

pub fn bundle_info() -> String {
    let db = card_db();
    serde_json::to_string(&json!({
        "cards": db.cards.len(),
        "crests": db.crests.len(),
        "bytes": bundle_bytes(),
    }))
    .expect("bundleInfo")
}

fn kind_str(kind: arena_engine::card::CardKind) -> &'static str {
    match kind {
        arena_engine::card::CardKind::Follower => "follower",
        arena_engine::card::CardKind::Spell => "spell",
        arena_engine::card::CardKind::Amulet => "amulet",
    }
}
