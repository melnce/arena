//! Search key: FNV-1a 64 over a hand-rolled canonical walk of `State`
//! except the RNG. Stable across runs of the same binary.

use crate::card::{Ability, CardId, Traits, TriggerTag, Until, VarKey};
use crate::state::{
    BoundRef, CardInstance, ChoiceNode, CrestInstance, DestroyedRecord, InstanceFlags, LeaderMod,
    PendingChoice, PendingKind, PendingPlayChoices, Phase, PlayCapture, PlayCapturedTarget,
    PlayPickState, PlayPickStep, PlayerState, QueuedTrigger, SourceRef, State, TargetOpt,
    TempTraitGrant, WorkFrame,
};
use crate::trace::fnv1a64;

/// FNV-1a 64 of every field that can change the future, excluding `State.rng`.
pub fn search_key(state: &State) -> u64 {
    let mut w = Writer::new();
    w.u8(state.active as u8);
    w.u8(state.first as u8);
    w.u32(state.turn);
    match state.winner {
        Some(p) => {
            w.u8(1);
            w.u8(p as u8);
        }
        None => w.u8(0),
    }
    hash_phase(&mut w, &state.phase);
    w.u64(state.step_counter);
    w.u32(state.next_instance);
    w.u32(state.crest_order);
    w.bool(state.suppress_last_words);
    match state.pending_play_rally {
        Some(p) => {
            w.u8(1);
            w.u8(p as u8);
        }
        None => w.u8(0),
    }
    match &state.deferred_play_rx {
        Some(rx) => {
            w.u8(1);
            w.u32(rx.len() as u32);
            for q in rx {
                hash_queued(&mut w, q);
            }
        }
        None => w.u8(0),
    }
    match &state.play_picks {
        Some(picks) => {
            w.u8(1);
            hash_play_picks(&mut w, picks);
        }
        None => w.u8(0),
    }
    match &state.pending_play_choices {
        Some(p) => {
            w.u8(1);
            hash_pending_play_choices(&mut w, p);
        }
        None => w.u8(0),
    }
    hash_opt_target(&mut w, state.event_subject.as_ref());
    w.u32(state.invoked_ids.len() as u32);
    for id in &state.invoked_ids {
        w.card(*id);
    }
    match state.event_base_cost {
        Some(n) => {
            w.u8(1);
            w.i32(n);
        }
        None => w.u8(0),
    }
    match state.event_inst_id {
        Some(n) => {
            w.u8(1);
            w.u32(n);
        }
        None => w.u8(0),
    }
    w.bool(state.attack_target_is_leader);
    w.bool(state.bind_append);
    w.bool(state.attacking_follower);
    hash_opt_target(&mut w, state.combat_opposing.as_ref());
    hash_opt_target(&mut w, state.combat_attacker.as_ref());
    match state.combat_defender_id {
        Some(n) => {
            w.u8(1);
            w.u32(n);
        }
        None => w.u8(0),
    }
    for p in &state.players {
        hash_player(&mut w, p);
    }
    w.u32(state.picks.len() as u32);
    for pick in &state.picks {
        w.bytes(format!("{:?}", pick).as_bytes());
    }
    w.u32(state.pending_work.len() as u32);
    for frame in &state.pending_work {
        hash_work(&mut w, frame);
    }
    w.u32(state.queue.len() as u32);
    for q in &state.queue {
        hash_queued(&mut w, q);
    }
    w.u32(state.bindings.len() as u32);
    for (k, refs) in &state.bindings {
        w.str(k);
        w.u32(refs.len() as u32);
        for r in refs {
            hash_bound(&mut w, r);
        }
    }
    fnv1a64(&w.buf)
}

/// Key for forced_lethal's cycle check and transposition set: the public snapshot hash at a quiescent state,
/// plus the in-flight state everywhere else.
pub fn lethal_key(state: &State) -> u64 {
    lethal_key_with(state, crate::snapshot::hash(state))
}

pub(crate) fn lethal_key_with(state: &State, public: u64) -> u64 {
    if is_lethal_quiescent(state) {
        return public;
    }
    let mut w = Writer::new();
    w.u64(public);
    w.u8(state.active as u8);
    w.u8(state.first as u8);
    w.u32(state.turn);
    match state.winner {
        Some(p) => {
            w.u8(1);
            w.u8(p as u8);
        }
        None => w.u8(0),
    }
    hash_phase_lethal(&mut w, &state.phase);
    w.bool(state.suppress_last_words);
    match state.pending_play_rally {
        Some(p) => {
            w.u8(1);
            w.u8(p as u8);
        }
        None => w.u8(0),
    }
    match &state.deferred_play_rx {
        Some(rx) => {
            w.u8(1);
            w.u32(rx.len() as u32);
            for q in rx {
                hash_queued(&mut w, q);
            }
        }
        None => w.u8(0),
    }
    match &state.play_picks {
        Some(picks) => {
            w.u8(1);
            hash_play_picks(&mut w, picks);
        }
        None => w.u8(0),
    }
    match &state.pending_play_choices {
        Some(p) => {
            w.u8(1);
            hash_pending_play_choices(&mut w, p);
        }
        None => w.u8(0),
    }
    hash_opt_target(&mut w, state.event_subject.as_ref());
    w.u32(state.invoked_ids.len() as u32);
    for id in &state.invoked_ids {
        w.card(*id);
    }
    match state.event_base_cost {
        Some(n) => {
            w.u8(1);
            w.i32(n);
        }
        None => w.u8(0),
    }
    match state.event_inst_id {
        Some(n) => {
            w.u8(1);
            w.u32(n);
        }
        None => w.u8(0),
    }
    w.bool(state.attack_target_is_leader);
    w.bool(state.bind_append);
    w.bool(state.attacking_follower);
    hash_opt_target(&mut w, state.combat_opposing.as_ref());
    hash_opt_target(&mut w, state.combat_attacker.as_ref());
    match state.combat_defender_id {
        Some(n) => {
            w.u8(1);
            w.u32(n);
        }
        None => w.u8(0),
    }
    w.u32(state.pending_work.len() as u32);
    for frame in &state.pending_work {
        hash_work(&mut w, frame);
    }
    w.u32(state.queue.len() as u32);
    for q in &state.queue {
        hash_queued(&mut w, q);
    }
    w.u32(state.bindings.len() as u32);
    for (k, refs) in &state.bindings {
        w.str(k);
        w.u32(refs.len() as u32);
        for r in refs {
            hash_bound(&mut w, r);
        }
    }
    fnv1a64(&w.buf)
}

fn is_lethal_quiescent(state: &State) -> bool {
    matches!(state.phase, Phase::Main)
        && state.pending_work.is_empty()
        && state.queue.is_empty()
        && state.play_picks.is_none()
        && state.pending_play_choices.is_none()
        && state.deferred_play_rx.is_none()
        && state.pending_play_rally.is_none()
}

fn hash_phase_lethal(w: &mut Writer, phase: &Phase) {
    match phase {
        Phase::Choice { player, node } => {
            let node = match node {
                ChoiceNode::Modes {
                    options,
                    pending,
                    picked,
                } => {
                    let mut picked = picked.clone();
                    picked.sort_unstable();
                    ChoiceNode::Modes {
                        options: options.clone(),
                        pending: pending.clone(),
                        picked,
                    }
                }
                ChoiceNode::FusePartners {
                    host,
                    options,
                    picked,
                } => {
                    let mut picked = picked.clone();
                    picked.sort_unstable();
                    ChoiceNode::FusePartners {
                        host: *host,
                        options: options.clone(),
                        picked,
                    }
                }
                other => other.clone(),
            };
            w.u8(3);
            w.u8(*player as u8);
            hash_choice(w, &node);
        }
        other => hash_phase(w, other),
    }
}

struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self {
            buf: Vec::with_capacity(4096),
        }
    }
    fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    fn bool(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }
    fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.buf.extend_from_slice(s.as_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.u32(b.len() as u32);
        self.buf.extend_from_slice(b);
    }
    fn card(&mut self, id: CardId) {
        self.u32(id.0);
    }
}

fn hash_phase(w: &mut Writer, phase: &Phase) {
    match phase {
        Phase::Mulligan { player } => {
            w.u8(0);
            w.u8(*player as u8);
        }
        Phase::Main => w.u8(1),
        Phase::Combat => w.u8(2),
        Phase::Choice { player, node } => {
            w.u8(3);
            w.u8(*player as u8);
            hash_choice(w, node);
        }
        Phase::End => w.u8(4),
        Phase::Terminal => w.u8(5),
    }
}

fn hash_choice(w: &mut Writer, node: &ChoiceNode) {
    match node {
        ChoiceNode::Targets { options, pending } => {
            w.u8(0);
            w.u32(options.len() as u32);
            for o in options {
                hash_target(w, o);
            }
            hash_pending(w, pending);
        }
        ChoiceNode::Modes {
            options,
            pending,
            picked,
        } => {
            w.u8(1);
            w.u32(options.len() as u32);
            for o in options {
                w.u8(*o);
            }
            hash_pending(w, pending);
            w.u32(picked.len() as u32);
            for p in picked {
                w.u8(*p);
            }
        }
        ChoiceNode::Cards { options, pending } => {
            w.u8(2);
            w.u32(options.len() as u32);
            for o in options {
                w.card(*o);
            }
            hash_pending(w, pending);
        }
        ChoiceNode::FusePartners {
            host,
            options,
            picked,
        } => {
            w.u8(3);
            w.u8(*host);
            w.u32(options.len() as u32);
            for o in options {
                w.u8(*o);
            }
            w.u32(picked.len() as u32);
            for p in picked {
                w.u8(*p);
            }
        }
        ChoiceNode::MultiPick {
            options,
            picked,
            pending,
        } => {
            w.u8(4);
            w.u32(options.len() as u32);
            for o in options {
                hash_target(w, o);
            }
            w.u32(picked.len() as u32);
            for p in picked {
                w.u8(*p);
            }
            hash_pending(w, pending);
        }
    }
}

fn hash_pending(w: &mut Writer, p: &PendingChoice) {
    w.u8(match p.kind {
        PendingKind::PlaySelect => 0,
        PendingKind::EffectSelect => 1,
        PendingKind::ModeSelect => 2,
        PendingKind::DiscardSelect => 3,
        PendingKind::EvolveSelect => 4,
    });
    w.u8(p.remaining);
}

fn hash_target(w: &mut Writer, t: &TargetOpt) {
    match t {
        TargetOpt::Slot { player, slot } => {
            w.u8(0);
            w.u8(*player as u8);
            w.u8(*slot);
        }
        TargetOpt::Leader { player } => {
            w.u8(1);
            w.u8(*player as u8);
        }
        TargetOpt::Hand { player, pos } => {
            w.u8(2);
            w.u8(*player as u8);
            w.u8(*pos);
        }
        TargetOpt::Deck { player, id } => {
            w.u8(3);
            w.u8(*player as u8);
            w.u32(*id);
        }
        TargetOpt::Card(id) => {
            w.u8(4);
            w.card(*id);
        }
        TargetOpt::Mode(m) => {
            w.u8(5);
            w.u8(*m);
        }
    }
}

fn hash_opt_target(w: &mut Writer, t: Option<&TargetOpt>) {
    match t {
        Some(t) => {
            w.u8(1);
            hash_target(w, t);
        }
        None => w.u8(0),
    }
}

fn hash_player(w: &mut Writer, p: &PlayerState) {
    w.i32(p.leader_defense);
    w.i32(p.leader_max);
    w.i32(p.pp);
    w.i32(p.pp_max);
    w.bool(p.bonus_pp.early_charge);
    w.bool(p.bonus_pp.late_charge);
    w.bool(p.bonus_pp.active);
    w.bool(p.bonus_pp.locked);
    w.i32(p.ep);
    w.i32(p.sep);
    w.i32(p.evolves_used);
    w.bool(p.evolved_this_turn);
    w.i32(p.shadows);
    w.i32(p.combo);
    w.i32(p.earth);
    match p.earth_slot {
        Some(s) => {
            w.u8(1);
            w.u8(s);
        }
        None => w.u8(0),
    }
    w.i32(p.faith);
    w.i32(p.rally);
    w.u32(p.leader_mods.len() as u32);
    for m in &p.leader_mods {
        hash_leader_mod(w, m);
    }
    w.u32(p.crests.len() as u32);
    for c in &p.crests {
        hash_crest(w, c);
    }
    w.u32(p.hand.len() as u32);
    for c in &p.hand {
        hash_inst(w, c);
    }
    for slot in &p.field {
        match slot {
            Some(c) => {
                w.u8(1);
                hash_inst(w, c);
            }
            None => w.u8(0),
        }
    }
    w.u32(p.deck.len() as u32);
    for c in &p.deck {
        hash_inst(w, c);
    }
    w.u32(p.cemetery.len() as u32);
    for c in &p.cemetery {
        hash_inst(w, c);
    }
    w.u32(p.banished.len() as u32);
    for c in &p.banished {
        hash_inst(w, c);
    }
    w.u32(p.destroyed_history.len() as u32);
    for d in &p.destroyed_history {
        hash_destroyed(w, d);
    }
    w.u32(p.played_this_turn.len() as u32);
    for id in &p.played_this_turn {
        w.card(*id);
    }
    w.u32(p.played_base_costs_this_match.len() as u32);
    for n in &p.played_base_costs_this_match {
        w.i32(*n);
    }
    w.bool(p.attacked_leader_this_turn);
    w.bool(p.attacked_leader_last_turn);
    w.u32(p.turns_taken);
    w.bool(p.is_second);
    w.u32(p.enter_counts.len() as u32);
    for (id, n) in &p.enter_counts {
        w.card(*id);
        w.i32(*n);
    }
    w.u32(p.starting_deck.len() as u32);
    for id in &p.starting_deck {
        w.card(*id);
    }
    w.u32(p.public_removals.len() as u32);
    for id in &p.public_removals {
        w.card(*id);
    }
    w.u32(p.public_hand_additions.len() as u32);
    for id in &p.public_hand_additions {
        w.card(*id);
    }
}

fn hash_leader_mod(w: &mut Writer, m: &LeaderMod) {
    match m.max_defense {
        Some(n) => {
            w.u8(1);
            w.i32(n);
        }
        None => w.u8(0),
    }
    match m.damage_cap {
        Some(n) => {
            w.u8(1);
            w.i32(n);
        }
        None => w.u8(0),
    }
    w.i32(m.damage_taken_bonus);
    match m.until {
        Some(Until::EndOfTurn) => w.u8(1),
        Some(Until::EndOfOpponentTurn) => w.u8(2),
        None => w.u8(0),
    }
}

fn hash_crest(w: &mut Writer, c: &CrestInstance) {
    w.str(&c.id);
    match c.countdown {
        Some(n) => {
            w.u8(1);
            w.i32(n);
        }
        None => w.u8(0),
    }
    w.bool(c.faith);
    w.u32(c.once_used.len() as u32);
    for t in &c.once_used {
        hash_tag(w, *t);
    }
    w.u32(c.granted_order);
    w.u32(c.granted.len() as u32);
    for a in &c.granted {
        hash_ability(w, a);
    }
    w.u32(c.choose_used.len() as u32);
    for i in &c.choose_used {
        w.u8(*i);
    }
}

fn hash_destroyed(w: &mut Writer, d: &DestroyedRecord) {
    w.card(d.card);
    w.i32(d.base_cost);
    w.u8(d.kind as u8);
    w.u8(d.owner as u8);
    w.bool(d.from_field);
}

fn hash_inst(w: &mut Writer, c: &CardInstance) {
    w.u32(c.id);
    w.card(c.card);
    w.i32(c.cost);
    w.i32(c.base_cost);
    w.i32(c.attack);
    w.i32(c.defense);
    w.i32(c.max_defense);
    w.bool(c.evolved);
    w.bool(c.super_evolved);
    hash_traits(w, &c.traits);
    w.u32(c.granted.len() as u32);
    for a in &c.granted {
        hash_ability(w, a);
    }
    w.u8(c.granted_whens);
    w.u32(c.printed_tags.len() as u32);
    for t in &c.printed_tags {
        w.str(t);
    }
    hash_flags(w, &c.flags);
    w.u32(c.vars.len() as u32);
    for (k, n) in &c.vars {
        w.u8(match k {
            VarKey::X => 0,
            VarKey::Y => 1,
            VarKey::Z => 2,
        });
        w.i32(*n);
    }
    w.i32(c.skybound);
    match c.countdown {
        Some(n) => {
            w.u8(1);
            w.i32(n);
        }
        None => w.u8(0),
    }
    w.i32(c.spellboost_count);
    w.u8(c.kind as u8);
    w.u8(c.class as u8);
    w.u32(c.tribes.len() as u32);
    for t in &c.tribes {
        w.bytes(format!("{t:?}").as_bytes());
    }
    w.str(&c.name);
    w.u32(c.once_used.len() as u32);
    for t in &c.once_used {
        hash_tag(w, *t);
    }
    w.u32(c.sequence_index);
    w.u32(c.temp_traits.len() as u32);
    for g in &c.temp_traits {
        hash_temp(w, g);
    }
    w.u32(c.choose_used.len() as u32);
    for i in &c.choose_used {
        w.u8(*i);
    }
}

fn hash_temp(w: &mut Writer, g: &TempTraitGrant) {
    hash_traits(w, &g.traits);
    w.u8(match g.until {
        Until::EndOfTurn => 0,
        Until::EndOfOpponentTurn => 1,
    });
    w.u8(g.caster as u8);
}

fn hash_flags(w: &mut Writer, f: &InstanceFlags) {
    w.bool(f.was_fused);
    w.u32(f.fused_kinds.len() as u32);
    for id in &f.fused_kinds {
        w.card(*id);
    }
    w.bool(f.ambush_active);
    w.bool(f.summoning_sick);
    w.bool(f.attacked_this_turn);
    w.i32(f.attacks_left);
    w.bool(f.engaged_this_turn);
    w.bool(f.fused_this_turn);
    w.i32(f.eot_attack);
    w.i32(f.eot_defense);
    match f.eot_cost {
        Some(n) => {
            w.u8(1);
            w.i32(n);
        }
        None => w.u8(0),
    }
    w.bool(f.enhanced);
    w.u32(f.temp_traits.len() as u32);
    for (until, traits, caster) in &f.temp_traits {
        w.u8(match until {
            Until::EndOfTurn => 0,
            Until::EndOfOpponentTurn => 1,
        });
        hash_traits(w, traits);
        w.u8(*caster as u8);
    }
}

fn hash_traits(w: &mut Writer, t: &Traits) {
    w.bytes(&serde_json::to_vec(t).unwrap_or_default());
}

fn hash_ability(w: &mut Writer, a: &Ability) {
    w.bytes(&serde_json::to_vec(a).unwrap_or_default());
}

fn hash_tag(w: &mut Writer, t: TriggerTag) {
    w.u8(t as u8);
}

fn hash_work(w: &mut Writer, frame: &WorkFrame) {
    w.bytes(&serde_json::to_vec(&work_tag(frame)).unwrap_or_default());
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
            w.u8(*controller as u8);
            hash_source(w, source);
            w.u32(effects.len() as u32);
            for e in effects {
                w.bytes(&serde_json::to_vec(e).unwrap_or_default());
            }
            w.u32(*index as u32);
            hash_opt_target(w, subject.as_ref());
            match subject_id {
                Some(id) => {
                    w.u8(1);
                    w.u32(*id);
                }
                None => w.u8(0),
            }
            w.bool(*e40);
        }
        WorkFrame::Aftermath(a) => {
            w.bytes(format!("{a:?}").as_bytes());
        }
    }
}

fn work_tag(frame: &WorkFrame) -> u8 {
    match frame {
        WorkFrame::Effects { .. } => 0,
        WorkFrame::Aftermath(_) => 1,
    }
}

fn hash_queued(w: &mut Writer, q: &QueuedTrigger) {
    w.u8(q.category);
    w.u32(q.entry);
    w.u8(q.printed_order);
    w.u8(q.controller as u8);
    hash_source(w, &q.source);
    hash_tag(w, q.tag);
    w.u32(q.effects.len() as u32);
    for e in &q.effects {
        w.bytes(&serde_json::to_vec(e).unwrap_or_default());
    }
    hash_opt_target(w, q.subject.as_ref());
    match q.subject_id {
        Some(id) => {
            w.u8(1);
            w.u32(id);
        }
        None => w.u8(0),
    }
}

fn hash_source(w: &mut Writer, s: &SourceRef) {
    match s {
        SourceRef::Field { player, id } => {
            w.u8(0);
            w.u8(*player as u8);
            w.u32(*id);
        }
        SourceRef::Hand { player, id } => {
            w.u8(1);
            w.u8(*player as u8);
            w.u32(*id);
        }
        SourceRef::Crest { player, index } => {
            w.u8(2);
            w.u8(*player as u8);
            w.u32(*index as u32);
        }
        SourceRef::Spell { player, card } => {
            w.u8(3);
            w.u8(*player as u8);
            w.card(*card);
        }
        SourceRef::Leader { player } => {
            w.u8(4);
            w.u8(*player as u8);
        }
    }
}

fn hash_play_captured(w: &mut Writer, t: &PlayCapturedTarget) {
    match t {
        PlayCapturedTarget::Field { player, id } => {
            w.u8(0);
            w.u8(*player as u8);
            w.u32(*id);
        }
        PlayCapturedTarget::Hand { player, id } => {
            w.u8(1);
            w.u8(*player as u8);
            w.u32(*id);
        }
        PlayCapturedTarget::Leader { player } => {
            w.u8(2);
            w.u8(*player as u8);
        }
        PlayCapturedTarget::Deck { player, id } => {
            w.u8(3);
            w.u8(*player as u8);
            w.u32(*id);
        }
        PlayCapturedTarget::Card(c) => {
            w.u8(4);
            w.card(*c);
        }
        PlayCapturedTarget::Mode(m) => {
            w.u8(5);
            w.u8(*m);
        }
    }
}

fn hash_play_picks(w: &mut Writer, picks: &PlayPickState) {
    w.u32(picks.cursor as u32);
    w.u32(picks.captures.len() as u32);
    for cap in &picks.captures {
        match cap {
            PlayCapture::Target(t) => {
                w.u8(0);
                hash_play_captured(w, t);
            }
            PlayCapture::Targets(ts) => {
                w.u8(2);
                w.u32(ts.len() as u32);
                for t in ts {
                    hash_play_captured(w, t);
                }
            }
            PlayCapture::Modes(idxs) => {
                w.u8(1);
                w.u32(idxs.len() as u32);
                for i in idxs {
                    w.u8(*i);
                }
            }
        }
    }
    w.u32(picks.partial_targets.len() as u32);
    for t in &picks.partial_targets {
        hash_play_captured(w, t);
    }
}

fn hash_pending_play_choices(w: &mut Writer, p: &PendingPlayChoices) {
    w.u32(p.step_idx as u32);
    w.u8(p.controller as u8);
    hash_source(w, &p.source);
    w.u32(p.deferred_rx.len() as u32);
    for q in &p.deferred_rx {
        hash_queued(w, q);
    }
    w.u32(p.steps.len() as u32);
    for step in &p.steps {
        match step {
            PlayPickStep::Target(e) | PlayPickStep::Mode(e) => {
                w.bytes(format!("{e:?}").as_bytes());
            }
        }
    }
}

fn hash_bound(w: &mut Writer, r: &BoundRef) {
    match r {
        BoundRef::Field {
            player,
            id,
            card,
            kind,
        } => {
            w.u8(0);
            w.u8(*player as u8);
            w.u32(*id);
            w.card(*card);
            w.u8(*kind as u8);
        }
        BoundRef::Leader { player } => {
            w.u8(1);
            w.u8(*player as u8);
        }
        BoundRef::Hand { player, id } => {
            w.u8(2);
            w.u8(*player as u8);
            w.u32(*id);
        }
        BoundRef::Deck { player, id } => {
            w.u8(3);
            w.u8(*player as u8);
            w.u32(*id);
        }
        BoundRef::Card(id) => {
            w.u8(4);
            w.card(*id);
        }
    }
}
