# review8 — 9 owner games vs the full-strength served bot, Portal AF mirror (2026-10-02)

Portal AF mirror (`meta-portal-af` both sides; the owner chose it as the bot's best deck in bot-vs-bot play), the owner
on side `a`, played through `py/serve.py` on engine **`1bcb402`** (v3 default leaf; WN merged, `tkill` / `tkroll` /
`okill` / `omacro` off), served spec `h0:nodes=32000,horizon=3,k=8` (serve.py's default since `3422feb`).

**Bot won 4 of 9.**

| # | game | first | winner | actions | finished (UTC) |
|---|---|---|---|---|---|
| 1 | `1276635885090433092-30b2a747` | bot | **bot** | 81 | 2026-10-02T18:22:45Z |
| 2 | `8586214166120924142-30b2a747` | owner | owner | 99 | 2026-10-02T18:53:45Z |
| 3 | `14129377960470028903-30b2a747` | owner | owner | 91 | 2026-10-02T19:06:13Z |
| 4 | `3893083728998910574-30b2a747` | bot | owner | 98 | 2026-10-02T19:12:53Z |
| 5 | `9617798903006090424-30b2a747` | owner | owner | 107 | 2026-10-02T19:20:43Z |
| 6 | `861286742315684199-30b2a747` | bot | **bot** | 71 | 2026-10-02T19:23:49Z |
| 7 | `8801639540343240212-30b2a747` | owner | **bot** | 87 | 2026-10-02T19:29:35Z |
| 8 | `8801639540343240213-30b2a747` | owner | **bot** | 103 | 2026-10-02T19:43:25Z |
| 9 | `3640492624637022863-30b2a747` | bot | owner | 89 | 2026-10-02T19:51:59Z |

`games/<game_id>.json` are the raw serve.py captures, byte for byte (`actions` index 0 and 1 are the mulligans; no
`reseed` steps).

**`undo/8801639540343240212-30b2a747-2.json` is not a game.** After losing game 7 (`8801639540343240212-30b2a747`), the owner pressed
Ctrl+Z in the UI to see whether he could have played better: this record shares game 7's first 45 actions, then
diverges at action index 45 — the owner (side `a`) evolves (`{"evolve": {"player": "a", "slot": 0, "super": false}}`)
where game 7 has him play card `90071130` from hand position 2 — and continues to 57 actions, unfinished (`final`
false). Kept as the owner's alternative line. Afterwards a restart on the same seed was cancelled and the page reloaded;
game 8 uses that seed + 1 (`8801639540343240213`), an unrelated deal.

Context: all earlier reviews were Abyss Midrange mirrors (review7: the same bot spec on d5f4310, bot 2/8). Net8 data
generation was running on the box during these games (node-limited search: the bot's choices are unaffected, only its
thinking time).
