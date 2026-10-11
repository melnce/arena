# Browser game captures

Every vs-bot game in the web client can be stored on the device and uploaded to
the `captures` branch of this repository. The JSON matches the format written by
`py/serve.py` (`_handle_game` / `maybe_write_game`) so `py/review.py` can read
PC server games and browser uploads the same way.

## On disk in git

Uploaded files land at:

```
captures/<YYYY-MM>/<game_id>-<session_id>.json
```

`game_id` is `"{seed}-{sha1}"` with the same canonical deck encoding as
`serve.make_game_id`. `session_id` is a per-browser-session id (rematches get a
new session).

Fetch them with:

```bash
git fetch origin captures
git checkout captures -- captures/
```

## Capture JSON (v1)

Browser captures add `source: "browser"`, `session_id`, `bot_meta` (per bot ply:
`backend` = `worker` | `main` | `server`, optional `ms` and `value`), and
`device`. Bot steps may carry `bot_value` on the action object when the server
returned a root value. Other fields align with the local server: `policy`,
`strong` (the UI build’s `__SERVED_STRONG__`), `engine`, `humanSide`, `winner`,
`final`, timestamps, and the action log.

## Vercel function

`POST /api/capture` (see `api/capture.ts`) validates the body, then commits via
the GitHub Contents API on branch `captures` (created from `main` if missing).
The function creates that branch from `main`'s head when it does not exist yet,
so `vercel.json` on **`main`** must include `"git": {"deploymentEnabled":
{"captures": false}}` before the first upload — otherwise every capture commit
would trigger a full Vercel deployment (wasm rebuild). GitHub Actions already
deploy only from `main` and pull requests.

Existing files are kept when the stored capture is `final` and the upload is
partial, or when the same `final` flag applies and the stored log is longer.

Environment variables (Vercel project settings):

| Variable | Purpose |
|----------|---------|
| `ARENA_CAPTURE_KEY` | Shared secret; clients send `x-capture-key` |
| `ARENA_CAPTURE_GH_TOKEN` | GitHub token with Contents read/write on this repo |
| `ARENA_CAPTURE_REPO` | Default `melnce/arena` |
| `ARENA_CAPTURE_BRANCH` | Default `captures` |

Missing key or token → `503`. Wrong key → `401`.

## Putting the upload key on a device

1. **One-time link:** open the site with `#capturekey=<key>` in the URL. The
   key is saved to `localStorage["svwb.captureKey"]` and stripped from the hash.
2. **Settings → Game uploads:** paste the key in **Upload key** and click **Save**.

Toggle **Upload my games** (default on). **Export games** downloads every stored
capture as one JSON file if upload is unavailable.

## Local server capture unchanged

Games played through `py/serve.py` still write under `results/games/` on the PC.
`reportFinishedGame` still POSTs to `http://127.0.0.1:8765/game`. When the local
server is used, the browser capture also records those bot plies with
`backend: "server"` in `bot_meta` — the same game may exist on the PC and on the
`captures` branch.
