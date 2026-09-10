//! Player policies: random-legal, first-legal, and H0 (determinized search).
//!
//! This module (and `encode` / `search_key` / `determinize`) must compile for
//! `wasm32-unknown-unknown`: no `std::time::{Instant, SystemTime}`, threads,
//! rayon, `std::fs`, or `getrandom` without the js feature. The node cap is
//! the only search budget. `Policy` is object-safe so WASM can hold
//! `Box<dyn Policy>`.

mod h0;

pub use h0::H0;

use crate::action::Action;
use crate::db::CardDb;
use crate::rng::Xoshiro256ss;
use crate::state::State;

/// Object-safe player. WASM holds `Box<dyn Policy>` from [`by_name`].
pub trait Policy {
    fn choose(
        &mut self,
        db: &CardDb,
        state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize;
}

const NAMES: &[&str] = &["random", "first-legal", "h0"];

/// Names [`by_name`] accepts, in announcement order.
pub fn names() -> &'static [&'static str] {
    NAMES
}

/// Construct a policy by the stable WASM / bench name.
///
/// `seed` is accepted so a host can pass the game seed at construction;
/// `Random` / `FirstLegal` / `H0` do not store it — `choose` uses the
/// caller-supplied rng (typically `policy_rng(seed)`).
pub fn by_name(name: &str, seed: u64) -> Option<Box<dyn Policy>> {
    let _ = seed;
    match name {
        "random" => Some(Box::new(Random)),
        "first-legal" => Some(Box::new(FirstLegal)),
        "h0" => Some(Box::new(H0::default())),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Random;

impl Policy for Random {
    fn choose(
        &mut self,
        _db: &CardDb,
        _state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize {
        if legal.is_empty() {
            return 0;
        }
        rng.gen_range(legal.len() as u32) as usize
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FirstLegal;

impl Policy for FirstLegal {
    fn choose(
        &mut self,
        _db: &CardDb,
        _state: &State,
        legal: &[Action],
        _rng: &mut Xoshiro256ss,
    ) -> usize {
        let _ = legal;
        0
    }
}

#[derive(Debug, Clone)]
pub enum AnyPolicy {
    Random(Random),
    FirstLegal(FirstLegal),
    H0(H0),
}

impl AnyPolicy {
    pub fn parse(s: &str) -> Self {
        match s {
            "first-legal" => AnyPolicy::FirstLegal(FirstLegal),
            "h0" => AnyPolicy::H0(H0::default()),
            _ => AnyPolicy::Random(Random),
        }
    }
}

impl Policy for AnyPolicy {
    fn choose(
        &mut self,
        db: &CardDb,
        state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize {
        match self {
            AnyPolicy::Random(p) => p.choose(db, state, legal, rng),
            AnyPolicy::FirstLegal(p) => p.choose(db, state, legal, rng),
            AnyPolicy::H0(p) => p.choose(db, state, legal, rng),
        }
    }
}
