//! Flat cloneable match state — `docs/engine-api.md`.

use std::collections::{BTreeMap, BTreeSet};

use crate::card::{Ability, Card, CardId, CardKind, Class, Traits, Tribe, TriggerTag, VarKey};
use crate::ids::{First, PlayerId};
use crate::rng::GameRng;

pub const HAND_LIMIT: usize = 9;
pub const FIELD_SIZE: usize = 5;
pub const DECK_SIZE: usize = 40;
pub const LEADER_MAX: i32 = 20;
pub const PP_CAP: i32 = 10;
pub const CREST_CAP: usize = 5;
pub const STEP_CEILING: u64 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceFlags {
    pub was_fused: bool,
    /// α remembers β / γ kinds across turns (card ids).
    pub fused_kinds: Vec<CardId>,
    pub ambush_active: bool,
    pub summoning_sick: bool,
    pub attacked_this_turn: bool,
    pub attacks_left: i32,
    pub engaged_this_turn: bool,
    pub fused_this_turn: bool,
    pub eot_attack: i32,
    pub eot_defense: i32,
    pub eot_cost: Option<i32>,
}

impl Default for InstanceFlags {
    fn default() -> Self {
        Self {
            was_fused: false,
            fused_kinds: Vec::new(),
            ambush_active: false,
            summoning_sick: true,
            attacked_this_turn: false,
            attacks_left: 1,
            engaged_this_turn: false,
            fused_this_turn: false,
            eot_attack: 0,
            eot_defense: 0,
            eot_cost: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CardInstance {
    pub id: u32,
    pub card: CardId,
    pub cost: i32,
    pub base_cost: i32,
    pub attack: i32,
    pub defense: i32,
    pub max_defense: i32,
    pub evolved: bool,
    pub super_evolved: bool,
    pub traits: Traits,
    pub granted: Vec<Ability>,
    /// Count of `Ability::When` entries in `granted`. Grants are dynamic and
    /// rare; a non-zero count forces the instance into the `when` scan.
    pub granted_whens: u8,
    /// Printed trigger tags at construction; `granted` snapshot is instance − this.
    pub printed_tags: BTreeSet<String>,
    pub flags: InstanceFlags,
    pub vars: BTreeMap<VarKey, i32>,
    pub skybound: i32,
    pub countdown: Option<i32>,
    pub spellboost_count: i32,
    /// Printed type; Accelerate play rewrites the corpse to Spell.
    pub kind: CardKind,
    pub class: Class,
    pub tribes: Vec<Tribe>,
    pub name: String,
    pub once_used: Vec<TriggerTag>,
}

impl CardInstance {
    pub fn from_card(card: &Card, id: u32) -> Self {
        let traits = card.traits().cloned().unwrap_or_default();
        let ambush = traits.ambush == Some(true);
        let attacks = traits.attacks_per_turn.unwrap_or(1);
        let mut vars = BTreeMap::new();
        if let Some(v) = card.vars() {
            vars.insert(VarKey::X, v.x);
        }
        Self {
            id,
            card: card.id(),
            cost: card.cost(),
            base_cost: card.cost(),
            attack: card.attack(),
            defense: card.defense(),
            max_defense: card.defense(),
            evolved: false,
            super_evolved: false,
            traits,
            granted: Vec::new(),
            granted_whens: 0,
            printed_tags: card.printed_trigger_tags(),
            flags: InstanceFlags {
                ambush_active: ambush,
                attacks_left: attacks,
                summoning_sick: true,
                ..InstanceFlags::default()
            },
            vars,
            skybound: 0,
            countdown: card.countdown(),
            spellboost_count: 0,
            kind: card.kind(),
            class: card.class(),
            tribes: card.tribes().to_vec(),
            name: card.name().to_string(),
            once_used: Vec::new(),
        }
    }

    pub fn has_trait(&self, ward: fn(&Traits) -> Option<bool>) -> bool {
        ward(&self.traits) == Some(true)
    }

    pub fn is_ward(&self) -> bool {
        self.traits.ward == Some(true)
    }
    pub fn is_storm(&self) -> bool {
        self.traits.storm == Some(true)
    }
    pub fn is_rush(&self) -> bool {
        self.traits.rush == Some(true)
    }
    pub fn is_bane(&self) -> bool {
        self.traits.bane == Some(true)
    }
    pub fn is_drain(&self) -> bool {
        self.traits.drain == Some(true)
    }
    pub fn is_barrier(&self) -> bool {
        self.traits.barrier == Some(true)
    }
    pub fn is_aura(&self) -> bool {
        self.traits.aura == Some(true)
    }
    pub fn is_intimidate(&self) -> bool {
        self.traits.intimidate == Some(true)
    }
    pub fn ambush_blocks(&self) -> bool {
        self.flags.ambush_active && self.traits.ambush == Some(true)
    }
    pub fn ignores_ward(&self) -> bool {
        self.traits.ignores_ward == Some(true)
    }
    pub fn cant_be_played(&self) -> bool {
        self.traits.cant_be_played == Some(true)
    }
    pub fn is_earth_sigil(&self) -> bool {
        self.tribes.contains(&Tribe::EarthSigil)
    }
    pub fn damaged(&self) -> bool {
        self.defense < self.max_defense
    }

    pub fn grant_ability(&mut self, ability: Ability) {
        if matches!(ability, Ability::When { .. }) {
            self.granted_whens = self.granted_whens.saturating_add(1);
        }
        self.granted.push(ability);
    }

    pub fn remove_granted_abilities(&mut self, on: Option<&[TriggerTag]>) {
        match on {
            Some(tags) => {
                self.granted.retain(|a| !tags.contains(&a.tag()));
                for t in tags {
                    if let TriggerTag::LastWords = t {
                        self.printed_tags.remove("lastWords");
                    }
                }
            }
            None => {
                self.granted.clear();
                self.printed_tags.clear();
            }
        }
        self.granted_whens = self
            .granted
            .iter()
            .filter(|a| matches!(a, Ability::When { .. }))
            .count() as u8;
    }
}

#[derive(Debug, Clone)]
pub struct CrestInstance {
    pub id: String,
    pub countdown: Option<i32>,
    pub faith: bool,
    pub once_used: Vec<TriggerTag>,
    pub granted_order: u32,
}

#[derive(Debug, Clone)]
pub struct LeaderMod {
    pub max_defense: Option<i32>,
    pub damage_cap: Option<i32>,
    pub damage_taken_bonus: i32,
    pub until: Option<crate::card::Until>,
}

#[derive(Debug, Clone)]
pub struct DestroyedRecord {
    pub card: CardId,
    pub base_cost: i32,
    pub kind: CardKind,
    pub owner: PlayerId,
    pub from_field: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BonusPp {
    pub early_charge: bool,
    pub late_charge: bool,
    /// Extra orb is currently on (usable PP may be max+1).
    pub active: bool,
    /// The bonus orb was spent this turn (regular first, orb last).
    /// Cancel is a no-op; the toggle is not offered again; EOT commits.
    /// Old engine `bonusPp.ts` / `canToggleSecondPlayerBonusPp`.
    pub locked: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub leader_defense: i32,
    pub leader_max: i32,
    pub pp: i32,
    pub pp_max: i32,
    pub bonus_pp: BonusPp,
    pub ep: i32,
    pub sep: i32,
    pub evolves_used: i32,
    /// Manual EP/SEP evolve this turn locks both kinds until the next turn.
    /// Effect-granted evolves do not set this. Old engine `canEvolve` / `usedThisTurn`.
    pub evolved_this_turn: bool,
    pub shadows: i32,
    pub combo: i32,
    pub earth: i32,
    pub earth_slot: Option<u8>,
    pub faith: i32,
    pub rally: i32,
    pub leader_mods: Vec<LeaderMod>,
    pub crests: Vec<CrestInstance>,
    pub hand: Vec<CardInstance>,
    pub field: [Option<CardInstance>; FIELD_SIZE],
    pub deck: Vec<CardInstance>,
    pub cemetery: Vec<CardInstance>,
    pub banished: Vec<CardInstance>,
    pub destroyed_history: Vec<DestroyedRecord>,
    pub played_this_turn: Vec<CardId>,
    pub played_base_costs_this_match: Vec<i32>,
    pub attacked_leader_this_turn: bool,
    pub attacked_leader_last_turn: bool,
    pub turns_taken: u32,
    pub is_second: bool,
    pub enter_counts: BTreeMap<CardId, i32>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            leader_defense: LEADER_MAX,
            leader_max: LEADER_MAX,
            pp: 0,
            pp_max: 0,
            bonus_pp: BonusPp::default(),
            ep: 2,
            sep: 2,
            evolves_used: 0,
            evolved_this_turn: false,
            shadows: 0,
            combo: 0,
            earth: 0,
            earth_slot: None,
            faith: 0,
            rally: 0,
            leader_mods: Vec::new(),
            crests: Vec::new(),
            hand: Vec::new(),
            field: Default::default(),
            deck: Vec::new(),
            cemetery: Vec::new(),
            banished: Vec::new(),
            destroyed_history: Vec::new(),
            played_this_turn: Vec::new(),
            played_base_costs_this_match: Vec::new(),
            attacked_leader_this_turn: false,
            attacked_leader_last_turn: false,
            turns_taken: 0,
            is_second: false,
            enter_counts: BTreeMap::new(),
        }
    }

    pub fn usable_pp(&self) -> i32 {
        self.pp + i32::from(self.bonus_pp.active)
    }

    pub fn field_count(&self) -> usize {
        self.field.iter().filter(|s| s.is_some()).count()
    }

    pub fn field_free(&self) -> usize {
        FIELD_SIZE - self.field_count()
    }

    pub fn compact_field(&mut self) {
        let mut kept: Vec<CardInstance> = self.field.iter_mut().filter_map(|s| s.take()).collect();
        self.field = Default::default();
        for (i, inst) in kept.drain(..).enumerate() {
            if i < FIELD_SIZE {
                self.field[i] = Some(inst);
            }
        }
        self.earth_slot = self.field.iter().enumerate().find_map(|(i, s)| {
            s.as_ref()
                .and_then(|c| c.is_earth_sigil().then_some(i as u8))
        });
    }

    pub fn first_empty_slot(&self) -> Option<u8> {
        self.field.iter().position(|s| s.is_none()).map(|i| i as u8)
    }

    pub fn spend_pp(&mut self, cost: i32) {
        let mut left = cost;
        let from_reg = left.min(self.pp);
        self.pp -= from_reg;
        left -= from_reg;
        // Regular orbs first; the bonus orb is spent last.
        if left > 0 && self.bonus_pp.active {
            self.bonus_pp.active = false;
            self.bonus_pp.locked = true;
        }
    }

    pub fn recover_pp(&mut self, amount: i32) {
        self.pp = (self.pp + amount).min(self.pp_max);
    }

    pub fn damage_cap(&self) -> Option<i32> {
        self.leader_mods.iter().filter_map(|m| m.damage_cap).min()
    }

    pub fn damage_taken_bonus(&self) -> i32 {
        self.leader_mods.iter().map(|m| m.damage_taken_bonus).sum()
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum Phase {
    Mulligan { player: PlayerId },
    Main,
    Combat,
    Choice { player: PlayerId, node: ChoiceNode },
    End,
    Terminal,
}

#[derive(Debug, Clone)]
pub enum ChoiceNode {
    Targets {
        options: Vec<TargetOpt>,
        pending: PendingChoice,
    },
    Modes {
        options: Vec<u8>,
        pending: PendingChoice,
    },
    Cards {
        options: Vec<CardId>,
        pending: PendingChoice,
    },
    FusePartners {
        host: u8,
        options: Vec<u8>,
        picked: Vec<u8>,
    },
    MultiPick {
        options: Vec<TargetOpt>,
        picked: Vec<u8>,
        pending: PendingChoice,
    },
}

#[derive(Debug, Clone)]
pub enum TargetOpt {
    Slot { player: PlayerId, slot: u8 },
    Leader { player: PlayerId },
    Hand { player: PlayerId, pos: u8 },
    Card(CardId),
    Mode(u8),
}

/// What to resume after a `Choose`.
#[derive(Debug, Clone)]
pub struct PendingChoice {
    pub kind: PendingKind,
    /// Remaining sequential `pick: choose` selections including the current one.
    /// Fuse Confirm is unrelated. `1` = single pick (default).
    pub remaining: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingKind {
    PlaySelect,
    EffectSelect,
    ModeSelect,
    DiscardSelect,
    EvolveSelect,
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub seed: u64,
    pub deck_a: Vec<CardId>,
    pub deck_b: Vec<CardId>,
    pub first: First,
    /// When set, `new_game` deals these ids from the deck multisets in
    /// draw order and does not roll opening-hand draws (`docs/trace-format.md`).
    pub opening_hands: Option<OpeningHands>,
}

/// Pre-mulligan opening hands in draw order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningHands {
    pub a: Vec<CardId>,
    pub b: Vec<CardId>,
}

/// A bound `as` target, stored by instance id so later `ref`s survive compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundRef {
    Field { player: PlayerId, id: u32 },
    Leader { player: PlayerId },
    Hand { player: PlayerId, id: u32 },
    Card(CardId),
}

#[derive(Debug, Clone)]
pub struct State {
    pub players: [PlayerState; 2],
    pub turn: u32,
    pub active: PlayerId,
    pub first: PlayerId,
    pub phase: Phase,
    pub winner: Option<PlayerId>,
    pub rng: GameRng,
    pub step_counter: u64,
    pub next_instance: u32,
    pub crest_order: u32,
    /// Picks recorded while applying the current action.
    pub picks: Vec<crate::trace::Pick>,
    pub pending_work: Vec<WorkFrame>,
    pub queue: Vec<QueuedTrigger>,
    pub suppress_last_words: bool,
    /// Per-resolution `as` / `bound` map. Shared by evolve + superEvolve
    /// of one card when both fire. Cleared at each new resolution.
    pub bindings: BTreeMap<String, Vec<BoundRef>>,
    /// Played follower: Rally increments when the play sequence (Fanfare)
    /// completes, not at entry. Summons increment at entry.
    pub pending_play_rally: Option<PlayerId>,
    /// Subject of the current `When` event (`pick: entering`, etc.).
    pub event_subject: Option<TargetOpt>,
}

#[derive(Debug, Clone)]
pub enum WorkFrame {
    Effects {
        controller: PlayerId,
        source: SourceRef,
        effects: Vec<crate::card::Effect>,
        index: usize,
        subject: Option<TargetOpt>,
    },
    Aftermath(Aftermath),
}

#[derive(Debug, Clone)]
pub enum Aftermath {
    AfterPlay {
        player: PlayerId,
        form: PlayForm,
        card: CardId,
    },
    AfterCombat {
        attacker_player: PlayerId,
        attacker_id: u32,
        target: crate::ids::AttackTarget,
        defender_id: Option<u32>,
        knockback: bool,
    },
    AfterEvolve {
        player: PlayerId,
        slot: u8,
        supered: bool,
        granted: bool,
    },
    DrainQueue,
    RestoreBindings(BTreeMap<String, Vec<BoundRef>>),
    ContinueTurnStart {
        step: u8,
    },
    ContinueTurnEnd {
        step: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayForm {
    Normal,
    Enhance { paid: i32 },
    Accelerate { paid: i32 },
    Crystallize { paid: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRef {
    Field { player: PlayerId, id: u32 },
    Hand { player: PlayerId, id: u32 },
    Crest { player: PlayerId, index: usize },
    Spell { player: PlayerId, card: CardId },
    Leader { player: PlayerId },
}

#[derive(Debug, Clone)]
pub struct QueuedTrigger {
    pub category: u8,
    pub entry: u32,
    pub printed_order: u8,
    pub controller: PlayerId,
    pub source: SourceRef,
    pub tag: TriggerTag,
    pub effects: Vec<crate::card::Effect>,
    /// Entering/attacking subject captured at enqueue so sequential enters
    /// (Adahime fanfare summons, Macmillan's 3 Zombies) each keep `pick: entering`.
    pub subject: Option<TargetOpt>,
}

impl State {
    pub fn player(&self, p: PlayerId) -> &PlayerState {
        &self.players[p.idx()]
    }

    pub fn player_mut(&mut self, p: PlayerId) -> &mut PlayerState {
        &mut self.players[p.idx()]
    }

    pub fn alloc_id(&mut self) -> u32 {
        let id = self.next_instance;
        self.next_instance += 1;
        id
    }

    pub fn find_field(&self, player: PlayerId, id: u32) -> Option<u8> {
        self.player(player)
            .field
            .iter()
            .position(|s| s.as_ref().map(|c| c.id) == Some(id))
            .map(|i| i as u8)
    }

    pub fn field_inst(&self, player: PlayerId, slot: u8) -> Option<&CardInstance> {
        self.player(player).field.get(slot as usize)?.as_ref()
    }

    pub fn field_inst_mut(&mut self, player: PlayerId, slot: u8) -> Option<&mut CardInstance> {
        self.player_mut(player)
            .field
            .get_mut(slot as usize)?
            .as_mut()
    }

    pub fn bump_step(&mut self) -> Result<(), crate::error::Illegal> {
        self.step_counter += 1;
        if self.step_counter > STEP_CEILING {
            Err(crate::error::Illegal::StepCeiling)
        } else {
            Ok(())
        }
    }
}
