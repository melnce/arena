//! The engine tests' state corpus is a named list, not a directory glob.

mod common;
use common::*;

#[test]
fn pinned_corpus_stems_exist() {
    let dir = repo_root().join("oracle/decks");
    let mut missing = Vec::new();
    for stem in PINNED_CORPUS_STEMS {
        if !dir.join(format!("{stem}.json")).is_file() {
            missing.push(*stem);
        }
    }
    assert!(
        missing.is_empty(),
        "pinned corpus stem(s) missing from oracle/decks/: {missing:?} \
         — a deletion must fail as a deletion, not silently shrink the corpus"
    );
    assert_eq!(
        PINNED_CORPUS_STEMS.len(),
        16,
        "pinned corpus is the sixteen pre-#64 decks"
    );
}

#[test]
fn pinned_corpus_loads_sixteen_ready_decks() {
    let db = load_db();
    let decks = load_pinned_corpus_decks(&db);
    assert_eq!(decks.len(), 16);
}

#[test]
fn pinned_corpus_is_pools_real_and_synthetic() {
    let path = repo_root().join("oracle/decks/POOLS.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let v: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("POOLS.json: {e}"));
    let mut from_pools = Vec::new();
    for key in ["real", "synthetic"] {
        let arr = v
            .get(key)
            .and_then(|x| x.as_array())
            .unwrap_or_else(|| panic!("POOLS.json missing {key} array"));
        for item in arr {
            let s = item
                .as_str()
                .unwrap_or_else(|| panic!("POOLS.json {key} entry not a string"));
            from_pools.push(s.to_string());
        }
    }
    from_pools.sort();
    let mut pinned: Vec<String> = PINNED_CORPUS_STEMS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    pinned.sort();
    assert_eq!(
        pinned, from_pools,
        "PINNED_CORPUS_STEMS is the pre-#64 real∪synthetic set. \
         If POOLS.json real/synthetic changed, do not silently retarget \
         the suite — that is a separate re-baseline."
    );
}
