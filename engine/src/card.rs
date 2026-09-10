//! Serde types for every definition in `schema/cards.schema.json`.
//! Field names match the schema (camelCase JSON). All closed enums;
//! every object is `deny_unknown_fields`.

use std::collections::BTreeSet;
use std::fmt;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

// ---------------------------------------------------------------------------
// Newtypes / closed scalars
// ---------------------------------------------------------------------------

/// Cygames id, 8 digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(pub u32);

impl CardId {
    pub fn as_str(self) -> String {
        format!("{:08}", self.0)
    }

    pub fn parse(s: &str) -> Option<Self> {
        if s.len() == 8 && s.bytes().all(|b| b.is_ascii_digit()) {
            s.parse::<u32>().ok().map(CardId)
        } else {
            None
        }
    }
}

impl fmt::Display for CardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08}", self.0)
    }
}

impl Serialize for CardId {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self.as_str())
    }
}

impl<'de> Deserialize<'de> for CardId {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        CardId::parse(&s).ok_or_else(|| de::Error::custom(format!("invalid CardId {s}")))
    }
}

/// `crest:XXXXXXXX` only — `faith:` is rejected (schema pattern on `crest.gain`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrestGainId(pub String);

impl Serialize for CrestGainId {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CrestGainId {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        let ok =
            s.starts_with("crest:") && s.len() == 14 && s[6..].bytes().all(|b| b.is_ascii_digit());
        if ok {
            Ok(CrestGainId(s))
        } else {
            Err(de::Error::custom(format!(
                "crest.gain must match ^crest:[0-9]{{8}}$, got {s}"
            )))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Class {
    Neutral,
    Forestcraft,
    Swordcraft,
    Runecraft,
    Dragoncraft,
    Abysscraft,
    Havencraft,
    Portalcraft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Rarity {
    Bronze,
    Silver,
    Gold,
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tribe {
    Anathema,
    Artifact,
    Departed,
    #[serde(rename = "earth sigil")]
    EarthSigil,
    Encroacher,
    Golem,
    Loot,
    Marine,
    Mysteria,
    Officer,
    Pixie,
    Puppetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CardKind {
    Follower,
    Spell,
    Amulet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VarKey {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Whose {
    Own,
    Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AbilityZone {
    Field,
    Hand,
    Deck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Ally,
    Enemy,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Zone {
    Field,
    Hand,
    Deck,
    Cemetery,
    Leader,
    Crests,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorKind {
    Follower,
    Amulet,
    Card,
    Leader,
    Character,
    Faith,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterKind {
    Follower,
    Spell,
    Amulet,
    Card,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderBy {
    Attack,
    Defense,
    Cost,
    BaseCost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TraitKey {
    Ward,
    Bane,
    Rush,
    Storm,
    Ambush,
    Aura,
    Intimidate,
    Drain,
    Barrier,
    IgnoresWard,
    CantAttackFollowers,
    CantAttackLeader,
    CantBeDestroyedByAbilities,
    CantBePlayed,
    AttacksPerTurn,
    DamageCap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerTag {
    Fanfare,
    LastWords,
    Evolve,
    SuperEvolve,
    AnyEvolve,
    AnySuperEvolve,
    Strike,
    FollowerStrike,
    Clash,
    Enter,
    Leave,
    Discarded,
    Invoked,
    Fused,
    Spellboost,
    Engage,
    StartOfTurn,
    EndOfTurn,
    When,
    Enhance,
}

impl TriggerTag {
    pub fn snapshot(self) -> &'static str {
        match self {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventName {
    AllyFollowerEnter,
    EnemyFollowerEnter,
    AllyFollowerDestroyed,
    EnemyFollowerDestroyed,
    AllyAmuletDestroyed,
    AllyCardPlayed,
    AllySpellPlayed,
    AllyFollowerAttacks,
    EnemyFollowerAttacks,
    AllyEvolve,
    AllySuperEvolve,
    AllyDraw,
    AllyEarthRite,
    AllyEngage,
    LeaderRestored,
    SelfBuffedUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReplicateKey {
    Fanfare,
    Evolve,
    SuperEvolve,
    LastWords,
    Engage,
    Strike,
    FollowerStrike,
    Clash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OptionsFrom {
    Fanfare,
    LastWords,
    Evolve,
    SuperEvolve,
    AnyEvolve,
    AnySuperEvolve,
    Strike,
    FollowerStrike,
    Clash,
    Enter,
    Leave,
    Discarded,
    Invoked,
    Fused,
    Spellboost,
    Engage,
    StartOfTurn,
    EndOfTurn,
    When,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PpAction {
    GainMax,
    Recover,
    Spend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpAction {
    Gain,
    Spend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CounterHow {
    Add,
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PayResource {
    Shadows,
    Earth,
    Pp,
    Faith,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrestPlayer {
    #[serde(rename = "self")]
    Self_,
    Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Until {
    EndOfTurn,
    EndOfOpponentTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Controller {
    #[serde(rename = "self")]
    Self_,
    Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeckPosition {
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StatWhich {
    Attack,
    Defense,
    Cost,
    BaseCost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChooseBy {
    Player,
    RandomUnused,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnOwner {
    #[serde(rename = "self")]
    Self_,
    Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FieldHasKind {
    Follower,
    Amulet,
    Card,
    Character,
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Traits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ward: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bane: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rush: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storm: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ambush: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aura: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intimidate: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drain: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barrier: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignores_ward: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cant_attack_followers: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cant_attack_leader: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cant_be_destroyed_by_abilities: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cant_be_played: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attacks_per_turn: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_cap: Option<i32>,
}

impl Traits {
    pub fn is_set(&self, key: TraitKey) -> bool {
        match key {
            TraitKey::Ward => self.ward == Some(true),
            TraitKey::Bane => self.bane == Some(true),
            TraitKey::Rush => self.rush == Some(true),
            TraitKey::Storm => self.storm == Some(true),
            TraitKey::Ambush => self.ambush == Some(true),
            TraitKey::Aura => self.aura == Some(true),
            TraitKey::Intimidate => self.intimidate == Some(true),
            TraitKey::Drain => self.drain == Some(true),
            TraitKey::Barrier => self.barrier == Some(true),
            TraitKey::IgnoresWard => self.ignores_ward == Some(true),
            TraitKey::CantAttackFollowers => self.cant_attack_followers == Some(true),
            TraitKey::CantAttackLeader => self.cant_attack_leader == Some(true),
            TraitKey::CantBeDestroyedByAbilities => {
                self.cant_be_destroyed_by_abilities == Some(true)
            }
            TraitKey::CantBePlayed => self.cant_be_played == Some(true),
            TraitKey::AttacksPerTurn => self.attacks_per_turn.unwrap_or(0) > 0,
            TraitKey::DamageCap => self.damage_cap.is_some(),
        }
    }

    pub fn merge_grant(&mut self, other: &Traits) {
        fn or_bool(dst: &mut Option<bool>, src: Option<bool>) {
            if src == Some(true) {
                *dst = Some(true);
            }
        }
        or_bool(&mut self.ward, other.ward);
        or_bool(&mut self.bane, other.bane);
        or_bool(&mut self.rush, other.rush);
        or_bool(&mut self.storm, other.storm);
        or_bool(&mut self.ambush, other.ambush);
        or_bool(&mut self.aura, other.aura);
        or_bool(&mut self.intimidate, other.intimidate);
        or_bool(&mut self.drain, other.drain);
        or_bool(&mut self.barrier, other.barrier);
        or_bool(&mut self.ignores_ward, other.ignores_ward);
        or_bool(&mut self.cant_attack_followers, other.cant_attack_followers);
        or_bool(&mut self.cant_attack_leader, other.cant_attack_leader);
        or_bool(
            &mut self.cant_be_destroyed_by_abilities,
            other.cant_be_destroyed_by_abilities,
        );
        or_bool(&mut self.cant_be_played, other.cant_be_played);
        if let Some(n) = other.attacks_per_turn {
            // "Can attack N times" grants stack as extra attacks (Verdilia +
            // Armes official Q&A: 2 + 2 printed extras → 3 attacks).
            let extra = (n - 1).max(0);
            let cur = self.attacks_per_turn.unwrap_or(1);
            self.attacks_per_turn = Some(cur + extra);
        }
        if let Some(n) = other.damage_cap {
            self.damage_cap = Some(match self.damage_cap {
                Some(c) => c.min(n),
                None => n,
            });
        }
    }

    pub fn merge_remove(&mut self, other: &Traits) {
        if other.ward == Some(true) {
            self.ward = None;
        }
        if other.bane == Some(true) {
            self.bane = None;
        }
        if other.rush == Some(true) {
            self.rush = None;
        }
        if other.storm == Some(true) {
            self.storm = None;
        }
        if other.ambush == Some(true) {
            self.ambush = None;
        }
        if other.aura == Some(true) {
            self.aura = None;
        }
        if other.intimidate == Some(true) {
            self.intimidate = None;
        }
        if other.drain == Some(true) {
            self.drain = None;
        }
        if other.barrier == Some(true) {
            self.barrier = None;
        }
        if other.ignores_ward == Some(true) {
            self.ignores_ward = None;
        }
        if other.cant_attack_followers == Some(true) {
            self.cant_attack_followers = None;
        }
        if other.cant_attack_leader == Some(true) {
            self.cant_attack_leader = None;
        }
        if other.cant_be_destroyed_by_abilities == Some(true) {
            self.cant_be_destroyed_by_abilities = None;
        }
        if other.cant_be_played == Some(true) {
            self.cant_be_played = None;
        }
        if let Some(n) = other.attacks_per_turn {
            let extra = (n - 1).max(0);
            let cur = self.attacks_per_turn.unwrap_or(1);
            let next = (cur - extra).max(1);
            self.attacks_per_turn = if next <= 1 { None } else { Some(next) };
        }
        if other.damage_cap.is_some() {
            self.damage_cap = None;
        }
    }

    /// Sorted trait tags for CanonicalState.
    pub fn snapshot_tags(&self) -> Vec<String> {
        let mut t = Vec::new();
        if self.ambush == Some(true) {
            t.push("ambush".into());
        }
        if self.aura == Some(true) {
            t.push("aura".into());
        }
        if self.bane == Some(true) {
            t.push("bane".into());
        }
        if self.barrier == Some(true) {
            t.push("barrier".into());
        }
        if self.cant_attack_followers == Some(true) {
            t.push("cantAttackFollowers".into());
        }
        if self.cant_attack_leader == Some(true) {
            t.push("cantAttackLeader".into());
        }
        if self.cant_be_destroyed_by_abilities == Some(true) {
            t.push("cantBeDestroyedByAbilities".into());
        }
        if self.cant_be_played == Some(true) {
            t.push("cantBePlayed".into());
        }
        if self.drain == Some(true) {
            t.push("drain".into());
        }
        if self.ignores_ward == Some(true) {
            t.push("ignoresWard".into());
        }
        if self.intimidate == Some(true) {
            t.push("intimidate".into());
        }
        if self.rush == Some(true) {
            t.push("rush".into());
        }
        if self.storm == Some(true) {
            t.push("storm".into());
        }
        if self.ward == Some(true) {
            t.push("ward".into());
        }
        t.sort();
        t
    }
}

// ---------------------------------------------------------------------------
// CounterKey, Filter, Selector, Amount, Condition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CounterKey {
    Named(NamedCounter),
    Var { var: VarKey },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NamedCounter {
    Combo,
    Earth,
    Faith,
    Shadows,
    SkyboundHand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Filter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all: Option<Vec<Filter>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub any: Option<Vec<Filter>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<Filter>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tribe: Option<TribeOrList>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<CardId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<CardId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_card: Option<CardId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<FilterKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<Class>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_eq: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_lte: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_gte: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_in: Option<Vec<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_cost_eq: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_cost_lte: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_cost_gte: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_cost_in: Option<Vec<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_lte: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_gte: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defense_lte: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defense_gte: Option<Box<Amount>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolved: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unevolved: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damaged: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_trait: Option<TraitKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enhanced: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub same_cost_group: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_last_words: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spellboost: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destroyed_this_match: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "didNotAttackThisTurn")]
    pub did_not_attack_this_turn: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "superEvolved")]
    pub super_evolved: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "notBound")]
    pub not_bound: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TribeOrList {
    One(Tribe),
    Many(Vec<Tribe>),
}

/// Reference pick vs pool pick. Pool picks require `side`, `zone`, `kind`.
/// Schema-mirror enums nest Filter/Selector/Condition; clippy 1.98's
/// `large_enum_variant` fires. Size is paid at `CardDb::load`, not in `State`.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Selector {
    Bound(BoundSelector),
    Ref(RefSelector),
    Pool(PoolSelector),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BoundSelector {
    pub pick: BoundPick,
    #[serde(rename = "ref")]
    pub ref_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BoundPick {
    Bound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RefSelector {
    pub pick: RefPick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RefPick {
    #[serde(rename = "self")]
    Self_,
    Entering,
    Attacker,
    Defender,
    Opposing,
    Selected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PoolSelector {
    pub pick: PoolPick,
    pub side: Side,
    pub zone: Zone,
    pub kind: SelectorKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<Amount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_leader: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderBy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PoolPick {
    All,
    Choose,
    Random,
    RandomDistinct,
    Leftmost,
    Highest,
    Lowest,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Amount {
    Int(i32),
    Count {
        count: Box<Selector>,
    },
    Counter {
        counter: CounterKey,
    },
    Stat {
        stat: AmountStat,
    },
    Var {
        var: VarKey,
    },
    Add {
        add: [Box<Amount>; 2],
    },
    Sub {
        sub: [Box<Amount>; 2],
    },
    Max {
        max: [Box<Amount>; 2],
    },
    Min {
        min: [Box<Amount>; 2],
    },
    Neg {
        neg: Box<Amount>,
    },
    DistinctNames {
        #[serde(rename = "distinctNames")]
        distinct_names: Box<Amount>,
    },
    EnteredThisMatch {
        #[serde(rename = "enteredThisMatch")]
        entered_this_match: Filter,
    },
    SumHighestBaseCosts {
        #[serde(rename = "sumHighestBaseCosts")]
        sum_highest_base_costs: SumHighestBaseCosts,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AmountStat {
    pub of: Box<Selector>,
    pub which: StatWhich,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SumHighestBaseCosts {
    pub n: i32,
    pub select: Box<Selector>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    All {
        all: Vec<Condition>,
    },
    Any {
        any: Vec<Condition>,
    },
    Not {
        not: Box<Condition>,
    },
    CountAtLeast {
        #[serde(rename = "countAtLeast")]
        count_at_least: CountAtLeast,
    },
    CounterAtLeast {
        #[serde(rename = "counterAtLeast")]
        counter_at_least: CounterAtLeast,
    },
    Evolved {
        evolved: bool,
    },
    SuperEvolutionUnlocked {
        #[serde(rename = "superEvolutionUnlocked")]
        super_evolution_unlocked: bool,
    },
    Combo {
        combo: ComboN,
    },
    Rally {
        rally: ComboN,
    },
    Overflow {
        overflow: bool,
    },
    /// Played instance's current cost. Severed Ties "If this card's cost is 3".
    CostEq {
        #[serde(rename = "costEq")]
        cost_eq: Amount,
    },
    /// Exact check of current `pp_max` against `n` (Dragonsign: 10).
    /// Overflow remains the separate ≥7-max-PP keyword.
    MaxPpAtLeast {
        #[serde(rename = "maxPpAtLeast")]
        max_pp_at_least: MaxPpAtLeast,
    },
    SkyboundArt {
        #[serde(rename = "skyboundArt")]
        skybound_art: ComboN,
    },
    WasFused {
        #[serde(rename = "wasFused")]
        was_fused: WasFused,
    },
    Did {
        did: String,
    },
    AttackedLeaderLastTurn {
        #[serde(rename = "attackedLeaderLastTurn")]
        attacked_leader_last_turn: bool,
    },
    /// Verdilia crest: "Whenever a super-evolved allied follower attacks a follower"
    AttackingFollower {
        #[serde(rename = "attackingFollower")]
        attacking_follower: bool,
    },
    TurnOwner {
        #[serde(rename = "turnOwner")]
        turn_owner: TurnOwner,
    },
    EvolvedCountAtLeast {
        #[serde(rename = "evolvedCountAtLeast")]
        evolved_count_at_least: ComboN,
    },
    PlayedBaseCostsThisMatch {
        #[serde(rename = "playedBaseCostsThisMatch")]
        played_base_costs_this_match: Vec<i32>,
    },
    HandHas {
        #[serde(rename = "handHas")]
        hand_has: HandHas,
    },
    FieldHas {
        #[serde(rename = "fieldHas")]
        field_has: FieldHas,
    },
    LeaderDefenseLte {
        #[serde(rename = "leaderDefenseLte")]
        leader_defense_lte: Amount,
    },
    VarAtLeast {
        #[serde(rename = "varAtLeast")]
        var_at_least: VarAtLeast,
    },
    EnterCountAtLeast {
        #[serde(rename = "enterCountAtLeast")]
        enter_count_at_least: EnterCountAtLeast,
    },
    HandSameCostAtLeast {
        #[serde(rename = "handSameCostAtLeast")]
        hand_same_cost_at_least: ComboN,
    },
    AmountAtLeast {
        #[serde(rename = "amountAtLeast")]
        amount_at_least: AmountAtLeast,
    },
    DeckHasNoDuplicates {
        #[serde(rename = "deckHasNoDuplicates")]
        deck_has_no_duplicates: bool,
    },
    AttackingLeader {
        #[serde(rename = "attackingLeader")]
        attacking_leader: bool,
    },
    BoundHas {
        #[serde(rename = "boundHas")]
        bound_has: BoundHas,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CountAtLeast {
    pub select: Selector,
    pub n: Amount,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CounterAtLeast {
    pub key: CounterKey,
    pub n: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ComboN {
    pub n: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct MaxPpAtLeast {
    pub n: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WasFused {
    Flag(bool),
    Both(BothWord),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BothWord {
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct HandHas {
    pub filter: Filter,
    pub n: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct FieldHas {
    pub filter: Filter,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<Amount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<FieldHasKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VarAtLeast {
    pub key: VarKey,
    pub n: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AmountAtLeast {
    pub of: Amount,
    pub n: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BoundHas {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub filter: Filter,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EnterCountAtLeast {
    pub card: CardId,
    pub n: Amount,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

// ---------------------------------------------------------------------------
// CardSource, Effect, Ability, Mode, Fuse, Card, Crest
// ---------------------------------------------------------------------------

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CardSource {
    Named {
        named: CardId,
    },
    Copy {
        #[serde(rename = "copyOf")]
        copy_of: Selector,
        exact: bool,
    },
    /// Put the selected instance itself onto the field (Chloe "summon it").
    From {
        from: Selector,
    },
    RandomFrom {
        #[serde(rename = "randomFrom")]
        random_from: Filter,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoosePick {
    N(u32),
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChoosePickDe {
    N(u32),
    Word(AllWord),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AllWord {
    All,
}

impl ChoosePickDe {
    pub fn as_pick(&self) -> ChoosePick {
        match self {
            ChoosePickDe::N(n) => ChoosePick::N(*n),
            ChoosePickDe::Word(AllWord::All) => ChoosePick::All,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ChooseOption {
    pub printed: String,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SequenceStep {
    pub printed: String,
    pub effects: Vec<Effect>,
}

/// Forced `leaderModifier.maxDefense` construction — `{set: N}` or `{delta: N}`.
/// Same split as `cost.set` / `cost.delta`; no sign-dependent bare int.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaxDefenseChange {
    Set { set: Amount },
    Delta { delta: Amount },
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Effect {
    Damage {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        amount: Amount,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        split: Option<bool>,
    },
    Restore {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        amount: Amount,
    },
    Buff {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attack: Option<Amount>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        defense: Option<Amount>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "untilEndOfTurn")]
        until_end_of_turn: Option<bool>,
    },
    /// Selection with no effect on the chosen card (Cassius `10473110`).
    Select {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
    },
    Destroy {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
    },
    Banish {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
    },
    #[serde(rename = "returnToHand")]
    ReturnToHand {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
    },
    #[serde(rename = "returnToDeck")]
    ReturnToDeck {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        position: DeckPosition,
    },
    Summon {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        card: CardSource,
        count: Amount,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        controller: Option<Controller>,
    },
    Reanimate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(rename = "maxCost")]
        max_cost: Amount,
    },
    #[serde(rename = "addToHand")]
    AddToHand {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        card: CardSource,
        count: Amount,
    },
    Draw {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        count: Amount,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<Filter>,
        #[serde(
            default,
            rename = "distinctNames",
            skip_serializing_if = "Option::is_none"
        )]
        distinct_names: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        player: Option<CrestPlayer>,
    },
    Discard {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
    },
    #[serde(rename = "addToDeck")]
    AddToDeck {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        card: CardSource,
        count: Amount,
        position: DeckPosition,
    },
    Evolve {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        #[serde(rename = "super")]
        super_evolve: bool,
    },
    #[serde(rename = "grantTraits")]
    GrantTraits {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        traits: Traits,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        until: Option<Until>,
    },
    #[serde(rename = "removeTraits")]
    RemoveTraits {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        traits: Traits,
    },
    #[serde(rename = "grantAbility")]
    GrantAbility {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        ability: Box<Ability>,
    },
    #[serde(rename = "removeAbilities")]
    RemoveAbilities {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        on: Option<Vec<TriggerTag>>,
    },
    Cost {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delta: Option<Amount>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        set: Option<Amount>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "untilEndOfTurn")]
        until_end_of_turn: Option<bool>,
    },
    Pp {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        action: PpAction,
        amount: Amount,
    },
    Ep {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        action: EpAction,
        #[serde(rename = "super")]
        super_ep: bool,
        amount: Amount,
    },
    Crest {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        gain: CrestGainId,
        player: CrestPlayer,
    },
    #[serde(rename = "removeCrests")]
    RemoveCrests {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
    },
    Countdown {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        delta: Amount,
    },
    Counter {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        key: CounterKey,
        how: CounterHow,
        amount: Amount,
    },
    Pay {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        resource: PayResource,
        amount: Amount,
        effects: Vec<Effect>,
    },
    Transform {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        into: CardSource,
    },
    #[serde(rename = "leaderModifier")]
    LeaderModifier {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        select: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "maxDefense")]
        max_defense: Option<MaxDefenseChange>,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "damageCap")]
        damage_cap: Option<Amount>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "damageTakenBonus"
        )]
        damage_taken_bonus: Option<Amount>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        until: Option<Until>,
    },
    Replicate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        ability: ReplicateKey,
    },
    Invoke {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
    },
    #[serde(rename = "spellboostHand")]
    SpellboostHand {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        times: Amount,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        select: Option<Selector>,
    },
    #[serde(rename = "randomSplit")]
    RandomSplit {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        keys: Vec<VarKey>,
        times: Amount,
        effects: Vec<Effect>,
    },
    Seq {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        effects: Vec<Effect>,
    },
    If {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        cond: Condition,
        then: Vec<Effect>,
        #[serde(default, rename = "else", skip_serializing_if = "Option::is_none")]
        else_effects: Option<Vec<Effect>>,
    },
    Choose {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        pick: ChoosePickDe,
        by: ChooseBy,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        options: Option<Vec<ChooseOption>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "optionsFrom")]
        options_from: Option<OptionsFrom>,
    },
    Repeat {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        times: Amount,
        effects: Vec<Effect>,
    },
    Sequence {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        printed: Option<String>,
        #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
        as_bind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        steps: Vec<SequenceStep>,
    },
}

impl Effect {
    pub fn printed(&self) -> Option<&str> {
        match self {
            Effect::Damage { printed, .. }
            | Effect::Restore { printed, .. }
            | Effect::Buff { printed, .. }
            | Effect::Select { printed, .. }
            | Effect::Destroy { printed, .. }
            | Effect::Banish { printed, .. }
            | Effect::ReturnToHand { printed, .. }
            | Effect::ReturnToDeck { printed, .. }
            | Effect::Summon { printed, .. }
            | Effect::Reanimate { printed, .. }
            | Effect::AddToHand { printed, .. }
            | Effect::Draw { printed, .. }
            | Effect::Discard { printed, .. }
            | Effect::AddToDeck { printed, .. }
            | Effect::Evolve { printed, .. }
            | Effect::GrantTraits { printed, .. }
            | Effect::RemoveTraits { printed, .. }
            | Effect::GrantAbility { printed, .. }
            | Effect::RemoveAbilities { printed, .. }
            | Effect::Cost { printed, .. }
            | Effect::Pp { printed, .. }
            | Effect::Ep { printed, .. }
            | Effect::Crest { printed, .. }
            | Effect::RemoveCrests { printed, .. }
            | Effect::Countdown { printed, .. }
            | Effect::Counter { printed, .. }
            | Effect::Pay { printed, .. }
            | Effect::Transform { printed, .. }
            | Effect::LeaderModifier { printed, .. }
            | Effect::Replicate { printed, .. }
            | Effect::Invoke { printed, .. }
            | Effect::SpellboostHand { printed, .. }
            | Effect::RandomSplit { printed, .. }
            | Effect::Seq { printed, .. }
            | Effect::If { printed, .. }
            | Effect::Choose { printed, .. }
            | Effect::Repeat { printed, .. }
            | Effect::Sequence { printed, .. } => printed.as_deref(),
        }
    }

    pub fn when_cond(&self) -> Option<&Condition> {
        match self {
            Effect::Damage { when, .. }
            | Effect::Restore { when, .. }
            | Effect::Buff { when, .. }
            | Effect::Select { when, .. }
            | Effect::Destroy { when, .. }
            | Effect::Banish { when, .. }
            | Effect::ReturnToHand { when, .. }
            | Effect::ReturnToDeck { when, .. }
            | Effect::Summon { when, .. }
            | Effect::Reanimate { when, .. }
            | Effect::AddToHand { when, .. }
            | Effect::Draw { when, .. }
            | Effect::Discard { when, .. }
            | Effect::AddToDeck { when, .. }
            | Effect::Evolve { when, .. }
            | Effect::GrantTraits { when, .. }
            | Effect::RemoveTraits { when, .. }
            | Effect::GrantAbility { when, .. }
            | Effect::RemoveAbilities { when, .. }
            | Effect::Cost { when, .. }
            | Effect::Pp { when, .. }
            | Effect::Ep { when, .. }
            | Effect::Crest { when, .. }
            | Effect::RemoveCrests { when, .. }
            | Effect::Countdown { when, .. }
            | Effect::Counter { when, .. }
            | Effect::Pay { when, .. }
            | Effect::Transform { when, .. }
            | Effect::LeaderModifier { when, .. }
            | Effect::Replicate { when, .. }
            | Effect::Invoke { when, .. }
            | Effect::SpellboostHand { when, .. }
            | Effect::RandomSplit { when, .. }
            | Effect::Seq { when, .. }
            | Effect::If { when, .. }
            | Effect::Choose { when, .. }
            | Effect::Repeat { when, .. }
            | Effect::Sequence { when, .. } => when.as_ref(),
        }
    }

    pub fn as_bind(&self) -> Option<&str> {
        match self {
            Effect::Damage { as_bind, .. }
            | Effect::Restore { as_bind, .. }
            | Effect::Buff { as_bind, .. }
            | Effect::Select { as_bind, .. }
            | Effect::Destroy { as_bind, .. }
            | Effect::Banish { as_bind, .. }
            | Effect::ReturnToHand { as_bind, .. }
            | Effect::ReturnToDeck { as_bind, .. }
            | Effect::Summon { as_bind, .. }
            | Effect::Reanimate { as_bind, .. }
            | Effect::AddToHand { as_bind, .. }
            | Effect::Draw { as_bind, .. }
            | Effect::Discard { as_bind, .. }
            | Effect::AddToDeck { as_bind, .. }
            | Effect::Evolve { as_bind, .. }
            | Effect::GrantTraits { as_bind, .. }
            | Effect::RemoveTraits { as_bind, .. }
            | Effect::GrantAbility { as_bind, .. }
            | Effect::RemoveAbilities { as_bind, .. }
            | Effect::Cost { as_bind, .. }
            | Effect::Pp { as_bind, .. }
            | Effect::Ep { as_bind, .. }
            | Effect::Crest { as_bind, .. }
            | Effect::RemoveCrests { as_bind, .. }
            | Effect::Countdown { as_bind, .. }
            | Effect::Counter { as_bind, .. }
            | Effect::Pay { as_bind, .. }
            | Effect::Transform { as_bind, .. }
            | Effect::LeaderModifier { as_bind, .. }
            | Effect::Replicate { as_bind, .. }
            | Effect::Invoke { as_bind, .. }
            | Effect::SpellboostHand { as_bind, .. }
            | Effect::RandomSplit { as_bind, .. }
            | Effect::Seq { as_bind, .. }
            | Effect::If { as_bind, .. }
            | Effect::Choose { as_bind, .. }
            | Effect::Repeat { as_bind, .. }
            | Effect::Sequence { as_bind, .. } => as_bind.as_deref(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "on", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Ability {
    Fanfare {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    #[serde(rename = "lastWords")]
    LastWords {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Evolve {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    SuperEvolve {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    AnyEvolve {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    AnySuperEvolve {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Strike {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    FollowerStrike {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Clash {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Enter {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Leave {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Discarded {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Invoked {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Fused {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Spellboost {
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Engage {
        cost: i32,
        #[serde(default)]
        sacrifice: bool,
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    StartOfTurn {
        whose: Whose,
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    EndOfTurn {
        whose: Whose,
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    When {
        event: EventName,
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<Filter>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "oncePerTurn")]
        once_per_turn: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces: Option<Replaces>,
    },
    Static {
        printed: String,
        modifier: StaticModifier,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone: Option<AbilityZone>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        when: Option<Condition>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Replaces {
    Evolve,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StaticModifier {
    pub suppress: Vec<TriggerTag>,
    pub select: Selector,
}

impl Ability {
    pub fn tag(&self) -> TriggerTag {
        match self {
            Ability::Fanfare { .. } => TriggerTag::Fanfare,
            Ability::LastWords { .. } => TriggerTag::LastWords,
            Ability::Evolve { .. } => TriggerTag::Evolve,
            Ability::SuperEvolve { .. } => TriggerTag::SuperEvolve,
            Ability::AnyEvolve { .. } => TriggerTag::AnyEvolve,
            Ability::AnySuperEvolve { .. } => TriggerTag::AnySuperEvolve,
            Ability::Strike { .. } => TriggerTag::Strike,
            Ability::FollowerStrike { .. } => TriggerTag::FollowerStrike,
            Ability::Clash { .. } => TriggerTag::Clash,
            Ability::Enter { .. } => TriggerTag::Enter,
            Ability::Leave { .. } => TriggerTag::Leave,
            Ability::Discarded { .. } => TriggerTag::Discarded,
            Ability::Invoked { .. } => TriggerTag::Invoked,
            Ability::Fused { .. } => TriggerTag::Fused,
            Ability::Spellboost { .. } => TriggerTag::Spellboost,
            Ability::Engage { .. } => TriggerTag::Engage,
            Ability::StartOfTurn { .. } => TriggerTag::StartOfTurn,
            Ability::EndOfTurn { .. } => TriggerTag::EndOfTurn,
            Ability::When { .. } => TriggerTag::When,
            Ability::Static { .. } => TriggerTag::When,
        }
    }

    pub fn effects(&self) -> &[Effect] {
        match self {
            Ability::Static { .. } => &[],
            Ability::Fanfare { effects, .. }
            | Ability::LastWords { effects, .. }
            | Ability::Evolve { effects, .. }
            | Ability::SuperEvolve { effects, .. }
            | Ability::AnyEvolve { effects, .. }
            | Ability::AnySuperEvolve { effects, .. }
            | Ability::Strike { effects, .. }
            | Ability::FollowerStrike { effects, .. }
            | Ability::Clash { effects, .. }
            | Ability::Enter { effects, .. }
            | Ability::Leave { effects, .. }
            | Ability::Discarded { effects, .. }
            | Ability::Invoked { effects, .. }
            | Ability::Fused { effects, .. }
            | Ability::Spellboost { effects, .. }
            | Ability::Engage { effects, .. }
            | Ability::StartOfTurn { effects, .. }
            | Ability::EndOfTurn { effects, .. }
            | Ability::When { effects, .. } => effects,
        }
    }

    pub fn zone(&self) -> AbilityZone {
        let z = match self {
            Ability::Fanfare { zone, .. }
            | Ability::LastWords { zone, .. }
            | Ability::Evolve { zone, .. }
            | Ability::SuperEvolve { zone, .. }
            | Ability::AnyEvolve { zone, .. }
            | Ability::AnySuperEvolve { zone, .. }
            | Ability::Strike { zone, .. }
            | Ability::FollowerStrike { zone, .. }
            | Ability::Clash { zone, .. }
            | Ability::Enter { zone, .. }
            | Ability::Leave { zone, .. }
            | Ability::Discarded { zone, .. }
            | Ability::Invoked { zone, .. }
            | Ability::Fused { zone, .. }
            | Ability::Spellboost { zone, .. }
            | Ability::Engage { zone, .. }
            | Ability::StartOfTurn { zone, .. }
            | Ability::EndOfTurn { zone, .. }
            | Ability::When { zone, .. }
            | Ability::Static { zone, .. } => *zone,
        };
        z.unwrap_or(AbilityZone::Field)
    }

    pub fn once_per_turn(&self) -> bool {
        match self {
            Ability::Static { .. } => false,
            Ability::Fanfare { once_per_turn, .. }
            | Ability::LastWords { once_per_turn, .. }
            | Ability::Evolve { once_per_turn, .. }
            | Ability::SuperEvolve { once_per_turn, .. }
            | Ability::AnyEvolve { once_per_turn, .. }
            | Ability::AnySuperEvolve { once_per_turn, .. }
            | Ability::Strike { once_per_turn, .. }
            | Ability::FollowerStrike { once_per_turn, .. }
            | Ability::Clash { once_per_turn, .. }
            | Ability::Enter { once_per_turn, .. }
            | Ability::Leave { once_per_turn, .. }
            | Ability::Discarded { once_per_turn, .. }
            | Ability::Invoked { once_per_turn, .. }
            | Ability::Fused { once_per_turn, .. }
            | Ability::Spellboost { once_per_turn, .. }
            | Ability::Engage { once_per_turn, .. }
            | Ability::StartOfTurn { once_per_turn, .. }
            | Ability::EndOfTurn { once_per_turn, .. }
            | Ability::When { once_per_turn, .. } => once_per_turn.unwrap_or(false),
        }
    }

    pub fn when_cond(&self) -> Option<&Condition> {
        match self {
            Ability::Fanfare { when, .. }
            | Ability::LastWords { when, .. }
            | Ability::Evolve { when, .. }
            | Ability::SuperEvolve { when, .. }
            | Ability::AnyEvolve { when, .. }
            | Ability::AnySuperEvolve { when, .. }
            | Ability::Strike { when, .. }
            | Ability::FollowerStrike { when, .. }
            | Ability::Clash { when, .. }
            | Ability::Enter { when, .. }
            | Ability::Leave { when, .. }
            | Ability::Discarded { when, .. }
            | Ability::Invoked { when, .. }
            | Ability::Fused { when, .. }
            | Ability::Spellboost { when, .. }
            | Ability::Engage { when, .. }
            | Ability::StartOfTurn { when, .. }
            | Ability::EndOfTurn { when, .. }
            | Ability::When { when, .. }
            | Ability::Static { when, .. } => when.as_ref(),
        }
    }

    pub fn replaces_evolve(&self) -> bool {
        matches!(
            self,
            Ability::SuperEvolve {
                replaces: Some(Replaces::Evolve),
                ..
            }
        )
    }

    pub fn snapshot_tag(&self) -> &'static str {
        match self {
            Ability::Static { .. } => "static",
            _ => self.tag().snapshot(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Mode {
    Enhance {
        cost: i32,
        printed: String,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "replacesBase")]
        replaces_base: Option<bool>,
    },
    Accelerate {
        cost: i32,
        printed: String,
        effects: Vec<Effect>,
    },
    Crystallize {
        cost: i32,
        printed: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        countdown: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        traits: Option<Traits>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        abilities: Option<Vec<Ability>>,
    },
}

impl Mode {
    pub fn cost(&self) -> i32 {
        match self {
            Mode::Enhance { cost, .. }
            | Mode::Accelerate { cost, .. }
            | Mode::Crystallize { cost, .. } => *cost,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Fuse {
    pub printed: String,
    pub partners: Filter,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipes: Option<Vec<FuseRecipe>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct FuseRecipe {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partners: Option<Filter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_total: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_total_gte: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<Vec<CardId>>,
    pub result: FuseResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FuseResult {
    Transform {
        #[serde(rename = "transformInto")]
        transform_into: CardId,
    },
    Consume {
        consume: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct CardVars {
    #[serde(rename = "X")]
    pub x: i32,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Card {
    Follower {
        id: CardId,
        name: String,
        class: Class,
        set: i32,
        rarity: Rarity,
        token: bool,
        cost: i32,
        text: String,
        attack: i32,
        defense: i32,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tribes: Vec<Tribe>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        traits: Option<Traits>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        abilities: Option<Vec<Ability>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modes: Option<Vec<Mode>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fuse: Option<Fuse>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vars: Option<CardVars>,
    },
    Spell {
        id: CardId,
        name: String,
        class: Class,
        set: i32,
        rarity: Rarity,
        token: bool,
        cost: i32,
        text: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tribes: Vec<Tribe>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        traits: Option<Traits>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        abilities: Option<Vec<Ability>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modes: Option<Vec<Mode>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fuse: Option<Fuse>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vars: Option<CardVars>,
    },
    Amulet {
        id: CardId,
        name: String,
        class: Class,
        set: i32,
        rarity: Rarity,
        token: bool,
        cost: i32,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        countdown: Option<i32>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tribes: Vec<Tribe>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        traits: Option<Traits>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        abilities: Option<Vec<Ability>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modes: Option<Vec<Mode>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fuse: Option<Fuse>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vars: Option<CardVars>,
    },
}

impl Card {
    pub fn id(&self) -> CardId {
        match self {
            Card::Follower { id, .. } | Card::Spell { id, .. } | Card::Amulet { id, .. } => *id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Card::Follower { name, .. } | Card::Spell { name, .. } | Card::Amulet { name, .. } => {
                name
            }
        }
    }

    pub fn kind(&self) -> CardKind {
        match self {
            Card::Follower { .. } => CardKind::Follower,
            Card::Spell { .. } => CardKind::Spell,
            Card::Amulet { .. } => CardKind::Amulet,
        }
    }

    pub fn class(&self) -> Class {
        match self {
            Card::Follower { class, .. }
            | Card::Spell { class, .. }
            | Card::Amulet { class, .. } => *class,
        }
    }

    pub fn set(&self) -> i32 {
        match self {
            Card::Follower { set, .. } | Card::Spell { set, .. } | Card::Amulet { set, .. } => *set,
        }
    }

    pub fn token(&self) -> bool {
        match self {
            Card::Follower { token, .. }
            | Card::Spell { token, .. }
            | Card::Amulet { token, .. } => *token,
        }
    }

    pub fn cost(&self) -> i32 {
        match self {
            Card::Follower { cost, .. } | Card::Spell { cost, .. } | Card::Amulet { cost, .. } => {
                *cost
            }
        }
    }

    pub fn attack(&self) -> i32 {
        match self {
            Card::Follower { attack, .. } => *attack,
            _ => 0,
        }
    }

    pub fn defense(&self) -> i32 {
        match self {
            Card::Follower { defense, .. } => *defense,
            _ => 0,
        }
    }

    pub fn countdown(&self) -> Option<i32> {
        match self {
            Card::Amulet { countdown, .. } => *countdown,
            _ => None,
        }
    }

    pub fn tribes(&self) -> &[Tribe] {
        match self {
            Card::Follower { tribes, .. }
            | Card::Spell { tribes, .. }
            | Card::Amulet { tribes, .. } => tribes,
        }
    }

    pub fn traits(&self) -> Option<&Traits> {
        match self {
            Card::Follower { traits, .. }
            | Card::Spell { traits, .. }
            | Card::Amulet { traits, .. } => traits.as_ref(),
        }
    }

    pub fn abilities(&self) -> &[Ability] {
        match self {
            Card::Follower { abilities, .. }
            | Card::Spell { abilities, .. }
            | Card::Amulet { abilities, .. } => abilities.as_deref().unwrap_or(&[]),
        }
    }

    /// Trigger tags the printed card carries (`docs/trace-format.md` `granted`).
    pub fn printed_trigger_tags(&self) -> BTreeSet<String> {
        let mut tags = BTreeSet::new();
        for a in self.abilities() {
            tags.insert(a.snapshot_tag().to_string());
        }
        if self
            .modes()
            .iter()
            .any(|m| matches!(m, Mode::Enhance { .. }))
        {
            tags.insert("enhance".into());
        }
        tags
    }

    pub fn modes(&self) -> &[Mode] {
        match self {
            Card::Follower { modes, .. }
            | Card::Spell { modes, .. }
            | Card::Amulet { modes, .. } => modes.as_deref().unwrap_or(&[]),
        }
    }

    pub fn fuse(&self) -> Option<&Fuse> {
        match self {
            Card::Follower { fuse, .. } | Card::Spell { fuse, .. } | Card::Amulet { fuse, .. } => {
                fuse.as_ref()
            }
        }
    }

    pub fn vars(&self) -> Option<&CardVars> {
        match self {
            Card::Follower { vars, .. } | Card::Spell { vars, .. } | Card::Amulet { vars, .. } => {
                vars.as_ref()
            }
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Card::Follower { text, .. } | Card::Spell { text, .. } | Card::Amulet { text, .. } => {
                text
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Crest {
    pub id: String,
    pub name: String,
    pub granted_by: Vec<CardId>,
    pub faith: bool,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub countdown: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abilities: Option<Vec<Ability>>,
}

impl Crest {
    pub fn abilities(&self) -> &[Ability] {
        self.abilities.as_deref().unwrap_or(&[])
    }
}

/// Top-level authored file: a collectible / token card or a crest / faith.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CardOrCrest {
    Card(Card),
    Crest(Crest),
}

fn default_deck_enabled() -> i32 {
    3
}

/// Catalog facts used for cross-checks (not a closed schema object).
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogRecord {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub class: String,
    #[serde(default)]
    pub tribes: Vec<String>,
    pub rarity: String,
    pub cost: i32,
    #[serde(default)]
    pub attack: i32,
    #[serde(default)]
    pub defense: i32,
    pub set: i32,
    pub token: bool,
    #[serde(default)]
    pub rotation: bool,
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_deck_enabled")]
    pub deck_enabled_num: i32,
    #[serde(default)]
    pub related_card_ids: Vec<String>,
    #[serde(default)]
    pub card_image_hash: Option<String>,
    #[serde(default)]
    pub card_banner_image_hash: Option<String>,
    #[serde(default)]
    pub evo_card_image_hash: Option<String>,
    #[serde(default)]
    pub evo_card_banner_image_hash: Option<String>,
}

impl Card {
    /// `ref` names on this card that no `as` on the same card can produce.
    pub fn unbound_refs(&self) -> Vec<String> {
        let mut produced = BTreeSet::new();
        let mut used = BTreeSet::new();
        for a in self.abilities() {
            walk_ability(a, &mut produced, &mut used);
        }
        for m in self.modes() {
            walk_mode(m, &mut produced, &mut used);
        }
        used.into_iter().filter(|n| !produced.contains(n)).collect()
    }
}

impl Crest {
    pub fn unbound_refs(&self) -> Vec<String> {
        let mut produced = BTreeSet::new();
        let mut used = BTreeSet::new();
        for a in self.abilities() {
            walk_ability(a, &mut produced, &mut used);
        }
        used.into_iter().filter(|n| !produced.contains(n)).collect()
    }
}

fn walk_ability(a: &Ability, produced: &mut BTreeSet<String>, used: &mut BTreeSet<String>) {
    for e in a.effects() {
        walk_effect(e, produced, used);
    }
    if let Ability::Static { modifier, .. } = a {
        walk_selector(&modifier.select, used);
    }
}

fn walk_mode(m: &Mode, produced: &mut BTreeSet<String>, used: &mut BTreeSet<String>) {
    match m {
        Mode::Enhance { effects, .. } | Mode::Accelerate { effects, .. } => {
            for e in effects {
                walk_effect(e, produced, used);
            }
        }
        Mode::Crystallize { abilities, .. } => {
            if let Some(abs) = abilities {
                for a in abs {
                    walk_ability(a, produced, used);
                }
            }
        }
    }
}

fn walk_effect(e: &Effect, produced: &mut BTreeSet<String>, used: &mut BTreeSet<String>) {
    if let Some(name) = e.as_bind() {
        produced.insert(name.to_string());
    }
    match e {
        Effect::Damage { select, amount, .. } => {
            walk_selector(select, used);
            walk_amount(amount, used);
        }
        Effect::Restore { select, amount, .. } => {
            walk_selector(select, used);
            walk_amount(amount, used);
        }
        Effect::Buff {
            select,
            attack,
            defense,
            ..
        } => {
            walk_selector(select, used);
            if let Some(a) = attack {
                walk_amount(a, used);
            }
            if let Some(a) = defense {
                walk_amount(a, used);
            }
        }
        Effect::Destroy { select, .. }
        | Effect::Select { select, .. }
        | Effect::Banish { select, .. }
        | Effect::ReturnToHand { select, .. }
        | Effect::ReturnToDeck { select, .. }
        | Effect::Discard { select, .. }
        | Effect::Evolve { select, .. }
        | Effect::GrantTraits { select, .. }
        | Effect::RemoveTraits { select, .. }
        | Effect::RemoveAbilities { select, .. }
        | Effect::RemoveCrests { select, .. } => walk_selector(select, used),
        Effect::LeaderModifier {
            select,
            max_defense,
            ..
        } => {
            walk_selector(select, used);
            match max_defense {
                Some(MaxDefenseChange::Set { set }) => walk_amount(set, used),
                Some(MaxDefenseChange::Delta { delta }) => walk_amount(delta, used),
                None => {}
            }
        }
        Effect::Countdown { select, delta, .. } => {
            walk_selector(select, used);
            walk_amount(delta, used);
        }
        Effect::Summon { card, count, .. } | Effect::AddToHand { card, count, .. } => {
            walk_source(card, used);
            walk_amount(count, used);
        }
        Effect::AddToDeck { card, count, .. } => {
            walk_source(card, used);
            walk_amount(count, used);
        }
        Effect::Draw { count, .. }
        | Effect::Pp { amount: count, .. }
        | Effect::Ep { amount: count, .. }
        | Effect::Reanimate {
            max_cost: count, ..
        } => walk_amount(count, used),
        Effect::SpellboostHand {
            times: count,
            select,
            ..
        } => {
            walk_amount(count, used);
            if let Some(s) = select {
                walk_selector(s, used);
            }
        }
        Effect::Cost {
            select, delta, set, ..
        } => {
            walk_selector(select, used);
            if let Some(a) = delta {
                walk_amount(a, used);
            }
            if let Some(a) = set {
                walk_amount(a, used);
            }
        }
        Effect::Counter { amount, .. } => walk_amount(amount, used),
        Effect::Pay {
            amount, effects, ..
        } => {
            walk_amount(amount, used);
            for x in effects {
                walk_effect(x, produced, used);
            }
        }
        Effect::Repeat { times, effects, .. } => {
            walk_amount(times, used);
            for x in effects {
                walk_effect(x, produced, used);
            }
        }
        Effect::Seq { effects, .. } => {
            for x in effects {
                walk_effect(x, produced, used);
            }
        }
        Effect::GrantAbility {
            select, ability, ..
        } => {
            walk_selector(select, used);
            walk_ability(ability, produced, used);
        }
        Effect::If {
            cond,
            then,
            else_effects,
            ..
        } => {
            walk_condition(cond, used);
            for x in then {
                walk_effect(x, produced, used);
            }
            if let Some(els) = else_effects {
                for x in els {
                    walk_effect(x, produced, used);
                }
            }
        }
        Effect::Choose {
            options: Some(opts),
            ..
        } => {
            for o in opts {
                for x in &o.effects {
                    walk_effect(x, produced, used);
                }
            }
        }
        Effect::Choose { options: None, .. } => {}
        Effect::Sequence { steps, .. } => {
            for s in steps {
                for x in &s.effects {
                    walk_effect(x, produced, used);
                }
            }
        }
        Effect::Transform { into, select, .. } => {
            walk_selector(select, used);
            walk_source(into, used);
        }
        _ => {}
    }
    if let Some(c) = e.when_cond() {
        walk_condition(c, used);
    }
}

fn walk_source(src: &CardSource, used: &mut BTreeSet<String>) {
    match src {
        CardSource::Copy { copy_of, .. } => walk_selector(copy_of, used),
        CardSource::From { from } => walk_selector(from, used),
        CardSource::RandomFrom { random_from } => walk_filter(random_from, used),
        CardSource::Named { .. } => {}
    }
}

fn walk_selector(s: &Selector, used: &mut BTreeSet<String>) {
    match s {
        Selector::Bound(b) => {
            used.insert(b.ref_name.clone());
        }
        Selector::Pool(p) => {
            if let Some(f) = &p.filter {
                walk_filter(f, used);
            }
            if let Some(c) = &p.count {
                walk_amount(c, used);
            }
        }
        Selector::Ref(_) => {}
    }
}

fn walk_filter(f: &Filter, used: &mut BTreeSet<String>) {
    if let Some(xs) = &f.all {
        for x in xs {
            walk_filter(x, used);
        }
    }
    if let Some(xs) = &f.any {
        for x in xs {
            walk_filter(x, used);
        }
    }
    if let Some(x) = &f.not {
        walk_filter(x, used);
    }
    for amt in [
        f.cost_eq.as_deref(),
        f.cost_lte.as_deref(),
        f.cost_gte.as_deref(),
        f.base_cost_eq.as_deref(),
        f.base_cost_lte.as_deref(),
        f.base_cost_gte.as_deref(),
        f.attack_lte.as_deref(),
        f.attack_gte.as_deref(),
        f.defense_lte.as_deref(),
        f.defense_gte.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        walk_amount(amt, used);
    }
}

fn walk_amount(a: &Amount, used: &mut BTreeSet<String>) {
    match a {
        Amount::Count { count } => walk_selector(count, used),
        Amount::DistinctNames { distinct_names } => walk_amount(distinct_names, used),
        Amount::Stat { stat } => walk_selector(&stat.of, used),
        Amount::Add { add }
        | Amount::Sub { sub: add }
        | Amount::Max { max: add }
        | Amount::Min { min: add } => {
            walk_amount(&add[0], used);
            walk_amount(&add[1], used);
        }
        Amount::Neg { neg } => walk_amount(neg, used),
        Amount::EnteredThisMatch { entered_this_match } => walk_filter(entered_this_match, used),
        _ => {}
    }
}

fn walk_condition(c: &Condition, used: &mut BTreeSet<String>) {
    match c {
        Condition::All { all } => {
            for x in all {
                walk_condition(x, used);
            }
        }
        Condition::Any { any } => {
            for x in any {
                walk_condition(x, used);
            }
        }
        Condition::Not { not } => walk_condition(not, used),
        Condition::CountAtLeast { count_at_least } => {
            walk_selector(&count_at_least.select, used);
            walk_amount(&count_at_least.n, used);
            if let Some(f) = &count_at_least.filter {
                walk_filter(f, used);
            }
        }
        Condition::MaxPpAtLeast { max_pp_at_least } => walk_amount(&max_pp_at_least.n, used),
        Condition::Combo { combo } => walk_amount(&combo.n, used),
        Condition::AmountAtLeast { amount_at_least } => {
            walk_amount(&amount_at_least.of, used);
            walk_amount(&amount_at_least.n, used);
        }
        _ => {}
    }
}
