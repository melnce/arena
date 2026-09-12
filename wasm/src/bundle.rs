//! Baked card bundle (`build.rs` → `OUT_DIR/bundle.json`).

use std::collections::HashSet;
use std::sync::OnceLock;

use arena_engine::card::{Ability, Class, Crest, Effect, Mode, Tribe};
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
                "crests": card_crest_entries(db, card),
                "forms": card_forms(card),
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
            "faith": crest.faith,
            "grantedBy": crest_granted_by(crest),
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

fn card_forms(card: &Card) -> Vec<serde_json::Value> {
    card.modes()
        .iter()
        .map(|mode| match mode {
            Mode::Enhance { cost, printed, .. } => json!({
                "kind": "enhance",
                "cost": cost,
                "printed": printed,
            }),
            Mode::Accelerate { cost, printed, .. } => json!({
                "kind": "accelerate",
                "cost": cost,
                "printed": printed,
            }),
            Mode::Crystallize { cost, printed, .. } => json!({
                "kind": "crystallize",
                "cost": cost,
                "printed": printed,
            }),
        })
        .collect()
}

fn crest_gain_ids(card: &Card) -> Vec<String> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    collect_crest_gains(card.abilities(), &mut ids, &mut seen);
    for mode in card.modes() {
        match mode {
            Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => {
                collect_crest_gains_effects(effects, &mut ids, &mut seen);
            }
            Mode::Crystallize { abilities, .. } => {
                if let Some(abs) = abilities {
                    collect_crest_gains(abs, &mut ids, &mut seen);
                }
            }
        }
    }
    ids
}

fn card_crest_ids(db: &CardDb, card: &Card) -> Vec<String> {
    let mut ids = crest_gain_ids(card);
    let seen: HashSet<String> = ids.iter().cloned().collect();
    for (crest_id, crest) in &db.crests {
        if crest.granted_by.contains(&card.id()) && !seen.contains(crest_id) {
            ids.push(crest_id.clone());
        }
    }
    ids
}

fn card_crest_entries(db: &CardDb, card: &Card) -> Vec<serde_json::Value> {
    card_crest_ids(db, card)
        .into_iter()
        .map(|id| {
            let crest = db
                .crest(&id)
                .unwrap_or_else(|e| panic!("crest {id} must resolve: {e}"));
            json!({
                "id": crest.id,
                "name": crest.name,
                "text": crest.text,
                "faith": crest.faith,
                "grantedBy": crest_granted_by(crest),
            })
        })
        .collect()
}

fn crest_granted_by(crest: &Crest) -> String {
    crest
        .granted_by
        .first()
        .map(|id| id.as_str())
        .unwrap_or_default()
}

fn collect_crest_gains(abilities: &[Ability], ids: &mut Vec<String>, seen: &mut HashSet<String>) {
    for ability in abilities {
        collect_crest_gains_effects(ability.effects(), ids, seen);
    }
}

fn collect_crest_gains_effects(
    effects: &[Effect],
    ids: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    for effect in effects {
        if let Effect::Crest { gain, .. } = effect {
            if seen.insert(gain.0.clone()) {
                ids.push(gain.0.clone());
            }
        }
        match effect {
            Effect::Choose {
                options: Some(opts),
                ..
            } => {
                for opt in opts {
                    collect_crest_gains_effects(&opt.effects, ids, seen);
                }
            }
            Effect::Sequence { steps, .. } => {
                for step in steps {
                    collect_crest_gains_effects(&step.effects, ids, seen);
                }
            }
            Effect::If {
                then, else_effects, ..
            } => {
                collect_crest_gains_effects(then, ids, seen);
                if let Some(els) = else_effects {
                    collect_crest_gains_effects(els, ids, seen);
                }
            }
            Effect::Seq { effects, .. }
            | Effect::Repeat { effects, .. }
            | Effect::Pay { effects, .. } => {
                collect_crest_gains_effects(effects, ids, seen);
            }
            Effect::GrantAbility { ability, .. } => {
                collect_crest_gains_effects(ability.effects(), ids, seen);
            }
            _ => {}
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn parse_text(id: &str) -> Value {
        serde_json::from_str(&card_text(id).expect(id)).expect(id)
    }

    #[test]
    fn card_text_crests_and_forms_sweep() {
        let db = card_db();
        let mut gain_text_cards = 0;
        let mut walked_cards = 0;
        let mut extra_ids = Vec::new();

        for (cid, card) in &db.cards {
            let id = cid.as_str();
            let v = parse_text(&id);
            let crests = v["crests"]
                .as_array()
                .unwrap_or_else(|| panic!("{id} crests"));
            let walked: HashSet<String> = crest_gain_ids(card).into_iter().collect();
            if !walked.is_empty() {
                walked_cards += 1;
            }
            assert_eq!(
                crests.len(),
                card_crest_ids(db, card).len(),
                "{id} crest count"
            );
            for crest in crests {
                let crest_id = crest["id"]
                    .as_str()
                    .unwrap_or_else(|| panic!("{id} crest id"));
                let resolved = db
                    .crest(crest_id)
                    .unwrap_or_else(|e| panic!("{id} → {crest_id} must resolve: {e}"));
                assert!(
                    !resolved.name.is_empty() && !resolved.text.is_empty(),
                    "{crest_id} empty name/text"
                );
                assert!(
                    !crest["name"].as_str().unwrap_or("").is_empty()
                        && !crest["text"].as_str().unwrap_or("").is_empty(),
                    "{id} serialized crest empty"
                );
                if !walked.contains(crest_id) {
                    extra_ids.push(crest_id.to_string());
                }
            }
            if card.text().to_ascii_lowercase().contains("gain crest:") {
                gain_text_cards += 1;
                assert!(
                    !crests.is_empty(),
                    "{id} prints Gain crest: but cardText.crests is empty"
                );
            }
        }

        assert_eq!(
            gain_text_cards, 35,
            "cards whose text matches (?i)gain crest:"
        );
        assert_eq!(walked_cards, 38, "cards that walk a crest gain");
        extra_ids.sort();
        assert_eq!(
            extra_ids,
            [
                "faith:10614120",
                "faith:10624120",
                "faith:10634120",
                "faith:10664120"
            ]
        );

        let maj = parse_text("10622310");
        let maj_crests = maj["crests"].as_array().expect("10622310 crests");
        assert_eq!(maj_crests.len(), 1);
        assert_eq!(maj_crests[0]["id"], "crest:10622310");
        assert_eq!(maj_crests[0]["grantedBy"], "10622310");
        assert_eq!(maj_crests[0]["name"], "Crest: Majestic Conquest");
        assert!(maj_crests[0]["text"]
            .as_str()
            .unwrap_or("")
            .starts_with("Countdown (2)"));

        let acc = parse_text("10844120");
        let acc_forms = acc["forms"].as_array().expect("10844120 forms");
        assert_eq!(acc_forms.len(), 1);
        assert_eq!(acc_forms[0]["kind"], "accelerate");
        assert_eq!(acc_forms[0]["cost"], 3);
        assert_eq!(
            acc_forms[0]["printed"],
            "Accelerate (3): Gain 1 max play point."
        );

        let cry = parse_text("10661110");
        let cry_forms = cry["forms"].as_array().expect("10661110 forms");
        assert_eq!(cry_forms.len(), 1);
        assert_eq!(cry_forms[0]["kind"], "crystallize");
        assert_eq!(cry_forms[0]["cost"], 2);
        assert_eq!(
            cry_forms[0]["printed"],
            "Crystallize (2): Countdown (3)\nLast Words: Summon a Prostrating Coward."
        );
    }
}
