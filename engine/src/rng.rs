//! xoshiro256** inside the state, plus a `ScriptedRng` mode that consumes
//! recorded `Pick`s **by outcome**.
//!
//! # Algorithm
//!
//! The generator is [xoshiro256**](https://prng.di.unimi.it/) (Blackman &
//! Vigna). A `u64` seed is expanded with SplitMix64 into the four 64-bit
//! words (`s0..s3`):
//!
//! ```text
//! splitmix64(z):
//!   z += 0x9E3779B97F4A7C15
//!   z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
//!   z = (z ^ (z >> 27)) * 0x94D049BB133111EB
//!   return z ^ (z >> 31)
//!
//! seed(s):
//!   sm = s
//!   s0,s1,s2,s3 = splitmix64(sm) four times
//!   if all-zero, set s0 = 1
//!
//! next():
//!   result = rotl(s1 * 5, 7) * 9
//!   t = s1 << 17
//!   s2 ^= s0; s3 ^= s1; s1 ^= s2; s0 ^= s3
//!   s2 ^= t
//!   s3 = rotl(s3, 45)
//!   return result
//! ```
//!
//! `gen_range(n)` (n > 0) uses rejection sampling on the high 32 bits of
//! `next()` so the distribution is uniform. `n == 0` is a bug and returns 0.
//!
//! # Scripted mode
//!
//! Replay feeds the trace's `rng` array. Each game-level random decision
//! (`draw`, `random_target`, `random_card`, `random_unused`, `random_split`,
//! `coin`, `reanimate`, `multiset_pick`) consumes the next recorded `Pick`
//! whose `chose` must be among the current candidates; otherwise
//! `OraclePickNotLegal`. The live generator is not advanced in scripted mode.
//!
//! The random-legal *policy* (arena-trace) uses a **separate** xoshiro256**
//! stream so the trace's `rng` array contains only the game's rolls.

use crate::error::OraclePickNotLegal;
use crate::ids::PlayerId;
use crate::trace::{Pick, PickChose, PickWhat};

const SPLITMIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX_M1: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX_M2: u64 = 0x94D0_49BB_1331_11EB;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(SPLITMIX_GAMMA);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(SPLITMIX_M1);
    z = (z ^ (z >> 27)).wrapping_mul(SPLITMIX_M2);
    z ^ (z >> 31)
}

fn rotl(x: u64, k: u32) -> u64 {
    x.rotate_left(k)
}

/// Cloneable xoshiro256** core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xoshiro256ss {
    s: [u64; 4],
}

impl Xoshiro256ss {
    pub fn from_seed(seed: u64) -> Self {
        let mut sm = seed;
        let mut s = [
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
        ];
        if s.iter().all(|&w| w == 0) {
            s[0] = 1;
        }
        Self { s }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = rotl(self.s[1].wrapping_mul(5), 7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = rotl(self.s[3], 45);
        result
    }

    pub fn gen_range(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        if n.is_power_of_two() {
            return (self.next_u64() as u32) & (n - 1);
        }
        let limit = (u32::MAX / n) * n;
        loop {
            let x = (self.next_u64() >> 32) as u32;
            if x < limit {
                return x % n;
            }
        }
    }
}

/// Live generator or a scripted replay of recorded outcomes.
#[derive(Debug, Clone)]
pub enum GameRng {
    Live(Xoshiro256ss),
    Scripted {
        picks: Vec<Pick>,
        index: usize,
        /// Advanced only if a call falls through (should not happen in a green replay).
        fallback: Xoshiro256ss,
    },
}

impl GameRng {
    pub fn live(seed: u64) -> Self {
        GameRng::Live(Xoshiro256ss::from_seed(seed))
    }

    pub fn scripted(picks: Vec<Pick>, seed: u64) -> Self {
        GameRng::Scripted {
            picks,
            index: 0,
            fallback: Xoshiro256ss::from_seed(seed),
        }
    }

    pub fn reseed(&mut self, seed: u64) {
        *self = GameRng::live(seed);
    }

    pub fn is_scripted(&self) -> bool {
        matches!(self, GameRng::Scripted { .. })
    }

    pub fn peek_what(&self) -> Option<PickWhat> {
        match self {
            GameRng::Scripted { picks, index, .. } => picks.get(*index).map(|p| p.what),
            GameRng::Live(_) => None,
        }
    }

    pub fn peek_chose_key(&self) -> Option<String> {
        match self {
            GameRng::Scripted { picks, index, .. } => picks.get(*index).map(|p| p.chose.as_key()),
            GameRng::Live(_) => None,
        }
    }

    pub fn gen_range(&mut self, n: u32) -> u32 {
        match self {
            GameRng::Live(g) => g.gen_range(n),
            GameRng::Scripted { fallback, .. } => fallback.gen_range(n),
        }
    }

    /// Consume a recorded pick or roll `gen_range(candidates.len())`.
    pub fn pick_index(
        &mut self,
        what: PickWhat,
        candidates: &[String],
        emit: &mut Vec<Pick>,
    ) -> Result<usize, OraclePickNotLegal> {
        self.pick_index_among(what, None, candidates, emit)
    }

    /// Like `pick_index`, but records `among` (e.g. `"deck"` for `multiset_pick`).
    pub fn pick_index_among(
        &mut self,
        what: PickWhat,
        among: Option<&str>,
        candidates: &[String],
        emit: &mut Vec<Pick>,
    ) -> Result<usize, OraclePickNotLegal> {
        if candidates.is_empty() {
            return Ok(0);
        }
        match self {
            GameRng::Live(g) => {
                let i = g.gen_range(candidates.len() as u32) as usize;
                emit.push(Pick {
                    what,
                    among: among.map(str::to_string),
                    chose: PickChose::Id(candidates[i].clone()),
                });
                Ok(i)
            }
            GameRng::Scripted { picks, index, .. } => {
                if *index >= picks.len() {
                    return Err(OraclePickNotLegal {
                        what,
                        chose: "<missing>".into(),
                        candidates: candidates.to_vec(),
                    });
                }
                let pick = picks[*index].clone();
                *index += 1;
                let chose = pick.chose.as_key();
                match candidates.iter().position(|c| c == &chose) {
                    Some(i) => Ok(i),
                    None => Err(OraclePickNotLegal {
                        what: pick.what,
                        chose,
                        candidates: candidates.to_vec(),
                    }),
                }
            }
        }
    }

    pub fn pick_player_coin(
        &mut self,
        emit: &mut Vec<Pick>,
    ) -> Result<PlayerId, OraclePickNotLegal> {
        let cands = ["a".to_string(), "b".to_string()];
        let i = self.pick_index(PickWhat::Coin, &cands, emit)?;
        Ok(if i == 0 { PlayerId::A } else { PlayerId::B })
    }

    pub fn take_scripted_split(&mut self) -> Result<Vec<i32>, OraclePickNotLegal> {
        match self {
            GameRng::Scripted { picks, index, .. } => {
                if *index >= picks.len() {
                    return Err(OraclePickNotLegal {
                        what: PickWhat::RandomSplit,
                        chose: "<missing>".into(),
                        candidates: vec![],
                    });
                }
                let pick = picks[*index].clone();
                *index += 1;
                match pick.chose {
                    PickChose::Counts(c) => Ok(c),
                    other => Err(OraclePickNotLegal {
                        what: PickWhat::RandomSplit,
                        chose: other.as_key(),
                        candidates: vec!["[counts]".into()],
                    }),
                }
            }
            GameRng::Live(_) => Err(OraclePickNotLegal {
                what: PickWhat::RandomSplit,
                chose: "<live>".into(),
                candidates: vec![],
            }),
        }
    }
}

/// Policy stream — never mixed with the game's `State.rng`.
pub fn policy_rng(seed: u64) -> Xoshiro256ss {
    Xoshiro256ss::from_seed(seed.wrapping_add(0xA5A5_A5A5_A5A5_A5A5))
}
