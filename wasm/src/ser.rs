//! JSON for `Event` and the full `State`. Engine types are not all `Serialize`.

use arena_engine::card::CardId;
use arena_engine::event::{Event, EventSource, EventTarget, ZoneLabel};
use arena_engine::ids::PlayerId;
use arena_engine::state::{
    BoundRef, CardInstance, ChoiceNode, CrestInstance, DestroyedRecord, LeaderMod, PendingKind,
    Phase, PlayForm, PlayerState, QueuedTrigger, SourceRef, State, TargetOpt, WorkFrame,
};
use serde_json::{json, Value};

pub fn phase_str(phase: &Phase) -> &'static str {
    match phase {
        Phase::Mulligan { .. } => "mulligan",
        Phase::Main | Phase::Combat => "main",
        Phase::Choice { .. } => "choice",
        Phase::End => "end",
        Phase::Terminal => "terminal",
    }
}

pub fn events_json(events: &[Event]) -> Result<String, String> {
    let arr: Vec<Value> = events.iter().map(event_json).collect();
    serde_json::to_string(&arr).map_err(|e| e.to_string())
}

fn event_json(event: &Event) -> Value {
    match event {
        Event::Draw { player, card } => {
            json!({"draw": {"player": pl(*player), "card": card.as_str()}})
        }
        Event::Mulligan { player, swapped } => json!({
            "mulligan": {"player": pl(*player), "swapped": ids(swapped)}
        }),
        Event::Play { player, card, form } => json!({
            "play": {"player": pl(*player), "card": card.as_str(), "form": play_form(form)}
        }),
        Event::Summon { player, card, slot } => json!({
            "summon": {"player": pl(*player), "card": card.as_str(), "slot": slot.0}
        }),
        Event::Enter { slot } => json!({"enter": {"slot": slot.0}}),
        Event::Damage {
            target,
            amount,
            lethal,
            unit,
        } => {
            json!({"damage": {"target": event_target(target, unit), "amount": amount, "lethal": lethal}})
        }
        Event::Restore { target, amount, unit } => {
            json!({"restore": {"target": event_target(target, unit), "amount": amount}})
        }
        Event::Destroy { slot, card, player, id } => {
            json!({"destroy": {"slot": slot.0, "card": card.as_str(), "player": pl(*player), "id": id}})
        }
        Event::Banish { card, from } => {
            json!({"banish": {"card": card.as_str(), "from": zone_label(*from)}})
        }
        Event::Transform { slot, into } => {
            json!({"transform": {"slot": slot.0, "into": into.as_str()}})
        }
        Event::Evolve {
            slot,
            super_evolve,
            granted,
        } => json!({"evolve": {"slot": slot.0, "super": super_evolve, "granted": granted}}),
        Event::Resolve { source } => json!({"resolve": {"source": event_source(source)}}),
        Event::ChoiceOffered { player, node, source } => json!({
            "choice_offered": {
                "player": pl(*player),
                "node": choice_node(node),
                "source": source.as_ref().map(source_ref),
            }
        }),
        Event::RandomPick { what, target, unit } => json!({
            "random_pick": {"what": what.as_str(), "target": random_pick_target(target, unit)}
        }),
        Event::Counter { key, value } => json!({"counter": {"key": key, "value": value}}),
        Event::CrestGain { player, id } => json!({"crest_gain": {"player": pl(*player), "id": id}}),
        Event::CrestRemove { player, id } => {
            json!({"crest_remove": {"player": pl(*player), "id": id}})
        }
        Event::Fuse { host, partners } => {
            json!({"fuse": {"host": host.as_str(), "partners": ids(partners)}})
        }
        Event::TurnStart { player, turn } => {
            json!({"turn_start": {"player": pl(*player), "turn": turn}})
        }
        Event::TurnEnd { player } => json!({"turn_end": {"player": pl(*player)}}),
        Event::Win { player } => json!({"win": {"player": pl(*player)}}),
    }
}

pub fn full_json(state: &State) -> Result<String, String> {
    let v = json!({
        "players": {
            "a": player_json(&state.players[0]),
            "b": player_json(&state.players[1]),
        },
        "turn": state.turn,
        "active": pl(state.active),
        "first": pl(state.first),
        "phase": phase_full(&state.phase),
        "winner": state.winner.map(pl),
        "rng": format!("{:?}", state.rng),
        "step_counter": state.step_counter,
        "next_instance": state.next_instance,
        "crest_order": state.crest_order,
        "picks": state.picks,
        "pending_work": state.pending_work.iter().map(work_frame).collect::<Vec<_>>(),
        "queue": state.queue.iter().map(queued).collect::<Vec<_>>(),
        "suppress_last_words": state.suppress_last_words,
        "bindings": state.bindings.iter().map(|(k, v)| {
            (k.clone(), json!(v.iter().map(bound_ref).collect::<Vec<_>>()))
        }).collect::<serde_json::Map<String, Value>>(),
        "pending_play_rally": state.pending_play_rally.map(pl),
        "event_subject": state.event_subject.as_ref().map(target_opt),
        "invoked_ids": state.invoked_ids.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "event_base_cost": state.event_base_cost,
        "event_inst_id": state.event_inst_id,
    });
    serde_json::to_string(&v).map_err(|e| e.to_string())
}

fn player_json(p: &PlayerState) -> Value {
    json!({
        "leader_defense": p.leader_defense,
        "leader_max": p.leader_max,
        "pp": p.pp,
        "pp_max": p.pp_max,
        "bonus_pp": {
            "early_charge": p.bonus_pp.early_charge,
            "late_charge": p.bonus_pp.late_charge,
            "active": p.bonus_pp.active,
            "locked": p.bonus_pp.locked,
        },
        "ep": p.ep,
        "sep": p.sep,
        "evolves_used": p.evolves_used,
        "evolved_this_turn": p.evolved_this_turn,
        "shadows": p.shadows,
        "combo": p.combo,
        "earth": p.earth,
        "earth_slot": p.earth_slot,
        "faith": p.faith,
        "rally": p.rally,
        "leader_mods": p.leader_mods.iter().map(leader_mod).collect::<Vec<_>>(),
        "crests": p.crests.iter().map(crest_json).collect::<Vec<_>>(),
        "hand": p.hand.iter().map(instance_json).collect::<Vec<_>>(),
        "field": p.field.iter().map(|s| s.as_ref().map(instance_json)).collect::<Vec<_>>(),
        "deck": p.deck.iter().map(instance_json).collect::<Vec<_>>(),
        "cemetery": p.cemetery.iter().map(instance_json).collect::<Vec<_>>(),
        "banished": p.banished.iter().map(instance_json).collect::<Vec<_>>(),
        "destroyed_history": p.destroyed_history.iter().map(destroyed).collect::<Vec<_>>(),
        "played_this_turn": ids(&p.played_this_turn),
        "played_base_costs_this_match": p.played_base_costs_this_match,
        "attacked_leader_this_turn": p.attacked_leader_this_turn,
        "attacked_leader_last_turn": p.attacked_leader_last_turn,
        "turns_taken": p.turns_taken,
        "is_second": p.is_second,
        "enter_counts": p.enter_counts.iter().map(|(k, n)| (k.as_str(), json!(n))).collect::<serde_json::Map<String, Value>>(),
    })
}

fn instance_json(c: &CardInstance) -> Value {
    json!({
        "id": c.id,
        "card": c.card.as_str(),
        "name": c.name,
        "kind": format!("{:?}", c.kind).to_ascii_lowercase(),
        "class": format!("{:?}", c.class).to_ascii_lowercase(),
        "cost": c.cost,
        "base_cost": c.base_cost,
        "attack": c.attack,
        "defense": c.defense,
        "max_defense": c.max_defense,
        "evolved": c.evolved,
        "super_evolved": c.super_evolved,
        "traits": c.traits.snapshot_tags(),
        "granted": c.granted,
        "granted_whens": c.granted_whens,
        "printed_tags": c.printed_tags.iter().collect::<Vec<_>>(),
        "flags": {
            "was_fused": c.flags.was_fused,
            "fused_kinds": ids(&c.flags.fused_kinds),
            "ambush_active": c.flags.ambush_active,
            "summoning_sick": c.flags.summoning_sick,
            "attacked_this_turn": c.flags.attacked_this_turn,
            "attacks_left": c.flags.attacks_left,
            "engaged_this_turn": c.flags.engaged_this_turn,
            "fused_this_turn": c.flags.fused_this_turn,
            "eot_attack": c.flags.eot_attack,
            "eot_defense": c.flags.eot_defense,
            "eot_cost": c.flags.eot_cost,
            "enhanced": c.flags.enhanced,
        },
        "vars": c.vars,
        "skybound": c.skybound,
        "countdown": c.countdown,
        "spellboost_count": c.spellboost_count,
        "tribes": c.tribes,
        "once_used": c.once_used,
    })
}

fn crest_json(c: &CrestInstance) -> Value {
    json!({
        "id": c.id,
        "countdown": c.countdown,
        "faith": c.faith,
        "once_used": c.once_used,
        "granted_order": c.granted_order,
        "granted": c.granted,
    })
}

fn leader_mod(m: &LeaderMod) -> Value {
    json!({
        "max_defense": m.max_defense,
        "damage_cap": m.damage_cap,
        "damage_taken_bonus": m.damage_taken_bonus,
        "until": m.until,
    })
}

fn destroyed(d: &DestroyedRecord) -> Value {
    json!({
        "card": d.card.as_str(),
        "base_cost": d.base_cost,
        "kind": format!("{:?}", d.kind).to_ascii_lowercase(),
        "owner": pl(d.owner),
        "from_field": d.from_field,
    })
}

fn phase_full(phase: &Phase) -> Value {
    match phase {
        Phase::Mulligan { player } => json!({"mulligan": {"player": pl(*player)}}),
        Phase::Main => json!("main"),
        Phase::Combat => json!("combat"),
        Phase::Choice { player, node } => {
            json!({"choice": {"player": pl(*player), "node": choice_node(node)}})
        }
        Phase::End => json!("end"),
        Phase::Terminal => json!("terminal"),
    }
}

fn choice_node(node: &ChoiceNode) -> Value {
    match node {
        ChoiceNode::Targets { options, pending } => json!({
            "targets": {
                "options": options.iter().map(target_opt).collect::<Vec<_>>(),
                "pending": pending_kind(pending.kind),
                "remaining": pending.remaining,
            }
        }),
        ChoiceNode::Modes {
            options,
            pending,
            picked,
        } => json!({
            "modes": {
                "options": options,
                "picked": picked,
                "pending": pending_kind(pending.kind),
                "remaining": pending.remaining,
            }
        }),
        ChoiceNode::Cards { options, pending } => json!({
            "cards": {
                "options": ids(options),
                "pending": pending_kind(pending.kind),
                "remaining": pending.remaining,
            }
        }),
        ChoiceNode::FusePartners {
            host,
            options,
            picked,
        } => json!({"fuse_partners": {"host": host, "options": options, "picked": picked}}),
        ChoiceNode::MultiPick {
            options,
            picked,
            pending,
        } => json!({
            "multi_pick": {
                "options": options.iter().map(target_opt).collect::<Vec<_>>(),
                "picked": picked,
                "pending": pending_kind(pending.kind),
                "remaining": pending.remaining,
            }
        }),
    }
}

fn target_opt(t: &TargetOpt) -> Value {
    match t {
        TargetOpt::Slot { player, slot } => json!({"slot": slot, "player": pl(*player)}),
        TargetOpt::Leader { player } => json!({"leader": pl(*player)}),
        TargetOpt::Hand { player, pos } => json!({"hand": {"player": pl(*player), "pos": pos}}),
        TargetOpt::Deck { player, id } => json!({"deck": {"player": pl(*player), "id": id}}),
        TargetOpt::Card(id) => json!({"card": id.as_str()}),
        TargetOpt::Mode(m) => json!({"mode": m}),
    }
}

fn work_frame(w: &WorkFrame) -> Value {
    match w {
        WorkFrame::Effects {
            controller,
            source,
            effects,
            index,
            subject,
            subject_id,
            e40,
        } => json!({
            "effects": {
                "controller": pl(*controller),
                "source": source_ref(source),
                "effects": effects,
                "index": index,
                "subject": subject.as_ref().map(target_opt),
                "subject_id": subject_id,
                "e40": e40,
            }
        }),
        WorkFrame::Aftermath(a) => json!({"aftermath": format!("{a:?}")}),
    }
}

fn queued(q: &QueuedTrigger) -> Value {
    json!({
        "category": q.category,
        "entry": q.entry,
        "printed_order": q.printed_order,
        "controller": pl(q.controller),
        "source": source_ref(&q.source),
        "tag": q.tag,
        "effects": q.effects,
        "subject": q.subject.as_ref().map(target_opt),
        "subject_id": q.subject_id,
    })
}

fn source_ref(s: &SourceRef) -> Value {
    match *s {
        SourceRef::Field { player, id } => json!({"field": {"player": pl(player), "id": id}}),
        SourceRef::Hand { player, id } => json!({"hand": {"player": pl(player), "id": id}}),
        SourceRef::Crest { player, index } => {
            json!({"crest": {"player": pl(player), "index": index}})
        }
        SourceRef::Spell { player, card } => {
            json!({"spell": {"player": pl(player), "card": card.as_str()}})
        }
        SourceRef::Leader { player } => json!({"leader": pl(player)}),
    }
}

fn bound_ref(b: &BoundRef) -> Value {
    match b {
        BoundRef::Field {
            player,
            id,
            card,
            kind,
        } => json!({
            "field": {
                "player": pl(*player),
                "id": id,
                "card": card.as_str(),
                "kind": format!("{kind:?}").to_ascii_lowercase(),
            }
        }),
        BoundRef::Leader { player } => json!({"leader": pl(*player)}),
        BoundRef::Hand { player, id } => json!({"hand": {"player": pl(*player), "id": id}}),
        BoundRef::Deck { player, id } => json!({"deck": {"player": pl(*player), "id": id}}),
        BoundRef::Card(id) => json!({"card": id.as_str()}),
    }
}

fn play_form(form: &PlayForm) -> Value {
    match *form {
        PlayForm::Normal => json!("normal"),
        PlayForm::Enhance { paid } => json!({"enhance": paid}),
        PlayForm::Accelerate { paid } => json!({"accelerate": paid}),
        PlayForm::Crystallize { paid } => json!({"crystallize": paid}),
    }
}

fn event_target(t: &EventTarget, unit: &Option<u32>) -> Value {
    match *t {
        EventTarget::Leader(p) => json!({"leader": pl(p)}),
        EventTarget::Slot(p, s) => {
            let mut v = json!({"slot": s.0, "player": pl(p)});
            if let Some(id) = unit {
                v["id"] = json!(id);
            }
            v
        }
    }
}

fn random_pick_target(t: &TargetOpt, unit: &Option<u32>) -> Value {
    match t {
        TargetOpt::Slot { player, slot } => {
            let mut v = json!({"slot": slot, "player": pl(*player)});
            if let Some(id) = unit {
                v["id"] = json!(id);
            }
            v
        }
        other => target_opt(other),
    }
}

fn event_source(s: &EventSource) -> Value {
    match s {
        EventSource::Ref(r) => source_ref(r),
        EventSource::Combat { player, id } => json!({"combat": {"player": pl(*player), "id": id}}),
    }
}

fn zone_label(z: ZoneLabel) -> &'static str {
    match z {
        ZoneLabel::Field => "field",
        ZoneLabel::Hand => "hand",
        ZoneLabel::Deck => "deck",
        ZoneLabel::Cemetery => "cemetery",
        ZoneLabel::Crests => "crests",
    }
}

fn pending_kind(k: PendingKind) -> &'static str {
    match k {
        PendingKind::PlaySelect => "play_select",
        PendingKind::EffectSelect => "effect_select",
        PendingKind::ModeSelect => "mode_select",
        PendingKind::DiscardSelect => "discard_select",
        PendingKind::EvolveSelect => "evolve_select",
    }
}

fn pl(p: PlayerId) -> &'static str {
    p.as_str()
}

fn ids(list: &[CardId]) -> Vec<String> {
    list.iter().map(|c| c.as_str()).collect()
}
