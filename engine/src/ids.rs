//! Index-addressed identifiers. No object identity is load-bearing for
//! the snapshot / trace contract.

use serde::{Deserialize, Serialize};

/// Seat A is `deck_a` / `"a"` in the trace; B is `"b"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerId {
    A,
    B,
}

impl PlayerId {
    pub const ALL: [PlayerId; 2] = [PlayerId::A, PlayerId::B];

    pub fn idx(self) -> usize {
        match self {
            PlayerId::A => 0,
            PlayerId::B => 1,
        }
    }

    pub fn opponent(self) -> PlayerId {
        match self {
            PlayerId::A => PlayerId::B,
            PlayerId::B => PlayerId::A,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PlayerId::A => "a",
            PlayerId::B => "b",
        }
    }

    pub fn from_idx(i: usize) -> Self {
        if i == 0 {
            PlayerId::A
        } else {
            PlayerId::B
        }
    }
}

/// Field slot 0..4, entry order, compacted after leaves. Leftmost = oldest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Slot(pub u8);

impl Slot {
    pub fn idx(self) -> usize {
        self.0 as usize
    }
}

/// Hand position in draw order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HandIndex(pub u8);

/// Who goes first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum First {
    Coin,
    A,
    B,
}

/// Combat / effect target: a field slot or a leader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackTarget {
    Slot(Slot),
    Leader,
}
