# Local bot server

Practise against the strongest H0 the workstation can run, in the same
web UI. The browser stays the board; search runs natively on
`127.0.0.1:8765`. Without the server the site is unchanged (wasm bot).

## Install / build

From the repo root, with a Rust **stable** toolchain (CI clippy is 1.98;
`cargo +stable …` if `rustc` is older) and a venv or user pip:

```
pip install maturin pytest numpy
maturin develop --release -m py/Cargo.toml
```

On Windows the path is `py\Cargo.toml`. `maturin` must compile with the
same stable rustc as clippy.

## Windows (owner)

```
cd C:\Users\agban\projects\arena
git pull --ff-only
.\.venv\Scripts\Activate.ps1
maturin develop --release -m py\Cargo.toml
python py\serve.py --strong "h0:nodes=16000"
```

## Run

```
python py/serve.py --strong "h0:nodes=16000"
```

On Windows: `python py\serve.py --strong "h0:nodes=16000"`.

The process prints the strong spec, the CORS origin list, and
`http://127.0.0.1:8765/`. Leave it running. Open the site in the **same
browser session** (the production HTTPS origin is allowed to call
`http://127.0.0.1` — it is a potentially-trustworthy origin). If a
browser still blocks the request, run the UI with `cd ui && npm run dev`.

Settings → **Use local bot server when available** (on by default; the
`?localbot=0` query flag also skips the probe and every `/bot` POST).
In vs-bot the badge next to the policy select should read something like
`bot: local server (h0:nodes=16000, 28 cpus)`. No server → `bot: browser`.

Chrome 142+ treats a public HTTPS page's first `fetch` to `127.0.0.1`
as local-network access. On the first vs-bot game after starting the
server it asks to allow local network access for
`arena-nu-one.vercel.app` — click **Allow**. The client retries the
health probe once for up to 30 s after a 400 ms timeout so the prompt
can be answered. If it was denied once, re-enable it in the site's
permissions (lock icon → Site settings → Local network access), then
reload.

## Strength

`--strong` is what every `h0` / `h0:…` request actually plays. The
client's "h0 (strong)" option is ignored for node count; change it here:

```
python py/serve.py --strong "h0:nodes=32000"
```

`random` and `first-legal` are never rewritten.

## Log line

Each decision prints one stdout line:

```
bot policy=h0:nodes=16000 turn=3 ms=412.0 hash=ok
```

`hash=ok` means the client's `Game.hash()` matched the replayed
position. `hash=mismatch` is a 409 (the UI then falls back to wasm for
the rest of that game).

## Expected decision time

Measured on this box (4 cores; basic-forest vs basic-rune, seed 3):

- H0 mulligan (swap-cost heuristic, no search): **< 1 ms**
- turn 6, 7 legal actions, `POST /bot` with `--strong h0:nodes=16000`:
  **854 ms** (`play` of `90011110`)
- a later, wider mid-game position will be slower; this VM is a
  pessimistic ceiling — the owner's workstation is roughly 3–4×
  faster per core.

"Up to a minute per turn is fine."
