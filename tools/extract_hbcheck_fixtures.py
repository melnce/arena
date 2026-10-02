#!/usr/bin/env python3
"""Extract hbcheck test fixtures from origin/results holdback1 and review7."""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

OUT = Path(__file__).resolve().parents[1] / "engine/tests/fixtures/hbcheck"

# Six positives: x_died_next, y_alive_end, can_kill_x sure=4; ≥3 free kills.
POSITIVE_MOMENTS = [
    ("play-100005", 28, "a", True),
    ("play-100026", 29, "b", True),
    ("play-100043", 16, "b", True),
    ("play-100085", 58, "a", True),
    ("play-100075", 25, "b", False),
    ("play-100137", 35, "b", False),
]

NEGATIVE_MOMENTS = [
    ("play-100010", 36),
    ("play-100087", 65),
    ("play-100154", 35),
    ("play-100177", 7),
    ("play-100160", 55),
    ("play-100244", 9),
]

OWNER_SERVE = ("review7/games/8752271578818342984-5ce21003.json", 33, "b")


def git_show(path: str) -> dict:
    raw = subprocess.check_output(["git", "show", f"origin/results:{path}"], text=True)
    return json.loads(raw)


def bot_actions_before(actions: list[dict], ply: int, bot: str) -> int:
    n = 0
    for step in actions[:ply]:
        body = next(iter(step.values())) if isinstance(step, dict) else {}
        if isinstance(body, dict) and body.get("player") == bot:
            n += 1
    return n


def trim_game(game: dict, ply: int, extra: dict | None = None) -> dict:
    cap = {
        "seed": game["seed"],
        "deckA": game["deckA"],
        "deckB": game["deckB"],
        "first": game.get("first", "coin"),
        "actions": game["actions"][:ply],
        "ply": ply,
    }
    if extra:
        cap.update(extra)
    return cap


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)

    path, ply, bot = OWNER_SERVE
    game = git_show(path)
    cap = trim_game(
        game,
        ply,
        {
            "serve": True,
            "bot": bot,
            "bot_actions_before": bot_actions_before(game["actions"], ply, bot),
        },
    )
    (OUT / "owner-feline.json").write_text(json.dumps(cap, indent=2) + "\n", encoding="utf-8")
    print("wrote owner-feline.json")

    for game_id, ply, bot, free_kill in POSITIVE_MOMENTS:
        game = git_show(f"holdback1/games/{game_id}.json")
        cap = trim_game(
            game,
            ply,
            {"bot": bot, "game_id": game_id, "free_kill": free_kill},
        )
        name = f"pos-{game_id}-ply{ply:04d}.json"
        (OUT / name).write_text(json.dumps(cap, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {name}")

    for game_id, ply in NEGATIVE_MOMENTS:
        game = git_show(f"holdback1/games/{game_id}.json")
        cap = trim_game(game, ply, {"game_id": game_id})
        name = f"neg-{game_id}-ply{ply:04d}.json"
        (OUT / name).write_text(json.dumps(cap, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {name}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
