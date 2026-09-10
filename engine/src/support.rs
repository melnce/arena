//! Honest-stub walk: a card whose data uses an unimplemented construct is
//! `Unsupported { card, construct }` at game construction (never a silent no-op).

use crate::card::{Ability, Card, CardId, CardSource, Condition, Effect, Mode, Selector};
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
        Effect::RandomSplit { .. } => Some("op:randomSplit".into()),
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
        | Effect::Repeat { effects, .. } => {
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
    vec!["op:randomSplit", "op:counter skyboundHand"]
}

#[allow(dead_code)]
pub fn named_in_source(src: &CardSource) -> Option<CardId> {
    match src {
        CardSource::Named { named } => Some(*named),
        _ => None,
    }
}
