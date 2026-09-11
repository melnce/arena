//! Read-only hand / board presentation helpers for the M4 client.
//! Not on the apply / legal_actions / snapshot / hash path.

use serde::Serialize;

use crate::action::Action;
use crate::apply::legal_actions;
use crate::card::{Ability, Amount, Condition, Effect, Mode, PayResource};
use crate::db::CardDb;
use crate::ids::PlayerId;
use crate::state::{CardInstance, PlayForm, State};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GateInfo {
    pub kind: String,
    pub need: i32,
    pub have: i32,
    pub met: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HandCardInfo {
    pub pos: u8,
    pub id: String,
    pub base_cost: i32,
    pub cost: Option<i32>,
    pub form: Option<String>,
    pub playable: bool,
    pub gates: Vec<GateInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BoardCardInfo {
    pub slot: u8,
    pub id: String,
    pub can_attack: bool,
    pub evolved: bool,
    pub super_evolved: bool,
    pub gates: Vec<GateInfo>,
}

/// Per-card presentation for `player`'s hand (draw order).
pub fn hand_info(db: &CardDb, state: &State, player: PlayerId) -> Vec<HandCardInfo> {
    let legal = legal_actions(db, state);
    let p = state.player(player);
    let pp = p.usable_pp();
    p.hand
        .iter()
        .enumerate()
        .map(|(i, inst)| {
            let playable = legal
                .iter()
                .any(|a| matches!(a, Action::Play { hand } if *hand == i as u8));
            let form = resolve_play_form(db, inst, pp);
            // Paid cost whenever a form is resolved (item 8 badge). `None` only
            // when no form is affordable — not merely when Play is illegal.
            let (cost, form_label) = match form {
                Some((paid, kind)) => (Some(paid), Some(form_label(kind).to_string())),
                None => (None, None),
            };
            HandCardInfo {
                pos: i as u8,
                id: inst.card.as_str(),
                base_cost: inst.base_cost,
                cost,
                form: form_label,
                playable,
                gates: collect_hand_gates(db, state, player, inst, pp),
            }
        })
        .collect()
}

/// Occupied field slots for `player` in engine slot order (compacted, 0 leftmost).
pub fn board_info(db: &CardDb, state: &State, player: PlayerId) -> Vec<BoardCardInfo> {
    let legal = legal_actions(db, state);
    let p = state.player(player);
    p.field
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| {
            let inst = slot.as_ref()?;
            let can_attack = legal.iter().any(|a| {
                matches!(
                    a,
                    Action::Attack { attacker, .. } if attacker.0 == i as u8
                )
            });
            Some(BoardCardInfo {
                slot: i as u8,
                id: inst.card.as_str(),
                can_attack,
                evolved: inst.evolved,
                super_evolved: inst.super_evolved,
                gates: collect_board_gates(db, state, player, inst),
            })
        })
        .collect()
}

fn form_label(form: PlayForm) -> &'static str {
    match form {
        PlayForm::Normal => "normal",
        PlayForm::Enhance { .. } => "enhance",
        PlayForm::Accelerate { .. } => "accelerate",
        PlayForm::Crystallize { .. } => "crystallize",
    }
}

/// Same selection rules as `apply::play_form` (cost / mode only; no target scan).
fn resolve_play_form(db: &CardDb, inst: &CardInstance, pp: i32) -> Option<(i32, PlayForm)> {
    let Ok(card) = db.card(inst.card) else {
        return None;
    };
    let effective = inst.cost;
    let enhance: Vec<&Mode> = if !inst.printed_tags.contains("enhance") {
        Vec::new()
    } else {
        card.modes()
            .iter()
            .filter(|m| matches!(m, Mode::Enhance { cost, .. } if pp >= *cost))
            .collect()
    };
    let accel = card.modes().iter().find_map(|m| match m {
        Mode::Accelerate { cost, .. } if pp < effective && pp >= *cost => Some(*cost),
        _ => None,
    });
    if !enhance.is_empty() && pp >= effective {
        let paid = enhance
            .iter()
            .map(|m| m.cost())
            .max()
            .unwrap_or(effective)
            .max(effective);
        if pp < paid {
            return None;
        }
        return Some((paid, PlayForm::Enhance { paid }));
    }
    if let Some(cost) = accel {
        return Some((cost, PlayForm::Accelerate { paid: cost }));
    }
    let crystal = card.modes().iter().find_map(|m| match m {
        Mode::Crystallize { cost, .. } if pp < effective && pp >= *cost => Some(*cost),
        _ => None,
    });
    if let Some(cost) = crystal {
        return Some((cost, PlayForm::Crystallize { paid: cost }));
    }
    if pp >= effective {
        Some((effective, PlayForm::Normal))
    } else {
        None
    }
}

fn collect_hand_gates(
    db: &CardDb,
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    pp: i32,
) -> Vec<GateInfo> {
    let Ok(card) = db.card(inst.card) else {
        return Vec::new();
    };
    let mut gates = Vec::new();
    for mode in card.modes() {
        match mode {
            Mode::Enhance { cost, effects, .. } => {
                push_gate(&mut gates, "enhance", *cost, pp, pp >= *cost);
                walk_effects(effects, state, player, inst, &mut gates);
            }
            Mode::Accelerate { cost, effects, .. } => {
                push_gate(
                    &mut gates,
                    "accelerate",
                    *cost,
                    pp,
                    pp >= *cost && pp < inst.cost,
                );
                walk_effects(effects, state, player, inst, &mut gates);
            }
            Mode::Crystallize {
                cost, abilities, ..
            } => {
                push_gate(
                    &mut gates,
                    "crystallize",
                    *cost,
                    pp,
                    pp >= *cost && pp < inst.cost,
                );
                if let Some(abs) = abilities {
                    for a in abs {
                        walk_ability(a, state, player, inst, &mut gates);
                    }
                }
            }
        }
    }
    for a in card.abilities() {
        walk_ability(a, state, player, inst, &mut gates);
    }
    for a in &inst.granted {
        walk_ability(a, state, player, inst, &mut gates);
    }
    if inst.printed_tags.contains("spellboost")
        || card
            .abilities()
            .iter()
            .any(|a| matches!(a, Ability::Spellboost { .. }))
    {
        push_gate(&mut gates, "spellboost", 0, inst.spellboost_count, false);
    }
    gates
}

fn collect_board_gates(
    db: &CardDb,
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
) -> Vec<GateInfo> {
    let Ok(card) = db.card(inst.card) else {
        return Vec::new();
    };
    let mut gates = Vec::new();
    for a in card.abilities() {
        walk_ability(a, state, player, inst, &mut gates);
    }
    for a in &inst.granted {
        walk_ability(a, state, player, inst, &mut gates);
    }
    gates.retain(|g| matches!(g.kind.as_str(), "rally" | "combo" | "overflow"));
    gates
}

fn walk_ability(
    ability: &Ability,
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    gates: &mut Vec<GateInfo>,
) {
    if let Some(c) = ability.when_cond() {
        walk_condition(c, state, player, gates);
    }
    walk_effects(ability.effects(), state, player, inst, gates);
}

fn walk_effects(
    effects: &[Effect],
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    gates: &mut Vec<GateInfo>,
) {
    for e in effects {
        walk_effect(e, state, player, inst, gates);
    }
}

fn walk_effect(
    effect: &Effect,
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    gates: &mut Vec<GateInfo>,
) {
    if let Some(c) = effect.when_cond() {
        walk_condition(c, state, player, gates);
    }
    match effect {
        Effect::Pay {
            resource,
            amount,
            effects,
            ..
        } => {
            if let Some(need) = amount_int(amount) {
                let (kind, have) = match resource {
                    PayResource::Shadows => ("necromancy", state.player(player).shadows),
                    PayResource::Earth => ("earth_rite", state.player(player).earth),
                    PayResource::Pp | PayResource::Faith => {
                        walk_effects(effects, state, player, inst, gates);
                        return;
                    }
                };
                push_gate(gates, kind, need, have, have >= need);
            }
            walk_effects(effects, state, player, inst, gates);
        }
        Effect::If {
            cond,
            then,
            else_effects,
            ..
        } => {
            walk_condition(cond, state, player, gates);
            walk_effects(then, state, player, inst, gates);
            if let Some(els) = else_effects {
                walk_effects(els, state, player, inst, gates);
            }
        }
        Effect::Seq { effects, .. } | Effect::Repeat { effects, .. } => {
            walk_effects(effects, state, player, inst, gates);
        }
        Effect::Choose {
            options: Some(opts),
            ..
        } => {
            for o in opts {
                walk_effects(&o.effects, state, player, inst, gates);
            }
        }
        Effect::Sequence { steps, .. } => {
            for s in steps {
                walk_effects(&s.effects, state, player, inst, gates);
            }
        }
        _ => {}
    }
}

fn walk_condition(cond: &Condition, state: &State, player: PlayerId, gates: &mut Vec<GateInfo>) {
    let p = state.player(player);
    match cond {
        Condition::Rally { rally } => {
            if let Some(need) = amount_int(&rally.n) {
                push_gate(gates, "rally", need, p.rally, p.rally >= need);
            }
        }
        Condition::Combo { combo } => {
            if let Some(need) = amount_int(&combo.n) {
                push_gate(gates, "combo", need, p.combo, p.combo >= need);
            }
        }
        Condition::Overflow { overflow } if *overflow => {
            push_gate(gates, "overflow", 7, p.pp_max, p.pp_max >= 7);
        }
        Condition::All { all } => {
            for c in all {
                walk_condition(c, state, player, gates);
            }
        }
        Condition::Any { any } => {
            for c in any {
                walk_condition(c, state, player, gates);
            }
        }
        Condition::Not { not } => walk_condition(not, state, player, gates),
        _ => {}
    }
}

fn amount_int(amount: &Amount) -> Option<i32> {
    match amount {
        Amount::Int(n) => Some(*n),
        _ => None,
    }
}

fn push_gate(gates: &mut Vec<GateInfo>, kind: &str, need: i32, have: i32, met: bool) {
    if gates.iter().any(|g| g.kind == kind && g.need == need) {
        return;
    }
    gates.push(GateInfo {
        kind: kind.to_string(),
        need,
        have,
        met,
    });
}
