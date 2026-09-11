//! Read-only hand / board presentation helpers for the M4 client.
//! Not on the apply / legal_actions / snapshot / hash path.

use serde::Serialize;

use crate::action::{acting_player, Action};
use crate::apply::{eval_cond, legal_actions, resolve_select};
use crate::card::{
    Ability, Amount, CardKind, Class, Condition, CounterKey, Effect, FieldHasKind, Filter,
    FilterKind, Mode, NamedCounter, PayResource, Selector, SelectorKind, Side, Tribe, TribeOrList,
    Zone,
};
use crate::db::CardDb;
use crate::ids::{AttackTarget, PlayerId};
use crate::state::{CardInstance, Phase, PlayForm, SourceRef, State, TargetOpt};
use crate::support;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GateInfo {
    pub kind: String,
    pub label: String,
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
    /// Why this card cannot be played right now; `None` when `playable`.
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BoardCardInfo {
    pub slot: u8,
    pub id: String,
    pub can_attack: bool,
    pub can_attack_leader: bool,
    pub rush_only: bool,
    pub evolved: bool,
    pub super_evolved: bool,
    pub gates: Vec<GateInfo>,
    /// Printed cannot-attack lock (not summoning sickness).
    pub cannot_attack_reason: Option<String>,
}

/// Per-player evolve / super-evolve unlock presentation (A9 / A10).
/// Thresholds match `can_evolve` and `Condition::SuperEvolutionUnlocked`:
/// second player 4 / 6, first player 5 / 7 (`turns_taken` vs those values).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PlayerInfo {
    pub evolve_unlocked: bool,
    pub super_evolve_unlocked: bool,
    /// Own turns still to start before the unlock; `0` once unlocked.
    pub evolve_unlock_in: u32,
    pub super_evolve_unlock_in: u32,
    /// Leader Barrier / damage cap is live.
    pub has_leader_barrier: bool,
}

/// Same turn-count gate `legal_actions` / `can_evolve` use. Not EP, not
/// `evolved_this_turn`, not whether a follower is currently eligible.
fn evolve_unlock_at(is_second: bool) -> u32 {
    if is_second {
        4
    } else {
        5
    }
}

fn super_evolve_unlock_at(is_second: bool) -> u32 {
    if is_second {
        6
    } else {
        7
    }
}

fn unlock_in(turns_taken: u32, at: u32) -> u32 {
    at.saturating_sub(turns_taken)
}

/// Unlock flags and remaining-turn countdowns for `player`.
pub fn player_info(_db: &CardDb, state: &State, player: PlayerId) -> PlayerInfo {
    let p = state.player(player);
    let evo_at = evolve_unlock_at(p.is_second);
    let super_at = super_evolve_unlock_at(p.is_second);
    let evolve_unlocked = p.turns_taken >= evo_at;
    let super_evolve_unlocked = p.turns_taken >= super_at;
    PlayerInfo {
        evolve_unlocked,
        super_evolve_unlocked,
        evolve_unlock_in: unlock_in(p.turns_taken, evo_at),
        super_evolve_unlock_in: unlock_in(p.turns_taken, super_at),
        has_leader_barrier: p.damage_cap().is_some(),
    }
}

/// Per-card presentation for `player`'s hand (draw order).
pub fn hand_info(db: &CardDb, state: &State, player: PlayerId) -> Vec<HandCardInfo> {
    let actor = acting_player(state);
    let legal = legal_actions(db, state);
    let p = state.player(player);
    let pp = p.usable_pp();
    p.hand
        .iter()
        .enumerate()
        .map(|(i, inst)| {
            let playable = player == actor
                && legal
                    .iter()
                    .any(|a| matches!(a, Action::Play { hand } if *hand == i as u8));
            let form = resolve_play_form(db, inst, pp);
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
                blocked_reason: if playable {
                    None
                } else {
                    Some(blocked_reason(
                        db,
                        state,
                        player,
                        actor,
                        inst,
                        form.as_ref(),
                    ))
                },
            }
        })
        .collect()
}

/// Occupied field slots for `player` in engine slot order (compacted, 0 leftmost).
pub fn board_info(db: &CardDb, state: &State, player: PlayerId) -> Vec<BoardCardInfo> {
    let actor = acting_player(state);
    let legal = legal_actions(db, state);
    let p = state.player(player);
    p.field
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| {
            let inst = slot.as_ref()?;
            let acting = player == actor;
            let can_attack = acting
                && legal.iter().any(|a| {
                    matches!(
                        a,
                        Action::Attack { attacker, .. } if attacker.0 == i as u8
                    )
                });
            let can_attack_leader = acting
                && legal.iter().any(|a| {
                    matches!(
                        a,
                        Action::Attack {
                            attacker,
                            target: AttackTarget::Leader,
                        } if attacker.0 == i as u8
                    )
                });
            let rush_only = inst.flags.summoning_sick && inst.is_rush() && !inst.is_storm();
            Some(BoardCardInfo {
                slot: i as u8,
                id: inst.card.as_str(),
                can_attack,
                can_attack_leader,
                rush_only,
                evolved: inst.evolved,
                super_evolved: inst.super_evolved,
                gates: collect_board_gates(db, state, player, inst),
                cannot_attack_reason: cannot_attack_reason(inst),
            })
        })
        .collect()
}

fn blocked_reason(
    db: &CardDb,
    state: &State,
    player: PlayerId,
    actor: PlayerId,
    inst: &CardInstance,
    form: Option<&(i32, PlayForm)>,
) -> String {
    if player != actor {
        return "Not your turn.".into();
    }
    if !matches!(state.phase, Phase::Main) {
        return "Wrong phase.".into();
    }
    if inst.cant_be_played() {
        return "Cannot be played.".into();
    }
    if let Ok(card) = db.card(inst.card) {
        if support::card_unsupported(card).is_some() {
            return "Cannot be played.".into();
        }
        if form.is_none() {
            return "Not enough PP.".into();
        }
        let as_spell = matches!(form, Some((_, PlayForm::Accelerate { .. })))
            || card.kind() == CardKind::Spell;
        if !as_spell && state.player(player).field_free() == 0 {
            return "Board is full.".into();
        }
    } else if form.is_none() {
        return "Not enough PP.".into();
    }
    "No legal target.".into()
}

fn cannot_attack_reason(inst: &CardInstance) -> Option<String> {
    let no_fol = inst.traits.cant_attack_followers == Some(true);
    let no_lead = inst.traits.cant_attack_leader == Some(true);
    if no_fol && no_lead {
        Some("Cannot attack.".into())
    } else if no_fol {
        Some("Cannot attack followers.".into())
    } else if no_lead {
        Some("Cannot attack the leader.".into())
    } else {
        None
    }
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
    _pp: i32,
) -> Vec<GateInfo> {
    let Ok(card) = db.card(inst.card) else {
        return Vec::new();
    };
    let mut gates = Vec::new();
    let source = SourceRef::Hand {
        player,
        id: inst.id,
    };
    for mode in card.modes() {
        match mode {
            Mode::Enhance { effects, .. } => {
                walk_effects(db, effects, state, player, inst, source, &mut gates);
            }
            Mode::Accelerate { effects, .. } => {
                walk_effects(db, effects, state, player, inst, source, &mut gates);
            }
            Mode::Crystallize { abilities, .. } => {
                if let Some(abs) = abilities {
                    for a in abs {
                        if matches!(a, Ability::Fanfare { .. }) {
                            walk_ability(db, a, state, player, inst, source, &mut gates);
                        }
                    }
                }
            }
        }
    }
    for a in card.abilities() {
        if matches!(a, Ability::Fanfare { .. }) {
            walk_ability(db, a, state, player, inst, source, &mut gates);
        }
    }
    if inst.printed_tags.contains("spellboost")
        || card
            .abilities()
            .iter()
            .any(|a| matches!(a, Ability::Spellboost { .. }))
    {
        push_gate(
            &mut gates,
            "spellboost",
            "spellboost",
            0,
            inst.spellboost_count,
            false,
        );
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
    let source = SourceRef::Field {
        player,
        id: inst.id,
    };
    for a in card.abilities() {
        walk_ability(db, a, state, player, inst, source, &mut gates);
    }
    for a in &inst.granted {
        walk_ability(db, a, state, player, inst, source, &mut gates);
    }
    gates.retain(|g| matches!(g.kind.as_str(), "rally" | "combo" | "overflow"));
    gates
}

fn walk_ability(
    db: &CardDb,
    ability: &Ability,
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    source: SourceRef,
    gates: &mut Vec<GateInfo>,
) {
    if let Some(c) = ability.when_cond() {
        walk_condition(db, c, state, player, inst, source, gates);
    }
    walk_effects(db, ability.effects(), state, player, inst, source, gates);
}

fn walk_effects(
    db: &CardDb,
    effects: &[Effect],
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    source: SourceRef,
    gates: &mut Vec<GateInfo>,
) {
    for e in effects {
        walk_effect(db, e, state, player, inst, source, gates);
    }
}

fn walk_effect(
    db: &CardDb,
    effect: &Effect,
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    source: SourceRef,
    gates: &mut Vec<GateInfo>,
) {
    if let Some(c) = effect.when_cond() {
        walk_condition(db, c, state, player, inst, source, gates);
    }
    match effect {
        Effect::Pay {
            resource,
            amount,
            effects,
            ..
        } => {
            if let Some(need) = amount_int(amount) {
                match resource {
                    PayResource::Shadows => {
                        let have = state.player(player).shadows;
                        push_gate(gates, "necromancy", "necromancy", need, have, have >= need);
                    }
                    PayResource::Earth => {
                        let have = state.player(player).earth;
                        push_gate(gates, "earth_rite", "earth rite", need, have, have >= need);
                    }
                    PayResource::Pp | PayResource::Faith => {}
                }
            }
            walk_effects(db, effects, state, player, inst, source, gates);
        }
        Effect::If {
            cond,
            then,
            else_effects,
            ..
        } => {
            walk_condition(db, cond, state, player, inst, source, gates);
            walk_effects(db, then, state, player, inst, source, gates);
            if let Some(els) = else_effects {
                walk_effects(db, els, state, player, inst, source, gates);
            }
        }
        Effect::Seq { effects, .. } | Effect::Repeat { effects, .. } => {
            walk_effects(db, effects, state, player, inst, source, gates);
        }
        Effect::Choose {
            options: Some(opts),
            ..
        } => {
            for o in opts {
                walk_effects(db, &o.effects, state, player, inst, source, gates);
            }
        }
        Effect::Sequence { steps, .. } => {
            for s in steps {
                walk_effects(db, &s.effects, state, player, inst, source, gates);
            }
        }
        _ => {}
    }
}

fn walk_condition(
    db: &CardDb,
    cond: &Condition,
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    source: SourceRef,
    gates: &mut Vec<GateInfo>,
) {
    match cond {
        Condition::Not { .. } => {}
        Condition::All { all } => {
            for c in all {
                walk_condition(db, c, state, player, inst, source, gates);
            }
        }
        Condition::Any { any } => {
            for c in any {
                walk_condition(db, c, state, player, inst, source, gates);
            }
        }
        Condition::Did { .. }
        | Condition::AttackingFollower { .. }
        | Condition::AttackingLeader { .. }
        | Condition::BoundHas { .. }
        | Condition::Evolved { .. }
        | Condition::CostEq { .. } => {}
        Condition::Rally { rally } => {
            if let Some(need) = amount_int(&rally.n) {
                let have = state.player(player).rally;
                push_gate(gates, "rally", "rally", need, have, have >= need);
            }
        }
        Condition::Combo { combo } => {
            if let Some(need) = amount_int(&combo.n) {
                let have = state.player(player).combo;
                push_gate(gates, "combo", "combo", need, have, have >= need);
            }
        }
        Condition::Overflow { overflow } if *overflow => {
            let have = state.player(player).pp_max;
            push_gate(gates, "overflow", "overflow", 7, have, have >= 7);
        }
        Condition::Overflow { .. } => {}
        other => push_evaluated(db, other, state, player, inst, source, gates),
    }
}

fn push_evaluated(
    db: &CardDb,
    cond: &Condition,
    state: &State,
    player: PlayerId,
    inst: &CardInstance,
    source: SourceRef,
    gates: &mut Vec<GateInfo>,
) {
    let met = eval_cond(db, state, player, Some(source), cond);
    let p = state.player(player);
    match cond {
        Condition::CountAtLeast { count_at_least } => {
            if matches!(count_at_least.select, Selector::Ref(_) | Selector::Bound(_)) {
                return;
            }
            let ts = resolve_select(db, state, player, source, &count_at_least.select);
            let have = match &count_at_least.filter {
                Some(f) => ts
                    .iter()
                    .filter(|t| target_matches_simple(db, state, t, f))
                    .count() as i32,
                None => ts.len() as i32,
            };
            let need = amount_int(&count_at_least.n).unwrap_or(0);
            let label =
                count_at_least_label(&count_at_least.select, count_at_least.filter.as_ref());
            push_gate(gates, "countAtLeast", &label, need, have, met);
        }
        Condition::FieldHas { field_has } => {
            let need = field_has.n.as_ref().and_then(amount_int).unwrap_or(1);
            let have = field_has_count(state, player, field_has);
            let label = field_has_label(field_has);
            push_gate(gates, "fieldHas", &label, need, have, met);
        }
        Condition::HandHas { hand_has } => {
            let need = amount_int(&hand_has.n).unwrap_or(0);
            let have = p
                .hand
                .iter()
                .filter(|c| inst_matches_simple(c, &hand_has.filter))
                .count() as i32;
            let label = filter_noun(Some(&hand_has.filter), "cards in your hand");
            push_gate(gates, "handHas", &label, need, have, met);
        }
        Condition::HandSameCostAtLeast {
            hand_same_cost_at_least,
        } => {
            let need = amount_int(&hand_same_cost_at_least.n).unwrap_or(0);
            let mut counts = std::collections::BTreeMap::<i32, i32>::new();
            for c in &p.hand {
                *counts.entry(c.cost).or_insert(0) += 1;
            }
            let have = counts.values().copied().max().unwrap_or(0);
            push_gate(
                gates,
                "handSameCostAtLeast",
                "cards of the same cost in hand",
                need,
                have,
                met,
            );
        }
        Condition::MaxPpAtLeast { max_pp_at_least } => {
            let need = amount_int(&max_pp_at_least.n).unwrap_or(0);
            push_gate(gates, "maxPpAtLeast", "max PP", need, p.pp_max, met);
        }
        Condition::LeaderDefenseLte { leader_defense_lte } => {
            let need = amount_int(leader_defense_lte).unwrap_or(0);
            push_gate(
                gates,
                "leaderDefenseLte",
                "leader defense",
                need,
                p.leader_defense,
                met,
            );
        }
        Condition::EvolvedCountAtLeast {
            evolved_count_at_least,
        } => {
            let need = amount_int(&evolved_count_at_least.n).unwrap_or(0);
            push_gate(
                gates,
                "evolvedCountAtLeast",
                "evolved followers",
                need,
                p.evolves_used,
                met,
            );
        }
        Condition::EnterCountAtLeast {
            enter_count_at_least,
        } => {
            let need = amount_int(&enter_count_at_least.n).unwrap_or(0);
            let side = match enter_count_at_least.side {
                Some(Side::Enemy) => player.opponent(),
                _ => player,
            };
            let mut have = state
                .player(side)
                .enter_counts
                .get(&enter_count_at_least.card)
                .copied()
                .unwrap_or(0);
            if enter_count_at_least.other == Some(true) {
                have = have.saturating_sub(1);
            }
            push_gate(
                gates,
                "enterCountAtLeast",
                "enters this match",
                need,
                have,
                met,
            );
        }
        Condition::CounterAtLeast { counter_at_least } => {
            let need = amount_int(&counter_at_least.n).unwrap_or(0);
            let (label, have) = counter_have(p, inst, &counter_at_least.key);
            push_gate(gates, "counterAtLeast", &label, need, have, met);
        }
        Condition::AmountAtLeast { amount_at_least } => {
            let need = amount_int(&amount_at_least.n).unwrap_or(0);
            let have = amount_int(&amount_at_least.of).unwrap_or(0);
            push_gate(gates, "amountAtLeast", "amount", need, have, met);
        }
        Condition::SkyboundArt { skybound_art } => {
            let need = amount_int(&skybound_art.n).unwrap_or(0);
            let have = state.turn as i32 + inst.skybound;
            push_gate(gates, "skyboundArt", "skybound art", need, have, met);
        }
        Condition::VarAtLeast { var_at_least } => {
            let need = amount_int(&var_at_least.n).unwrap_or(0);
            let have = inst.vars.get(&var_at_least.key).copied().unwrap_or(0);
            push_gate(gates, "varAtLeast", "variable", need, have, met);
        }
        Condition::SuperEvolutionUnlocked { .. } => {
            push_gate(
                gates,
                "superEvolutionUnlocked",
                "super-evolution unlocked",
                0,
                0,
                met,
            );
        }
        Condition::WasFused { .. } => {
            push_gate(gates, "wasFused", "fused", 0, 0, met);
        }
        Condition::DeckHasNoDuplicates { .. } => {
            push_gate(
                gates,
                "deckHasNoDuplicates",
                "no duplicate cards in deck",
                0,
                0,
                met,
            );
        }
        Condition::PlayedBaseCostsThisMatch { .. } => {
            push_gate(
                gates,
                "playedBaseCostsThisMatch",
                "played those costs this match",
                0,
                0,
                met,
            );
        }
        Condition::AttackedLeaderLastTurn { .. } => {
            push_gate(
                gates,
                "attackedLeaderLastTurn",
                "attacked the enemy leader last turn",
                0,
                0,
                met,
            );
        }
        Condition::TurnOwner { .. } => {
            push_gate(gates, "turnOwner", "your turn", 0, 0, met);
        }
        _ => {}
    }
}

fn counter_have(
    p: &crate::state::PlayerState,
    inst: &CardInstance,
    key: &CounterKey,
) -> (String, i32) {
    match key {
        CounterKey::Named(NamedCounter::Combo) => ("combo".into(), p.combo),
        CounterKey::Named(NamedCounter::Earth) => ("earth rite".into(), p.earth),
        CounterKey::Named(NamedCounter::Faith) => ("faith".into(), p.faith),
        CounterKey::Named(NamedCounter::Shadows) => ("shadows".into(), p.shadows),
        CounterKey::Named(NamedCounter::SkyboundHand) => ("skybound art".into(), inst.skybound),
        CounterKey::Var { var } => ("variable".into(), inst.vars.get(var).copied().unwrap_or(0)),
    }
}

fn field_has_count(state: &State, who: PlayerId, field_has: &crate::card::FieldHas) -> i32 {
    let sides: Vec<PlayerId> = match field_has.side {
        Some(Side::Enemy) => vec![who.opponent()],
        Some(Side::Any) => vec![who, who.opponent()],
        None | Some(Side::Ally) => vec![who],
    };
    let mut count = 0i32;
    for p in sides {
        for c in state.player(p).field.iter().flatten() {
            if let Some(k) = field_has.kind {
                let ok = match k {
                    FieldHasKind::Follower => c.kind == crate::card::CardKind::Follower,
                    FieldHasKind::Amulet => c.kind == crate::card::CardKind::Amulet,
                    FieldHasKind::Card | FieldHasKind::Character => true,
                };
                if !ok {
                    continue;
                }
            }
            if inst_matches_simple(c, &field_has.filter) {
                count += 1;
            }
        }
    }
    count
}

fn field_has_label(field_has: &crate::card::FieldHas) -> String {
    let side = match field_has.side {
        Some(Side::Enemy) => "enemy",
        Some(Side::Any) => "",
        None | Some(Side::Ally) => "allied",
    };
    let kind = match field_has.kind {
        Some(FieldHasKind::Follower) => "followers",
        Some(FieldHasKind::Amulet) => "amulets",
        Some(FieldHasKind::Card) | Some(FieldHasKind::Character) | None => "cards",
    };
    let noun = filter_noun(Some(&field_has.filter), kind);
    let mut parts = Vec::new();
    if !side.is_empty() {
        parts.push(side.to_string());
    }
    parts.push(noun);
    parts.push("on the field".into());
    parts.join(" ")
}

fn count_at_least_label(sel: &Selector, filter: Option<&Filter>) -> String {
    let Selector::Pool(p) = sel else {
        return "cards".into();
    };
    let side = match p.side {
        Side::Ally => "allied",
        Side::Enemy => "enemy",
        Side::Any => "",
    };
    let zone = match p.zone {
        Zone::Field => "on the field",
        Zone::Hand => {
            if p.side == Side::Ally {
                "in your hand"
            } else {
                "in hand"
            }
        }
        Zone::Deck => "in the deck",
        Zone::Cemetery => "in the cemetery",
        Zone::Leader => "leaders",
        Zone::Crests => "crests",
    };
    let kind = match p.kind {
        SelectorKind::Follower => "followers",
        SelectorKind::Amulet => "amulets",
        SelectorKind::Card => "cards",
        SelectorKind::Leader => "leaders",
        SelectorKind::Character => "characters",
        SelectorKind::Faith => "faith",
    };
    let filt = filter.or(p.filter.as_ref());
    let noun = filter_noun(filt, kind);
    if p.zone == Zone::Hand && p.side == Side::Ally && filt.is_none() {
        return "cards in your hand".into();
    }
    let mut parts = Vec::new();
    if !side.is_empty() && p.zone != Zone::Hand {
        parts.push(side.to_string());
    }
    parts.push(noun);
    if p.zone != Zone::Leader {
        parts.push(zone.into());
    }
    parts.join(" ")
}

fn filter_noun(filter: Option<&Filter>, fallback: &str) -> String {
    let Some(f) = filter else {
        return fallback.to_string();
    };
    if let Some(t) = &f.tribe {
        let name = match t {
            TribeOrList::One(tr) => tribe_name(*tr),
            TribeOrList::Many(ts) => {
                return if ts.len() == 1 {
                    format!("{} {}", tribe_name(ts[0]), fallback)
                } else {
                    fallback.to_string()
                };
            }
        };
        return format!("{name} {fallback}");
    }
    if let Some(cl) = f.class {
        return format!("{} {}", class_name(cl), fallback);
    }
    if let Some(id) = f.card {
        return format!("copies of {id}");
    }
    if let Some(k) = f.kind {
        return match k {
            FilterKind::Follower => "followers".into(),
            FilterKind::Spell => "spells".into(),
            FilterKind::Amulet => "amulets".into(),
            FilterKind::Card => fallback.into(),
        };
    }
    fallback.to_string()
}

fn tribe_name(t: Tribe) -> &'static str {
    match t {
        Tribe::Anathema => "anathema",
        Tribe::Artifact => "artifact",
        Tribe::Departed => "departed",
        Tribe::EarthSigil => "earth sigil",
        Tribe::Encroacher => "encroacher",
        Tribe::Golem => "golem",
        Tribe::Loot => "loot",
        Tribe::Marine => "marine",
        Tribe::Mysteria => "mysteria",
        Tribe::Officer => "officer",
        Tribe::Pixie => "pixie",
        Tribe::Puppetry => "puppetry",
    }
}

fn class_name(c: Class) -> &'static str {
    match c {
        Class::Neutral => "neutral",
        Class::Forestcraft => "forestcraft",
        Class::Swordcraft => "swordcraft",
        Class::Runecraft => "runecraft",
        Class::Dragoncraft => "dragoncraft",
        Class::Abysscraft => "abysscraft",
        Class::Havencraft => "havencraft",
        Class::Portalcraft => "portalcraft",
    }
}

fn target_matches_simple(db: &CardDb, state: &State, t: &TargetOpt, f: &Filter) -> bool {
    match t {
        TargetOpt::Slot { player, slot } => state
            .field_inst(*player, *slot)
            .map(|c| inst_matches_simple(c, f))
            .unwrap_or(false),
        TargetOpt::Hand { player, pos } => state
            .player(*player)
            .hand
            .get(*pos as usize)
            .map(|c| inst_matches_simple(c, f))
            .unwrap_or(false),
        TargetOpt::Deck { player, id } => state
            .player(*player)
            .deck
            .iter()
            .find(|c| c.id == *id)
            .map(|c| inst_matches_simple(c, f))
            .unwrap_or(false),
        TargetOpt::Card(id) => db
            .card(*id)
            .ok()
            .map(|c| inst_matches_simple(&CardInstance::from_card(c, 0), f))
            .unwrap_or(false),
        _ => false,
    }
}

fn inst_matches_simple(c: &CardInstance, f: &Filter) -> bool {
    if let Some(all) = &f.all {
        return all.iter().all(|x| inst_matches_simple(c, x));
    }
    if let Some(any) = &f.any {
        return any.iter().any(|x| inst_matches_simple(c, x));
    }
    if let Some(n) = &f.not {
        return !inst_matches_simple(c, n);
    }
    if let Some(t) = &f.tribe {
        let ok = match t {
            TribeOrList::One(tr) => c.tribes.contains(tr),
            TribeOrList::Many(ts) => ts.iter().any(|tr| c.tribes.contains(tr)),
        };
        if !ok {
            return false;
        }
    }
    if let Some(id) = f.card {
        if c.card != id {
            return false;
        }
    }
    if let Some(ids) = &f.cards {
        if !ids.contains(&c.card) {
            return false;
        }
    }
    if let Some(id) = f.not_card {
        if c.card == id {
            return false;
        }
    }
    if let Some(k) = f.kind {
        let ok = match k {
            FilterKind::Follower => c.kind == crate::card::CardKind::Follower,
            FilterKind::Spell => c.kind == crate::card::CardKind::Spell,
            FilterKind::Amulet => c.kind == crate::card::CardKind::Amulet,
            FilterKind::Card => true,
        };
        if !ok {
            return false;
        }
    }
    if let Some(cl) = f.class {
        if c.class != cl {
            return false;
        }
    }
    true
}

fn amount_int(amount: &Amount) -> Option<i32> {
    match amount {
        Amount::Int(n) => Some(*n),
        _ => None,
    }
}

fn push_gate(gates: &mut Vec<GateInfo>, kind: &str, label: &str, need: i32, have: i32, met: bool) {
    if gates
        .iter()
        .any(|g| g.kind == kind && g.label == label && g.need == need)
    {
        return;
    }
    gates.push(GateInfo {
        kind: kind.to_string(),
        label: label.to_string(),
        need,
        have,
        met,
    });
}
