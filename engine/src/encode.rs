//! Perspective-masked observation. Perfect information stays in `State`.

use std::collections::BTreeSet;

use crate::action_id::ActionId;
use crate::card::{CardId, CardKind};
use crate::db::CardDb;
use crate::ids::PlayerId;
use crate::state::{
    BoundRef, CardInstance, ChoiceNode, Phase, PlayerState, State, CREST_CAP, FIELD_SIZE,
    HAND_LIMIT,
};

/// Padded histogram width over the two decklists' union (plus tokens).
pub const HIST_WIDTH: usize = 96;

/// Version-1 feature length (unchanged from M5).
pub const LEN_V1: usize = 353 + 2 * HIST_WIDTH;

/// Version-2 feature length (v1 block + 18 zone-bonus + 4 bonus-PP features).
pub const LEN_V2: usize = LEN_V1 + 22;

/// Version-3 feature length (v2 block + amulet/enter/cemetery extras).
pub const LEN_V3: usize = LEN_V2 + 394;

/// `own_deck_hist` offset + 2 × histogram (version 1).
pub const LEN: usize = LEN_V1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutField {
    pub name: &'static str,
    pub offset: usize,
    pub width: usize,
}

/// Name / offset / width of every version-1 feature block. Contiguous, covers `LEN`.
pub const LAYOUT: &[LayoutField] = &[
    LayoutField {
        name: "phase",
        offset: 0,
        width: 5,
    },
    LayoutField {
        name: "turn",
        offset: 5,
        width: 1,
    },
    LayoutField {
        name: "i_am_active",
        offset: 6,
        width: 1,
    },
    LayoutField {
        name: "winner",
        offset: 7,
        width: 1,
    },
    LayoutField {
        name: "first_is_me",
        offset: 8,
        width: 1,
    },
    LayoutField {
        name: "choice_ids",
        offset: 9,
        width: 32,
    },
    LayoutField {
        name: "me_scalars",
        offset: 41,
        width: 29,
    },
    LayoutField {
        name: "opp_scalars",
        offset: 70,
        width: 29,
    },
    LayoutField {
        name: "own_hand_cost",
        offset: 99,
        width: 9,
    },
    LayoutField {
        name: "own_hand_spellboost",
        offset: 108,
        width: 9,
    },
    LayoutField {
        name: "own_hand_skybound",
        offset: 117,
        width: 9,
    },
    LayoutField {
        name: "own_hand_fused",
        offset: 126,
        width: 9,
    },
    LayoutField {
        name: "own_hand_cant_play",
        offset: 135,
        width: 9,
    },
    LayoutField {
        name: "own_hand_once_used",
        offset: 144,
        width: 9,
    },
    LayoutField {
        name: "own_board",
        offset: 153,
        width: 100,
    },
    LayoutField {
        name: "opp_board",
        offset: 253,
        width: 100,
    },
    LayoutField {
        name: "own_deck_hist",
        offset: 353,
        width: HIST_WIDTH,
    },
    LayoutField {
        name: "opp_known_pool_hist",
        offset: 353 + HIST_WIDTH,
        width: HIST_WIDTH,
    },
];

/// Version-2-only features appended after the version-1 block.
pub const LAYOUT_V2_EXTRA: &[LayoutField] = &[
    LayoutField {
        name: "own_deck_atk_bonus_le2",
        offset: LEN_V1,
        width: 1,
    },
    LayoutField {
        name: "own_deck_atk_bonus_cost_3_4",
        offset: LEN_V1 + 1,
        width: 1,
    },
    LayoutField {
        name: "own_deck_atk_bonus_cost_ge5",
        offset: LEN_V1 + 2,
        width: 1,
    },
    LayoutField {
        name: "own_deck_atk_bonus_storm",
        offset: LEN_V1 + 3,
        width: 1,
    },
    LayoutField {
        name: "own_deck_def_bonus_le2",
        offset: LEN_V1 + 4,
        width: 1,
    },
    LayoutField {
        name: "own_deck_def_bonus_cost_3_4",
        offset: LEN_V1 + 5,
        width: 1,
    },
    LayoutField {
        name: "own_deck_def_bonus_cost_ge5",
        offset: LEN_V1 + 6,
        width: 1,
    },
    LayoutField {
        name: "own_deck_def_bonus_storm",
        offset: LEN_V1 + 7,
        width: 1,
    },
    LayoutField {
        name: "own_deck_cost_reduction",
        offset: LEN_V1 + 8,
        width: 1,
    },
    LayoutField {
        name: "own_hand_atk_bonus_le2",
        offset: LEN_V1 + 9,
        width: 1,
    },
    LayoutField {
        name: "own_hand_atk_bonus_cost_3_4",
        offset: LEN_V1 + 10,
        width: 1,
    },
    LayoutField {
        name: "own_hand_atk_bonus_cost_ge5",
        offset: LEN_V1 + 11,
        width: 1,
    },
    LayoutField {
        name: "own_hand_atk_bonus_storm",
        offset: LEN_V1 + 12,
        width: 1,
    },
    LayoutField {
        name: "own_hand_def_bonus_le2",
        offset: LEN_V1 + 13,
        width: 1,
    },
    LayoutField {
        name: "own_hand_def_bonus_cost_3_4",
        offset: LEN_V1 + 14,
        width: 1,
    },
    LayoutField {
        name: "own_hand_def_bonus_cost_ge5",
        offset: LEN_V1 + 15,
        width: 1,
    },
    LayoutField {
        name: "own_hand_def_bonus_storm",
        offset: LEN_V1 + 16,
        width: 1,
    },
    LayoutField {
        name: "own_hand_cost_reduction",
        offset: LEN_V1 + 17,
        width: 1,
    },
    LayoutField {
        name: "own_bonus_early",
        offset: LEN_V1 + 18,
        width: 1,
    },
    LayoutField {
        name: "own_bonus_late",
        offset: LEN_V1 + 19,
        width: 1,
    },
    LayoutField {
        name: "opp_bonus_early",
        offset: LEN_V1 + 20,
        width: 1,
    },
    LayoutField {
        name: "opp_bonus_late",
        offset: LEN_V1 + 21,
        width: 1,
    },
];

/// Version-3-only features appended after the version-2 block.
pub const LAYOUT_V3_EXTRA: &[LayoutField] = &[
    LayoutField {
        name: "own_amulet_soon",
        offset: LEN_V2,
        width: FIELD_SIZE,
    },
    LayoutField {
        name: "opp_amulet_soon",
        offset: LEN_V2 + FIELD_SIZE,
        width: FIELD_SIZE,
    },
    LayoutField {
        name: "own_entered_hist",
        offset: LEN_V2 + 2 * FIELD_SIZE,
        width: HIST_WIDTH,
    },
    LayoutField {
        name: "opp_entered_hist",
        offset: LEN_V2 + 2 * FIELD_SIZE + HIST_WIDTH,
        width: HIST_WIDTH,
    },
    LayoutField {
        name: "own_cemetery_hist",
        offset: LEN_V2 + 2 * FIELD_SIZE + 2 * HIST_WIDTH,
        width: HIST_WIDTH,
    },
    LayoutField {
        name: "opp_cemetery_hist",
        offset: LEN_V2 + 2 * FIELD_SIZE + 3 * HIST_WIDTH,
        width: HIST_WIDTH,
    },
];

/// Leaf observation encoding version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EncodingVersion {
    #[default]
    V1 = 1,
    V2 = 2,
    V3 = 3,
}

impl EncodingVersion {
    pub fn parse(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::V1),
            2 => Some(Self::V2),
            3 => Some(Self::V3),
            _ => None,
        }
    }

    pub fn feature_len(self) -> usize {
        match self {
            Self::V1 => LEN_V1,
            Self::V2 => LEN_V2,
            Self::V3 => LEN_V3,
        }
    }

    pub fn ids_len(self) -> usize {
        match self {
            Self::V1 | Self::V2 => IDS_LEN,
            Self::V3 => IDS_LEN_V3,
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Self::V1 => 1,
            Self::V2 => 2,
            Self::V3 => 3,
        }
    }
}

/// Parallel card-id vector (never one-hot). Opponent-hand region is always 0.
pub const IDS_OWN_HAND: usize = 0;
pub const IDS_OWN_DECK: usize = 9;
pub const IDS_OPP_BOARD: usize = 9 + HIST_WIDTH;
pub const IDS_OWN_BOARD: usize = 9 + HIST_WIDTH + FIELD_SIZE;
pub const IDS_OPP_POOL: usize = 9 + HIST_WIDTH + 2 * FIELD_SIZE;
pub const IDS_OPP_HAND: usize = 9 + 2 * HIST_WIDTH + 2 * FIELD_SIZE;
pub const IDS_LEN: usize = IDS_OPP_HAND + HAND_LIMIT;
pub const IDS_OWN_CRESTS: usize = IDS_LEN;
pub const IDS_OPP_CRESTS: usize = IDS_OWN_CRESTS + CREST_CAP;
pub const IDS_LEN_V3: usize = IDS_OPP_CRESTS + CREST_CAP;

pub const BOARD_WIDTH: usize = 20;

/// Non-linear race inputs derived from HP and board attack (encoding 1/2).
pub const RACE_LEN: usize = 12;

pub const RACE_NAMES: [&str; RACE_LEN] = [
    "lo5_me",
    "lo10_me",
    "threat_me",
    "near_me",
    "margin_me",
    "inter_me",
    "lo5_opp",
    "lo10_opp",
    "threat_opp",
    "near_opp",
    "margin_opp",
    "inter_opp",
];

const ME_SCALARS: usize = 41;
const OPP_SCALARS: usize = 70;
const OWN_BOARD: usize = 153;
const OPP_BOARD: usize = 253;

fn board_attack(features: &[f32], base: usize) -> f32 {
    let mut sum = 0.0f32;
    for slot in 0..FIELD_SIZE {
        let off = base + slot * BOARD_WIDTH;
        if features.get(off + 19).copied().unwrap_or(0.0) == 1.0 {
            sum += features.get(off).copied().unwrap_or(0.0);
        }
    }
    sum
}

fn race_side(hp: f32, atk: f32) -> [f32; 6] {
    let lo5 = (5.0 - hp).max(0.0);
    let lo10 = (10.0 - hp).max(0.0);
    let threat = if atk >= hp { 1.0 } else { 0.0 };
    let near = if atk >= hp - 3.0 { 1.0 } else { 0.0 };
    let margin = (hp - atk).clamp(-5.0, 10.0);
    let inter = hp * atk / 20.0;
    [lo5, lo10, threat, near, margin, inter]
}

/// Pure race inputs from a feature vector (v1 block offsets; no allocation).
pub fn race_features(features: &[f32]) -> [f32; RACE_LEN] {
    let me_hp = features.get(ME_SCALARS).copied().unwrap_or(0.0);
    let opp_hp = features.get(OPP_SCALARS).copied().unwrap_or(0.0);
    let vs_me = board_attack(features, OPP_BOARD);
    let vs_opp = board_attack(features, OWN_BOARD);
    let me = race_side(me_hp, vs_me);
    let opp = race_side(opp_hp, vs_opp);
    [
        me[0], me[1], me[2], me[3], me[4], me[5], opp[0], opp[1], opp[2], opp[3], opp[4], opp[5],
    ]
}

#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub features: Vec<f32>,
    pub ids: Vec<u32>,
}

impl Observation {
    pub const LEN: usize = LEN;
    pub const LEN_V2: usize = LEN_V2;
    pub const LEN_V3: usize = LEN_V3;
    pub const LAYOUT: &'static [LayoutField] = LAYOUT;
    pub const LAYOUT_V2_EXTRA: &'static [LayoutField] = LAYOUT_V2_EXTRA;
    pub const LAYOUT_V3_EXTRA: &'static [LayoutField] = LAYOUT_V3_EXTRA;
    pub const IDS_LEN: usize = IDS_LEN;
    pub const IDS_LEN_V3: usize = IDS_LEN_V3;
    pub const IDS_OPP_HAND: usize = IDS_OPP_HAND;
}

pub fn encode(state: &State, perspective: PlayerId) -> Observation {
    encode_with_vocab(state, perspective, &vocab(state), EncodingVersion::V1, None)
}

pub fn encode_version(
    state: &State,
    perspective: PlayerId,
    version: EncodingVersion,
    db: Option<&CardDb>,
) -> Observation {
    encode_with_vocab(state, perspective, &vocab(state), version, db)
}

/// Same as [`encode`], but the histogram vocabulary is supplied by the
/// caller. Lookups are binary search (`vocab` is sorted). A token that is
/// not in `vocab` is omitted from the histograms (empty column).
pub fn encode_with_vocab(
    state: &State,
    perspective: PlayerId,
    vocab: &[CardId],
    version: EncodingVersion,
    db: Option<&CardDb>,
) -> Observation {
    let vocab = if vocab.len() > HIST_WIDTH {
        &vocab[..HIST_WIDTH]
    } else {
        vocab
    };
    let feat_len = version.feature_len();
    let ids_len = version.ids_len();
    let mut feat = vec![0.0f32; feat_len];
    let mut ids = vec![0u32; ids_len];
    let me = perspective;
    let opp = perspective.opponent();

    match &state.phase {
        Phase::Mulligan { .. } => feat[0] = 1.0,
        Phase::Main | Phase::Combat => feat[1] = 1.0,
        Phase::Choice { .. } => feat[2] = 1.0,
        Phase::End => feat[3] = 1.0,
        Phase::Terminal => feat[4] = 1.0,
    }
    feat[5] = state.turn as f32;
    feat[6] = f32::from(crate::action::acting_player(state) == me);
    feat[7] = match state.winner {
        Some(w) if w == me => 1.0,
        Some(_) => -1.0,
        None => 0.0,
    };
    feat[8] = f32::from(state.first == me);

    if let Phase::Choice { node, .. } = &state.phase {
        let n = choice_option_count(node);
        for i in 0..n.min(ActionId::CHOOSE_N) {
            feat[9 + i] = 1.0;
        }
    }

    write_scalars(&mut feat, 41, state.player(me));
    write_scalars(&mut feat, 70, state.player(opp));

    let hand = &state.player(me).hand;
    for (i, c) in hand.iter().take(HAND_LIMIT).enumerate() {
        feat[99 + i] = c.cost as f32;
        feat[108 + i] = c.spellboost_count as f32;
        feat[117 + i] = c.skybound as f32;
        feat[126 + i] = f32::from(c.flags.was_fused);
        feat[135 + i] = f32::from(c.cant_be_played());
        feat[144 + i] = f32::from(!c.once_used.is_empty());
        ids[IDS_OWN_HAND + i] = c.card.0;
    }

    write_board(&mut feat, 153, &mut ids, IDS_OWN_BOARD, state, me);
    write_board(&mut feat, 253, &mut ids, IDS_OPP_BOARD, state, opp);

    let own_deck = deck_hist(state.player(me), vocab);
    let opp_pool = match version {
        EncodingVersion::V1 => pool_hist(state.player(opp), vocab),
        EncodingVersion::V2 | EncodingVersion::V3 => strict_pool_hist(state.player(opp), vocab),
    };
    for i in 0..HIST_WIDTH {
        feat[353 + i] = own_deck[i];
        feat[353 + HIST_WIDTH + i] = opp_pool[i];
        if i < vocab.len() {
            ids[IDS_OWN_DECK + i] = vocab[i].0;
            ids[IDS_OPP_POOL + i] = vocab[i].0;
        }
    }

    if matches!(version, EncodingVersion::V2 | EncodingVersion::V3) {
        let db = db.expect("encoding v2/v3 requires CardDb for printed stats");
        let deck_bonus = zone_bonuses(&state.player(me).deck, db);
        let hand_bonus = zone_bonuses(&state.player(me).hand, db);
        for (i, v) in deck_bonus.iter().chain(hand_bonus.iter()).enumerate() {
            feat[LEN_V1 + i] = *v;
        }
        let me_own_turn = state.active == me;
        feat[LEN_V1 + 18] = f32::from(state.player(me).can_use_bonus_early(me_own_turn));
        feat[LEN_V1 + 19] = f32::from(state.player(me).can_use_bonus_late());
        feat[LEN_V1 + 20] = f32::from(state.player(opp).can_use_bonus_early(state.active == opp));
        feat[LEN_V1 + 21] = f32::from(state.player(opp).can_use_bonus_late());
    }

    if version == EncodingVersion::V3 {
        write_amulet_soon(&mut feat, LEN_V2, &state.player(me).field);
        write_amulet_soon(&mut feat, LEN_V2 + FIELD_SIZE, &state.player(opp).field);
        let own_entered = enter_hist(state.player(me), vocab);
        let opp_entered = enter_hist(state.player(opp), vocab);
        let empty_hidden = BTreeSet::new();
        let own_cem = cemetery_hist(state.player(me), vocab, &empty_hidden);
        let opp_hidden = opponent_hidden_cemetery(state.player(opp));
        let opp_cem = cemetery_hist(state.player(opp), vocab, &opp_hidden);
        let off_enter = LEN_V2 + 2 * FIELD_SIZE;
        let off_own_cem = off_enter + 2 * HIST_WIDTH;
        for i in 0..HIST_WIDTH {
            feat[off_enter + i] = own_entered[i];
            feat[off_enter + HIST_WIDTH + i] = opp_entered[i];
            feat[off_own_cem + i] = own_cem[i];
            feat[off_own_cem + HIST_WIDTH + i] = opp_cem[i];
        }
        write_crest_ids(&mut ids, IDS_OWN_CRESTS, state.player(me));
        write_crest_ids(&mut ids, IDS_OPP_CRESTS, state.player(opp));
    }

    Observation {
        features: feat,
        ids,
    }
}

fn choice_option_count(node: &ChoiceNode) -> usize {
    match node {
        ChoiceNode::Targets { options, .. } | ChoiceNode::MultiPick { options, .. } => {
            options.len()
        }
        ChoiceNode::Modes { options, .. } => options.len(),
        ChoiceNode::Cards { options, .. } => options.len(),
        ChoiceNode::FusePartners {
            options, picked, ..
        } => options.iter().filter(|p| !picked.contains(p)).count(),
    }
}

fn write_scalars(feat: &mut [f32], off: usize, p: &PlayerState) {
    feat[off] = p.leader_defense as f32;
    feat[off + 1] = p.leader_max as f32;
    feat[off + 2] = p.pp as f32;
    feat[off + 3] = p.pp_max as f32;
    feat[off + 4] = f32::from(p.bonus_pp.active);
    feat[off + 5] = p.ep as f32;
    feat[off + 6] = p.sep as f32;
    feat[off + 7] = p.evolves_used as f32;
    feat[off + 8] = p.shadows as f32;
    feat[off + 9] = p.combo as f32;
    feat[off + 10] = p.earth as f32;
    feat[off + 11] = p.faith as f32;
    feat[off + 12] = p.rally as f32;
    feat[off + 13] = p.turns_taken as f32;
    feat[off + 14] = f32::from(p.is_second);
    feat[off + 15] = p.hand.len() as f32;
    feat[off + 16] = p.deck.len() as f32;
    feat[off + 17] = p.crests.len() as f32;
    for i in 0..CREST_CAP {
        if let Some(c) = p.crests.get(i) {
            feat[off + 18 + i] = c.countdown.unwrap_or(0) as f32;
            feat[off + 23 + i] = f32::from(c.faith);
        }
    }
    feat[off + 28] = f32::from(p.evolved_this_turn);
}

fn write_board(
    feat: &mut [f32],
    off: usize,
    ids: &mut [u32],
    id_off: usize,
    state: &State,
    who: PlayerId,
) {
    for slot in 0..FIELD_SIZE {
        let base = off + slot * BOARD_WIDTH;
        let Some(c) = state.player(who).field[slot].as_ref() else {
            continue;
        };
        ids[id_off + slot] = c.card.0;
        write_board_slot(&mut feat[base..base + BOARD_WIDTH], c, is_bound(state, c));
    }
}

fn write_board_slot(f: &mut [f32], c: &CardInstance, bound: bool) {
    f[0] = c.attack as f32;
    f[1] = c.defense as f32;
    f[2] = c.max_defense as f32;
    f[3] = f32::from(c.evolved);
    f[4] = f32::from(c.super_evolved);
    f[5] = f32::from(c.is_ward());
    f[6] = f32::from(c.is_storm());
    f[7] = f32::from(c.is_rush());
    f[8] = f32::from(c.is_bane());
    f[9] = f32::from(c.is_drain());
    f[10] = f32::from(c.is_barrier());
    f[11] = f32::from(c.ambush_blocks());
    f[12] = f32::from(c.is_aura());
    f[13] = f32::from(c.is_intimidate());
    f[14] = c.traits.damage_cap.unwrap_or(0) as f32;
    f[15] = c.flags.attacks_left as f32;
    f[16] = f32::from(!c.once_used.is_empty());
    f[17] = c.sequence_index as f32;
    f[18] = f32::from(bound);
    f[19] = match c.kind {
        CardKind::Follower => 1.0,
        CardKind::Amulet => 2.0,
        CardKind::Spell => 3.0,
    };
}

fn is_bound(state: &State, c: &CardInstance) -> bool {
    state.bindings.values().flatten().any(|b| match b {
        BoundRef::Field { id, .. } | BoundRef::Hand { id, .. } | BoundRef::Deck { id, .. } => {
            *id == c.id
        }
        _ => false,
    })
}

/// Sorted union of both starting decklists plus visible token ids, capped
/// at [`HIST_WIDTH`]. Public so H0 can compute it once per `choose`.
pub fn vocab(state: &State) -> Vec<crate::card::CardId> {
    let mut set = BTreeSet::new();
    for p in &state.players {
        for id in &p.starting_deck {
            set.insert(*id);
        }
        let (_, adds) = p.derive_public_knowledge();
        for id in adds {
            set.insert(id);
        }
    }
    set.into_iter().take(HIST_WIDTH).collect()
}

fn deck_hist(p: &PlayerState, vocab: &[CardId]) -> [f32; HIST_WIDTH] {
    let mut h = [0.0f32; HIST_WIDTH];
    for c in &p.deck {
        if let Ok(i) = vocab.binary_search(&c.card) {
            h[i] += 1.0;
        }
    }
    h
}

fn pool_hist(p: &PlayerState, vocab: &[CardId]) -> [f32; HIST_WIDTH] {
    let mut h = [0.0f32; HIST_WIDTH];
    for (id, n) in p.known_remaining_pool() {
        if let Ok(i) = vocab.binary_search(&id) {
            h[i] = n as f32;
        }
    }
    h
}

fn write_amulet_soon(feat: &mut [f32], off: usize, field: &[Option<CardInstance>; FIELD_SIZE]) {
    for slot in 0..FIELD_SIZE {
        feat[off + slot] = match field[slot].as_ref() {
            Some(c) if c.kind == CardKind::Amulet => match c.countdown {
                Some(n) => 1.0 / (n.max(1) as f32),
                None => 0.0,
            },
            _ => 0.0,
        };
    }
}

fn enter_hist(p: &PlayerState, vocab: &[CardId]) -> [f32; HIST_WIDTH] {
    let mut h = [0.0f32; HIST_WIDTH];
    for (i, &card) in vocab.iter().enumerate() {
        if let Some(&n) = p.enter_counts.get(&card) {
            h[i] = n as f32;
        }
    }
    h
}

fn opponent_hidden_cemetery(p: &PlayerState) -> BTreeSet<u32> {
    let mut hidden = BTreeSet::new();
    for &id in &p.hidden_removals {
        hidden.insert(id);
    }
    for &id in &p.hidden_cemetery_restores {
        hidden.insert(id);
    }
    hidden
}

fn cemetery_hist(p: &PlayerState, vocab: &[CardId], exclude: &BTreeSet<u32>) -> [f32; HIST_WIDTH] {
    let mut h = [0.0f32; HIST_WIDTH];
    for c in &p.cemetery {
        if exclude.contains(&c.id) {
            continue;
        }
        if let Ok(i) = vocab.binary_search(&c.card) {
            h[i] += 1.0;
        }
    }
    h
}

fn crest_card_id(id: &str) -> u32 {
    let rest = id
        .strip_prefix("crest:")
        .or_else(|| id.strip_prefix("faith:"))
        .unwrap_or("");
    rest.parse().unwrap_or(0)
}

fn write_crest_ids(ids: &mut [u32], off: usize, p: &PlayerState) {
    for i in 0..CREST_CAP {
        if let Some(c) = p.crests.get(i) {
            ids[off + i] = crest_card_id(&c.id);
        }
    }
}

fn strict_pool_hist(p: &PlayerState, vocab: &[CardId]) -> [f32; HIST_WIDTH] {
    let mut h = [0.0f32; HIST_WIDTH];
    for (id, n) in p.strict_remaining_pool() {
        if let Ok(i) = vocab.binary_search(&id) {
            h[i] = n as f32;
        }
    }
    h
}

/// Order-free zone sums: attack/defence bonuses by cost bucket and storm,
/// plus cost reduction. Nine floats per zone.
fn zone_bonuses(cards: &[CardInstance], db: &CardDb) -> [f32; 9] {
    let mut out = [0.0f32; 9];
    for c in cards {
        let printed_cost = match db.card(c.card) {
            Ok(card) => card.cost(),
            Err(_) => c.cost,
        };
        let cost_red = (printed_cost - c.cost).max(0);
        out[8] += cost_red as f32;

        if c.kind != CardKind::Follower {
            continue;
        }
        let (printed_atk, printed_def) = match db.card(c.card) {
            Ok(card) => (card.attack(), card.defense()),
            Err(_) => (c.attack, c.max_defense),
        };
        let atk_bonus = (c.attack - printed_atk) as f32;
        let def_bonus = (c.max_defense - printed_def) as f32;
        let cost = c.cost;
        if cost <= 2 {
            out[0] += atk_bonus;
            out[4] += def_bonus;
        } else if cost <= 4 {
            out[1] += atk_bonus;
            out[5] += def_bonus;
        } else {
            out[2] += atk_bonus;
            out[6] += def_bonus;
        }
        if c.is_storm() {
            out[3] += atk_bonus;
            out[7] += def_bonus;
        }
    }
    out
}

/// Number of options on the current choice node (0 if not in Choice).
pub fn choice_option_len(state: &State) -> usize {
    match &state.phase {
        Phase::Choice { node, .. } => choice_option_count(node),
        _ => 0,
    }
}
