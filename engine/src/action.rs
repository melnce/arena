//! Actions and the NeutralAction ↔ Action map (`docs/engine-api.md`,
//! `docs/trace-format.md`).

use crate::ids::{AttackTarget, PlayerId, Slot};
use crate::state::{ChoiceNode, Phase, PlayForm, State, TargetOpt};
use crate::trace::{AttackTargetJson, ChooseOptionJson, LeaderWord, NeutralAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    MulliganConfirm {
        swap: [bool; 4],
    },
    Play {
        hand: u8,
    },
    Attack {
        attacker: Slot,
        target: AttackTarget,
    },
    Evolve {
        slot: Slot,
        super_evolve: bool,
    },
    Engage {
        slot: Slot,
    },
    Fuse {
        host: u8,
    },
    BonusPp,
    Choose(u8),
    Confirm,
    EndTurn,
}

pub fn acting_player(state: &State) -> PlayerId {
    match &state.phase {
        Phase::Mulligan { player } | Phase::Choice { player, .. } => *player,
        _ => state.active,
    }
}

pub fn to_neutral(state: &State, action: &Action) -> NeutralAction {
    let p = acting_player(state).as_str().to_string();
    match action {
        Action::MulliganConfirm { swap } => NeutralAction::Mulligan {
            player: p,
            swap: *swap,
        },
        Action::Play { hand } => {
            let card = state
                .player(acting_player(state))
                .hand
                .get(*hand as usize)
                .map(|c| c.card.as_str())
                .unwrap_or_default();
            NeutralAction::Play {
                player: p,
                hand_pos: *hand,
                card,
            }
        }
        Action::Attack { attacker, target } => NeutralAction::Attack {
            player: p,
            attacker_slot: attacker.0,
            target: match target {
                AttackTarget::Slot(s) => AttackTargetJson::Slot { slot: s.0 },
                AttackTarget::Leader => AttackTargetJson::Leader(LeaderWord::Leader),
            },
        },
        Action::Evolve { slot, super_evolve } => NeutralAction::Evolve {
            player: p,
            slot: slot.0,
            super_evolve: *super_evolve,
        },
        Action::Engage { slot } => NeutralAction::Engage {
            player: p,
            slot: slot.0,
        },
        Action::Fuse { host } => NeutralAction::Fuse {
            player: p,
            host_pos: *host,
            partner_pos: Vec::new(),
        },
        Action::BonusPp => NeutralAction::BonusPp { player: p },
        Action::Choose(i) => NeutralAction::Choose {
            player: p,
            option: choose_option_json(state, *i),
        },
        Action::Confirm => NeutralAction::Confirm { player: p },
        Action::EndTurn => NeutralAction::EndTurn { player: p },
    }
}

fn choose_option_json(state: &State, i: u8) -> ChooseOptionJson {
    if let Phase::Choice { node, .. } = &state.phase {
        match node {
            ChoiceNode::Targets { options, .. } | ChoiceNode::MultiPick { options, .. } => {
                return match options.get(i as usize) {
                    Some(TargetOpt::Slot { slot, .. }) => ChooseOptionJson::Slot { slot: *slot },
                    Some(TargetOpt::Leader { .. }) => ChooseOptionJson::Leader(LeaderWord::Leader),
                    Some(TargetOpt::Card(id)) => ChooseOptionJson::Card { card: id.as_str() },
                    Some(TargetOpt::Mode(m)) => ChooseOptionJson::Mode { mode: *m },
                    Some(TargetOpt::Hand { .. }) | None => ChooseOptionJson::Mode { mode: i },
                };
            }
            ChoiceNode::Modes { options, .. } => {
                let m = options.get(i as usize).copied().unwrap_or(i);
                return ChooseOptionJson::Mode { mode: m };
            }
            ChoiceNode::Cards { options, .. } => {
                if let Some(c) = options.get(i as usize) {
                    return ChooseOptionJson::Card { card: c.as_str() };
                }
            }
            ChoiceNode::FusePartners { options, .. } => {
                if let Some(pos) = options.get(i as usize) {
                    return ChooseOptionJson::Mode { mode: *pos };
                }
            }
        }
    }
    ChooseOptionJson::Mode { mode: i }
}

pub fn from_neutral(state: &State, n: &NeutralAction) -> Option<Action> {
    match n {
        NeutralAction::Mulligan { swap, .. } => Some(Action::MulliganConfirm { swap: *swap }),
        NeutralAction::Play { hand_pos, .. } => Some(Action::Play { hand: *hand_pos }),
        NeutralAction::Attack {
            attacker_slot,
            target,
            ..
        } => Some(Action::Attack {
            attacker: Slot(*attacker_slot),
            target: match target {
                AttackTargetJson::Slot { slot } => AttackTarget::Slot(Slot(*slot)),
                AttackTargetJson::Leader(_) => AttackTarget::Leader,
            },
        }),
        NeutralAction::Evolve {
            slot, super_evolve, ..
        } => Some(Action::Evolve {
            slot: Slot(*slot),
            super_evolve: *super_evolve,
        }),
        NeutralAction::Engage { slot, .. } => Some(Action::Engage { slot: Slot(*slot) }),
        NeutralAction::Fuse {
            host_pos,
            partner_pos,
            ..
        } => {
            if partner_pos.is_empty() {
                Some(Action::Fuse { host: *host_pos })
            } else {
                // Completed fuse from the old engine: stash partners on a
                // synthetic Confirm after setting a fuse node. Handled in apply.
                Some(Action::Fuse { host: *host_pos })
            }
        }
        NeutralAction::BonusPp { .. } => Some(Action::BonusPp),
        NeutralAction::Choose { option, .. } => Some(Action::Choose(option_index(state, option))),
        NeutralAction::Confirm { .. } => Some(Action::Confirm),
        NeutralAction::EndTurn { .. } => Some(Action::EndTurn),
    }
}

fn option_index(state: &State, opt: &ChooseOptionJson) -> u8 {
    if let Phase::Choice { node, .. } = &state.phase {
        let opts: Vec<ChooseOptionJson> = match node {
            ChoiceNode::Targets { options, .. } | ChoiceNode::MultiPick { options, .. } => options
                .iter()
                .enumerate()
                .map(|(i, _)| choose_option_json(state, i as u8))
                .collect(),
            ChoiceNode::Modes { options, .. } => options
                .iter()
                .map(|m| ChooseOptionJson::Mode { mode: *m })
                .collect(),
            ChoiceNode::Cards { options, .. } => options
                .iter()
                .map(|c| ChooseOptionJson::Card { card: c.as_str() })
                .collect(),
            ChoiceNode::FusePartners { options, .. } => options
                .iter()
                .map(|p| ChooseOptionJson::Mode { mode: *p })
                .collect(),
        };
        if let Some(i) = opts.iter().position(|o| o == opt) {
            return i as u8;
        }
    }
    0
}

pub fn play_form_label(form: PlayForm) -> &'static str {
    match form {
        PlayForm::Normal => "normal",
        PlayForm::Enhance { .. } => "enhance",
        PlayForm::Accelerate { .. } => "accelerate",
    }
}
