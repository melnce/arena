//! Closed event list from `docs/engine-api.md`.

use crate::card::{CardId, TriggerTag};
use crate::ids::{PlayerId, Slot};
use crate::state::{ChoiceNode, PlayForm};
use crate::trace::PickWhat;

#[derive(Debug, Clone)]
pub enum Event {
    Draw {
        player: PlayerId,
        card: CardId,
    },
    Mulligan {
        player: PlayerId,
        swapped: Vec<CardId>,
    },
    Play {
        player: PlayerId,
        card: CardId,
        form: PlayForm,
    },
    Summon {
        player: PlayerId,
        card: CardId,
        slot: Slot,
    },
    Enter {
        slot: Slot,
    },
    Damage {
        target: EventTarget,
        amount: i32,
        lethal: bool,
    },
    Restore {
        target: EventTarget,
        amount: i32,
    },
    Destroy {
        slot: Slot,
        card: CardId,
    },
    Banish {
        card: CardId,
        from: ZoneLabel,
    },
    Transform {
        slot: Slot,
        into: CardId,
    },
    Evolve {
        slot: Slot,
        super_evolve: bool,
        granted: bool,
    },
    TriggerFired {
        on: TriggerTag,
    },
    ChoiceOffered {
        player: PlayerId,
        node: ChoiceNode,
    },
    RandomPick {
        what: PickWhat,
    },
    Counter {
        key: String,
        value: i32,
    },
    CrestGain {
        player: PlayerId,
        id: String,
    },
    CrestRemove {
        player: PlayerId,
        id: String,
    },
    Fuse {
        host: CardId,
        partners: Vec<CardId>,
    },
    TurnStart {
        player: PlayerId,
        turn: u32,
    },
    TurnEnd {
        player: PlayerId,
    },
    Win {
        player: PlayerId,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum EventTarget {
    Leader(PlayerId),
    Slot(PlayerId, Slot),
}

#[derive(Debug, Clone, Copy)]
pub enum ZoneLabel {
    Field,
    Hand,
    Deck,
    Cemetery,
    Crests,
}
