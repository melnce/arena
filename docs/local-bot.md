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
python py\serve.py
```

## Run

```
python py/serve.py
```

On Windows: `python py\serve.py`.

`--strong "<spec>"` overrides the default, e.g. `--strong "h0:nodes=16000"` for the previous bot.

The process prints the strong spec, the CORS origin list, and
`http://127.0.0.1:8765/`. Leave it running. Open the site in the **same
browser session** (the production HTTPS origin is allowed to call
`http://127.0.0.1` — it is a potentially-trustworthy origin). If a
browser still blocks the request, run the UI with `cd ui && npm run dev`.

Settings → **Use local bot server when available** (on by default; the
`?localbot=0` query flag also skips the probe and every `/bot` POST).
In vs-bot the badge next to the policy select should read something like
`bot: local server (h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=8,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=h0-value-v69, 28 cpus)`. No server → `bot: browser` when idle, or `bot: browser (<running spec>)` during a vs-bot game (e.g. `bot: browser (h0:nodes=6000)`). Menu labels name each bot (Quick / Medium / Full strength / Custom spec…); **Full strength** is `py/serve.py`’s `DEFAULT_STRONG` at build time. When the local server is up, it plays **its** spec for every `h0` menu choice—the hint under the policy select says so.
The default samples sixteen worlds at twice the previous node cap. The served bot
takes a deterministic kill before searching (`tkill`, up to 10 000 solver
applies outside the node cap; runbook 32 audit at results `e2c8e1d`) and a
chance-dependent kill only when it holds under eight rerolls on every world
(`tkroll`; sweep 34 at results `c3daad1`). When the search would end the turn
with a kill attack available, the served bot re-scores End Turn against each
kill attack, assuming the opponent's best sure removal line (up to 3 actions),
and attacks when that scores better (`hbcheck`, up to 2 000 applies per root,
outside the node cap; decision holdback3, sweep 35 `376e3a8` and review10
`729410e`). The served bot drops no-op fuses (a fuse whose every
`fused` effect needs PP it cannot pay, with no `wasFused` ability, hand below 9;
sweep 42, results `fdd50ab`, pooled 0.567, +46.6 Elo in the Test Subject mirror).
One vs-bot loop runs per game, so acting while the bot is thinking never
starts a second one. A reply that arrives after the position has moved is
dropped and the move is recomputed, and after three such drops in a row
that decision falls back to the browser bot so play always continues.

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
default includes `okill=8` since sweep 48 (2026-10-10).
`okill=15` since 2026-10-19 (sweep 58, +6 Elo at eight worlds): after each of its lines the bot looks for the opponent's kill also through a Ward-break prefix, slot-freeing trades and play-through lines (plays that only open evolve lines, one follower attack per target); it thinks ≈ 5–7 % longer per decision.
`hbk=16` since 2026-10-11 (sweep 49): when the bot is about to end its turn with a kill attack available, it checks End Turn and each kill again on 16 fresh worlds.
`alloc=world,threads=8` since 2026-10-11: the search runs its worlds on 8 cores; moves are the same for any thread
count. `nodes=64000,k=16` since 2026-10-12 (sweep 50): sixteen sampled worlds instead of eight, about twice the thinking
time per decision on the same cores. `lostrank=4000` since 2026-10-12 (the owner's tablet games): in a turn where every move loses in every world, the bot re-ranks its moves without the opponent's reply instead of playing the first legal one; at sixteen worlds 4 000 nodes give each world the share 2 000 gave at eight.
`oopen=1,onext=40` since 2026-10-15 (sweep 55, +12 Elo at eight worlds): the bot searches the opponent's first action
of his reply and looks one turn further at the end of each reply line; it thinks ≈ 1.5–2 × as long per decision.
`onext=0` since brief YG (2026-10-26; sweep 67: pooled 2 461 / 4 608 = 0.534, +23.7 Elo [+15.3, +32.2], against the same stack with `onext=40`, at eight
worlds): with the network leaf the bot no longer finishes its next turn greedily at the end of each reply line to
look for the opponent's kill after it; the leaf already prices that danger, and the look took about a third of each
decision. The bot thinks about a quarter less per decision (sweep 67's self-play at eight worlds: × 0.76 per action).
`order=h0-order-v59,oorder=h0-order-v53` since brief YA (2026-10-21; sweep 59, +9.7 Elo at eight worlds): the bot
ranks its own moves with a small neural network trained on the search's own scores and keeps the linear
`h0-order-v53` for its model of the opponent's replies; it thinks ≈ 30 % longer per decision.
`net=h0-value-v69` since brief YJ (2026-10-28; sweeps 69 / 69b: +7 Elo against brief YF's `h0-value-v66`, which was
+4 Elo against brief YE's `h0-value-v65`, +9 against brief YC's `h0-value-v64`, itself +32 against
`h0-linear-v4`, all at eight worlds): the bot scores the end of each line it reads with a small neural network
trained on the results of its own games instead of the linear `h0-linear-v4`; it thinks about as long as before.
`deal=block` since brief YK (2026-10-28; sweeps 70 and 70b: +5 Elo against the same stack with independent
deals, at eight worlds): the bot's worlds deal the opponent's hidden hand as consecutive blocks of one shuffle of
the cards it cannot rule out, so together they cover his possible hands evenly. The cheater (`info=all`) sees the
true hand and keeps its spec. The bot thinks as long as before (sweeps 70 and 70b's self-play: × 0.993 and × 0.964 games per hour against independent deals). The client's
"h0 (strong)" option is ignored for node count; change it here:

```
python py/serve.py --strong "h0:nodes=32000"
```

`random` and `first-legal` are never rewritten.

## Cheater (sparring)

The **h0 (cheater — sees your hand)** option is a separate sparring partner
with full information (`info=all`): it clones the true state (your hand and
what is left in both decks) and only reseeds the RNG per search root, so
future draws and random effects stay random. Leaf encoding still masks the
hand — this is a hard-mode diagnostic, not a fair opponent. Without an
explicit `k=`, it builds the default four roots (same as the fair bot).

Pick it in Settings → vs-bot policy (listed after **h0 (strong)**). The UI
sends `h0:nodes=6000,info=all` to the server; the server rewrites that to
its `--cheat` spec (default `h0:nodes=64000,horizon=3,k=16,info=all,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=8,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=h0-value-v69`). Override
with e.g. `python py/serve.py --cheat "h0:nodes=32000,info=all"`. The
`GET /health` payload includes `"cheat": "<spec>"` alongside `"strong"`.

An older server without the `cheat` field would rewrite the cheater request
to its fair `--strong` spec, so the UI refuses to POST and plays the cheater
in the browser instead (badge: `bot: browser (cheater needs an updated local
server)`).

## Log line

Each decision prints one stdout line:

```
bot policy=h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=8,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=h0-value-v69 turn=3 ms=412.0 hash=ok game=1-a1b2c3d4
```

`hash=ok` means the client's `Game.hash()` matched the replayed
position. `hash=mismatch` is a 409 (the UI then falls back to wasm for
the rest of that game). `game=` is the capture id (`seed` plus an
8-hex digest of the decks and first player).

## Your games are recorded

Every successful `/bot` writes the position log to
`results/games/<game_id>.json` (`--games-dir` to change that). A stale
or out-of-order request cannot truncate a file: the server only
overwrites when the incoming `actions` list is longer. When the vs-bot
game ends, the client POSTs the finished log plus the winner to
`/game` (fire-and-forget; failures are a single `console.debug`). That
sets `"final": true` and records who won, including the human's last
turns after the bot's last decision. The finished POST also includes
`humanSide` so `review.py` scores the bot's seat, not the human's. A
repeated seed rolls over to
`-2`, `-3`, so replaying the same seed does not overwrite an earlier
game.

`--no-games` switches capture off. A write failure is logged and
never breaks a bot reply.

`bot_value` on a capture step is attached by the decision's ply: a step can
lack a value, but never carries another step's.

To rank the bot's mistakes against a deep reference:

```
python py/review.py --games results/games --tag review1
```

## Expected decision time

Measured on this box (4 cores; basic-forest vs basic-rune, seed 3):

- H0 mulligan (swap-cost heuristic, no search): **< 1 ms**
- turn 6, 7 legal actions, `POST /bot` with `--strong h0:nodes=16000`:
  **854 ms** (`play` of `90011110`)
- a later, wider mid-game position will be slower; this VM is a
  pessimistic ceiling — the owner's workstation is roughly 3–4×
  faster per core.

"Up to a minute per turn is fine."

## Browser worker (no local server)

When `serve.py` is not reachable, `h0` policies still run in wasm. By default
search runs in a **Web Worker** so the UI stays interactive; `?botworker=0`
restores the synchronous main-thread path (with a full-page “slow mode” overlay
so taps do not hit the board). Each decision replays the action log into a fresh
`Game` in the worker (same model as `/bot`). If a worker decision throws, the
client respawns the pool and retries once, then tries a single fresh worker; only
then does it fall back to the main thread for that move. A worker error is shown
on the badge but does not disable workers for the rest of the page — only after
three consecutive worker failures in one game does the rest of that game stay on
the main-thread path. Test hooks: `?botworker=fail` (always fail),
`?botworker=failonce` (first pool decide fails once, then retry succeeds).

Menu entries **h0 (PC strength)** and **h0 (cheater, PC strength)** are read
from `DEFAULT_STRONG` / `DEFAULT_CHEAT` in this file at UI build time. On a
tablet they use the worker; full PC specs can take seconds to minutes per move.

The settings drawer **Bot speed test** benchmarks ten fixed positions twice
(1 core vs your **Bot cores** setting) in dedicated bench workers (it does not
respawn the game's bot worker pool) and prints a fingerprint for cross-device
comparison.

### Bot cores (browser worker pool)

Settings → **Bot cores**: `Auto (N)`, 1, 2, 4, 6, or 8. Auto uses
`max(1, min(8, navigator.hardwareConcurrency − 1))`. The choice is stored in
`localStorage` (`svwb.botCores`). `?botcores=<n>` overrides it for tests.

When the local server is not used, world-split `h0` specs (`alloc=world` in the
served strong spec) fan out across workers via the H0 job API (`h0JobStart` on
worker 0, parallel kill/world steps, `h0JobFinish`). Worker 0 also runs world
budgets from the queue between coordinator steps. `W = 1` is the same as the
original single-worker path. Test hook: `window.__arena.botPool()` reports
`{ size, alive, busy, generation }`.

## Browser uploads

Vs-bot games can also be captured in the browser and uploaded to the `captures`
branch (see [captures.md](./captures.md)). That path is independent of
`serve.py`’s `results/games/` files; using the local server does not disable
browser uploads.
