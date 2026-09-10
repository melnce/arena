//! CanonicalState JSON — `docs/trace-format.md`. Keys sorted; five field slots.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::card::{CardId, VarKey};
use crate::ids::PlayerId;
use crate::state::{Phase, PlayerState, State, FIELD_SIZE};
use crate::trace::{fnv1a64, sort_json};

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalState {
    pub active: String,
    pub phase: String,
    pub players: CanonicalPlayers,
    pub turn: u32,
    pub winner: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalPlayers {
    pub a: CanonicalPlayer,
    pub b: CanonicalPlayer,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalPlayer {
    pub banished: BTreeMap<String, u32>,
    pub cemetery: BTreeMap<String, u32>,
    pub combo: i32,
    pub crests: Vec<CanonicalCrest>,
    pub deck: BTreeMap<String, u32>,
    pub earth: i32,
    pub ep: i32,
    pub evolves_used: i32,
    pub faith: i32,
    pub field: [Option<CanonicalField>; 5],
    pub hand: Vec<CanonicalHand>,
    pub leader_defense: i32,
    pub leader_max: i32,
    pub pp: i32,
    pub pp_bonus: i32,
    pub pp_max: i32,
    pub rally: i32,
    pub sep: i32,
    pub shadows: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalCrest {
    pub countdown: Option<i32>,
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalHand {
    pub card: String,
    pub cost: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vars: Option<BTreeMap<String, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skybound: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalField {
    pub attack: i32,
    pub attacks_left: i32,
    pub can_attack: bool,
    pub card: String,
    pub countdown: Option<i32>,
    pub defense: i32,
    pub evolved: bool,
    pub max_defense: i32,
    #[serde(rename = "super")]
    pub super_evolved: bool,
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vars: Option<BTreeMap<String, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granted: Option<Vec<String>>,
}

fn multiset(cards: &[crate::state::CardInstance]) -> BTreeMap<String, u32> {
    let mut m = BTreeMap::new();
    for c in cards {
        *m.entry(c.card.as_str()).or_insert(0) += 1;
    }
    m
}

/// Runtime grants only: instance trigger tags minus the printed card's tags.
/// Omitted when empty — `docs/trace-format.md` Conventions.
fn granted_tags(c: &crate::state::CardInstance) -> Option<Vec<String>> {
    let mut tags: std::collections::BTreeSet<String> = c
        .granted
        .iter()
        .map(|a| a.snapshot_tag().to_string())
        .collect();
    for printed in &c.printed_tags {
        tags.remove(printed);
    }
    if tags.is_empty() {
        None
    } else {
        Some(tags.into_iter().collect())
    }
}

fn vars_map(v: &BTreeMap<VarKey, i32>) -> Option<BTreeMap<String, i32>> {
    if v.is_empty() {
        return None;
    }
    let mut m = BTreeMap::new();
    for (k, n) in v {
        let key = match k {
            VarKey::X => "X",
            VarKey::Y => "Y",
            VarKey::Z => "Z",
        };
        m.insert(key.into(), *n);
    }
    Some(m)
}

fn snap_player(p: &PlayerState) -> CanonicalPlayer {
    let mut field: [Option<CanonicalField>; FIELD_SIZE] = Default::default();
    for (i, slot) in p.field.iter().enumerate() {
        field[i] = slot.as_ref().map(|c| CanonicalField {
            attack: c.attack,
            attacks_left: c.flags.attacks_left,
            can_attack: can_attack_now(c),
            card: c.card.as_str(),
            countdown: c.countdown,
            defense: c.defense,
            evolved: c.evolved,
            max_defense: c.max_defense,
            super_evolved: c.super_evolved,
            traits: c.traits.snapshot_tags(),
            vars: vars_map(&c.vars),
            granted: granted_tags(c),
        });
    }
    let hand = p
        .hand
        .iter()
        .map(|c| CanonicalHand {
            card: c.card.as_str(),
            cost: c.cost,
            vars: vars_map(&c.vars),
            skybound: if c.skybound != 0 {
                Some(c.skybound)
            } else {
                None
            },
        })
        .collect();
    CanonicalPlayer {
        banished: multiset(&p.banished),
        cemetery: multiset(&p.cemetery),
        combo: p.combo,
        crests: p
            .crests
            .iter()
            .map(|c| CanonicalCrest {
                countdown: c.countdown,
                id: c.id.clone(),
            })
            .collect(),
        deck: multiset(&p.deck),
        earth: p.earth,
        ep: p.ep,
        evolves_used: p.evolves_used,
        faith: p.faith,
        field,
        hand,
        leader_defense: p.leader_defense,
        leader_max: p.leader_max,
        pp: p.pp,
        pp_bonus: i32::from(p.bonus_pp.active),
        pp_max: p.pp_max,
        rally: p.rally,
        sep: p.sep,
        shadows: p.shadows,
    }
}

fn can_attack_now(c: &crate::state::CardInstance) -> bool {
    if c.kind != crate::card::CardKind::Follower {
        return false;
    }
    if c.flags.attacks_left <= 0 {
        return false;
    }
    if c.traits.cant_attack_followers == Some(true) && c.traits.cant_attack_leader == Some(true) {
        return false;
    }
    if c.flags.summoning_sick && !c.is_storm() && !c.is_rush() && !c.evolved {
        return false;
    }
    true
}

fn phase_str(p: &Phase) -> String {
    match p {
        Phase::Mulligan { .. } => "mulligan".into(),
        Phase::Main | Phase::Combat => "main".into(),
        Phase::Choice { .. } => "choice".into(),
        Phase::End => "end".into(),
        Phase::Terminal => "terminal".into(),
    }
}

pub fn snapshot(state: &State) -> CanonicalState {
    CanonicalState {
        active: state.active.as_str().into(),
        phase: phase_str(&state.phase),
        players: CanonicalPlayers {
            a: snap_player(&state.players[0]),
            b: snap_player(&state.players[1]),
        },
        turn: state.turn,
        winner: state.winner.map(|w| w.as_str().to_string()),
    }
}

pub fn snapshot_json(state: &State) -> serde_json::Value {
    let v = serde_json::to_value(snapshot(state)).expect("canonical state");
    sort_json(v)
}

pub fn hash(state: &State) -> u64 {
    let json = snapshot_json(state);
    let bytes = serde_json::to_vec(&json).expect("canonical json");
    fnv1a64(&bytes)
}

pub fn card_id_key(id: CardId) -> String {
    id.as_str()
}

pub fn player_letter(p: PlayerId) -> &'static str {
    p.as_str()
}
