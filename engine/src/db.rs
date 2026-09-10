//! `CardDb::load(root)` — every `cards/**/*.json` except the catalog, plus
//! the catalog for facts.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::card::{Card, CardId, CardOrCrest, CatalogRecord, Crest};
use crate::error::LoadError;
use crate::support;

#[derive(Debug, Clone)]
pub struct CardDb {
    pub cards: BTreeMap<CardId, Card>,
    pub crests: BTreeMap<String, Crest>,
    pub catalog: BTreeMap<String, CatalogRecord>,
    pub paths: BTreeMap<String, PathBuf>,
}

impl CardDb {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, LoadError> {
        let root = root.as_ref();
        let mut db = CardDb {
            cards: BTreeMap::new(),
            crests: BTreeMap::new(),
            catalog: BTreeMap::new(),
            paths: BTreeMap::new(),
        };
        let catalog_path = root.join("cards/official/catalog.json");
        if catalog_path.exists() {
            let text = fs::read_to_string(&catalog_path).map_err(|source| LoadError::Io {
                path: catalog_path.display().to_string(),
                source,
            })?;
            let raw: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text)
                .map_err(|source| LoadError::Catalog {
                    path: catalog_path.display().to_string(),
                    source,
                })?;
            for (k, v) in raw {
                if k.len() == 8 && k.bytes().all(|b| b.is_ascii_digit()) {
                    let rec: CatalogRecord =
                        serde_json::from_value(v).map_err(|source| LoadError::Catalog {
                            path: format!("{}#{k}", catalog_path.display()),
                            source,
                        })?;
                    db.catalog.insert(k, rec);
                }
            }
        }
        let cards_root = root.join("cards");
        if cards_root.exists() {
            walk_json(&cards_root, &mut |path| {
                if path.file_name().and_then(|s| s.to_str()) == Some("catalog.json") {
                    return Ok(());
                }
                db.load_file(path)
            })?;
        }
        Ok(db)
    }

    /// Load extra authored files (engine fixtures) without touching `cards/`.
    pub fn load_extra_dir(&mut self, dir: impl AsRef<Path>) -> Result<(), LoadError> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Ok(());
        }
        walk_json(dir, &mut |path| self.load_file(path))
    }

    fn load_file(&mut self, path: &Path) -> Result<(), LoadError> {
        let text = fs::read_to_string(path).map_err(|source| LoadError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let parsed: CardOrCrest =
            serde_json::from_str(&text).map_err(|source| LoadError::Parse {
                path: path.display().to_string(),
                source,
            })?;
        match parsed {
            CardOrCrest::Card(card) => {
                let id = card.id();
                let key = id.as_str();
                if let Some(prev) = self.paths.get(&key) {
                    return Err(LoadError::Duplicate {
                        id: key,
                        first: prev.display().to_string(),
                        second: path.display().to_string(),
                    });
                }
                if let Some(name) = card.unbound_refs().into_iter().next() {
                    return Err(LoadError::UnboundRef {
                        card: key.clone(),
                        name,
                    });
                }
                self.paths.insert(key, path.to_path_buf());
                self.cards.insert(id, card);
            }
            CardOrCrest::Crest(crest) => {
                let key = crest.id.clone();
                if let Some(prev) = self.paths.get(&key) {
                    return Err(LoadError::Duplicate {
                        id: key,
                        first: prev.display().to_string(),
                        second: path.display().to_string(),
                    });
                }
                if let Some(name) = crest.unbound_refs().into_iter().next() {
                    return Err(LoadError::UnboundRef {
                        card: key.clone(),
                        name,
                    });
                }
                self.paths.insert(key.clone(), path.to_path_buf());
                self.crests.insert(key, crest);
            }
        }
        Ok(())
    }

    pub fn card(&self, id: CardId) -> Result<&Card, LoadError> {
        self.cards.get(&id).ok_or(LoadError::MissingCard(id))
    }

    pub fn crest(&self, id: &str) -> Result<&Crest, LoadError> {
        self.crests
            .get(id)
            .ok_or_else(|| LoadError::MissingCrest(id.to_string()))
    }

    pub fn require_supported(&self, id: CardId) -> Result<&Card, LoadError> {
        let card = self.card(id)?;
        if let Some(u) = support::card_unsupported(card) {
            return Err(LoadError::Unsupported(u));
        }
        Ok(card)
    }

    pub fn has_card(&self, id: CardId) -> bool {
        self.cards.contains_key(&id)
    }
}

fn walk_json(
    dir: &Path,
    f: &mut impl FnMut(&Path) -> Result<(), LoadError>,
) -> Result<(), LoadError> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let rd = fs::read_dir(&cur).map_err(|source| LoadError::Io {
            path: cur.display().to_string(),
            source,
        })?;
        let mut ents: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        ents.sort();
        for p in ents {
            if p.is_dir() {
                if p.file_name().and_then(|s| s.to_str()) == Some("official") {
                    continue;
                }
                stack.push(p);
            } else if p.extension().and_then(|s| s.to_str()) == Some("json") {
                f(&p)?;
            }
        }
    }
    Ok(())
}
