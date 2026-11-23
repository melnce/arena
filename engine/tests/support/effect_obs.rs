//! Strict per-effect observation for mode_sweep random / autopick gates (tests only).

use std::collections::BTreeMap;

use arena_engine::card::{Amount, CardSource, Effect, RefPick, Selector, Side, Traits, Zone};
use arena_engine::{CardDb, CardId, PlayerId, State};

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldInstSnap {
    pub inst_id: u32,
    pub card: CardId,
    pub attack: i32,
    pub defense: i32,
    pub evolved: bool,
    pub super_evolved: bool,
    pub trait_sig: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandInstSnap {
    pub inst_id: u32,
    pub card: CardId,
    pub cost: i32,
}

#[derive(Clone, Debug, Default)]
pub struct DeepSnap {
    pub ally_field: BTreeMap<u32, FieldInstSnap>,
    pub enemy_field: BTreeMap<u32, FieldInstSnap>,
    pub ally_hand: Vec<HandInstSnap>,
    pub ally_leader_hp: i32,
    pub enemy_leader_hp: i32,
    pub ally_pp: i32,
    pub ally_ep: i32,
    pub ally_sep: i32,
    pub ally_hand_len: usize,
    pub ally_deck_len: usize,
    pub ally_cemetery_len: usize,
    pub faith: i32,
    pub earth: i32,
    pub shadows: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayCtx {
    pub me: PlayerId,
    pub played_card: Option<CardId>,
    pub play_cost: i32,
    pub played_from_hand: bool,
}

pub fn trait_sig(t: &Traits) -> String {
    format!(
        "w:{} b:{} r:{} s:{} a:{} i:{} d:{} br:{}",
        t.ward == Some(true),
        t.bane == Some(true),
        t.rush == Some(true),
        t.storm == Some(true),
        t.ambush == Some(true),
        t.intimidate == Some(true),
        t.drain == Some(true),
        t.barrier == Some(true),
    )
}

fn field_snap(st: &State, who: PlayerId) -> BTreeMap<u32, FieldInstSnap> {
    st.player(who)
        .field
        .iter()
        .flatten()
        .map(|c| {
            (
                c.id,
                FieldInstSnap {
                    inst_id: c.id,
                    card: c.card,
                    attack: c.attack,
                    defense: c.defense,
                    evolved: c.evolved,
                    super_evolved: c.super_evolved,
                    trait_sig: trait_sig(&c.traits),
                },
            )
        })
        .collect()
}

pub fn deep_snap(st: &State, me: PlayerId) -> DeepSnap {
    let p = st.player(me);
    let opp = me.opponent();
    DeepSnap {
        ally_field: field_snap(st, me),
        enemy_field: field_snap(st, opp),
        ally_hand: p
            .hand
            .iter()
            .map(|c| HandInstSnap {
                inst_id: c.id,
                card: c.card,
                cost: c.cost,
            })
            .collect(),
        ally_leader_hp: p.leader_defense,
        enemy_leader_hp: st.player(opp).leader_defense,
        ally_pp: p.pp,
        ally_ep: p.ep,
        ally_sep: p.sep,
        ally_hand_len: p.hand.len(),
        ally_deck_len: p.deck.len(),
        ally_cemetery_len: p.cemetery.len(),
        faith: p.faith,
        earth: p.earth,
        shadows: p.shadows,
    }
}

pub fn hand_growth_excl_play(before: &DeepSnap, after: &DeepSnap, ctx: PlayCtx) -> i32 {
    let adj = if ctx.played_from_hand { 1i32 } else { 0 };
    after.ally_hand_len as i32 - before.ally_hand_len as i32 + adj
}

pub fn pp_delta_excl_play(before: &DeepSnap, after: &DeepSnap, ctx: PlayCtx) -> i32 {
    after.ally_pp - before.ally_pp + ctx.play_cost
}

fn new_ally_field(before: &DeepSnap, after: &DeepSnap) -> Vec<FieldInstSnap> {
    after
        .ally_field
        .values()
        .filter(|f| !before.ally_field.contains_key(&f.inst_id))
        .cloned()
        .collect()
}

fn card_named_id(_db: &CardDb, src: &CardSource) -> Option<CardId> {
    match src {
        CardSource::Named { named } => Some(*named),
        _ => None,
    }
}

fn amount_min(a: &Amount) -> i32 {
    match a {
        Amount::Int(n) => *n,
        _ => 1,
    }
}

fn selector_is_self(select: &Selector) -> bool {
    matches!(
        select,
        Selector::Ref(r) if r.pick == RefPick::Self_ || r.pick == RefPick::Entering
    )
}

#[allow(clippy::too_many_lines)]
pub fn effect_observed_strict(
    db: &CardDb,
    before: &DeepSnap,
    after: &DeepSnap,
    ctx: PlayCtx,
    e: &Effect,
) -> Result<(), String> {
    match e {
        Effect::Seq { effects, .. } | Effect::Repeat { effects, .. } => {
            for x in effects {
                effect_observed_strict(db, before, after, ctx, x)?;
            }
            Ok(())
        }
        Effect::Sequence { steps, .. } => {
            for s in steps {
                for x in &s.effects {
                    effect_observed_strict(db, before, after, ctx, x)?;
                }
            }
            Ok(())
        }
        Effect::If {
            then, else_effects, ..
        } => {
            for x in then {
                effect_observed_strict(db, before, after, ctx, x)?;
            }
            if let Some(els) = else_effects {
                for x in els {
                    effect_observed_strict(db, before, after, ctx, x)?;
                }
            }
            Ok(())
        }
        Effect::Draw { count, .. } => {
            let need = amount_min(count);
            let got = hand_growth_excl_play(before, after, ctx);
            if got >= need {
                Ok(())
            } else {
                Err(format!("draw: hand grew by {got}, need >= {need}"))
            }
        }
        Effect::AddToHand { card, count, .. } => {
            let named = card_named_id(db, card).map(|c| c.as_str().to_string());
            let grew = hand_growth_excl_play(before, after, ctx);
            let need = amount_min(count);
            if grew >= need {
                if let Some(id) = named {
                    let has = after.ally_hand.iter().any(|h| h.card.as_str() == id);
                    if has {
                        return Ok(());
                    }
                    return Err(format!("addToHand: hand grew but missing {id}"));
                }
                return Ok(());
            }
            Err(format!("addToHand: hand grew by {grew}, need >= {need}"))
        }
        Effect::Summon { card, count, .. } => {
            let want = card_named_id(db, card);
            let newbies = new_ally_field(before, after);
            let need = amount_min(count) as usize;
            let n = newbies
                .iter()
                .filter(|f| want.is_none_or(|w| f.card == w))
                .count();
            if n >= need {
                Ok(())
            } else {
                Err(format!("summon: {n} new field cards, need >= {need}"))
            }
        }
        Effect::Damage { select, .. } => {
            if let Selector::Pool(p) = select {
                if matches!(p.side, Side::Enemy | Side::Any) {
                    if p.zone == Zone::Leader && after.enemy_leader_hp < before.enemy_leader_hp {
                        return Ok(());
                    }
                    if p.zone == Zone::Field
                        && matches!(
                            p.kind,
                            arena_engine::card::SelectorKind::Follower
                                | arena_engine::card::SelectorKind::Card
                        )
                    {
                        let damaged = before.enemy_field.iter().any(|(id, b)| {
                            after
                                .enemy_field
                                .get(id)
                                .is_some_and(|a| a.defense < b.defense)
                        });
                        let removed = after.enemy_field.len() < before.enemy_field.len();
                        if damaged || removed {
                            return Ok(());
                        }
                    }
                }
                if matches!(p.side, Side::Ally | Side::Any)
                    && p.zone == Zone::Leader
                    && after.ally_leader_hp < before.ally_leader_hp
                {
                    return Ok(());
                }
            }
            Err("damage: no observable HP/field change on enemy".into())
        }
        Effect::Destroy { select, .. } | Effect::Banish { select, .. } => {
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Field
                    && matches!(p.side, Side::Enemy | Side::Any)
                    && after.enemy_field.len() < before.enemy_field.len()
                {
                    return Ok(());
                }
            }
            if matches!(select, Selector::Bound { .. })
                && after.enemy_field.len() < before.enemy_field.len()
            {
                return Ok(());
            }
            Err("destroy/banish: enemy field unchanged".into())
        }
        Effect::Restore { select, .. } => {
            if let Selector::Pool(p) = select {
                if p.side == Side::Ally
                    && p.zone == Zone::Leader
                    && after.ally_leader_hp > before.ally_leader_hp
                {
                    return Ok(());
                }
            }
            Err("restore: ally leader HP unchanged".into())
        }
        Effect::Pp { action, .. } => {
            use arena_engine::card::PpAction;
            match action {
                PpAction::Recover => {
                    if pp_delta_excl_play(before, after, ctx) > 0 {
                        Ok(())
                    } else {
                        Err(format!(
                            "pp recover: delta excl play = {}",
                            pp_delta_excl_play(before, after, ctx)
                        ))
                    }
                }
                _ => Err(format!("pp op {action:?}: not checked")),
            }
        }
        Effect::Buff { select, .. } => {
            if matches!(select, Selector::Bound { .. }) {
                let newbies = new_ally_field(before, after);
                if newbies.iter().any(|f| {
                    before
                        .ally_field
                        .get(&f.inst_id)
                        .is_none_or(|b| f.attack > b.attack || f.defense > b.defense)
                }) {
                    return Ok(());
                }
                return Err("buff bound: new summon not buffed".into());
            }
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Field
                    && matches!(p.side, Side::Ally | Side::Any)
                    && before.ally_field.iter().any(|(id, b)| {
                        after
                            .ally_field
                            .get(id)
                            .is_some_and(|a| a.attack > b.attack || a.defense > b.defense)
                    })
                {
                    return Ok(());
                }
            }
            if selector_is_self(select) {
                if let Some(pid) = ctx.played_card {
                    if after.ally_field.values().any(|f| {
                        f.card == pid
                            && before
                                .ally_field
                                .get(&f.inst_id)
                                .is_none_or(|b| f.attack > b.attack || f.defense > b.defense)
                    }) {
                        return Ok(());
                    }
                }
            }
            Err("buff: no stat change on pre-existing ally field".into())
        }
        Effect::GrantTraits { select, .. } => {
            let changed_inst = |b: &FieldInstSnap, a: &FieldInstSnap| a.trait_sig != b.trait_sig;
            if let Selector::Bound { .. } = select {
                for f in new_ally_field(before, after) {
                    if ctx.played_card.is_none_or(|pid| f.card == pid) {
                        return Ok(());
                    }
                }
            }
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Field && matches!(p.side, Side::Ally | Side::Any) {
                    for (id, b) in &before.ally_field {
                        if let Some(a) = after.ally_field.get(id) {
                            if changed_inst(b, a) {
                                return Ok(());
                            }
                        }
                    }
                }
            }
            if selector_is_self(select) {
                if let Some(pid) = ctx.played_card {
                    for (id, b) in &before.ally_field {
                        if let Some(a) = after.ally_field.get(id) {
                            if a.card == pid && changed_inst(b, a) {
                                return Ok(());
                            }
                        }
                    }
                    if let Ok(base) = db.card(pid) {
                        let base_sig = base
                            .traits()
                            .map(trait_sig)
                            .unwrap_or_else(|| trait_sig(&Traits::default()));
                        for f in new_ally_field(before, after) {
                            if f.card == pid && f.trait_sig != base_sig {
                                return Ok(());
                            }
                        }
                        if after
                            .ally_field
                            .values()
                            .any(|f| f.card == pid && f.trait_sig != base_sig)
                        {
                            return Ok(());
                        }
                    }
                }
            }
            Err("grantTraits: traits unchanged".into())
        }
        Effect::Cost { select, .. } => {
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Hand {
                    let mut saw = false;
                    for hb in &before.ally_hand {
                        if let Some(ha) = after.ally_hand.iter().find(|x| x.inst_id == hb.inst_id) {
                            if ctx.played_card == Some(hb.card)
                                && !after.ally_hand.iter().any(|x| x.inst_id == hb.inst_id)
                            {
                                continue;
                            }
                            if ha.cost < hb.cost {
                                saw = true;
                            }
                        }
                    }
                    if saw {
                        return Ok(());
                    }
                }
            }
            Err("cost: no hand cost reduction on surviving cards".into())
        }
        Effect::Evolve {
            select,
            super_evolve,
            ..
        } => {
            let want_super = *super_evolve;
            let evolved = |b: &FieldInstSnap, a: &FieldInstSnap| {
                if want_super {
                    a.super_evolved && !b.super_evolved
                } else {
                    a.evolved && !b.evolved
                }
            };
            if selector_is_self(select) {
                for (id, b) in &before.ally_field {
                    if let Some(a) = after.ally_field.get(id) {
                        if evolved(b, a) {
                            return Ok(());
                        }
                    }
                }
                for f in new_ally_field(before, after) {
                    if (want_super && f.super_evolved) || (!want_super && f.evolved) {
                        return Ok(());
                    }
                }
            }
            Err("evolve: no evolved follower".into())
        }
        Effect::Ep { .. } => {
            if after.ally_ep > before.ally_ep || after.ally_sep > before.ally_sep {
                Ok(())
            } else {
                Err("ep: unchanged".into())
            }
        }
        Effect::Counter { .. } => {
            if after.faith != before.faith
                || after.earth != before.earth
                || after.shadows != before.shadows
            {
                Ok(())
            } else {
                Err("counter: player counters unchanged".into())
            }
        }
        Effect::Replicate { .. } => {
            if after.enemy_leader_hp < before.enemy_leader_hp
                || after.enemy_field.len() < before.enemy_field.len()
                || pp_delta_excl_play(before, after, ctx) > 0
                || hand_growth_excl_play(before, after, ctx) > 0
            {
                Ok(())
            } else {
                let extra = after
                    .ally_field
                    .values()
                    .filter(|f| {
                        before
                            .ally_field
                            .get(&f.inst_id)
                            .is_none_or(|b| f.attack > b.attack || f.defense > b.defense)
                    })
                    .count();
                if extra > 0 {
                    Ok(())
                } else {
                    Err("replicate: no nested fanfare footprint".into())
                }
            }
        }
        Effect::RemoveCrests { .. } => Ok(()),
        Effect::Choose { .. } => Ok(()),
        _ => Err(format!(
            "effect {:?}: no strict observer (add harness or justify)",
            e.printed().unwrap_or("?")
        )),
    }
}

pub fn all_effects_observed(
    db: &CardDb,
    before: &DeepSnap,
    after: &DeepSnap,
    ctx: PlayCtx,
    effects: &[Effect],
) -> Result<(), String> {
    for e in effects {
        effect_observed_strict(db, before, after, ctx, e)?;
    }
    Ok(())
}

pub fn fanfare_player_options(db: &CardDb, card_key: &str) -> Vec<u8> {
    let Ok(card) = db.card(cid(card_key)) else {
        return vec![];
    };
    let Some(ab) = card
        .abilities()
        .iter()
        .find(|a| matches!(a, Ability::Fanfare { .. }))
    else {
        return vec![];
    };
    let mut raw = Vec::new();
    let mut ord = 0u8;
    super::collect_chooses_in_effects(ab.effects(), &mut raw, &mut ord);
    let mut out: Vec<u8> = raw
        .into_iter()
        .filter(|(co, _, by, _)| *co == 0 && *by == super::ChooseBy::Player)
        .map(|(_, oi, _, _)| oi)
        .collect();
    out.sort();
    out.dedup();
    out
}
