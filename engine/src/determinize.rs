//! Resample hidden cards according to an information regime.
//!
//! [`encode`](crate::encode::encode) is unchanged and still masks the
//! opponent's hand. [`Info`] governs what the **search** simulates, not
//! what the **leaf** observes. That separation is deliberate.

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
    /// hand and deck are resampled. Own hand is untouched.
    Fair,
    /// Today's default: own draw order is exact, opponent is resampled.
    #[default]
    Draws,
    /// Hard-mode sparring: both sides are the true state. No resampling.
    All,
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

fn counts(ids: &[CardId]) -> std::collections::BTreeMap<CardId, u32> {
    let mut m = std::collections::BTreeMap::new();
    for id in ids {
        *m.entry(*id).or_insert(0) += 1;
    }
    m
}

fn take_one(m: &mut std::collections::BTreeMap<CardId, u32>, id: CardId) -> bool {
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
