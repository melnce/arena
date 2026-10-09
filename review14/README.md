# review14 — 4 owner games vs the served bot: the owner's Rune Test Subject against the bot's Haven Evo (2026-10-08/09)

- Engine **`ed574a4`** (v4 default leaf), the same build and spec as review13.
- Every capture's `policy` and `strong` is `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1`,
  serve.py's default.
- serve.py was started by the runner session at 23:39 UTC on 2026-10-08 and stopped after the games. Its log is
  `results\serve-review14.log` on the box.
- The owner is on side `a`, against the fair bot only, playing **Rune Test Subject** (`meta-rune-test-subject`). The bot
  plays **Haven Evo** (`meta-haven-evo`). All four games are that pairing, not a mirror (game ids `…-d90f8eb2`).
- On the box at the same time, sweep 45b's final was running. The search is node-limited, so this changed thinking time
  only, not moves.

**Bot won 1 of 4** (game 2). The owner flagged nothing in particular this time. The held-back scan finds more moments
per game than any earlier review with a scan (review9 on; below).

| # | game | first | winner | actions | finished (UTC) | bot think: median / max |
|---|---|---|---|---|---|---|
| 1 | `6598261483642061665-d90f8eb2` | bot | owner | 104 | 2026-10-08T23:46:48Z | 0.5 s / 12.9 s |
| 2 | `10916045087454016876-d90f8eb2` | bot | **bot** | 86 | 2026-10-08T23:58:34Z | 1.1 s / 77.4 s |
| 3 | `12644260486301391828-d90f8eb2` | bot | owner | 68 | 2026-10-09T00:03:23Z | 1.2 s / 16.7 s |
| 4 | `14600367900189587136-d90f8eb2` | owner | owner | 86 | 2026-10-09T00:11:37Z | 0.8 s / 42.1 s |

`games/<game_id>.json`: the raw serve.py captures, byte for byte.

- All 4 are `final`, `humanSide` `a`, engine `ed574a4`, `first` `coin`, no `reseed`.
- Each replays to a terminal state with the recorded winner.
- "first" is the player to act after the mulligans.
- Think times come from serve.py's log, one line per bot request. Each game's line count equals its bot actions in the
  capture: 159 in all.

## Exact replay of the served decisions

This is review13's method. The browser sends `botSeed` = the game's seed + `botSeq` (`ui/src/session.ts`,
`nextBotSeed`). So `bot_action_value(spec, seed + k)`, where k counts the bot's earlier actions in the capture, reproduces
each served decision. `scripts/s13_replay.py` → `replay.txt`:

- **All 159 bot decisions replay exactly.** 133 of 133 with a recorded `bot_value` give the same action and value
  (to 1e-6), and 26 of 26 without one give the same action.
- This confirms the seed formula. It does not by itself rule out rewinds: undo winds `botSeq` back.

## Restarts: none

- The owner reported no redone game.
- serve.py's log agrees.
  - Each game's bot-request count equals its bot actions.
  - Turns never go backwards within a `game_id`, and every hash check is `ok`.
  - A rewind across a bot turn would have added request lines.
- All 4 seeds differ and all 4 captures are `final`.

## Preview, not a deciding measurement: the held-back scan

`holdback_audit.py scan` (holdback1's script, unchanged) ran on these 4 captures, bot seat only, on the `ed574a4`
bindings: `holdback_scan_preview.jsonl`.

- It finds **5 HB moments: 1.25 per game, 0.147 per End Turn.** That is the most per game of any review with a scan
  (review9 on): review12 had 0.83 and review13 0.33.
- All five are free kills: X survives the attack.
- In every case X died on the owner's next turn, and the owner had a sure kill line on it (the line kills
  under 4 of 4 rerolls).
- The bot lost all three games that hold a moment.
- The script's printed `VERDICT` line is holdback1's rule and does not apply here.

Each moment explained at the exact served seed (`scripts/s14_hb.py` → `holdback_explain.txt`). The attack values are
for X attacking Y, the trade the scan flags, unless noted.

| game # | ply | turn | X (bot) → Y (owner) | at the served seed | `info=all`, served seed | seeds 1–8 |
|---|---|---|---|---|---|---|
| 3 | 15 | 4 | Sofina, Inspiring Strength 4/4 (evolved, Ward) → Obsessed Test Subject 2/2 | End Turn 36.16 vs attack 25.15 (best attack: 34.52, on the owner's Enamored Researcher 1/1) | End Turn 43.73 vs attack 16.49 (on the Researcher: 43.74) | End Turn 6, attack on a Test Subject 1, on the Researcher 1 |
| 3 | 40 | 6 | Sofina 4/4 → Tico, Mysterian Spellcrafter 3/2 | End Turn −20.82 vs attack −24.15 | attack −18.60 vs End Turn −28.59 | attack 6, End Turn 2 |
| 4 | 69 | 8 | Erralde, Signet Convict 8/8 (evolved) → Sweet Abomination 5/1 | End Turn −39.83 vs attack −41.60 | attack −46.31 vs End Turn −47.35 | attack 5, End Turn 3 |
| 1 | 27 | 4 | Sofina 4/4 → Obsessed Test Subject 2/2 | End Turn 45.85 vs attack 44.28 | attack 41.73 vs End Turn 41.58 | attack 8 of 8 |
| 1 | 52 | 6 | Zoe, Dazzling Hope 4/6 (evolved) → Clay Golem 2/2 | End Turn 9.19 vs attack −3.46 | End Turn −3.26 vs attack −6.38 | End Turn 6, attack 2 |

- **Three moments were narrow calls at the served seed:** End Turn by 1.6–3.3 points, where full information and most
  other seeds choose the attack (game 3 ply 40, game 4 ply 69, game 1 ply 27).
  - Game 1, ply 27, is the most consistent across seeds: 8 of 8 attack. Full information is nearly even there
    (41.73 vs 41.58).
- **Game 3, ply 15, clearly rejects the flagged trade.**
  - End Turn beats the attack on the Test Subject by 11.0 points at the served seed and by 27.2 with full information.
  - 1 of 8 seeds takes that trade.
  - The near-tie in that row is an attack on the owner's Enamored Researcher 1/1, which Sofina's end-of-turn -1/-1
    kills anyway on End Turn.
- **The Zoe moment (game 1, ply 52) is a consistent choice:** End Turn wins by 12.6 points at the served seed, also with
  full information, and at 6 of 8 seeds.
- **The served held-back check (`hbcheck=2000`) ran at all five moments and kept End Turn each time.**
  - Explain's `holdback` record has its own End′ above every kill attack's A′: by 2.4–3.4 points at the four Sofina and
    Erralde moments (3.2 at game 3 ply 15, against the best kill there), and by 17.9 at the Zoe moment.
  - So no `holdback_trade` override fired (every explain reports `path search`).
- **Three of the five are Sofina, Inspiring Strength:** a 4/4 Ward follower once evolved, *"At the end of your turn, if
  this follower is evolved, give all other followers on the field -1/-1."*
  - At those End Turns the effect does not kill the Test Subject or Tico that Sofina could have attacked.
  - At game 3 ply 15 it does kill the owner's Enamored Researcher 1/1.
  - The owner killed Sofina on the next turn in all three cases.

This is a 4-game preview, not a measurement.

Bot record against the owner (recent reviews):

| review | bot | bot record |
|---|---|---|
| review12 | served + `fuseguard=1` (v3 leaf), Rune Test Subject mirror | 2/6 |
| review13 | same, v4 leaf (`ed574a4`); the bot's Abyss Midrange against the owner's Rune Test Subject | 1/6 |
| review14 | same; the bot's Haven Evo against the owner's Rune Test Subject | 1/4 |

## Files

- `games/`: the four captures.
- `replay.txt`: the exact replay of every bot decision.
- `holdback_scan_preview.jsonl`: the held-back scan.
- `holdback_explain.txt`: the five moments explained at the served seed.
- `scripts/s13_replay.py` and `scripts/s14_hb.py`: the scripts behind them.

Run the scripts from the repo root (they load `cards/`).

- `s13_replay.py` takes the capture paths as arguments: `python s13_replay.py games/<game_id>.json ...`.
- `s14_hb.py` takes `holdback_scan_preview.jsonl` as its argument and reads `results/games/<game_id>.json`, so copy the
  captures there first.
