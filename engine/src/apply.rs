//! `apply` / `legal_actions` / `new_game`. Every rule cites its source.

use crate::action::{from_neutral, Action};
use crate::card::{
    Ability, AbilityZone, Amount, Card, CardId, CardKind, CardSource, ChooseBy, Condition,
    Controller, CounterHow, CounterKey, CrestPlayer, Effect, EventName, Filter, FilterKind,
    FuseResult, Mode, NamedCounter, OrderBy, PayResource, PoolPick, PoolSelector, PpAction,
    RefPick, ReplicateKey, Selector, SelectorKind, Side, Traits, TriggerTag, Until, Whose, Zone,
};
use crate::db::CardDb;
use crate::error::{Illegal, LoadError, Unsupported};
use crate::event::{Event, EventTarget, ZoneLabel};
use crate::ids::{AttackTarget, First, PlayerId, Slot};
use crate::rng::GameRng;
use crate::state::{
    Aftermath, BoundRef, CardInstance, ChoiceNode, CrestInstance, DestroyedRecord, GameConfig,
    LeaderMod, PendingChoice, PendingKind, Phase, PlayForm, PlayerState, QueuedTrigger, SourceRef,
    State, TargetOpt, WorkFrame, CREST_CAP, DECK_SIZE, HAND_LIMIT, PP_CAP,
};
use crate::support;
use crate::trace::{NeutralAction, PickWhat};

// Hand overflow: owner-rulings.md — Hand overflow destroys without Last Words — 2026-08-10

// =========================================================================
// Construction
// =========================================================================

/// 20 defense, 40-card decks, coin, 4-card opening hand.
/// Rulebook: Match Flow.
pub fn new_game(db: &CardDb, cfg: GameConfig) -> Result<State, LoadError> {
    if cfg.deck_a.len() != DECK_SIZE || cfg.deck_b.len() != DECK_SIZE {
        // Still construct; soak / tests may use other sizes. Missing cards error.
    }
    for id in cfg.deck_a.iter().chain(cfg.deck_b.iter()) {
        db.require_supported(*id)?;
    }
    let mut state = State {
        players: [PlayerState::new(), PlayerState::new()],
        turn: 0,
        active: PlayerId::A,
        first: PlayerId::A,
        phase: Phase::Mulligan {
            player: PlayerId::A,
        },
        winner: None,
        rng: GameRng::live(cfg.seed),
        step_counter: 0,
        next_instance: 1,
        crest_order: 1,
        picks: Vec::new(),
        pending_work: Vec::new(),
        queue: Vec::new(),
        suppress_last_words: false,
        bindings: std::collections::BTreeMap::new(),
        pending_play_rally: None,
        event_subject: None,
    };
    fill_deck(db, &mut state, PlayerId::A, &cfg.deck_a)?;
    fill_deck(db, &mut state, PlayerId::B, &cfg.deck_b)?;
    let mut picks = Vec::new();
    let first = match cfg.first {
        First::A => PlayerId::A,
        First::B => PlayerId::B,
        First::Coin => state.rng.pick_player_coin(&mut picks).map_err(|o| {
            LoadError::Unsupported(Unsupported {
                card: "coin".into(),
                construct: o.to_string(),
            })
        })?,
    };
    state.picks = picks;
    state.first = first;
    state.active = first;
    let second = first.opponent();
    state.player_mut(second).is_second = true;
    state.player_mut(second).bonus_pp.early_charge = true;
    state.player_mut(second).bonus_pp.late_charge = true;
    if let Some(hands) = &cfg.opening_hands {
        deal_opening_hand(&mut state, PlayerId::A, &hands.a)?;
        deal_opening_hand(&mut state, PlayerId::B, &hands.b)?;
    } else {
        for p in PlayerId::ALL {
            for _ in 0..4 {
                let _ = draw_one(&mut state, p, None).map_err(|e| {
                    LoadError::Unsupported(Unsupported {
                        card: "draw".into(),
                        construct: e.to_string(),
                    })
                })?;
            }
        }
    }
    state.phase = Phase::Mulligan { player: first };
    Ok(state)
}

/// Build the pre-mulligan hand by removing listed ids from the deck multiset
/// in draw order — `docs/trace-format.md` `opening_hands`.
fn deal_opening_hand(state: &mut State, who: PlayerId, ids: &[CardId]) -> Result<(), LoadError> {
    for id in ids {
        let pos = state
            .player(who)
            .deck
            .iter()
            .position(|c| c.card == *id)
            .ok_or(LoadError::OpeningHandNotInDeck {
                player: who.as_str().into(),
                card: id.as_str(),
            })?;
        let inst = state.player_mut(who).deck.remove(pos);
        add_to_hand(state, who, inst);
    }
    Ok(())
}

fn fill_deck(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    ids: &[CardId],
) -> Result<(), LoadError> {
    for id in ids {
        let card = db.require_supported(*id)?;
        let inst = CardInstance::from_card(card, state.alloc_id());
        state.player_mut(who).deck.push(inst);
    }
    Ok(())
}

// =========================================================================
// Draw / overflow  (rulebook Zones; ruling 2026-08-10)
// =========================================================================

fn commit_play_rally(state: &mut State) {
    if let Some(who) = state.pending_play_rally.take() {
        state.player_mut(who).rally += 1;
    }
}

fn note_draw(db: &CardDb, state: &mut State, who: PlayerId, card: CardId) {
    let subject = state
        .player(who)
        .hand
        .iter()
        .find(|c| c.card == card)
        .cloned();
    raise_when(db, state, who, EventName::AllyDraw, subject.as_ref(), who);
}

fn draw_one(
    state: &mut State,
    who: PlayerId,
    filter: Option<&Filter>,
) -> Result<Option<CardId>, Illegal> {
    if state.winner.is_some() {
        return Ok(None);
    }
    let idxs: Vec<usize> = {
        let p = state.player(who);
        p.deck
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                filter
                    .map(|f| inst_matches_filter(state, who, c, f))
                    .unwrap_or(true)
            })
            .map(|(i, _)| i)
            .collect()
    };
    if idxs.is_empty() {
        if filter.is_none() {
            // deck-out only on an unfiltered turn/effect draw from empty deck
            if state.player(who).deck.is_empty() {
                let opp = who.opponent();
                state.winner = Some(opp);
                state.phase = Phase::Terminal;
            }
        }
        return Ok(None);
    }
    let keys: Vec<String> = idxs
        .iter()
        .map(|&i| state.player(who).deck[i].card.as_str())
        .collect();
    let mut emit = Vec::new();
    let pick = state
        .rng
        .pick_index(PickWhat::Draw, &keys, &mut emit)
        .map_err(Illegal::OraclePickNotLegal)?;
    state.picks.extend(emit);
    let deck_i = idxs[pick.min(idxs.len() - 1)];
    let mut inst = state.player_mut(who).deck.remove(deck_i);
    inst.flags.summoning_sick = false;
    let id = inst.card;
    add_to_hand(state, who, inst);
    Ok(Some(id))
}

/// Hand limit 9; overflow destroys without Last Words.
/// `HAND_OVERFLOW_RULING`
fn add_to_hand(state: &mut State, who: PlayerId, inst: CardInstance) {
    if state.player(who).hand.len() >= HAND_LIMIT {
        overflow_destroy(state, who, inst);
    } else {
        state.player_mut(who).hand.push(inst);
    }
}

fn overflow_destroy(state: &mut State, who: PlayerId, inst: CardInstance) {
    // no Last Words — ruling 2026-08-10
    state.player_mut(who).shadows += 1;
    state.player_mut(who).cemetery.push(inst);
}

// =========================================================================
// Legal actions
// =========================================================================

/// Deterministic, stable order, never mutates, empty iff terminal.
pub fn legal_actions(db: &CardDb, state: &State) -> Vec<Action> {
    if state.winner.is_some() || matches!(state.phase, Phase::Terminal) {
        return Vec::new();
    }
    match &state.phase {
        Phase::Mulligan { .. } => {
            let mut out = Vec::with_capacity(16);
            for mask in 0..16u8 {
                let mut swap = [false; 4];
                for (i, bit) in swap.iter_mut().enumerate() {
                    *bit = (mask & (1 << i)) != 0;
                }
                out.push(Action::MulliganConfirm { swap });
            }
            out
        }
        Phase::Choice { node, .. } => legal_choice(node),
        Phase::Main | Phase::Combat => legal_main(db, state),
        Phase::End | Phase::Terminal => Vec::new(),
    }
}

fn legal_choice(node: &ChoiceNode) -> Vec<Action> {
    match node {
        ChoiceNode::Targets { options, .. } | ChoiceNode::MultiPick { options, .. } => {
            let mut v: Vec<Action> = (0..options.len() as u8).map(Action::Choose).collect();
            if matches!(node, ChoiceNode::MultiPick { .. }) {
                v.push(Action::Confirm);
            }
            v
        }
        ChoiceNode::Modes { options, .. } => (0..options.len() as u8).map(Action::Choose).collect(),
        ChoiceNode::Cards { options, .. } => (0..options.len() as u8).map(Action::Choose).collect(),
        ChoiceNode::FusePartners {
            options, picked, ..
        } => {
            let mut v: Vec<Action> = (0..options.len() as u8).map(Action::Choose).collect();
            if !picked.is_empty() {
                v.push(Action::Confirm);
            }
            v
        }
    }
}

fn legal_main(db: &CardDb, state: &State) -> Vec<Action> {
    let me = state.active;
    let p = state.player(me);
    let mut out = Vec::new();
    for (i, inst) in p.hand.iter().enumerate() {
        if playable(db, state, me, i, inst) {
            out.push(Action::Play { hand: i as u8 });
        }
    }
    for (i, inst) in p.hand.iter().enumerate() {
        if can_fuse(db, state, me, i, inst) {
            out.push(Action::Fuse { host: i as u8 });
        }
    }
    if can_toggle_bonus_pp(p) {
        out.push(Action::BonusPp);
    }
    for (slot, maybe) in p.field.iter().enumerate() {
        let Some(inst) = maybe else { continue };
        if inst.kind == CardKind::Follower {
            for t in attack_targets(state, me, slot as u8, inst) {
                out.push(Action::Attack {
                    attacker: Slot(slot as u8),
                    target: t,
                });
            }
            if can_evolve(state, me, inst, false) {
                out.push(Action::Evolve {
                    slot: Slot(slot as u8),
                    super_evolve: false,
                });
            }
            if can_evolve(state, me, inst, true) {
                out.push(Action::Evolve {
                    slot: Slot(slot as u8),
                    super_evolve: true,
                });
            }
        }
        if can_engage(db, state, me, slot as u8, inst) {
            out.push(Action::Engage {
                slot: Slot(slot as u8),
            });
        }
    }
    out.push(Action::EndTurn);
    out
}

/// Second-player Bonus PP is a **toggle** (old engine `canToggleSecondPlayerBonusPp`).
/// Activate the current-tier charge; cancel while the bonus orb is unspent
/// (`pp_bonus > 0`), regardless of regular PP. Regular orbs are spent first;
/// the bonus orb last. Once the orb is spent, the toggle is not offered again
/// that turn. Rulebook "Bonus PP" — two charges (turns ≤ 5 / from turn 6); EOT commits.
fn can_toggle_bonus_pp(p: &PlayerState) -> bool {
    if !p.is_second || p.turns_taken == 0 {
        return false;
    }
    if p.bonus_pp.locked {
        return false;
    }
    if p.bonus_pp.active {
        return true;
    }
    has_bonus_charge(p)
}

fn has_bonus_charge(p: &PlayerState) -> bool {
    if p.turns_taken < 6 {
        p.bonus_pp.early_charge
    } else {
        p.bonus_pp.late_charge
    }
}

/// Evolve from player turn 5/4, super 7/6. Rulebook Evolution Point Rules.
/// One manual evolution per player per turn (EP or SEP locks both).
fn can_evolve(state: &State, me: PlayerId, inst: &CardInstance, supered: bool) -> bool {
    if inst.kind != CardKind::Follower || inst.evolved {
        return false;
    }
    let p = state.player(me);
    if p.evolved_this_turn {
        return false;
    }
    let t = p.turns_taken;
    if supered {
        let unlock = if p.is_second { 6 } else { 7 };
        t >= unlock && p.sep > 0
    } else {
        let unlock = if p.is_second { 4 } else { 5 };
        t >= unlock && p.ep > 0
    }
}

fn can_engage(db: &CardDb, _state: &State, _me: PlayerId, _slot: u8, inst: &CardInstance) -> bool {
    if inst.flags.engaged_this_turn {
        return false;
    }
    let Ok(card) = db.card(inst.card) else {
        return false;
    };
    card.abilities()
        .iter()
        .any(|a| matches!(a, Ability::Engage { .. }))
        && engage_cost(card)
            .map(|c| _state.player(_me).usable_pp() >= c)
            .unwrap_or(false)
}

fn engage_cost(card: &Card) -> Option<i32> {
    card.abilities().iter().find_map(|a| match a {
        Ability::Engage { cost, .. } => Some(*cost),
        _ => None,
    })
}

fn can_fuse(db: &CardDb, state: &State, me: PlayerId, host: usize, inst: &CardInstance) -> bool {
    if inst.flags.fused_this_turn {
        return false;
    }
    let Ok(card) = db.card(inst.card) else {
        return false;
    };
    let Some(fuse) = card.fuse() else {
        return false;
    };
    // β/γ/Ω cannot initiate — no recipes that transform *from* them as host
    // except α. Hosts without fuse data already returned.
    let partners = fuse_partner_indices(db, state, me, host, fuse);
    !partners.is_empty()
}

fn fuse_partner_indices(
    _db: &CardDb,
    state: &State,
    me: PlayerId,
    host: usize,
    fuse: &crate::card::Fuse,
) -> Vec<u8> {
    let hand = &state.player(me).hand;
    let host_inst = &hand[host];
    hand.iter()
        .enumerate()
        .filter(|(i, c)| {
            *i != host
                && filter_matches_card(state, me, c, &fuse.partners)
                && !already_fused_kind(host_inst, c)
        })
        .map(|(i, _)| i as u8)
        .collect()
}

fn already_fused_kind(host: &CardInstance, partner: &CardInstance) -> bool {
    host.flags.fused_kinds.contains(&partner.card)
}

fn attack_targets(
    state: &State,
    me: PlayerId,
    _slot: u8,
    inst: &CardInstance,
) -> Vec<AttackTarget> {
    if inst.kind != CardKind::Follower || inst.flags.attacks_left <= 0 || inst.defense <= 0 {
        return Vec::new();
    }
    let sick = inst.flags.summoning_sick && !inst.is_storm() && !inst.is_rush() && !inst.evolved;
    if sick {
        return Vec::new();
    }
    let can_leader = !inst.flags.summoning_sick || inst.is_storm();
    // evolved this turn: follower attacks only (like Rush), not leader unless Storm
    let can_leader = can_leader && (inst.is_storm() || !inst.flags.summoning_sick);
    let _ = can_leader;
    let can_leader = inst.is_storm() || !inst.flags.summoning_sick;
    if inst.traits.cant_attack_leader == Some(true)
        && inst.traits.cant_attack_followers == Some(true)
    {
        return Vec::new();
    }
    let opp = me.opponent();
    let enemy = state.player(opp);
    let mut wards = Vec::new();
    let mut others = Vec::new();
    for (i, s) in enemy.field.iter().enumerate() {
        let Some(f) = s else { continue };
        if f.kind != CardKind::Follower || f.defense <= 0 {
            continue;
        }
        if f.ambush_blocks() || f.is_intimidate() {
            continue;
        }
        if inst.traits.cant_attack_followers == Some(true) {
            continue;
        }
        if f.is_ward() && !inst.ignores_ward() {
            wards.push(AttackTarget::Slot(Slot(i as u8)));
        } else {
            others.push(AttackTarget::Slot(Slot(i as u8)));
        }
    }
    let ward_gate = !wards.is_empty() && !inst.ignores_ward();
    let mut out = if ward_gate { wards } else { others };
    if can_leader && inst.traits.cant_attack_leader != Some(true) && !ward_gate {
        out.push(AttackTarget::Leader);
    }
    out
}

fn playable(db: &CardDb, state: &State, me: PlayerId, hand_i: usize, inst: &CardInstance) -> bool {
    if inst.cant_be_played() {
        return false;
    }
    let Ok(card) = db.card(inst.card) else {
        return false;
    };
    if support::card_unsupported(card).is_some() {
        return false;
    }
    let pp = state.player(me).usable_pp();
    let form = play_form(card, inst, pp, state.player(me).field_free());
    let Some((paid, kind, effects)) = form else {
        return false;
    };
    let _ = paid;
    if kind == CardKind::Follower || kind == CardKind::Amulet {
        if state.player(me).field_free() == 0 && !is_earth_merge(db, state, me, card) {
            return false;
        }
        return true;
    }
    // spell / accelerate: mandatory Select must have targets
    // owner-rulings 2026-08-16 / official Q&A 2026-09-06
    spell_playable(db, state, me, hand_i, card, &effects)
}

fn is_earth_merge(db: &CardDb, state: &State, me: PlayerId, card: &Card) -> bool {
    card.tribes().contains(&crate::card::Tribe::EarthSigil)
        && state.player(me).earth_slot.is_some()
        && db.card(card.id()).is_ok()
}

fn play_form(
    card: &Card,
    inst: &CardInstance,
    pp: i32,
    _free: usize,
) -> Option<(i32, CardKind, Vec<Effect>)> {
    let effective = inst.cost;
    let enhance: Vec<&Mode> = card
        .modes()
        .iter()
        .filter(|m| matches!(m, Mode::Enhance { cost, .. } if pp >= *cost))
        .collect();
    let accel = card.modes().iter().find_map(|m| match m {
        Mode::Accelerate { cost, effects, .. } if pp < effective && pp >= *cost => {
            Some((*cost, effects.clone()))
        }
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
        let mut fx = fanfare_effects(card);
        for m in enhance {
            if let Mode::Enhance {
                effects,
                replaces_base,
                ..
            } = m
            {
                if *replaces_base == Some(true) {
                    fx = effects.clone();
                } else {
                    fx.extend(effects.iter().cloned());
                }
            }
        }
        return Some((paid, card.kind(), fx));
    }
    if let Some((cost, fx)) = accel {
        return Some((cost, CardKind::Spell, fx));
    }
    if pp >= effective {
        Some((effective, card.kind(), fanfare_effects(card)))
    } else {
        None
    }
}

fn fanfare_effects(card: &Card) -> Vec<Effect> {
    card.abilities()
        .iter()
        .filter(|a| matches!(a, Ability::Fanfare { .. }))
        .flat_map(|a| a.effects().iter().cloned())
        .collect()
}

fn spell_playable(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    hand_i: usize,
    card: &Card,
    effects: &[Effect],
) -> bool {
    reachable_chooses_ok(db, state, me, hand_i, card, effects)
}

fn reachable_chooses_ok(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    hand_i: usize,
    card: &Card,
    effects: &[Effect],
) -> bool {
    for e in effects {
        if !choose_ok_one(db, state, me, hand_i, card, e) {
            return false;
        }
    }
    true
}

fn choose_ok_one(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    hand_i: usize,
    card: &Card,
    e: &Effect,
) -> bool {
    if let Some(cond) = e.when_cond() {
        if !eval_cond(db, state, me, None, cond) {
            return true;
        }
    }
    match e {
        Effect::If {
            cond,
            then,
            else_effects,
            ..
        } => {
            if eval_cond(db, state, me, None, cond) {
                reachable_chooses_ok(db, state, me, hand_i, card, then)
            } else if let Some(els) = else_effects {
                reachable_chooses_ok(db, state, me, hand_i, card, els)
            } else {
                true
            }
        }
        Effect::Seq { effects, .. }
        | Effect::Repeat { effects, .. }
        | Effect::Pay { effects, .. } => reachable_chooses_ok(db, state, me, hand_i, card, effects),
        Effect::Choose {
            by: ChooseBy::Player,
            ..
        } => true,
        Effect::Damage { select, .. }
        | Effect::Restore { select, .. }
        | Effect::Buff { select, .. }
        | Effect::Destroy { select, .. }
        | Effect::Banish { select, .. }
        | Effect::ReturnToHand { select, .. }
        | Effect::ReturnToDeck { select, .. }
        | Effect::Discard { select, .. }
        | Effect::Evolve { select, .. }
        | Effect::GrantTraits { select, .. }
        | Effect::RemoveTraits { select, .. }
        | Effect::Countdown { select, .. } => choose_select_ok(db, state, me, hand_i, select),
        _ => true,
    }
}

fn choose_select_ok(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    hand_i: usize,
    select: &Selector,
) -> bool {
    match select {
        Selector::Pool(p) if p.pick == PoolPick::Choose => {
            let n = p
                .count
                .as_ref()
                .map(|a| eval_amount(db, state, me, None, a))
                .unwrap_or(1);
            let cands = pool_candidates(db, state, me, None, p, Some(hand_i));
            cands.len() as i32 >= n.max(1)
        }
        _ => true,
    }
}

// =========================================================================
// apply
// =========================================================================

pub fn apply(db: &CardDb, state: &mut State, action: Action) -> Result<Vec<Event>, Illegal> {
    if state.winner.is_some() {
        return Err(Illegal::Terminal);
    }
    let legal = legal_actions(db, state);
    if !legal.contains(&action) {
        // Completed fuse with partners is applied as Fuse then auto-confirm in replay.
        if !matches!(action, Action::Fuse { .. }) {
            return Err(Illegal::NotLegal);
        }
    }
    state.picks.clear();
    state.bindings.clear();
    let mut events = Vec::new();
    match action {
        Action::MulliganConfirm { swap } => apply_mulligan(db, state, swap, &mut events)?,
        Action::Play { hand } => apply_play(db, state, hand, &mut events)?,
        Action::Attack { attacker, target } => {
            apply_attack(db, state, attacker, target, &mut events)?
        }
        Action::Evolve { slot, super_evolve } => {
            apply_evolve_action(db, state, slot, super_evolve, false, &mut events)?;
        }
        Action::Engage { slot } => apply_engage(db, state, slot, &mut events)?,
        Action::Fuse { host } => apply_fuse_start(db, state, host, &mut events)?,
        Action::BonusPp => apply_bonus(state, &mut events)?,
        Action::Choose(i) => apply_choose(db, state, i, &mut events)?,
        Action::Confirm => apply_confirm(db, state, &mut events)?,
        Action::EndTurn => apply_end_turn(db, state, &mut events)?,
    }
    drain_until_quiet(db, state, &mut events)?;
    Ok(events)
}

/// Apply a recorded `NeutralAction`, including an old-engine completed fuse
/// (`partner_pos` are pre-action hand positions, mapped onto `options`).
pub fn apply_neutral(
    db: &CardDb,
    state: &mut State,
    action: &NeutralAction,
) -> Result<Vec<Event>, Illegal> {
    if let NeutralAction::Fuse {
        host_pos,
        partner_pos,
        ..
    } = action
    {
        if !partner_pos.is_empty() {
            let mut events = apply(db, state, Action::Fuse { host: *host_pos })?;
            let options = match &state.phase {
                Phase::Choice {
                    node: ChoiceNode::FusePartners { options, .. },
                    ..
                } => options.clone(),
                _ => return Err(Illegal::NotLegal),
            };
            for p in partner_pos {
                let idx = options
                    .iter()
                    .position(|&x| x == *p)
                    .ok_or(Illegal::NotLegal)?;
                events.extend(apply(db, state, Action::Choose(idx as u8))?);
            }
            events.extend(apply(db, state, Action::Confirm)?);
            return Ok(events);
        }
    }
    let act = from_neutral(state, action).ok_or(Illegal::NotLegal)?;
    apply(db, state, act)
}

fn apply_mulligan(
    db: &CardDb,
    state: &mut State,
    swap: [bool; 4],
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let Phase::Mulligan { player } = state.phase else {
        return Err(Illegal::NotLegal);
    };
    let n = state.player(player).hand.len().min(4);
    let mut swapped = Vec::new();
    let mut returning = Vec::new();
    for (i, inst) in state.player(player).hand.iter().take(n).enumerate() {
        if swap[i] {
            swapped.push(inst.card);
        }
    }
    // replacements drawn BEFORE swapped cards return — Practice-Tool PR #253
    let mut keep = Vec::new();
    let hand = std::mem::take(&mut state.player_mut(player).hand);
    for (i, inst) in hand.into_iter().enumerate() {
        if i < 4 && swap[i] {
            returning.push(inst);
        } else {
            keep.push(inst);
        }
    }
    state.player_mut(player).hand = keep;
    for _ in 0..returning.len() {
        let _ = draw_one(state, player, None)?;
    }
    for inst in returning {
        state.player_mut(player).deck.push(inst);
    }
    events.push(Event::Mulligan { player, swapped });
    let next = if player == state.first {
        player.opponent()
    } else {
        // both done — start first turn
        start_turn_full(db, state, state.first, events)?;
        return Ok(());
    };
    state.phase = Phase::Mulligan { player: next };
    Ok(())
}

fn apply_bonus(state: &mut State, _events: &mut [Event]) -> Result<(), Illegal> {
    let me = state.active;
    if !can_toggle_bonus_pp(state.player(me)) {
        return Err(Illegal::NotLegal);
    }
    let p = state.player_mut(me);
    if p.bonus_pp.active {
        // cancel while unspent — charge stays available
        p.bonus_pp.active = false;
    } else {
        p.bonus_pp.active = true;
    }
    Ok(())
}

/// End of turn commits an activated (or spent) Bonus PP charge.
/// Rulebook Bonus PP; old engine commits on turn end, not on the click.
fn commit_bonus_pp(p: &mut PlayerState) {
    if p.bonus_pp.active || p.bonus_pp.locked {
        if p.turns_taken < 6 {
            p.bonus_pp.early_charge = false;
        } else {
            p.bonus_pp.late_charge = false;
        }
    }
    p.bonus_pp.active = false;
    p.bonus_pp.locked = false;
}

fn apply_play(
    db: &CardDb,
    state: &mut State,
    hand: u8,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let me = state.active;
    let inst = state
        .player(me)
        .hand
        .get(hand as usize)
        .cloned()
        .ok_or(Illegal::NotLegal)?;
    let card = db.card(inst.card).map_err(|_| Illegal::NotLegal)?;
    let pp = state.player(me).usable_pp();
    let (paid, kind, effects) =
        play_form(card, &inst, pp, state.player(me).field_free()).ok_or(Illegal::NotLegal)?;
    state.player_mut(me).spend_pp(paid);
    let inst = state.player_mut(me).hand.remove(hand as usize);
    let form = if kind == CardKind::Spell && card.kind() != CardKind::Spell {
        PlayForm::Accelerate { paid }
    } else if paid > inst.cost
        || card
            .modes()
            .iter()
            .any(|m| matches!(m, Mode::Enhance { cost, .. } if paid >= *cost && pp >= *cost))
    {
        PlayForm::Enhance { paid }
    } else {
        PlayForm::Normal
    };
    // Combo counts the played card — official Q&A May
    state.player_mut(me).combo += 1;
    state.player_mut(me).played_this_turn.push(inst.card);
    let base_for_ladder = if matches!(form, PlayForm::Accelerate { .. }) {
        paid
    } else {
        inst.base_cost
    };
    state
        .player_mut(me)
        .played_base_costs_this_match
        .push(base_for_ladder);
    events.push(Event::Play {
        player: me,
        card: inst.card,
        form,
    });
    raise_when(db, state, me, EventName::AllyCardPlayed, Some(&inst), me);
    if kind == CardKind::Spell {
        raise_when(db, state, me, EventName::AllySpellPlayed, Some(&inst), me);
        let src = SourceRef::Spell {
            player: me,
            card: inst.card,
        };
        // Spellboost the hand (Accelerate counts as a spell) — ruling 2026-09-02
        spellboost_hand(db, state, me, events)?;
        push_effects(state, me, src, effects);
        // corpse
        let mut corpse = inst;
        corpse.kind = CardKind::Spell;
        corpse.base_cost = base_for_ladder;
        corpse.cost = paid;
        state.player_mut(me).shadows += 1;
        state.player_mut(me).cemetery.push(corpse);
    } else {
        enter_from_play(db, state, me, inst, effects, events)?;
    }
    Ok(())
}

fn enter_from_play(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    mut inst: CardInstance,
    fanfare: Vec<Effect>,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    inst.flags.summoning_sick = true;
    inst.flags.attacks_left = inst.traits.attacks_per_turn.unwrap_or(1);
    if inst.is_earth_sigil() && merge_earth(db, state, me, &inst, events)? {
        let src = SourceRef::Field {
            player: me,
            id: inst.id,
        };
        push_effects(state, me, src, fanfare);
        return Ok(());
    }
    let Some(slot) = state.player(me).first_empty_slot() else {
        // excess summons fail, do not count as entries — Rally ruling 2026-08-12
        return Ok(());
    };
    let id = inst.id;
    let card_id = inst.card;
    let kind_entered = inst.kind;
    if inst.kind == CardKind::Follower {
        // Rally for a *played* follower increments when the play sequence
        // completes (after Fanfare). Rulebook Rally.
        state.pending_play_rally = Some(me);
        *state
            .player_mut(me)
            .enter_counts
            .entry(card_id)
            .or_insert(0) += 1;
    }
    state.player_mut(me).field[slot as usize] = Some(inst);
    if state.player(me).field[slot as usize]
        .as_ref()
        .is_some_and(|c| c.is_earth_sigil())
    {
        state.player_mut(me).earth_slot = Some(slot);
        if state.player(me).earth == 0 {
            state.player_mut(me).earth = 1;
        }
    }
    events.push(Event::Summon {
        player: me,
        card: card_id,
        slot: Slot(slot),
    });
    events.push(Event::Enter { slot: Slot(slot) });
    let src = SourceRef::Field { player: me, id };
    if kind_entered == CardKind::Follower {
        if let Some(entered) = state.field_inst(me, slot).cloned() {
            raise_follower_enter(db, state, me, &entered);
        }
    }
    // play sequence: enter reactions → Fanfare → crest → board
    // rulebook Fanfare and Enter-Play Trigger Order
    queue_enter_reactions(db, state, me, card_id, id, true);
    let gated = gated_fanfare(db, state, me, src, card_id, fanfare);
    push_effects(state, me, src, gated);
    Ok(())
}

/// Honour Fanfare `when` (e.g. Rally) at the moment of entry; keep Enhance extras.
fn gated_fanfare(
    db: &CardDb,
    state: &State,
    me: PlayerId,
    src: SourceRef,
    card_id: CardId,
    fanfare: Vec<Effect>,
) -> Vec<Effect> {
    let Ok(card) = db.card(card_id) else {
        return fanfare;
    };
    let printed = fanfare_effects(card);
    if fanfare == printed {
        card.abilities()
            .iter()
            .filter(|a| matches!(a, Ability::Fanfare { .. }))
            .filter(|a| {
                a.when_cond()
                    .map(|c| eval_cond(db, state, me, Some(src), c))
                    .unwrap_or(true)
            })
            .flat_map(|a| a.effects().iter().cloned())
            .collect()
    } else {
        fanfare
    }
}

/// Witch's New Brew always wins an Earth Sigil merge — 2026-08-30
fn merge_earth(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    incoming: &CardInstance,
    events: &mut Vec<Event>,
) -> Result<bool, Illegal> {
    let Some(slot) = state.player(me).earth_slot else {
        return Ok(false);
    };
    let Some(holder) = state.player(me).field[slot as usize].clone() else {
        return Ok(false);
    };
    let inc_token = db.card(incoming.card).map(|c| c.token()).unwrap_or(true);
    let hold_token = db.card(holder.card).map(|c| c.token()).unwrap_or(true);
    state.player_mut(me).earth += 1;
    if !inc_token && hold_token {
        // incoming collectible replaces token, keeps stack
        destroy_slot(db, state, me, slot, true, events)?;
        let Some(empty) = state.player(me).first_empty_slot() else {
            return Ok(true);
        };
        let mut body = incoming.clone();
        body.flags.summoning_sick = true;
        state.player_mut(me).field[empty as usize] = Some(body);
        state.player_mut(me).earth_slot = Some(empty);
        return Ok(true);
    }
    // token into collectible, or same-class merge: increment only
    Ok(true)
}

fn spellboost_hand(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    _events: &mut [Event],
) -> Result<(), Illegal> {
    let ids: Vec<u32> = state.player(me).hand.iter().map(|c| c.id).collect();
    for hid in ids {
        let Some(pos) = state.player(me).hand.iter().position(|c| c.id == hid) else {
            continue;
        };
        let card_id = state.player(me).hand[pos].card;
        let Ok(card) = db.card(card_id) else { continue };
        let fx: Vec<Effect> = card
            .abilities()
            .iter()
            .filter(|a| matches!(a, Ability::Spellboost { .. }))
            .flat_map(|a| a.effects().iter().cloned())
            .collect();
        if fx.is_empty() {
            continue;
        }
        let src = SourceRef::Hand {
            player: me,
            id: hid,
        };
        push_effects(state, me, src, fx);
    }
    Ok(())
}

fn apply_attack(
    db: &CardDb,
    state: &mut State,
    attacker: Slot,
    target: AttackTarget,
    _events: &mut [Event],
) -> Result<(), Illegal> {
    let me = state.active;
    let Some(att) = state.field_inst(me, attacker.0).cloned() else {
        return Err(Illegal::NotLegal);
    };
    if let Some(f) = state.field_inst_mut(me, attacker.0) {
        f.flags.attacks_left -= 1;
        f.flags.attacked_this_turn = true;
        f.flags.ambush_active = false;
        if f.traits.ambush == Some(true) {
            f.traits.ambush = None;
        }
    }
    if matches!(target, AttackTarget::Leader) {
        state.player_mut(me).attacked_leader_this_turn = true;
    }
    raise_when(
        db,
        state,
        me,
        EventName::AllyFollowerAttacks,
        Some(&att),
        me,
    );
    raise_when(
        db,
        state,
        me.opponent(),
        EventName::EnemyFollowerAttacks,
        Some(&att),
        me,
    );
    // Strike / Clash before damage — rulebook Combat Timing
    queue_combat_triggers(db, state, me, attacker.0, target);
    state
        .pending_work
        .push(WorkFrame::Aftermath(Aftermath::AfterCombat {
            attacker_player: me,
            attacker_id: att.id,
            target,
            knockback: att.super_evolved,
        }));
    Ok(())
}

fn queue_combat_triggers(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    slot: u8,
    target: AttackTarget,
) {
    let Some(att) = state.field_inst(me, slot).cloned() else {
        return;
    };
    let vs_follower = matches!(target, AttackTarget::Slot(_));
    enqueue_card_triggers(db, state, me, att.id, TriggerTag::Strike, 4);
    if vs_follower {
        enqueue_card_triggers(db, state, me, att.id, TriggerTag::FollowerStrike, 4);
        enqueue_card_triggers(db, state, me, att.id, TriggerTag::Clash, 4);
        if let AttackTarget::Slot(ds) = target {
            if let Some(def) = state.field_inst(me.opponent(), ds.0).cloned() {
                enqueue_card_triggers(db, state, me.opponent(), def.id, TriggerTag::Clash, 6);
            }
        }
    }
}

fn apply_evolve_action(
    db: &CardDb,
    state: &mut State,
    slot: Slot,
    supered: bool,
    granted: bool,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let me = state.active;
    let Some(inst) = state.field_inst_mut(me, slot.0) else {
        return Err(Illegal::NotLegal);
    };
    if inst.evolved && !granted {
        return Err(Illegal::NotLegal);
    }
    if !granted {
        if supered {
            state.player_mut(me).sep -= 1;
        } else {
            state.player_mut(me).ep -= 1;
        }
        state.player_mut(me).evolved_this_turn = true;
    }
    // EP, SEP or effect — `docs/trace-format.md` evolves_used.
    state.player_mut(me).evolves_used += 1;
    let Some(inst) = state.field_inst_mut(me, slot.0) else {
        return Err(Illegal::NotLegal);
    };
    inst.evolved = true;
    inst.super_evolved = inst.super_evolved || supered;
    let (da, dd) = if supered { (3, 3) } else { (2, 2) };
    inst.attack += da;
    inst.defense += dd;
    inst.max_defense += dd;
    // evolving lifts summoning sickness for follower attacks
    inst.flags.summoning_sick = inst.flags.summoning_sick && !inst.is_storm();
    events.push(Event::Evolve {
        slot,
        super_evolve: supered,
        granted,
    });
    let id = inst.id;
    let card_id = inst.card;
    let Ok(card) = db.card(card_id) else {
        return Ok(());
    };
    let replace = card.abilities().iter().any(|a| a.replaces_evolve());
    let mut fx = Vec::new();
    if supered && replace {
        for a in card.abilities() {
            if matches!(a, Ability::SuperEvolve { .. }) {
                fx.extend(a.effects().iter().cloned());
            }
        }
    } else {
        if !granted {
            for a in card.abilities() {
                if matches!(a, Ability::Evolve { .. }) {
                    fx.extend(a.effects().iter().cloned());
                }
            }
        }
        for a in card.abilities() {
            if matches!(a, Ability::AnyEvolve { .. }) {
                fx.extend(a.effects().iter().cloned());
            }
        }
        if supered {
            for a in card.abilities() {
                if matches!(
                    a,
                    Ability::SuperEvolve { .. } | Ability::AnySuperEvolve { .. }
                ) {
                    fx.extend(a.effects().iter().cloned());
                }
            }
        }
    }
    let src = SourceRef::Field { player: me, id };
    if let Some(evolved) = state.field_inst(me, slot.0).cloned() {
        raise_when(db, state, me, EventName::AllyEvolve, Some(&evolved), me);
        if supered {
            raise_when(
                db,
                state,
                me,
                EventName::AllySuperEvolve,
                Some(&evolved),
                me,
            );
        }
    }
    push_effects(state, me, src, fx);
    Ok(())
}

fn apply_engage(
    db: &CardDb,
    state: &mut State,
    slot: Slot,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let me = state.active;
    let Some(inst) = state.field_inst(me, slot.0).cloned() else {
        return Err(Illegal::NotLegal);
    };
    let card = db.card(inst.card).map_err(|_| Illegal::NotLegal)?;
    let (cost, sacrifice, fx) = card
        .abilities()
        .iter()
        .find_map(|a| match a {
            Ability::Engage {
                cost,
                sacrifice,
                effects,
                ..
            } => Some((*cost, *sacrifice, effects.clone())),
            _ => None,
        })
        .ok_or(Illegal::NotLegal)?;
    state.player_mut(me).spend_pp(cost);
    if let Some(f) = state.field_inst_mut(me, slot.0) {
        f.flags.engaged_this_turn = true;
    }
    raise_when(db, state, me, EventName::AllyEngage, Some(&inst), me);
    let src = SourceRef::Field {
        player: me,
        id: inst.id,
    };
    push_effects(state, me, src, fx);
    if sacrifice {
        // Engage self-sacrifice is destruction — 2026-09-09
        destroy_slot(db, state, me, slot.0, false, events)?;
    }
    Ok(())
}

fn apply_fuse_start(
    db: &CardDb,
    state: &mut State,
    host: u8,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let me = state.active;
    let inst = state
        .player(me)
        .hand
        .get(host as usize)
        .cloned()
        .ok_or(Illegal::NotLegal)?;
    let card = db.card(inst.card).map_err(|_| Illegal::NotLegal)?;
    let fuse = card.fuse().ok_or(Illegal::NotLegal)?;
    let options = fuse_partner_indices(db, state, me, host as usize, fuse);
    state.phase = Phase::Choice {
        player: me,
        node: ChoiceNode::FusePartners {
            host,
            options,
            picked: Vec::new(),
        },
    };
    events.push(Event::ChoiceOffered {
        player: me,
        node: state.phase_node(),
    });
    Ok(())
}

impl State {
    fn phase_node(&self) -> ChoiceNode {
        match &self.phase {
            Phase::Choice { node, .. } => node.clone(),
            _ => ChoiceNode::Modes {
                options: vec![],
                pending: PendingChoice {
                    kind: PendingKind::ModeSelect,
                    remaining: 1,
                },
            },
        }
    }
}

fn apply_choose(
    db: &CardDb,
    state: &mut State,
    i: u8,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let Phase::Choice { player, node } = state.phase.clone() else {
        return Err(Illegal::NotLegal);
    };
    match node {
        ChoiceNode::FusePartners {
            host,
            options,
            mut picked,
        } => {
            let Some(&pos) = options.get(i as usize) else {
                return Err(Illegal::NotLegal);
            };
            if let Some(p) = picked.iter().position(|&x| x == pos) {
                picked.remove(p);
            } else {
                picked.push(pos);
            }
            state.phase = Phase::Choice {
                player,
                node: ChoiceNode::FusePartners {
                    host,
                    options,
                    picked,
                },
            };
        }
        ChoiceNode::Modes { options, pending } => {
            let idx = options.get(i as usize).copied().ok_or(Illegal::NotLegal)?;
            state.phase = Phase::Main;
            resume_mode(db, state, player, idx, pending, events)?;
        }
        ChoiceNode::Targets { options, pending } => {
            let opt = options.get(i as usize).cloned().ok_or(Illegal::NotLegal)?;
            state.phase = Phase::Main;
            resume_target(db, state, player, opt, pending, events)?;
        }
        ChoiceNode::Cards { options, pending } => {
            let card = options.get(i as usize).copied().ok_or(Illegal::NotLegal)?;
            state.phase = Phase::Main;
            resume_target(db, state, player, TargetOpt::Card(card), pending, events)?;
        }
        ChoiceNode::MultiPick {
            options,
            mut picked,
            pending,
        } => {
            if (i as usize) >= options.len() && !picked.contains(&i) {
                return Err(Illegal::NotLegal);
            }
            if let Some(p) = picked.iter().position(|&x| x == i) {
                picked.remove(p);
            } else {
                picked.push(i);
            }
            state.phase = Phase::Choice {
                player,
                node: ChoiceNode::MultiPick {
                    options,
                    picked,
                    pending,
                },
            };
        }
    }
    Ok(())
}

fn apply_confirm(db: &CardDb, state: &mut State, events: &mut Vec<Event>) -> Result<(), Illegal> {
    let Phase::Choice { player, node } = state.phase.clone() else {
        return Err(Illegal::NotLegal);
    };
    match node {
        ChoiceNode::FusePartners { host, picked, .. } => {
            state.phase = Phase::Main;
            commit_fuse(db, state, player, host, &picked, events)?;
        }
        ChoiceNode::MultiPick {
            options,
            picked,
            pending,
        } => {
            state.phase = Phase::Main;
            for i in picked {
                if let Some(opt) = options.get(i as usize).cloned() {
                    resume_target(db, state, player, opt, pending.clone(), events)?;
                }
            }
        }
        _ => return Err(Illegal::NotLegal),
    }
    Ok(())
}

fn commit_fuse(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    host: u8,
    picked: &[u8],
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    if picked.is_empty() {
        return Err(Illegal::NotLegal);
    }
    let mut partners = Vec::new();
    let mut partner_ids = Vec::new();
    let mut total_cost = 0;
    let mut sorted: Vec<u8> = picked.to_vec();
    sorted.sort_by(|a, b| b.cmp(a));
    for pos in sorted {
        if (pos as usize) < state.player(me).hand.len() && pos != host {
            let inst = state.player_mut(me).hand.remove(pos as usize);
            total_cost += inst.cost;
            partner_ids.push(inst.card);
            partners.push(inst);
        }
    }
    let host_pos = if host as usize >= state.player(me).hand.len() {
        // host index may have shifted; find by... we stored host before removals.
        // Recalc: we removed from the right, so host shifts down for each removed > host?
        // We removed descending, so indices < host stay, indices > host shift -1 each greater.
        let shift = picked.iter().filter(|&&p| p < host).count();
        host as usize - shift
    } else {
        let shift = picked.iter().filter(|&&p| p < host).count();
        host as usize - shift
    };
    if host_pos >= state.player(me).hand.len() {
        return Err(Illegal::NotLegal);
    }
    let host_card = state.player(me).hand[host_pos].card;
    let card = db.card(host_card).map_err(|_| Illegal::NotLegal)?;
    events.push(Event::Fuse {
        host: host_card,
        partners: partner_ids.clone(),
    });
    // partners banished — ruling 2026-09-02
    for inst in partners {
        state.player_mut(me).banished.push(inst);
    }
    if let Some(h) = state.player_mut(me).hand.get_mut(host_pos) {
        h.flags.was_fused = true;
        h.flags.fused_this_turn = true;
        for pid in &partner_ids {
            if !h.flags.fused_kinds.contains(pid) {
                h.flags.fused_kinds.push(*pid);
            }
        }
    }
    if let Some(fuse) = card.fuse() {
        if let Some(recipes) = &fuse.recipes {
            let kinds = state.player(me).hand[host_pos].flags.fused_kinds.clone();
            for rec in recipes {
                let cost_ok = rec.cost_total.map(|n| total_cost == n).unwrap_or(true)
                    && rec.cost_total_gte.map(|n| total_cost >= n).unwrap_or(true);
                let req_ok = rec
                    .requires
                    .as_ref()
                    .map(|req| req.iter().all(|r| kinds.contains(r)))
                    .unwrap_or(true);
                if cost_ok && req_ok {
                    if let FuseResult::Transform { transform_into } = rec.result {
                        if let Ok(newc) = db.require_supported(transform_into) {
                            let mut neu =
                                CardInstance::from_card(newc, state.player(me).hand[host_pos].id);
                            // Transform is a new card: fresh fuse budget and empty
                            // β/γ memory (α itself starts empty). Keep was_fused.
                            // Ruling 2026-09-06 / 2026-08-29.
                            neu.flags.was_fused = true;
                            state.player_mut(me).hand[host_pos] = neu;
                            events.push(Event::Transform {
                                slot: Slot(0),
                                into: transform_into,
                            });
                        }
                    }
                    break;
                }
            }
        }
    }
    Ok(())
}

fn resume_mode(
    db: &CardDb,
    state: &mut State,
    _player: PlayerId,
    idx: u8,
    _pending: PendingChoice,
    _events: &mut [Event],
) -> Result<(), Illegal> {
    if let Some(WorkFrame::Effects {
        controller,
        source,
        effects,
        index,
    }) = state.pending_work.pop()
    {
        if let Some(Effect::Choose {
            options: Some(opts),
            ..
        }) = effects.get(index).cloned()
        {
            if let Some(opt) = opts.get(idx as usize) {
                let mut rest = effects;
                rest.remove(index);
                state.pending_work.push(WorkFrame::Effects {
                    controller,
                    source,
                    effects: rest,
                    index,
                });
                push_effects(state, controller, source, opt.effects.clone());
            }
        } else {
            state.pending_work.push(WorkFrame::Effects {
                controller,
                source,
                effects,
                index,
            });
        }
    }
    let _ = db;
    Ok(())
}

fn resume_target(
    db: &CardDb,
    state: &mut State,
    player: PlayerId,
    opt: TargetOpt,
    pending: PendingChoice,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    if let Some(WorkFrame::Effects {
        controller,
        source,
        effects,
        index,
    }) = state.pending_work.pop()
    {
        if index < effects.len() {
            let e = effects[index].clone();
            apply_effect_with_targets(db, state, controller, source, &e, &[opt], events)?;
            let left = pending.remaining.saturating_sub(1);
            if left > 0 {
                flush_reactions(db, state, events)?;
                if let Some(ChoiceNode::Targets { options, .. }) =
                    effect_choice_node(db, state, controller, source, &e)
                {
                    if !options.is_empty() {
                        state.pending_work.push(WorkFrame::Effects {
                            controller,
                            source,
                            effects,
                            index,
                        });
                        state.phase = Phase::Choice {
                            player,
                            node: ChoiceNode::Targets {
                                options: options.clone(),
                                pending: PendingChoice {
                                    kind: pending.kind,
                                    remaining: left,
                                },
                            },
                        };
                        events.push(Event::ChoiceOffered {
                            player,
                            node: state.phase_node(),
                        });
                        return Ok(());
                    }
                }
            }
            if index + 1 < effects.len() {
                state.pending_work.push(WorkFrame::Effects {
                    controller,
                    source,
                    effects,
                    index: index + 1,
                });
            }
        }
    }
    let _ = player;
    Ok(())
}

fn apply_end_turn(db: &CardDb, state: &mut State, events: &mut Vec<Event>) -> Result<(), Illegal> {
    let me = state.active;
    events.push(Event::TurnEnd { player: me });
    // two-phase end sequence — rulebook Start-of-Turn and End-of-Turn Sequences
    queue_turn_boundary(db, state, me, false);
    state
        .pending_work
        .push(WorkFrame::Aftermath(Aftermath::ContinueTurnEnd { step: 7 }));
    Ok(())
}

fn begin_turn(state: &mut State, who: PlayerId, events: &mut Vec<Event>) -> Result<(), Illegal> {
    state.active = who;
    state.phase = Phase::Main;
    let turn = {
        let p = state.player_mut(who);
        p.turns_taken += 1;
        if p.pp_max < PP_CAP {
            p.pp_max += 1;
        }
        p.pp = p.pp_max;
        p.bonus_pp.active = false;
        p.bonus_pp.locked = false;
        p.combo = 0;
        p.evolved_this_turn = false;
        p.played_this_turn.clear();
        p.attacked_leader_last_turn = p.attacked_leader_this_turn;
        p.attacked_leader_this_turn = false;
        for slot in p.field.iter_mut().flatten() {
            slot.flags.engaged_this_turn = false;
            slot.flags.attacked_this_turn = false;
            slot.flags.attacks_left = slot.traits.attacks_per_turn.unwrap_or(1);
            slot.flags.summoning_sick = false;
            slot.once_used.clear();
        }
        for c in &mut p.crests {
            c.once_used.clear();
        }
        for h in &mut p.hand {
            h.flags.fused_this_turn = false;
        }
        p.turns_taken
    };
    state.turn = turn;
    events.push(Event::TurnStart { player: who, turn });
    Ok(())
}

fn start_turn_full(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    begin_turn(state, who, events)?;
    queue_turn_boundary(db, state, who, true);
    state
        .pending_work
        .push(WorkFrame::Aftermath(Aftermath::ContinueTurnStart {
            step: 8,
        }));
    Ok(())
}

fn queue_turn_boundary(db: &CardDb, state: &mut State, whose_turn: PlayerId, start: bool) {
    // crests before board; owner-scoped — ruling 2026-08-31
    let own = whose_turn;
    let opp = whose_turn.opponent();
    if start {
        tick_crests(db, state, own);
        enqueue_boundary(db, state, own, true, Whose::Own, 3, start);
        tick_amulets(db, state, own);
        enqueue_boundary(db, state, own, false, Whose::Own, 4, start);
        enqueue_boundary(db, state, opp, true, Whose::Opponent, 5, start);
        enqueue_boundary(db, state, opp, false, Whose::Opponent, 6, start);
    } else {
        enqueue_boundary(db, state, own, true, Whose::Own, 3, start);
        enqueue_boundary(db, state, own, false, Whose::Own, 4, start);
        enqueue_boundary(db, state, opp, true, Whose::Opponent, 5, start);
        enqueue_boundary(db, state, opp, false, Whose::Opponent, 6, start);
    }
}

fn tick_crests(_db: &CardDb, state: &mut State, who: PlayerId) {
    let mut doomed = Vec::new();
    for (i, c) in state.player_mut(who).crests.iter_mut().enumerate() {
        if let Some(cd) = c.countdown.as_mut() {
            *cd -= 1;
            if *cd <= 0 {
                doomed.push(i);
            }
        }
    }
    for i in doomed.into_iter().rev() {
        let c = state.player_mut(who).crests.remove(i);
        // Last Words on crest — queued via abilities when we have db in drain
        state
            .pending_work
            .push(WorkFrame::Aftermath(Aftermath::DrainQueue));
        let _ = c;
    }
}

fn tick_amulets(db: &CardDb, state: &mut State, who: PlayerId) {
    // Countdown advances once at the owner's start of turn — rulebook Countdown.
    let mut doomed = Vec::new();
    for (i, s) in state.player_mut(who).field.iter_mut().enumerate() {
        if let Some(c) = s {
            if let Some(cd) = c.countdown.as_mut() {
                *cd -= 1;
                if *cd <= 0 {
                    doomed.push(i as u8);
                }
            }
        }
    }
    for slot in doomed {
        let _ = destroy_slot(db, state, who, slot, false, &mut Vec::new());
    }
}

fn enqueue_boundary(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    crests: bool,
    whose: Whose,
    cat: u8,
    start: bool,
) {
    let mut pending: Vec<(u32, u8, SourceRef, Ability)> = Vec::new();
    if crests {
        for (i, c) in state.player(who).crests.iter().enumerate() {
            if let Ok(def) = db.crest(&c.id) {
                for (ord, a) in def.abilities().iter().enumerate() {
                    if ability_boundary(a, whose, start) {
                        pending.push((
                            c.granted_order,
                            ord as u8,
                            SourceRef::Crest {
                                player: who,
                                index: i,
                            },
                            a.clone(),
                        ));
                    }
                }
            }
        }
    } else {
        for (si, s) in state.player(who).field.iter().enumerate() {
            let Some(inst) = s else { continue };
            if let Ok(card) = db.card(inst.card) {
                for (ord, a) in card.abilities().iter().enumerate() {
                    if ability_boundary(a, whose, start) {
                        pending.push((
                            si as u32,
                            ord as u8,
                            SourceRef::Field {
                                player: who,
                                id: inst.id,
                            },
                            a.clone(),
                        ));
                    }
                }
            }
        }
    }
    for (entry, printed, source, a) in pending {
        enqueue(state, cat, entry, printed, who, source, &a);
    }
}

fn ability_boundary(a: &Ability, whose: Whose, start: bool) -> bool {
    match a {
        Ability::StartOfTurn { whose: w, .. } => start && *w == whose,
        Ability::EndOfTurn { whose: w, .. } => !start && *w == whose,
        _ => false,
    }
}

fn enqueue(
    state: &mut State,
    cat: u8,
    entry: u32,
    printed: u8,
    controller: PlayerId,
    source: SourceRef,
    a: &Ability,
) {
    if a.once_per_turn() {
        // checked at fire
    }
    state.queue.push(QueuedTrigger {
        category: cat,
        entry,
        printed_order: printed,
        controller,
        source,
        tag: a.tag(),
        effects: a.effects().to_vec(),
    });
}

fn enqueue_card_triggers(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    inst_id: u32,
    tag: TriggerTag,
    cat: u8,
) {
    let Some(slot) = state.find_field(who, inst_id) else {
        return;
    };
    let Some(inst) = state.field_inst(who, slot).cloned() else {
        return;
    };
    let Ok(card) = db.card(inst.card) else { return };
    for (ord, a) in card
        .abilities()
        .iter()
        .chain(inst.granted.iter())
        .enumerate()
    {
        if a.tag() == tag {
            enqueue(
                state,
                cat,
                slot as u32,
                ord as u8,
                who,
                SourceRef::Field {
                    player: who,
                    id: inst.id,
                },
                a,
            );
        }
    }
}

fn queue_enter_reactions(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    card: CardId,
    inst_id: u32,
    from_play: bool,
) {
    let _ = (db, card, from_play);
    enqueue_card_triggers(db, state, me, inst_id, TriggerTag::Enter, 4);
}

fn raise_follower_enter(db: &CardDb, state: &mut State, owner: PlayerId, entered: &CardInstance) {
    raise_when(
        db,
        state,
        owner,
        EventName::AllyFollowerEnter,
        Some(entered),
        owner,
    );
    raise_when(
        db,
        state,
        owner.opponent(),
        EventName::EnemyFollowerEnter,
        Some(entered),
        owner,
    );
}

fn subject_target(state: &State, owner: PlayerId, inst: &CardInstance) -> Option<TargetOpt> {
    if let Some(slot) = state.find_field(owner, inst.id) {
        return Some(TargetOpt::Slot {
            player: owner,
            slot,
        });
    }
    state
        .player(owner)
        .hand
        .iter()
        .position(|c| c.id == inst.id)
        .map(|pos| TargetOpt::Hand {
            player: owner,
            pos: pos as u8,
        })
}

/// Enqueue matching `When` abilities of `observer_side`'s field cards and crests
/// (and hand/deck when `zone` says so). Active side category 4, opponent 6.
fn raise_when(
    db: &CardDb,
    state: &mut State,
    observer_side: PlayerId,
    event: EventName,
    subject: Option<&CardInstance>,
    subject_owner: PlayerId,
) {
    if event == EventName::SelfBuffedUp {
        if let Some(inst) = subject {
            enqueue_when_on(
                db,
                state,
                observer_side,
                event,
                subject,
                Some(inst.id),
                true,
            );
        }
        return;
    }
    if let Some(inst) = subject {
        state.event_subject = subject_target(state, subject_owner, inst);
    } else {
        state.event_subject = Some(TargetOpt::Leader {
            player: subject_owner,
        });
    }
    enqueue_when_on(db, state, observer_side, event, subject, None, false);
}

struct PendingWhen {
    entry: u32,
    printed: u8,
    source: SourceRef,
    ability: Ability,
    mark_id: Option<u32>,
    mark_crest: Option<usize>,
}

struct WhenScan<'a> {
    db: &'a CardDb,
    observer: PlayerId,
    event: EventName,
    subject: Option<&'a CardInstance>,
}

struct WhenLoc {
    zone: AbilityZone,
    entry: u32,
    source: SourceRef,
    mark_id: Option<u32>,
    mark_crest: Option<usize>,
}

fn enqueue_when_on(
    db: &CardDb,
    state: &mut State,
    observer_side: PlayerId,
    event: EventName,
    subject: Option<&CardInstance>,
    only_inst: Option<u32>,
    self_buff: bool,
) {
    let cat = if observer_side == state.active { 4 } else { 6 };
    let mut pending: Vec<PendingWhen> = Vec::new();
    let scan = WhenScan {
        db,
        observer: observer_side,
        event,
        subject,
    };

    let field: Vec<(u8, CardInstance)> = state
        .player(observer_side)
        .field
        .iter()
        .enumerate()
        .filter_map(|(si, s)| s.as_ref().map(|c| (si as u8, c.clone())))
        .collect();
    let crests = state.player(observer_side).crests.clone();
    let hand = state.player(observer_side).hand.clone();
    let deck = state.player(observer_side).deck.clone();

    for (si, inst) in &field {
        if let Some(id) = only_inst {
            if inst.id != id {
                continue;
            }
        }
        if self_buff {
            state.event_subject = Some(TargetOpt::Slot {
                player: observer_side,
                slot: *si,
            });
        }
        collect_when_from_abilities(
            &scan,
            state,
            WhenLoc {
                zone: AbilityZone::Field,
                entry: *si as u32,
                source: SourceRef::Field {
                    player: observer_side,
                    id: inst.id,
                },
                mark_id: Some(inst.id),
                mark_crest: None,
            },
            inst.card,
            &inst.granted,
            &inst.once_used,
            &mut pending,
        );
    }

    if only_inst.is_none() {
        for (i, c) in crests.iter().enumerate() {
            let Ok(def) = db.crest(&c.id) else { continue };
            collect_when_from_list(
                &scan,
                state,
                def.abilities(),
                WhenLoc {
                    zone: AbilityZone::Field,
                    entry: i as u32,
                    source: SourceRef::Crest {
                        player: observer_side,
                        index: i,
                    },
                    mark_id: None,
                    mark_crest: Some(i),
                },
                &c.once_used,
                &mut pending,
            );
        }
        for (hi, h) in hand.iter().enumerate() {
            collect_when_from_abilities(
                &scan,
                state,
                WhenLoc {
                    zone: AbilityZone::Hand,
                    entry: hi as u32,
                    source: SourceRef::Hand {
                        player: observer_side,
                        id: h.id,
                    },
                    mark_id: Some(h.id),
                    mark_crest: None,
                },
                h.card,
                &h.granted,
                &h.once_used,
                &mut pending,
            );
        }
        for (di, d) in deck.iter().enumerate() {
            collect_when_from_abilities(
                &scan,
                state,
                WhenLoc {
                    zone: AbilityZone::Deck,
                    entry: di as u32,
                    source: SourceRef::Hand {
                        player: observer_side,
                        id: d.id,
                    },
                    mark_id: Some(d.id),
                    mark_crest: None,
                },
                d.card,
                &d.granted,
                &d.once_used,
                &mut pending,
            );
        }
    }

    for PendingWhen {
        entry,
        printed,
        source,
        ability: a,
        mark_id,
        mark_crest,
    } in pending
    {
        if a.once_per_turn() {
            if let Some(id) = mark_id {
                if let Some(slot) = state.find_field(observer_side, id) {
                    if let Some(inst) = state.field_inst_mut(observer_side, slot) {
                        inst.once_used.push(TriggerTag::When);
                    }
                } else if let Some(h) = state
                    .player_mut(observer_side)
                    .hand
                    .iter_mut()
                    .find(|c| c.id == id)
                {
                    h.once_used.push(TriggerTag::When);
                }
            }
            if let Some(ci) = mark_crest {
                if let Some(c) = state.player_mut(observer_side).crests.get_mut(ci) {
                    c.once_used.push(TriggerTag::When);
                }
            }
        }
        enqueue(state, cat, entry, printed, observer_side, source, &a);
    }
}

fn collect_when_from_abilities(
    scan: &WhenScan<'_>,
    state: &State,
    loc: WhenLoc,
    card_id: CardId,
    granted: &[Ability],
    once_used: &[TriggerTag],
    pending: &mut Vec<PendingWhen>,
) {
    let mut list: Vec<Ability> = Vec::new();
    if let Ok(card) = scan.db.card(card_id) {
        list.extend(card.abilities().iter().cloned());
    }
    list.extend(granted.iter().cloned());
    collect_when_from_list(scan, state, &list, loc, once_used, pending);
}

fn collect_when_from_list(
    scan: &WhenScan<'_>,
    state: &State,
    abilities: &[Ability],
    loc: WhenLoc,
    once_used: &[TriggerTag],
    pending: &mut Vec<PendingWhen>,
) {
    for (ord, a) in abilities.iter().enumerate() {
        let Ability::When {
            event: ev, filter, ..
        } = a
        else {
            continue;
        };
        if *ev != scan.event {
            continue;
        }
        // Crests default to Field in the schema; treat them as in-play.
        if a.zone() != loc.zone {
            continue;
        }
        if let Some(f) = filter {
            let Some(subj) = scan.subject else { continue };
            if !inst_matches_filter(state, scan.observer, subj, f) {
                continue;
            }
        }
        if let Some(cond) = a.when_cond() {
            if !eval_cond(scan.db, state, scan.observer, Some(loc.source), cond) {
                continue;
            }
        }
        if a.once_per_turn() && once_used.contains(&TriggerTag::When) {
            continue;
        }
        pending.push(PendingWhen {
            entry: loc.entry,
            printed: ord as u8,
            source: loc.source,
            ability: a.clone(),
            mark_id: loc.mark_id,
            mark_crest: loc.mark_crest,
        });
    }
}

// =========================================================================
// Quiescence
// =========================================================================

fn drain_until_quiet(
    db: &CardDb,
    state: &mut State,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    loop {
        if state.winner.is_some() {
            state.phase = Phase::Terminal;
            return Ok(());
        }
        if matches!(state.phase, Phase::Choice { .. } | Phase::Mulligan { .. }) {
            return Ok(());
        }
        state.bump_step()?;
        // A mid-effect frame (index > 0) is the resolving effect: never interrupt it.
        // Newly pushed frames wait behind the reactive queue (play sequence,
        // Strike-before-damage, start-of-turn draw at step 8).
        // Rulebook: Trigger queue; Fanfare and Enter-Play Trigger Order;
        // Combat Timing; Start-of-Turn and End-of-Turn Sequences.
        let continue_effect = matches!(
            state.pending_work.last(),
            Some(WorkFrame::Effects { index, .. }) if *index > 0
        );
        if continue_effect {
            if let Some(WorkFrame::Effects {
                controller,
                source,
                effects,
                index,
            }) = state.pending_work.pop()
            {
                resolve_effect_list(db, state, controller, source, effects, index, events)?;
            }
            continue;
        }
        if !state.queue.is_empty() {
            drain_queue(db, state, events)?;
            continue;
        }
        if let Some(frame) = state.pending_work.pop() {
            match frame {
                WorkFrame::Effects {
                    controller,
                    source,
                    effects,
                    index,
                } => {
                    resolve_effect_list(db, state, controller, source, effects, index, events)?;
                }
                WorkFrame::Aftermath(a) => {
                    run_aftermath(db, state, a, events)?;
                }
            }
            continue;
        }
        settle_deaths(db, state, events)?;
        if state.pending_work.is_empty() && state.queue.is_empty() {
            commit_play_rally(state);
            if matches!(state.phase, Phase::Main | Phase::Combat | Phase::End) {
                state.phase = if state.winner.is_some() {
                    Phase::Terminal
                } else {
                    Phase::Main
                };
            }
            return Ok(());
        }
    }
}

fn run_aftermath(
    db: &CardDb,
    state: &mut State,
    a: Aftermath,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    match a {
        Aftermath::AfterCombat {
            attacker_player,
            attacker_id,
            target,
            knockback,
        } => {
            combat_damage(
                db,
                state,
                attacker_player,
                attacker_id,
                target,
                knockback,
                events,
            )?;
        }
        Aftermath::ContinueTurnEnd { step: 7 } => {
            // until EOT wears off — rulebook end step 7
            expire_eot(state, state.active);
            expire_eot(state, state.active.opponent());
            expire_until(state, Until::EndOfTurn, state.active);
            commit_bonus_pp(state.player_mut(state.active));
            let next = state.active.opponent();
            start_turn_full(db, state, next, events)?;
        }
        Aftermath::ContinueTurnStart { step: 8 } => {
            let who = state.active;
            if let Some(id) = draw_one(state, who, None)? {
                events.push(Event::Draw {
                    player: who,
                    card: id,
                });
                note_draw(db, state, who, id);
            }
        }
        Aftermath::DrainQueue => drain_queue(db, state, events)?,
        _ => {}
    }
    Ok(())
}

fn expire_eot(state: &mut State, who: PlayerId) {
    for slot in state.player_mut(who).field.iter_mut().flatten() {
        slot.attack -= slot.flags.eot_attack;
        slot.max_defense -= slot.flags.eot_defense;
        slot.defense = slot.defense.min(slot.max_defense);
        slot.flags.eot_attack = 0;
        slot.flags.eot_defense = 0;
        if let Some(c) = slot.flags.eot_cost.take() {
            slot.cost = c;
        }
    }
    for h in &mut state.player_mut(who).hand {
        if let Some(c) = h.flags.eot_cost.take() {
            h.cost = c;
        }
    }
}

fn expire_until(state: &mut State, until: Until, whose_turn_ended: PlayerId) {
    for p in PlayerId::ALL {
        state.player_mut(p).leader_mods.retain(|m| match m.until {
            Some(Until::EndOfTurn) => until != Until::EndOfTurn,
            Some(Until::EndOfOpponentTurn) => {
                !(until == Until::EndOfTurn && p == whose_turn_ended.opponent())
            }
            None => true,
        });
    }
}

/// Drain the reactive queue and any frames it pushes, leaving pre-existing
/// `pending_work` untouched. Used before a player-choice pause.
fn flush_reactions(db: &CardDb, state: &mut State, events: &mut Vec<Event>) -> Result<(), Illegal> {
    let floor = state.pending_work.len();
    loop {
        if state.winner.is_some() {
            return Ok(());
        }
        if matches!(state.phase, Phase::Choice { .. }) {
            return Ok(());
        }
        state.bump_step()?;
        if !state.queue.is_empty() {
            drain_queue(db, state, events)?;
            continue;
        }
        if state.pending_work.len() <= floor {
            break;
        }
        match state.pending_work.pop() {
            Some(WorkFrame::Effects {
                controller,
                source,
                effects,
                index,
            }) => {
                resolve_effect_list(db, state, controller, source, effects, index, events)?;
            }
            Some(WorkFrame::Aftermath(a)) => {
                run_aftermath(db, state, a, events)?;
            }
            None => break,
        }
    }
    Ok(())
}

fn drain_queue(db: &CardDb, state: &mut State, _events: &mut [Event]) -> Result<(), Illegal> {
    if state.queue.is_empty() {
        return Ok(());
    }
    state
        .queue
        .sort_by_key(|t| (t.category, t.entry, t.printed_order));
    let t = state.queue.remove(0);
    state.bindings.clear();
    push_effects(state, t.controller, t.source, t.effects);
    let _ = db;
    Ok(())
}

fn push_effects(state: &mut State, controller: PlayerId, source: SourceRef, effects: Vec<Effect>) {
    if effects.is_empty() {
        return;
    }
    state.pending_work.push(WorkFrame::Effects {
        controller,
        source,
        effects,
        index: 0,
    });
}

fn resolve_effect_list(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    effects: Vec<Effect>,
    index: usize,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    if index >= effects.len() {
        return Ok(());
    }
    let e = effects[index].clone();
    if let Some(cond) = e.when_cond() {
        if !eval_cond(db, state, controller, Some(source), cond) {
            if index + 1 < effects.len() {
                state.pending_work.push(WorkFrame::Effects {
                    controller,
                    source,
                    effects,
                    index: index + 1,
                });
            }
            return Ok(());
        }
    }
    // pause if this effect needs a player choose — drain reactions first
    // so the choice-node snapshot shows them (`docs/engine-internals.md`).
    if effect_choice_node(db, state, controller, source, &e).is_some() {
        flush_reactions(db, state, events)?;
        if matches!(state.phase, Phase::Choice { .. }) {
            state.pending_work.push(WorkFrame::Effects {
                controller,
                source,
                effects,
                index,
            });
            return Ok(());
        }
    }
    if let Some(node) = effect_choice_node(db, state, controller, source, &e) {
        state.pending_work.push(WorkFrame::Effects {
            controller,
            source,
            effects,
            index,
        });
        state.phase = Phase::Choice {
            player: controller,
            node,
        };
        events.push(Event::ChoiceOffered {
            player: controller,
            node: state.phase_node(),
        });
        return Ok(());
    }
    apply_effect(db, state, controller, source, &e, events)?;
    if index + 1 < effects.len() && !matches!(state.phase, Phase::Choice { .. }) {
        state.pending_work.push(WorkFrame::Effects {
            controller,
            source,
            effects,
            index: index + 1,
        });
    }
    Ok(())
}

fn effect_choice_node(
    db: &CardDb,
    state: &State,
    controller: PlayerId,
    source: SourceRef,
    e: &Effect,
) -> Option<ChoiceNode> {
    match e {
        Effect::Choose {
            by: ChooseBy::Player,
            options: Some(opts),
            ..
        } => Some(ChoiceNode::Modes {
            options: (0..opts.len() as u8).collect(),
            pending: PendingChoice {
                kind: PendingKind::ModeSelect,
                remaining: 1,
            },
        }),
        Effect::Damage { select, .. }
        | Effect::Restore { select, .. }
        | Effect::Buff { select, .. }
        | Effect::Destroy { select, .. }
        | Effect::Banish { select, .. }
        | Effect::ReturnToHand { select, .. }
        | Effect::ReturnToDeck { select, .. }
        | Effect::Discard { select, .. }
        | Effect::Evolve { select, .. }
        | Effect::GrantTraits { select, .. }
        | Effect::RemoveTraits { select, .. }
        | Effect::Countdown { select, .. } => {
            if let Selector::Pool(p) = select {
                if p.pick == PoolPick::Choose {
                    let opts = pool_target_opts(db, state, controller, source, p);
                    if opts.is_empty() {
                        return None; // fizzle
                    }
                    let n = p
                        .count
                        .as_ref()
                        .map(|a| eval_amount(db, state, controller, Some(source), a).max(1) as u8)
                        .unwrap_or(1);
                    let remaining = n.max(1).min(opts.len() as u8);
                    return Some(ChoiceNode::Targets {
                        options: opts,
                        pending: PendingChoice {
                            kind: PendingKind::EffectSelect,
                            remaining,
                        },
                    });
                }
            }
            None
        }
        _ => None,
    }
}

fn apply_effect_with_targets(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    e: &Effect,
    targets: &[TargetOpt],
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    match e {
        Effect::Damage { amount, .. } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            apply_each_captured(state, targets, |st, t| {
                deal_to_opt(db, st, controller, t, n, events)
            })?;
        }
        Effect::Restore { amount, .. } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            apply_each_captured(state, targets, |st, t| {
                restore_opt(db, st, t, n, events);
                Ok(())
            })?;
        }
        Effect::Destroy { .. } => {
            apply_each_captured(state, targets, |st, t| {
                if let TargetOpt::Slot { player, slot } = t {
                    destroy_slot(db, st, *player, *slot, false, events)?;
                }
                Ok(())
            })?;
        }
        Effect::Banish { .. } => {
            apply_each_captured(state, targets, |st, t| {
                banish_opt(st, t, events);
                Ok(())
            })?;
        }
        Effect::Buff {
            attack,
            defense,
            until_end_of_turn,
            ..
        } => {
            let da = attack
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a))
                .unwrap_or(0);
            let dd = defense
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a))
                .unwrap_or(0);
            apply_each_captured(state, targets, |st, t| {
                buff_opt(db, st, t, da, dd, until_end_of_turn.unwrap_or(false));
                Ok(())
            })?;
        }
        Effect::ReturnToHand { .. } => {
            for t in targets {
                bounce_opt(db, state, controller, t, events)?;
            }
        }
        Effect::ReturnToDeck { .. } => {
            for t in targets {
                return_deck_opt(db, state, t);
            }
        }
        Effect::Discard { .. } => {
            for t in targets {
                discard_opt(db, state, t, events)?;
            }
        }
        Effect::Evolve { super_evolve, .. } => {
            for t in targets {
                if let TargetOpt::Slot { player, slot } = t {
                    if *player == controller {
                        apply_evolve_action(db, state, Slot(*slot), *super_evolve, true, events)?;
                    }
                }
            }
        }
        Effect::GrantTraits { traits, .. } => {
            for t in targets {
                grant_traits_opt(state, t, traits);
            }
        }
        Effect::RemoveTraits { traits, .. } => {
            for t in targets {
                remove_traits_opt(state, t, traits);
            }
        }
        Effect::Countdown { delta, .. } => {
            let d = eval_amount(db, state, controller, Some(source), delta);
            for t in targets {
                countdown_opt(db, state, t, d, events)?;
            }
        }
        _ => {
            apply_effect(db, state, controller, source, e, events)?;
            return Ok(());
        }
    }
    maybe_bind(state, e, targets);
    Ok(())
}

fn apply_effect(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    e: &Effect,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    match e {
        Effect::Seq { effects, .. } => {
            push_effects(state, controller, source, effects.clone());
        }
        Effect::If {
            cond,
            then,
            else_effects,
            ..
        } => {
            if eval_cond(db, state, controller, Some(source), cond) {
                push_effects(state, controller, source, then.clone());
            } else if let Some(els) = else_effects {
                push_effects(state, controller, source, els.clone());
            }
        }
        Effect::Repeat { times, effects, .. } => {
            let n = eval_amount(db, state, controller, Some(source), times).max(0);
            for _ in 0..n {
                push_effects(state, controller, source, effects.clone());
            }
        }
        Effect::Pay {
            resource,
            amount,
            effects,
            ..
        } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            if pay_resource(state, controller, *resource, n) {
                if *resource == PayResource::Earth {
                    raise_when(
                        db,
                        state,
                        controller,
                        EventName::AllyEarthRite,
                        None,
                        controller,
                    );
                }
                push_effects(state, controller, source, effects.clone());
            }
        }
        Effect::Choose {
            by, pick, options, ..
        } => match by {
            ChooseBy::Player => {}
            ChooseBy::Random | ChooseBy::RandomUnused => {
                if let Some(opts) = options {
                    let n = match pick.as_pick() {
                        crate::card::ChoosePick::All => opts.len(),
                        crate::card::ChoosePick::N(k) => k as usize,
                    };
                    let mut unused: Vec<usize> = (0..opts.len()).collect();
                    for _ in 0..n {
                        if unused.is_empty() {
                            break;
                        }
                        let keys: Vec<String> = unused.iter().map(|i| i.to_string()).collect();
                        let mut emit = Vec::new();
                        let j = state
                            .rng
                            .pick_index(PickWhat::RandomUnused, &keys, &mut emit)
                            .map_err(Illegal::OraclePickNotLegal)?;
                        state.picks.extend(emit);
                        let oi = unused.remove(j.min(unused.len() - 1));
                        push_effects(state, controller, source, opts[oi].effects.clone());
                    }
                }
            }
        },
        Effect::Damage {
            select,
            amount,
            split,
            ..
        } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            if *split == Some(true) {
                split_damage(db, state, controller, &ts, n, events)?;
            } else {
                apply_each_captured(state, &ts, |st, t| {
                    deal_to_opt(db, st, controller, t, n, events)
                })?;
            }
        }
        Effect::Restore { select, amount, .. } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| {
                restore_opt(db, st, t, n, events);
                Ok(())
            })?;
        }
        Effect::Buff {
            select,
            attack,
            defense,
            until_end_of_turn,
            ..
        } => {
            let da = attack
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a))
                .unwrap_or(0);
            let dd = defense
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a))
                .unwrap_or(0);
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| {
                buff_opt(db, st, t, da, dd, until_end_of_turn.unwrap_or(false));
                Ok(())
            })?;
        }
        Effect::Destroy { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| {
                if let TargetOpt::Slot { player, slot } = t {
                    destroy_slot(db, st, *player, *slot, false, events)?;
                }
                Ok(())
            })?;
        }
        Effect::Banish { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| {
                banish_opt(st, t, events);
                Ok(())
            })?;
        }
        Effect::ReturnToHand { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| {
                bounce_opt(db, st, controller, t, events)
            })?;
        }
        Effect::ReturnToDeck { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| {
                return_deck_opt(db, st, t);
                Ok(())
            })?;
        }
        Effect::Summon {
            card,
            count,
            controller: ctrl,
            ..
        } => {
            let n = eval_amount(db, state, controller, Some(source), count).max(0);
            let who = match ctrl {
                Some(Controller::Opponent) => controller.opponent(),
                _ => controller,
            };
            let mut summoned = Vec::new();
            for _ in 0..n {
                // sequential one-at-a-time — ruling 2026-09-05
                if let Some(t) = summon_source(db, state, who, card, false, events)? {
                    summoned.push(t);
                }
            }
            maybe_bind(state, e, &summoned);
        }
        Effect::AddToHand { card, count, .. } => {
            let n = eval_amount(db, state, controller, Some(source), count).max(0);
            for _ in 0..n {
                add_source_to_hand(db, state, controller, card)?;
            }
        }
        Effect::Draw { count, filter, .. } => {
            let n = eval_amount(db, state, controller, Some(source), count).max(0);
            for _ in 0..n {
                if let Some(id) = draw_one(state, controller, filter.as_ref())? {
                    events.push(Event::Draw {
                        player: controller,
                        card: id,
                    });
                    note_draw(db, state, controller, id);
                }
            }
        }
        Effect::Discard { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| discard_opt(db, st, t, events))?;
        }
        Effect::Evolve {
            select,
            super_evolve,
            ..
        } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| {
                if let TargetOpt::Slot { player, slot } = t {
                    if *player == controller {
                        apply_evolve_action(db, st, Slot(*slot), *super_evolve, true, events)?;
                    }
                }
                Ok(())
            })?;
        }
        Effect::GrantTraits { select, traits, .. } => {
            let ts = resolve_select(db, state, controller, source, select);
            for t in &ts {
                grant_traits_opt(state, t, traits);
            }
            maybe_bind(state, e, &ts);
        }
        Effect::RemoveTraits { select, traits, .. } => {
            let ts = resolve_select(db, state, controller, source, select);
            for t in &ts {
                remove_traits_opt(state, t, traits);
            }
            maybe_bind(state, e, &ts);
        }
        Effect::Pp { action, amount, .. } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            let p = state.player_mut(controller);
            match action {
                PpAction::GainMax => {
                    p.pp_max = (p.pp_max + n).min(PP_CAP);
                }
                PpAction::Recover => p.recover_pp(n),
                PpAction::Spend => p.spend_pp(n),
            }
        }
        Effect::Crest { gain, player, .. } => {
            let who = match player {
                CrestPlayer::Self_ => controller,
                CrestPlayer::Opponent => controller.opponent(),
            };
            gain_crest(db, state, who, &gain.0, events);
        }
        Effect::RemoveCrests { select, .. } => {
            // "banish all crests" — Faith icons survive (rulebook) but none in M1
            let _ = select;
            let who = match select {
                Selector::Pool(p) if p.side == Side::Enemy => controller.opponent(),
                _ => controller,
            };
            let mut kept = Vec::new();
            for c in state.player_mut(who).crests.drain(..) {
                if c.faith {
                    kept.push(c);
                } else {
                    events.push(Event::CrestRemove {
                        player: who,
                        id: c.id.clone(),
                    });
                }
            }
            state.player_mut(who).crests = kept;
        }
        Effect::Countdown { select, delta, .. } => {
            let d = eval_amount(db, state, controller, Some(source), delta);
            for t in resolve_select(db, state, controller, source, select) {
                countdown_opt(db, state, &t, d, events)?;
            }
        }
        Effect::Counter {
            key, how, amount, ..
        } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            apply_counter(state, controller, source, key, *how, n, events);
        }
        Effect::Reanimate { max_cost, .. } => {
            let x = eval_amount(db, state, controller, Some(source), max_cost);
            reanimate(db, state, controller, x, events)?;
        }
        Effect::Replicate { ability, .. } => {
            replicate(db, state, controller, source, *ability)?;
        }
        Effect::SpellboostHand { times, .. } => {
            let n = eval_amount(db, state, controller, Some(source), times).max(0);
            for _ in 0..n {
                spellboost_hand(db, state, controller, events)?;
            }
        }
        Effect::LeaderModifier {
            select,
            max_defense,
            damage_cap,
            damage_taken_bonus,
            until,
            ..
        } => {
            let _ = select;
            let md_v = max_defense
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a));
            let cap_v = damage_cap
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a));
            let bonus_v = damage_taken_bonus
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a))
                .unwrap_or(0);
            let p = state.player_mut(controller);
            if let Some(v) = md_v {
                p.leader_max = v;
                p.leader_defense = p.leader_defense.min(v);
            }
            p.leader_mods.push(LeaderMod {
                max_defense: None,
                damage_cap: cap_v,
                damage_taken_bonus: bonus_v,
                until: *until,
            });
        }
        Effect::Cost {
            select,
            delta,
            set,
            until_end_of_turn,
            ..
        } => {
            for t in resolve_select(db, state, controller, source, select) {
                cost_opt(
                    state,
                    &t,
                    delta.as_ref(),
                    set.as_ref(),
                    until_end_of_turn.unwrap_or(false),
                    db,
                    controller,
                    source,
                );
            }
        }
        Effect::GrantAbility {
            select, ability, ..
        } => {
            for t in resolve_select(db, state, controller, source, select) {
                if let TargetOpt::Slot { player, slot } = t {
                    if let Some(f) = state.field_inst_mut(player, slot) {
                        f.granted.push((**ability).clone());
                    }
                }
            }
        }
        Effect::RemoveAbilities { select, on, .. } => {
            let ts = resolve_select(db, state, controller, source, select);
            for t in &ts {
                if let TargetOpt::Slot { player, slot } = t {
                    if let Some(f) = state.field_inst_mut(*player, *slot) {
                        if let Some(tags) = on {
                            f.granted.retain(|a| !tags.contains(&a.tag()));
                        } else {
                            f.granted.clear();
                        }
                    }
                }
            }
            maybe_bind(state, e, &ts);
        }
        Effect::Invoke { .. }
        | Effect::RandomSplit { .. }
        | Effect::Sequence { .. }
        | Effect::Transform { .. }
        | Effect::AddToDeck { .. }
        | Effect::Ep { .. } => {
            return Err(Illegal::Unsupported(Unsupported {
                card: format!("{controller:?}"),
                construct: "reached unimplemented op".into(),
            }));
        }
    }
    Ok(())
}

fn pay_resource(state: &mut State, who: PlayerId, res: PayResource, n: i32) -> bool {
    let p = state.player_mut(who);
    match res {
        PayResource::Shadows => {
            if p.shadows >= n {
                p.shadows -= n;
                true
            } else {
                false
            }
        }
        PayResource::Earth => {
            if p.earth >= n {
                p.earth -= n;
                if p.earth <= 0 {
                    if let Some(slot) = p.earth_slot.take() {
                        p.field[slot as usize] = None;
                        p.compact_field();
                    }
                    p.earth = 0;
                }
                true
            } else {
                false
            }
        }
        PayResource::Pp => {
            if p.usable_pp() >= n {
                p.spend_pp(n);
                true
            } else {
                false
            }
        }
        PayResource::Faith => false,
    }
}

fn apply_counter(
    state: &mut State,
    who: PlayerId,
    source: SourceRef,
    key: &CounterKey,
    how: CounterHow,
    n: i32,
    events: &mut Vec<Event>,
) {
    match key {
        CounterKey::Named(NamedCounter::Combo) => {
            let p = state.player_mut(who);
            p.combo = if how == CounterHow::Set {
                n
            } else {
                p.combo + n
            };
            events.push(Event::Counter {
                key: "combo".into(),
                value: p.combo,
            });
        }
        CounterKey::Named(NamedCounter::Earth) => {
            let p = state.player_mut(who);
            p.earth = if how == CounterHow::Set {
                n
            } else {
                p.earth + n
            };
            events.push(Event::Counter {
                key: "earth".into(),
                value: p.earth,
            });
        }
        CounterKey::Named(NamedCounter::Shadows) => {
            let p = state.player_mut(who);
            p.shadows = if how == CounterHow::Set {
                n
            } else {
                p.shadows + n
            };
        }
        CounterKey::Var { var } => {
            if let SourceRef::Field { player, id } | SourceRef::Hand { player, id } = source {
                let p = state.player_mut(player);
                if let Some(h) = p.hand.iter_mut().find(|c| c.id == id) {
                    let e = h.vars.entry(*var).or_insert(0);
                    *e = if how == CounterHow::Set { n } else { *e + n };
                }
                if let Some(slot) = p.field.iter_mut().flatten().find(|c| c.id == id) {
                    let e = slot.vars.entry(*var).or_insert(0);
                    *e = if how == CounterHow::Set { n } else { *e + n };
                }
            }
        }
        _ => {}
    }
}

fn combat_damage(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    attacker_id: u32,
    target: AttackTarget,
    knockback: bool,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let Some(slot) = state.find_field(me, attacker_id) else {
        return Ok(());
    };
    let Some(att) = state.field_inst(me, slot).cloned() else {
        return Ok(());
    };
    let opp = me.opponent();
    match target {
        AttackTarget::Leader => {
            deal_leader(state, opp, att.attack.max(0), events);
            if att.is_drain() {
                restore_leader(db, state, me, att.attack.max(0), events);
            }
        }
        AttackTarget::Slot(ds) => {
            let Some(def) = state.field_inst(opp, ds.0).cloned() else {
                return Ok(());
            };
            deal_follower(state, opp, ds.0, att.attack.max(0), me, events);
            deal_follower(state, me, slot, def.attack.max(0), opp, events);
            if att.is_drain() {
                restore_leader(db, state, me, att.attack.max(0), events);
            }
            // Bane even at 0 — rulebook Bane
            if att.is_bane() {
                if let Some(d) = state.field_inst(opp, ds.0) {
                    if !bane_blocked(state, opp, d) {
                        destroy_slot(db, state, opp, ds.0, false, events)?;
                    }
                }
            }
            if def.is_bane() {
                if let Some(a) = state.field_inst(me, slot) {
                    if !bane_blocked(state, me, a) {
                        destroy_slot(db, state, me, slot, false, events)?;
                    }
                }
            }
            settle_deaths(db, state, events)?;
            let target_dead = state.field_inst(opp, ds.0).is_none();
            if knockback && target_dead {
                deal_leader(state, opp, 1, events);
            }
        }
    }
    Ok(())
}

/// Super-evolved own-turn protection: cannot be destroyed by abilities/effects
/// (including Bane). Rulebook Evolution stat bonuses.
fn bane_blocked(state: &State, owner: PlayerId, inst: &CardInstance) -> bool {
    inst.traits.cant_be_destroyed_by_abilities == Some(true)
        || (inst.super_evolved && state.active == owner)
}

fn deal_to_opt(
    _db: &CardDb,
    state: &mut State,
    _ctrl: PlayerId,
    t: &TargetOpt,
    n: i32,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    match t {
        TargetOpt::Leader { player } => deal_leader(state, *player, n, events),
        TargetOpt::Slot { player, slot } => {
            deal_follower(state, *player, *slot, n, *player, events)
        }
        _ => {}
    }
    Ok(())
}

fn deal_leader(state: &mut State, who: PlayerId, raw: i32, events: &mut Vec<Event>) {
    let bonus = state.player(who).damage_taken_bonus();
    let mut amt = raw + bonus;
    if let Some(cap) = state.player(who).damage_cap() {
        amt = amt.min(cap);
    }
    amt = amt.max(0);
    // Barrier on leader is modeled as a leader_mod? skip — no M1 card
    let p = state.player_mut(who);
    p.leader_defense = (p.leader_defense - amt).max(0);
    let lethal = p.leader_defense <= 0;
    events.push(Event::Damage {
        target: EventTarget::Leader(who),
        amount: amt,
        lethal,
    });
    if lethal {
        check_leader_lethal(state);
    }
}

/// Simultaneous leader lethal ⇒ the **active** player loses. Rulebook Win/Loss.
fn check_leader_lethal(state: &mut State) {
    let a_dead = state.player(PlayerId::A).leader_defense <= 0;
    let b_dead = state.player(PlayerId::B).leader_defense <= 0;
    if a_dead && b_dead {
        state.winner = Some(state.active.opponent());
        state.phase = Phase::Terminal;
    } else if a_dead {
        state.winner = Some(PlayerId::B);
        state.phase = Phase::Terminal;
    } else if b_dead {
        state.winner = Some(PlayerId::A);
        state.phase = Phase::Terminal;
    }
}

fn deal_follower(
    state: &mut State,
    who: PlayerId,
    slot: u8,
    raw: i32,
    _src_player: PlayerId,
    events: &mut Vec<Event>,
) {
    let active = state.active;
    let Some(f) = state.field_inst_mut(who, slot) else {
        return;
    };
    let mut amt = raw;
    // "takes N more damage" stacks and applies to a 0-damage event — 2026-08-31
    // (follower-level bonus not separately stored in M1 beyond leader mods)
    if f.is_barrier() {
        f.traits.barrier = None;
        amt = 0;
    } else if f.super_evolved && active == who {
        // own-turn protection: damage to 0 but counts as taking damage — 2026-08-23
        amt = 0;
    }
    if let Some(cap) = f.traits.damage_cap {
        amt = amt.min(cap);
    }
    amt = amt.max(0);
    f.defense -= amt;
    let lethal = f.defense <= 0;
    events.push(Event::Damage {
        target: EventTarget::Slot(who, Slot(slot)),
        amount: amt,
        lethal,
    });
}

fn restore_opt(db: &CardDb, state: &mut State, t: &TargetOpt, n: i32, events: &mut Vec<Event>) {
    match t {
        TargetOpt::Leader { player } => restore_leader(db, state, *player, n, events),
        TargetOpt::Slot { player, slot } => {
            if let Some(f) = state.field_inst_mut(*player, *slot) {
                let room = (f.max_defense - f.defense).max(0);
                let g = n.min(room);
                f.defense += g;
                events.push(Event::Restore {
                    target: EventTarget::Slot(*player, Slot(*slot)),
                    amount: g,
                });
            }
        }
        _ => {}
    }
}

fn restore_leader(db: &CardDb, state: &mut State, who: PlayerId, n: i32, events: &mut Vec<Event>) {
    let p = state.player_mut(who);
    let room = (p.leader_max - p.leader_defense).max(0);
    let g = n.min(room);
    p.leader_defense += g;
    events.push(Event::Restore {
        target: EventTarget::Leader(who),
        amount: g,
    });
    if g > 0 {
        raise_when(db, state, who, EventName::LeaderRestored, None, who);
    }
}

fn buff_opt(db: &CardDb, state: &mut State, t: &TargetOpt, da: i32, dd: i32, eot: bool) {
    if let TargetOpt::Slot { player, slot } = t {
        if let Some(f) = state.field_inst_mut(*player, *slot) {
            f.attack += da;
            f.max_defense += dd;
            f.defense += dd;
            if eot {
                f.flags.eot_attack += da;
                f.flags.eot_defense += dd;
            }
        }
        if (da > 0 || dd > 0) && state.field_inst(*player, *slot).is_some() {
            if let Some(inst) = state.field_inst(*player, *slot).cloned() {
                raise_when(
                    db,
                    state,
                    *player,
                    EventName::SelfBuffedUp,
                    Some(&inst),
                    *player,
                );
            }
        }
    }
}

fn grant_traits_opt(state: &mut State, t: &TargetOpt, traits: &Traits) {
    if let TargetOpt::Slot { player, slot } = t {
        if let Some(f) = state.field_inst_mut(*player, *slot) {
            f.traits.merge_grant(traits);
            if traits.ambush == Some(true) {
                f.flags.ambush_active = true;
            }
            if let Some(n) = traits.attacks_per_turn {
                f.flags.attacks_left = f.flags.attacks_left.max(n);
            }
        }
    }
}

fn remove_traits_opt(state: &mut State, t: &TargetOpt, traits: &Traits) {
    if let TargetOpt::Slot { player, slot } = t {
        if let Some(f) = state.field_inst_mut(*player, *slot) {
            f.traits.merge_remove(traits);
            if traits.ambush == Some(true) {
                f.flags.ambush_active = false;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cost_opt(
    state: &mut State,
    t: &TargetOpt,
    delta: Option<&Amount>,
    set: Option<&Amount>,
    eot: bool,
    db: &CardDb,
    controller: PlayerId,
    source: SourceRef,
) {
    let set_v = set.map(|s| eval_amount(db, state, controller, Some(source), s));
    let delta_v = delta.map(|d| eval_amount(db, state, controller, Some(source), d));
    match t {
        TargetOpt::Hand { player, pos } => {
            if let Some(h) = state.player_mut(*player).hand.get_mut(*pos as usize) {
                if eot && h.flags.eot_cost.is_none() {
                    h.flags.eot_cost = Some(h.cost);
                }
                if let Some(s) = set_v {
                    h.cost = s;
                } else if let Some(d) = delta_v {
                    h.cost += d;
                }
                h.cost = h.cost.max(0);
            }
        }
        TargetOpt::Slot { player, slot } => {
            if let Some(f) = state.field_inst_mut(*player, *slot) {
                if eot && f.flags.eot_cost.is_none() {
                    f.flags.eot_cost = Some(f.cost);
                }
                if let Some(s) = set_v {
                    f.cost = s;
                } else if let Some(d) = delta_v {
                    f.cost += d;
                }
                f.cost = f.cost.max(0);
            }
        }
        _ => {}
    }
}

fn destroy_slot(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    slot: u8,
    no_lw: bool,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let Some(inst) = state.player_mut(who).field[slot as usize].take() else {
        return Ok(());
    };
    events.push(Event::Destroy {
        slot: Slot(slot),
        card: inst.card,
    });
    match inst.kind {
        CardKind::Follower => {
            raise_when(
                db,
                state,
                who,
                EventName::AllyFollowerDestroyed,
                Some(&inst),
                who,
            );
        }
        CardKind::Amulet => {
            raise_when(
                db,
                state,
                who,
                EventName::AllyAmuletDestroyed,
                Some(&inst),
                who,
            );
        }
        CardKind::Spell => {}
    }
    if inst.kind == CardKind::Follower {
        state
            .player_mut(who)
            .destroyed_history
            .push(DestroyedRecord {
                card: inst.card,
                base_cost: inst.base_cost,
                kind: inst.kind,
                owner: who,
                from_field: true,
            });
    }
    if !no_lw {
        if let Ok(card) = db.card(inst.card) {
            for a in card.abilities() {
                if matches!(a, Ability::LastWords { .. }) {
                    state.queue.push(QueuedTrigger {
                        category: if who == state.active { 4 } else { 6 },
                        entry: slot as u32,
                        printed_order: 0,
                        controller: who,
                        source: SourceRef::Spell {
                            player: who,
                            card: inst.card,
                        },
                        tag: TriggerTag::LastWords,
                        effects: a.effects().to_vec(),
                    });
                }
            }
        }
    }
    state.player_mut(who).shadows += 1;
    state.player_mut(who).cemetery.push(inst);
    state.player_mut(who).compact_field();
    Ok(())
}

fn banish_opt(state: &mut State, t: &TargetOpt, events: &mut Vec<Event>) {
    match t {
        TargetOpt::Slot { player, slot } => {
            if let Some(inst) = state.player_mut(*player).field[*slot as usize].take() {
                events.push(Event::Banish {
                    card: inst.card,
                    from: ZoneLabel::Field,
                });
                state.player_mut(*player).banished.push(inst);
                state.player_mut(*player).compact_field();
            }
        }
        TargetOpt::Hand { player, pos } if (*pos as usize) < state.player(*player).hand.len() => {
            let inst = state.player_mut(*player).hand.remove(*pos as usize);
            events.push(Event::Banish {
                card: inst.card,
                from: ZoneLabel::Hand,
            });
            state.player_mut(*player).banished.push(inst);
        }
        _ => {}
    }
}

fn bounce_opt(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    t: &TargetOpt,
    _events: &mut [Event],
) -> Result<(), Illegal> {
    match t {
        TargetOpt::Slot { player, slot } => {
            if let Some(inst) = state.player_mut(*player).field[*slot as usize].take() {
                state.player_mut(*player).compact_field();
                add_to_hand(state, *player, reset_off_field(db, inst));
            }
        }
        TargetOpt::Card(id) => {
            // Search / put-from-deck: the pick (if random) already ran.
            for who in [controller, controller.opponent()] {
                if let Some(inst) = remove_from_deck_by_id(state, who, *id) {
                    add_to_hand(state, controller, inst);
                    break;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn return_deck_opt(db: &CardDb, state: &mut State, t: &TargetOpt) {
    match t {
        TargetOpt::Hand { player, pos } if (*pos as usize) < state.player(*player).hand.len() => {
            let inst = state.player_mut(*player).hand.remove(*pos as usize);
            state.player_mut(*player).deck.push(inst);
        }
        TargetOpt::Slot { player, slot } => {
            if let Some(inst) = state.player_mut(*player).field[*slot as usize].take() {
                state.player_mut(*player).compact_field();
                state
                    .player_mut(*player)
                    .deck
                    .push(reset_off_field(db, inst));
            }
        }
        _ => {}
    }
}

/// Field → hand/deck is a fresh printed copy. `was_fused` is the one flag
/// that stays (ruling 2026-08-29).
fn reset_off_field(db: &CardDb, inst: CardInstance) -> CardInstance {
    let Ok(card) = db.card(inst.card) else {
        return inst;
    };
    let mut neu = CardInstance::from_card(card, inst.id);
    neu.flags.was_fused = inst.flags.was_fused;
    neu
}

fn discard_opt(
    db: &CardDb,
    state: &mut State,
    t: &TargetOpt,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    if let TargetOpt::Hand { player, pos } = t {
        if (*pos as usize) < state.player(*player).hand.len() {
            let inst = state.player_mut(*player).hand.remove(*pos as usize);
            if let Ok(card) = db.card(inst.card) {
                for a in card.abilities() {
                    if matches!(a, Ability::Discarded { .. }) {
                        state.queue.push(QueuedTrigger {
                            category: if *player == state.active { 4 } else { 6 },
                            entry: *pos as u32,
                            printed_order: 0,
                            controller: *player,
                            source: SourceRef::Spell {
                                player: *player,
                                card: inst.card,
                            },
                            tag: TriggerTag::Discarded,
                            effects: a.effects().to_vec(),
                        });
                    }
                }
            }
            state.player_mut(*player).shadows += 1;
            state.player_mut(*player).cemetery.push(inst);
        }
    }
    let _ = events;
    Ok(())
}

fn countdown_opt(
    db: &CardDb,
    state: &mut State,
    t: &TargetOpt,
    delta: i32,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    if let TargetOpt::Slot { player, slot } = t {
        if let Some(f) = state.field_inst_mut(*player, *slot) {
            if let Some(cd) = f.countdown.as_mut() {
                *cd += delta;
                if *cd <= 0 {
                    destroy_slot(db, state, *player, *slot, false, events)?;
                }
            }
        }
    }
    Ok(())
}

fn settle_deaths(db: &CardDb, state: &mut State, events: &mut Vec<Event>) -> Result<(), Illegal> {
    let mut dead = Vec::new();
    for p in PlayerId::ALL {
        for (i, s) in state.player(p).field.iter().enumerate() {
            if let Some(c) = s {
                if c.kind == CardKind::Follower && c.defense <= 0 {
                    dead.push((p, i as u8));
                }
            }
        }
    }
    dead.sort_by_key(|a| std::cmp::Reverse(a.1));
    for (p, slot) in dead {
        destroy_slot(db, state, p, slot, false, events)?;
    }
    Ok(())
}

fn split_damage(
    db: &CardDb,
    state: &mut State,
    ctrl: PlayerId,
    targets: &[TargetOpt],
    mut pool: i32,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    // oldest first — Barrier consumes allocation — 2026-08-29
    let caps = capture_targets(state, targets);
    for cap in caps {
        if pool <= 0 {
            break;
        }
        let Some(TargetOpt::Slot { player, slot }) = live_captured(state, &cap) else {
            continue;
        };
        let def = state
            .field_inst(player, slot)
            .map(|c| c.defense.max(0))
            .unwrap_or(0);
        let take = pool.min(def.max(1));
        deal_follower(state, player, slot, take, ctrl, events);
        pool -= take;
    }
    let _ = db;
    Ok(())
}

fn summon_source(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    src: &CardSource,
    exact: bool,
    events: &mut Vec<Event>,
) -> Result<Option<TargetOpt>, Illegal> {
    let inst = match src {
        CardSource::Named { named } => {
            let card = db.require_supported(*named).map_err(|e| match e {
                LoadError::Unsupported(u) => Illegal::Unsupported(u),
                _ => Illegal::NotLegal,
            })?;
            CardInstance::from_card(card, state.alloc_id())
        }
        CardSource::Copy { copy_of, exact: ex } => {
            let ts = resolve_select(db, state, who, SourceRef::Leader { player: who }, copy_of);
            if let Some(TargetOpt::Slot { player, slot }) = ts.first() {
                if let Some(c) = state.field_inst(*player, *slot).cloned() {
                    let mut n = if *ex {
                        c
                    } else {
                        let card = db.card(c.card).map_err(|_| Illegal::NotLegal)?;
                        CardInstance::from_card(card, 0)
                    };
                    n.id = state.alloc_id();
                    n.flags.summoning_sick = true;
                    n
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        }
        CardSource::RandomFrom { .. } => {
            return Err(Illegal::Unsupported(Unsupported {
                card: who.as_str().into(),
                construct: "CardSource.randomFrom".into(),
            }));
        }
    };
    let _ = exact;
    let mut inst = inst;
    inst.flags.summoning_sick = true;
    if inst.is_earth_sigil() && merge_earth(db, state, who, &inst, events)? {
        return Ok(None);
    }
    let Some(slot) = state.player(who).first_empty_slot() else {
        return Ok(None);
    };
    if inst.kind == CardKind::Follower {
        state.player_mut(who).rally += 1;
        *state
            .player_mut(who)
            .enter_counts
            .entry(inst.card)
            .or_insert(0) += 1;
    }
    let id = inst.card;
    let kind = inst.kind;
    state.player_mut(who).field[slot as usize] = Some(inst);
    events.push(Event::Summon {
        player: who,
        card: id,
        slot: Slot(slot),
    });
    if kind == CardKind::Follower {
        if let Some(entered) = state.field_inst(who, slot).cloned() {
            raise_follower_enter(db, state, who, &entered);
        }
        queue_enter_reactions(
            db,
            state,
            who,
            id,
            state.field_inst(who, slot).map(|c| c.id).unwrap_or(0),
            false,
        );
    }
    Ok(Some(TargetOpt::Slot { player: who, slot }))
}

fn add_source_to_hand(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    src: &CardSource,
) -> Result<(), Illegal> {
    match src {
        CardSource::Named { named } => {
            let card = db.require_supported(*named).map_err(|e| match e {
                LoadError::Unsupported(u) => Illegal::Unsupported(u),
                _ => Illegal::NotLegal,
            })?;
            let inst = CardInstance::from_card(card, state.alloc_id());
            add_to_hand(state, who, inst);
            Ok(())
        }
        CardSource::Copy { copy_of, exact } => {
            let ts = resolve_select(db, state, who, SourceRef::Leader { player: who }, copy_of);
            if let Some(TargetOpt::Slot { player, slot }) = ts.first() {
                if let Some(c) = state.field_inst(*player, *slot).cloned() {
                    let mut n = if *exact {
                        c
                    } else {
                        let card = db.card(c.card).map_err(|_| Illegal::NotLegal)?;
                        CardInstance::from_card(card, 0)
                    };
                    n.id = state.alloc_id();
                    add_to_hand(state, who, n);
                }
            }
            Ok(())
        }
        _ => Err(Illegal::Unsupported(Unsupported {
            card: who.as_str().into(),
            construct: "CardSource.randomFrom".into(),
        })),
    }
}

/// Reanimate: field-destroyed only, highest cost ≤ X, random among ties,
/// summoning-sick, no Fanfare — rulings 2026-09-02.
fn reanimate(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    max_cost: i32,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let cands: Vec<(CardId, i32)> = state
        .player(who)
        .destroyed_history
        .iter()
        .filter(|r| r.from_field && r.kind == CardKind::Follower && r.base_cost <= max_cost)
        .map(|r| (r.card, r.base_cost))
        .collect();
    if cands.is_empty() {
        return Ok(());
    }
    let best = cands.iter().map(|(_, c)| *c).max().unwrap_or(0);
    let tied: Vec<CardId> = cands
        .into_iter()
        .filter(|(_, c)| *c == best)
        .map(|(id, _)| id)
        .collect();
    let keys: Vec<String> = tied.iter().map(|c| c.as_str()).collect();
    let mut emit = Vec::new();
    let i = state
        .rng
        .pick_index(PickWhat::Reanimate, &keys, &mut emit)
        .map_err(Illegal::OraclePickNotLegal)?;
    state.picks.extend(emit);
    let id = tied[i.min(tied.len() - 1)];
    let card = db.card(id).map_err(|_| Illegal::NotLegal)?;
    let mut inst = CardInstance::from_card(card, state.alloc_id());
    inst.flags.summoning_sick = true;
    let Some(slot) = state.player(who).first_empty_slot() else {
        return Ok(());
    };
    state.player_mut(who).rally += 1;
    state.player_mut(who).field[slot as usize] = Some(inst);
    events.push(Event::Summon {
        player: who,
        card: id,
        slot: Slot(slot),
    });
    if let Some(entered) = state.field_inst(who, slot).cloned() {
        raise_follower_enter(db, state, who, &entered);
        queue_enter_reactions(db, state, who, id, entered.id, false);
    }
    Ok(())
}

fn replicate(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    key: ReplicateKey,
) -> Result<(), Illegal> {
    let card_id = match source {
        SourceRef::Field { player, id } => state
            .find_field(player, id)
            .and_then(|s| state.field_inst(player, s).map(|c| c.card)),
        SourceRef::Spell { card, .. } => Some(card),
        _ => None,
    };
    let Some(cid) = card_id else {
        return Ok(());
    };
    let card = db.card(cid).map_err(|_| Illegal::NotLegal)?;
    let tag = match key {
        ReplicateKey::Fanfare => TriggerTag::Fanfare,
        ReplicateKey::Evolve => TriggerTag::Evolve,
        ReplicateKey::SuperEvolve => TriggerTag::SuperEvolve,
        ReplicateKey::LastWords => TriggerTag::LastWords,
        ReplicateKey::Engage => TriggerTag::Engage,
        ReplicateKey::Strike => TriggerTag::Strike,
        ReplicateKey::FollowerStrike => TriggerTag::FollowerStrike,
        ReplicateKey::Clash => TriggerTag::Clash,
    };
    let fx: Vec<Effect> = card
        .abilities()
        .iter()
        .filter(|a| a.tag() == tag)
        .flat_map(|a| a.effects().iter().cloned())
        .collect();
    push_effects(state, controller, source, fx);
    Ok(())
}

fn gain_crest(db: &CardDb, state: &mut State, who: PlayerId, id: &str, events: &mut Vec<Event>) {
    if state.player(who).crests.iter().any(|c| c.id == id) {
        // duplicate bounces — 2026-09-05
        return;
    }
    let icons = state.player(who).crests.len();
    if icons >= CREST_CAP {
        return;
    }
    let cd = db.crest(id).ok().and_then(|c| c.countdown);
    let faith = db.crest(id).map(|c| c.faith).unwrap_or(false);
    let order = state.crest_order;
    state.crest_order += 1;
    state.player_mut(who).crests.push(CrestInstance {
        id: id.to_string(),
        countdown: cd,
        faith,
        once_used: Vec::new(),
        granted_order: order,
    });
    events.push(Event::CrestGain {
        player: who,
        id: id.to_string(),
    });
}

// ----- selectors / filters / conditions / amounts -----

fn maybe_bind(state: &mut State, e: &Effect, targets: &[TargetOpt]) {
    if let Some(name) = e.as_bind() {
        let refs: Vec<BoundRef> = targets
            .iter()
            .filter_map(|t| opt_to_bound(state, t))
            .collect();
        state.bindings.insert(name.to_string(), refs);
    }
}

fn opt_to_bound(state: &State, t: &TargetOpt) -> Option<BoundRef> {
    match t {
        TargetOpt::Slot { player, slot } => {
            state.field_inst(*player, *slot).map(|c| BoundRef::Field {
                player: *player,
                id: c.id,
            })
        }
        TargetOpt::Leader { player } => Some(BoundRef::Leader { player: *player }),
        TargetOpt::Hand { player, pos } => {
            state
                .player(*player)
                .hand
                .get(*pos as usize)
                .map(|c| BoundRef::Hand {
                    player: *player,
                    id: c.id,
                })
        }
        TargetOpt::Card(id) => Some(BoundRef::Card(*id)),
        TargetOpt::Mode(_) => None,
    }
}

fn resolve_bound(state: &State, name: &str) -> Vec<TargetOpt> {
    let Some(refs) = state.bindings.get(name) else {
        return Vec::new();
    };
    refs.iter()
        .filter_map(|r| match r {
            BoundRef::Field { player, id } => {
                state.find_field(*player, *id).map(|slot| TargetOpt::Slot {
                    player: *player,
                    slot,
                })
            }
            BoundRef::Leader { player } => Some(TargetOpt::Leader { player: *player }),
            BoundRef::Hand { player, id } => state
                .player(*player)
                .hand
                .iter()
                .position(|c| c.id == *id)
                .map(|pos| TargetOpt::Hand {
                    player: *player,
                    pos: pos as u8,
                }),
            BoundRef::Card(id) => Some(TargetOpt::Card(*id)),
        })
        .collect()
}

fn resolve_select_rolling(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    sel: &Selector,
) -> Result<Vec<TargetOpt>, Illegal> {
    match sel {
        Selector::Pool(p) if p.pick == PoolPick::Random || p.pick == PoolPick::RandomDistinct => {
            let cands = pool_target_opts(db, state, controller, source, p);
            let n = p
                .count
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a).max(1) as usize)
                .unwrap_or(1);
            random_pool_apply(state, cands, n, p.pick == PoolPick::RandomDistinct)
        }
        Selector::Pool(p) if p.pick == PoolPick::Highest || p.pick == PoolPick::Lowest => {
            let cands = pool_target_opts(db, state, controller, source, p);
            pick_extremum(state, p, cands, p.pick == PoolPick::Highest)
        }
        _ => Ok(resolve_select(db, state, controller, source, sel)),
    }
}

fn resolve_select(
    db: &CardDb,
    state: &State,
    controller: PlayerId,
    source: SourceRef,
    sel: &Selector,
) -> Vec<TargetOpt> {
    match sel {
        Selector::Ref(r) => match r.pick {
            RefPick::Self_ => source_as_target(state, source).into_iter().collect(),
            RefPick::Entering => state.event_subject.clone().into_iter().collect(),
            RefPick::Selected | RefPick::Attacker | RefPick::Defender | RefPick::Opposing => {
                Vec::new()
            }
        },
        Selector::Bound(b) => resolve_bound(state, &b.ref_name),
        Selector::Pool(p) => {
            let mut c = pool_target_opts(db, state, controller, source, p);
            match p.pick {
                PoolPick::All | PoolPick::Choose => c,
                PoolPick::Leftmost => c.into_iter().take(1).collect(),
                PoolPick::Random | PoolPick::RandomDistinct => {
                    // caller that needs RNG should use pick; here take first for
                    // non-choice resolution — apply_effect rolls below via helper
                    pick_random_targets(state, p, &mut c)
                }
                PoolPick::Highest | PoolPick::Lowest => extremum_without_roll(state, p, c),
            }
        }
    }
}

fn pick_random_targets(state: &State, p: &PoolSelector, cands: &mut [TargetOpt]) -> Vec<TargetOpt> {
    let n = p.count.as_ref().map(|_| 1).unwrap_or(1);
    let _ = (state, n);
    if cands.is_empty() {
        return Vec::new();
    }
    // Live roll happens in apply_effect via dedicated path; here we return all
    // and let damage/random use rng in a wrapper. For random pick we take 1
    // via a deterministic fallback only when scripted consumed elsewhere.
    cands.iter().take(1).cloned().collect()
}

fn source_as_target(state: &State, source: SourceRef) -> Option<TargetOpt> {
    match source {
        SourceRef::Field { player, id } => state
            .find_field(player, id)
            .map(|slot| TargetOpt::Slot { player, slot }),
        SourceRef::Hand { player, id } => state
            .player(player)
            .hand
            .iter()
            .position(|c| c.id == id)
            .map(|pos| TargetOpt::Hand {
                player,
                pos: pos as u8,
            }),
        SourceRef::Leader { player } => Some(TargetOpt::Leader { player }),
        SourceRef::Spell { player, .. } => Some(TargetOpt::Leader { player }),
        SourceRef::Crest { player, .. } => Some(TargetOpt::Leader { player }),
    }
}

fn pool_target_opts(
    db: &CardDb,
    state: &State,
    controller: PlayerId,
    source: SourceRef,
    p: &PoolSelector,
) -> Vec<TargetOpt> {
    pool_candidates(db, state, controller, Some(source), p, None)
}

fn pool_candidates(
    db: &CardDb,
    state: &State,
    controller: PlayerId,
    source: Option<SourceRef>,
    p: &PoolSelector,
    exclude_hand: Option<usize>,
) -> Vec<TargetOpt> {
    let sides: Vec<PlayerId> = match p.side {
        Side::Ally => vec![controller],
        Side::Enemy => vec![controller.opponent()],
        Side::Any => vec![controller, controller.opponent()],
    };
    let mut out = Vec::new();
    for who in sides {
        match p.zone {
            Zone::Leader => {
                if matches!(
                    p.kind,
                    SelectorKind::Leader | SelectorKind::Character | SelectorKind::Card
                ) {
                    out.push(TargetOpt::Leader { player: who });
                }
            }
            Zone::Field => {
                for (i, s) in state.player(who).field.iter().enumerate() {
                    let Some(c) = s else { continue };
                    if p.other == Some(true) {
                        if let Some(SourceRef::Field { id, .. }) = source {
                            if c.id == id {
                                continue;
                            }
                        }
                    }
                    if !kind_ok(c, p.kind) {
                        continue;
                    }
                    if c.kind == CardKind::Follower && c.defense <= 0 {
                        continue;
                    }
                    if let Some(f) = &p.filter {
                        if !inst_matches_filter(state, who, c, f) {
                            continue;
                        }
                    }
                    // Aura: not selectable by the enemy
                    if who != controller && c.is_aura() && p.pick == PoolPick::Choose {
                        continue;
                    }
                    if who != controller && c.ambush_blocks() && p.pick == PoolPick::Choose {
                        continue;
                    }
                    out.push(TargetOpt::Slot {
                        player: who,
                        slot: i as u8,
                    });
                }
                // `kind: character` includes the leader; `includeLeader` includes
                // it for any kind. docs/schema.md `includeLeader`.
                if p.kind == SelectorKind::Character || p.include_leader == Some(true) {
                    out.push(TargetOpt::Leader { player: who });
                }
            }
            Zone::Hand => {
                for (i, c) in state.player(who).hand.iter().enumerate() {
                    if exclude_hand == Some(i) && who == controller {
                        continue;
                    }
                    if let Some(f) = &p.filter {
                        if !inst_matches_filter(state, who, c, f) {
                            continue;
                        }
                    }
                    out.push(TargetOpt::Hand {
                        player: who,
                        pos: i as u8,
                    });
                }
            }
            Zone::Deck => {
                for c in &state.player(who).deck {
                    if !kind_ok(c, p.kind) {
                        continue;
                    }
                    if let Some(f) = &p.filter {
                        if !inst_matches_filter(state, who, c, f) {
                            continue;
                        }
                    }
                    out.push(TargetOpt::Card(c.card));
                }
            }
            Zone::Cemetery | Zone::Crests => {}
        }
    }
    let _ = db;
    out
}

fn kind_ok(c: &CardInstance, k: SelectorKind) -> bool {
    match k {
        SelectorKind::Follower => c.kind == CardKind::Follower,
        SelectorKind::Amulet => c.kind == CardKind::Amulet,
        SelectorKind::Card => true,
        SelectorKind::Character => c.kind == CardKind::Follower,
        SelectorKind::Leader | SelectorKind::Faith => false,
    }
}

fn inst_matches_filter(state: &State, who: PlayerId, c: &CardInstance, f: &Filter) -> bool {
    if let Some(all) = &f.all {
        return all.iter().all(|x| inst_matches_filter(state, who, c, x));
    }
    if let Some(any) = &f.any {
        return any.iter().any(|x| inst_matches_filter(state, who, c, x));
    }
    if let Some(n) = &f.not {
        return !inst_matches_filter(state, who, c, n);
    }
    if let Some(t) = &f.tribe {
        let ok = match t {
            crate::card::TribeOrList::One(tr) => c.tribes.contains(tr),
            crate::card::TribeOrList::Many(ts) => ts.iter().any(|tr| c.tribes.contains(tr)),
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
            FilterKind::Follower => c.kind == CardKind::Follower,
            FilterKind::Spell => c.kind == CardKind::Spell,
            FilterKind::Amulet => c.kind == CardKind::Amulet,
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
    if let Some(n) = &f.cost_eq {
        if c.cost != eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.cost_lte {
        if c.cost > eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.cost_gte {
        if c.cost < eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(ns) = &f.cost_in {
        if !ns.contains(&c.cost) {
            return false;
        }
    }
    if let Some(n) = &f.base_cost_gte {
        if c.base_cost < eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(b) = f.evolved {
        if c.evolved != b {
            return false;
        }
    }
    if let Some(b) = f.unevolved {
        if c.evolved == b {
            return false;
        }
    }
    if let Some(b) = f.damaged {
        if c.damaged() != b {
            return false;
        }
    }
    if let Some(tk) = f.has_trait {
        if !c.traits.is_set(tk) {
            return false;
        }
    }
    if let Some(b) = f.has_last_words {
        let _ = (b, who, state);
    }
    true
}

fn filter_matches_card(state: &State, who: PlayerId, inst: &CardInstance, f: &Filter) -> bool {
    inst_matches_filter(state, who, inst, f)
}

fn eval_amount_simple(a: &Amount) -> i32 {
    match a {
        Amount::Int(n) => *n,
        _ => 0,
    }
}

fn eval_amount(
    db: &CardDb,
    state: &State,
    who: PlayerId,
    source: Option<SourceRef>,
    a: &Amount,
) -> i32 {
    match a {
        Amount::Int(n) => *n,
        Amount::Count { count } => {
            if let Some(src) = source {
                resolve_select(db, state, who, src, count).len() as i32
            } else {
                0
            }
        }
        Amount::Counter { counter } => match counter {
            CounterKey::Named(NamedCounter::Combo) => state.player(who).combo,
            CounterKey::Named(NamedCounter::Earth) => state.player(who).earth,
            CounterKey::Named(NamedCounter::Shadows) => state.player(who).shadows,
            CounterKey::Named(NamedCounter::Faith) => state.player(who).faith,
            _ => 0,
        },
        Amount::Add { add } => {
            eval_amount(db, state, who, source, &add[0])
                + eval_amount(db, state, who, source, &add[1])
        }
        Amount::Sub { sub } => {
            eval_amount(db, state, who, source, &sub[0])
                - eval_amount(db, state, who, source, &sub[1])
        }
        Amount::Max { max } => eval_amount(db, state, who, source, &max[0])
            .max(eval_amount(db, state, who, source, &max[1])),
        Amount::Min { min } => eval_amount(db, state, who, source, &min[0])
            .min(eval_amount(db, state, who, source, &min[1])),
        Amount::Neg { neg } => -eval_amount(db, state, who, source, neg),
        Amount::Var { var } => {
            if let Some(SourceRef::Field { player, id } | SourceRef::Hand { player, id }) = source {
                if let Some(c) = state.player(player).hand.iter().find(|c| c.id == id) {
                    return *c.vars.get(var).unwrap_or(&0);
                }
                if let Some(slot) = state.find_field(player, id) {
                    if let Some(c) = state.field_inst(player, slot) {
                        return *c.vars.get(var).unwrap_or(&0);
                    }
                }
            }
            0
        }
        _ => 0,
    }
}

fn eval_cond(
    db: &CardDb,
    state: &State,
    who: PlayerId,
    source: Option<SourceRef>,
    c: &Condition,
) -> bool {
    match c {
        Condition::All { all } => all.iter().all(|x| eval_cond(db, state, who, source, x)),
        Condition::Any { any } => any.iter().any(|x| eval_cond(db, state, who, source, x)),
        Condition::Not { not } => !eval_cond(db, state, who, source, not),
        Condition::Combo { combo } => {
            state.player(who).combo >= eval_amount(db, state, who, source, &combo.n)
        }
        Condition::Rally { rally } => {
            state.player(who).rally >= eval_amount(db, state, who, source, &rally.n)
        }
        Condition::Overflow { overflow } => {
            // Overflow: max PP ≥ 7; Bonus PP does not count — rulebook Overflow
            (state.player(who).pp_max >= 7) == *overflow
        }
        Condition::MaxPpAtLeast { max_pp_at_least } => {
            // Dragonsign / schema `maxPpAtLeast` — current pp_max vs n at resolution.
            state.player(who).pp_max >= eval_amount(db, state, who, source, &max_pp_at_least.n)
        }
        Condition::Evolved { evolved } => {
            if let Some(SourceRef::Field { player, id }) = source {
                state
                    .find_field(player, id)
                    .and_then(|s| state.field_inst(player, s))
                    .map(|c| c.evolved == *evolved)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        Condition::WasFused { was_fused } => {
            let flag = match source {
                Some(SourceRef::Field { player, id } | SourceRef::Hand { player, id }) => state
                    .player(player)
                    .hand
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.flags.was_fused)
                    .or_else(|| {
                        state
                            .find_field(player, id)
                            .and_then(|s| state.field_inst(player, s).map(|c| c.flags.was_fused))
                    })
                    .unwrap_or(false),
                _ => false,
            };
            match was_fused {
                crate::card::WasFused::Flag(b) => flag == *b,
                crate::card::WasFused::Both(_) => flag,
            }
        }
        Condition::AttackedLeaderLastTurn {
            attacked_leader_last_turn,
        } => state.player(who).attacked_leader_last_turn == *attacked_leader_last_turn,
        Condition::SuperEvolutionUnlocked {
            super_evolution_unlocked,
        } => {
            let p = state.player(who);
            let unlock = if p.is_second { 6 } else { 7 };
            (p.turns_taken >= unlock) == *super_evolution_unlocked
        }
        Condition::CountAtLeast { count_at_least } => {
            if let Some(src) = source {
                resolve_select(db, state, who, src, &count_at_least.select).len() as i32
                    >= eval_amount(db, state, who, source, &count_at_least.n)
            } else {
                false
            }
        }
        Condition::CounterAtLeast { counter_at_least } => {
            eval_amount(
                db,
                state,
                who,
                source,
                &Amount::Counter {
                    counter: counter_at_least.key.clone(),
                },
            ) >= eval_amount(db, state, who, source, &counter_at_least.n)
        }
        _ => false,
    }
}

enum CapturedTarget {
    Field { player: PlayerId, id: u32 },
    Keep(TargetOpt),
}

fn capture_targets(state: &State, ts: &[TargetOpt]) -> Vec<CapturedTarget> {
    ts.iter()
        .map(|t| match t {
            TargetOpt::Slot { player, slot } => {
                if let Some(c) = state.field_inst(*player, *slot) {
                    CapturedTarget::Field {
                        player: *player,
                        id: c.id,
                    }
                } else {
                    CapturedTarget::Keep(t.clone())
                }
            }
            other => CapturedTarget::Keep(other.clone()),
        })
        .collect()
}

fn live_captured(state: &State, cap: &CapturedTarget) -> Option<TargetOpt> {
    match cap {
        CapturedTarget::Field { player, id } => {
            state.find_field(*player, *id).map(|slot| TargetOpt::Slot {
                player: *player,
                slot,
            })
        }
        CapturedTarget::Keep(t) => Some(t.clone()),
    }
}

fn apply_each_captured<F>(state: &mut State, ts: &[TargetOpt], mut f: F) -> Result<(), Illegal>
where
    F: FnMut(&mut State, &TargetOpt) -> Result<(), Illegal>,
{
    let caps = capture_targets(state, ts);
    for cap in caps {
        if let Some(t) = live_captured(state, &cap) {
            f(state, &t)?;
        }
    }
    Ok(())
}

fn order_key(state: &State, t: &TargetOpt, order: Option<OrderBy>) -> i32 {
    let Some(c) = (match t {
        TargetOpt::Slot { player, slot } => state.field_inst(*player, *slot),
        TargetOpt::Hand { player, pos } => state.player(*player).hand.get(*pos as usize),
        _ => None,
    }) else {
        return 0;
    };
    match order {
        Some(OrderBy::Attack) => c.attack,
        Some(OrderBy::Defense) => c.defense,
        Some(OrderBy::Cost) => c.cost,
        Some(OrderBy::BaseCost) => c.base_cost,
        None => 0,
    }
}

fn extremum_without_roll(state: &State, p: &PoolSelector, cands: Vec<TargetOpt>) -> Vec<TargetOpt> {
    if cands.is_empty() {
        return cands;
    }
    let high = p.pick == PoolPick::Highest;
    let best = cands.iter().map(|t| order_key(state, t, p.order_by)).fold(
        if high { i32::MIN } else { i32::MAX },
        |acc, k| if high { acc.max(k) } else { acc.min(k) },
    );
    cands
        .into_iter()
        .filter(|t| order_key(state, t, p.order_by) == best)
        .take(1)
        .collect()
}

fn pick_extremum(
    state: &mut State,
    p: &PoolSelector,
    cands: Vec<TargetOpt>,
    highest: bool,
) -> Result<Vec<TargetOpt>, Illegal> {
    if cands.is_empty() {
        return Ok(cands);
    }
    let best = cands.iter().map(|t| order_key(state, t, p.order_by)).fold(
        if highest { i32::MIN } else { i32::MAX },
        |acc, k| if highest { acc.max(k) } else { acc.min(k) },
    );
    let tied: Vec<TargetOpt> = cands
        .into_iter()
        .filter(|t| order_key(state, t, p.order_by) == best)
        .collect();
    random_pool_apply(state, tied, 1, false)
}

fn random_pool_apply(
    state: &mut State,
    cands: Vec<TargetOpt>,
    n: usize,
    distinct: bool,
) -> Result<Vec<TargetOpt>, Illegal> {
    if cands.is_empty() {
        return Ok(Vec::new());
    }
    let deck_search = cands.iter().all(|t| matches!(t, TargetOpt::Card(_)));
    let mut left = cands;
    let mut out = Vec::new();
    for _ in 0..n {
        if left.is_empty() {
            break;
        }
        let keys: Vec<String> = left
            .iter()
            .map(|t| match t {
                TargetOpt::Slot { slot, .. } => format!("slot:{slot}"),
                TargetOpt::Leader { .. } => "leader".into(),
                TargetOpt::Card(c) => c.as_str(),
                TargetOpt::Hand { pos, .. } => format!("hand:{pos}"),
                TargetOpt::Mode(m) => format!("mode:{m}"),
            })
            .collect();
        let mut emit = Vec::new();
        let i = if deck_search {
            state
                .rng
                .pick_index_among(PickWhat::MultisetPick, Some("deck"), &keys, &mut emit)
                .map_err(Illegal::OraclePickNotLegal)?
        } else {
            state
                .rng
                .pick_index(PickWhat::RandomTarget, &keys, &mut emit)
                .map_err(Illegal::OraclePickNotLegal)?
        };
        state.picks.extend(emit);
        let i = i.min(left.len() - 1);
        if distinct {
            out.push(left.remove(i));
        } else {
            out.push(left[i].clone());
        }
    }
    Ok(out)
}

/// Remove one copy of `id` from `who`'s deck. No extra pick — the caller
/// already recorded `multiset_pick` (or the player chose).
fn remove_from_deck_by_id(state: &mut State, who: PlayerId, id: CardId) -> Option<CardInstance> {
    let pos = state.player(who).deck.iter().position(|c| c.card == id)?;
    Some(state.player_mut(who).deck.remove(pos))
}

/// Used by soak / accounting.
pub fn zone_count(p: &PlayerState) -> usize {
    p.deck.len() + p.hand.len() + p.field_count() + p.cemetery.len() + p.banished.len()
}
