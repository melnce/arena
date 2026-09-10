//! JSON ↔ Python dicts, NeutralAction, Event, and a perfect-info State dump.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use arena_engine::card::{CardKind, Class, TriggerTag, VarKey};
use arena_engine::event::{Event, EventTarget, ZoneLabel};
use arena_engine::ids::PlayerId;
use arena_engine::state::TargetOpt;
use arena_engine::{
    CardDb, CardId, ChoiceNode, Illegal, LoadError, OpeningHands, Phase, PlayForm, State,
};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use serde_json::{json, Value};

pub fn py_err_load(e: LoadError) -> PyErr {
    PyValueError::new_err(e.to_string())
}

pub fn py_err_illegal(e: Illegal) -> PyErr {
    crate::Illegal::new_err(e.to_string())
}

pub fn py_err_msg(msg: impl Into<String>) -> PyErr {
    PyValueError::new_err(msg.into())
}

pub fn py_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    let json = obj.py().import("json")?;
    let s: String = json.call_method1("dumps", (obj,))?.extract()?;
    serde_json::from_str(&s).map_err(|e| PyValueError::new_err(e.to_string()))
}

pub fn value_to_py<'py>(py: Python<'py>, v: &Value) -> PyResult<Bound<'py, PyAny>> {
    let json = py.import("json")?;
    let s = serde_json::to_string(v).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    json.call_method1("loads", (s,))
}

pub fn resolve_cards_root(root: &str) -> PathBuf {
    let p = Path::new(root);
    if p.join("cards").is_dir() {
        return p.to_path_buf();
    }
    if p.join("official").is_dir() || p.file_name().and_then(|s| s.to_str()) == Some("cards") {
        return p.parent().unwrap_or(Path::new(".")).to_path_buf();
    }
    p.to_path_buf()
}

pub fn deck_from_value(v: &Value) -> PyResult<Vec<CardId>> {
    let obj = v
        .as_object()
        .ok_or_else(|| py_err_msg("deck must be a dict of card id → count"))?;
    let mut map = BTreeMap::new();
    for (k, n) in obj {
        let count = n
            .as_u64()
            .ok_or_else(|| py_err_msg(format!("deck count for {k} must be an int")))?;
        map.insert(k.clone(), count);
    }
    expand_deck(&map)
}

pub fn expand_deck(map: &BTreeMap<String, u64>) -> PyResult<Vec<CardId>> {
    let mut ids = Vec::new();
    for (k, n) in map {
        let id = CardId::parse(k).ok_or_else(|| py_err_msg(format!("invalid card id {k}")))?;
        for _ in 0..*n {
            ids.push(id);
        }
    }
    Ok(ids)
}

pub fn parse_first(s: &str) -> PyResult<arena_engine::First> {
    match s {
        "coin" => Ok(arena_engine::First::Coin),
        "a" => Ok(arena_engine::First::A),
        "b" => Ok(arena_engine::First::B),
        other => Err(py_err_msg(format!(
            "first must be 'coin', 'a', or 'b' (got {other})"
        ))),
    }
}

pub fn opening_hands_from_value(v: &Value) -> PyResult<OpeningHands> {
    let obj = v
        .as_object()
        .ok_or_else(|| py_err_msg("opening_hands must be a dict with 'a' and 'b'"))?;
    Ok(OpeningHands {
        a: ids_from_array(obj.get("a").unwrap_or(&Value::Null), "a")?,
        b: ids_from_array(obj.get("b").unwrap_or(&Value::Null), "b")?,
    })
}

fn ids_from_array(v: &Value, side: &str) -> PyResult<Vec<CardId>> {
    let arr = v
        .as_array()
        .ok_or_else(|| py_err_msg(format!("opening_hands.{side} must be a list of card ids")))?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| py_err_msg(format!("opening_hands.{side} entries must be strings")))?;
        let id = CardId::parse(s).ok_or_else(|| py_err_msg(format!("invalid card id {s}")))?;
        out.push(id);
    }
    Ok(out)
}

pub fn phase_str(p: &Phase) -> &'static str {
    match p {
        Phase::Mulligan { .. } => "mulligan",
        Phase::Main | Phase::Combat => "main",
        Phase::Choice { .. } => "choice",
        Phase::End => "end",
        Phase::Terminal => "terminal",
    }
}

pub fn player_str(p: PlayerId) -> &'static str {
    p.as_str()
}

pub fn event_to_value(ev: &Event) -> Value {
    match ev {
        Event::Draw { player, card } => {
            json!({"draw": {"player": player_str(*player), "card": card.as_str()}})
        }
        Event::Mulligan { player, swapped } => json!({
            "mulligan": {
                "player": player_str(*player),
                "swapped": swapped.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
            }
        }),
        Event::Play { player, card, form } => json!({
            "play": {
                "player": player_str(*player),
                "card": card.as_str(),
                "form": play_form_value(*form),
            }
        }),
        Event::Summon { player, card, slot } => json!({
            "summon": {
                "player": player_str(*player),
                "card": card.as_str(),
                "slot": slot.0,
            }
        }),
        Event::Enter { slot } => json!({"enter": {"slot": slot.0}}),
        Event::Damage {
            target,
            amount,
            lethal,
        } => {
            json!({"damage": {"target": event_target(target), "amount": amount, "lethal": lethal}})
        }
        Event::Restore { target, amount } => {
            json!({"restore": {"target": event_target(target), "amount": amount}})
        }
        Event::Destroy { slot, card } => {
            json!({"destroy": {"slot": slot.0, "card": card.as_str()}})
        }
        Event::Banish { card, from } => {
            json!({"banish": {"card": card.as_str(), "from": zone_str(*from)}})
        }
        Event::Transform { slot, into } => {
            json!({"transform": {"slot": slot.0, "into": into.as_str()}})
        }
        Event::Evolve {
            slot,
            super_evolve,
            granted,
        } => json!({
            "evolve": {
                "slot": slot.0,
                "super": super_evolve,
                "granted": granted,
            }
        }),
        Event::TriggerFired { on } => json!({"trigger_fired": {"on": trigger_tag_str(*on)}}),
        Event::ChoiceOffered { player, node } => json!({
            "choice_offered": {
                "player": player_str(*player),
                "node": choice_node_value(node),
            }
        }),
        Event::RandomPick { what } => json!({"random_pick": {"what": what.as_str()}}),
        Event::Counter { key, value } => json!({"counter": {"key": key, "value": value}}),
        Event::CrestGain { player, id } => {
            json!({"crest_gain": {"player": player_str(*player), "id": id}})
        }
        Event::CrestRemove { player, id } => {
            json!({"crest_remove": {"player": player_str(*player), "id": id}})
        }
        Event::Fuse { host, partners } => json!({
            "fuse": {
                "host": host.as_str(),
                "partners": partners.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
            }
        }),
        Event::TurnStart { player, turn } => {
            json!({"turn_start": {"player": player_str(*player), "turn": turn}})
        }
        Event::TurnEnd { player } => json!({"turn_end": {"player": player_str(*player)}}),
        Event::Win { player } => json!({"win": {"player": player_str(*player)}}),
    }
}

fn play_form_value(form: PlayForm) -> Value {
    match form {
        PlayForm::Normal => json!("normal"),
        PlayForm::Enhance { paid } => json!({"enhance": {"paid": paid}}),
        PlayForm::Accelerate { paid } => json!({"accelerate": {"paid": paid}}),
        PlayForm::Crystallize { paid } => json!({"crystallize": {"paid": paid}}),
    }
}

fn event_target(t: &EventTarget) -> Value {
    match t {
        EventTarget::Leader(p) => json!({"leader": player_str(*p)}),
        EventTarget::Slot(p, s) => json!({"player": player_str(*p), "slot": s.0}),
    }
}

fn zone_str(z: ZoneLabel) -> &'static str {
    match z {
        ZoneLabel::Field => "field",
        ZoneLabel::Hand => "hand",
        ZoneLabel::Deck => "deck",
        ZoneLabel::Cemetery => "cemetery",
        ZoneLabel::Crests => "crests",
    }
}

fn trigger_tag_str(t: TriggerTag) -> &'static str {
    match t {
        TriggerTag::Fanfare => "fanfare",
        TriggerTag::LastWords => "lastWords",
        TriggerTag::Evolve => "evolve",
        TriggerTag::SuperEvolve => "superEvolve",
        TriggerTag::AnyEvolve => "anyEvolve",
        TriggerTag::AnySuperEvolve => "anySuperEvolve",
        TriggerTag::Strike => "strike",
        TriggerTag::FollowerStrike => "followerStrike",
        TriggerTag::Clash => "clash",
        TriggerTag::Enter => "enter",
        TriggerTag::Leave => "leave",
        TriggerTag::Discarded => "discarded",
        TriggerTag::Invoked => "invoked",
        TriggerTag::Fused => "fused",
        TriggerTag::Spellboost => "spellboost",
        TriggerTag::Engage => "engage",
        TriggerTag::StartOfTurn => "startOfTurn",
        TriggerTag::EndOfTurn => "endOfTurn",
        TriggerTag::When => "when",
        TriggerTag::Enhance => "enhance",
    }
}

fn choice_node_value(node: &ChoiceNode) -> Value {
    match node {
        ChoiceNode::Targets { options, .. } => json!({
            "targets": {"options": options.iter().map(target_opt_value).collect::<Vec<_>>()}
        }),
        ChoiceNode::Modes { options, .. } => json!({"modes": {"options": options}}),
        ChoiceNode::Cards { options, .. } => json!({
            "cards": {"options": options.iter().map(|c| c.as_str()).collect::<Vec<_>>()}
        }),
        ChoiceNode::FusePartners {
            host,
            options,
            picked,
        } => json!({"fuse_partners": {"host": host, "options": options, "picked": picked}}),
        ChoiceNode::MultiPick {
            options, picked, ..
        } => json!({
            "multi_pick": {
                "options": options.iter().map(target_opt_value).collect::<Vec<_>>(),
                "picked": picked,
            }
        }),
    }
}

fn target_opt_value(t: &TargetOpt) -> Value {
    match t {
        TargetOpt::Slot { player, slot } => {
            json!({"slot": slot, "player": player_str(*player)})
        }
        TargetOpt::Leader { player } => json!({"leader": player_str(*player)}),
        TargetOpt::Hand { player, pos } => {
            json!({"hand": {"player": player_str(*player), "pos": pos}})
        }
        TargetOpt::Deck { player, id } => {
            json!({"deck": {"player": player_str(*player), "id": id}})
        }
        TargetOpt::Card(c) => json!({"card": c.as_str()}),
        TargetOpt::Mode(m) => json!({"mode": m}),
    }
}

/// Perfect-information dump of public `State` fields. RNG words and pending
/// work frames are omitted — they are not `Serialize` and not on the snapshot
/// contract. Listed in the PR as an M5 gap.
pub fn full_state_json(state: &State) -> Value {
    json!({
        "active": player_str(state.active),
        "first": player_str(state.first),
        "phase": phase_str(&state.phase),
        "turn": state.turn,
        "winner": state.winner.map(|w| Value::String(w.as_str().into())).unwrap_or(Value::Null),
        "hash": crate::hash_u64(state),
        "next_instance": state.next_instance,
        "step_counter": state.step_counter,
        "players": {
            "a": full_player(&state.players[0]),
            "b": full_player(&state.players[1]),
        },
        "snapshot": serde_json::to_value(arena_engine::snapshot(state)).unwrap_or(Value::Null),
    })
}

fn full_player(p: &arena_engine::PlayerState) -> Value {
    json!({
        "leader_defense": p.leader_defense,
        "leader_max": p.leader_max,
        "pp": p.pp,
        "pp_max": p.pp_max,
        "pp_bonus_active": p.bonus_pp.active,
        "pp_bonus_early": p.bonus_pp.early_charge,
        "pp_bonus_late": p.bonus_pp.late_charge,
        "pp_bonus_locked": p.bonus_pp.locked,
        "ep": p.ep,
        "sep": p.sep,
        "evolves_used": p.evolves_used,
        "evolved_this_turn": p.evolved_this_turn,
        "shadows": p.shadows,
        "combo": p.combo,
        "earth": p.earth,
        "faith": p.faith,
        "rally": p.rally,
        "turns_taken": p.turns_taken,
        "is_second": p.is_second,
        "attacked_leader_this_turn": p.attacked_leader_this_turn,
        "attacked_leader_last_turn": p.attacked_leader_last_turn,
        "hand": p.hand.iter().map(full_instance).collect::<Vec<_>>(),
        "field": p.field.iter().map(|s| s.as_ref().map(full_instance)).collect::<Vec<_>>(),
        "deck": p.deck.iter().map(full_instance).collect::<Vec<_>>(),
        "cemetery": p.cemetery.iter().map(full_instance).collect::<Vec<_>>(),
        "banished": p.banished.iter().map(full_instance).collect::<Vec<_>>(),
        "crests": p.crests.iter().map(|c| json!({
            "id": c.id,
            "countdown": c.countdown,
            "faith": c.faith,
        })).collect::<Vec<_>>(),
        "destroyed_history": p.destroyed_history.iter().map(|d| json!({
            "card": d.card.as_str(),
            "base_cost": d.base_cost,
            "from_field": d.from_field,
            "owner": player_str(d.owner),
        })).collect::<Vec<_>>(),
        "played_this_turn": p.played_this_turn.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "enter_counts": p.enter_counts.iter().map(|(k, n)| (k.as_str(), *n)).collect::<BTreeMap<_, _>>(),
    })
}

fn full_instance(c: &arena_engine::CardInstance) -> Value {
    let mut vars = BTreeMap::new();
    for (k, n) in &c.vars {
        if *n != 0 {
            let key = match k {
                VarKey::X => "X",
                VarKey::Y => "Y",
                VarKey::Z => "Z",
            };
            vars.insert(key, *n);
        }
    }
    json!({
        "id": c.id,
        "card": c.card.as_str(),
        "name": c.name,
        "kind": kind_str(c.kind),
        "class": class_str(c.class),
        "cost": c.cost,
        "base_cost": c.base_cost,
        "attack": c.attack,
        "defense": c.defense,
        "max_defense": c.max_defense,
        "evolved": c.evolved,
        "super_evolved": c.super_evolved,
        "skybound": c.skybound,
        "countdown": c.countdown,
        "spellboost_count": c.spellboost_count,
        "traits": c.traits.snapshot_tags(),
        "granted": c.granted.iter().map(|a| a.snapshot_tag()).collect::<Vec<_>>(),
        "vars": vars,
        "flags": {
            "was_fused": c.flags.was_fused,
            "ambush_active": c.flags.ambush_active,
            "summoning_sick": c.flags.summoning_sick,
            "attacked_this_turn": c.flags.attacked_this_turn,
            "attacks_left": c.flags.attacks_left,
            "engaged_this_turn": c.flags.engaged_this_turn,
            "fused_this_turn": c.flags.fused_this_turn,
            "enhanced": c.flags.enhanced,
        },
    })
}

fn kind_str(k: CardKind) -> &'static str {
    match k {
        CardKind::Follower => "follower",
        CardKind::Spell => "spell",
        CardKind::Amulet => "amulet",
    }
}

fn class_str(c: Class) -> &'static str {
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

/// Compile-time proof the engine types the matchup runner needs are `Send`.
/// `CardDb: Sync` lets rayon share one copy across workers.
pub fn assert_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<State>();
    assert_send::<CardDb>();
    assert_send::<CardId>();
    assert_sync::<CardDb>();
    assert_sync::<CardId>();
}
