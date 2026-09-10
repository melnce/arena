"""Replay committed oracle traces through `Game.apply` — Python == native."""

from __future__ import annotations

import gzip
import json
from pathlib import Path

# Five green basic-forest mirrors (none are in oracle/known-divergences.json).
REPLAY_TRACES = [
    "basic-forest-mirror/trace-20260910-0.jsonl.gz",
    "basic-forest-mirror/trace-20260910-1.jsonl.gz",
    "basic-forest-mirror/trace-20260910-2.jsonl.gz",
    "basic-forest-mirror/trace-20260910-3.jsonl.gz",
    "basic-forest-mirror/trace-20260910-4.jsonl.gz",
]


def _counts(ids: list[str]) -> dict[str, int]:
    out: dict[str, int] = {}
    for i in ids:
        out[i] = out.get(i, 0) + 1
    return out


def _replay_one(db, path: Path) -> None:
    import arena

    with gzip.open(path, "rt", encoding="utf-8") as f:
        lines = [json.loads(line) for line in f if line.strip()]
    header, *actions = lines
    game = arena.Game(
        db,
        header["seed"],
        _counts(header["deck_a"]),
        _counts(header["deck_b"]),
        first=header["first"],
        opening_hands=header["opening_hands"],
    )
    for rec in actions:
        game.apply(rec["action"], rng=rec.get("rng"))
        got = game.snapshot()
        want = rec["state"]
        if got.get("phase") == "terminal" and want.get("phase") == "terminal":
            assert got["winner"] == want["winner"], f"{path.name} terminal winner"
            continue
        assert got == want, f"{path.name} i={rec.get('i')} snapshot mismatch"


def test_replay_five_committed_traces(db, root: Path) -> None:
    for rel in REPLAY_TRACES:
        path = root / "oracle" / "traces" / rel
        assert path.is_file(), path
        _replay_one(db, path)
