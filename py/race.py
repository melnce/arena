"""Race inputs for the linear leaf (HP bands, board attack vs HP)."""

from __future__ import annotations

import numpy as np

ME_SCALARS = 41
OPP_SCALARS = 70
OWN_BOARD = 153
OPP_BOARD = 253
BOARD_WIDTH = 20
FIELD_SIZE = 5

RACE_NAMES: tuple[str, ...] = (
    "lo5_me",
    "lo10_me",
    "threat_me",
    "near_me",
    "margin_me",
    "inter_me",
    "lo5_opp",
    "lo10_opp",
    "threat_opp",
    "near_opp",
    "margin_opp",
    "inter_opp",
)


def _board_attack(features: np.ndarray, base: int) -> np.ndarray:
    """Sum follower attack on board slots (kind == 1.0)."""
    atk = features[:, base + np.arange(FIELD_SIZE) * BOARD_WIDTH].astype(np.float32, copy=False)
    kind = features[:, base + np.arange(FIELD_SIZE) * BOARD_WIDTH + 19].astype(np.float32, copy=False)
    return (atk * (kind == 1.0)).sum(axis=1, dtype=np.float32)


def _race_side(hp: np.ndarray, atk: np.ndarray) -> tuple[np.ndarray, ...]:
    lo5 = np.maximum(0.0, 5.0 - hp, dtype=np.float32)
    lo10 = np.maximum(0.0, 10.0 - hp, dtype=np.float32)
    threat = (atk >= hp).astype(np.float32)
    near = (atk >= hp - 3.0).astype(np.float32)
    margin = np.clip(hp - atk, -5.0, 10.0).astype(np.float32)
    inter = (hp * atk / 20.0).astype(np.float32)
    return lo5, lo10, threat, near, margin, inter


def race_features(features: np.ndarray) -> np.ndarray:
    """Vectorised race inputs; returns (n, 12) float32."""
    f = np.asarray(features, dtype=np.float32)
    if f.ndim == 1:
        f = f.reshape(1, -1)
    me_hp = f[:, ME_SCALARS]
    opp_hp = f[:, OPP_SCALARS]
    vs_me = _board_attack(f, OPP_BOARD)
    vs_opp = _board_attack(f, OWN_BOARD)
    me = _race_side(me_hp, vs_me)
    opp = _race_side(opp_hp, vs_opp)
    return np.column_stack((*me, *opp)).astype(np.float32, copy=False)
