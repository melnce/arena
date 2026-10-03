# review10 — 12 owner games vs the served bot with tkill, tkroll and the held-back check (2026-10-03)

- Engine **`1c551eb`** (WS: optional race block for linear leaves; v3 default leaf, which has no race block, so the
  served bot plays as on `33de0a9`). Rebuilt 2026-10-03 19:43 local; `pytest py/tests -q` on this build: 161 passed,
  6 skipped, 1 xfailed (`test_race.py` 7 passed).
- Served with `py\serve.py --strong "h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000"`; serve.py printed
  `strong:   h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000` (health check: version 1c551eb).
  That is the served default plus `hbcheck=2000`. Every capture's `policy` and `strong` carry that spec.
- Abyss Midrange mirror, the owner on side `a`, the fair bot only (no cheater games).
- On the box at the same time: pytest (until 17:52 UTC, game 1), the four sweep 37 leaf trainings (17:53–18:04 UTC,
  games 1–3) and sweep 37 (from 18:04 UTC, games 4–12). The search is node-limited, so this changed thinking time
  only, not moves.

**Bot won 7 of 12.**

| # | game | first | winner | actions | finished (UTC) |
|---|---|---|---|---|---|
| 1 | `1278821193826391812-5ce21003` | owner | **bot** | 107 | 2026-10-03T17:55:26Z |
| 2 | `9965237821980154247-5ce21003` | bot | **bot** | 97 | 2026-10-03T18:01:01Z |
| 3 | `7554964078655355856-5ce21003` | bot | **bot** | 70 | 2026-10-03T18:04:29Z |
| 4 | `15637789661517106482-5ce21003` | bot | owner | 80 | 2026-10-03T18:11:49Z |
| 5 | `11739911988108632947-5ce21003` | owner | owner | 53 | 2026-10-03T18:16:09Z |
| 6 | `12807105805322567332-5ce21003` | bot | **bot** | 102 | 2026-10-03T18:21:39Z |
| 7 | `2591565314664888832-5ce21003` | bot | owner | 76 | 2026-10-03T18:27:57Z |
| 8 | `1934873130615857609-5ce21003` | bot | **bot** | 121 | 2026-10-03T18:38:19Z |
| 9 | `234541894358375963-5ce21003` | bot | **bot** | 91 | 2026-10-03T18:46:30Z |
| 10 | `3435803039681778271-5ce21003` | bot | owner | 104 | 2026-10-03T18:52:06Z |
| 11 | `954632202902876694-5ce21003` | bot | owner | 78 | 2026-10-03T18:56:16Z |
| 12 | `14239169779918067249-5ce21003` | owner | **bot** | 90 | 2026-10-03T19:11:46Z |

`games/<game_id>.json`: the raw serve.py captures, byte for byte. All 12 are `final`, `humanSide` `a`, engine
`1c551eb`, no `reseed`. Each replays on the `1c551eb` bindings to a terminal state with the recorded winner. "first" is
the player to act after the mulligans; the same method reproduces review9's column.

## Game 12 was restarted once, and only the replayed line is captured

The owner made a mistake in game 12 and restarted it. He replayed his own moves exactly as before up to that point and
played on from there.

- **Same game, same id.** The restart reused the seed, decks and `game_id`. serve.py overwrites a non-final capture with
  a longer log (it rolls over to `-2.json` only after a `final` capture). So the abandoned line was overwritten and is
  not kept, and no `undo/` folder exists (unlike review8).
- **The server log shows the restart.**
  - Game 12 has 68 bot calls. The first 29 run from turn 0 to turn 7, then the turn number drops back to 1.
  - Two connection-aborted errors fall at that point: bot replies the browser dropped when the line was abandoned.
  - No other game has a rewind.
- **The two lines match up to turn 7.** They make the same number of bot calls per turn through turn 6 (turn 4: 5/5,
  turn 5: 5/5, turn 6: 6/6) and on the first three calls of turn 7. Turn 6's thinking times are close (24.0 / 30.0 /
  24.1 / 12.1 s, then 23.4 / 28.7 / 24.7 / 13.3 s). The restart came during turn 7.
- **The box cannot prove the bot's replayed moves were identical.** The site sends each bot request's seed
  (`botSeed`), and the abandoned actions are gone. The call pattern is consistent with the same line.
- **Game 12 counts as one game**, the captured line, which the bot won.

## Preview, not a deciding measurement: the held-back scan

`holdback_audit.py scan` (holdback1's script, unchanged) on these 12 captures, on this box's `1c551eb` bindings, bot seat
only: `holdback_scan_preview.jsonl`.

- It finds **4 HB moments** (0.33 per game, 0.038 per End Turn). review9 had 5 (0.42 per game).
- The script's printed `VERDICT` line is holdback1's rule and does not apply here.
- Two of the four are in game 12 at turns 5 and 6, inside the prefix that was played twice.

| game # | ply | turn | primary trade X -> Y (card ids) | X next turn | can the owner kill X |
|---|---|---|---|---|---|
| 2 | 28 | 5 | 10752110 (evolved) -> 90051110 | died | kill (sure 4/4) |
| 9 | 80 | 9 | 90051140 -> 90051110 | survived | kill (sure 4/4) |
| 12 | 37 | 5 | 10752110 (evolved) -> 90051110 | died | kill (sure 4/4) |
| 12 | 49 | 6 | 10754110 (evolved) -> 10452130 | died | kill (sure 1/4) |

Report totals: W (wasted) 0.5, D (X died next turn) 0.75, K (owner could kill X) 0.75, n = 4. All four primary trades
are free kills (X survives the attack). The bot won all three games that hold a moment.

**Owner's notes on clearly bad trades:** none given while playing.

Bot record against the owner in mirrors:

| review | bot | bot record |
|---|---|---|
| review3 / 4 | fair | 1/12 |
| review6 | fair | 3/7 |
| review7 | 8-world, Abyss | 2/8 |
| review8 | Portal AF | 4/9 |
| review9 | with hbcheck | 5/12 |
| review10 | with tkill, tkroll and hbcheck | 7/12 |
