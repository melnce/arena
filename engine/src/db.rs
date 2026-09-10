//! `CardDb::load(root)` — every `cards/**/*.json` except the catalog, plus
//! the catalog for facts.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::card::{
    Ability, AbilityZone, Card, CardId, CardOrCrest, CatalogRecord, Crest, EventName,
};
use crate::error::LoadError;
use crate::support;

#[derive(Debug, Clone)]
pub struct CardDb {
    pub cards: BTreeMap<CardId, Card>,
    pub crests: BTreeMap<String, Crest>,
    pub catalog: BTreeMap<String, CatalogRecord>,
    pub paths: BTreeMap<String, PathBuf>,
    /// Printed `When` abilities, keyed by `(event, zone)` → card ids.
    when_cards: HashMap<(EventName, AbilityZone), HashSet<CardId>>,
    /// Printed `When` abilities on crests, keyed by `(event, zone)` → crest ids.
    when_crests: HashMap<(EventName, AbilityZone), HashSet<String>>,
    /// Cards that print a start/end-of-turn ability in a given zone.
    /// `true` = startOfTurn, `false` = endOfTurn.
    boundary_cards: HashMap<(AbilityZone, bool), HashSet<CardId>>,
}

impl CardDb {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, LoadError> {
        let root = root.as_ref();
        let mut db = CardDb {
            cards: BTreeMap::new(),
            crests: BTreeMap::new(),
            catalog: BTreeMap::new(),
            paths: BTreeMap::new(),
            when_cards: HashMap::new(),
            when_crests: HashMap::new(),
            boundary_cards: HashMap::new(),
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
                db.load_file(path, false)
            })?;
        }
        db.rebuild_when_index();
        db.rebuild_boundary_index();
        Ok(db)
    }

    /// Load extra authored files (engine fixtures) without touching `cards/`.
    /// IDs already present from `cards/**` are skipped so stand-in fixtures do
    /// not collide once RD's card pool lands on the same tree.
    pub fn load_extra_dir(&mut self, dir: impl AsRef<Path>) -> Result<(), LoadError> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Ok(());
        }
        walk_json(dir, &mut |path| self.load_file(path, true))?;
        self.rebuild_when_index();
        self.rebuild_boundary_index();
        Ok(())
    }

    fn load_file(&mut self, path: &Path, skip_existing: bool) -> Result<(), LoadError> {
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
                    if skip_existing {
                        return Ok(());
                    }
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
                    if skip_existing {
                        return Ok(());
                    }
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

    /// Card ids that print a `When` for `(event, zone)`.
    pub fn card_has_when(&self, id: CardId, event: EventName, zone: AbilityZone) -> bool {
        self.when_cards
            .get(&(event, zone))
            .is_some_and(|set| set.contains(&id))
    }

    /// Crest ids that print a `When` for `(event, zone)`.
    pub fn crest_has_when(&self, id: &str, event: EventName, zone: AbilityZone) -> bool {
        self.when_crests
            .get(&(event, zone))
            .is_some_and(|set| set.contains(id))
    }

    /// Any printed card `When` for this `(event, zone)` — used to skip hand/deck.
    pub fn zone_has_when(&self, event: EventName, zone: AbilityZone) -> bool {
        self.when_cards
            .get(&(event, zone))
            .is_some_and(|set| !set.is_empty())
    }

    /// Any printed start/end-of-turn ability in `zone`. Skip deck/hand scans
    /// when the index is empty (Sandalphon's deck `startOfTurn` is the first
    /// deck-boundary card; decks without it cost nothing).
    pub fn zone_has_boundary(&self, zone: AbilityZone, start: bool) -> bool {
        self.boundary_cards
            .get(&(zone, start))
            .is_some_and(|set| !set.is_empty())
    }

    fn rebuild_when_index(&mut self) {
        let mut when_cards: HashMap<(EventName, AbilityZone), HashSet<CardId>> = HashMap::new();
        let mut when_crests: HashMap<(EventName, AbilityZone), HashSet<String>> = HashMap::new();
        for (id, card) in &self.cards {
            index_when_abilities(card.abilities(), |event, zone| {
                when_cards.entry((event, zone)).or_default().insert(*id);
            });
        }
        for (id, crest) in &self.crests {
            index_when_abilities(crest.abilities(), |event, zone| {
                when_crests
                    .entry((event, zone))
                    .or_default()
                    .insert(id.clone());
            });
        }
        self.when_cards = when_cards;
        self.when_crests = when_crests;
    }

    fn rebuild_boundary_index(&mut self) {
        let mut boundary_cards: HashMap<(AbilityZone, bool), HashSet<CardId>> = HashMap::new();
        for (id, card) in &self.cards {
            for a in card.abilities() {
                let start = match a {
                    Ability::StartOfTurn { .. } => Some(true),
                    Ability::EndOfTurn { .. } => Some(false),
                    _ => None,
                };
                if let Some(start) = start {
                    boundary_cards
                        .entry((a.zone(), start))
                        .or_default()
                        .insert(*id);
                }
            }
        }
        self.boundary_cards = boundary_cards;
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

fn index_when_abilities(abilities: &[Ability], mut on_when: impl FnMut(EventName, AbilityZone)) {
    for a in abilities {
        if let Ability::When { event, .. } = a {
            on_when(*event, a.zone());
        }
    }
}
