//! Exhaustive `Side` / `Controller` / `CrestPlayer` resolution — no `_ =>` fallbacks.

use crate::card::{Controller, CrestPlayer, Side};
use crate::ids::PlayerId;

/// Which sides a pool selector covers (`Side::Any` = both leaders).
pub fn players_for_side(side: Side, controller: PlayerId) -> Vec<PlayerId> {
    match side {
        Side::Ally => vec![controller],
        Side::Enemy => vec![controller.opponent()],
        Side::Any => vec![controller, controller.opponent()],
    }
}

/// Condition / `fieldHas` selectors: default `None` → allied only.
pub fn players_for_option_side(side: Option<Side>, who: PlayerId) -> Vec<PlayerId> {
    match side {
        None | Some(Side::Ally) => vec![who],
        Some(Side::Enemy) => vec![who.opponent()],
        Some(Side::Any) => vec![who, who.opponent()],
    }
}

/// `enterCountAtLeast`: default `None` and `ally` → acting player; `enemy` → opponent.
/// `any` is rejected at card load (`card.invalid_side_usage`).
pub fn player_enter_count_at_least(side: Option<Side>, who: PlayerId) -> PlayerId {
    match side {
        None | Some(Side::Ally) => who,
        Some(Side::Enemy) => who.opponent(),
        Some(Side::Any) => {
            panic!("enterCountAtLeast side=any must be rejected at load");
        }
    }
}

/// Summon / copy-instance controller hint; default `None` and `Self_` → effect controller.
pub fn player_for_controller(ctrl: Option<Controller>, controller: PlayerId) -> PlayerId {
    match ctrl {
        None | Some(Controller::Self_) => controller,
        Some(Controller::Opponent) => controller.opponent(),
    }
}

/// Crest draw / similar; default `None` and `Self_` → effect controller.
pub fn player_for_crest_player(player: Option<CrestPlayer>, controller: PlayerId) -> PlayerId {
    match player {
        None | Some(CrestPlayer::Self_) => controller,
        Some(CrestPlayer::Opponent) => controller.opponent(),
    }
}

/// `boundHas` side filter on a resolved player.
pub fn bound_has_side_ok(side: Option<Side>, who: PlayerId, player: PlayerId) -> bool {
    match side {
        None | Some(Side::Any) => true,
        Some(Side::Ally) => player == who,
        Some(Side::Enemy) => player == who.opponent(),
    }
}

/// Static-ability owner filter.
pub fn static_pool_side_ok(side: Side, inst_owner: PlayerId, static_owner: PlayerId) -> bool {
    match side {
        Side::Any => true,
        Side::Ally => inst_owner == static_owner,
        Side::Enemy => inst_owner != static_owner,
    }
}
