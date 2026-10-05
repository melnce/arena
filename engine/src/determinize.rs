//! Resample hidden cards according to an information regime.
//!
//! [`encode`](crate::encode::encode) is unchanged and still masks the
//! opponent's hand. [`Info`] governs what the **search** simulates, not
//! what the **leaf** observes. That separation is deliberate.

use std::collections::{BTreeMap, BTreeSet};

use crate::card::{CardId, CardKind};
use crate::ids::PlayerId;
use crate::rng::Xoshiro256ss;
use crate::state::{CardInstance, State, TurnEnd};

/// Distinct from the opponent-hand shuffle stream so the two sides do
/// not correlate under [`Info::Fair`].
const OWN_DECK_SEED_XOR: u64 = 0x9E37_79B9_7F4A_7C15;

/// Distinct from the root shuffle stream for hand-reading SIR candidates.
const HREAD_SEED_XOR: u64 = 0xC2B2_AE3D_27D4_EB4F;

/// Hand-reading deal weights (`hread=on` defaults).
pub const HREAD_EPS_FA: f32 = 0.15;
pub const HREAD_EPS_S: f32 = 0.5;
pub const HREAD_DELTA: f32 = 0.8;
pub const HREAD_M_DEFAULT: u32 = 256;

/// Opt-in hand-reading deal parameters (`hread` spec key).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HreadDeal {
    pub eps_fa: f32,
    pub eps_s: f32,
    pub delta: f32,
    pub m: u32,
}

/// How much hidden information a search root is allowed to know.
///
/// This is a search-time knob. [`crate::encode::encode`] still masks the
/// opponent's hand on every leaf; `info` does not change that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Info {
    /// Deal the opponent only what the bot cannot rule out; own deck is
    /// shuffled (draws pick uniformly at random from the deck, so order
    /// does not affect which card is drawn).
    Open,
    /// A human with open decklists: opponent hand/deck resampled from the
    /// known pool; own deck shuffled (hand untouched). Draws are uniform
    /// random from the deck, so shuffling does not change draw outcomes.
    Fair,
    /// Opponent hand/deck resampled from the known pool; own side untouched.
    #[default]
    Draws,
    /// Hard-mode sparring: true hidden state (opponent hand and both decks'
    /// contents). Only the RNG is reseeded per root — future draws and
    /// random effects stay random.
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
    determinize_block(
        state,
        perspective,
        seed,
        0,
        0,
        info,
        open_stats,
        false,
        None,
    )
}

/// Block-deal variant: one shared shuffle of the opponent's unknown pool
/// per decision; world `i` takes the `i`th consecutive block of `h` (or
/// `need`) cards. `deal_seed` is ignored when `block` is false.
#[allow(clippy::too_many_arguments)]
pub fn determinize_block(
    state: &State,
    perspective: PlayerId,
    root_seed: u64,
    deal_seed: u64,
    world: u32,
    info: Info,
    open_stats: Option<&mut OpenStats>,
    block: bool,
    hread: Option<HreadDeal>,
) -> State {
    match info {
        Info::All => {
            let mut out = state.clone();
            out.rng.reseed(root_seed);
            out
        }
        Info::Draws => {
            determinize_draws_impl(state, perspective, root_seed, deal_seed, world, block)
        }
        Info::Fair => {
            let mut out =
                determinize_draws_impl(state, perspective, root_seed, deal_seed, world, block);
            resample_own_deck(&mut out, perspective, root_seed ^ OWN_DECK_SEED_XOR);
            out
        }
        Info::Open => determinize_open_impl(
            state,
            perspective,
            root_seed,
            deal_seed,
            world,
            open_stats,
            block,
            hread,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn determinize_open_impl(
    state: &State,
    perspective: PlayerId,
    root_seed: u64,
    deal_seed: u64,
    world: u32,
    open_stats: Option<&mut OpenStats>,
    block: bool,
    hread: Option<HreadDeal>,
) -> State {
    let mut out = state.clone();
    out.rng.reseed(root_seed);
    resample_own_deck(&mut out, perspective, root_seed ^ OWN_DECK_SEED_XOR);
    let stats = determinize_open_opponent(
        &mut out,
        perspective,
        root_seed,
        deal_seed,
        world,
        block,
        hread,
    );
    if let Some(s) = open_stats {
        *s = stats;
    }
    out.rng.reseed(root_seed);
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

#[allow(clippy::too_many_arguments)]
fn determinize_open_opponent(
    out: &mut State,
    perspective: PlayerId,
    root_seed: u64,
    deal_seed: u64,
    world: u32,
    block: bool,
    hread: Option<HreadDeal>,
) -> OpenStats {
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
    let mut seen_ids = BTreeSet::new();
    let mut cemetery_hits: Vec<(usize, u32)> = Vec::new();
    let mut banished_hits: Vec<(usize, u32)> = Vec::new();
    for &id in &hidden_ids {
        if !seen_ids.insert(id) {
            continue;
        }
        if let Some(i) = out.player(opp).cemetery.iter().position(|c| c.id == id) {
            cemetery_hits.push((i, id));
        } else if let Some(i) = out.player(opp).banished.iter().position(|c| c.id == id) {
            banished_hits.push((i, id));
        }
    }
    let mut hidden_slots = Vec::new();
    cemetery_hits.sort_by_key(|hit| std::cmp::Reverse(hit.0));
    for (index, id) in cemetery_hits {
        let Some(pos) = out.player(opp).cemetery.iter().position(|c| c.id == id) else {
            continue;
        };
        let inst = out.player_mut(opp).cemetery.remove(pos);
        hidden_slots.push(HiddenSlot {
            inst,
            zone: HiddenZone::Cemetery,
            index,
        });
    }
    banished_hits.sort_by_key(|hit| std::cmp::Reverse(hit.0));
    for (index, id) in banished_hits {
        let Some(pos) = out.player(opp).banished.iter().position(|c| c.id == id) else {
            continue;
        };
        let inst = out.player_mut(opp).banished.remove(pos);
        hidden_slots.push(HiddenSlot {
            inst,
            zone: HiddenZone::Banished,
            index,
        });
    }

    let r_ids: BTreeSet<u32> = hidden_slots.iter().map(|s| s.inst.id).collect();
    let hidden_meta: Vec<(u32, HiddenZone, usize)> = hidden_slots
        .iter()
        .map(|s| (s.inst.id, s.zone, s.index))
        .collect();

    let slot_hand_since: Vec<u32> = unknown_hand.iter().map(|i| i.hand_since).collect();
    let turn_ends = out.player(opp).turn_ends.clone();

    let mut pool = unknown_hand;
    pool.extend(deck);
    for slot in hidden_slots {
        pool.push(slot.inst);
    }
    canon_sort(&mut pool);

    if let Some(hd) = hread {
        pool = hread_deal_pool(pool, h, &slot_hand_since, &turn_ends, root_seed, hd);
    } else if block {
        let mut rng = Xoshiro256ss::from_seed(deal_seed);
        shuffle(&mut pool, &mut rng);
        if !pool.is_empty() {
            let n = pool.len();
            pool.rotate_left((world as usize * h) % n);
        }
    } else {
        let mut rng = Xoshiro256ss::from_seed(root_seed);
        shuffle(&mut pool, &mut rng);
    }

    let mut hidden_drawn = 0u32;
    for (s, inst) in pool.drain(..h.min(pool.len())).enumerate() {
        if r_ids.contains(&inst.id) {
            hidden_drawn += 1;
        }
        let mut dealt = inst;
        if s < slot_hand_since.len() {
            dealt.hand_since = slot_hand_since[s];
        }
        fixed.push(dealt);
    }
    out.player_mut(opp).hand = fixed;

    if block {
        let mut rng = Xoshiro256ss::from_seed(root_seed);
        shuffle(&mut pool, &mut rng);
    }

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
        out.player_mut(opp).cemetery.insert(i, inst);
    }
    for (i, inst) in banished_restores {
        out.player_mut(opp).banished.insert(i, inst);
    }

    OpenStats {
        hidden: hidden_drawn,
        hosts,
    }
}

fn determinize_draws_impl(
    state: &State,
    perspective: PlayerId,
    root_seed: u64,
    deal_seed: u64,
    world: u32,
    block: bool,
) -> State {
    let mut out = state.clone();
    out.rng.reseed(root_seed);
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

    let need = hand_size.saturating_sub(stay.len());
    if block {
        let mut rng = Xoshiro256ss::from_seed(deal_seed);
        shuffle(&mut rest, &mut rng);
        if !rest.is_empty() {
            let n = rest.len();
            rest.rotate_left((world as usize * need) % n);
        }
    } else {
        let mut rng = Xoshiro256ss::from_seed(root_seed);
        shuffle(&mut rest, &mut rng);
    }

    let take = need.min(rest.len());
    stay.extend(rest.drain(..take));
    if block {
        let mut rng = Xoshiro256ss::from_seed(root_seed);
        shuffle(&mut rest, &mut rng);
    }
    out.player_mut(opp).hand = stay;
    out.player_mut(opp).deck = rest;
    out
}

/// Canon-sort then shuffle the perspective player's deck. Hand is not
/// touched. Draws pick uniformly at random from the deck, so this shuffle
/// does not change which card is drawn — it only varies the stored order.
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

fn hread_factor(card: &CardInstance, te: &TurnEnd, eps_fa: f32, eps_s: f32, delta: f32) -> f32 {
    let cost = card.base_cost;
    if cost <= te.unspent {
        let playable = card.kind != CardKind::Follower || !te.board_full;
        if playable {
            return match card.kind {
                CardKind::Spell => eps_s,
                CardKind::Follower | CardKind::Amulet => eps_fa,
            };
        }
    }
    if cost <= te.pp_max {
        delta
    } else {
        1.0
    }
}

fn hread_slot_log_weight(
    card: &CardInstance,
    hand_since: u32,
    turn_ends: &[TurnEnd],
    eps_fa: f32,
    eps_s: f32,
    delta: f32,
) -> f32 {
    let mut log_w = 0.0f32;
    for te in turn_ends.iter().skip(hand_since as usize) {
        let f = hread_factor(card, te, eps_fa, eps_s, delta);
        if f != 1.0 {
            log_w += f.ln();
        }
    }
    log_w
}

fn hread_perm_log_weight(
    perm: &[CardInstance],
    h: usize,
    slot_hand_since: &[u32],
    turn_ends: &[TurnEnd],
    hd: HreadDeal,
) -> f32 {
    let mut log_w = 0.0f32;
    for s in 0..h.min(perm.len()).min(slot_hand_since.len()) {
        log_w += hread_slot_log_weight(
            &perm[s],
            slot_hand_since[s],
            turn_ends,
            hd.eps_fa,
            hd.eps_s,
            hd.delta,
        );
    }
    log_w
}

fn weighted_pick(log_weights: &[f32], rng: &mut Xoshiro256ss) -> usize {
    if log_weights.is_empty() {
        return 0;
    }
    let max_log = log_weights
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let mut weights = Vec::with_capacity(log_weights.len());
    let mut sum = 0.0f64;
    for lw in log_weights {
        let w = (lw - max_log).exp() as f64;
        weights.push(w);
        sum += w;
    }
    if sum <= 0.0 {
        return rng.gen_range(log_weights.len() as u32) as usize;
    }
    let u = (rng.next_u64() as f64) / (u64::MAX as f64) * sum;
    let mut acc = 0.0f64;
    for (i, w) in weights.iter().enumerate() {
        acc += *w;
        if u < acc {
            return i;
        }
    }
    weights.len() - 1
}

fn hread_deal_pool(
    pool: Vec<CardInstance>,
    h: usize,
    slot_hand_since: &[u32],
    turn_ends: &[TurnEnd],
    root_seed: u64,
    hd: HreadDeal,
) -> Vec<CardInstance> {
    if h == 0 || pool.is_empty() {
        return pool;
    }
    let m = hd.m.max(1) as usize;
    let mut rng = Xoshiro256ss::from_seed(root_seed ^ HREAD_SEED_XOR);
    let mut perms = Vec::with_capacity(m);
    let mut log_weights = Vec::with_capacity(m);
    for _ in 0..m {
        let mut perm = pool.clone();
        shuffle(&mut perm, &mut rng);
        log_weights.push(hread_perm_log_weight(
            &perm,
            h,
            slot_hand_since,
            turn_ends,
            hd,
        ));
        perms.push(perm);
    }
    let pick = weighted_pick(&log_weights, &mut rng);
    perms.remove(pick)
}
