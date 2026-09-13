//! Self-play sample recorder. Wraps any [`Policy`] and logs one sample per
//! `choose` from that seat's perspective. No file IO — this module must
//! compile for `wasm32-unknown-unknown` with the rest of `policy/`.

use crate::action::Action;
use crate::db::CardDb;
use crate::encode::encode;
use crate::ids::PlayerId;
use crate::play::Outcome;
use crate::rng::Xoshiro256ss;
use crate::state::{Phase, State};

use super::h0::value_v0;
use super::Policy;

/// One acting-player decision. `label` is `0.0` until [`Recorder::label`].
#[derive(Debug, Clone)]
pub struct Sample {
    pub features: Vec<f32>,
    pub ids: Vec<u32>,
    pub turn: u32,
    pub phase: u8,
    pub v0: f32,
    pub legal_len: u32,
    pub chosen: u32,
    pub random: bool,
    pub decision_index: u32,
    pub label: f32,
}

/// Wraps `inner` and records every `choose` for seat `me`.
pub struct Recorder {
    inner: Box<dyn Policy>,
    me: PlayerId,
    /// Probability of replacing the inner choice with a uniform legal index.
    /// `0.0` (default) consumes no extra rng — byte-identical to `inner`.
    pub epsilon: f32,
    samples: Vec<Sample>,
    decision_index: u32,
}

impl Recorder {
    pub fn new(inner: Box<dyn Policy>, me: PlayerId) -> Self {
        Self {
            inner,
            me,
            epsilon: 0.0,
            samples: Vec::new(),
            decision_index: 0,
        }
    }

    pub fn with_epsilon(mut self, epsilon: f32) -> Self {
        self.epsilon = epsilon;
        self
    }

    pub fn me(&self) -> PlayerId {
        self.me
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// `+1` if `me` won, `−1` if the opponent won, `0` on a draw.
    pub fn label(&mut self, outcome: &Outcome) {
        let label = match outcome.winner {
            Some(w) if w == self.me => 1.0,
            Some(_) => -1.0,
            None => 0.0,
        };
        for s in &mut self.samples {
            s.label = label;
        }
    }

    pub fn take(self) -> Vec<Sample> {
        self.samples
    }
}

impl Policy for Recorder {
    fn choose(
        &mut self,
        db: &CardDb,
        state: &State,
        legal: &[Action],
        rng: &mut Xoshiro256ss,
    ) -> usize {
        let obs = encode(state, self.me);
        let v0 = value_v0(state, self.me);
        let mut idx = self.inner.choose(db, state, legal, rng);
        let mut random = false;
        // `epsilon == 0` must not touch rng: inner consumption stays identical.
        if self.epsilon > 0.0 && !legal.is_empty() {
            let explore = self.epsilon >= 1.0 || unit_f32(rng) < self.epsilon;
            if explore {
                idx = rng.gen_range(legal.len() as u32) as usize;
                random = true;
            }
        }
        let decision_index = self.decision_index;
        self.decision_index += 1;
        self.samples.push(Sample {
            features: obs.features,
            ids: obs.ids,
            turn: state.turn,
            phase: phase_code(&state.phase),
            v0,
            legal_len: legal.len() as u32,
            chosen: idx as u32,
            random,
            decision_index,
            label: 0.0,
        });
        idx
    }
}

fn phase_code(phase: &Phase) -> u8 {
    match phase {
        Phase::Mulligan { .. } => 0,
        Phase::Main | Phase::Combat => 1,
        Phase::Choice { .. } => 2,
        Phase::End => 3,
        Phase::Terminal => 4,
    }
}

/// Uniform `[0, 1)` from 24 high mantissa bits. Only used when `epsilon > 0`.
fn unit_f32(rng: &mut Xoshiro256ss) -> f32 {
    let u = (rng.next_u64() >> 40) as u32;
    (u as f32) * (1.0 / ((1u32 << 24) as f32))
}
