"""Random games: `Game.hash()` matches arena-trace's snapshot hash."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path

import pytest

SEEDS = list(range(1, 21))
FNV_OFFSET = 0xCBF29CE484222325
FNV_PRIME = 0x0100000001B3


def fnv1a64(data: bytes) -> int:
    h = FNV_OFFSET
    for b in data:
        h ^= b
        h = (h * FNV_PRIME) & 0xFFFFFFFFFFFFFFFF
    return h


def canonical_bytes(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode("utf-8")


def find_arena_trace(root: Path) -> list[str]:
    env = os.environ.get("ARENA_TRACE")
    if env:
        return [env]
    for cand in (
        root / "target" / "release" / "arena-trace",
        root / "target" / "debug" / "arena-trace",
    ):
        if cand.is_file():
            return [str(cand)]
    cargo = shutil.which("cargo")
    if cargo:
        return [cargo, "run", "--quiet", "--release", "--bin", "arena-trace", "--"]
    raise FileNotFoundError("arena-trace binary not found; build it or set ARENA_TRACE")


def run_arena_trace(root: Path, seed: int, out_dir: Path) -> Path:
    deck = root / "engine" / "tests" / "fixtures" / "decks" / "basic-neutral-forest.json"
    cmd = find_arena_trace(root) + [
        "--seed",
        str(seed),
        "--games",
        "1",
        "--deck-a",
        str(deck),
        "--deck-b",
        str(deck),
        "--first",
        "coin",
        "--out",
        str(out_dir),
    ]
    subprocess.run(cmd, cwd=root, check=True, capture_output=True, text=True)
    path = out_dir / f"{seed}.jsonl"
    assert path.is_file(), path
    return path


def test_hash_matches_arena_trace_twenty_seeds(db, root: Path, tmp_path: Path) -> None:
    import arena

    deck_path = root / "engine" / "tests" / "fixtures" / "decks" / "basic-neutral-forest.json"
    deck = json.loads(deck_path.read_text())
    out = tmp_path / "traces"
    out.mkdir()
    for seed in SEEDS:
        trace = run_arena_trace(root, seed, out)
        lines = [json.loads(l) for l in trace.read_text().splitlines() if l.strip()]
        header, *recs = lines
        game = arena.Game(db, seed, deck, deck, first="coin")
        assert header["seed"] == seed
        for rec in recs:
            game.apply(rec["action"])
            snap = game.snapshot()
            expected = fnv1a64(canonical_bytes(rec["state"]))
            assert game.hash() == expected, f"seed={seed} i={rec.get('i')} hash"
            if snap.get("phase") == "terminal" and rec["state"].get("phase") == "terminal":
                assert snap["winner"] == rec["state"]["winner"]
            else:
                assert snap == rec["state"], f"seed={seed} i={rec.get('i')} snapshot"


def test_illegal_raises(db, root: Path) -> None:
    import arena

    deck = json.loads(
        (root / "engine" / "tests" / "fixtures" / "decks" / "basic-neutral-forest.json").read_text()
    )
    game = arena.Game(db, 1, deck, deck, first="a")
    with pytest.raises(arena.Illegal):
        game.apply({"end_turn": {"player": "a"}})
