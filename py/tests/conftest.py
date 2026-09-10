"""Shared paths for the Python binding tests."""

from __future__ import annotations

import json
from pathlib import Path

import pytest


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for d in here.parents:
        if (d / "cards").is_dir() and (d / "engine").is_dir():
            return d
    return Path.cwd()


@pytest.fixture(scope="session")
def root() -> Path:
    return repo_root()


@pytest.fixture(scope="session")
def db(root: Path):
    import arena

    return arena.load_cards(str(root / "cards"))


def load_deck(path: Path) -> dict[str, int]:
    with path.open() as f:
        raw = json.load(f)
    return {str(k): int(v) for k, v in raw.items()}
