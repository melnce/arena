//! `apply` / `legal_actions` / `new_game`. Every rule cites its source.

use crate::action::{from_neutral, to_neutral, Action};
use crate::card::{
    Ability, AbilityZone, Amount, Card, CardId, CardKind, CardSource, ChooseBy, ChooseOption,
    ChoosePick, Condition, Controller, CounterHow, CounterKey, CrestPlayer, DeckPosition, Effect,
    EpAction, EventName, FieldHasKind, Filter, FilterKind, FuseResult, MaxDefenseChange, Mode,
    NamedCounter, OptionsFrom, OrderBy, PayResource, PoolPick, PoolSelector, PpAction, RefPick,
    ReplicateKey, Selector, SelectorKind, Side, StatWhich, Traits, TriggerTag, TurnOwner, Until,
    Whose, Zone,
};
use crate::db::CardDb;
use crate::error::{Illegal, LoadError, Unsupported};
use crate::event::{Event, EventTarget, ZoneLabel};
use crate::ids::{AttackTarget, First, PlayerId, Slot};
use crate::rng::GameRng;
use crate::state::{
    Aftermath, BoundRef, CardInstance, ChoiceNode, CrestInstance, DestroyedRecord, GameConfig,
    LeaderMod, PendingChoice, PendingKind, Phase, PlayForm, PlayerState, QueuedTrigger, SourceRef,
    State, TargetOpt, TempTraitGrant, WorkFrame, CREST_CAP, DECK_SIZE, HAND_LIMIT, PP_CAP,
};
use crate::support;
use crate::trace::{NeutralAction, PickWhat};

// Hand overflow: owner-rulings.md — Hand overflow destroys without Last Words — 2026-08-10

/// Official Q&A (World of Games `10503210` / Divine Thunder): "a card on
/// the field other than it" includes the enemy field. Owner 2026-09-10:
/// "yes any card". Set `false` to count allied field only (old engine).
const WORLD_OF_GAMES_COUNTS_EITHER_SIDE: bool = true;

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
        invoked_ids: std::collections::BTreeSet::new(),
        event_base_cost: None,
        event_inst_id: None,
        attack_target_is_leader: false,
        bind_append: false,
        attacking_follower: false,
        combat_opposing: None,
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
                let _ = draw_one(&mut state, p, None, &[]).map_err(|e| {
                    LoadError::Unsupported(Unsupported {
                        card: "draw".into(),
                        construct: e.to_string(),
                    })
                })?;
            }
        }
    }
    grant_starting_faiths(db, &mut state);
    state.phase = Phase::Mulligan { player: first };
    Ok(state)
}

/// Sham-Nacha / engine-api: a Faith crest is granted at match start to every
/// player whose starting deck or opening hand contains a card that carries it.
fn grant_starting_faiths(db: &CardDb, state: &mut State) {
    let faiths: Vec<(String, Vec<CardId>)> = db
        .crests
        .iter()
        .filter(|(_, c)| c.faith)
        .map(|(id, c)| (id.clone(), c.granted_by.clone()))
        .collect();
    if faiths.is_empty() {
        return;
    }
    for who in PlayerId::ALL {
        let mut ids = std::collections::BTreeSet::new();
        for c in state
            .player(who)
            .deck
            .iter()
            .chain(state.player(who).hand.iter())
        {
            ids.insert(c.card);
        }
        for (fid, granted_by) in &faiths {
            if granted_by.iter().any(|g| ids.contains(g)) {
                gain_crest(db, state, who, fid, &mut Vec::new());
            }
        }
    }
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
    // Draw appends; "when you draw this card" is the newest copy, not the
    // first same-id already in hand (Swift Staffmaster).
    let subject = state
        .player(who)
        .hand
        .iter()
        .rev()
        .find(|c| c.card == card)
        .cloned();
    raise_when(db, state, who, EventName::AllyDraw, subject.as_ref(), who);
}

fn draw_one(
    state: &mut State,
    who: PlayerId,
    filter: Option<&Filter>,
    exclude_names: &[CardId],
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
                !exclude_names.contains(&c.card)
                    && filter
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
    let among = if filter.is_some() { Some("deck") } else { None };
    let pick = state
        .rng
        .pick_index_among(PickWhat::Draw, among, &keys, &mut emit)
        .map_err(Illegal::OraclePickNotLegal)?;
    state.picks.extend(emit);
    // Duplicate ids can carry different mods (Thestae deck +1/+1 vs a copy
    // just returned). The recorded pick is the id; take the first matching
    // copy so arena-trace → arena-replay agrees.
    let chosen = state.player(who).deck[idxs[pick.min(idxs.len() - 1)]].card;
    let deck_i = idxs
        .iter()
        .copied()
        .find(|&i| state.player(who).deck[i].card == chosen)
        .unwrap_or(idxs[pick.min(idxs.len() - 1)]);
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
        Phase::Choice { .. } => legal_choice(state),
        Phase::Main | Phase::Combat => legal_main(db, state),
        Phase::End | Phase::Terminal => Vec::new(),
    }
}

fn legal_choice(state: &State) -> Vec<Action> {
    let Phase::Choice { node, .. } = &state.phase else {
        return Vec::new();
    };
    let raw: Vec<Action> = match node {
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
            // Unpicked partners only — `{card}` cannot mean both "select another
            // copy" and "deselect". Confirm commits; a second Choose of the
            // same id takes the next unpicked copy (E30, lowest-position).
            let mut v: Vec<Action> = options
                .iter()
                .enumerate()
                .filter(|(_, pos)| !picked.contains(pos))
                .map(|(i, _)| Action::Choose(i as u8))
                .collect();
            if !picked.is_empty() {
                v.push(Action::Confirm);
            }
            v
        }
    };
    // Trace convention: `legal` is a set; `choose {card}` collapses copies of
    // one id to the lowest position (docs/trace-format.md). Without this,
    // a policy can pick a later copy, emit `{card: id}`, and replay takes
    // the first copy — self-consistency then diverges on hand order.
    let mut out = Vec::new();
    let mut seen = Vec::new();
    for a in raw {
        let n = to_neutral(state, &a);
        if !seen.contains(&n) {
            seen.push(n);
            out.push(a);
        }
    }
    out
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
    // Kind-memory is the Artifact α rule (β/γ `requires` on recipes).
    // Sephie / Scholar print "Fuse: Cards" — any card, including a second
    // copy of a kind already fused this match (owner 2026-08-29).
    let remember_kinds = fuse.recipes.as_ref().is_some_and(|rs| {
        rs.iter()
            .any(|r| r.requires.as_ref().is_some_and(|req| !req.is_empty()))
    });
    hand.iter()
        .enumerate()
        .filter(|(i, c)| {
            *i != host
                && filter_matches_card(state, me, c, &fuse.partners)
                && !(remember_kinds && already_fused_kind(host_inst, c))
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
        // Earth Sigil amulet is an amulet play like any other (2026-09-10).
        // Merge only ever runs on a non-full field.
        if state.player(me).field_free() == 0 {
            return false;
        }
        return true;
    }
    // spell / accelerate: mandatory Select must have targets
    // owner-rulings 2026-08-16 / official Q&A 2026-09-06
    spell_playable(db, state, me, hand_i, card, &effects)
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
    let crystal = card.modes().iter().find_map(|m| match m {
        Mode::Crystallize { cost, .. } if pp < effective && pp >= *cost => Some(*cost),
        _ => None,
    });
    if let Some(cost) = crystal {
        return Some((cost, CardKind::Amulet, Vec::new()));
    }
    if pp >= effective {
        Some((effective, card.kind(), fanfare_effects(card)))
    } else {
        None
    }
}

fn apply_crystallize_form(card: &Card, inst: &mut CardInstance, paid: i32) {
    inst.kind = CardKind::Amulet;
    inst.base_cost = paid;
    inst.cost = paid;
    inst.attack = 0;
    inst.defense = 0;
    inst.max_defense = 0;
    inst.traits = Traits::default();
    if let Some(Mode::Crystallize {
        countdown,
        abilities,
        ..
    }) = card
        .modes()
        .iter()
        .find(|m| matches!(m, Mode::Crystallize { .. }))
    {
        inst.countdown = *countdown;
        inst.printed_tags = abilities
            .as_ref()
            .map(|abs| abs.iter().map(|a| a.snapshot_tag().to_string()).collect())
            .unwrap_or_default();
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
        | Effect::Select { select, .. }
        | Effect::Destroy { select, .. }
        | Effect::Banish { select, .. }
        | Effect::ReturnToHand { select, .. }
        | Effect::ReturnToDeck { select, .. }
        | Effect::Discard { select, .. }
        | Effect::Evolve { select, .. }
        | Effect::GrantTraits { select, .. }
        | Effect::RemoveTraits { select, .. }
        | Effect::Countdown { select, .. }
        | Effect::Transform { select, .. } => choose_select_ok(db, state, me, hand_i, select),
        Effect::AddToHand { card, .. } => {
            if let Some(p) = card_source_choose_pool(card) {
                choose_select_ok(db, state, me, hand_i, &Selector::Pool(p.clone()))
            } else {
                true
            }
        }
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
            let who = state.active;
            apply_evolve_action(db, state, who, slot, super_evolve, false, &mut events)?;
        }
        Action::Engage { slot } => apply_engage(db, state, slot, &mut events)?,
        Action::Fuse { host } => apply_fuse_start(db, state, host, &mut events)?,
        Action::BonusPp => apply_bonus(state, &mut events)?,
        Action::Choose(i) => apply_choose(db, state, i, &mut events)?,
        Action::Confirm => apply_confirm(db, state, &mut events)?,
        Action::EndTurn => apply_end_turn(db, state, &mut events)?,
    }
    drain_until_quiet(db, state, &mut events)?;
    // Bindings are per-resolution. Choice is the only pause mid-list;
    // Mulligan / Main / Combat / End / Terminal have finished the action.
    if !matches!(state.phase, Phase::Choice { .. }) {
        state.bindings.clear();
    }
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
        let _ = draw_one(state, player, None, &[])?;
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
    let mut inst = state.player_mut(me).hand.remove(hand as usize);
    let form = if kind == CardKind::Amulet && card.kind() != CardKind::Amulet {
        PlayForm::Crystallize { paid }
    } else if kind == CardKind::Spell && card.kind() != CardKind::Spell {
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
    if matches!(form, PlayForm::Enhance { .. }) {
        inst.flags.enhanced = true;
    }
    if matches!(form, PlayForm::Crystallize { .. }) {
        apply_crystallize_form(card, &mut inst, paid);
    }
    // Combo counts the played card — official Q&A May
    state.player_mut(me).combo += 1;
    state.player_mut(me).played_this_turn.push(inst.card);
    let base_for_ladder = if matches!(
        form,
        PlayForm::Accelerate { .. } | PlayForm::Crystallize { .. }
    ) {
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
    // E39: play reactions (`ally_card_played` / `ally_spell_played` /
    // "whenever you play …") enqueue now and resolve before Fanfare / spell
    // text. Enter reactions stay on the queue until the play sequence ends
    // (E34). Official Q&A World of Games / Divine Thunder; owner 2026-09-10.
    raise_when(db, state, me, EventName::AllyCardPlayed, Some(&inst), me);
    if kind == CardKind::Spell {
        raise_when(db, state, me, EventName::AllySpellPlayed, Some(&inst), me);
    }
    let play_rx = std::mem::take(&mut state.queue);
    let played_id = inst.id;
    if kind == CardKind::Spell {
        let src = SourceRef::Spell {
            player: me,
            card: inst.card,
        };
        // Spellboost the hand (Accelerate counts as a spell) — ruling 2026-09-02
        spellboost_hand(db, state, me, events)?;
        // Corpse before spell text so `{var: X}` / costEq see the boosted instance
        // (Stormy Blast, Amethyst's Naptime, Beheading Eld Blades).
        let mut corpse = inst;
        corpse.kind = CardKind::Spell;
        corpse.base_cost = base_for_ladder;
        corpse.cost = paid;
        state.player_mut(me).shadows += 1;
        state.player_mut(me).cemetery.push(corpse);
        push_effects(state, me, src, effects);
    } else {
        enter_from_play(db, state, me, inst, effects, events)?;
        // AllyCardPlayed is raised before enter, so `subject_target` is None
        // and `raise_when` restores `event_subject`. Play reactions (Yuel
        // `pick: entering`) resolve after the body is on the field.
        if let Some(slot) = state.find_field(me, played_id) {
            state.event_subject = Some(TargetOpt::Slot { player: me, slot });
            state.event_inst_id = Some(played_id);
        }
    }
    flush_play_reactions_ahead(state, play_rx);
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
        state.player_mut(me).earth += 1;
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
    // E39: play reactions (`whenever you play`) are flushed onto pending_work
    // above Fanfare in `apply_play`. E34: other cards' enter reactions stay on
    // the queue until the play completes. E38: the entrant's own `on:enter` is
    // one of those reactions (board age, oldest first), not a jump ahead of them.
    let gated = gated_fanfare(db, state, me, src, card_id, fanfare);
    push_effects(state, me, src, gated);
    queue_enter_reactions(db, state, me, card_id, id, true);
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

/// Official glossary / owner 2026-09-10: when an Earth Sigil amulet enters,
/// every other allied Earth Sigil is **banished** (no shadow, no Last Words)
/// and the new amulet takes their counts. The caller still places `incoming`.
/// Returns `false` so the place path runs. "Gain X earth sigils" does not
/// go through here — that increments the holder or summons one Sediment.
fn merge_earth(
    _db: &CardDb,
    state: &mut State,
    me: PlayerId,
    _incoming: &CardInstance,
    events: &mut Vec<Event>,
) -> Result<bool, Illegal> {
    let holders: Vec<u32> = state
        .player(me)
        .field
        .iter()
        .flatten()
        .filter(|c| c.is_earth_sigil())
        .map(|c| c.id)
        .collect();
    if holders.is_empty() {
        return Ok(false);
    }
    let stack = state.player(me).earth;
    for id in holders {
        let Some(slot) = state.find_field(me, id) else {
            continue;
        };
        if let Some(inst) = state.player_mut(me).field[slot as usize].take() {
            events.push(Event::Banish {
                card: inst.card,
                from: ZoneLabel::Field,
            });
            state.player_mut(me).banished.push(inst);
            state.player_mut(me).compact_field();
        }
    }
    state.player_mut(me).earth = stack;
    state.player_mut(me).earth_slot = None;
    Ok(false)
}

fn spellboost_hand(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    events: &mut [Event],
) -> Result<(), Illegal> {
    let ids: Vec<u32> = state.player(me).hand.iter().map(|c| c.id).collect();
    for hid in ids {
        spellboost_instance(db, state, me, hid, events)?;
    }
    Ok(())
}

fn spellboost_instance(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    hid: u32,
    _events: &mut [Event],
) -> Result<(), Illegal> {
    let Some(pos) = state.player(me).hand.iter().position(|c| c.id == hid) else {
        return Ok(());
    };
    let card_id = state.player(me).hand[pos].card;
    let Ok(card) = db.card(card_id) else {
        return Ok(());
    };
    if let Some(h) = state.player_mut(me).hand.get_mut(pos) {
        h.spellboost_count += 1;
    }
    let fx: Vec<Effect> = card
        .abilities()
        .iter()
        .filter(|a| matches!(a, Ability::Spellboost { .. }))
        .flat_map(|a| a.effects().iter().cloned())
        .collect();
    if fx.is_empty() {
        return Ok(());
    }
    let src = SourceRef::Hand {
        player: me,
        id: hid,
    };
    push_effects(state, me, src, fx);
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
    state.attack_target_is_leader = matches!(target, AttackTarget::Leader);
    state.attacking_follower = matches!(target, AttackTarget::Slot(_));
    // Strike `if {attackingFollower}` (Giada / Verdilia) and
    // Follower Strike `pick: opposing` (Okita) read this while those
    // triggers resolve, before combat damage.
    state.combat_opposing = Some(match target {
        AttackTarget::Slot(ds) => TargetOpt::Slot {
            player: me.opponent(),
            slot: ds.0,
        },
        AttackTarget::Leader => TargetOpt::Leader {
            player: me.opponent(),
        },
    });
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
    // Keep attack-target flags through Strike (Giada `op: if`, Lu Woh
    // `attackingLeader`) until AfterCombat. Verdilia already evaluated
    // `attackingFollower` on AllyFollowerAttacks.
    // Strike / Clash before damage — rulebook Combat Timing
    queue_combat_triggers(db, state, me, attacker.0, target);
    let defender_id = match target {
        AttackTarget::Slot(ds) => state.field_inst(me.opponent(), ds.0).map(|c| c.id),
        AttackTarget::Leader => None,
    };
    state
        .pending_work
        .push(WorkFrame::Aftermath(Aftermath::AfterCombat {
            attacker_player: me,
            attacker_id: att.id,
            target,
            defender_id,
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
    who: PlayerId,
    slot: Slot,
    supered: bool,
    granted: bool,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let Some(inst) = state.field_inst_mut(who, slot.0) else {
        // Effect-evolve of a missing slot (compact after banish, or the
        // entering follower already left) is a skip, like an already-evolved
        // target. A player EP/SEP click of an empty slot is illegal.
        if granted {
            return Ok(());
        }
        return Err(Illegal::NotLegal);
    };
    // Glossary Evolution / owner 2026-09-10: an evolved follower can't be
    // evolved again (including EP then SEP, and an effect-evolve).
    // Effect-evolve of an already-evolved follower is skipped (Camiscilla /
    // Substandard). A player EP/SEP evolve of one is illegal.
    if inst.evolved {
        if granted {
            return Ok(());
        }
        return Err(Illegal::NotLegal);
    }
    if !granted {
        if supered {
            state.player_mut(who).sep -= 1;
        } else {
            state.player_mut(who).ep -= 1;
        }
        state.player_mut(who).evolved_this_turn = true;
    }
    // EP, SEP or effect — `docs/trace-format.md` evolves_used.
    state.player_mut(who).evolves_used += 1;
    let Some(inst) = state.field_inst_mut(who, slot.0) else {
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
    let src = SourceRef::Field { player: who, id };
    let when_ok = |a: &Ability| {
        a.when_cond()
            .map(|c| eval_cond(db, state, who, Some(src), c))
            .unwrap_or(true)
    };
    let replace = card.abilities().iter().any(|a| a.replaces_evolve());
    let mut fx = Vec::new();
    if supered && replace {
        for a in card.abilities() {
            if matches!(a, Ability::SuperEvolve { .. }) && when_ok(a) {
                fx.extend(a.effects().iter().cloned());
            }
        }
    } else {
        if !granted {
            for a in card.abilities() {
                if matches!(a, Ability::Evolve { .. }) && when_ok(a) {
                    fx.extend(a.effects().iter().cloned());
                }
            }
        }
        for a in card.abilities() {
            if matches!(a, Ability::AnyEvolve { .. }) && when_ok(a) {
                fx.extend(a.effects().iter().cloned());
            }
        }
        if supered {
            for a in card.abilities() {
                if matches!(
                    a,
                    Ability::SuperEvolve { .. } | Ability::AnySuperEvolve { .. }
                ) && when_ok(a)
                {
                    fx.extend(a.effects().iter().cloned());
                }
            }
        }
    }
    // Skybound Art gauge = current turn number + evolves/boosts stored on the
    // instance (rulebook). Evolves while a copy is in hand increment that
    // copy's stored bonus; turn number is added at evaluation so M1 snapshots
    // (no Skybound cards) stay at the omitted-zero the old traces emit.
    for h in &mut state.player_mut(who).hand {
        if card_tracks_skybound(db, h.card) {
            h.skybound += 1;
        }
    }
    // E37: push the evolving follower's Evolve/Super-Evolve list first so it
    // sits under the reaction wave (pending_work is LIFO). Then raise
    // when-triggers and flush them on top — Faith/crests and other cards'
    // `when ally_evolve` drain before the Evolve: list starts. Reactions
    // raised *during* that list still wait (A2 / play-sequence E34).
    push_effects(state, who, src, fx);
    if let Some(evolved) = state.field_inst(who, slot.0).cloned() {
        raise_when(db, state, who, EventName::AllyEvolve, Some(&evolved), who);
        if supered {
            raise_when(
                db,
                state,
                who,
                EventName::AllySuperEvolve,
                Some(&evolved),
                who,
            );
        }
    }
    drain_queue(db, state, events)?;
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
                picked: vec![],
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
            let idx = lowest_unpicked_fuse_index(state, player, &options, &picked, i as usize);
            let Some(&pos) = options.get(idx) else {
                return Err(Illegal::NotLegal);
            };
            if picked.contains(&pos) {
                return Err(Illegal::NotLegal);
            }
            picked.push(pos);
            state.phase = Phase::Choice {
                player,
                node: ChoiceNode::FusePartners {
                    host,
                    options,
                    picked,
                },
            };
        }
        ChoiceNode::Modes {
            options,
            pending,
            mut picked,
        } => {
            let idx = options.get(i as usize).copied().ok_or(Illegal::NotLegal)?;
            if picked.contains(&idx) {
                return Err(Illegal::NotLegal);
            }
            picked.push(idx);
            let left = pending.remaining.saturating_sub(1);
            if left > 0 {
                let remain: Vec<u8> = options
                    .iter()
                    .copied()
                    .filter(|o| !picked.contains(o))
                    .collect();
                state.phase = Phase::Choice {
                    player,
                    node: ChoiceNode::Modes {
                        options: remain,
                        pending: PendingChoice {
                            kind: pending.kind,
                            remaining: left,
                        },
                        picked,
                    },
                };
            } else {
                state.phase = Phase::Main;
                picked.sort_unstable();
                resume_modes(db, state, player, &picked, pending, events)?;
            }
        }
        ChoiceNode::Targets { options, pending } => {
            // `choose {card}`: by content, lowest-position copy (E30). A
            // later duplicate index is the same card; snap to the first so
            // arena-trace → arena-replay keeps hand order.
            let idx = lowest_hand_copy_index(state, &options, i as usize);
            let opt = options.get(idx).cloned().ok_or(Illegal::NotLegal)?;
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

/// `{card: id}` identify by content, lowest-position copy (E30). A live
/// Choose(n) that points at a later copy of the same card is snapped to
/// that copy so `arena-replay` of the recorded `{card}` agrees with apply.
fn lowest_hand_copy_index(state: &State, options: &[TargetOpt], i: usize) -> usize {
    let Some(TargetOpt::Hand { player, pos }) = options.get(i) else {
        return i;
    };
    let Some(card) = state
        .player(*player)
        .hand
        .get(*pos as usize)
        .map(|c| c.card)
    else {
        return i;
    };
    options
        .iter()
        .position(|o| match o {
            TargetOpt::Hand {
                player: p,
                pos: pos2,
            } if *p == *player => state
                .player(*player)
                .hand
                .get(*pos2 as usize)
                .is_some_and(|c| c.card == card),
            _ => false,
        })
        .unwrap_or(i)
}

/// Fuse partners: a second `{card}` of the same id takes the next unpicked
/// copy so two Missiles (or two Ticos) can be fused together.
fn lowest_unpicked_fuse_index(
    state: &State,
    player: PlayerId,
    options: &[u8],
    picked: &[u8],
    i: usize,
) -> usize {
    let Some(&pos) = options.get(i) else {
        return i;
    };
    let Some(card) = state.player(player).hand.get(pos as usize).map(|c| c.card) else {
        return i;
    };
    options
        .iter()
        .position(|&p| {
            !picked.contains(&p)
                && state
                    .player(player)
                    .hand
                    .get(p as usize)
                    .is_some_and(|c| c.card == card)
        })
        .unwrap_or(i)
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
    // "Whenever you Fuse to this card" fires on the host that was fused to
    // (Sephie 10934110: spend 2 PP, summon Obsessed Test Subject). The
    // abilities are those printed on the pre-transform host; the instance
    // id is kept across a recipe transform.
    let host_id = state.player(me).hand[host_pos].id;
    let src = SourceRef::Hand {
        player: me,
        id: host_id,
    };
    if let Ok(card) = db.card(host_card) {
        let mut fx = Vec::new();
        for a in card.abilities() {
            if !matches!(a, Ability::Fused { .. }) {
                continue;
            }
            if let Some(cond) = a.when_cond() {
                if !eval_cond(db, state, me, Some(src), cond) {
                    continue;
                }
            }
            fx.extend(a.effects().iter().cloned());
        }
        push_effects(state, me, src, fx);
    }
    Ok(())
}

fn resume_modes(
    db: &CardDb,
    state: &mut State,
    _player: PlayerId,
    idxs: &[u8],
    _pending: PendingChoice,
    _events: &mut [Event],
) -> Result<(), Illegal> {
    if let Some(WorkFrame::Effects {
        controller,
        source,
        effects,
        index,
        subject,
        subject_id,
        e40,
    }) = state.pending_work.pop()
    {
        restore_event_subject(state, subject.clone(), subject_id);
        if let Some(Effect::Choose { .. }) = effects.get(index).cloned() {
            if let Some(opts) = resolve_choose_options(db, state, source, &effects[index]) {
                let mut rest = effects;
                rest.remove(index);
                push_work(state, controller, source, rest, index, subject.clone(), e40);
                // LIFO: push later-listed last so they resolve in listed order.
                for &idx in idxs.iter().rev() {
                    if let Some(opt) = opts.get(idx as usize) {
                        push_effects(state, controller, source, opt.effects.clone());
                    }
                }
            }
        } else {
            push_work(
                state,
                controller,
                source,
                effects,
                index,
                subject.clone(),
                e40,
            );
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
        subject,
        subject_id,
        e40,
    }) = state.pending_work.pop()
    {
        restore_event_subject(state, subject.clone(), subject_id);
        if index < effects.len() {
            let e = effects[index].clone();
            apply_effect_with_targets(db, state, controller, source, &e, &[opt], events)?;
            let left = pending.remaining.saturating_sub(1);
            if left > 0 {
                state.bind_append = true;
                if let Some(ChoiceNode::Targets { mut options, .. }) =
                    effect_choice_node(db, state, controller, source, &e)
                {
                    if let Some(name) = e.as_bind() {
                        if let Some(refs) = state.bindings.get(name) {
                            options.retain(|t| {
                                opt_to_bound(state, t).is_none_or(|r| !refs.contains(&r))
                            });
                        }
                    }
                    if !options.is_empty() {
                        push_work(
                            state,
                            controller,
                            source,
                            effects.clone(),
                            index,
                            subject.clone(),
                            e40,
                        );
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
            state.bind_append = false;
            if index + 1 < effects.len() {
                push_work(
                    state,
                    controller,
                    source,
                    effects,
                    index + 1,
                    state.event_subject.clone(),
                    e40,
                );
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
    // One Invoke copy per card id per boundary window (official glossary).
    if start {
        state.invoked_ids.clear();
    }
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

fn tick_crests(db: &CardDb, state: &mut State, who: PlayerId) {
    let mut doomed = Vec::new();
    for c in state.player_mut(who).crests.iter_mut() {
        if let Some(cd) = c.countdown.as_mut() {
            *cd -= 1;
            if *cd <= 0 {
                doomed.push(c.granted_order);
            }
        }
    }
    for order in doomed {
        let _ = expire_crest(db, state, who, order, &mut Vec::new());
    }
}

fn expire_crest(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    order: u32,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let crests = &mut state.player_mut(who).crests;
    let Some(i) = crests.iter().position(|c| c.granted_order == order) else {
        return Ok(());
    };
    let c = crests.remove(i);
    events.push(Event::CrestRemove {
        player: who,
        id: c.id.clone(),
    });
    if let Ok(def) = db.crest(&c.id) {
        let src = SourceRef::Crest {
            player: who,
            index: 0,
        };
        for a in def.abilities() {
            if matches!(a, Ability::LastWords { .. }) {
                push_effects(state, who, src, a.effects().to_vec());
            }
        }
    }
    Ok(())
}

fn tick_amulets(db: &CardDb, state: &mut State, who: PlayerId) {
    // Countdown advances once at the owner's start of turn — rulebook Countdown.
    // Capture by instance id: destroy_slot compacts, so a later slot index
    // would hit whoever moved in (E29 / same root as E14).
    let mut doomed = Vec::new();
    for c in state.player_mut(who).field.iter_mut().flatten() {
        if let Some(cd) = c.countdown.as_mut() {
            *cd -= 1;
            if *cd <= 0 {
                doomed.push(c.id);
            }
        }
    }
    for id in doomed {
        if let Some(slot) = state.find_field(who, id) {
            let _ = destroy_slot(db, state, who, slot, false, &mut Vec::new());
        }
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
    // RF: look up abilities by index (no Ability clones). Wave-1: also scan
    // hand/deck with AbilityZone so zone:hand endOfTurn fires, and evaluate
    // `when` at enqueue (Garodeth leaderDefenseLte).
    if crests {
        let mut pending: Vec<(u32, u8, usize, String)> = Vec::new();
        for (i, c) in state.player(who).crests.iter().enumerate() {
            let Ok(def) = db.crest(&c.id) else { continue };
            for (ord, a) in def.abilities().iter().enumerate() {
                if ability_boundary(a, whose, start, AbilityZone::Field) {
                    pending.push((c.granted_order, ord as u8, i, c.id.clone()));
                }
            }
        }
        for (entry, printed, index, id) in pending {
            let Ok(def) = db.crest(&id) else { continue };
            let Some(a) = def.abilities().get(printed as usize) else {
                continue;
            };
            let source = SourceRef::Crest { player: who, index };
            if !boundary_when_ok(db, state, who, source, a) {
                continue;
            }
            enqueue(state, cat, entry, printed, who, source, a);
        }
        return;
    }

    let mut field: Vec<(u32, u8, CardId, SourceRef)> = Vec::new();
    for (si, s) in state.player(who).field.iter().enumerate() {
        let Some(inst) = s else { continue };
        let Ok(card) = db.card(inst.card) else {
            continue;
        };
        for (ord, a) in card.abilities().iter().enumerate() {
            if ability_boundary(a, whose, start, AbilityZone::Field) {
                field.push((
                    si as u32,
                    ord as u8,
                    inst.card,
                    SourceRef::Field {
                        player: who,
                        id: inst.id,
                    },
                ));
            }
        }
    }
    enqueue_boundary_lookups(db, state, who, cat, field);

    if db.zone_has_boundary(AbilityZone::Hand, start) {
        let mut hand: Vec<(u32, u8, CardId, SourceRef)> = Vec::new();
        for (hi, h) in state.player(who).hand.iter().enumerate() {
            let Ok(card) = db.card(h.card) else { continue };
            for (ord, a) in card.abilities().iter().enumerate() {
                if ability_boundary(a, whose, start, AbilityZone::Hand) {
                    hand.push((
                        hi as u32,
                        ord as u8,
                        h.card,
                        SourceRef::Hand {
                            player: who,
                            id: h.id,
                        },
                    ));
                }
            }
        }
        enqueue_boundary_lookups(db, state, who, cat, hand);
    }

    if db.zone_has_boundary(AbilityZone::Deck, start) {
        let mut deck: Vec<(u32, u8, CardId, SourceRef)> = Vec::new();
        for (di, d) in state.player(who).deck.iter().enumerate() {
            let Ok(card) = db.card(d.card) else { continue };
            for (ord, a) in card.abilities().iter().enumerate() {
                if ability_boundary(a, whose, start, AbilityZone::Deck) {
                    deck.push((
                        di as u32,
                        ord as u8,
                        d.card,
                        SourceRef::Hand {
                            player: who,
                            id: d.id,
                        },
                    ));
                }
            }
        }
        enqueue_boundary_lookups(db, state, who, cat, deck);
    }
}

fn boundary_when_ok(
    db: &CardDb,
    state: &State,
    who: PlayerId,
    source: SourceRef,
    a: &Ability,
) -> bool {
    match a.when_cond() {
        Some(cond) => eval_cond(db, state, who, Some(source), cond),
        None => true,
    }
}

fn enqueue_boundary_lookups(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    cat: u8,
    pending: Vec<(u32, u8, CardId, SourceRef)>,
) {
    for (entry, printed, card_id, source) in pending {
        let Ok(card) = db.card(card_id) else { continue };
        let Some(a) = card.abilities().get(printed as usize) else {
            continue;
        };
        if !boundary_when_ok(db, state, who, source, a) {
            continue;
        }
        enqueue(state, cat, entry, printed, who, source, a);
    }
}

fn ability_boundary(a: &Ability, whose: Whose, start: bool, zone: AbilityZone) -> bool {
    if a.zone() != zone {
        return false;
    }
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
        subject: state.event_subject.clone(),
        subject_id: state.event_inst_id,
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
    let n_printed = card.abilities().len();
    for (ord, a) in card
        .abilities()
        .iter()
        .chain(inst.granted.iter())
        .enumerate()
    {
        if a.tag() == tag {
            if ord < n_printed && !inst.printed_tags.contains(a.snapshot_tag()) {
                continue;
            }
            if let Some(cond) = a.when_cond() {
                if !eval_cond(
                    db,
                    state,
                    who,
                    Some(SourceRef::Field {
                        player: who,
                        id: inst.id,
                    }),
                    cond,
                ) {
                    continue;
                }
            }
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
    _card: CardId,
    inst_id: u32,
    _from_play: bool,
) {
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

/// Re-resolve a stored field `subject` after `compact_field`. Slot indexes
/// move; instance ids do not (Camiscilla `pick: entering` after Bahamut
/// banishes the rest of the board).
fn live_field_subject(
    state: &State,
    subject: Option<TargetOpt>,
    subject_id: Option<u32>,
) -> Option<TargetOpt> {
    if let Some(id) = subject_id {
        let hinted = match &subject {
            Some(TargetOpt::Slot { player, .. }) => Some(*player),
            _ => None,
        };
        let order = match hinted {
            Some(p) => [p, p.opponent()],
            None => PlayerId::ALL,
        };
        for p in order {
            if let Some(slot) = state.find_field(p, id) {
                return Some(TargetOpt::Slot { player: p, slot });
            }
        }
        return None;
    }
    subject
}

fn restore_event_subject(state: &mut State, subject: Option<TargetOpt>, subject_id: Option<u32>) {
    if subject.is_none() && subject_id.is_none() {
        return;
    }
    if subject_id.is_some() {
        state.event_inst_id = subject_id;
    }
    state.event_subject = live_field_subject(state, subject, subject_id);
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
    // Nested events (Earth Rite during an enter-triggered `pay`) must not
    // clobber the in-flight list's `pick: entering` (Emperor of Elements).
    let prev_subject = state.event_subject.clone();
    let prev_base = state.event_base_cost;
    let prev_inst = state.event_inst_id;
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
        state.event_subject = prev_subject;
        state.event_base_cost = prev_base;
        state.event_inst_id = prev_inst;
        return;
    }
    if let Some(inst) = subject {
        state.event_base_cost = Some(inst.base_cost);
        state.event_inst_id = Some(inst.id);
        state.event_subject = subject_target(state, subject_owner, inst);
    } else {
        state.event_base_cost = None;
        state.event_inst_id = None;
        state.event_subject = Some(TargetOpt::Leader {
            player: subject_owner,
        });
    }
    enqueue_when_on(db, state, observer_side, event, subject, None, false);
    state.event_subject = prev_subject;
    state.event_base_cost = prev_base;
    state.event_inst_id = prev_inst;
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

#[derive(Clone, Copy)]
struct WhenLoc {
    zone: AbilityZone,
    entry: u32,
    source: SourceRef,
    mark_id: Option<u32>,
    mark_crest: Option<usize>,
}

struct WhenCand {
    entry: u32,
    id: u32,
    card: CardId,
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
    let mut pending: Vec<PendingWhen> = Vec::new();
    let scan = WhenScan {
        db,
        observer: observer_side,
        event,
        subject,
    };

    let field_cands: Vec<WhenCand> = state
        .player(observer_side)
        .field
        .iter()
        .enumerate()
        .filter_map(|(si, s)| {
            let inst = s.as_ref()?;
            if let Some(id) = only_inst {
                if inst.id != id {
                    return None;
                }
            }
            if inst.granted_whens == 0 && !db.card_has_when(inst.card, event, AbilityZone::Field) {
                return None;
            }
            Some(WhenCand {
                entry: si as u32,
                id: inst.id,
                card: inst.card,
            })
        })
        .collect();

    for cand in &field_cands {
        if self_buff {
            state.event_subject = Some(TargetOpt::Slot {
                player: observer_side,
                slot: cand.entry as u8,
            });
        }
        let (granted, once_when, tags) = {
            let Some(inst) = state
                .player(observer_side)
                .field
                .get(cand.entry as usize)
                .and_then(|s| s.as_ref())
                .filter(|c| c.id == cand.id)
            else {
                continue;
            };
            let granted = if inst.granted_whens > 0 {
                inst.granted.clone()
            } else {
                Vec::new()
            };
            (
                granted,
                inst.once_used.contains(&TriggerTag::When),
                inst.printed_tags.clone(),
            )
        };
        collect_when_from_abilities(
            &scan,
            state,
            WhenLoc {
                zone: AbilityZone::Field,
                entry: cand.entry,
                source: SourceRef::Field {
                    player: observer_side,
                    id: cand.id,
                },
                mark_id: Some(cand.id),
                mark_crest: None,
            },
            cand.card,
            &granted,
            once_when,
            Some(&tags),
            &mut pending,
        );
    }

    if only_inst.is_none() {
        let crest_cands: Vec<(usize, String, bool)> = state
            .player(observer_side)
            .crests
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                db.crest_has_when(&c.id, event, AbilityZone::Field) || !c.granted.is_empty()
            })
            .map(|(i, c)| (i, c.id.clone(), !c.granted.is_empty()))
            .collect();
        for (i, id, has_granted) in crest_cands {
            let once_when = match state.player(observer_side).crests.get(i) {
                Some(c) if c.id == id => c.once_used.contains(&TriggerTag::When),
                _ => continue,
            };
            let Ok(def) = db.crest(&id) else { continue };
            collect_when_from_list(
                &scan,
                state,
                def.abilities(),
                0,
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
                once_when,
                None,
                &mut pending,
            );
            if has_granted {
                let granted = state
                    .player(observer_side)
                    .crests
                    .get(i)
                    .map(|c| c.granted.clone())
                    .unwrap_or_default();
                collect_when_from_list(
                    &scan,
                    state,
                    &granted,
                    def.abilities().len(),
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
                    once_when,
                    None,
                    &mut pending,
                );
            }
        }

        if db.zone_has_when(event, AbilityZone::Hand) {
            let hand_cands: Vec<WhenCand> = state
                .player(observer_side)
                .hand
                .iter()
                .enumerate()
                .filter(|(_, h)| {
                    if event == EventName::AllyDraw {
                        if let Some(subj) = subject {
                            if h.id != subj.id {
                                return false;
                            }
                        }
                    }
                    h.granted_whens > 0 || db.card_has_when(h.card, event, AbilityZone::Hand)
                })
                .map(|(hi, h)| WhenCand {
                    entry: hi as u32,
                    id: h.id,
                    card: h.card,
                })
                .collect();
            for cand in hand_cands {
                let (granted, once_when, tags) = {
                    let Some(h) = state
                        .player(observer_side)
                        .hand
                        .iter()
                        .find(|c| c.id == cand.id)
                    else {
                        continue;
                    };
                    let granted = if h.granted_whens > 0 {
                        h.granted.clone()
                    } else {
                        Vec::new()
                    };
                    (
                        granted,
                        h.once_used.contains(&TriggerTag::When),
                        h.printed_tags.clone(),
                    )
                };
                collect_when_from_abilities(
                    &scan,
                    state,
                    WhenLoc {
                        zone: AbilityZone::Hand,
                        entry: cand.entry,
                        source: SourceRef::Hand {
                            player: observer_side,
                            id: cand.id,
                        },
                        mark_id: Some(cand.id),
                        mark_crest: None,
                    },
                    cand.card,
                    &granted,
                    once_when,
                    Some(&tags),
                    &mut pending,
                );
            }
        }

        if db.zone_has_when(event, AbilityZone::Deck) {
            let deck_cands: Vec<WhenCand> = state
                .player(observer_side)
                .deck
                .iter()
                .enumerate()
                .filter(|(_, d)| {
                    d.granted_whens > 0 || db.card_has_when(d.card, event, AbilityZone::Deck)
                })
                .map(|(di, d)| WhenCand {
                    entry: di as u32,
                    id: d.id,
                    card: d.card,
                })
                .collect();
            for cand in deck_cands {
                let (granted, once_when, tags) = {
                    let Some(d) = state
                        .player(observer_side)
                        .deck
                        .iter()
                        .find(|c| c.id == cand.id)
                    else {
                        continue;
                    };
                    let granted = if d.granted_whens > 0 {
                        d.granted.clone()
                    } else {
                        Vec::new()
                    };
                    (
                        granted,
                        d.once_used.contains(&TriggerTag::When),
                        d.printed_tags.clone(),
                    )
                };
                collect_when_from_abilities(
                    &scan,
                    state,
                    WhenLoc {
                        zone: AbilityZone::Deck,
                        entry: cand.entry,
                        source: SourceRef::Hand {
                            player: observer_side,
                            id: cand.id,
                        },
                        mark_id: Some(cand.id),
                        mark_crest: None,
                    },
                    cand.card,
                    &granted,
                    once_when,
                    Some(&tags),
                    &mut pending,
                );
            }
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
        // Crests before board at the same timing (rulebook turn boundaries
        // step 2 vs 3; E37 evolve). Active then opponent.
        let cat = match source {
            SourceRef::Crest { .. } => {
                if observer_side == state.active {
                    3
                } else {
                    5
                }
            }
            _ => {
                if observer_side == state.active {
                    4
                } else {
                    6
                }
            }
        };
        enqueue(state, cat, entry, printed, observer_side, source, &a);
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_when_from_abilities(
    scan: &WhenScan<'_>,
    state: &State,
    loc: WhenLoc,
    card_id: CardId,
    granted: &[Ability],
    once_when: bool,
    printed_tags: Option<&std::collections::BTreeSet<String>>,
    pending: &mut Vec<PendingWhen>,
) {
    let printed = scan.db.card(card_id).map(Card::abilities).unwrap_or(&[]);
    collect_when_from_list(
        scan,
        state,
        printed,
        0,
        loc,
        once_when,
        printed_tags,
        pending,
    );
    if !granted.is_empty() {
        collect_when_from_list(
            scan,
            state,
            granted,
            printed.len(),
            loc,
            once_when,
            None,
            pending,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_when_from_list(
    scan: &WhenScan<'_>,
    state: &State,
    abilities: &[Ability],
    order_base: usize,
    loc: WhenLoc,
    once_when: bool,
    printed_tags: Option<&std::collections::BTreeSet<String>>,
    pending: &mut Vec<PendingWhen>,
) {
    for (ord, a) in abilities.iter().enumerate() {
        let Ability::When {
            event: ev, filter, ..
        } = a
        else {
            continue;
        };
        if let Some(tags) = printed_tags {
            if !tags.contains(a.snapshot_tag()) {
                continue;
            }
        }
        if *ev != scan.event {
            continue;
        }
        // "Whenever another allied follower enters" — the observer is not
        // "another" relative to its own enter (Adahime / Gildaria).
        if scan.event == EventName::AllyFollowerEnter {
            if let (Some(subj), SourceRef::Field { id, .. }) = (scan.subject, loc.source) {
                if subj.id == id {
                    continue;
                }
            }
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
        if a.once_per_turn() && once_when {
            continue;
        }
        pending.push(PendingWhen {
            entry: loc.entry,
            printed: (order_base + ord) as u8,
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
        // Queue first when the next frame is a later op of an in-flight list
        // (E28: ally_draw after `draw count: N` before the next op). Nested
        // bodies and freshly flushed trigger waves sit as index-0 frames;
        // leave the queue behind those so a later-raised Last Words cannot
        // jump a trigger already in the wave (E32 / Grimnir) and so other
        // cards' enter reactions wait for Fanfare, including a Fanfare
        // choice (E34 / A2). Play reactions are flushed onto pending_work
        // before Fanfare (E39) and do not wait here.
        if !state.queue.is_empty()
            && !next_frame_is_choice_continuation(db, state)
            && next_frame_allows_queue_drain(state)
        {
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
                    subject,
                    subject_id,
                    e40,
                } => {
                    if e40 && index == 0 && !trigger_source_still_present(state, source) {
                        continue;
                    }
                    restore_event_subject(state, subject, subject_id);
                    resolve_effect_list(
                        db, state, controller, source, effects, index, e40, events,
                    )?;
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
            defender_id,
            knockback: _,
        } => {
            combat_damage(
                db,
                state,
                attacker_player,
                attacker_id,
                target,
                defender_id,
                events,
            )?;
            state.attacking_follower = false;
            state.attack_target_is_leader = false;
            state.combat_opposing = None;
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
            if let Some(id) = draw_one(state, who, None, &[])? {
                events.push(Event::Draw {
                    player: who,
                    card: id,
                });
                note_draw(db, state, who, id);
            }
        }
        Aftermath::DrainQueue => drain_queue(db, state, events)?,
        Aftermath::FlushPlayLastWords => flush_play_last_words(db, state, events)?,
        Aftermath::RestoreBindings(map) => {
            state.bindings = map;
        }
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
        for slot in state.player_mut(p).field.iter_mut().flatten() {
            expire_temp_traits(slot, until, whose_turn_ended);
        }
        for h in &mut state.player_mut(p).hand {
            expire_temp_traits(h, until, whose_turn_ended);
        }
    }
}

fn expire_temp_traits(inst: &mut CardInstance, until: Until, whose_turn_ended: PlayerId) {
    let mut i = 0;
    while i < inst.temp_traits.len() {
        let g = &inst.temp_traits[i];
        let drop = match g.until {
            Until::EndOfTurn => until == Until::EndOfTurn && whose_turn_ended == g.caster,
            Until::EndOfOpponentTurn => {
                until == Until::EndOfTurn && whose_turn_ended == g.caster.opponent()
            }
        };
        if drop {
            let g = inst.temp_traits.remove(i);
            inst.traits.merge_remove(&g.traits);
            if g.traits.ambush == Some(true) {
                inst.flags.ambush_active = inst.traits.ambush == Some(true);
            }
            if g.traits.attacks_per_turn.is_some() {
                inst.flags.attacks_left = inst.flags.attacks_left.min(1);
            }
        } else {
            i += 1;
        }
    }
}

/// True when the next pending frame is a later op of an in-flight list that
/// will immediately offer a player choice. The reactive queue then waits
/// (owner 2026-09-10 / E19).
fn next_frame_is_choice_continuation(db: &CardDb, state: &State) -> bool {
    let Some(WorkFrame::Effects {
        controller,
        source,
        effects,
        index,
        ..
    }) = state.pending_work.last()
    else {
        return false;
    };
    if *index == 0 || *index >= effects.len() {
        return false;
    }
    effect_choice_node(db, state, *controller, *source, &effects[*index]).is_some()
}

fn next_frame_allows_queue_drain(state: &State) -> bool {
    // A flushed trigger wave sits above RestoreBindings. Newly raised items
    // (Last Words of a follower destroyed by queued item (1)) wait until
    // (2) and (3) — already on pending_work — finish (E32 / Grimnir).
    if trigger_wave_in_flight(state) {
        return false;
    }
    match state.pending_work.last() {
        None => true,
        Some(WorkFrame::Effects { index, .. }) => *index > 0,
        Some(WorkFrame::Aftermath(Aftermath::RestoreBindings(_))) => false,
        Some(WorkFrame::Aftermath(Aftermath::FlushPlayLastWords)) => false,
        Some(WorkFrame::Aftermath(Aftermath::AfterCombat { .. })) => true,
        Some(WorkFrame::Aftermath(_)) => true,
    }
}

fn trigger_wave_in_flight(state: &State) -> bool {
    state
        .pending_work
        .iter()
        .any(|f| matches!(f, WorkFrame::Aftermath(Aftermath::RestoreBindings(_))))
}

/// E39: flush `whenever you play` reactions onto `pending_work` above the
/// Fanfare / spell body (crest-then-board). Last Words those reactions
/// cause (World of Games dying on play) then run via `FlushPlayLastWords`
/// still before Fanfare; enter reactions stay queued (E34).
fn flush_play_reactions_ahead(state: &mut State, mut play_rx: Vec<QueuedTrigger>) {
    if play_rx.is_empty() {
        return;
    }
    state
        .pending_work
        .push(WorkFrame::Aftermath(Aftermath::FlushPlayLastWords));
    play_rx.sort_by_key(|t| (t.category, t.entry, t.printed_order));
    let saved = std::mem::take(&mut state.bindings);
    state
        .pending_work
        .push(WorkFrame::Aftermath(Aftermath::RestoreBindings(saved)));
    for t in play_rx.into_iter().rev() {
        restore_event_subject(state, t.subject.clone(), t.subject_id);
        push_work(
            state,
            t.controller,
            t.source,
            t.effects,
            0,
            t.subject,
            e40_applies(t.tag),
        );
    }
}

fn flush_play_last_words(
    _db: &CardDb,
    state: &mut State,
    _events: &mut [Event],
) -> Result<(), Illegal> {
    let mut last_words = Vec::new();
    let mut rest = Vec::new();
    for t in std::mem::take(&mut state.queue) {
        if t.tag == TriggerTag::LastWords {
            last_words.push(t);
        } else {
            rest.push(t);
        }
    }
    state.queue = rest;
    if last_words.is_empty() {
        return Ok(());
    }
    last_words.sort_by_key(|t| (t.category, t.entry, t.printed_order));
    let saved = std::mem::take(&mut state.bindings);
    state
        .pending_work
        .push(WorkFrame::Aftermath(Aftermath::RestoreBindings(saved)));
    for t in last_words.into_iter().rev() {
        restore_event_subject(state, t.subject.clone(), t.subject_id);
        push_work(
            state,
            t.controller,
            t.source,
            t.effects,
            0,
            t.subject,
            e40_applies(t.tag),
        );
    }
    Ok(())
}

fn drain_queue(db: &CardDb, state: &mut State, _events: &mut [Event]) -> Result<(), Illegal> {
    if state.queue.is_empty() {
        return Ok(());
    }
    // Flush the whole current wave. One-at-a-time onto a LIFO stack reversed
    // the wave (B's Last Words ran before A's). Rulebook: active side first,
    // entry order within a side (E27).
    state
        .queue
        .sort_by_key(|t| (t.category, t.entry, t.printed_order));
    let batch = std::mem::take(&mut state.queue);
    // Isolate trigger binds without dropping the enclosing list's `as`
    // names: restore after this wave (Lieutenant Last Words summon-then-buff).
    let saved = std::mem::take(&mut state.bindings);
    state
        .pending_work
        .push(WorkFrame::Aftermath(Aftermath::RestoreBindings(saved)));
    for t in batch.into_iter().rev() {
        restore_event_subject(state, t.subject.clone(), t.subject_id);
        push_work(
            state,
            t.controller,
            t.source,
            t.effects,
            0,
            t.subject,
            e40_applies(t.tag),
        );
    }
    let _ = db;
    Ok(())
}

fn e40_applies(tag: TriggerTag) -> bool {
    !matches!(
        tag,
        TriggerTag::LastWords
            | TriggerTag::Leave
            | TriggerTag::Strike
            | TriggerTag::FollowerStrike
            | TriggerTag::Clash
    )
}

fn trigger_source_still_present(state: &State, source: SourceRef) -> bool {
    match source {
        SourceRef::Field { player, id } => state.find_field(player, id).is_some(),
        SourceRef::Hand { player, id } => {
            // Deck-zone boundary abilities are stored as `SourceRef::Hand`
            // (Sandalphon Invoke). Treat the instance as present if it is
            // still in hand or still in deck.
            let p = state.player(player);
            p.hand.iter().any(|c| c.id == id) || p.deck.iter().any(|c| c.id == id)
        }
        SourceRef::Crest { player, index } => state.player(player).crests.get(index).is_some(),
        SourceRef::Spell { .. } | SourceRef::Leader { .. } => true,
    }
}

fn push_work(
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    effects: Vec<Effect>,
    index: usize,
    subject: Option<TargetOpt>,
    e40: bool,
) {
    state.pending_work.push(WorkFrame::Effects {
        controller,
        source,
        effects,
        index,
        subject,
        subject_id: state.event_inst_id,
        e40,
    });
}

fn push_effects(state: &mut State, controller: PlayerId, source: SourceRef, effects: Vec<Effect>) {
    if effects.is_empty() {
        return;
    }
    push_work(state, controller, source, effects, 0, None, false);
}

#[allow(clippy::too_many_arguments)]
fn resolve_effect_list(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    effects: Vec<Effect>,
    index: usize,
    e40: bool,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    if index >= effects.len() {
        return Ok(());
    }
    let e = effects[index].clone();
    if let Some(cond) = e.when_cond() {
        if !eval_cond(db, state, controller, Some(source), cond) {
            if index + 1 < effects.len() {
                push_work(
                    state,
                    controller,
                    source,
                    effects,
                    index + 1,
                    state.event_subject.clone(),
                    e40,
                );
            }
            return Ok(());
        }
    }
    // pause if this effect needs a player choose — do not drain the
    // reactive queue first (owner 2026-09-10); it waits until the list
    // completes (`docs/engine-internals.md`).
    if let Some(node) = effect_choice_node(db, state, controller, source, &e) {
        push_work(
            state,
            controller,
            source,
            effects,
            index,
            state.event_subject.clone(),
            e40,
        );
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
    // Push the rest of this list first so a nested body (`repeat` / `if` /
    // `seq` / `choose` options) pushed by `apply_effect` sits on top and
    // resolves completely before the next enclosing op (E25).
    if index + 1 < effects.len() {
        push_work(
            state,
            controller,
            source,
            effects,
            index + 1,
            state.event_subject.clone(),
            e40,
        );
    }
    apply_effect(db, state, controller, source, &e, events)?;
    Ok(())
}

fn source_instance_cost(state: &State, source: Option<SourceRef>) -> Option<i32> {
    match source {
        Some(SourceRef::Field { player, id }) => state
            .find_field(player, id)
            .and_then(|s| state.field_inst(player, s).map(|c| c.cost)),
        Some(SourceRef::Hand { player, id }) => state
            .player(player)
            .hand
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.cost),
        Some(SourceRef::Spell { player, card }) => state
            .player(player)
            .cemetery
            .iter()
            .rev()
            .find(|c| c.card == card)
            .map(|c| c.cost),
        _ => None,
    }
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
            pick,
            ..
        } => {
            if matches!(pick.as_pick(), ChoosePick::All) {
                None
            } else {
                resolve_choose_options(db, state, source, e).map(|opts| {
                    let n = match pick.as_pick() {
                        crate::card::ChoosePick::N(k) => (k.max(1) as u8).max(1),
                        crate::card::ChoosePick::All => 1,
                    };
                    ChoiceNode::Modes {
                        options: (0..opts.len() as u8).collect(),
                        pending: PendingChoice {
                            kind: PendingKind::ModeSelect,
                            remaining: n,
                        },
                        picked: vec![],
                    }
                })
            }
        }
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
        | Effect::GrantAbility { select, .. }
        | Effect::RemoveAbilities { select, .. }
        | Effect::Cost { select, .. }
        | Effect::Countdown { select, .. }
        | Effect::Transform { select, .. }
        | Effect::SpellboostHand {
            select: Some(select),
            ..
        } => {
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
        Effect::AddToHand { card, .. } | Effect::Summon { card, .. } => {
            if let Some(p) = card_source_choose_pool(card) {
                let opts = pool_target_opts(db, state, controller, source, p);
                if opts.is_empty() {
                    return None;
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
            maybe_bind(state, e, targets);
            apply_each_captured(state, targets, |st, t| {
                if let TargetOpt::Slot { player, slot } = t {
                    destroy_by_ability(db, st, *player, *slot, events)?;
                }
                Ok(())
            })?;
            return Ok(());
        }
        Effect::Banish { .. } => {
            // Bind while the instance is still on the field (Allure exact copy).
            maybe_bind(state, e, targets);
            let bound: Vec<TargetOpt> = targets
                .iter()
                .filter_map(|t| target_as_card(state, t))
                .collect();
            apply_each_captured(state, targets, |st, t| {
                banish_opt(st, t, events);
                Ok(())
            })?;
            maybe_bind(state, e, &bound);
            return Ok(());
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
            let bound: Vec<TargetOpt> = targets
                .iter()
                .filter_map(|t| target_as_card(state, t))
                .collect();
            apply_each_captured(state, targets, |st, t| {
                return_deck_opt(db, st, t);
                Ok(())
            })?;
            maybe_bind(state, e, &bound);
            return Ok(());
        }
        Effect::Discard { .. } => {
            bind_discard_as_cards(state, e, targets);
            for t in targets {
                discard_opt(db, state, t, events)?;
            }
            return Ok(());
        }
        Effect::Evolve { super_evolve, .. } => {
            for t in targets {
                if let TargetOpt::Slot { player, slot } = t {
                    if *player == controller {
                        apply_evolve_action(
                            db,
                            state,
                            *player,
                            Slot(*slot),
                            *super_evolve,
                            true,
                            events,
                        )?;
                    }
                }
            }
        }
        Effect::Transform { into, .. } => {
            apply_each_captured(state, targets, |st, t| {
                transform_opt(db, st, t, into, events)
            })?;
        }
        Effect::Summon {
            card,
            controller: ctrl,
            ..
        } => {
            let exact = match card {
                CardSource::Copy { exact, .. } => *exact,
                _ => false,
            };
            let move_instance = matches!(card, CardSource::From { .. });
            let who = match ctrl {
                Some(Controller::Opponent) => controller.opponent(),
                _ => controller,
            };
            let mut summoned = Vec::new();
            for t in targets {
                if let Some(s) = summon_from_opt(db, state, who, t, exact, move_instance, events)? {
                    summoned.push(s);
                }
            }
            maybe_bind(state, e, &summoned);
            return Ok(());
        }
        Effect::AddToHand { card, .. } => {
            let mut added = Vec::new();
            let exact = match card {
                CardSource::Copy { exact, .. } => *exact,
                _ => false,
            };
            for t in targets {
                if let Some(h) = copy_target_to_hand(db, state, controller, t, exact)? {
                    added.push(h);
                }
            }
            maybe_bind(state, e, &added);
            return Ok(());
        }
        Effect::GrantTraits { traits, until, .. } => {
            for t in targets {
                grant_traits_opt(state, controller, t, traits, until.as_ref());
            }
        }
        Effect::SpellboostHand { times, .. } => {
            let n = eval_amount(db, state, controller, Some(source), times).max(0);
            let ids: Vec<(PlayerId, u32)> = targets
                .iter()
                .filter_map(|t| match t {
                    TargetOpt::Hand { player, pos } => state
                        .player(*player)
                        .hand
                        .get(*pos as usize)
                        .map(|c| (*player, c.id)),
                    _ => None,
                })
                .collect();
            for (who, hid) in ids {
                for _ in 0..n {
                    spellboost_instance(db, state, who, hid, events)?;
                }
            }
            return Ok(());
        }
        Effect::RemoveTraits { traits, .. } => {
            for t in targets {
                remove_traits_opt(state, t, traits);
            }
        }
        Effect::Countdown { delta, .. } => {
            let d = eval_amount(db, state, controller, Some(source), delta);
            apply_each_captured(state, targets, |st, t| countdown_opt(db, st, t, d, events))?;
        }
        Effect::GrantAbility { ability, .. } => {
            for t in targets {
                if let TargetOpt::Slot { player, slot } = t {
                    if let Some(f) = state.field_inst_mut(*player, *slot) {
                        f.granted.push((**ability).clone());
                    }
                }
            }
        }
        Effect::Select { .. } => {}
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
            ChooseBy::Player => {
                if matches!(pick.as_pick(), ChoosePick::All) {
                    if let Some(opts) = resolve_choose_options(db, state, source, e) {
                        for opt in opts.iter().rev() {
                            push_effects(state, controller, source, opt.effects.clone());
                        }
                    }
                }
            }
            ChooseBy::Random | ChooseBy::RandomUnused => {
                let resolved = if options.is_some() {
                    options.clone()
                } else {
                    resolve_choose_options(db, state, source, e)
                };
                if let Some(opts) = resolved {
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
            // Targets for this op only, selected now (E24) — not when the
            // enclosing list started.
            let n = eval_amount(db, state, controller, Some(source), amount);
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            if *split == Some(true) {
                split_damage(db, state, controller, &ts, n, events)?;
            } else {
                apply_each_captured(state, &ts, |st, t| {
                    deal_to_opt(db, st, controller, t, n, events)
                })?;
            }
            maybe_bind(state, e, &ts);
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
            maybe_bind(state, e, &ts);
        }
        Effect::Select { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            maybe_bind(state, e, &ts);
        }
        Effect::Destroy { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            maybe_bind(state, e, &ts);
            apply_each_captured(state, &ts, |st, t| {
                if let TargetOpt::Slot { player, slot } = t {
                    destroy_by_ability(db, st, *player, *slot, events)?;
                }
                Ok(())
            })?;
            maybe_bind(state, e, &ts);
        }
        Effect::Banish { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            let bound: Vec<TargetOpt> =
                ts.iter().filter_map(|t| target_as_card(state, t)).collect();
            apply_each_captured(state, &ts, |st, t| {
                banish_opt(st, t, events);
                Ok(())
            })?;
            maybe_bind(state, e, &bound);
        }
        Effect::ReturnToHand { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| {
                bounce_opt(db, st, controller, t, events)
            })?;
        }
        Effect::ReturnToDeck { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            let bound: Vec<TargetOpt> =
                ts.iter().filter_map(|t| target_as_card(state, t)).collect();
            apply_each_captured(state, &ts, |st, t| {
                return_deck_opt(db, st, t);
                Ok(())
            })?;
            maybe_bind(state, e, &bound);
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
            let mut used_names = Vec::new();
            for _ in 0..n {
                // sequential one-at-a-time — ruling 2026-09-05
                if let Some(t) =
                    summon_source(db, state, who, card, source, false, &used_names, events)?
                {
                    if let TargetOpt::Slot { player, slot } = t {
                        if let Some(c) = state.field_inst(player, slot) {
                            used_names.push(c.card);
                        }
                    }
                    summoned.push(t);
                }
            }
            maybe_bind(state, e, &summoned);
        }
        Effect::AddToHand { card, count, .. } => {
            let mut added = Vec::new();
            match card {
                CardSource::Copy { copy_of, exact } => {
                    // One resolve: copy each target (Wolfraud's 5 deck
                    // instances). Do not loop `count` times on `ts.first()`.
                    let ts = resolve_select_rolling(db, state, controller, source, copy_of)?;
                    for t in &ts {
                        if let Some(h) = copy_target_to_hand(db, state, controller, t, *exact)? {
                            added.push(h);
                        }
                    }
                }
                _ => {
                    let n = eval_amount(db, state, controller, Some(source), count).max(0);
                    for _ in 0..n {
                        if let Some(t) = add_source_to_hand(db, state, controller, source, card)? {
                            added.push(t);
                        }
                    }
                }
            }
            maybe_bind(state, e, &added);
        }
        Effect::Draw {
            count,
            filter,
            distinct_names,
            player,
            ..
        } => {
            let who = match player {
                Some(CrestPlayer::Opponent) => controller.opponent(),
                _ => controller,
            };
            let n = eval_amount(db, state, controller, Some(source), count).max(0);
            let mut exclude: Vec<CardId> = Vec::new();
            let mut drawn = Vec::new();
            for _ in 0..n {
                if let Some(id) = draw_one(state, who, filter.as_ref(), &exclude)? {
                    if distinct_names == &Some(true) {
                        exclude.push(id);
                    }
                    events.push(Event::Draw {
                        player: who,
                        card: id,
                    });
                    note_draw(db, state, who, id);
                    if let Some(pos) = state.player(who).hand.iter().rposition(|c| c.card == id) {
                        drawn.push(TargetOpt::Hand {
                            player: who,
                            pos: pos as u8,
                        });
                    }
                }
            }
            maybe_bind(state, e, &drawn);
        }
        Effect::Discard { select, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            bind_discard_as_cards(state, e, &ts);
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
                        apply_evolve_action(
                            db,
                            st,
                            *player,
                            Slot(*slot),
                            *super_evolve,
                            true,
                            events,
                        )?;
                    }
                }
                Ok(())
            })?;
        }
        Effect::GrantTraits {
            select,
            traits,
            until,
            ..
        } => {
            let ts = resolve_select(db, state, controller, source, select);
            for t in &ts {
                grant_traits_opt(state, controller, t, traits, until.as_ref());
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
            remove_matching_crests(db, state, controller, select, events);
        }
        Effect::Countdown { select, delta, .. } => {
            let d = eval_amount(db, state, controller, Some(source), delta);
            if matches!(select, Selector::Pool(p) if p.zone == Zone::Crests) {
                apply_crest_countdown(db, state, controller, source, select, d, events)?;
            } else {
                let ts = resolve_select(db, state, controller, source, select);
                apply_each_captured(state, &ts, |st, t| countdown_opt(db, st, t, d, events))?;
            }
        }
        Effect::Counter {
            key, how, amount, ..
        } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            apply_counter(db, state, controller, source, key, *how, n, events)?;
        }
        Effect::Reanimate { max_cost, .. } => {
            let x = eval_amount(db, state, controller, Some(source), max_cost);
            let summoned = reanimate(db, state, controller, x, events)?;
            if let Some(t) = summoned {
                maybe_bind(state, e, &[t]);
            } else {
                maybe_bind(state, e, &[]);
            }
        }
        Effect::Replicate { ability, .. } => {
            replicate(db, state, controller, source, *ability)?;
        }
        Effect::Ep {
            action,
            super_ep,
            amount,
            ..
        } => {
            let n = eval_amount(db, state, controller, Some(source), amount);
            let p = state.player_mut(controller);
            match (*action, *super_ep) {
                (EpAction::Gain, false) => p.ep += n,
                (EpAction::Gain, true) => p.sep += n,
                (EpAction::Spend, false) => p.ep = (p.ep - n).max(0),
                (EpAction::Spend, true) => p.sep = (p.sep - n).max(0),
            }
        }
        Effect::SpellboostHand { times, select, .. } => {
            let n = eval_amount(db, state, controller, Some(source), times).max(0);
            if let Some(sel) = select {
                let ts = resolve_select_rolling(db, state, controller, source, sel)?;
                let ids: Vec<(PlayerId, u32)> = ts
                    .iter()
                    .filter_map(|t| match t {
                        TargetOpt::Hand { player, pos } => state
                            .player(*player)
                            .hand
                            .get(*pos as usize)
                            .map(|c| (*player, c.id)),
                        _ => None,
                    })
                    .collect();
                for (who, hid) in ids {
                    for _ in 0..n {
                        spellboost_instance(db, state, who, hid, events)?;
                    }
                }
            } else {
                for _ in 0..n {
                    spellboost_hand(db, state, controller, events)?;
                }
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
            let md_change = max_defense.as_ref().map(|md| match md {
                MaxDefenseChange::Set { set } => {
                    (true, eval_amount(db, state, controller, Some(source), set))
                }
                MaxDefenseChange::Delta { delta } => (
                    false,
                    eval_amount(db, state, controller, Some(source), delta),
                ),
            });
            let cap_v = damage_cap
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a));
            let bonus_v = damage_taken_bonus
                .as_ref()
                .map(|a| eval_amount(db, state, controller, Some(source), a))
                .unwrap_or(0);
            let ts = resolve_select(db, state, controller, source, select);
            let players: Vec<PlayerId> = {
                let mut v: Vec<PlayerId> = ts
                    .iter()
                    .filter_map(|t| match t {
                        TargetOpt::Leader { player } => Some(*player),
                        _ => None,
                    })
                    .collect();
                if v.is_empty() {
                    v.push(controller);
                }
                v
            };
            for player in players {
                let p = state.player_mut(player);
                if let Some((is_set, v)) = md_change {
                    if is_set {
                        p.leader_max = v.max(0);
                    } else {
                        // Delta floors at 0; max 0 clamps defense to 0 and is lethal.
                        p.leader_max = (p.leader_max + v).max(0);
                    }
                    p.leader_defense = p.leader_defense.min(p.leader_max);
                }
                p.leader_mods.push(LeaderMod {
                    max_defense: None,
                    damage_cap: cap_v,
                    damage_taken_bonus: bonus_v,
                    until: *until,
                });
            }
            check_leader_lethal(state);
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
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Crests {
                    let who = match p.side {
                        Side::Enemy => controller.opponent(),
                        _ => controller,
                    };
                    for c in &mut state.player_mut(who).crests {
                        if p.kind == SelectorKind::Faith && !c.faith {
                            continue;
                        }
                        c.granted.push((**ability).clone());
                    }
                    return Ok(());
                }
            }
            for t in resolve_select(db, state, controller, source, select) {
                if let TargetOpt::Slot { player, slot } = t {
                    if let Some(f) = state.field_inst_mut(player, slot) {
                        f.grant_ability((**ability).clone());
                    }
                }
            }
        }
        Effect::RemoveAbilities { select, on, .. } => {
            let ts = resolve_select(db, state, controller, source, select);
            for t in &ts {
                remove_abilities_opt(state, t, on.as_deref());
            }
            maybe_bind(state, e, &ts);
        }
        Effect::Invoke { .. } => {
            apply_invoke(db, state, controller, source, events)?;
        }
        Effect::Transform { select, into, .. } => {
            let ts = resolve_select_rolling(db, state, controller, source, select)?;
            apply_each_captured(state, &ts, |st, t| transform_opt(db, st, t, into, events))?;
            maybe_bind(state, e, &ts);
        }
        Effect::Sequence { steps, .. } => {
            if steps.is_empty() {
                return Ok(());
            }
            let idx = sequence_index_of(state, source);
            let i = (idx as usize) % steps.len();
            set_sequence_index(state, source, ((i + 1) % steps.len()) as u32);
            push_effects(state, controller, source, steps[i].effects.clone());
        }
        Effect::AddToDeck {
            card,
            count,
            position,
            ..
        } => {
            let n = eval_amount(db, state, controller, Some(source), count).max(0);
            let mut added = Vec::new();
            for _ in 0..n {
                if let Some(t) = add_source_to_deck(db, state, controller, source, card, *position)?
                {
                    added.push(t);
                }
            }
            maybe_bind(state, e, &added);
        }
        Effect::RandomSplit { .. } => {
            return Err(Illegal::Unsupported(Unsupported {
                card: format!("{controller:?}"),
                construct: "op:randomSplit".into(),
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
                        if let Some(inst) = p.field[slot as usize].take() {
                            p.shadows += 1;
                            p.cemetery.push(inst);
                            p.compact_field();
                        }
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
        PayResource::Faith => {
            if p.faith >= n {
                p.faith -= n;
                true
            } else {
                false
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_counter(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    source: SourceRef,
    key: &CounterKey,
    how: CounterHow,
    n: i32,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
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
            if how == CounterHow::Set {
                let p = state.player_mut(who);
                p.earth = n;
                events.push(Event::Counter {
                    key: "earth".into(),
                    value: p.earth,
                });
                return Ok(());
            }
            if state.player(who).earth_slot.is_some() {
                // Holder on the field: spells (and Engage) raise the stack.
                // Owner 2026-09-10: a full board still allows this.
                let p = state.player_mut(who);
                p.earth += n;
                events.push(Event::Counter {
                    key: "earth".into(),
                    value: p.earth,
                });
                return Ok(());
            }
            // No holder: summon one Magic Sediment with sigil count X
            // (glossary). Full field → excess skipped, the sigil is lost.
            let sediment = CardId::parse("90031210").expect("sediment id");
            let src = CardSource::Named { named: sediment };
            summon_source(db, state, who, &src, source, false, &[], events)?;
            if state.player(who).earth_slot.is_some() && n > 1 {
                state.player_mut(who).earth = n;
            }
            events.push(Event::Counter {
                key: "earth".into(),
                value: state.player(who).earth,
            });
        }
        CounterKey::Named(NamedCounter::Faith) => {
            let p = state.player_mut(who);
            p.faith = if how == CounterHow::Set {
                n
            } else {
                p.faith + n
            };
            events.push(Event::Counter {
                key: "faith".into(),
                value: p.faith,
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
    Ok(())
}

fn combat_damage(
    db: &CardDb,
    state: &mut State,
    me: PlayerId,
    attacker_id: u32,
    target: AttackTarget,
    defender_id: Option<u32>,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let Some(slot) = state.find_field(me, attacker_id) else {
        return Ok(());
    };
    let Some(att) = state.field_inst(me, slot).cloned() else {
        return Ok(());
    };
    let knockback = att.super_evolved;
    let opp = me.opponent();
    match target {
        AttackTarget::Leader => {
            let dealt = deal_leader(state, opp, att.attack.max(0), events);
            if att.is_drain() {
                restore_leader(db, state, me, dealt, events);
            }
        }
        AttackTarget::Slot(ds) => {
            // Knockback keys off the defender captured at declaration, not
            // whoever compacted into `ds` after other deaths (E23).
            let def_slot = match defender_id {
                Some(id) => match state.find_field(opp, id) {
                    Some(slot) => slot,
                    None => {
                        if knockback {
                            deal_leader(state, opp, 1, events);
                        }
                        return Ok(());
                    }
                },
                None => ds.0,
            };
            let Some(def) = state.field_inst(opp, def_slot).cloned() else {
                if knockback && defender_id.is_some_and(|id| state.find_field(opp, id).is_none()) {
                    deal_leader(state, opp, 1, events);
                }
                return Ok(());
            };
            // Strike / Clash resolve first. A follower already at 0 defense
            // (or an attacker killed by Clash) does not exchange combat damage.
            if att.defense <= 0 || def.defense <= 0 {
                return Ok(());
            }
            let dealt = deal_follower(state, opp, def_slot, att.attack.max(0), me, events);
            let _ = deal_follower(state, me, slot, def.attack.max(0), opp, events);
            if att.is_drain() {
                // Barrier (and own-turn SE protection) can reduce the instance
                // to 0; Drain restores the damage actually dealt (rulebook).
                restore_leader(db, state, me, dealt, events);
            }
            // Bane even at 0 — rulebook Bane
            if att.is_bane() {
                if let Some(dslot) = state.find_field(opp, def.id) {
                    if let Some(d) = state.field_inst(opp, dslot) {
                        if !bane_blocked(state, opp, d) {
                            destroy_slot(db, state, opp, dslot, false, events)?;
                        }
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
            let target_dead = state.find_field(opp, def.id).is_none();
            if knockback && target_dead {
                deal_leader(state, opp, 1, events);
            }
        }
    }
    Ok(())
}

/// Cannot be destroyed by abilities/effects (including Bane): printed
/// `cantBeDestroyedByAbilities`, Earth Sigil (official glossary 2026-09-10),
/// or super-evolved on the owner's turn (E31).
fn bane_blocked(state: &State, owner: PlayerId, inst: &CardInstance) -> bool {
    inst.traits.cant_be_destroyed_by_abilities == Some(true)
        || inst.is_earth_sigil()
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
        TargetOpt::Leader { player } => {
            let _ = deal_leader(state, *player, n, events);
        }
        TargetOpt::Slot { player, slot } => {
            let _ = deal_follower(state, *player, *slot, n, *player, events);
        }
        _ => {}
    }
    Ok(())
}

fn deal_leader(state: &mut State, who: PlayerId, raw: i32, events: &mut Vec<Event>) -> i32 {
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
    amt
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
) -> i32 {
    let active = state.active;
    let Some(f) = state.field_inst_mut(who, slot) else {
        return 0;
    };
    let mut amt = raw;
    // "takes N more damage" stacks and applies to a 0-damage event — 2026-08-31
    // (follower-level bonus not separately stored in M1 beyond leader mods)
    if raw <= 0 {
        // No damage to prevent — Barrier stays (Giada Strike vs 0-atk).
        amt = 0;
    } else if f.is_barrier() {
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
    amt
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
    // Any restore effect that resolves counts as "restored", including a
    // 0-heal at max defense (official Q&A Burnite `10144110`).
    raise_when(db, state, who, EventName::LeaderRestored, None, who);
}

fn buff_opt(db: &CardDb, state: &mut State, t: &TargetOpt, da: i32, dd: i32, eot: bool) {
    match t {
        TargetOpt::Slot { player, slot } => {
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
        TargetOpt::Hand { player, pos } => {
            if let Some(h) = state.player_mut(*player).hand.get_mut(*pos as usize) {
                h.attack += da;
                h.max_defense += dd;
                h.defense += dd;
            }
        }
        TargetOpt::Deck { player, id } => {
            if let Some(c) = state
                .player_mut(*player)
                .deck
                .iter_mut()
                .find(|c| c.id == *id)
            {
                c.attack += da;
                c.max_defense += dd;
                c.defense += dd;
            }
        }
        _ => {}
    }
}

fn grant_traits_opt(
    state: &mut State,
    caster: PlayerId,
    t: &TargetOpt,
    traits: &Traits,
    until: Option<&Until>,
) {
    match t {
        TargetOpt::Slot { player, slot } => {
            if let Some(f) = state.field_inst_mut(*player, *slot) {
                f.traits.merge_grant(traits);
                if traits.ambush == Some(true) {
                    f.flags.ambush_active = true;
                }
                if let Some(n) = traits.attacks_per_turn {
                    let extra = (n - 1).max(0);
                    f.flags.attacks_left += extra;
                }
                if let Some(u) = until {
                    f.temp_traits.push(TempTraitGrant {
                        traits: traits.clone(),
                        until: *u,
                        caster,
                    });
                }
            }
        }
        TargetOpt::Hand { player, pos } => {
            if let Some(h) = state.player_mut(*player).hand.get_mut(*pos as usize) {
                h.traits.merge_grant(traits);
                if let Some(u) = until {
                    h.temp_traits.push(TempTraitGrant {
                        traits: traits.clone(),
                        until: *u,
                        caster,
                    });
                }
            }
        }
        TargetOpt::Deck { player, id } => {
            if let Some(c) = state
                .player_mut(*player)
                .deck
                .iter_mut()
                .find(|c| c.id == *id)
            {
                c.traits.merge_grant(traits);
                if let Some(u) = until {
                    c.temp_traits.push(TempTraitGrant {
                        traits: traits.clone(),
                        until: *u,
                        caster,
                    });
                }
            }
        }
        _ => {}
    }
}

fn remove_abilities_opt(state: &mut State, t: &TargetOpt, on: Option<&[TriggerTag]>) {
    match t {
        TargetOpt::Slot { player, slot } => {
            if let Some(f) = state.field_inst_mut(*player, *slot) {
                f.remove_granted_abilities(on);
            }
        }
        TargetOpt::Hand { player, pos } => {
            if let Some(h) = state.player_mut(*player).hand.get_mut(*pos as usize) {
                h.remove_granted_abilities(on);
            }
        }
        TargetOpt::Deck { player, id } => {
            if let Some(c) = state
                .player_mut(*player)
                .deck
                .iter_mut()
                .find(|c| c.id == *id)
            {
                c.remove_granted_abilities(on);
            }
        }
        _ => {}
    }
}

fn target_as_card(state: &State, t: &TargetOpt) -> Option<TargetOpt> {
    match t {
        TargetOpt::Slot { player, slot } => state
            .field_inst(*player, *slot)
            .map(|c| TargetOpt::Card(c.card)),
        TargetOpt::Hand { player, pos } => state
            .player(*player)
            .hand
            .get(*pos as usize)
            .map(|c| TargetOpt::Card(c.card)),
        TargetOpt::Deck { player, id } => state
            .player(*player)
            .deck
            .iter()
            .find(|c| c.id == *id)
            .map(|c| TargetOpt::Card(c.card)),
        TargetOpt::Card(id) => Some(TargetOpt::Card(*id)),
        _ => None,
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

fn leave_redirects_to_banish(db: &CardDb, inst: &CardInstance) -> bool {
    let mut abs: Vec<Ability> = Vec::new();
    if let Ok(card) = db.card(inst.card) {
        abs.extend(card.abilities().iter().cloned());
    }
    abs.extend(inst.granted.iter().cloned());
    abs.iter().any(|a| {
        matches!(a, Ability::Leave { .. }) && a.effects().iter().any(effect_is_banish_self)
    })
}

fn effect_is_banish_self(e: &Effect) -> bool {
    match e {
        Effect::Banish {
            select: Selector::Ref(r),
            ..
        } if r.pick == RefPick::Self_ => true,
        Effect::Seq { effects, .. } | Effect::Pay { effects, .. } => {
            effects.iter().any(effect_is_banish_self)
        }
        _ => false,
    }
}

fn queue_last_words(db: &CardDb, state: &mut State, who: PlayerId, inst: &CardInstance, slot: u8) {
    let cat = if who == state.active { 4 } else { 6 };
    let src = SourceRef::Spell {
        player: who,
        card: inst.card,
    };
    let mut printed = 0u8;
    if inst.kind == CardKind::Amulet {
        if let Ok(card) = db.card(inst.card) {
            if card.kind() == CardKind::Follower {
                // Crystallize form: Last Words from the amulet mode, not the follower.
                for m in card.modes() {
                    if let Mode::Crystallize {
                        abilities: Some(abs),
                        ..
                    } = m
                    {
                        for a in abs {
                            if matches!(a, Ability::LastWords { .. }) {
                                state.queue.push(QueuedTrigger {
                                    category: cat,
                                    entry: slot as u32,
                                    printed_order: printed,
                                    controller: who,
                                    source: src,
                                    tag: TriggerTag::LastWords,
                                    effects: a.effects().to_vec(),
                                    subject: None,
                                    subject_id: None,
                                });
                                printed += 1;
                            }
                        }
                    }
                }
                for a in &inst.granted {
                    if matches!(a, Ability::LastWords { .. }) {
                        state.queue.push(QueuedTrigger {
                            category: cat,
                            entry: slot as u32,
                            printed_order: printed,
                            controller: who,
                            source: src,
                            tag: TriggerTag::LastWords,
                            effects: a.effects().to_vec(),
                            subject: None,
                            subject_id: None,
                        });
                        printed += 1;
                    }
                }
                return;
            }
        }
    }
    if inst.printed_tags.contains("lastWords") {
        if let Ok(card) = db.card(inst.card) {
            for a in card.abilities() {
                if matches!(a, Ability::LastWords { .. }) {
                    state.queue.push(QueuedTrigger {
                        category: cat,
                        entry: slot as u32,
                        printed_order: printed,
                        controller: who,
                        source: src,
                        tag: TriggerTag::LastWords,
                        effects: a.effects().to_vec(),
                        subject: None,
                        subject_id: None,
                    });
                    printed += 1;
                }
            }
        }
    }
    for a in &inst.granted {
        if matches!(a, Ability::LastWords { .. }) {
            state.queue.push(QueuedTrigger {
                category: cat,
                entry: slot as u32,
                printed_order: printed,
                controller: who,
                source: src,
                tag: TriggerTag::LastWords,
                effects: a.effects().to_vec(),
                subject: None,
                subject_id: None,
            });
            printed += 1;
        }
    }
}

/// Super-evolve own-turn: legal candidate, destroy from an ability/effect
/// fizzles (E31). Lethal 0-defense still goes through `destroy_slot`.
fn destroy_by_ability(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    slot: u8,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let Some(peek) = state.player(who).field[slot as usize].as_ref() else {
        return Ok(());
    };
    if bane_blocked(state, who, peek) {
        return Ok(());
    }
    destroy_slot(db, state, who, slot, false, events)
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
    if leave_redirects_to_banish(db, &inst) {
        // Ghost-style "When this card leaves the field, banish it."
        events.push(Event::Banish {
            card: inst.card,
            from: ZoneLabel::Field,
        });
        raise_when(
            db,
            state,
            who,
            EventName::AllyFollowerDestroyed,
            Some(&inst),
            who,
        );
        raise_when(
            db,
            state,
            who.opponent(),
            EventName::EnemyFollowerDestroyed,
            Some(&inst),
            who,
        );
        state.player_mut(who).banished.push(inst);
        state.player_mut(who).compact_field();
        return Ok(());
    }
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
            raise_when(
                db,
                state,
                who.opponent(),
                EventName::EnemyFollowerDestroyed,
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
    if inst.kind == CardKind::Follower || inst.kind == CardKind::Amulet {
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
        queue_last_words(db, state, who, &inst, slot);
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
        TargetOpt::Deck { player, id } => {
            if let Some(i) = state.player(*player).deck.iter().position(|c| c.id == *id) {
                let inst = state.player_mut(*player).deck.remove(i);
                events.push(Event::Banish {
                    card: inst.card,
                    from: ZoneLabel::Deck,
                });
                state.player_mut(*player).banished.push(inst);
            }
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
        TargetOpt::Deck { player, id } => {
            if let Some(pos) = state.player(*player).deck.iter().position(|c| c.id == *id) {
                let inst = state.player_mut(*player).deck.remove(pos);
                add_to_hand(state, controller, inst);
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
            insert_deck_random(state, *player, inst);
        }
        TargetOpt::Slot { player, slot } => {
            if let Some(inst) = state.player_mut(*player).field[*slot as usize].take() {
                state.player_mut(*player).compact_field();
                insert_deck_random(state, *player, reset_off_field(db, inst));
            }
        }
        _ => {}
    }
}

/// `position: random` — the old emitter records a `raw` shuffle that replay
/// ignores, so this must not emit a comparable pick. Deck order is not in
/// CanonicalState (multiset). Append so the oldest copy of an id stays first;
/// draws take that copy when the recorded pick is only the id.
fn insert_deck_random(state: &mut State, who: PlayerId, inst: CardInstance) {
    state.player_mut(who).deck.push(inst);
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
                            subject: None,
                            subject_id: None,
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

fn apply_crest_countdown(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    select: &Selector,
    delta: i32,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let (sides, want) = match select {
        Selector::Pool(p) if p.zone == Zone::Crests => {
            let sides = match p.side {
                Side::Ally => vec![controller],
                Side::Enemy => vec![controller.opponent()],
                Side::Any => vec![controller, controller.opponent()],
            };
            (sides, p.filter.as_ref().and_then(|f| f.card))
        }
        _ => return Ok(()),
    };
    let _ = source;
    let mut doomed: Vec<(PlayerId, u32)> = Vec::new();
    for who in sides {
        for c in state.player_mut(who).crests.iter_mut() {
            let match_id = want
                .map(|id| c.id == format!("crest:{}", id.as_str()) || c.id.ends_with(&id.as_str()))
                .unwrap_or(true);
            if !match_id {
                continue;
            }
            if let Some(cd) = c.countdown.as_mut() {
                *cd += delta;
                if *cd <= 0 {
                    doomed.push((who, c.granted_order));
                }
            }
        }
    }
    for (who, order) in doomed {
        expire_crest(db, state, who, order, events)?;
    }
    Ok(())
}

fn settle_deaths(db: &CardDb, state: &mut State, events: &mut Vec<Event>) -> Result<(), Illegal> {
    let mut dead = Vec::new();
    for p in PlayerId::ALL {
        for c in state.player(p).field.iter().flatten() {
            if c.kind == CardKind::Follower && c.defense <= 0 {
                dead.push((p, c.id));
            }
        }
    }
    for (p, id) in dead {
        if let Some(slot) = state.find_field(p, id) {
            destroy_slot(db, state, p, slot, false, events)?;
        }
    }
    Ok(())
}

fn split_damage(
    db: &CardDb,
    state: &mut State,
    ctrl: PlayerId,
    targets: &[TargetOpt],
    pool: i32,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    // Live: oldest first — Barrier consumes allocation — 2026-08-29.
    // Scripted: the old engine records `random_split` counts per target in
    // field order; consume and apply those so the pick stream stays aligned.
    let caps = capture_targets(state, targets);
    if state.rng.peek_what() == Some(PickWhat::RandomSplit) {
        if let Ok(counts) = state.rng.take_scripted_split() {
            for (cap, take) in caps.iter().zip(counts.iter()) {
                if *take <= 0 {
                    continue;
                }
                let Some(TargetOpt::Slot { player, slot }) = live_captured(state, cap) else {
                    continue;
                };
                let _ = deal_follower(state, player, slot, *take, ctrl, events);
            }
            let _ = db;
            return Ok(());
        }
    }
    let mut counts = vec![0i32; caps.len()];
    let mut live: Vec<usize> = Vec::new();
    let mut leader_i: Option<usize> = None;
    for (i, cap) in caps.iter().enumerate() {
        match live_captured(state, cap) {
            Some(TargetOpt::Slot { .. }) => live.push(i),
            Some(TargetOpt::Leader { .. }) => leader_i = Some(i),
            _ => {}
        }
    }
    let mut remaining = pool;
    let last = live.len().saturating_sub(1);
    for (k, &i) in live.iter().enumerate() {
        let Some(TargetOpt::Slot { player, slot }) = live_captured(state, &caps[i]) else {
            continue;
        };
        let def = state
            .field_inst(player, slot)
            .map(|c| c.defense.max(0))
            .unwrap_or(0);
        if leader_i.is_none() && k == last {
            // Glossary: leftover is one hit on the last follower when the
            // ability only targets followers (`includeLeader` is the
            // leader-spill case and is kept apart).
            counts[i] = remaining;
            remaining = 0;
        } else {
            let take = remaining.min(def);
            counts[i] = take;
            remaining -= take;
        }
    }
    if let Some(i) = leader_i {
        counts[i] = remaining;
        remaining = 0;
    }
    let _ = remaining;
    for (i, cap) in caps.iter().enumerate() {
        let take = counts[i];
        if take <= 0 {
            continue;
        }
        match live_captured(state, cap) {
            Some(TargetOpt::Slot { player, slot }) => {
                let _ = deal_follower(state, player, slot, take, ctrl, events);
            }
            Some(TargetOpt::Leader { player }) => {
                let _ = deal_leader(state, player, take, events);
            }
            _ => {}
        }
    }
    state.picks.push(crate::trace::Pick {
        what: PickWhat::RandomSplit,
        among: None,
        chose: crate::trace::PickChose::Counts(counts),
    });
    let _ = db;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn summon_source(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    src: &CardSource,
    source: SourceRef,
    exact: bool,
    used_names: &[CardId],
    events: &mut Vec<Event>,
) -> Result<Option<TargetOpt>, Illegal> {
    if state.player(who).first_empty_slot().is_none() {
        // Excess summons are skipped with no Rally and no RNG — ruling 2026-08-12.
        let earth = match src {
            CardSource::Named { named } => db
                .card(*named)
                .map(|c| c.tribes().contains(&crate::card::Tribe::EarthSigil))
                .unwrap_or(false),
            _ => false,
        };
        if !earth {
            return Ok(None);
        }
    }
    let inst = match src {
        CardSource::Named { named } => {
            let card = db.require_supported(*named).map_err(|e| {
                eprintln!("summon require_supported {named} {e}");
                match e {
                    LoadError::Unsupported(u) => Illegal::Unsupported(u),
                    _ => Illegal::NotLegal,
                }
            })?;
            CardInstance::from_card(card, state.alloc_id())
        }
        CardSource::From { from } => {
            let ts = resolve_select(db, state, who, source, from);
            // Full field already returned above — the card stays where it is.
            match ts.first() {
                Some(TargetOpt::Hand { player, pos }) => {
                    if (*pos as usize) >= state.player(*player).hand.len() {
                        return Ok(None);
                    }
                    let mut taken = state.player_mut(*player).hand.remove(*pos as usize);
                    taken.flags.summoning_sick = true;
                    taken
                }
                Some(TargetOpt::Deck { player, id }) => {
                    let Some(pos) = state.player(*player).deck.iter().position(|c| c.id == *id)
                    else {
                        return Ok(None);
                    };
                    let mut taken = state.player_mut(*player).deck.remove(pos);
                    taken.flags.summoning_sick = true;
                    taken
                }
                _ => return Ok(None),
            }
        }
        CardSource::Copy { copy_of, exact: ex } => {
            let ts = resolve_select_rolling(db, state, who, source, copy_of)?;
            let pick = ts
                .into_iter()
                .find(|t| card_id_of(state, t).is_some_and(|id| !used_names.contains(&id)));
            if let Some(t) = pick {
                if let Some(mut n) = copy_target_instance(db, state, &t, *ex)? {
                    n.flags.summoning_sick = true;
                    n
                } else {
                    return Ok(None);
                }
            } else if matches!(
                copy_of,
                Selector::Ref(r) if r.pick == RefPick::Self_
            ) {
                // Last Words after death: host is gone; "a copy" is a fresh print
                // (owner 2026-08-13 / 2026-09-05).
                let card_id = match source {
                    SourceRef::Spell { card, .. } => card,
                    SourceRef::Field { player, id } => {
                        if let Some(c) = state
                            .find_field(player, id)
                            .and_then(|s| state.field_inst(player, s).map(|c| c.card))
                        {
                            c
                        } else if let Some(c) = state.player(who).cemetery.last() {
                            c.card
                        } else {
                            return Ok(None);
                        }
                    }
                    _ => return Ok(None),
                };
                let card = db.card(card_id).map_err(|_| Illegal::NotLegal)?;
                let mut n = CardInstance::from_card(card, state.alloc_id());
                n.flags.summoning_sick = true;
                n
            } else if let Some(c) = bound_copy_from_banished(state, copy_of) {
                // Allure of the Mightiest: bind then banish; the field ref is
                // gone, but the instance is still in that player's banished pile.
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
        }
        CardSource::RandomFrom { random_from } => {
            let idxs: Vec<usize> = state
                .player(who)
                .deck
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    !used_names.contains(&c.card) && inst_matches_filter(state, who, c, random_from)
                })
                .map(|(i, _)| i)
                .collect();
            if idxs.is_empty() {
                return Ok(None);
            }
            let keys: Vec<String> = idxs
                .iter()
                .map(|&i| state.player(who).deck[i].card.as_str())
                .collect();
            let mut emit = Vec::new();
            let i = state
                .rng
                .pick_index_among(PickWhat::MultisetPick, Some("deck"), &keys, &mut emit)
                .map_err(Illegal::OraclePickNotLegal)?;
            state.picks.extend(emit);
            let chosen = state.player(who).deck[idxs[i.min(idxs.len() - 1)]].card;
            let pos = idxs
                .iter()
                .copied()
                .find(|&j| state.player(who).deck[j].card == chosen)
                .unwrap_or(idxs[i.min(idxs.len() - 1)]);
            let mut taken = state.player_mut(who).deck.remove(pos);
            taken.id = state.alloc_id();
            taken.flags.summoning_sick = true;
            taken
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
    let is_sigil = inst.is_earth_sigil();
    state.player_mut(who).field[slot as usize] = Some(inst);
    if is_sigil {
        state.player_mut(who).earth_slot = Some(slot);
        state.player_mut(who).earth += 1;
    }
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

fn bound_copy_from_banished(state: &State, copy_of: &Selector) -> Option<CardInstance> {
    let Selector::Bound(b) = copy_of else {
        return None;
    };
    let refs = state.bindings.get(&b.ref_name)?;
    for r in refs {
        if let BoundRef::Field { player, id, .. } = r {
            if let Some(c) = state.player(*player).banished.iter().find(|c| c.id == *id) {
                return Some(c.clone());
            }
            if let Some(c) = state.player(*player).cemetery.iter().find(|c| c.id == *id) {
                return Some(c.clone());
            }
        }
    }
    None
}

fn add_source_to_deck(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    source: SourceRef,
    src: &CardSource,
    position: DeckPosition,
) -> Result<Option<TargetOpt>, Illegal> {
    let inst = match src {
        CardSource::Named { named } => {
            let card = db.require_supported(*named).map_err(|e| match e {
                LoadError::Unsupported(u) => Illegal::Unsupported(u),
                _ => Illegal::NotLegal,
            })?;
            Some(CardInstance::from_card(card, state.alloc_id()))
        }
        CardSource::Copy { copy_of, exact } => {
            let ts = resolve_select(db, state, who, source, copy_of);
            let Some(t) = ts.first() else {
                return Ok(None);
            };
            copy_target_instance(db, state, t, *exact)?
        }
        CardSource::RandomFrom { random_from } => {
            let _ = random_from;
            return Ok(None);
        }
        CardSource::From { .. } => {
            return Err(Illegal::Unsupported(Unsupported {
                card: who.as_str().into(),
                construct: "CardSource.from (addToDeck)".into(),
            }));
        }
    };
    let Some(inst) = inst else {
        return Ok(None);
    };
    // `position: random` — CanonicalState is a deck multiset; do not emit a pick.
    let _ = position;
    let id = inst.id;
    insert_deck_random(state, who, inst);
    Ok(Some(TargetOpt::Deck { player: who, id }))
}

fn add_source_to_hand(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    source: SourceRef,
    src: &CardSource,
) -> Result<Option<TargetOpt>, Illegal> {
    match src {
        CardSource::Named { named } => {
            let card = db.require_supported(*named).map_err(|e| match e {
                LoadError::Unsupported(u) => Illegal::Unsupported(u),
                _ => Illegal::NotLegal,
            })?;
            let inst = CardInstance::from_card(card, state.alloc_id());
            Ok(push_hand_target(state, who, inst))
        }
        CardSource::Copy { copy_of, exact } => {
            let ts = resolve_select_rolling(db, state, who, source, copy_of).unwrap_or_default();
            let mut last = None;
            for t in &ts {
                last = copy_target_to_hand(db, state, who, t, *exact)?;
            }
            Ok(last)
        }
        CardSource::From { .. } => Err(Illegal::Unsupported(Unsupported {
            card: who.as_str().into(),
            construct: "CardSource.from (addToHand)".into(),
        })),
        CardSource::RandomFrom { .. } => Err(Illegal::Unsupported(Unsupported {
            card: who.as_str().into(),
            construct: "CardSource.randomFrom".into(),
        })),
    }
}

fn push_hand_target(state: &mut State, who: PlayerId, inst: CardInstance) -> Option<TargetOpt> {
    let before = state.player(who).hand.len();
    add_to_hand(state, who, inst);
    if state.player(who).hand.len() > before {
        Some(TargetOpt::Hand {
            player: who,
            pos: before as u8,
        })
    } else {
        None
    }
}

fn copy_target_to_hand(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    t: &TargetOpt,
    exact: bool,
) -> Result<Option<TargetOpt>, Illegal> {
    let src_inst = match t {
        TargetOpt::Slot { player, slot } => state.field_inst(*player, *slot).cloned(),
        TargetOpt::Hand { player, pos } => state.player(*player).hand.get(*pos as usize).cloned(),
        TargetOpt::Deck { player, id } => state
            .player(*player)
            .deck
            .iter()
            .find(|c| c.id == *id)
            .cloned(),
        TargetOpt::Card(id) => db.card(*id).ok().map(|c| CardInstance::from_card(c, 0)),
        _ => None,
    };
    let Some(c) = src_inst else {
        return Ok(None);
    };
    let mut n = if exact {
        c
    } else {
        // A copy, not an exact copy: fresh printed card — ruling 2026-09-05.
        let card = db.card(c.card).map_err(|_| Illegal::NotLegal)?;
        CardInstance::from_card(card, 0)
    };
    n.id = state.alloc_id();
    Ok(push_hand_target(state, who, n))
}

/// Field transform: original ceases in place (no Last Words, no shadow, no
/// cemetery, not a leave). `exact: false` is a fresh print; `exact: true`
/// clones the rolled instance (cost modifiers included — Grandeur / glossary
/// Exact Copy). The replacement did not enter (no Rally, no enter triggers,
/// no enteredThisMatch). Rush/Storm still allow attacking that turn — owner
/// 2026-09-10.
fn transform_slot(
    db: &CardDb,
    state: &mut State,
    player: PlayerId,
    slot: u8,
    into: &CardSource,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let Some(mut neu) = materialize_transform(db, state, player, into)? else {
        return Ok(());
    };
    let Some(old) = state.player_mut(player).field[slot as usize].take() else {
        return Ok(());
    };
    if old.is_earth_sigil() && state.player(player).earth_slot == Some(slot) {
        state.player_mut(player).earth_slot = None;
        state.player_mut(player).earth = 0;
    }
    let named = neu.card;
    neu.flags.summoning_sick = true;
    neu.flags.attacks_left = neu.traits.attacks_per_turn.unwrap_or(1);
    state.player_mut(player).field[slot as usize] = Some(neu);
    events.push(Event::Transform {
        slot: Slot(slot),
        into: named,
    });
    let _ = old;
    Ok(())
}

enum ResolvedTransform {
    Fresh(CardId),
    Exact(Box<CardInstance>),
}

fn resolve_transform_into(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    into: &CardSource,
) -> Result<Option<ResolvedTransform>, Illegal> {
    match into {
        CardSource::Named { named } => Ok(Some(ResolvedTransform::Fresh(*named))),
        CardSource::Copy { copy_of, exact } => {
            // Independent roll per target (Grandeur official Q&A: two Clay
            // Golems each 50/50, not the same card for both).
            let ts = resolve_select_rolling(
                db,
                state,
                controller,
                SourceRef::Leader { player: controller },
                copy_of,
            )?;
            let inst = match ts.first() {
                Some(TargetOpt::Slot { player, slot }) => state.field_inst(*player, *slot).cloned(),
                Some(TargetOpt::Hand { player, pos }) => {
                    state.player(*player).hand.get(*pos as usize).cloned()
                }
                Some(TargetOpt::Deck { player, id }) => state
                    .player(*player)
                    .deck
                    .iter()
                    .find(|c| c.id == *id)
                    .cloned(),
                Some(TargetOpt::Card(id)) => {
                    return Ok(Some(ResolvedTransform::Fresh(*id)));
                }
                _ => None,
            };
            Ok(match inst {
                Some(c) if *exact => Some(ResolvedTransform::Exact(Box::new(c))),
                Some(c) => Some(ResolvedTransform::Fresh(c.card)),
                None => None,
            })
        }
        CardSource::From { .. } => Err(Illegal::Unsupported(Unsupported {
            card: controller.as_str().into(),
            construct: "transform from".into(),
        })),
        CardSource::RandomFrom { .. } => Err(Illegal::Unsupported(Unsupported {
            card: controller.as_str().into(),
            construct: "transform randomFrom".into(),
        })),
    }
}

fn materialize_transform(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    into: &CardSource,
) -> Result<Option<CardInstance>, Illegal> {
    match resolve_transform_into(db, state, controller, into)? {
        None => Ok(None),
        Some(ResolvedTransform::Fresh(named)) => {
            let card = db.require_supported(named).map_err(|e| match e {
                LoadError::Unsupported(u) => Illegal::Unsupported(u),
                _ => Illegal::NotLegal,
            })?;
            Ok(Some(CardInstance::from_card(card, state.alloc_id())))
        }
        Some(ResolvedTransform::Exact(mut inst)) => {
            inst.id = state.alloc_id();
            Ok(Some(*inst))
        }
    }
}

fn transform_opt(
    db: &CardDb,
    state: &mut State,
    t: &TargetOpt,
    into: &CardSource,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    match t {
        TargetOpt::Slot { player, slot } => transform_slot(db, state, *player, *slot, into, events),
        TargetOpt::Hand { player, pos } => {
            let Some(neu) = materialize_transform(db, state, *player, into)? else {
                return Ok(());
            };
            if (*pos as usize) < state.player(*player).hand.len() {
                state.player_mut(*player).hand[*pos as usize] = neu;
            }
            Ok(())
        }
        TargetOpt::Deck { player, id } => {
            let Some(mut neu) = materialize_transform(db, state, *player, into)? else {
                return Ok(());
            };
            if let Some(c) = state
                .player_mut(*player)
                .deck
                .iter_mut()
                .find(|c| c.id == *id)
            {
                neu.id = *id;
                *c = neu;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn card_id_of(state: &State, t: &TargetOpt) -> Option<CardId> {
    match t {
        TargetOpt::Slot { player, slot } => state.field_inst(*player, *slot).map(|c| c.card),
        TargetOpt::Hand { player, pos } => state
            .player(*player)
            .hand
            .get(*pos as usize)
            .map(|c| c.card),
        TargetOpt::Deck { player, id } => state
            .player(*player)
            .deck
            .iter()
            .find(|c| c.id == *id)
            .map(|c| c.card),
        TargetOpt::Card(id) => Some(*id),
        _ => None,
    }
}

fn sequence_index_of(state: &State, source: SourceRef) -> u32 {
    match source {
        SourceRef::Field { player, id } => state
            .find_field(player, id)
            .and_then(|s| state.field_inst(player, s))
            .map(|c| c.sequence_index)
            .unwrap_or(0),
        _ => 0,
    }
}

fn set_sequence_index(state: &mut State, source: SourceRef, v: u32) {
    if let SourceRef::Field { player, id } = source {
        if let Some(slot) = state.find_field(player, id) {
            if let Some(c) = state.field_inst_mut(player, slot) {
                c.sequence_index = v;
            }
        }
    }
}

fn crest_matches_filter(c: &CrestInstance, filter: Option<&Filter>) -> bool {
    let Some(f) = filter else {
        return !c.faith;
    };
    if let Some(id) = f.card {
        let want = format!("crest:{id}");
        return c.id == want || c.id == id.as_str();
    }
    true
}

fn remove_matching_crests(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    select: &Selector,
    events: &mut Vec<Event>,
) {
    let (who, filter) = match select {
        Selector::Pool(p) => {
            let who = match p.side {
                Side::Enemy => controller.opponent(),
                _ => controller,
            };
            (who, p.filter.as_ref())
        }
        _ => (controller, None),
    };
    let mut kept = Vec::new();
    let mut gone = Vec::new();
    for c in state.player_mut(who).crests.drain(..) {
        if crest_matches_filter(&c, filter) {
            gone.push(c);
        } else {
            kept.push(c);
        }
    }
    state.player_mut(who).crests = kept;
    for c in gone {
        events.push(Event::CrestRemove {
            player: who,
            id: c.id.clone(),
        });
        queue_crest_last_words(db, state, who, &c);
    }
}

fn queue_crest_last_words(db: &CardDb, state: &mut State, who: PlayerId, crest: &CrestInstance) {
    let Ok(def) = db.crest(&crest.id) else {
        return;
    };
    let cat = if who == state.active { 4 } else { 6 };
    let src = SourceRef::Leader { player: who };
    for (ord, a) in def.abilities().iter().enumerate() {
        if matches!(a, Ability::LastWords { .. }) {
            state.queue.push(QueuedTrigger {
                category: cat,
                entry: crest.granted_order,
                printed_order: ord as u8,
                controller: who,
                source: src,
                tag: TriggerTag::LastWords,
                effects: a.effects().to_vec(),
                subject: None,
                subject_id: None,
            });
        }
    }
}

fn copy_target_instance(
    db: &CardDb,
    state: &mut State,
    t: &TargetOpt,
    exact: bool,
) -> Result<Option<CardInstance>, Illegal> {
    let src_inst = match t {
        TargetOpt::Slot { player, slot } => state.field_inst(*player, *slot).cloned(),
        TargetOpt::Hand { player, pos } => state.player(*player).hand.get(*pos as usize).cloned(),
        TargetOpt::Deck { player, id } => state
            .player(*player)
            .deck
            .iter()
            .find(|c| c.id == *id)
            .cloned(),
        TargetOpt::Card(id) => {
            let card = db.card(*id).map_err(|_| Illegal::NotLegal)?;
            Some(CardInstance::from_card(card, 0))
        }
        _ => None,
    };
    let Some(c) = src_inst else {
        return Ok(None);
    };
    let mut n = if exact {
        c
    } else {
        let card = db.card(c.card).map_err(|_| Illegal::NotLegal)?;
        CardInstance::from_card(card, 0)
    };
    n.id = state.alloc_id();
    Ok(Some(n))
}

/// Invoke: named deck summon, no pick. One copy per card id per window.
/// Full field: stay in deck, Invoked does not fire. Rally and enter do count
/// (ruling 2026-08-12). Then `on: invoked`.
fn apply_invoke(
    db: &CardDb,
    state: &mut State,
    controller: PlayerId,
    source: SourceRef,
    events: &mut Vec<Event>,
) -> Result<(), Illegal> {
    let inst_id = match source {
        SourceRef::Hand { id, .. } | SourceRef::Field { id, .. } => id,
        _ => return Ok(()),
    };
    let Some(pos) = state
        .player(controller)
        .deck
        .iter()
        .position(|c| c.id == inst_id)
    else {
        return Ok(());
    };
    let card_id = state.player(controller).deck[pos].card;
    if state.invoked_ids.contains(&card_id) {
        return Ok(());
    }
    if state.player(controller).first_empty_slot().is_none() {
        return Ok(());
    }
    let mut inst = state.player_mut(controller).deck.remove(pos);
    state.invoked_ids.insert(card_id);
    inst.flags.summoning_sick = true;
    inst.flags.attacks_left = inst.traits.attacks_per_turn.unwrap_or(1);
    if inst.is_earth_sigil() && merge_earth(db, state, controller, &inst, events)? {
        return Ok(());
    }
    let Some(slot) = state.player(controller).first_empty_slot() else {
        return Ok(());
    };
    if inst.kind == CardKind::Follower {
        state.player_mut(controller).rally += 1;
        *state
            .player_mut(controller)
            .enter_counts
            .entry(inst.card)
            .or_insert(0) += 1;
    }
    let id = inst.card;
    let kind = inst.kind;
    let inst_uid = inst.id;
    let is_sigil = inst.is_earth_sigil();
    state.player_mut(controller).field[slot as usize] = Some(inst);
    if is_sigil {
        state.player_mut(controller).earth_slot = Some(slot);
        if state.player(controller).earth == 0 {
            state.player_mut(controller).earth = 1;
        }
    }
    events.push(Event::Summon {
        player: controller,
        card: id,
        slot: Slot(slot),
    });
    if kind == CardKind::Follower {
        if let Some(entered) = state.field_inst(controller, slot).cloned() {
            raise_follower_enter(db, state, controller, &entered);
        }
        queue_enter_reactions(db, state, controller, id, inst_uid, false);
        enqueue_card_triggers(db, state, controller, inst_uid, TriggerTag::Invoked, 4);
    }
    Ok(())
}

fn card_source_choose_pool(src: &CardSource) -> Option<&PoolSelector> {
    match src {
        CardSource::Copy {
            copy_of: Selector::Pool(p),
            ..
        }
        | CardSource::From {
            from: Selector::Pool(p),
        } if p.pick == PoolPick::Choose => Some(p),
        _ => None,
    }
}

fn resolve_choose_options(
    db: &CardDb,
    state: &State,
    source: SourceRef,
    e: &Effect,
) -> Option<Vec<ChooseOption>> {
    let Effect::Choose {
        options,
        options_from,
        ..
    } = e
    else {
        return None;
    };
    if let Some(opts) = options {
        return Some(opts.clone());
    }
    let from = options_from.as_ref()?;
    let card_id = source_card_id(state, source)?;
    let card = db.card(card_id).ok()?;
    let tag = match from {
        OptionsFrom::Fanfare => TriggerTag::Fanfare,
        OptionsFrom::LastWords => TriggerTag::LastWords,
        OptionsFrom::Evolve => TriggerTag::Evolve,
        OptionsFrom::SuperEvolve => TriggerTag::SuperEvolve,
        OptionsFrom::AnyEvolve => TriggerTag::AnyEvolve,
        OptionsFrom::AnySuperEvolve => TriggerTag::AnySuperEvolve,
        OptionsFrom::Strike => TriggerTag::Strike,
        OptionsFrom::FollowerStrike => TriggerTag::FollowerStrike,
        OptionsFrom::Clash => TriggerTag::Clash,
        OptionsFrom::Enter => TriggerTag::Enter,
        OptionsFrom::Leave => TriggerTag::Leave,
        OptionsFrom::Discarded => TriggerTag::Discarded,
        OptionsFrom::Invoked => TriggerTag::Invoked,
        OptionsFrom::Fused => TriggerTag::Fused,
        OptionsFrom::Spellboost => TriggerTag::Spellboost,
        OptionsFrom::Engage => TriggerTag::Engage,
        OptionsFrom::StartOfTurn => TriggerTag::StartOfTurn,
        OptionsFrom::EndOfTurn => TriggerTag::EndOfTurn,
        OptionsFrom::When => TriggerTag::When,
    };
    for a in card.abilities() {
        if a.tag() == tag {
            if let Some(opts) = first_choose_options(a.effects()) {
                return Some(opts);
            }
        }
    }
    None
}

fn first_choose_options(effects: &[Effect]) -> Option<Vec<ChooseOption>> {
    for e in effects {
        match e {
            Effect::Choose {
                options: Some(opts),
                ..
            } => return Some(opts.clone()),
            Effect::If {
                then, else_effects, ..
            } => {
                if let Some(o) = first_choose_options(then) {
                    return Some(o);
                }
                if let Some(els) = else_effects {
                    if let Some(o) = first_choose_options(els) {
                        return Some(o);
                    }
                }
            }
            Effect::Seq { effects, .. }
            | Effect::Pay { effects, .. }
            | Effect::Repeat { effects, .. } => {
                if let Some(o) = first_choose_options(effects) {
                    return Some(o);
                }
            }
            _ => {}
        }
    }
    None
}

fn source_card_id(state: &State, source: SourceRef) -> Option<CardId> {
    match source {
        SourceRef::Field { player, id } => state
            .find_field(player, id)
            .and_then(|s| state.field_inst(player, s).map(|c| c.card)),
        SourceRef::Hand { player, id } => state
            .player(player)
            .hand
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.card)
            .or_else(|| {
                state
                    .player(player)
                    .deck
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.card)
            })
            .or_else(|| {
                state
                    .player(player)
                    .cemetery
                    .iter()
                    .rev()
                    .find(|c| c.id == id)
                    .map(|c| c.card)
            }),
        SourceRef::Spell { card, .. } => Some(card),
        _ => None,
    }
}

/// Reanimate: field-destroyed only, highest cost ≤ X, random among ties
/// weighted by destroyed *instances* (not distinct names), summoning-sick,
/// no Fanfare, Departed on the instance — glossary 2026-09-10 / rulings 2026-09-02.
fn reanimate(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    max_cost: i32,
    events: &mut Vec<Event>,
) -> Result<Option<TargetOpt>, Illegal> {
    let cands: Vec<(CardId, i32)> = state
        .player(who)
        .destroyed_history
        .iter()
        .filter(|r| r.from_field && r.kind == CardKind::Follower && r.base_cost <= max_cost)
        .map(|r| (r.card, r.base_cost))
        .collect();
    if cands.is_empty() {
        return Ok(None);
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
    // Official glossary, 2026-09-10: Reanimate "gives it the Departed trait."
    // On the instance only — a later printed copy does not inherit it.
    if !inst.tribes.contains(&crate::card::Tribe::Departed) {
        inst.tribes.push(crate::card::Tribe::Departed);
    }
    let Some(slot) = state.player(who).first_empty_slot() else {
        return Ok(None);
    };
    state.player_mut(who).rally += 1;
    *state.player_mut(who).enter_counts.entry(id).or_insert(0) += 1;
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
    Ok(Some(TargetOpt::Slot { player: who, slot }))
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
        granted: Vec::new(),
    });
    events.push(Event::CrestGain {
        player: who,
        id: id.to_string(),
    });
}

// ----- selectors / filters / conditions / amounts -----

fn bind_discard_as_cards(state: &mut State, e: &Effect, targets: &[TargetOpt]) {
    let Some(name) = e.as_bind() else {
        return;
    };
    let refs: Vec<BoundRef> = targets
        .iter()
        .filter_map(|t| match t {
            TargetOpt::Hand { player, pos } => state
                .player(*player)
                .hand
                .get(*pos as usize)
                .map(|c| BoundRef::Card(c.card)),
            TargetOpt::Card(id) => Some(BoundRef::Card(*id)),
            other => opt_to_bound(state, other),
        })
        .collect();
    state.bindings.insert(name.to_string(), refs);
}

fn target_matches_count_filter(
    db: &CardDb,
    state: &State,
    who: PlayerId,
    t: &TargetOpt,
    f: &Filter,
) -> bool {
    match t {
        TargetOpt::Slot { player, slot } => state
            .field_inst(*player, *slot)
            .map(|c| inst_matches_filter(state, who, c, f))
            .unwrap_or(false),
        TargetOpt::Hand { player, pos } => state
            .player(*player)
            .hand
            .get(*pos as usize)
            .map(|c| inst_matches_filter(state, who, c, f))
            .unwrap_or(false),
        TargetOpt::Deck { player, id } => state
            .player(*player)
            .deck
            .iter()
            .find(|c| c.id == *id)
            .map(|c| inst_matches_filter(state, who, c, f))
            .unwrap_or(false),
        TargetOpt::Card(id) => db
            .card(*id)
            .ok()
            .map(|c| {
                let inst = CardInstance::from_card(c, 0);
                inst_matches_filter(state, who, &inst, f)
            })
            .unwrap_or(false),
        _ => false,
    }
}

fn summon_from_opt(
    db: &CardDb,
    state: &mut State,
    who: PlayerId,
    t: &TargetOpt,
    exact: bool,
    move_instance: bool,
    events: &mut Vec<Event>,
) -> Result<Option<TargetOpt>, Illegal> {
    if state.player(who).first_empty_slot().is_none() {
        return Ok(None);
    }
    let inst = match t {
        TargetOpt::Hand { player, pos } => {
            if (*pos as usize) >= state.player(*player).hand.len() {
                return Ok(None);
            }
            if move_instance {
                let mut taken = state.player_mut(*player).hand.remove(*pos as usize);
                taken.flags.summoning_sick = true;
                taken
            } else {
                let c = state.player(*player).hand[*pos as usize].clone();
                let mut n = if exact {
                    c
                } else {
                    let card = db.card(c.card).map_err(|_| Illegal::NotLegal)?;
                    CardInstance::from_card(card, 0)
                };
                n.id = state.alloc_id();
                n.flags.summoning_sick = true;
                n
            }
        }
        TargetOpt::Deck { player, id } => {
            let Some(pos) = state.player(*player).deck.iter().position(|c| c.id == *id) else {
                return Ok(None);
            };
            if move_instance {
                let mut taken = state.player_mut(*player).deck.remove(pos);
                taken.flags.summoning_sick = true;
                taken
            } else {
                let c = state.player(*player).deck[pos].clone();
                let mut n = if exact {
                    c
                } else {
                    let card = db.card(c.card).map_err(|_| Illegal::NotLegal)?;
                    CardInstance::from_card(card, 0)
                };
                n.id = state.alloc_id();
                n.flags.summoning_sick = true;
                n
            }
        }
        TargetOpt::Slot { player, slot } => {
            if move_instance {
                return Ok(None);
            }
            let Some(c) = state.field_inst(*player, *slot).cloned() else {
                return Ok(None);
            };
            let mut n = if exact {
                c
            } else {
                let card = db.card(c.card).map_err(|_| Illegal::NotLegal)?;
                CardInstance::from_card(card, 0)
            };
            n.id = state.alloc_id();
            n.flags.summoning_sick = true;
            n
        }
        TargetOpt::Card(id) => {
            let card = db.card(*id).map_err(|_| Illegal::NotLegal)?;
            let mut n = CardInstance::from_card(card, state.alloc_id());
            n.flags.summoning_sick = true;
            n
        }
        _ => return Ok(None),
    };
    // Reuse the named-summon placement by wrapping as a one-off Named path
    // is awkward; place the instance here the same way summon_source does.
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
    let is_sigil = inst.is_earth_sigil();
    state.player_mut(who).field[slot as usize] = Some(inst);
    if is_sigil {
        state.player_mut(who).earth_slot = Some(slot);
        state.player_mut(who).earth += 1;
    }
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

fn maybe_bind(state: &mut State, e: &Effect, targets: &[TargetOpt]) {
    if let Some(name) = e.as_bind() {
        let refs: Vec<BoundRef> = targets
            .iter()
            .filter_map(|t| opt_to_bound(state, t))
            .collect();
        if state.bind_append {
            state
                .bindings
                .entry(name.to_string())
                .or_default()
                .extend(refs);
        } else {
            state.bindings.insert(name.to_string(), refs);
        }
    }
}

fn opt_to_bound(state: &State, t: &TargetOpt) -> Option<BoundRef> {
    match t {
        TargetOpt::Slot { player, slot } => {
            state.field_inst(*player, *slot).map(|c| BoundRef::Field {
                player: *player,
                id: c.id,
                card: c.card,
                kind: c.kind,
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
        TargetOpt::Deck { player, id } => Some(BoundRef::Deck {
            player: *player,
            id: *id,
        }),
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
            BoundRef::Field { player, id, .. } => {
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
            BoundRef::Deck { player, id } => {
                let live = state.player(*player).deck.iter().any(|c| c.id == *id);
                live.then_some(TargetOpt::Deck {
                    player: *player,
                    id: *id,
                })
            }
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
                .map(|a| eval_amount(db, state, controller, Some(source), a))
                .unwrap_or(1);
            if n <= 0 {
                return Ok(Vec::new());
            }
            random_pool_apply(state, cands, n as usize, p.pick == PoolPick::RandomDistinct)
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
            RefPick::Entering => {
                live_field_subject(state, state.event_subject.clone(), state.event_inst_id)
                    .into_iter()
                    .collect()
            }
            RefPick::Opposing => state.combat_opposing.clone().into_iter().collect(),
            RefPick::Selected | RefPick::Attacker | RefPick::Defender => Vec::new(),
        },
        Selector::Bound(b) => resolve_bound(state, &b.ref_name),
        Selector::Pool(p) => {
            let mut c = pool_target_opts(db, state, controller, source, p);
            match p.pick {
                PoolPick::All | PoolPick::Choose => c,
                PoolPick::Leftmost => {
                    let n = match &p.count {
                        Some(Amount::Int(k)) => (*k).max(1) as usize,
                        _ => 1,
                    };
                    c.into_iter().take(n).collect()
                }
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
    if p.filter.as_ref().is_some_and(filter_wants_destroyed) {
        return pool_destroyed_history(db, state, controller, p);
    }
    let mut out = Vec::new();
    for who in sides {
        if p.filter.as_ref().and_then(|f| f.destroyed_this_match) == Some(true) {
            let mut seen = std::collections::BTreeSet::new();
            for r in &state.player(who).destroyed_history {
                if !r.from_field {
                    continue;
                }
                if p.kind == SelectorKind::Follower && r.kind != CardKind::Follower {
                    continue;
                }
                if let Some(f) = &p.filter {
                    if let Some(t) = &f.tribe {
                        let Ok(card) = db.card(r.card) else { continue };
                        let ok = match t {
                            crate::card::TribeOrList::One(tr) => card.tribes().contains(tr),
                            crate::card::TribeOrList::Many(ts) => {
                                ts.iter().any(|tr| card.tribes().contains(tr))
                            }
                        };
                        if !ok {
                            continue;
                        }
                    }
                    if let Some(id) = f.card {
                        if r.card != id {
                            continue;
                        }
                    }
                }
                if p.pick == PoolPick::RandomDistinct && !seen.insert(r.card) {
                    continue;
                }
                out.push(TargetOpt::Card(r.card));
            }
            continue;
        }
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
                    // Aura: not selectable by the enemy for `pick: choose`.
                    // Random still rolls them (M1 ramp traces); Earth Sigil
                    // amulets are also `kind: amulet` so follower random skips them.
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
                    out.push(TargetOpt::Deck {
                        player: who,
                        id: c.id,
                    });
                }
            }
            Zone::Cemetery | Zone::Crests => {}
        }
    }
    let _ = db;
    out
}

fn filter_wants_destroyed(f: &Filter) -> bool {
    if f.destroyed_this_match == Some(true) {
        return true;
    }
    if let Some(all) = &f.all {
        return all.iter().any(filter_wants_destroyed);
    }
    false
}

fn pool_destroyed_history(
    db: &CardDb,
    state: &State,
    controller: PlayerId,
    p: &PoolSelector,
) -> Vec<TargetOpt> {
    let sides: Vec<PlayerId> = match p.side {
        Side::Ally => vec![controller],
        Side::Enemy => vec![controller.opponent()],
        Side::Any => vec![controller, controller.opponent()],
    };
    let mut out = Vec::new();
    for who in sides {
        for rec in &state.player(who).destroyed_history {
            if rec.owner != who {
                continue;
            }
            let kind_ok = match p.kind {
                SelectorKind::Follower => rec.kind == CardKind::Follower,
                SelectorKind::Amulet => rec.kind == CardKind::Amulet,
                SelectorKind::Card | SelectorKind::Character => true,
                SelectorKind::Leader | SelectorKind::Faith => false,
            };
            if !kind_ok {
                continue;
            }
            let Ok(card) = db.card(rec.card) else {
                continue;
            };
            let mut inst = CardInstance::from_card(card, 0);
            inst.base_cost = rec.base_cost;
            inst.kind = rec.kind;
            if let Some(f) = &p.filter {
                if !inst_matches_filter(state, who, &inst, f) {
                    continue;
                }
            }
            out.push(TargetOpt::Card(rec.card));
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

fn inst_matches_filter(_state: &State, _who: PlayerId, c: &CardInstance, f: &Filter) -> bool {
    if let Some(all) = &f.all {
        return all.iter().all(|x| inst_matches_filter(_state, _who, c, x));
    }
    if let Some(any) = &f.any {
        return any.iter().any(|x| inst_matches_filter(_state, _who, c, x));
    }
    if let Some(n) = &f.not {
        return !inst_matches_filter(_state, _who, c, n);
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
    if let Some(n) = &f.base_cost_eq {
        if c.base_cost != eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.base_cost_lte {
        if c.base_cost > eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.base_cost_gte {
        if c.base_cost < eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(ns) = &f.base_cost_in {
        if !ns.contains(&c.base_cost) {
            return false;
        }
    }
    if let Some(n) = &f.attack_lte {
        if c.attack > eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.attack_gte {
        if c.attack < eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.defense_lte {
        if c.defense > eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.defense_gte {
        if c.defense < eval_amount_simple(n) {
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
    if let Some(n) = &f.defense_lte {
        if c.defense > eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.defense_gte {
        if c.defense < eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.base_cost_lte {
        if c.base_cost > eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(n) = &f.base_cost_eq {
        if c.base_cost != eval_amount_simple(n) {
            return false;
        }
    }
    if let Some(tk) = f.has_trait {
        if !c.traits.is_set(tk) {
            return false;
        }
    }
    if let Some(b) = f.has_last_words {
        let has = c.printed_tags.contains("lastWords")
            || c.granted
                .iter()
                .any(|a| matches!(a, Ability::LastWords { .. }));
        if has != b {
            return false;
        }
    }
    if let Some(b) = f.has_spellboost {
        let has = c.printed_tags.contains("spellboost");
        if has != b {
            return false;
        }
    }
    if let Some(b) = f.enhanced {
        if c.flags.enhanced != b {
            return false;
        }
    }
    if f.same_cost_group == Some(true) {
        let Some(base) = _state.event_base_cost else {
            return false;
        };
        if c.base_cost != base {
            return false;
        }
        if _state.event_inst_id == Some(c.id) {
            return false;
        }
    }
    if let Some(b) = f.did_not_attack_this_turn {
        if c.flags.attacked_this_turn == b {
            return false;
        }
    }
    if let Some(b) = f.super_evolved {
        if c.super_evolved != b {
            return false;
        }
    }
    if let Some(name) = &f.not_bound {
        if let Some(refs) = _state.bindings.get(name) {
            for r in refs {
                if let BoundRef::Field { id, .. } = r {
                    if c.id == *id {
                        return false;
                    }
                }
            }
        }
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
        Amount::DistinctNames { distinct_names } => match distinct_names.as_ref() {
            Amount::EnteredThisMatch { entered_this_match } => {
                entered_ids_matching(db, state, who, entered_this_match).len() as i32
            }
            inner => eval_amount(db, state, who, source, inner),
        },
        Amount::EnteredThisMatch { entered_this_match } => {
            entered_ids_matching(db, state, who, entered_this_match)
                .into_iter()
                .map(|(_, n)| n)
                .sum()
        }
        Amount::Var { var } => vars_of(state, source, *var),
        Amount::Stat { stat } => {
            let Some(src) = source else {
                return 0;
            };
            let ts = resolve_select(db, state, who, src, &stat.of);
            let Some(t) = ts.first() else {
                return 0;
            };
            inst_stat(state, t, stat.which).unwrap_or(0)
        }
        Amount::SumHighestBaseCosts {
            sum_highest_base_costs,
        } => {
            let Some(src) = source else {
                return 0;
            };
            let mut costs: Vec<i32> =
                resolve_select(db, state, who, src, &sum_highest_base_costs.select)
                    .iter()
                    .filter_map(|t| inst_stat(state, t, StatWhich::BaseCost))
                    .collect();
            costs.sort_by(|a, b| b.cmp(a));
            costs
                .into_iter()
                .take(sum_highest_base_costs.n.max(0) as usize)
                .sum()
        }
    }
}

fn inst_stat(state: &State, t: &TargetOpt, which: StatWhich) -> Option<i32> {
    if let TargetOpt::Leader { player } = t {
        return Some(match which {
            StatWhich::Defense => state.player(*player).leader_defense,
            StatWhich::Attack | StatWhich::Cost | StatWhich::BaseCost => 0,
        });
    }
    let c = match t {
        TargetOpt::Slot { player, slot } => state.field_inst(*player, *slot)?,
        TargetOpt::Hand { player, pos } => state.player(*player).hand.get(*pos as usize)?,
        TargetOpt::Deck { player, id } => {
            state.player(*player).deck.iter().find(|c| c.id == *id)?
        }
        _ => return None,
    };
    Some(match which {
        StatWhich::Attack => c.attack,
        StatWhich::Defense => c.defense,
        StatWhich::Cost => c.cost,
        StatWhich::BaseCost => c.base_cost,
    })
}

fn card_tracks_skybound(db: &CardDb, id: CardId) -> bool {
    let Ok(card) = db.card(id) else {
        return false;
    };
    card.abilities()
        .iter()
        .any(|a| a.effects().iter().any(effect_has_skybound))
        || card.modes().iter().any(|m| match m {
            Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => {
                effects.iter().any(effect_has_skybound)
            }
            Mode::Crystallize { abilities, .. } => abilities.as_ref().is_some_and(|abs| {
                abs.iter()
                    .any(|a| a.effects().iter().any(effect_has_skybound))
            }),
        })
}

fn effect_has_skybound(e: &Effect) -> bool {
    match e {
        Effect::If {
            cond,
            then,
            else_effects,
            ..
        } => {
            cond_has_skybound(cond)
                || then.iter().any(effect_has_skybound)
                || else_effects
                    .as_ref()
                    .is_some_and(|els| els.iter().any(effect_has_skybound))
        }
        Effect::Pay { effects, .. }
        | Effect::Seq { effects, .. }
        | Effect::Repeat { effects, .. } => effects.iter().any(effect_has_skybound),
        Effect::Choose { options, .. } => options.as_ref().is_some_and(|opts| {
            opts.iter()
                .any(|o| o.effects.iter().any(effect_has_skybound))
        }),
        _ => false,
    }
}

fn cond_has_skybound(c: &Condition) -> bool {
    match c {
        Condition::SkyboundArt { .. } => true,
        Condition::All { all } => all.iter().any(cond_has_skybound),
        Condition::Any { any } => any.iter().any(cond_has_skybound),
        Condition::Not { not } => cond_has_skybound(not),
        _ => false,
    }
}

fn skybound_of(state: &State, source: Option<SourceRef>) -> i32 {
    match source {
        Some(SourceRef::Field { player, id }) => state
            .find_field(player, id)
            .and_then(|s| state.field_inst(player, s).map(|c| c.skybound))
            .unwrap_or(0),
        Some(SourceRef::Hand { player, id }) => state
            .player(player)
            .hand
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.skybound)
            .unwrap_or(0),
        Some(SourceRef::Spell { player, card }) => state
            .player(player)
            .cemetery
            .iter()
            .rev()
            .find(|c| c.card == card)
            .map(|c| c.skybound)
            .unwrap_or(0),
        _ => 0,
    }
}

fn vars_of(state: &State, source: Option<SourceRef>, key: crate::card::VarKey) -> i32 {
    match source {
        Some(SourceRef::Field { player, id } | SourceRef::Hand { player, id }) => {
            if let Some(c) = state.player(player).hand.iter().find(|c| c.id == id) {
                return *c.vars.get(&key).unwrap_or(&0);
            }
            state
                .find_field(player, id)
                .and_then(|s| state.field_inst(player, s))
                .and_then(|c| c.vars.get(&key).copied())
                .unwrap_or(0)
        }
        Some(SourceRef::Spell { player, card }) => state
            .player(player)
            .cemetery
            .iter()
            .rev()
            .find(|c| c.card == card)
            .and_then(|c| c.vars.get(&key).copied())
            .unwrap_or(0),
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
        Condition::CostEq { cost_eq } => {
            // Played instance's current cost (Severed Ties). Spells read the
            // cemetery corpse written at play (`cost = paid`).
            source_instance_cost(state, source)
                == Some(eval_amount(db, state, who, source, cost_eq))
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
        Condition::AttackingFollower { attacking_follower } => {
            state.attacking_follower == *attacking_follower
        }
        Condition::SuperEvolutionUnlocked {
            super_evolution_unlocked,
        } => {
            let p = state.player(who);
            let unlock = if p.is_second { 6 } else { 7 };
            (p.turns_taken >= unlock) == *super_evolution_unlocked
        }
        Condition::CountAtLeast { count_at_least } => {
            if let Some(src) = source {
                let ts = resolve_select(db, state, who, src, &count_at_least.select);
                let n = eval_amount(db, state, who, source, &count_at_least.n);
                if let Some(f) = &count_at_least.filter {
                    ts.iter()
                        .filter(|t| target_matches_count_filter(db, state, who, t, f))
                        .count() as i32
                        >= n
                } else {
                    ts.len() as i32 >= n
                }
            } else {
                false
            }
        }
        Condition::Did { did } => state
            .bindings
            .get(did)
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        Condition::HandHas { hand_has } => {
            let n = eval_amount(db, state, who, source, &hand_has.n);
            state
                .player(who)
                .hand
                .iter()
                .filter(|c| inst_matches_filter(state, who, c, &hand_has.filter))
                .count() as i32
                >= n
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
        Condition::LeaderDefenseLte { leader_defense_lte } => {
            state.player(who).leader_defense
                <= eval_amount(db, state, who, source, leader_defense_lte)
        }
        Condition::SkyboundArt { skybound_art } => {
            let n = eval_amount(db, state, who, source, &skybound_art.n);
            // Rulebook: gauge = current turn number (trace `turn`) + evolves
            // witnessed while this copy was in hand. `skybound` on the
            // instance is only the evolve count; the turn is added here.
            // `turns_taken` equals the round for the player who is acting.
            state.turn as i32 + skybound_of(state, source) >= n
        }
        Condition::TurnOwner { turn_owner } => match turn_owner {
            TurnOwner::Self_ => state.active == who,
            TurnOwner::Opponent => state.active != who,
        },
        Condition::FieldHas { field_has } => {
            let n = field_has
                .n
                .as_ref()
                .map(|a| eval_amount(db, state, who, source, a))
                .unwrap_or(1);
            let sides: Vec<PlayerId> = match field_has.side {
                Some(Side::Enemy) => vec![who.opponent()],
                Some(Side::Any) if WORLD_OF_GAMES_COUNTS_EITHER_SIDE => {
                    vec![who, who.opponent()]
                }
                Some(Side::Any) => vec![who],
                None | Some(Side::Ally) => vec![who],
            };
            let mut count = 0i32;
            for p in sides {
                for c in state.player(p).field.iter().flatten() {
                    if let Some(k) = field_has.kind {
                        let ok = match k {
                            FieldHasKind::Follower => c.kind == CardKind::Follower,
                            FieldHasKind::Amulet => c.kind == CardKind::Amulet,
                            FieldHasKind::Card | FieldHasKind::Character => true,
                        };
                        if !ok {
                            continue;
                        }
                    }
                    if inst_matches_filter(state, who, c, &field_has.filter) {
                        count += 1;
                    }
                }
            }
            count >= n
        }
        Condition::AmountAtLeast { amount_at_least } => {
            eval_amount(db, state, who, source, &amount_at_least.of)
                >= eval_amount(db, state, who, source, &amount_at_least.n)
        }
        Condition::PlayedBaseCostsThisMatch {
            played_base_costs_this_match,
        } => {
            let have: std::collections::BTreeSet<i32> = state
                .player(who)
                .played_base_costs_this_match
                .iter()
                .copied()
                .collect();
            played_base_costs_this_match
                .iter()
                .all(|c| have.contains(c))
        }
        Condition::BoundHas { bound_has } => {
            let Some(refs) = state.bindings.get(&bound_has.ref_name) else {
                return false;
            };
            refs.iter().any(|r| match r {
                BoundRef::Field {
                    player, card, kind, ..
                } => {
                    if let Some(side) = bound_has.side {
                        let want = match side {
                            Side::Ally => who,
                            Side::Enemy => who.opponent(),
                            Side::Any => *player,
                        };
                        if *player != want && side != Side::Any {
                            return false;
                        }
                    }
                    let Ok(def) = db.card(*card) else {
                        return false;
                    };
                    let mut inst = CardInstance::from_card(def, 0);
                    inst.kind = *kind;
                    inst_matches_filter(state, *player, &inst, &bound_has.filter)
                }
                BoundRef::Card(id) => {
                    let Ok(def) = db.card(*id) else {
                        return false;
                    };
                    let inst = CardInstance::from_card(def, 0);
                    inst_matches_filter(state, who, &inst, &bound_has.filter)
                }
                _ => false,
            })
        }
        Condition::EvolvedCountAtLeast {
            evolved_count_at_least,
        } => {
            state.player(who).evolves_used
                >= eval_amount(db, state, who, source, &evolved_count_at_least.n)
        }
        Condition::VarAtLeast { var_at_least } => {
            let n = eval_amount(db, state, who, source, &var_at_least.n);
            vars_of(state, source, var_at_least.key) >= n
        }
        Condition::HandSameCostAtLeast {
            hand_same_cost_at_least,
        } => {
            let n = eval_amount(db, state, who, source, &hand_same_cost_at_least.n);
            let mut counts: std::collections::BTreeMap<i32, i32> =
                std::collections::BTreeMap::new();
            for c in &state.player(who).hand {
                *counts.entry(c.cost).or_insert(0) += 1;
            }
            counts.values().any(|&c| c >= n)
        }
        Condition::DeckHasNoDuplicates {
            deck_has_no_duplicates,
        } => {
            let mut seen = std::collections::BTreeSet::new();
            let unique = state.player(who).deck.iter().all(|c| seen.insert(c.card));
            unique == *deck_has_no_duplicates
        }
        Condition::AttackingLeader { attacking_leader } => {
            state.attack_target_is_leader == *attacking_leader
        }
        Condition::EnterCountAtLeast {
            enter_count_at_least,
        } => {
            let n = eval_amount(db, state, who, source, &enter_count_at_least.n);
            let side = match enter_count_at_least.side {
                Some(Side::Enemy) => who.opponent(),
                _ => who,
            };
            let mut count = state
                .player(side)
                .enter_counts
                .get(&enter_count_at_least.card)
                .copied()
                .unwrap_or(0);
            if enter_count_at_least.other == Some(true) {
                count = count.saturating_sub(1);
            }
            count >= n
        }
    }
}

fn entered_ids_matching(
    db: &CardDb,
    state: &State,
    who: PlayerId,
    filter: &Filter,
) -> Vec<(CardId, i32)> {
    state
        .player(who)
        .enter_counts
        .iter()
        .filter(|(id, _)| {
            let Ok(card) = db.card(**id) else {
                return false;
            };
            let inst = CardInstance::from_card(card, 0);
            inst_matches_filter(state, who, &inst, filter)
        })
        .map(|(id, n)| (*id, *n))
        .collect()
}

enum CapturedTarget {
    Field { player: PlayerId, id: u32 },
    Hand { player: PlayerId, id: u32 },
    Keep(TargetOpt),
}

fn capture_targets(state: &State, ts: &[TargetOpt]) -> Vec<CapturedTarget> {
    ts.iter()
        .filter_map(|t| match t {
            TargetOpt::Slot { player, slot } => {
                state
                    .field_inst(*player, *slot)
                    .map(|c| CapturedTarget::Field {
                        player: *player,
                        id: c.id,
                    })
            }
            TargetOpt::Hand { player, pos } => {
                if let Some(c) = state.player(*player).hand.get(*pos as usize) {
                    Some(CapturedTarget::Hand {
                        player: *player,
                        id: c.id,
                    })
                } else {
                    Some(CapturedTarget::Keep(t.clone()))
                }
            }
            other => Some(CapturedTarget::Keep(other.clone())),
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
        CapturedTarget::Hand { player, id } => state
            .player(*player)
            .hand
            .iter()
            .position(|c| c.id == *id)
            .map(|pos| TargetOpt::Hand {
                player: *player,
                pos: pos as u8,
            }),
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
    if let TargetOpt::Leader { player } = t {
        return match order {
            Some(OrderBy::Defense) | None => state.player(*player).leader_defense,
            Some(OrderBy::Attack) => 0,
            Some(OrderBy::Cost) | Some(OrderBy::BaseCost) => 0,
        };
    }
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
    let tied: Vec<TargetOpt> = cands
        .into_iter()
        .filter(|t| order_key(state, t, p.order_by) == best)
        .collect();
    // `lowest` = all with the lowest (Tyrannical Fists ties hit both leaders).
    if p.pick == PoolPick::Lowest {
        tied
    } else {
        tied.into_iter().take(1).collect()
    }
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
    if !highest {
        return Ok(tied);
    }
    random_pool_apply(state, tied, 1, false)
}

fn board_card_survives(c: &CardInstance) -> bool {
    match c.kind {
        CardKind::Follower => c.defense > 0,
        CardKind::Amulet => c.countdown.is_none_or(|n| n > 0),
        CardKind::Spell => true,
    }
}

/// 0-based index among surviving cards on that player's field (E36).
/// Followers at 0 defense / marked for destruction and amulets at
/// countdown 0 are skipped; order of the rest is preserved.
/// `skip` is slots already chosen in this `randomDistinct` wave — they are
/// gone at the next roll even though destroy has not applied yet.
fn surviving_board_index(
    state: &State,
    player: PlayerId,
    slot: u8,
    skip: &[(PlayerId, u8)],
) -> Option<u8> {
    let mut i = 0u8;
    for (si, cell) in state.player(player).field.iter().enumerate() {
        let Some(c) = cell else {
            continue;
        };
        if skip.iter().any(|(p, s)| *p == player && *s == si as u8) {
            continue;
        }
        if !board_card_survives(c) {
            continue;
        }
        if si == slot as usize {
            return Some(i);
        }
        i = i.saturating_add(1);
    }
    None
}

fn surviving_slot_key(
    state: &State,
    player: PlayerId,
    slot: u8,
    skip: &[(PlayerId, u8)],
) -> String {
    match surviving_board_index(state, player, slot, skip) {
        Some(i) => format!("slot:{i}"),
        None => format!("slot:{slot}"),
    }
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
    let deck_search = cands
        .iter()
        .all(|t| matches!(t, TargetOpt::Card(_) | TargetOpt::Deck { .. }));
    let mut left = cands;
    let mut out = Vec::new();
    let mut skipped: Vec<(PlayerId, u8)> = Vec::new();
    for _ in 0..n {
        if left.is_empty() {
            break;
        }
        let multi_player = {
            let mut seen = [false, false];
            for t in &left {
                let p = match t {
                    TargetOpt::Slot { player, .. } | TargetOpt::Leader { player } => Some(*player),
                    _ => None,
                };
                if let Some(p) = p {
                    seen[p.idx()] = true;
                }
            }
            seen[0] && seen[1]
        };
        let mut keys: Vec<String> = left
            .iter()
            .map(|t| match t {
                TargetOpt::Slot { player, slot } => {
                    let k = surviving_slot_key(state, *player, *slot, &skipped);
                    if multi_player {
                        format!("{}:{k}", player.as_str())
                    } else {
                        k
                    }
                }
                TargetOpt::Leader { player } => {
                    if multi_player {
                        format!("leader:{}", player.as_str())
                    } else {
                        "leader".into()
                    }
                }
                TargetOpt::Card(c) => c.as_str(),
                TargetOpt::Deck { player, id } => state
                    .player(*player)
                    .deck
                    .iter()
                    .find(|c| c.id == *id)
                    .map(|c| c.card.as_str())
                    .unwrap_or_default(),
                TargetOpt::Hand { pos, .. } => format!("hand:{pos}"),
                TargetOpt::Mode(m) => format!("mode:{m}"),
            })
            .collect();
        alias_scripted_raw_slot(state, &left, &mut keys);
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
        let picked = if distinct {
            left.remove(i)
        } else {
            left[i].clone()
        };
        if distinct {
            if let TargetOpt::Slot { player, slot } = &picked {
                skipped.push((*player, *slot));
            }
            if let TargetOpt::Card(id) = &picked {
                left.retain(|t| !matches!(t, TargetOpt::Card(x) if x == id));
            }
        }
        out.push(picked);
    }
    Ok(out)
}

/// Scripted `random_target` matches survivor-index keys first (E36). If the
/// recorded `chose` is not among those keys, a raw field slot is accepted as
/// an alias when that label is not already a survivor key of another
/// candidate — M1 ramp traces numbered by raw slot. Live RNG is unchanged
/// (one key per candidate; expanding would bias the roll).
fn alias_scripted_raw_slot(state: &State, left: &[TargetOpt], keys: &mut [String]) {
    if !state.rng.is_scripted() || state.rng.peek_what() != Some(PickWhat::RandomTarget) {
        return;
    }
    let Some(chose) = state.rng.peek_chose_key() else {
        return;
    };
    if keys.iter().any(|k| k == &chose) {
        return;
    }
    for (i, t) in left.iter().enumerate() {
        let TargetOpt::Slot { slot, .. } = t else {
            continue;
        };
        let raw = format!("slot:{slot}");
        if raw == chose && !keys.iter().any(|k| k == &raw) {
            keys[i] = raw;
            return;
        }
    }
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
