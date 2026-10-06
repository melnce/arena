//! Closed event list from `docs/engine-api.md`.

use crate::card::CardId;
use crate::ids::{PlayerId, Slot};
use crate::state::{ChoiceNode, PlayForm, SourceRef, TargetOpt};
use crate::trace::PickWhat;

/// Event-only source marker for `resolve` (wraps `SourceRef` or combat attacker).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSource {
    Ref(SourceRef),
    Combat { player: PlayerId, id: u32 },
}

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
        unit: Option<u32>,
    },
    Restore {
        target: EventTarget,
        amount: i32,
        unit: Option<u32>,
    },
    Destroy {
        slot: Slot,
        card: CardId,
        player: PlayerId,
        id: u32,
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
    Resolve {
        source: EventSource,
    },
    ChoiceOffered {
        player: PlayerId,
        node: ChoiceNode,
        source: Option<SourceRef>,
    },
    RandomPick {
        what: PickWhat,
        target: TargetOpt,
        unit: Option<u32>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
