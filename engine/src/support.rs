//! Honest-stub walk: a card whose data uses an unimplemented construct is
//! `Unsupported { card, construct }` at game construction (never a silent no-op).

use std::collections::BTreeSet;

use crate::card::{
    Ability, Card, CardId, CardSource, Condition, Effect, FuseResult, Mode, Selector,
};
use crate::db::CardDb;
use crate::error::Unsupported;

/// Constructs the M1 engine will not resolve. Loading them into a game fails.
pub fn card_unsupported(card: &Card) -> Option<Unsupported> {
    let id = card.id().as_str();
    for a in card.abilities() {
        if let Some(c) = ability_unsupported(a) {
            return Some(Unsupported {
                card: id,
                construct: c,
            });
        }
    }
    for m in card.modes() {
        if let Some(c) = mode_unsupported(m) {
            return Some(Unsupported {
                card: id,
                construct: c,
            });
        }
    }
    None
}

fn mode_unsupported(mode: &Mode) -> Option<String> {
    match mode {
        Mode::Crystallize { abilities, .. } => {
            if let Some(abs) = abilities {
                for a in abs {
                    if let Some(c) = ability_unsupported(a) {
                        return Some(c);
                    }
                }
            }
            None
        }
        Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => {
            for e in effects {
                if let Some(c) = effect_unsupported(e) {
                    return Some(c);
                }
            }
            None
        }
    }
}

fn ability_unsupported(a: &Ability) -> Option<String> {
    match a {
        Ability::When { event, .. } => {
            // All 16 events are implemented; keep the hook for future cuts.
            let _ = event;
            walk_ability_effects(a)
        }
        _ => walk_ability_effects(a),
    }
}

fn walk_ability_effects(a: &Ability) -> Option<String> {
    for e in a.effects() {
        if let Some(c) = effect_unsupported(e) {
            return Some(c);
        }
    }
    None
}

fn effect_unsupported(e: &Effect) -> Option<String> {
    match e {
        Effect::Counter {
            key: crate::card::CounterKey::Named(crate::card::NamedCounter::SkyboundHand),
            ..
        } => Some("op:counter skyboundHand".into()),
        other => walk_nested(other),
    }
}

fn walk_nested(e: &Effect) -> Option<String> {
    match e {
        Effect::Pay { effects, .. }
        | Effect::Seq { effects, .. }
        | Effect::Repeat { effects, .. }
        | Effect::RandomSplit { effects, .. } => {
            for x in effects {
                if let Some(c) = effect_unsupported(x) {
                    return Some(c);
                }
            }
            None
        }
        Effect::If {
            then,
            else_effects,
            cond,
            ..
        } => {
            if let Some(c) = condition_unsupported(cond) {
                return Some(c);
            }
            for x in then {
                if let Some(c) = effect_unsupported(x) {
                    return Some(c);
                }
            }
            if let Some(els) = else_effects {
                for x in els {
                    if let Some(c) = effect_unsupported(x) {
                        return Some(c);
                    }
                }
            }
            None
        }
        Effect::Choose { options, .. } => {
            if let Some(opts) = options {
                for o in opts {
                    for x in &o.effects {
                        if let Some(c) = effect_unsupported(x) {
                            return Some(c);
                        }
                    }
                }
            }
            None
        }
        Effect::Sequence { steps, .. } => {
            for s in steps {
                for x in &s.effects {
                    if let Some(c) = effect_unsupported(x) {
                        return Some(c);
                    }
                }
            }
            None
        }
        Effect::Summon { card, .. } | Effect::AddToHand { card, .. } => source_unsupported(card),
        Effect::Damage { select, .. }
        | Effect::Restore { select, .. }
        | Effect::Buff { select, .. }
        | Effect::Select { select, .. }
        | Effect::Destroy { select, .. }
        | Effect::Banish { select, .. }
        | Effect::ReturnToHand { select, .. }
        | Effect::ReturnToDeck { select, .. }
        | Effect::Discard { select, .. }
        | Effect::Evolve { select, .. }
        | Effect::GrantTraits { select, .. }
        | Effect::RemoveTraits { select, .. }
        | Effect::RemoveAbilities { select, .. }
        | Effect::Cost { select, .. }
        | Effect::Countdown { select, .. }
        | Effect::RemoveCrests { select, .. }
        | Effect::LeaderModifier { select, .. } => selector_unsupported(select),
        _ => None,
    }
}

fn source_unsupported(src: &CardSource) -> Option<String> {
    match src {
        CardSource::RandomFrom { .. }
        | CardSource::Named { .. }
        | CardSource::Copy { .. }
        | CardSource::From { .. } => None,
    }
}

fn selector_unsupported(_s: &Selector) -> Option<String> {
    None
}

fn condition_unsupported(_c: &Condition) -> Option<String> {
    None
}

/// M1 `Unsupported` variants reachable from the intended M1 pool (or from
/// cards on `main` that a deck might try to play). Listed in the PR body.
pub fn m1_unsupported_list() -> Vec<&'static str> {
    vec!["op:counter skyboundHand"]
}

pub fn named_in_source(src: &CardSource) -> Option<CardId> {
    match src {
        CardSource::Named { named } => Some(*named),
        _ => None,
    }
}

fn push_named(id: CardId, out: &mut Vec<CardId>) {
    if !out.contains(&id) {
        out.push(id);
    }
}

fn collect_from_source(src: &CardSource, out: &mut Vec<CardId>) {
    if let CardSource::Named { named } = src {
        push_named(*named, out);
    }
}

fn collect_from_effect(e: &Effect, out: &mut Vec<CardId>) {
    match e {
        Effect::Summon { card, .. }
        | Effect::AddToHand { card, .. }
        | Effect::AddToDeck { card, .. } => collect_from_source(card, out),
        Effect::Transform { into, .. } => collect_from_source(into, out),
        Effect::Seq { effects, .. }
        | Effect::Pay { effects, .. }
        | Effect::Repeat { effects, .. }
        | Effect::RandomSplit { effects, .. } => {
            for x in effects {
                collect_from_effect(x, out);
            }
        }
        Effect::If {
            then, else_effects, ..
        } => {
            for x in then {
                collect_from_effect(x, out);
            }
            if let Some(els) = else_effects {
                for x in els {
                    collect_from_effect(x, out);
                }
            }
        }
        Effect::Choose {
            options: Some(opts),
            ..
        } => {
            for o in opts {
                for x in &o.effects {
                    collect_from_effect(x, out);
                }
            }
        }
        Effect::Sequence { steps, .. } => {
            for s in steps {
                for x in &s.effects {
                    collect_from_effect(x, out);
                }
            }
        }
        Effect::GrantAbility { ability, .. } => collect_from_ability(ability, out),
        _ => {}
    }
}

fn collect_from_ability(a: &Ability, out: &mut Vec<CardId>) {
    for e in a.effects() {
        collect_from_effect(e, out);
    }
}

fn collect_from_mode(m: &Mode, out: &mut Vec<CardId>) {
    match m {
        Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => {
            for e in effects {
                collect_from_effect(e, out);
            }
        }
        Mode::Crystallize { abilities, .. } => {
            if let Some(abs) = abilities {
                for a in abs {
                    collect_from_ability(a, out);
                }
            }
        }
    }
}

/// Every `CardSource::Named` id printed on `card` (abilities, modes, fuse).
pub fn named_from_card(card: &Card) -> Vec<CardId> {
    let mut out = Vec::new();
    for a in card.abilities() {
        collect_from_ability(a, &mut out);
    }
    for m in card.modes() {
        collect_from_mode(m, &mut out);
    }
    if let Some(fuse) = card.fuse() {
        if let Some(recipes) = &fuse.recipes {
            for recipe in recipes {
                if let FuseResult::Transform { transform_into } = &recipe.result {
                    push_named(*transform_into, &mut out);
                }
                if let Some(req) = &recipe.requires {
                    for id in req {
                        push_named(*id, &mut out);
                    }
                }
            }
        }
    }
    out
}

/// Transitive closure of every card `root` can create via `named` references
/// and catalog `related_card_ids`, excluding `root` itself.
pub fn named_reachable(db: &CardDb, root: CardId) -> Vec<CardId> {
    let mut seen = BTreeSet::new();
    seen.insert(root);
    let mut out = Vec::new();
    let mut stack = named_from_card_id(db, root);
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        out.push(id);
        stack.extend(named_from_card_id(db, id));
    }
    out
}

fn named_from_card_id(db: &CardDb, id: CardId) -> Vec<CardId> {
    let mut out = Vec::new();
    if let Ok(card) = db.card(id) {
        out.extend(named_from_card(card));
    }
    if let Some(rec) = db.catalog.get(&id.as_str()) {
        for r in &rec.related_card_ids {
            if let Some(cid) = CardId::parse(r) {
                push_named(cid, &mut out);
            }
        }
    }
    out
}
