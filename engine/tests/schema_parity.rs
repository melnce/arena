//! `schema_parity`: every `cards/**` file parses; broken fixtures are rejected.

use arena_engine::{m1_unsupported_list, CardOrCrest};
use std::fs;
use std::path::Path;

mod common;

fn walk_json(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut ents: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    ents.sort_by_key(|e| e.file_name());
    for e in ents {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|s| s.to_str()) == Some("official") {
                continue;
            }
            walk_json(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(p);
        }
    }
}

#[test]
fn schema_parity_every_cards_file_parses() {
    let mut files = Vec::new();
    walk_json(&common::repo_root().join("cards"), &mut files);
    assert!(!files.is_empty(), "expected cards/** json");
    for p in files {
        if p.file_name().and_then(|s| s.to_str()) == Some("catalog.json") {
            continue;
        }
        let text = fs::read_to_string(&p).unwrap();
        let parsed: Result<CardOrCrest, _> = serde_json::from_str(&text);
        if parsed.is_err() {
            if let Err(e) = serde_json::from_str::<arena_engine::Card>(&text) {
                panic!("parse {}: {e}", p.display());
            }
            panic!("parse {} as crest/card: {:?}", p.display(), parsed.err());
        }
    }
}

#[test]
fn schema_parity_fixture_cards_parse() {
    let mut files = Vec::new();
    walk_json(&common::fixtures_dir().join("cards"), &mut files);
    assert!(!files.is_empty());
    for p in files {
        let text = fs::read_to_string(&p).unwrap();
        let parsed: Result<CardOrCrest, _> = serde_json::from_str(&text);
        if parsed.is_err() {
            if let Err(e) = serde_json::from_str::<arena_engine::Card>(&text) {
                panic!("parse {}: {e}", p.display());
            }
            panic!("parse {} as crest/card: {:?}", p.display(), parsed.err());
        }
    }
}

#[test]
fn schema_parity_rejects_unknown_field() {
    let t = fs::read_to_string(common::fixtures_dir().join("invalid/unknown_field.json")).unwrap();
    assert!(serde_json::from_str::<CardOrCrest>(&t).is_err());
}

#[test]
fn schema_parity_rejects_wrong_enum() {
    let t = fs::read_to_string(common::fixtures_dir().join("invalid/wrong_enum.json")).unwrap();
    assert!(serde_json::from_str::<CardOrCrest>(&t).is_err());
}

#[test]
fn schema_parity_rejects_pool_selector_missing_zone() {
    let t =
        fs::read_to_string(common::fixtures_dir().join("invalid/pool_selector_missing_zone.json"))
            .unwrap();
    assert!(serde_json::from_str::<CardOrCrest>(&t).is_err());
}

#[test]
fn schema_parity_rejects_faith_in_crest_gain() {
    let t = fs::read_to_string(common::fixtures_dir().join("invalid/faith_in_crest_gain.json"))
        .unwrap();
    assert!(serde_json::from_str::<CardOrCrest>(&t).is_err());
}

fn assert_invalid_side_card(path: &Path, want_id: &str) {
    let text = fs::read_to_string(path).unwrap();
    let card: arena_engine::Card = serde_json::from_str(&text).unwrap();
    assert_eq!(card.id().as_str(), want_id);
    let detail = card
        .invalid_side_usage()
        .unwrap_or_else(|| panic!("{} should fail invalid_side_usage", want_id));
    assert!(detail.contains("enterCountAtLeast"), "{want_id}: {detail}");
    let label = format!("test/{want_id}.json");
    match arena_engine::CardDb::from_json(&[(label, text)]) {
        Err(arena_engine::LoadError::InvalidSide { card, detail: d }) => {
            assert_eq!(card, want_id);
            assert!(d.contains("enterCountAtLeast"), "{d}");
        }
        Ok(_) => panic!("expected InvalidSide for {want_id}"),
        Err(other) => panic!("expected InvalidSide, got {other:?}"),
    }
}

#[test]
fn schema_parity_rejects_enter_count_at_least_side_any() {
    assert_invalid_side_card(
        &common::fixtures_dir().join("invalid/enter_count_side_any.json"),
        "89501991",
    );
}

#[test]
fn schema_parity_rejects_nested_grant_ability_enter_count_side_any() {
    assert_invalid_side_card(
        &common::fixtures_dir().join("invalid/grant_ability_nested_enter_count_side_any.json"),
        "89501992",
    );
}

#[test]
fn m1_unsupported_list_is_nonempty() {
    let v = m1_unsupported_list();
    assert!(!v.is_empty());
    assert!(!v.contains(&"on:invoked"));
    assert!(!v.contains(&"op:transform"));
    assert!(!v.contains(&"op:invoke"));
}

#[test]
fn card_db_missing_card_is_typed() {
    let db = common::load_db();
    let missing = arena_engine::CardId::parse("19999999").unwrap();
    match db.card(missing) {
        Err(arena_engine::LoadError::MissingCard(id)) => assert_eq!(id, missing),
        other => panic!("expected MissingCard, got {other:?}"),
    }
}
