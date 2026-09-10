//! Perspective-masked observation. Perfect information stays in `State`.

use std::collections::BTreeSet;

use crate::action_id::ActionId;
use crate::ids::PlayerId;
use crate::state::{
    BoundRef, CardInstance, ChoiceNode, Phase, PlayerState, State, CREST_CAP, FIELD_SIZE,
    HAND_LIMIT,
};

/// Padded histogram width over the two decklists' union (plus tokens).
pub const HIST_WIDTH: usize = 96;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutField {
    pub name: &'static str,
    pub offset: usize,
    pub width: usize,
}

/// Name / offset / width of every feature block. Contiguous, covers `LEN`.
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

/// `own_deck_hist` offset + 2 × histogram.
pub const LEN: usize = 353 + 2 * HIST_WIDTH;

/// Parallel card-id vector (never one-hot). Opponent-hand region is always 0.
pub const IDS_OWN_HAND: usize = 0;
pub const IDS_OWN_DECK: usize = 9;
pub const IDS_OPP_BOARD: usize = 9 + HIST_WIDTH;
pub const IDS_OWN_BOARD: usize = 9 + HIST_WIDTH + FIELD_SIZE;
pub const IDS_OPP_POOL: usize = 9 + HIST_WIDTH + 2 * FIELD_SIZE;
pub const IDS_OPP_HAND: usize = 9 + 2 * HIST_WIDTH + 2 * FIELD_SIZE;
pub const IDS_LEN: usize = IDS_OPP_HAND + HAND_LIMIT;

const BOARD_WIDTH: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub features: Vec<f32>,
    pub ids: Vec<u32>,
}

impl Observation {
    pub const LEN: usize = LEN;
    pub const LAYOUT: &'static [LayoutField] = LAYOUT;
    pub const IDS_LEN: usize = IDS_LEN;
    pub const IDS_OPP_HAND: usize = IDS_OPP_HAND;
}

pub fn encode(state: &State, perspective: PlayerId) -> Observation {
    let mut feat = vec![0.0f32; LEN];
    let mut ids = vec![0u32; IDS_LEN];
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

    let vocab = vocab(state);
    write_board(&mut feat, 153, &mut ids, IDS_OWN_BOARD, state, me);
    write_board(&mut feat, 253, &mut ids, IDS_OPP_BOARD, state, opp);

    let own_deck = deck_hist(state.player(me), &vocab);
    let opp_pool = pool_hist(state.player(opp), &vocab);
    for i in 0..HIST_WIDTH {
        feat[353 + i] = own_deck[i];
        feat[353 + HIST_WIDTH + i] = opp_pool[i];
        if i < vocab.len() {
            ids[IDS_OWN_DECK + i] = vocab[i].0;
            ids[IDS_OPP_POOL + i] = vocab[i].0;
        }
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
        crate::card::CardKind::Follower => 1.0,
        crate::card::CardKind::Amulet => 2.0,
        crate::card::CardKind::Spell => 3.0,
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

fn vocab(state: &State) -> Vec<crate::card::CardId> {
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

fn deck_hist(p: &PlayerState, vocab: &[crate::card::CardId]) -> [f32; HIST_WIDTH] {
    let mut h = [0.0f32; HIST_WIDTH];
    for c in &p.deck {
        if let Some(i) = vocab.iter().position(|id| *id == c.card) {
            h[i] += 1.0;
        }
    }
    h
}

fn pool_hist(p: &PlayerState, vocab: &[crate::card::CardId]) -> [f32; HIST_WIDTH] {
    let mut h = [0.0f32; HIST_WIDTH];
    for (id, n) in p.known_remaining_pool() {
        if let Some(i) = vocab.iter().position(|v| *v == id) {
            h[i] = n as f32;
        }
    }
    h
}

/// Number of options on the current choice node (0 if not in Choice).
pub fn choice_option_len(state: &State) -> usize {
    match &state.phase {
        Phase::Choice { node, .. } => choice_option_count(node),
        _ => 0,
    }
}
