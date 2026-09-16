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
