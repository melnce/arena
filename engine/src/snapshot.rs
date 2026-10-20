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
    #[serde(skip_serializing_if = "Option::is_none")]
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

/// Earth Sigil's glossary `cantBeDestroyedByAbilities` stays on the card and
/// instance; CanonicalState still lists only `aura`, matching the old engine.
fn field_trait_tags(c: &crate::state::CardInstance) -> Vec<String> {
    let mut t = c.traits.snapshot_tags();
    if c.is_earth_sigil() {
        t.retain(|s| s != "cantBeDestroyedByAbilities");
    }
    t
}

/// Runtime grants only: instance trigger tags minus the printed card's tags.
/// Omitted when empty — `docs/trace-format.md` Conventions.
fn granted_tags(c: &crate::state::CardInstance) -> Option<Vec<String>> {
    let mut tags: std::collections::BTreeSet<String> = c
        .granted
        .iter()
        .map(|a| a.snapshot_tag().to_string())
        .collect();
    for printed in &c.statics.printed_tags {
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
        if *n == 0 {
            continue;
        }
        let key = match k {
            VarKey::X => "X",
            VarKey::Y => "Y",
            VarKey::Z => "Z",
        };
        m.insert(key.into(), *n);
    }
    if m.is_empty() {
        None
    } else {
        Some(m)
    }
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
            traits: field_trait_tags(c),
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
    let snap = snapshot(state);
    let mut w = Fnv1aWriter(0xcbf2_9ce4_8422_2325);
    serde_json::to_writer(&mut w, &SortedState::of(&snap)).expect("canonical json");
    w.0
}

/// The old route, kept to check [`hash`] against: `serde_json::Value`, sorted, serialized, then FNV-1a.
pub fn hash_via_value(state: &State) -> u64 {
    let json = snapshot_json(state);
    let bytes = serde_json::to_vec(&json).expect("canonical json");
    fnv1a64(&bytes)
}

/// FNV-1a 64 as an `io::Write` sink: the same value as `fnv1a64` of everything written.
struct Fnv1aWriter(u64);

impl std::io::Write for Fnv1aWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut h = self.0;
        for &b in buf {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        self.0 = h;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// `snapshot_json` turns every struct into a map, and serde_json's maps are sorted by key, so the bytes `hash` covers
// have every object's keys in byte order. These views serialize a `CanonicalState` with their fields declared in that
// order, which gives the same bytes without building the `Value` tree. `CanonicalState`, `CanonicalPlayers`,
// `CanonicalPlayer` and `CanonicalCrest` already declare their fields sorted; `CanonicalHand` and `CanonicalField`
// do not, so they get their own view.
#[derive(Serialize)]
struct SortedState<'a> {
    active: &'a str,
    phase: &'a str,
    players: SortedPlayers<'a>,
    turn: u32,
    winner: &'a Option<String>,
}

#[derive(Serialize)]
struct SortedPlayers<'a> {
    a: SortedPlayer<'a>,
    b: SortedPlayer<'a>,
}

#[derive(Serialize)]
struct SortedPlayer<'a> {
    banished: &'a BTreeMap<String, u32>,
    cemetery: &'a BTreeMap<String, u32>,
    combo: i32,
    crests: &'a [CanonicalCrest],
    deck: &'a BTreeMap<String, u32>,
    earth: i32,
    ep: i32,
    evolves_used: i32,
    faith: i32,
    field: [Option<SortedField<'a>>; 5],
    hand: Vec<SortedHand<'a>>,
    leader_defense: i32,
    leader_max: i32,
    pp: i32,
    pp_bonus: i32,
    pp_max: i32,
    rally: i32,
    sep: i32,
    shadows: i32,
}

#[derive(Serialize)]
struct SortedHand<'a> {
    card: &'a str,
    cost: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    skybound: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vars: Option<&'a BTreeMap<String, i32>>,
}

#[derive(Serialize)]
struct SortedField<'a> {
    attack: i32,
    attacks_left: i32,
    can_attack: bool,
    card: &'a str,
    countdown: Option<i32>,
    defense: i32,
    evolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    granted: Option<&'a Vec<String>>,
    max_defense: i32,
    #[serde(rename = "super")]
    super_evolved: bool,
    traits: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    vars: Option<&'a BTreeMap<String, i32>>,
}

impl<'a> SortedState<'a> {
    fn of(s: &'a CanonicalState) -> Self {
        SortedState {
            active: &s.active,
            phase: &s.phase,
            players: SortedPlayers {
                a: SortedPlayer::of(&s.players.a),
                b: SortedPlayer::of(&s.players.b),
            },
            turn: s.turn,
            winner: &s.winner,
        }
    }
}

impl<'a> SortedPlayer<'a> {
    fn of(p: &'a CanonicalPlayer) -> Self {
        let mut field: [Option<SortedField<'a>>; 5] = Default::default();
        for (i, slot) in p.field.iter().enumerate() {
            field[i] = slot.as_ref().map(|f| SortedField {
                attack: f.attack,
                attacks_left: f.attacks_left,
                can_attack: f.can_attack,
                card: &f.card,
                countdown: f.countdown,
                defense: f.defense,
                evolved: f.evolved,
                granted: f.granted.as_ref(),
                max_defense: f.max_defense,
                super_evolved: f.super_evolved,
                traits: &f.traits,
                vars: f.vars.as_ref(),
            });
        }
        SortedPlayer {
            banished: &p.banished,
            cemetery: &p.cemetery,
            combo: p.combo,
            crests: &p.crests,
            deck: &p.deck,
            earth: p.earth,
            ep: p.ep,
            evolves_used: p.evolves_used,
            faith: p.faith,
            field,
            hand: p
                .hand
                .iter()
                .map(|h| SortedHand {
                    card: &h.card,
                    cost: h.cost,
                    skybound: h.skybound,
                    vars: h.vars.as_ref(),
                })
                .collect(),
            leader_defense: p.leader_defense,
            leader_max: p.leader_max,
            pp: p.pp,
            pp_bonus: p.pp_bonus,
            pp_max: p.pp_max,
            rally: p.rally,
            sep: p.sep,
            shadows: p.shadows,
        }
    }
}

pub fn card_id_key(id: CardId) -> String {
    id.as_str()
}

pub fn player_letter(p: PlayerId) -> &'static str {
    p.as_str()
}
