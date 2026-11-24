//! Strict per-effect observation for mode_sweep random / autopick gates (tests only).

use std::collections::BTreeMap;

use arena_engine::card::{
    Amount, CardSource, Effect, PayResource, PoolPick, RefPick, Selector, Side, Traits, Zone,
};
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
    pub spellboost_count: i32,
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
    pub ally_crest_ids: Vec<String>,
    pub ally_clue_total: i32,
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
        "w:{} b:{} r:{} s:{} a:{} i:{} d:{} br:{} apt:{}",
        t.ward == Some(true),
        t.bane == Some(true),
        t.rush == Some(true),
        t.storm == Some(true),
        t.ambush == Some(true),
        t.intimidate == Some(true),
        t.drain == Some(true),
        t.barrier == Some(true),
        t.attacks_per_turn.unwrap_or(1),
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
                spellboost_count: c.spellboost_count,
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
        ally_crest_ids: p.crests.iter().map(|c| c.id.clone()).collect(),
        ally_clue_total: p.field.iter().flatten().map(|c| c.clue_count).sum(),
    }
}

pub fn hand_all_cost_delta(db: &CardDb, before: &DeepSnap, after: &DeepSnap, delta: i32) -> bool {
    for hb in &before.ally_hand {
        let Some(ha) = after.ally_hand.iter().find(|x| x.inst_id == hb.inst_id) else {
            return false;
        };
        let expect = (hb.cost + delta).max(0);
        if ha.cost != expect {
            return false;
        }
    }
    for ha in &after.ally_hand {
        if before.ally_hand.iter().any(|hb| hb.inst_id == ha.inst_id) {
            continue;
        }
        if let Ok(card) = db.card(ha.card) {
            let base = card.cost().max(0);
            // Turn-start draw is after the SoT queue (`ContinueTurnStart` step 8).
            let expect = base;
            if ha.cost != expect {
                return false;
            }
        }
    }
    true
}

fn hand_cost_changed(
    db: &CardDb,
    before: &DeepSnap,
    after: &DeepSnap,
    want_increase: bool,
) -> bool {
    for hb in &before.ally_hand {
        let Some(ha) = after.ally_hand.iter().find(|x| x.inst_id == hb.inst_id) else {
            continue;
        };
        if want_increase {
            if ha.cost > hb.cost {
                return true;
            }
        } else if ha.cost < hb.cost {
            return true;
        }
    }
    for ha in &after.ally_hand {
        if before.ally_hand.iter().any(|hb| hb.inst_id == ha.inst_id) {
            continue;
        }
        if let Ok(card) = db.card(ha.card) {
            let base = card.cost().max(0);
            if !want_increase && ha.cost < base {
                return true;
            }
            if want_increase && ha.cost > base {
                return true;
            }
        }
    }
    false
}

fn resource_before(pay: &PayResource, snap: &DeepSnap) -> i32 {
    match pay {
        PayResource::Earth => snap.earth,
        PayResource::Faith => snap.faith,
        PayResource::Shadows => snap.shadows,
        PayResource::Pp => snap.ally_pp,
        PayResource::Clues => snap.ally_clue_total,
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

fn amount_signed(a: &Option<Amount>) -> Option<i32> {
    match a.as_ref()? {
        Amount::Int(n) => Some(*n),
        _ => None,
    }
}

fn buff_pool_ally_ok(
    before: &DeepSnap,
    after: &DeepSnap,
    da: Option<i32>,
    dd: Option<i32>,
) -> bool {
    let want_atk_up = da.is_some_and(|x| x > 0);
    let want_def_up = dd.is_some_and(|x| x > 0);
    let want_atk_down = da.is_some_and(|x| x < 0);
    let want_def_down = dd.is_some_and(|x| x < 0);
    let mut atk_up = false;
    let mut def_up = false;
    let mut atk_down = false;
    let mut def_down = false;
    let removed = after.ally_field.len() < before.ally_field.len();
    for (id, b) in &before.ally_field {
        if let Some(a) = after.ally_field.get(id) {
            if want_atk_up && a.attack > b.attack {
                atk_up = true;
            }
            if want_def_up && a.defense > b.defense {
                def_up = true;
            }
            if want_atk_down && a.attack < b.attack {
                atk_down = true;
            }
            if want_def_down && a.defense < b.defense {
                def_down = true;
            }
        }
    }
    if want_atk_up && !atk_up {
        return false;
    }
    if want_def_up && !def_up {
        return false;
    }
    if (want_atk_down || want_def_down) && !(atk_down || def_down || removed) {
        return false;
    }
    if !want_atk_up && !want_def_up && !want_atk_down && !want_def_down {
        return before.ally_field.iter().any(|(id, b)| {
            after
                .ally_field
                .get(id)
                .is_some_and(|a| a.attack != b.attack || a.defense != b.defense)
        });
    }
    true
}

fn buff_pool_enemy_ok(
    before: &DeepSnap,
    after: &DeepSnap,
    da: Option<i32>,
    dd: Option<i32>,
) -> bool {
    let want_down = da.is_some_and(|x| x < 0) || dd.is_some_and(|x| x < 0);
    if !want_down {
        return before.enemy_field.iter().any(|(id, b)| {
            after
                .enemy_field
                .get(id)
                .is_some_and(|a| a.attack > b.attack || a.defense > b.defense)
        });
    }
    let debuffed = before.enemy_field.iter().any(|(id, b)| {
        after.enemy_field.get(id).is_some_and(|a| {
            (da.is_some_and(|x| x < 0) && a.attack < b.attack)
                || (dd.is_some_and(|x| x < 0) && a.defense < b.defense)
        })
    });
    let removed = after.enemy_field.len() < before.enemy_field.len();
    debuffed || removed
}

const EVOLVE_STAT_BONUS: i32 = 2;

fn buff_expected_new_follower(
    db: &CardDb,
    f: &FieldInstSnap,
    da: Option<i32>,
    dd: Option<i32>,
) -> bool {
    let Ok(card) = db.card(f.card) else {
        return false;
    };
    let pa = card.attack().max(0);
    let pd = card.defense().max(0);
    let ea = da.unwrap_or(0);
    let ed = dd.unwrap_or(0);
    if ea > 0 && f.attack != pa + ea {
        return false;
    }
    if ea < 0 && f.attack > pa + ea {
        return false;
    }
    if ed > 0 && f.defense != pd + ed {
        return false;
    }
    if ed < 0 && f.defense > (pd + ed).max(0) {
        return false;
    }
    ea != 0 || ed != 0
}

fn buff_expected_bound_pair(
    b: &FieldInstSnap,
    a: &FieldInstSnap,
    da: Option<i32>,
    dd: Option<i32>,
) -> bool {
    let ea = da.unwrap_or(0);
    let ed = dd.unwrap_or(0);
    let (evo_a, evo_d) = if !b.evolved && a.evolved {
        (EVOLVE_STAT_BONUS, EVOLVE_STAT_BONUS)
    } else {
        (0, 0)
    };
    let exp_atk = b.attack + evo_a + ea;
    let exp_def = b.defense + evo_d + ed;
    if ea > 0 || ed > 0 {
        return a.attack == exp_atk && a.defense == exp_def;
    }
    if ea < 0 || ed < 0 {
        return a.attack <= exp_atk
            && a.defense <= exp_def
            && (a.attack < b.attack || a.defense < b.defense);
    }
    false
}

fn buff_bound_ally_ok(
    db: &CardDb,
    before: &DeepSnap,
    after: &DeepSnap,
    da: Option<i32>,
    dd: Option<i32>,
) -> bool {
    for (id, b) in &before.ally_field {
        if let Some(a) = after.ally_field.get(id) {
            if buff_expected_bound_pair(b, a, da, dd) {
                return true;
            }
        }
    }
    new_ally_field(before, after)
        .iter()
        .any(|f| buff_expected_new_follower(db, f, da, dd))
}

fn buff_self_ok(
    db: &CardDb,
    before: &DeepSnap,
    after: &DeepSnap,
    pid: CardId,
    da: Option<i32>,
    dd: Option<i32>,
) -> bool {
    for (id, b) in &before.ally_field {
        if let Some(a) = after.ally_field.get(id) {
            if a.card != pid {
                continue;
            }
            if da.is_some_and(|x| x > 0) && a.attack <= b.attack {
                return false;
            }
            if dd.is_some_and(|x| x > 0) && a.defense <= b.defense {
                return false;
            }
            if (da.is_some_and(|x| x < 0) || dd.is_some_and(|x| x < 0))
                && a.attack >= b.attack
                && a.defense >= b.defense
            {
                return false;
            }
            return true;
        }
    }
    after
        .ally_field
        .values()
        .any(|f| f.card == pid && buff_expected_new_follower(db, f, da, dd))
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
            if matches!(select, Selector::Pool(p) if p.zone == Zone::Leader && matches!(p.side, Side::Ally | Side::Any))
                && after.ally_leader_hp < before.ally_leader_hp
            {
                return Ok(());
            }
            Err("damage: no observable HP/field change".into())
        }
        Effect::Destroy { select, .. } | Effect::Banish { select, .. } => {
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Field {
                    if matches!(p.side, Side::Enemy | Side::Any)
                        && after.enemy_field.len() < before.enemy_field.len()
                    {
                        return Ok(());
                    }
                    if matches!(p.side, Side::Ally | Side::Any)
                        && after.ally_field.len() < before.ally_field.len()
                    {
                        return Ok(());
                    }
                }
            }
            if matches!(select, Selector::Bound { .. })
                && (after.enemy_field.len() < before.enemy_field.len()
                    || after.ally_field.len() < before.ally_field.len())
            {
                return Ok(());
            }
            Err("destroy/banish: field unchanged".into())
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
        Effect::Buff {
            select,
            attack,
            defense,
            ..
        } => {
            let da = amount_signed(attack);
            let dd = amount_signed(defense);
            if attack.is_some() && da.is_none() || defense.is_some() && dd.is_none() {
                return Err("buff: non-Int amount (list option in harness)".into());
            }
            if matches!(select, Selector::Bound { .. }) {
                if buff_bound_ally_ok(db, before, after, da, dd) {
                    return Ok(());
                }
                return Err("buff bound: wrong direction".into());
            }
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Field && matches!(p.side, Side::Enemy | Side::Any) {
                    if buff_pool_enemy_ok(before, after, da, dd) {
                        return Ok(());
                    }
                    return Err("buff enemy: wrong direction".into());
                }
                if p.zone == Zone::Field && matches!(p.side, Side::Ally | Side::Any) {
                    if buff_pool_ally_ok(before, after, da, dd) {
                        return Ok(());
                    }
                    return Err("buff ally: wrong direction".into());
                }
            }
            if selector_is_self(select) {
                if let Some(pid) = ctx.played_card {
                    if buff_self_ok(db, before, after, pid, da, dd) {
                        return Ok(());
                    }
                }
            }
            Err("buff: no stat change in expected direction".into())
        }
        Effect::GrantTraits { select, .. } => {
            let changed_inst = |b: &FieldInstSnap, a: &FieldInstSnap| a.trait_sig != b.trait_sig;
            if let Selector::Bound { .. } = select {
                for f in new_ally_field(before, after) {
                    if let Ok(base) = db.card(f.card) {
                        let base_sig = base
                            .traits()
                            .map(trait_sig)
                            .unwrap_or_else(|| trait_sig(&Traits::default()));
                        if f.trait_sig != base_sig {
                            return Ok(());
                        }
                    } else if f.trait_sig != trait_sig(&Traits::default()) {
                        return Ok(());
                    }
                }
                for (id, b) in &before.ally_field {
                    if let Some(a) = after.ally_field.get(id) {
                        if changed_inst(b, a) {
                            return Ok(());
                        }
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
        Effect::Cost { select, delta, .. } => {
            let Some(d) = delta.as_ref().and_then(|a| amount_signed(&Some(a.clone()))) else {
                return Err("cost: non-Int delta or set-only (list option in harness)".into());
            };
            if d == 0 {
                return Err("cost: zero delta".into());
            }
            let want_increase = d > 0;
            if matches!(select, Selector::Bound { .. }) || matches!(select, Selector::Ref(_)) {
                if hand_cost_changed(db, before, after, want_increase) {
                    return Ok(());
                }
                return Err("cost: bound/ref hand wrong direction".into());
            }
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Hand {
                    let ok = if matches!(p.pick, PoolPick::All) {
                        hand_all_cost_delta(db, before, after, d)
                    } else {
                        hand_cost_changed(db, before, after, want_increase)
                    };
                    if ok {
                        return Ok(());
                    }
                }
            }
            Err("cost: no hand cost change in expected direction".into())
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
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Field && matches!(p.side, Side::Ally | Side::Any) {
                    for (id, b) in &before.ally_field {
                        if let Some(a) = after.ally_field.get(id) {
                            if evolved(b, a) {
                                return Ok(());
                            }
                        }
                    }
                }
            }
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
        Effect::RemoveCrests { select, .. } => {
            if let Selector::Pool(p) = select {
                if p.zone == Zone::Crests
                    && after.ally_crest_ids.len() < before.ally_crest_ids.len()
                {
                    return Ok(());
                }
            }
            if after.ally_crest_ids != before.ally_crest_ids {
                Ok(())
            } else {
                Err("removeCrests: crest area unchanged".into())
            }
        }
        Effect::SpellboostHand { times, .. } => {
            let need = amount_min(times);
            let boosted: i32 = before
                .ally_hand
                .iter()
                .filter_map(|hb| {
                    after
                        .ally_hand
                        .iter()
                        .find(|ha| ha.inst_id == hb.inst_id)
                        .map(|ha| ha.spellboost_count - hb.spellboost_count)
                })
                .sum();
            let new_boosted: i32 = after
                .ally_hand
                .iter()
                .filter(|ha| !before.ally_hand.iter().any(|hb| hb.inst_id == ha.inst_id))
                .map(|ha| ha.spellboost_count)
                .sum();
            if boosted + new_boosted >= need {
                Ok(())
            } else {
                Err(format!(
                    "spellboostHand: +{boosted} on old cards, +{new_boosted} on new, need >= {need}"
                ))
            }
        }
        Effect::Reanimate { .. } => {
            let field_growth = after
                .ally_field
                .len()
                .saturating_sub(before.ally_field.len());
            if field_growth > 0 {
                Ok(())
            } else {
                Err(format!("reanimate: field +{field_growth}"))
            }
        }
        Effect::Pay {
            resource,
            amount,
            effects,
            ..
        } => {
            let need = amount_min(amount);
            let after_res = resource_before(resource, after);
            let before_res = resource_before(resource, before);
            if before_res >= need && after_res < before_res {
                for x in effects {
                    effect_observed_strict(db, before, after, ctx, x)?;
                }
                Ok(())
            } else if before_res < need {
                Err(format!(
                    "pay {:?}: not affordable at baseline (board gap)",
                    resource
                ))
            } else {
                Err(format!("pay {:?}: resource unchanged", resource))
            }
        }
        Effect::Crest { .. } => {
            if after.ally_crest_ids.len() > before.ally_crest_ids.len()
                || after.ally_crest_ids != before.ally_crest_ids
            {
                Ok(())
            } else {
                Err("crest: no new crest".into())
            }
        }
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
