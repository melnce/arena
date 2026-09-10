//! Fixed action table — every `Action` the engine can return, independent of state.

use crate::action::Action;
use crate::apply::legal_actions;
use crate::db::CardDb;
use crate::ids::{AttackTarget, Slot};
use crate::state::{State, HAND_LIMIT};

/// Total, fixed enumeration. Layout is documented in `docs/engine-api.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionId(pub u8);

impl ActionId {
    pub const COUNT: usize = 114;

    pub const MULLIGAN_BASE: usize = 0;
    pub const MULLIGAN_N: usize = 16;
    pub const PLAY_BASE: usize = 16;
    pub const PLAY_N: usize = HAND_LIMIT;
    pub const ATTACK_BASE: usize = 25;
    pub const ATTACK_N: usize = 30;
    pub const EVOLVE_BASE: usize = 55;
    pub const EVOLVE_N: usize = 10;
    pub const ENGAGE_BASE: usize = 65;
    pub const ENGAGE_N: usize = 5;
    pub const FUSE_BASE: usize = 70;
    pub const FUSE_N: usize = HAND_LIMIT;
    pub const BONUS_PP: usize = 79;
    pub const CHOOSE_BASE: usize = 80;
    pub const CHOOSE_N: usize = 32;
    pub const CONFIRM: usize = 112;
    pub const END_TURN: usize = 113;

    pub fn from_action(action: &Action) -> ActionId {
        let i = match action {
            Action::MulliganConfirm { swap } => {
                let mut mask = 0u8;
                for (b, bit) in swap.iter().enumerate() {
                    if *bit {
                        mask |= 1 << b;
                    }
                }
                Self::MULLIGAN_BASE + mask as usize
            }
            Action::Play { hand } => Self::PLAY_BASE + *hand as usize,
            Action::Attack { attacker, target } => {
                let t = match target {
                    AttackTarget::Slot(s) => s.0 as usize,
                    AttackTarget::Leader => 5,
                };
                Self::ATTACK_BASE + attacker.0 as usize * 6 + t
            }
            Action::Evolve { slot, super_evolve } => {
                Self::EVOLVE_BASE + slot.0 as usize * 2 + usize::from(*super_evolve)
            }
            Action::Engage { slot } => Self::ENGAGE_BASE + slot.0 as usize,
            Action::Fuse { host } => Self::FUSE_BASE + *host as usize,
            Action::BonusPp => Self::BONUS_PP,
            Action::Choose(i) => Self::CHOOSE_BASE + *i as usize,
            Action::Confirm => Self::CONFIRM,
            Action::EndTurn => Self::END_TURN,
        };
        ActionId(i as u8)
    }

    pub fn to_action(self) -> Action {
        let i = self.0 as usize;
        if i < Self::PLAY_BASE {
            let mask = (i - Self::MULLIGAN_BASE) as u8;
            let mut swap = [false; 4];
            for (b, bit) in swap.iter_mut().enumerate() {
                *bit = (mask & (1 << b)) != 0;
            }
            return Action::MulliganConfirm { swap };
        }
        if i < Self::ATTACK_BASE {
            return Action::Play {
                hand: (i - Self::PLAY_BASE) as u8,
            };
        }
        if i < Self::EVOLVE_BASE {
            let rel = i - Self::ATTACK_BASE;
            let attacker = Slot((rel / 6) as u8);
            let t = rel % 6;
            let target = if t < 5 {
                AttackTarget::Slot(Slot(t as u8))
            } else {
                AttackTarget::Leader
            };
            return Action::Attack { attacker, target };
        }
        if i < Self::ENGAGE_BASE {
            let rel = i - Self::EVOLVE_BASE;
            return Action::Evolve {
                slot: Slot((rel / 2) as u8),
                super_evolve: rel % 2 == 1,
            };
        }
        if i < Self::FUSE_BASE {
            return Action::Engage {
                slot: Slot((i - Self::ENGAGE_BASE) as u8),
            };
        }
        if i < Self::BONUS_PP {
            return Action::Fuse {
                host: (i - Self::FUSE_BASE) as u8,
            };
        }
        if i == Self::BONUS_PP {
            return Action::BonusPp;
        }
        if i < Self::CONFIRM {
            return Action::Choose((i - Self::CHOOSE_BASE) as u8);
        }
        if i == Self::CONFIRM {
            return Action::Confirm;
        }
        Action::EndTurn
    }
}

/// Legal bits in table order. Built from `legal_actions`.
pub fn legal_mask(db: &CardDb, state: &State) -> [bool; ActionId::COUNT] {
    let mut mask = [false; ActionId::COUNT];
    for a in legal_actions(db, state) {
        let id = ActionId::from_action(&a);
        let i = id.0 as usize;
        if i < ActionId::COUNT {
            mask[i] = true;
        }
    }
    mask
}

/// `legal_actions` order, as table ids.
pub fn legal_ids(db: &CardDb, state: &State) -> Vec<ActionId> {
    legal_actions(db, state)
        .iter()
        .map(ActionId::from_action)
        .collect()
}
