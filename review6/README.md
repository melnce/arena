# review6 — 14 owner games vs the fair and the cheater bot (2026-10-01)

Abyss Midrange mirror only (`meta-abyss-midrange` both sides), the owner on side `a`, played alternately fair /
cheater through `py/serve.py` on engine **`a69b248`** (WK merged; `okill` / `omacro` off by default; WJ: one
transposition table per root under `info=all`, so the cheater averages four real worlds). First player by coin.
The served specs:

- fair: `h0:nodes=16000,horizon=3`
- cheater: `h0:nodes=16000,horizon=3,info=all` (sees the owner's hand and both decks' contents, not the draw order)

| bot | games | bot won |
|---|---|---|
| fair | 7 | 3 |
| cheater | 7 | 1 |

| # | game | bot | spec | first | winner | actions | finished (UTC) |
|---|---|---|---|---|---|---|---|
| 1 | `8635128015626262023-5ce21003` | fair | `h0:nodes=16000,horizon=3` | owner | **bot** | 92 | 2026-10-01T18:23:52Z |
| 2 | `12232547989608083664-5ce21003` | cheater | `h0:nodes=16000,horizon=3,info=all` | bot | owner | 115 | 2026-10-01T18:30:23Z |
| 3 | `862378526299154215-5ce21003` | fair | `h0:nodes=16000,horizon=3` | owner | **bot** | 88 | 2026-10-01T18:35:31Z |
| 4 | `13687886034282483276-5ce21003` | cheater | `h0:nodes=16000,horizon=3,info=all` | bot | owner | 65 | 2026-10-01T18:39:21Z |
| 5 | `3553353363339288488-5ce21003` | fair | `h0:nodes=16000,horizon=3` | bot | owner | 81 | 2026-10-01T18:43:03Z |
| 6 | `4708361415292007057-5ce21003` | cheater | `h0:nodes=16000,horizon=3,info=all` | bot | owner | 83 | 2026-10-01T18:46:39Z |
| 7 | `8003226776130031633-5ce21003` | fair | `h0:nodes=16000,horizon=3` | owner | owner | 98 | 2026-10-01T18:51:27Z |
| 8 | `6994896475300051650-5ce21003` | cheater | `h0:nodes=16000,horizon=3,info=all` | owner | owner | 57 | 2026-10-01T18:55:09Z |
| 9 | `2660760995577294294-5ce21003` | fair | `h0:nodes=16000,horizon=3` | owner | owner | 87 | 2026-10-01T18:58:59Z |
| 10 | `9027709759949265190-5ce21003` | cheater | `h0:nodes=16000,horizon=3,info=all` | owner | owner | 86 | 2026-10-01T19:03:48Z |
| 11 | `612169728001713502-5ce21003` | fair | `h0:nodes=16000,horizon=3` | bot | owner | 63 | 2026-10-01T19:06:27Z |
| 12 | `8358890658307947999-5ce21003` | cheater | `h0:nodes=16000,horizon=3,info=all` | owner | **bot** | 56 | 2026-10-01T19:08:38Z |
| 13 | `17329012734403277217-5ce21003` | fair | `h0:nodes=16000,horizon=3` | owner | **bot** | 113 | 2026-10-01T19:14:11Z |
| 14 | `5732955890759623403-5ce21003` | cheater | `h0:nodes=16000,horizon=3,info=all` | bot | owner | 79 | 2026-10-01T19:18:54Z |

`games/<game_id>.json` are the raw serve.py captures, byte for byte (`actions` index 0 and 1 are the mulligans; no
`reseed` steps). Context: review3/4 mirror games vs fair bots 1/12; review5 (the one-world cheater on 2d12e25) 8/21;
sweep 29 (bot vs bot, this mirror, this budget): the one-world cheater 0.448, the four-world cheater 0.478 vs the fair
bot. Sweep 31 was running on the same box during these games (node-limited search: the bot's choices are unaffected,
only its thinking time).
