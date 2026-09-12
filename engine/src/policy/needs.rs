//! State-free per-card resource needs, built once per [`CardDb`].
//!
//! Walks printed abilities and modes the same way `info::walk_effects` does
//! (Pay / If / Seq / Repeat / Choose options / Sequence / nested effects),
//! plus Rally and named-counter thresholds. Combo is not collected: it
//! resets every turn, so at H0's leaves it is always 0.

use std::collections::HashMap;

use crate::card::{
    Ability, Amount, Card, CardId, Condition, CounterKey, Effect, Mode, NamedCounter, PayResource,
};
use crate::db::CardDb;

/// Thresholds and flags one card cares about.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CardNeeds {
    /// Every constant `Pay { Shadows }` and `CounterAtLeast { Shadows }`.
    pub shadows: Vec<i32>,
    /// Every constant `Pay { Earth }` and `CounterAtLeast { Earth }`.
    pub earth: Vec<i32>,
    /// Every constant `Pay { Faith }` and `CounterAtLeast { Faith }`.
    pub faith: Vec<i32>,
    /// Every constant `Condition::Rally { n }`.
    pub rally: Vec<i32>,
    /// The card prints an `Ability::Spellboost`.
    pub spellboost: bool,
    /// The card prints an `Ability::LastWords`.
    pub last_words: bool,
}

impl CardNeeds {
    fn is_empty(&self) -> bool {
        self.shadows.is_empty()
            && self.earth.is_empty()
            && self.faith.is_empty()
            && self.rally.is_empty()
            && !self.spellboost
            && !self.last_words
    }

    /// Any shadows / earth / faith / rally threshold (used by the live term).
    pub fn has_threshold(&self) -> bool {
        !self.shadows.is_empty()
            || !self.earth.is_empty()
            || !self.faith.is_empty()
            || !self.rally.is_empty()
    }
}

/// A Pay / Rally / CounterAtLeast whose `Amount` was not a constant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedAmount {
    pub id: CardId,
    pub kind: &'static str,
}

/// `CardId` → [`CardNeeds`] for cards that have at least one need or flag.
#[derive(Debug, Clone, Default)]
pub struct NeedsTable {
    map: HashMap<CardId, CardNeeds>,
    skipped: Vec<SkippedAmount>,
}

impl NeedsTable {
    pub fn build(db: &CardDb) -> Self {
        let mut map = HashMap::new();
        let mut skipped = Vec::new();
        for (id, card) in &db.cards {
            let (needs, mut skip) = collect_card(card);
            skipped.append(&mut skip);
            if !needs.is_empty() {
                map.insert(*id, needs);
            }
        }
        skipped.sort_by_key(|s| (s.id, s.kind));
        Self { map, skipped }
    }

    pub fn get(&self, id: CardId) -> Option<&CardNeeds> {
        self.map.get(&id)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Non-constant amounts the walk skipped (for the PR report).
    pub fn skipped(&self) -> &[SkippedAmount] {
        &self.skipped
    }

    pub fn iter(&self) -> impl Iterator<Item = (CardId, &CardNeeds)> {
        self.map.iter().map(|(id, n)| (*id, n))
    }
}

fn collect_card(card: &Card) -> (CardNeeds, Vec<SkippedAmount>) {
    let mut needs = CardNeeds::default();
    let mut skipped = Vec::new();
    let id = card.id();
    for a in card.abilities() {
        walk_ability(id, a, &mut needs, &mut skipped);
    }
    for mode in card.modes() {
        match mode {
            Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => {
                walk_effects(id, effects, &mut needs, &mut skipped);
            }
            Mode::Crystallize { abilities, .. } => {
                if let Some(abs) = abilities {
                    for a in abs {
                        walk_ability(id, a, &mut needs, &mut skipped);
                    }
                }
            }
        }
    }
    (needs, skipped)
}

fn walk_ability(
    id: CardId,
    ability: &Ability,
    needs: &mut CardNeeds,
    skipped: &mut Vec<SkippedAmount>,
) {
    if matches!(ability, Ability::Spellboost { .. }) {
        needs.spellboost = true;
    }
    if matches!(ability, Ability::LastWords { .. }) {
        needs.last_words = true;
    }
    if let Some(c) = ability.when_cond() {
        walk_condition(id, c, needs, skipped);
    }
    walk_effects(id, ability.effects(), needs, skipped);
}

fn walk_effects(
    id: CardId,
    effects: &[Effect],
    needs: &mut CardNeeds,
    skipped: &mut Vec<SkippedAmount>,
) {
    for e in effects {
        walk_effect(id, e, needs, skipped);
    }
}

fn walk_effect(
    id: CardId,
    effect: &Effect,
    needs: &mut CardNeeds,
    skipped: &mut Vec<SkippedAmount>,
) {
    if let Some(c) = effect.when_cond() {
        walk_condition(id, c, needs, skipped);
    }
    match effect {
        Effect::Pay {
            resource,
            amount,
            effects,
            ..
        } => {
            push_pay(id, *resource, amount, needs, skipped);
            walk_effects(id, effects, needs, skipped);
        }
        Effect::If {
            cond,
            then,
            else_effects,
            ..
        } => {
            walk_condition(id, cond, needs, skipped);
            walk_effects(id, then, needs, skipped);
            if let Some(els) = else_effects {
                walk_effects(id, els, needs, skipped);
            }
        }
        Effect::Seq { effects, .. } | Effect::Repeat { effects, .. } => {
            walk_effects(id, effects, needs, skipped);
        }
        Effect::Choose {
            options: Some(opts),
            ..
        } => {
            for o in opts {
                walk_effects(id, &o.effects, needs, skipped);
            }
        }
        Effect::Sequence { steps, .. } => {
            for s in steps {
                walk_effects(id, &s.effects, needs, skipped);
            }
        }
        _ => {}
    }
}

fn walk_condition(
    id: CardId,
    cond: &Condition,
    needs: &mut CardNeeds,
    skipped: &mut Vec<SkippedAmount>,
) {
    match cond {
        Condition::All { all } => {
            for c in all {
                walk_condition(id, c, needs, skipped);
            }
        }
        Condition::Any { any } => {
            for c in any {
                walk_condition(id, c, needs, skipped);
            }
        }
        Condition::Not { not } => walk_condition(id, not, needs, skipped),
        Condition::Rally { rally } => match amount_int(&rally.n) {
            Some(n) => needs.rally.push(n),
            None => skipped.push(SkippedAmount { id, kind: "rally" }),
        },
        Condition::CounterAtLeast { counter_at_least } => {
            let key = match &counter_at_least.key {
                CounterKey::Named(NamedCounter::Shadows) => Some(("counter shadows", "shadows")),
                CounterKey::Named(NamedCounter::Earth) => Some(("counter earth", "earth")),
                CounterKey::Named(NamedCounter::Faith) => Some(("counter faith", "faith")),
                _ => None,
            };
            if let Some((skip_kind, dest)) = key {
                match amount_int(&counter_at_least.n) {
                    Some(n) => match dest {
                        "shadows" => needs.shadows.push(n),
                        "earth" => needs.earth.push(n),
                        "faith" => needs.faith.push(n),
                        _ => {}
                    },
                    None => skipped.push(SkippedAmount {
                        id,
                        kind: skip_kind,
                    }),
                }
            }
        }
        _ => {}
    }
}

fn push_pay(
    id: CardId,
    resource: PayResource,
    amount: &Amount,
    needs: &mut CardNeeds,
    skipped: &mut Vec<SkippedAmount>,
) {
    let (skip_kind, dest): (&'static str, Option<&str>) = match resource {
        PayResource::Shadows => ("pay shadows", Some("shadows")),
        PayResource::Earth => ("pay earth", Some("earth")),
        PayResource::Faith => ("pay faith", Some("faith")),
        PayResource::Pp => return,
    };
    match amount_int(amount) {
        Some(n) => match dest {
            Some("shadows") => needs.shadows.push(n),
            Some("earth") => needs.earth.push(n),
            Some("faith") => needs.faith.push(n),
            _ => {}
        },
        None => skipped.push(SkippedAmount {
            id,
            kind: skip_kind,
        }),
    }
}

fn amount_int(amount: &Amount) -> Option<i32> {
    match amount {
        Amount::Int(n) => Some(*n),
        _ => None,
    }
}
