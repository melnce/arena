"""arena.matchup game_offset and matchup.py --game-offset."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

_REPO = Path(__file__).resolve().parents[2]
_MATCHUP = _REPO / "py" / "matchup.py"
_FAST = "h0:depth=2,beam=2,k=1,nodes=80,value=v0,tt=0"


def _two_decks(root: Path) -> dict[str, dict[str, int]]:
    names = ("basic-forest", "basic-rune")
    out = {}
    for n in names:
        path = root / "oracle" / "decks" / f"{n}.json"
        out[n] = {str(k): int(v) for k, v in json.loads(path.read_text()).items()}
    return out


def _run_matchup(tmp: Path, *extra: str) -> dict:
    out = tmp / "out.json"
    tmp.mkdir(parents=True, exist_ok=True)
    cmd = [
        sys.executable,
        str(_MATCHUP),
        "--decks",
        "basic-forest",
        "basic-rune",
        "--games",
        "2",
        "--seed",
        "99",
        "--policy-a",
        _FAST,
        "--policy-b",
        _FAST,
        "--records",
        "--out",
        str(out),
        *extra,
    ]
    r = subprocess.run(cmd, cwd=_REPO, capture_output=True, text=True)
    assert r.returncode == 0, r.stderr or r.stdout
    return json.loads(out.read_text())


def test_slices_equal_whole(db, root: Path, tmp_path: Path) -> None:
    import arena

    decks = _two_decks(root)
    whole = arena.matchup(
        db, decks, 4, 99, policy_a=_FAST, policy_b=_FAST, records=True, threads=1
    )
    part0 = arena.matchup(
        db,
        decks,
        2,
        99,
        policy_a=_FAST,
        policy_b=_FAST,
        records=True,
        threads=1,
        game_offset=0,
    )
    part2 = arena.matchup(
        db,
        decks,
        2,
        99,
        policy_a=_FAST,
        policy_b=_FAST,
        records=True,
        threads=1,
        game_offset=2,
    )
    recs = sorted(whole["records"], key=lambda r: (r["a"], r["b"], r["g"]))
    pairs = len(decks) * len(decks)
    assert len(recs) == pairs * 4
    merged = sorted(part0["records"] + part2["records"], key=lambda r: (r["a"], r["b"], r["g"]))
    for a, b in zip(recs, merged, strict=True):
        assert a["g"] == b["g"]
        assert a["seed"] == b["seed"]
        assert a["first"] == b["first"]
        assert a["winner"] == b["winner"]
        assert a["turns"] == b["turns"]
        assert a["actions"] == b["actions"]
    w_aw = sum(p["a_wins"] for row in whole["matrix"].values() for p in row.values())
    p0_aw = sum(p["a_wins"] for row in part0["matrix"].values() for p in row.values())
    p2_aw = sum(p["a_wins"] for row in part2["matrix"].values() for p in row.values())
    assert w_aw == p0_aw + p2_aw


def test_default_unchanged(db, root: Path, tmp_path: Path) -> None:
    import arena

    decks = _two_decks(root)
    base = arena.matchup(
        db, decks, 2, 7, policy_a=_FAST, policy_b=_FAST, threads=1
    )
    explicit0 = arena.matchup(
        db,
        decks,
        2,
        7,
        policy_a=_FAST,
        policy_b=_FAST,
        threads=1,
        game_offset=0,
    )
    assert "game_offset" not in base
    assert "game_offset" not in explicit0
    assert base["matrix"] == explicit0["matrix"]

    no_offset = _run_matchup(tmp_path, "--seed", "7")
    via_cli = _run_matchup(tmp_path / "off0", "--game-offset", "0", "--seed", "7")
    assert "game_offset" not in no_offset
    assert "game_offset" not in via_cli
    assert no_offset["matrix"] == via_cli["matrix"]


def test_refusals(db, root: Path) -> None:
    import arena

    decks = _two_decks(root)
    with pytest.raises(ValueError, match="game_offset"):
        arena.matchup(
            db,
            decks,
            2,
            1,
            policy_a=_FAST,
            policy_b=_FAST,
            export="/tmp/x",
            game_offset=1,
        )
    with pytest.raises((ValueError, OverflowError)):
        arena.matchup(
            db,
            decks,
            2,
            1,
            policy_a=_FAST,
            policy_b=_FAST,
            game_offset=-1,
        )

    r = subprocess.run(
        [
            sys.executable,
            str(_MATCHUP),
            "--decks",
            "basic-forest",
            "basic-rune",
            "--games",
            "1",
            "--game-offset",
            "-1",
            "--out",
            "/tmp/bad.json",
        ],
        cwd=_REPO,
        capture_output=True,
        text=True,
    )
    assert r.returncode != 0
    assert "game-offset" in (r.stderr + r.stdout).lower()
