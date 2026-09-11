//! `State::reseed` replaces the RNG only. Hash stays the same; draws change.

mod common;

use arena_engine::{hash, PlayerId, State};

use common::{end_turn, load_db, started_decks};

/// Card ids in hand then remaining deck — `draw_one` picks by card id, so
/// this fingerprint only moves when the drawn *kind* changes.
fn pile(st: &State, who: PlayerId) -> Vec<String> {
    let p = st.player(who);
    p.hand
        .iter()
        .map(|c| c.card.as_str())
        .chain(p.deck.iter().map(|c| c.card.as_str()))
        .collect()
}

/// Forty distinct ids so two RNG picks cannot collapse onto the same
/// padded vanilla (`draw_one` takes the first matching `card.id`).
fn unique_mix(db: &arena_engine::CardDb) -> Vec<String> {
    db.cards.keys().map(|id| id.as_str()).take(40).collect()
}

#[test]
fn reseed_same_seed_draws_the_same_different_seeds_differ() {
    let db = load_db();
    let owned = unique_mix(&db);
    assert_eq!(owned.len(), 40, "need 40 distinct card ids from the db");
    let mix: Vec<&str> = owned.iter().map(String::as_str).collect();
    let base = started_decks(&db, 1, &mix, &mix);
    let h0 = hash(&base);

    let mut a = base.clone();
    a.reseed(11);
    assert_eq!(hash(&a), h0, "reseed must not change the canonical hash");
    end_turn(&db, &mut a);
    let pile_a = pile(&a, PlayerId::B);

    let mut b = base.clone();
    b.reseed(11);
    end_turn(&db, &mut b);
    assert_eq!(
        pile(&b, PlayerId::B),
        pile_a,
        "same reseed seed must draw the same"
    );

    let mut c = base.clone();
    c.reseed(99);
    assert_eq!(hash(&c), h0);
    end_turn(&db, &mut c);
    assert_ne!(
        pile(&c, PlayerId::B),
        pile_a,
        "different reseed seeds must draw differently"
    );
}
