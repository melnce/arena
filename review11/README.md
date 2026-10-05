# review11 — 13 owner games vs the served bot with tkill, tkroll and the held-back check (2026-10-04)

- Engine **`1c551eb`** (WS race block; v3 default leaf, no race block), the same build as sweeps 37–39.
- Every capture's `policy` and `strong` is `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000`. That is
  the served default plus `hbcheck=2000`, the same spec as review10.
- serve.py was started outside the runner session for these games, so the box has no server log for them (see
  "Restarts" below).
- The owner is on side `a`, against the fair bot only (no cheater games). There are two batches:
  - games 1–7: the **Rune Test Subject** mirror (`meta-rune-test-subject`, game ids `…-104cf827`);
  - games 8–13: the **Abyss Midrange** mirror (`meta-abyss-midrange`, `…-5ce21003`).
- On the box at the same time:
  - batch 1 (games finished 11:25–13:06 UTC) overlapped the end of sweep 39's main arm (until 11:42 UTC) and its
    reverse arm;
  - batch 2 (games finished 21:40–22:14 UTC) overlapped sweep 39b's main arm (20:21–04:34 UTC).
  - No tp stage overlapped either batch.
  - The search is node-limited, so this changed thinking time only, not moves.

**Bot won 3 of 13:** 1 of 7 in the Rune Test Subject mirror, 2 of 6 in the Abyss Midrange mirror.

| # | game | deck (mirror) | first | winner | actions | finished (UTC) |
|---|---|---|---|---|---|---|
| 1 | `7260600291112487607-104cf827` | Rune Test Subject | bot | **bot** | 115 | 2026-10-04T11:25:48Z |
| 2 | `15446949829857486569-104cf827` | Rune Test Subject | bot | owner | 96 | 2026-10-04T11:37:51Z |
| 3 | `5935752092966957148-104cf827` | Rune Test Subject | bot | owner | 103 | 2026-10-04T11:49:18Z |
| 4 | `2044796132966629961-104cf827` | Rune Test Subject | owner | owner | 93 | 2026-10-04T11:56:52Z |
| 5 | `5368404598479714909-104cf827` | Rune Test Subject | bot | owner | 62 | 2026-10-04T12:03:53Z |
| 6 | `2164703529906618017-104cf827` | Rune Test Subject | bot | owner | 113 | 2026-10-04T12:48:50Z |
| 7 | `3505567890135518157-104cf827` | Rune Test Subject | bot | owner | 93 | 2026-10-04T13:05:57Z |
| 8 | `8044316535444780097-5ce21003` | Abyss Midrange | bot | owner | 86 | 2026-10-04T21:40:01Z |
| 9 | `9914891805638046376-5ce21003` | Abyss Midrange | owner | owner | 91 | 2026-10-04T21:45:25Z |
| 10 | `14423057095953136733-5ce21003` | Abyss Midrange | bot | **bot** | 59 | 2026-10-04T21:48:10Z |
| 11 | `731162716593271772-5ce21003` | Abyss Midrange | bot | owner | 115 | 2026-10-04T22:01:29Z |
| 12 | `10897487595688697553-5ce21003` | Abyss Midrange | owner | **bot** | 64 | 2026-10-04T22:07:42Z |
| 13 | `5120746196322041219-5ce21003` | Abyss Midrange | owner | owner | 73 | 2026-10-04T22:14:26Z |

`games/<game_id>.json`: the raw serve.py captures, byte for byte. All 13 are `final`, `humanSide` `a`, engine
`1c551eb`, `first` `coin`, no `reseed`. Each replays on the `1c551eb` bindings to a terminal state with the recorded
winner. "first" is the player to act after the mulligans, the same method as review10, which reproduces review9's
column.

## Restarts: none (the owner's word; the captures agree)

**The owner confirms that none of these 13 games was redone.**

review10 found a restarted game through serve.py's log, where the turn number went backwards inside one `game_id`.
These games have no such log on the box, so the captures are the only check.

- **Nothing in the captures points to a restart.**
  - All 13 seeds differ, so these are 13 different deals.
  - No two captures share more than their first action (13 pairs open with an identical mulligan record).
  - No `-2.json` rollover exists.
- **One kind of restart would still be invisible.** A restart reuses the `game_id`, and serve.py overwrites the
  capture when the incoming log is at least as long.
  - If the replayed game ran at least as long as the abandoned attempt, the abandoned line would leave no trace.
  - A shorter replay would have left a non-final capture. All 13 are final.
- **One gap is longer than the rest.** Game 6 finished 45 min after game 5. Within each batch, the other consecutive
  games finished 3–17 min apart. The batches themselves are 8½ h apart.
  - The owner explains it: in the Rune Test Subject games the bot sometimes thought for a very long time. The owner
    turned to a phone meanwhile, and once forgot for a while that the game was still running.
  - The box cannot measure those think times. The captures carry no timing, and there is no server log.

## Preview, not a deciding measurement: the held-back scan

`holdback_audit.py scan` (holdback1's script, unchanged) on these 13 captures, on this box's `1c551eb` bindings, bot seat
only: `holdback_scan_preview.jsonl`.

- It finds **6 HB moments** (0.46 per game, 0.056 per End Turn). review10 had 4 (0.33 per game) and review9 had 5
  (0.42 per game).
- The script's printed `VERDICT` line is holdback1's rule and does not apply here.

| game # | ply | turn | primary trade X -> Y (card ids) | X next turn | can the owner kill X |
|---|---|---|---|---|---|
| 2 | 53 | 6 | 10931110 -> 10931110 (not a free kill) | died | kill (sure 4/4) |
| 3 | 40 | 5 | 10931110 -> 10932110 | died | kill (sure 2/4) |
| 3 | 96 | 9 | 10931110 -> 10931110 | game over | kill (sure 4/4) |
| 5 | 28 | 5 | 10833110 (evolved) -> 10931110 | died | kill (sure 4/4) |
| 9 | 66 | 7 | 10954120 (evolved) -> 90051110 | died | kill (sure 4/4) |
| 11 | 86 | 9 | 90051140 -> 10951120 | died | kill (sure 4/4) |

Report totals:

| measure | value |
|---|---|
| W (wasted) | 0.6 (n = 5) |
| D (X died next turn) | 1.0 (n = 5) |
| K (the owner had a sure kill line on X, 4/4 reseeds) | 0.833 (n = 6; game 3 at ply 40 was 2/4) |
| primary trades that are free kills | 5 of 6 |
| the HB side's other followers that died next turn | 6/9 |

The bot lost all five games that hold a moment. Four of the six moments are in the Rune Test Subject mirror.

**Owner's notes on clearly bad trades:** none given.

Bot record against the owner in mirrors:

| review | bot | bot record |
|---|---|---|
| review3 / 4 | fair | 1/12 |
| review6 | fair | 3/7 |
| review7 | 8-world, Abyss | 2/8 |
| review8 | Portal AF | 4/9 |
| review9 | with hbcheck | 5/12 |
| review10 | with tkill, tkroll and hbcheck, Abyss | 7/12 |
| review11 | same spec | 3/13 (Rune Test Subject 1/7, Abyss 2/6) |
