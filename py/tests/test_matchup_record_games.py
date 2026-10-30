"""Replay archives from ``matchup.py --record-games``."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

pytest.importorskip("arena")

ROOT = Path(__file__).resolve().parents[2]
CARDS = str(ROOT / "cards")
DECKS = ROOT / "oracle" / "decks"


def _deck(stem: str) -> dict[str, int]:
    raw = json.loads((DECKS / f"{stem}.json").read_text(encoding="utf-8"))
    return {str(k): int(v) for k, v in raw.items()}


def _run_matchup(tmp_path: Path, tag: str, *extra: str) -> dict:
    out = tmp_path / f"{tag}.json"
    cmd = [
        sys.executable,
        str(ROOT / "py" / "matchup.py"),
        "--cards",
        CARDS,
        "--decks",
        "basic-forest",
        "basic-rune",
        "--games",
        "3",
        "--seed",
        "17",
        "--game-offset",
        "2",
        "--policy-a",
        "h0:nodes=300",
        "--policy-b",
        "random",
        "--threads",
        "1",
        "--out",
        str(out),
        *extra,
    ]
    subprocess.run(cmd, check=True, cwd=ROOT)
    return json.loads(out.read_text(encoding="utf-8"))


def test_record_games_replay_and_out_unchanged(tmp_path: Path) -> None:
    import arena

    db = arena.load_cards(CARDS)
    decks = {"basic-forest": _deck("basic-forest"), "basic-rune": _deck("basic-rune")}
    games_path = tmp_path / "rec.jsonl.gz"
    with_rec = _run_matchup(tmp_path, "a", "--record-games", str(games_path))
    without = _run_matchup(tmp_path, "b")

    def strip_timing(d: dict) -> dict:
        out = json.loads(json.dumps(d))
        out.pop("seconds", None)
        out.pop("games_per_second", None)
        if "summary" in out:
            out["summary"].pop("seconds", None)
            out["summary"].pop("games_per_second", None)
        return out

    assert strip_timing(with_rec) == strip_timing(without)

    import gzip

    lines = gzip.open(games_path, "rt", encoding="utf-8").read().splitlines()
    header = json.loads(lines[0])
    assert header["v"] == 1
    assert header["game_offset"] == 2
    rows = [json.loads(ln) for ln in lines[1:]]
    assert rows
    for row in rows:
        assert row["g"] >= 2
        g = arena.Game(db, int(row["seed"]), decks[row["a"]], decks[row["b"]], row["first"])
        for idx in row["chosen"]:
            legal = g.legal_engine()
            g.apply(legal[int(idx)])
        assert g.hash() == int(row["final_hash"], 16)
        assert g.winner == row["winner"]
        assert g.turn == row["turns"]
        assert len(row["chosen"]) == row["actions"]
        for code in row["paths"]:
            assert code == 255 or 0 <= code <= 9
