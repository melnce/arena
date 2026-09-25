//! Resample hidden cards according to an information regime.
//!
//! [`encode`](crate::encode::encode) is unchanged and still masks the
//! opponent's hand. [`Info`] governs what the **search** simulates, not
//! what the **leaf** observes. That separation is deliberate.

use std::collections::{BTreeMap, BTreeSet};

use crate::card::CardId;
use crate::ids::PlayerId;
use crate::rng::Xoshiro256ss;
use crate::state::{CardInstance, State};

/// Distinct from the opponent-hand shuffle stream so the two sides do
/// not correlate under [`Info::Fair`].
const OWN_DECK_SEED_XOR: u64 = 0x9E37_79B9_7F4A_7C15;

/// How much hidden information a search root is allowed to know.
///
/// This is a search-time knob. [`crate::encode::encode`] still masks the
/// opponent's hand on every leaf; `info` does not change that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Info {
    /// A human with open decklists: own draw order is resampled, opponent
    /// hand and deck are resampled under open-information rules.
    Open,
    /// A human with open decklists: own draw order is resampled, opponent
    /// hand and deck are resampled. Own hand is untouched.
    Fair,
    /// Today's default: own draw order is exact, opponent is resampled.
    #[default]
    Draws,
    /// Hard-mode sparring: both sides are the true state. No resampling.
    All,
}

/// Per-root counters for [`Info::Open`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpenStats {
    /// Unknown cards dealt from privately removed instances into hand or deck.
    pub hidden: u32,
    /// Revealed fuse hosts held fixed in the opponent hand.
    pub hosts: u32,
}

/// Clone `state` and resample the opponent's hand and deck from their known
/// remaining pool, consistent with `encode(state, perspective)`:
/// hand size is preserved, public tokens stay in hand, the rest are drawn
/// uniformly from the pool, leftover pool cards become the deck. Own side
/// is untouched. `State.rng` is reseeded from `seed`.
///
/// This is the [`Info::Draws`] path and is bit-identical to the pre-`info`
/// function.
pub fn determinize(state: &State, perspective: PlayerId, seed: u64) -> State {
    determinize_with(state, perspective, seed, Info::Draws)
}

/// [`determinize`] with an explicit [`Info`] regime.
pub fn determinize_with(state: &State, perspective: PlayerId, seed: u64, info: Info) -> State {
    determinize_with_stats(state, perspective, seed, info, None)
}

/// Like [`determinize_with`], optionally recording [`OpenStats`] for
/// [`Info::Open`].
pub fn determinize_with_stats(
    state: &State,
    perspective: PlayerId,
    seed: u64,
    info: Info,
    open_stats: Option<&mut OpenStats>,
) -> State {
    match info {
        Info::All => {
            let mut out = state.clone();
            out.rng.reseed(seed);
            out
        }
        Info::Draws => determinize_draws(state, perspective, seed),
        Info::Fair => {
            let mut out = determinize_draws(state, perspective, seed);
            resample_own_deck(&mut out, perspective, seed ^ OWN_DECK_SEED_XOR);
            out
        }
        Info::Open => determinize_open(state, perspective, seed, open_stats),
    }
}

fn determinize_open(
    state: &State,
    perspective: PlayerId,
    seed: u64,
    open_stats: Option<&mut OpenStats>,
) -> State {
    let mut out = state.clone();
    out.rng.reseed(seed);
    resample_own_deck(&mut out, perspective, seed ^ OWN_DECK_SEED_XOR);
    let stats = determinize_open_opponent(&mut out, perspective, seed);
    if let Some(s) = open_stats {
        *s = stats;
    }
    out.rng.reseed(seed);
    out
}

#[derive(Clone, Copy)]
enum HiddenZone {
    Cemetery,
    Banished,
}

#[derive(Clone)]
struct HiddenSlot {
    inst: CardInstance,
    zone: HiddenZone,
    index: usize,
}

fn determinize_open_opponent(out: &mut State, perspective: PlayerId, seed: u64) -> OpenStats {
    let opp = perspective.opponent();
    let (_, additions) = out.player(opp).derive_public_knowledge();
    let mut add_left = counts(&additions);

    let hand = std::mem::take(&mut out.player_mut(opp).hand);
    let deck = std::mem::take(&mut out.player_mut(opp).deck);

    let mut fixed = Vec::new();
    let mut unknown_hand = Vec::new();
    let mut hosts = 0u32;
    for inst in hand {
        if inst.flags.was_fused {
            fixed.push(inst);
            hosts += 1;
        } else if take_one(&mut add_left, inst.card) {
            fixed.push(inst);
        } else {
            unknown_hand.push(inst);
        }
    }

    let h = unknown_hand.len();
    let d = deck.len();

    let hidden_ids = out.player(opp).hidden_removals.clone();
    let mut hidden_slots = Vec::new();
    for &id in &hidden_ids {
        let cem = &mut out.player_mut(opp).cemetery;
        if let Some(i) = cem.iter().position(|c| c.id == id) {
            let inst = cem.remove(i);
            hidden_slots.push(HiddenSlot {
                inst,
                zone: HiddenZone::Cemetery,
                index: i,
            });
            continue;
        }
        let ban = &mut out.player_mut(opp).banished;
        if let Some(i) = ban.iter().position(|c| c.id == id) {
            let inst = ban.remove(i);
            hidden_slots.push(HiddenSlot {
                inst,
                zone: HiddenZone::Banished,
                index: i,
            });
        }
    }

    let r_ids: BTreeSet<u32> = hidden_slots.iter().map(|s| s.inst.id).collect();
    let hidden_meta: Vec<(u32, HiddenZone, usize)> = hidden_slots
        .iter()
        .map(|s| (s.inst.id, s.zone, s.index))
        .collect();

    let mut pool = unknown_hand;
    pool.extend(deck);
    for slot in hidden_slots {
        pool.push(slot.inst);
    }
    canon_sort(&mut pool);

    let mut rng = Xoshiro256ss::from_seed(seed);
    shuffle(&mut pool, &mut rng);

    let mut hidden_drawn = 0u32;
    for inst in pool.drain(..h.min(pool.len())) {
        if r_ids.contains(&inst.id) {
            hidden_drawn += 1;
        }
        fixed.push(inst);
    }
    out.player_mut(opp).hand = fixed;

    let mut new_deck = Vec::with_capacity(d);
    for inst in pool.drain(..d.min(pool.len())) {
        if r_ids.contains(&inst.id) {
            hidden_drawn += 1;
        }
        new_deck.push(inst);
    }
    out.player_mut(opp).deck = new_deck;

    let mut slots: Vec<(HiddenZone, usize)> = hidden_meta
        .iter()
        .map(|(_, zone, index)| (*zone, *index))
        .collect();
    slots.sort_by_key(|(zone, index)| {
        let z = match zone {
            HiddenZone::Cemetery => 0u8,
            HiddenZone::Banished => 1,
        };
        (z, *index)
    });
    let mut cemetery_restores: Vec<(usize, CardInstance)> = Vec::new();
    let mut banished_restores: Vec<(usize, CardInstance)> = Vec::new();
    for (inst, (zone, index)) in pool.into_iter().zip(slots.iter()) {
        match zone {
            HiddenZone::Cemetery => cemetery_restores.push((*index, inst)),
            HiddenZone::Banished => banished_restores.push((*index, inst)),
        }
    }
    cemetery_restores.sort_by_key(|(i, _)| *i);
    banished_restores.sort_by_key(|(i, _)| *i);
    for (i, inst) in cemetery_restores {
        let idx = i.min(out.player_mut(opp).cemetery.len());
        out.player_mut(opp).cemetery.insert(idx, inst);
    }
    for (i, inst) in banished_restores {
        let idx = i.min(out.player_mut(opp).banished.len());
        out.player_mut(opp).banished.insert(idx, inst);
    }

    OpenStats {
        hidden: hidden_drawn,
        hosts,
    }
}

fn determinize_draws(state: &State, perspective: PlayerId, seed: u64) -> State {
    let mut out = state.clone();
    out.rng.reseed(seed);
    let opp = perspective.opponent();
    let hand_size = out.player(opp).hand.len();
    let (_, additions) = out.player(opp).derive_public_knowledge();
    let mut add_left = counts(&additions);

    let mut hand = std::mem::take(&mut out.player_mut(opp).hand);
    let mut deck = std::mem::take(&mut out.player_mut(opp).deck);
    // Instance order is not information; sort so the same multiset + seed
    // yields the same deal (needed so H0's roots are observation-stable).
    canon_sort(&mut hand);
    canon_sort(&mut deck);
    let mut stay = Vec::new();
    let mut rest = Vec::new();
    for inst in hand {
        if take_one(&mut add_left, inst.card) {
            stay.push(inst);
        } else {
            rest.push(inst);
        }
    }
    rest.extend(deck);
    canon_sort(&mut stay);
    canon_sort(&mut rest);

    let mut rng = Xoshiro256ss::from_seed(seed);
    shuffle(&mut rest, &mut rng);

    let need = hand_size.saturating_sub(stay.len());
    let take = need.min(rest.len());
    stay.extend(rest.drain(..take));
    out.player_mut(opp).hand = stay;
    out.player_mut(opp).deck = rest;
    out
}

/// Canon-sort then shuffle the perspective player's deck. Hand is not
/// touched — a player knows their own hand; only the draw order is hidden.
fn resample_own_deck(out: &mut State, perspective: PlayerId, seed: u64) {
    let mut deck = std::mem::take(&mut out.player_mut(perspective).deck);
    canon_sort(&mut deck);
    let mut rng = Xoshiro256ss::from_seed(seed);
    shuffle(&mut deck, &mut rng);
    out.player_mut(perspective).deck = deck;
}

fn counts(ids: &[CardId]) -> BTreeMap<CardId, u32> {
    let mut m = BTreeMap::new();
    for id in ids {
        *m.entry(*id).or_insert(0) += 1;
    }
    m
}

fn take_one(m: &mut BTreeMap<CardId, u32>, id: CardId) -> bool {
    match m.get_mut(&id) {
        Some(n) if *n > 0 => {
            *n -= 1;
            true
        }
        _ => false,
    }
}

fn canon_sort(items: &mut [CardInstance]) {
    items.sort_by(|a, b| a.card.cmp(&b.card).then(a.id.cmp(&b.id)));
}

fn shuffle(items: &mut [CardInstance], rng: &mut Xoshiro256ss) {
    if items.len() < 2 {
        return;
    }
    for i in (1..items.len()).rev() {
        let j = rng.gen_range((i + 1) as u32) as usize;
        items.swap(i, j);
    }
}
