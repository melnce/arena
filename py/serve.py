#!/usr/bin/env python3
"""Local HTTP server: native-engine bot decisions for the web UI.

Stdlib only. Bind 127.0.0.1 by default. See docs/local-bot.md.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

DEFAULT_HOST = "127.0.0.1"
DEFAULT_PORT = 8765
DEFAULT_STRONG = "h0:nodes=16000"
DEFAULT_ORIGINS = (
    "https://arena-nu-one.vercel.app,"
    "http://localhost:5173,"
    "http://127.0.0.1:5173"
)


def repo_root() -> Path:
    """Same walk as `matchup.py`: repo root has `oracle/decks` and `cards/`."""
    here = Path(__file__).resolve().parent
    for d in (Path.cwd(), here, *here.parents):
        if (d / "oracle" / "decks").is_dir() and (d / "cards").is_dir():
            return d
    return Path.cwd()


def parse_deck_json(raw: object) -> dict[str, int]:
    """Client import shapes: `{id: count}` or `[ids]` — same as `matchup.py`."""
    if isinstance(raw, list):
        out: dict[str, int] = {}
        for item in raw:
            key = str(item)
            out[key] = out.get(key, 0) + 1
        return out
    if isinstance(raw, dict):
        out: dict[str, int] = {}
        for k, n in raw.items():
            try:
                count = int(n)
            except (TypeError, ValueError):
                continue
            if count <= 0:
                continue
            out[str(k)] = count
        return out
    raise ValueError("deck JSON must be {id: count} or [ids]")


def decode_deck(raw: object) -> dict[str, int]:
    """Accept a JSON string (UI position log) or an already-parsed object."""
    if isinstance(raw, str):
        raw = json.loads(raw)
    return parse_deck_json(raw)


def is_h0_variant(policy: str) -> bool:
    return policy == "h0" or policy.startswith("h0:")


def effective_policy(requested: str, strong: str) -> str:
    return strong if is_h0_variant(requested) else requested


def git_version(root: Path) -> str:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=root,
            text=True,
            stderr=subprocess.DEVNULL,
        )
        rev = out.strip()
        return rev or "dev"
    except (OSError, subprocess.CalledProcessError):
        return "dev"


def parse_origins(raw: str) -> list[str]:
    return [part.strip() for part in raw.split(",") if part.strip()]


class ServerContext:
    def __init__(
        self,
        db: Any,
        strong: str,
        origins: list[str],
        version: str,
    ) -> None:
        self.db = db
        self.strong = strong
        self.origins = set(origins)
        self.version = version
        self.lock = threading.Lock()


def make_handler(ctx: ServerContext) -> type[BaseHTTPRequestHandler]:
    class Handler(BaseHTTPRequestHandler):
        server_version = "arena-local-bot/1"

        def log_message(self, fmt: str, *args: object) -> None:
            return

        def _origin(self) -> str | None:
            origin = self.headers.get("Origin")
            if origin and origin in ctx.origins:
                return origin
            return None

        def _send_cors(self) -> None:
            allowed = self._origin()
            if allowed:
                self.send_header("Access-Control-Allow-Origin", allowed)
            self.send_header("Access-Control-Allow-Headers", "content-type")

        def _write_json(self, code: int, payload: dict[str, Any]) -> None:
            body = json.dumps(payload).encode("utf-8")
            self.send_response(code)
            self._send_cors()
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def _error(self, code: int, message: str, extra: dict[str, Any] | None = None) -> None:
            payload: dict[str, Any] = {"error": message}
            if extra:
                payload.update(extra)
            self._write_json(code, payload)

        def do_OPTIONS(self) -> None:  # noqa: N802
            self.send_response(204)
            self._send_cors()
            self.end_headers()

        def do_GET(self) -> None:  # noqa: N802
            path = urlparse(self.path).path
            if path != "/health":
                self._error(404, "not found")
                return
            self._write_json(
                200,
                {
                    "ok": True,
                    "strong": ctx.strong,
                    "cpus": os.cpu_count(),
                    "version": ctx.version,
                },
            )

        def do_POST(self) -> None:  # noqa: N802
            path = urlparse(self.path).path
            if path != "/bot":
                self._error(404, "not found")
                return
            try:
                length = int(self.headers.get("Content-Length") or "0")
            except ValueError:
                self._error(400, "invalid Content-Length")
                return
            try:
                raw = self.rfile.read(length)
                body = json.loads(raw.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as e:
                self._error(400, f"invalid JSON: {e}")
                return
            if not isinstance(body, dict):
                self._error(400, "body must be a JSON object")
                return
            with ctx.lock:
                self._handle_bot(body)

        def _handle_bot(self, body: dict[str, Any]) -> None:
            import arena

            try:
                seed = int(body["seed"])
                bot_seed = int(body["botSeed"])
                first = str(body.get("first") or "coin")
                requested = str(body.get("policy") or "")
                if not requested:
                    raise ValueError("policy is required")
                deck_a = decode_deck(body["deckA"])
                deck_b = decode_deck(body["deckB"])
            except (KeyError, TypeError, ValueError) as e:
                self._error(400, str(e))
                return

            policy = effective_policy(requested, ctx.strong)
            hash_ok = "n/a"
            t_ms = 0.0
            turn = 0
            try:
                game = arena.Game(ctx.db, seed, deck_a, deck_b, first)
                for i, step in enumerate(body.get("actions") or []):
                    if not isinstance(step, dict):
                        raise ValueError(f"actions[{i}] must be an object")
                    if "reseed" in step:
                        game.reseed(int(step["reseed"]))
                    else:
                        game.apply(step)
                turn = int(game.turn)
                server_hash = str(game.hash())
                client_hash = body.get("hash")
                if client_hash is not None and str(client_hash) != server_hash:
                    hash_ok = "mismatch"
                    print(
                        f"bot policy={policy} turn={turn} ms=0 hash={hash_ok}",
                        flush=True,
                    )
                    self._write_json(
                        409,
                        {
                            "error": "state hash mismatch",
                            "server": server_hash,
                            "client": str(client_hash),
                        },
                    )
                    return
                hash_ok = "ok"
                t0 = time.perf_counter()
                action = game.bot_action(policy, bot_seed)
                t_ms = (time.perf_counter() - t0) * 1000.0
            except arena.Illegal as e:
                print(
                    f"bot policy={policy} turn={turn} ms={t_ms:.1f} hash={hash_ok}",
                    flush=True,
                )
                self._error(400, str(e))
                return
            except ValueError as e:
                print(
                    f"bot policy={policy} turn={turn} ms={t_ms:.1f} hash={hash_ok}",
                    flush=True,
                )
                self._error(400, str(e))
                return
            except Exception as e:  # noqa: BLE001
                print(
                    f"bot policy={policy} turn={turn} ms={t_ms:.1f} hash={hash_ok}",
                    flush=True,
                )
                self._error(500, str(e))
                return

            print(
                f"bot policy={policy} turn={turn} ms={t_ms:.1f} hash={hash_ok}",
                flush=True,
            )
            self._write_json(
                200,
                {
                    "action": action,
                    "policy": policy,
                    "ms": t_ms,
                    "hash": server_hash,
                },
            )

    return Handler


def make_server(args: argparse.Namespace, db: Any | None = None) -> ThreadingHTTPServer:
    root = repo_root()
    cards = Path(args.cards) if args.cards else (root / "cards")
    if db is None:
        import arena

        db = arena.load_cards(str(cards))
    origins = parse_origins(args.origins)
    ctx = ServerContext(
        db=db,
        strong=args.strong,
        origins=origins,
        version=git_version(root),
    )
    handler = make_handler(ctx)
    httpd = ThreadingHTTPServer((args.host, int(args.port)), handler)
    httpd.arena_ctx = ctx  # type: ignore[attr-defined]
    return httpd


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="py/serve.py",
        description=(
            "Serve native-engine bot decisions to the arena web UI on 127.0.0.1. "
            "The UI probes /health and POSTs the position log to /bot."
        ),
    )
    p.add_argument(
        "--host",
        default=DEFAULT_HOST,
        help="bind address (default: 127.0.0.1 — do not expose)",
    )
    p.add_argument("--port", type=int, default=DEFAULT_PORT, help="bind port (default: 8765)")
    p.add_argument(
        "--strong",
        default=DEFAULT_STRONG,
        help='spec used for every h0 / h0:… request (default: "h0:nodes=16000")',
    )
    p.add_argument(
        "--origins",
        default=DEFAULT_ORIGINS,
        help="comma-separated CORS allow list",
    )
    p.add_argument(
        "--cards",
        default=None,
        help="cards/ directory or repo root (default: repo cards/ as matchup.py finds it)",
    )
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        httpd = make_server(args)
    except Exception as e:  # noqa: BLE001
        print(f"failed to start: {e}", file=sys.stderr)
        return 1
    host, port = httpd.server_address[:2]
    origins = parse_origins(args.origins)
    print(f"arena local bot server")
    print(f"  strong:   {args.strong}")
    print(f"  origins:  {', '.join(origins)}")
    print(f"  listening http://{host}:{port}/")
    print(
        "  Open the site in the same browser session. "
        "http://127.0.0.1 is a potentially-trustworthy origin, so the HTTPS "
        "site may call it; if a browser still blocks mixed content, run the "
        "UI with `npm run dev`."
    )
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nshutting down")
    finally:
        httpd.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
