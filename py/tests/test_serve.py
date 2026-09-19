"""HTTP tests for `py/serve.py` (in-process ThreadingHTTPServer)."""

from __future__ import annotations

import importlib.util
import json
import sys
import threading
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request, urlopen

import pytest

pytest.importorskip("arena")


def _repo_root() -> Path:
    here = Path(__file__).resolve()
    for d in here.parents:
        if (d / "cards").is_dir() and (d / "engine").is_dir():
            return d
    return Path.cwd()


def _load_serve():
    root = _repo_root()
    path = root / "py" / "serve.py"
    spec = importlib.util.spec_from_file_location("arena_serve", path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["arena_serve"] = mod
    spec.loader.exec_module(mod)
    return mod


serve = _load_serve()


def _decks(root: Path) -> tuple[dict[str, int], dict[str, int]]:
    forest = json.loads((root / "oracle" / "decks" / "basic-forest.json").read_text())
    rune = json.loads((root / "oracle" / "decks" / "basic-rune.json").read_text())
    return serve.parse_deck_json(forest), serve.parse_deck_json(rune)


@pytest.fixture
def server(db, root: Path):
    args = serve.parse_args(
        [
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--strong",
            "h0:nodes=16000",
            "--cards",
            str(root / "cards"),
            "--no-games",
        ]
    )
    httpd = serve.make_server(args, db=db)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    host, port = httpd.server_address[:2]
    yield f"http://{host}:{port}"
    httpd.shutdown()
    httpd.server_close()
    thread.join(timeout=2)


def _request(
    url: str,
    method: str = "GET",
    body: dict | None = None,
    origin: str | None = None,
    timeout: float = 180.0,
):
    data = None if body is None else json.dumps(body).encode("utf-8")
    headers = {}
    if data is not None:
        headers["Content-Type"] = "application/json"
    if origin is not None:
        headers["Origin"] = origin
    req = Request(url, data=data, headers=headers, method=method)
    try:
        with urlopen(req, timeout=timeout) as resp:
            raw = resp.read()
            payload = json.loads(raw.decode("utf-8")) if raw else {}
            return resp.status, payload, dict(resp.headers)
    except HTTPError as e:
        raw = e.read()
        payload = json.loads(raw.decode("utf-8")) if raw else {}
        return e.code, payload, dict(e.headers)


def test_health_echoes_strong(server: str) -> None:
    status, payload, _ = _request(f"{server}/health")
    assert status == 200
    assert payload["ok"] is True
    assert payload["strong"] == "h0:nodes=16000"
    assert payload["cpus"] is None or int(payload["cpus"]) >= 1
    assert payload["version"]


def _play_plies(db, root: Path, plies: int = 6, seed: int = 1):
    import arena

    deck_a, deck_b = _decks(root)
    game = arena.Game(db, seed, deck_a, deck_b, "a")
    actions = []
    for i in range(plies):
        act = game.bot_action("first-legal", seed + i)
        actions.append(act)
        game.apply(act)
    return game, actions, deck_a, deck_b


def test_bot_h0_uses_strong_and_matches_hash(server: str, db, root: Path) -> None:
    game, actions, deck_a, deck_b = _play_plies(db, root)
    client_hash = str(game.hash())
    legal = game.legal()
    status, payload, _ = _request(
        f"{server}/bot",
        method="POST",
        body={
            "seed": "1",
            "deckA": json.dumps(deck_a),
            "deckB": json.dumps(deck_b),
            "first": "a",
            "actions": actions,
            "policy": "h0",
            "botSeed": "99",
            "hash": client_hash,
        },
    )
    assert status == 200, payload
    assert payload["policy"] == "h0:nodes=16000"
    assert payload["hash"] == client_hash
    assert payload["action"] in legal
    assert payload["ms"] >= 0


def test_bot_wrong_hash_is_409(server: str, db, root: Path) -> None:
    game, actions, deck_a, deck_b = _play_plies(db, root)
    client_hash = str(game.hash())
    status, payload, _ = _request(
        f"{server}/bot",
        method="POST",
        body={
            "seed": "1",
            "deckA": json.dumps(deck_a),
            "deckB": json.dumps(deck_b),
            "first": "a",
            "actions": actions,
            "policy": "h0",
            "botSeed": "99",
            "hash": "0",
        },
    )
    assert status == 409
    assert payload["error"] == "state hash mismatch"
    assert payload["server"] == client_hash
    assert payload["client"] == "0"


def test_bot_random_passes_through(server: str, db, root: Path) -> None:
    game, actions, deck_a, deck_b = _play_plies(db, root)
    status, payload, _ = _request(
        f"{server}/bot",
        method="POST",
        body={
            "seed": "1",
            "deckA": json.dumps(deck_a),
            "deckB": json.dumps(deck_b),
            "first": "a",
            "actions": actions,
            "policy": "random",
            "botSeed": "7",
            "hash": str(game.hash()),
        },
    )
    assert status == 200, payload
    assert payload["policy"] == "random"
    assert payload["action"] in game.legal()


def test_reseed_step_replays(server: str, db, root: Path) -> None:
    import arena

    deck_a, deck_b = _decks(root)
    seed = 1
    game = arena.Game(db, seed, deck_a, deck_b, "a")
    actions = []
    for i in range(2):
        act = game.bot_action("first-legal", seed + i)
        actions.append(act)
        game.apply(act)
    game.reseed(42)
    actions.append({"reseed": "42"})
    for i in range(2):
        act = game.bot_action("first-legal", seed + 10 + i)
        actions.append(act)
        game.apply(act)
    expected = str(game.hash())
    status, payload, _ = _request(
        f"{server}/bot",
        method="POST",
        body={
            "seed": "1",
            "deckA": json.dumps(deck_a),
            "deckB": json.dumps(deck_b),
            "first": "a",
            "actions": actions,
            "policy": "first-legal",
            "botSeed": "3",
            "hash": expected,
        },
    )
    assert status == 200, payload
    assert payload["hash"] == expected
    check = arena.Game(db, seed, deck_a, deck_b, "a")
    for step in actions:
        if "reseed" in step:
            check.reseed(int(step["reseed"]))
        else:
            check.apply(step)
    assert str(check.hash()) == expected


def test_unknown_policy_is_400(server: str, db, root: Path) -> None:
    game, actions, deck_a, deck_b = _play_plies(db, root, plies=1)
    status, payload, _ = _request(
        f"{server}/bot",
        method="POST",
        body={
            "seed": "1",
            "deckA": json.dumps(deck_a),
            "deckB": json.dumps(deck_b),
            "first": "a",
            "actions": actions,
            "policy": "no-such-policy",
            "botSeed": "1",
            "hash": str(game.hash()),
        },
    )
    assert status == 400
    assert "no-such-policy" in payload["error"]


def test_cors_origin_allow_list(server: str) -> None:
    allowed = "http://localhost:5173"
    denied = "https://evil.example"
    _, _, good_headers = _request(f"{server}/health", origin=allowed)
    _, _, bad_headers = _request(f"{server}/health", origin=denied)
    allow_keys = [k for k in good_headers if k.lower() == "access-control-allow-origin"]
    deny_keys = [k for k in bad_headers if k.lower() == "access-control-allow-origin"]
    assert allow_keys
    assert good_headers[allow_keys[0]] == allowed
    assert not deny_keys


def _start_server(db, root: Path, extra: list[str]):
    args = serve.parse_args(
        [
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--strong",
            "h0:nodes=16000",
            "--cards",
            str(root / "cards"),
            *extra,
        ]
    )
    httpd = serve.make_server(args, db=db)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    host, port = httpd.server_address[:2]
    return httpd, thread, f"http://{host}:{port}"


def _stop_server(httpd, thread) -> None:
    httpd.shutdown()
    httpd.server_close()
    thread.join(timeout=2)


def test_bot_writes_game_and_longer_log_wins(db, root: Path, tmp_path: Path) -> None:
    games = tmp_path / "games"
    httpd, thread, url = _start_server(db, root, ["--games-dir", str(games)])
    try:
        game, actions, deck_a, deck_b = _play_plies(db, root, plies=2)
        gid = serve.make_game_id(1, deck_a, deck_b, "a")
        status, payload, _ = _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "policy": "first-legal",
                "botSeed": "3",
                "hash": str(game.hash()),
            },
        )
        assert status == 200, payload
        path = games / f"{gid}.json"
        assert path.is_file()
        first = json.loads(path.read_text())
        assert first["game_id"] == gid
        assert first["final"] is False
        assert first["actions"] == actions
        assert first["policy"] == "first-legal"
        shorter = actions[:1]
        status, _, _ = _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": shorter,
                "policy": "first-legal",
                "botSeed": "4",
            },
        )
        assert status == 200
        mid = json.loads(path.read_text())
        assert mid["actions"] == actions
        game2, longer, _, _ = _play_plies(db, root, plies=4)
        status, _, _ = _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": longer,
                "policy": "first-legal",
                "botSeed": "5",
                "hash": str(game2.hash()),
            },
        )
        assert status == 200
        last = json.loads(path.read_text())
        assert last["actions"] == longer
    finally:
        _stop_server(httpd, thread)


def test_post_game_sets_final_and_winner(db, root: Path, tmp_path: Path) -> None:
    games = tmp_path / "games"
    httpd, thread, url = _start_server(db, root, ["--games-dir", str(games)])
    try:
        game, actions, deck_a, deck_b = _play_plies(db, root, plies=3)
        gid = serve.make_game_id(1, deck_a, deck_b, "a")
        _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions[:2],
                "policy": "first-legal",
                "botSeed": "3",
            },
        )
        status, payload, _ = _request(
            f"{url}/game",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "winner": "a",
            },
        )
        assert status == 200, payload
        assert payload["ok"] is True
        assert payload["game_id"] == gid
        rec = json.loads((games / f"{gid}.json").read_text())
        assert rec["final"] is True
        assert rec["winner"] == "a"
        assert rec["finished"]
        assert rec["actions"] == actions
        assert rec.get("humanSide") is None
        stale = _request(
            f"{url}/game",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions[:1],
                "winner": "b",
            },
        )
        assert stale[0] == 200
        rec2 = json.loads((games / f"{gid}.json").read_text())
        assert rec2["winner"] == "a"
        assert rec2["actions"] == actions
    finally:
        _stop_server(httpd, thread)


def test_no_games_writes_nothing(db, root: Path, tmp_path: Path) -> None:
    games = tmp_path / "games"
    httpd, thread, url = _start_server(db, root, ["--games-dir", str(games), "--no-games"])
    try:
        game, actions, deck_a, deck_b = _play_plies(db, root, plies=2)
        status, payload, _ = _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "policy": "first-legal",
                "botSeed": "3",
                "hash": str(game.hash()),
            },
        )
        assert status == 200, payload
        _request(
            f"{url}/game",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "winner": None,
            },
        )
        assert not games.exists() or not any(games.glob("*.json"))
    finally:
        _stop_server(httpd, thread)


def test_bot_after_final_rolls_over_and_keeps_winner(db, root: Path, tmp_path: Path) -> None:
    games = tmp_path / "games"
    httpd, thread, url = _start_server(db, root, ["--games-dir", str(games)])
    try:
        game, actions, deck_a, deck_b = _play_plies(db, root, plies=3)
        gid = serve.make_game_id(1, deck_a, deck_b, "a")
        _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions[:2],
                "policy": "first-legal",
                "botSeed": "3",
            },
        )
        _request(
            f"{url}/game",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "winner": "a",
            },
        )
        first_path = games / f"{gid}.json"
        first = json.loads(first_path.read_text())
        assert first["final"] is True
        assert first["winner"] == "a"
        first_text = first_path.read_text()
        rematch, rematch_actions, _, _ = _play_plies(db, root, plies=2)
        status, payload, _ = _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": rematch_actions,
                "policy": "first-legal",
                "botSeed": "9",
                "hash": str(rematch.hash()),
            },
        )
        assert status == 200, payload
        second = games / f"{gid}-2.json"
        assert second.is_file()
        rec2 = json.loads(second.read_text())
        assert rec2["final"] is False
        assert rec2["actions"] == rematch_actions
        assert rec2["game_id"] == gid
        assert first_path.read_text() == first_text
        assert json.loads(first_path.read_text())["winner"] == "a"
        assert not (games / f"{gid}-3.json").exists()
    finally:
        _stop_server(httpd, thread)


def test_game_after_rollover_finalises_second_file(db, root: Path, tmp_path: Path) -> None:
    games = tmp_path / "games"
    httpd, thread, url = _start_server(db, root, ["--games-dir", str(games)])
    try:
        game, actions, deck_a, deck_b = _play_plies(db, root, plies=3)
        gid = serve.make_game_id(1, deck_a, deck_b, "a")
        _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions[:2],
                "policy": "first-legal",
                "botSeed": "3",
            },
        )
        _request(
            f"{url}/game",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "winner": "b",
            },
        )
        rematch, rematch_actions, _, _ = _play_plies(db, root, plies=4)
        _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": rematch_actions[:2],
                "policy": "first-legal",
                "botSeed": "11",
            },
        )
        status, payload, _ = _request(
            f"{url}/game",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": rematch_actions,
                "winner": "a",
            },
        )
        assert status == 200, payload
        first = json.loads((games / f"{gid}.json").read_text())
        second = json.loads((games / f"{gid}-2.json").read_text())
        assert first["final"] is True
        assert first["winner"] == "b"
        assert first["actions"] == actions
        assert second["final"] is True
        assert second["winner"] == "a"
        assert second["actions"] == rematch_actions
        assert second["finished"]
        assert payload["game_id"] == gid
    finally:
        _stop_server(httpd, thread)


def test_games_dir_write_failure_still_replies(db, root: Path, tmp_path: Path) -> None:
    blocked = tmp_path / "not-a-dir"
    blocked.write_text("file", encoding="utf-8")
    httpd, thread, url = _start_server(db, root, ["--games-dir", str(blocked)])
    try:
        game, actions, deck_a, deck_b = _play_plies(db, root, plies=2)
        status, payload, _ = _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "policy": "first-legal",
                "botSeed": "3",
                "hash": str(game.hash()),
            },
        )
        assert status == 200, payload
        assert payload["action"]
    finally:
        _stop_server(httpd, thread)


def test_game_persists_human_side(db, root: Path, tmp_path: Path) -> None:
    games = tmp_path / "games"
    httpd, thread, url = _start_server(db, root, ["--games-dir", str(games)])
    try:
        game, actions, deck_a, deck_b = _play_plies(db, root, plies=3)
        gid = serve.make_game_id(1, deck_a, deck_b, "a")
        status, payload, _ = _request(
            f"{url}/game",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "winner": "b",
                "humanSide": "b",
            },
        )
        assert status == 200, payload
        rec = json.loads((games / f"{gid}.json").read_text())
        assert rec["humanSide"] == "b"
        assert rec["winner"] == "b"
    finally:
        _stop_server(httpd, thread)


def test_bot_records_root_value_on_next_capture(db, root: Path, tmp_path: Path) -> None:
    games = tmp_path / "games"
    httpd, thread, url = _start_server(db, root, ["--games-dir", str(games)])
    try:
        game, actions, deck_a, deck_b = _play_plies(db, root, plies=2)
        gid = serve.make_game_id(1, deck_a, deck_b, "a")
        status, payload, _ = _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": actions,
                "policy": "h0",
                "botSeed": "99",
                "hash": str(game.hash()),
                "humanSide": "a",
            },
        )
        assert status == 200, payload
        action = payload["action"]
        assert action
        if "value" in payload:
            assert isinstance(payload["value"], (int, float))
        first = json.loads((games / f"{gid}.json").read_text())
        assert first["humanSide"] == "a"
        game.apply(action)
        longer = [*actions, action]
        status, payload2, _ = _request(
            f"{url}/bot",
            method="POST",
            body={
                "seed": "1",
                "deckA": json.dumps(deck_a),
                "deckB": json.dumps(deck_b),
                "first": "a",
                "actions": longer,
                "policy": "h0",
                "botSeed": "100",
                "hash": str(game.hash()),
                "humanSide": "a",
            },
        )
        assert status == 200, payload2
        rec = json.loads((games / f"{gid}.json").read_text())
        stored = rec["actions"][-1]
        if "value" in payload:
            assert stored.get("bot_value") == payload["value"]
    finally:
        _stop_server(httpd, thread)
