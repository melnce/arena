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
/// caller-supplied rng (typically `policy_rng(seed)`). Unknown names and
/// malformed specs return `None` (the WASM client still lists only
/// [`names`]).
pub fn by_name(name: &str, seed: u64) -> Option<Box<dyn Policy>> {
    let _ = seed;
    Some(Box::new(AnyPolicy::parse_spec(name).ok()?))
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
        Self::parse_spec(s).unwrap_or_else(|e| panic!("{e}"))
    }

    /// `"random"` | `"first-legal"` | `"h0"` ([`H0::default`]) | `"h0-fast"`
    /// ([`H0::fast`]) | `"h0:depth=6,beam=4,k=4,nodes=2000"` — any subset of
    /// keys, the rest default; `k` = `determinizations`, `nodes` = `node_cap`.
    ///
    /// `Err` names the offending token: unknown policy, unknown key, or bad
    /// number.
    pub fn parse_spec(s: &str) -> Result<AnyPolicy, String> {
        let s = s.trim();
        match s {
            "random" => Ok(AnyPolicy::Random(Random)),
            "first-legal" => Ok(AnyPolicy::FirstLegal(FirstLegal)),
            "h0" => Ok(AnyPolicy::H0(H0::default())),
            "h0-fast" => Ok(AnyPolicy::H0(H0::fast())),
            other if other.starts_with("h0:") => parse_h0_params(&other[3..]).map(AnyPolicy::H0),
            other => Err(format!("unknown policy '{other}'")),
        }
    }

    /// Canonical form: `"random"`, `"first-legal"`, `"h0"`, `"h0-fast"`, or
    /// `"h0:depth=…,beam=…,k=…,nodes=…"`.
    pub fn spec(&self) -> String {
        match self {
            AnyPolicy::Random(_) => "random".to_string(),
            AnyPolicy::FirstLegal(_) => "first-legal".to_string(),
            AnyPolicy::H0(h) => {
                if h0_fields_eq(h, &H0::default()) {
                    "h0".to_string()
                } else if h0_fields_eq(h, &H0::fast()) {
                    "h0-fast".to_string()
                } else {
                    format!(
                        "h0:depth={},beam={},k={},nodes={}",
                        h.depth, h.beam, h.determinizations, h.node_cap
                    )
                }
            }
        }
    }
}

fn h0_fields_eq(a: &H0, b: &H0) -> bool {
    a.depth == b.depth
        && a.beam == b.beam
        && a.determinizations == b.determinizations
        && a.node_cap == b.node_cap
}

fn parse_h0_params(body: &str) -> Result<H0, String> {
    let mut h = H0::default();
    if body.is_empty() {
        return Ok(h);
    }
    for token in body.split(',') {
        let token = token.trim();
        if token.is_empty() {
            return Err("unknown key ''".to_string());
        }
        let Some((key, val)) = token.split_once('=') else {
            return Err(format!("unknown key '{token}'"));
        };
        let key = key.trim();
        let val = val.trim();
        match key {
            "depth" => h.depth = parse_num(val)?,
            "beam" => h.beam = parse_num(val)?,
            "k" => h.determinizations = parse_num(val)?,
            "nodes" => h.node_cap = parse_num(val)?,
            other => return Err(format!("unknown key '{other}'")),
        }
    }
    Ok(h)
}

fn parse_num<T: std::str::FromStr>(val: &str) -> Result<T, String> {
    val.parse().map_err(|_| format!("bad number '{val}'"))
}

impl PartialEq for AnyPolicy {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Random(_), Self::Random(_)) | (Self::FirstLegal(_), Self::FirstLegal(_)) => true,
            (Self::H0(a), Self::H0(b)) => h0_fields_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for AnyPolicy {}

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
