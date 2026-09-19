"""Yardstick helpers in ``py/runlib.py``: pooled estimate and verdict pin."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

_PY = Path(__file__).resolve().parents[1]
if str(_PY) not in sys.path:
    sys.path.insert(0, str(_PY))

_spec = importlib.util.spec_from_file_location("arena_runlib", _PY / "runlib.py")
assert _spec and _spec.loader
runlib = importlib.util.module_from_spec(_spec)
sys.modules["arena_runlib"] = runlib
_spec.loader.exec_module(runlib)

from stats import wilson  # noqa: E402


def _summary(
    *,
    a_wins: int,
    decisive: int,
    lo: float = 0.4,
    hi: float = 0.6,
    extra: dict | None = None,
) -> dict:
    rate = a_wins / decisive if decisive else 0.0
    out = {
        "policy_a_win_rate": rate,
        "decisive": decisive,
        "wilson95": [lo, hi],
    }
    if extra:
        out.update(extra)
    return out


def _matrix_one_cell(a_wins: int, games: int, draws: int = 0) -> dict:
    return {
        "d": {
            "d": {
                "a_wins": a_wins,
                "b_wins": games - draws - a_wins,
                "games": games,
                "draws": draws,
            }
        }
    }


def test_reverse_candidate_seat_flip() -> None:
    """A reverse summary where A won 60 % means the candidate won 40 %."""
    rate, ci = runlib.reverse_candidate(
        {"policy_a_win_rate": 0.60, "wilson95": [0.50, 0.69]}
    )
    assert rate == pytest.approx(0.40)
    assert ci == pytest.approx((1.0 - 0.69, 1.0 - 0.50))


def test_pooled_uses_reverse_seat_flip() -> None:
    main = {"summary": _summary(a_wins=50, decisive=100)}
    reverse = {"summary": _summary(a_wins=60, decisive=100)}
    rate, _ci = runlib.pooled_candidate(main, reverse, tag="seat-flip")
    # candidate: 50 (main, seat A) + 40 (reverse, seat B) = 90 / 200
    assert rate == pytest.approx(0.45)


def test_pooled_counts_equal_sum_of_arms() -> None:
    main = {
        "summary": _summary(a_wins=6, decisive=10),
        "matrix": _matrix_one_cell(6, 10),
    }
    reverse = {
        "summary": _summary(a_wins=4, decisive=10),
        "matrix": _matrix_one_cell(4, 10),
    }
    rate, ci = runlib.pooled_candidate(main, reverse, tag="sum-arms")
    # main candidate 6/10 + reverse candidate (10-4)=6/10 → 12/20
    assert rate == pytest.approx(12 / 20)
    assert ci == wilson(12, 20)


def test_pooled_matrix_vs_rate_raises_when_they_disagree() -> None:
    main = {
        "summary": _summary(a_wins=9, decisive=10),
        "matrix": _matrix_one_cell(1, 10),
    }
    reverse = {"summary": _summary(a_wins=5, decisive=10)}
    with pytest.raises(ValueError, match="broken-tag"):
        runlib.pooled_candidate(main, reverse, tag="broken-tag")


def test_verdict_rule_pin() -> None:
    """Standing rule is unchanged: both lows must clear; pooled is not a gate."""
    assert (
        runlib.verdict(0.536, (0.521, 0.551), 0.529, (0.507, 0.550)) == "better"
    )
    assert runlib.verdict(0.389, (0.347, 0.432), 0.500, (0.40, 0.60)) == "worse"
    assert (
        runlib.verdict(0.496, (0.466, 0.527), 0.490, (0.46, 0.52)) == "coin flip"
    )
    both = runlib.verdict(0.510, (0.480, 0.540), 0.400, (0.370, 0.430))
    assert both == "unclear (main, reverse)"
    only_main = runlib.verdict(0.500, (0.478, 0.522), 0.523, (0.508, 0.538))
    assert only_main == "unclear (main)"

    # Sweep 7 finalists (published SUMMARY.md figures).
    nodes = runlib.verdict(0.521, (0.505, 0.536), 0.512, (0.491, 0.534))
    assert nodes == "unclear (reverse)"
    wv60 = runlib.verdict(0.497, (0.482, 0.512), 0.493, (0.472, 0.515))
    assert wv60 == "coin flip"


def test_format_verdict_line_marks_pooled_not_gated() -> None:
    line = runlib.format_verdict_line(
        "h0:nodes=6000",
        0.521,
        (0.505, 0.536),
        0.512,
        (0.491, 0.534),
        0.518,
        (0.505, 0.530),
    )
    assert line.startswith("verdict: h0:nodes=6000 unclear (reverse) — ")
    assert "main 0.521 [0.505, 0.536]" in line
    assert "reverse 0.512 [0.491, 0.534]" in line
    assert "pooled 0.518 [0.505, 0.530] (not gated)" in line
    bare = runlib.format_verdict_line(
        "one-seat", 0.52, (0.50, 0.54), 0.51, (0.49, 0.53)
    )
    assert "pooled" not in bare
