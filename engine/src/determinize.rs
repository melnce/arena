//! Resample the opponent's hidden cards from the known remaining pool.

use crate::card::CardId;
use crate::ids::PlayerId;
use crate::rng::Xoshiro256ss;
use crate::state::{CardInstance, State};

/// Clone `state` and resample the opponent's hand and deck from their known
/// remaining pool, consistent with `encode(state, perspective)`:
/// hand size is preserved, public tokens stay in hand, the rest are drawn
/// uniformly from the pool, leftover pool cards become the deck. Own side
/// is untouched. `State.rng` is reseeded from `seed`.
pub fn determinize(state: &State, perspective: PlayerId, seed: u64) -> State {
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
