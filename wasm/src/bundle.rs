//! Baked card bundle (`build.rs` → `OUT_DIR/bundle.json`).

use std::sync::OnceLock;

use arena_engine::card::{Ability, Class, Effect, Mode, Tribe};
use arena_engine::{Card, CardDb, CardId};
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
                "class": class_str(card.class()),
                "tribes": card.tribes().iter().map(|t| tribe_str(*t)).collect::<Vec<_>>(),
                "set": card.set(),
                "tags": text_tags(card),
                "cost": card.cost(),
                "attack": card.attack(),
                "defense": card.defense(),
                "modes": choose_printed(card),
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

fn class_str(class: Class) -> &'static str {
    match class {
        Class::Neutral => "Neutral",
        Class::Forestcraft => "Forestcraft",
        Class::Swordcraft => "Swordcraft",
        Class::Runecraft => "Runecraft",
        Class::Dragoncraft => "Dragoncraft",
        Class::Abysscraft => "Abysscraft",
        Class::Havencraft => "Havencraft",
        Class::Portalcraft => "Portalcraft",
    }
}

fn tribe_str(t: Tribe) -> &'static str {
    match t {
        Tribe::Anathema => "Anathema",
        Tribe::Artifact => "Artifact",
        Tribe::Departed => "Departed",
        Tribe::EarthSigil => "Earth Sigil",
        Tribe::Encroacher => "Encroacher",
        Tribe::Golem => "Golem",
        Tribe::Loot => "Loot",
        Tribe::Marine => "Marine",
        Tribe::Mysteria => "Mysteria",
        Tribe::Officer => "Officer",
        Tribe::Pixie => "Pixie",
        Tribe::Puppetry => "Puppetry",
    }
}

fn text_tags(card: &Card) -> Vec<String> {
    let mut tags: Vec<String> = card.printed_trigger_tags().into_iter().collect();
    let has_ongoing = card
        .abilities()
        .iter()
        .any(|a| matches!(a, Ability::Static { .. }))
        || card.text().to_ascii_lowercase().contains("ongoing");
    if has_ongoing && !tags.iter().any(|t| t == "ongoing") {
        tags.push("ongoing".into());
    }
    tags
}

fn kind_str(kind: arena_engine::card::CardKind) -> &'static str {
    match kind {
        arena_engine::card::CardKind::Follower => "follower",
        arena_engine::card::CardKind::Spell => "spell",
        arena_engine::card::CardKind::Amulet => "amulet",
    }
}

fn choose_printed(card: &Card) -> Vec<String> {
    let mut out = Vec::new();
    collect_choose_abilities(card.abilities(), &mut out);
    for mode in card.modes() {
        match mode {
            Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => {
                collect_choose_effects(effects, &mut out);
            }
            Mode::Crystallize { abilities, .. } => {
                if let Some(abs) = abilities {
                    collect_choose_abilities(abs, &mut out);
                }
            }
        }
    }
    out
}

fn collect_choose_abilities(abilities: &[Ability], out: &mut Vec<String>) {
    for a in abilities {
        collect_choose_effects(a.effects(), out);
    }
}

fn collect_choose_effects(effects: &[Effect], out: &mut Vec<String>) {
    for e in effects {
        if let Effect::Choose {
            options: Some(opts),
            ..
        } = e
        {
            for o in opts {
                if !out.contains(&o.printed) {
                    out.push(o.printed.clone());
                }
                collect_choose_effects(&o.effects, out);
            }
        }
        if let Effect::Sequence { steps, .. } = e {
            for s in steps {
                collect_choose_effects(&s.effects, out);
            }
        }
        if let Effect::If {
            then, else_effects, ..
        } = e
        {
            collect_choose_effects(then, out);
            if let Some(els) = else_effects {
                collect_choose_effects(els, out);
            }
        }
        if let Effect::Seq { effects, .. }
        | Effect::Repeat { effects, .. }
        | Effect::Pay { effects, .. } = e
        {
            collect_choose_effects(effects, out);
        }
    }
}
