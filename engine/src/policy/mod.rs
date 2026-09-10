//! Player policies: random-legal, first-legal, and H0 (determinized search).

mod h0;

pub use h0::H0;

use crate::action::Action;
use crate::db::CardDb;
use crate::rng::Xoshiro256ss;
use crate::state::State;

pub trait Policy {
    fn choose(
        &mut self,
        db: &CardDb,
        state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize;
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
