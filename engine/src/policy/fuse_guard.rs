//! No-op fuse detection for [`H0::fuseguard`].

use crate::action::Action;
use crate::card::{Ability, Amount, Card, CardId, Condition, Effect, PayResource};
use crate::db::CardDb;
use crate::ids::PlayerId;
use crate::state::{State, HAND_LIMIT};

/// A fuse into `host` is a no-op when every `fused` trigger only demands unaffordable
/// PP, the host has no latent `wasFused` value, and the hand is not full.
pub fn fuse_is_noop(db: &CardDb, host: CardId, pp: i32, hand_len: usize) -> bool {
    if hand_len >= HAND_LIMIT {
        return false;
    }
    let Ok(card) = db.card(host) else {
        return false;
    };
    if card_has_was_fused_condition(card) {
        return false;
    }
    let fused: Vec<&Ability> = card
        .abilities()
        .iter()
        .filter(|a| matches!(a, Ability::Fused { .. }))
        .collect();
    if fused.is_empty() {
        return false;
    }
    fused.iter().all(|ability| {
        ability
            .effects()
            .iter()
            .all(|effect| effect_is_unaffordable_pp_pay(effect, pp))
    })
}

/// Whether `action` is a bot no-op fuse in `state` for `me`.
pub fn fuse_action_is_noop(db: &CardDb, state: &State, me: PlayerId, action: &Action) -> bool {
    match action {
        Action::Fuse { host } => {
            let p = state.player(me);
            let card_id = p.hand.get(*host as usize).map(|c| c.card);
            card_id.is_some_and(|id| fuse_is_noop(db, id, p.usable_pp(), p.hand.len()))
        }
        _ => false,
    }
}

/// Drop no-op fuse indices from root candidates. If every candidate would be
/// removed, the original list is kept and `0` is returned.
pub fn filter_cand_fuseguard(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    fuseguard: bool,
    legal: &[Action],
    cand: Vec<usize>,
) -> (Vec<usize>, u32) {
    if !fuseguard {
        return (cand, 0);
    }
    let mut dropped = 0u32;
    let mut kept = Vec::with_capacity(cand.len());
    for &i in &cand {
        if fuse_action_is_noop(db, state, me, &legal[i]) {
            dropped += 1;
        } else {
            kept.push(i);
        }
    }
    if dropped > 0 && !kept.is_empty() {
        (kept, dropped)
    } else {
        (cand, 0)
    }
}

fn card_has_was_fused_condition(card: &Card) -> bool {
    for ability in card.abilities() {
        if condition_has_was_fused(ability.when_cond()) {
            return true;
        }
        if walk_effects_for_was_fused(ability.effects()) {
            return true;
        }
    }
    false
}

fn walk_effects_for_was_fused(effects: &[Effect]) -> bool {
    effects.iter().any(effect_has_was_fused)
}

fn effect_has_was_fused(effect: &Effect) -> bool {
    if condition_has_was_fused(effect.when_cond()) {
        return true;
    }
    match effect {
        Effect::Pay { effects, .. } => walk_effects_for_was_fused(effects),
        Effect::If {
            cond,
            then,
            else_effects,
            ..
        } => {
            condition_has_was_fused(Some(cond))
                || walk_effects_for_was_fused(then)
                || else_effects
                    .as_ref()
                    .is_some_and(|els| walk_effects_for_was_fused(els))
        }
        Effect::Seq { effects, .. } | Effect::Repeat { effects, .. } => {
            walk_effects_for_was_fused(effects)
        }
        Effect::Choose {
            options: Some(opts),
            ..
        } => opts.iter().any(|o| walk_effects_for_was_fused(&o.effects)),
        Effect::Sequence { steps, .. } => {
            steps.iter().any(|s| walk_effects_for_was_fused(&s.effects))
        }
        _ => false,
    }
}

fn condition_has_was_fused(cond: Option<&Condition>) -> bool {
    match cond {
        None => false,
        Some(c) => condition_has_was_fused_inner(c),
    }
}

fn condition_has_was_fused_inner(cond: &Condition) -> bool {
    match cond {
        Condition::WasFused { .. } => true,
        Condition::All { all } => all.iter().any(condition_has_was_fused_inner),
        Condition::Any { any } => any.iter().any(condition_has_was_fused_inner),
        Condition::Not { not } => condition_has_was_fused_inner(not),
        _ => false,
    }
}

fn effect_is_unaffordable_pp_pay(effect: &Effect, pp: i32) -> bool {
    match effect {
        Effect::Pay {
            resource: PayResource::Pp,
            amount,
            ..
        } => match amount_int(amount) {
            Some(amt) => amt > pp,
            None => false,
        },
        _ => false,
    }
}

fn amount_int(amount: &Amount) -> Option<i32> {
    match amount {
        Amount::Int(n) => Some(*n),
        _ => None,
    }
}
