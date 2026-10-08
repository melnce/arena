# review13 — 6 owner games vs the served bot: the owner's Rune Test Subject against the bot's Abyss Midrange (2026-10-08)

- Engine **`ed574a4`**: the v4 default leaf (`h0-linear-v4`), the build since runbook 44's pull. These are the owner's
  first games against the v4 bot.
- Every capture's `policy` and `strong` is `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1`.
  That is serve.py's default, and it serves the v4 leaf on this build.
- serve.py was started by the runner session at 17:58 UTC and stopped after the games. Its log is
  `results\serve-review13.log` on the box.
- The owner is on side `a`, against the fair bot only (no cheater games).
  - The owner played **Rune Test Subject** (`meta-rune-test-subject`); the bot played **Abyss Midrange**
    (`meta-abyss-midrange`).
  - All six games are that pairing, not a mirror (game ids `…-b4973563`).
- On the box at the same time, sweep 45b's screen was running (from 15:51 UTC). The search is node-limited, so this
  changed thinking time only, not moves.

**Bot won 1 of 6** (game 6). The owner flagged game 1, turn 6, where the bot left the owner's 2/1 Test Subject alive
although it had a better line that killed it. That analysis comes first, below the table.

| # | game | first | winner | actions | finished (UTC) | bot think: median / max |
|---|---|---|---|---|---|---|
| 1 | `4984932781931433298-b4973563` | bot | owner | 63 | 2026-10-08T18:05:23Z | 0.6 s / 19.8 s |
| 2 | `5136633045010458541-b4973563` | owner | owner | 103 | 2026-10-08T18:16:02Z | 1.0 s / 59.1 s |
| 3 | `14040273279660052122-b4973563` | owner | owner | 91 | 2026-10-08T18:23:03Z | 0.8 s / 31.1 s |
| 4 | `6154771475660196378-b4973563` | owner | owner | 105 | 2026-10-08T18:34:32Z | 1.0 s / 42.6 s |
| 5 | `13222403987759895244-b4973563` | bot | owner | 95 | 2026-10-08T18:49:24Z | 2.0 s / 85.4 s |
| 6 | `7035443222918452723-b4973563` | bot | **bot** | 83 | 2026-10-08T18:54:41Z | 0.6 s / 15.6 s |

`games/<game_id>.json`: the raw serve.py captures, byte for byte.

- All 6 are `final`, `humanSide` `a`, engine `ed574a4`, `first` `coin`, no `reseed`.
- Each replays to a terminal state with the recorded winner.
- "first" is the player to act after the mulligans, the same method as review10–12.
- Think times come from serve.py's log, one line per bot request. Each game's line count equals its bot actions in the
  capture: 217 in all.

## New: the served bot's decisions can be replayed exactly

The browser sends `botSeed` = the game's seed + `botSeq` (`ui/src/session.ts`, `nextBotSeed`:
`s.cfg.seed + BigInt(s.botSeq)`). `botSeq` is the number of bot requests on the current line of play; undo and
checkpoint restore wind it back. So `bot_action_value(spec, seed + k)` reproduces each served decision, where k counts
the bot's earlier actions in the capture, its mulligan included.

**All 217 bot decisions of the six games replay exactly** (`replay.txt`):

- 171 of 171 decisions with a recorded `bot_value` give the same action and the same value (to 1e-6).
- 46 of 46 without one give the same action: 6 mulligans, 37 single-option End Turns, 2 single-option choices, and 1
  End Turn whose only alternative was `bonus_pp`.
- The tests ran on the box's current `ed574a4` module, the same one serve.py loaded.

What this means:

- **Only review12 explained decisions, and it did so at seeds 1–3,** which merely stood in for the served seed. From
  here on, the served decision itself can be explained.
- **The exact match confirms the seed formula** and that serve.py ran this build.
- **It does not rule out rewinds by itself.** Undo and checkpoint restore wind `botSeq` back, so after a rewind the later
  seeds still match.
- **Rewinds are checked in "Restarts" below.** A rewind that crossed a bot turn would add request lines to serve.py's
  log, and the log shows none. A rewind within the owner's own turn leaves no trace anywhere.
- **`game1/s13_replay.py` runs this check for any capture.**

## Game 1 (owner's note): turn 6 left the owner's 2/1 Test Subject alive, and so did turn 5

The owner's note: on turn 6 the bot evolved Highwire Feline and dealt 3 damage to a Test Subject, leaving it at 1.
Then it attacked that same Test Subject with the 6/7 Feline, which would have killed it anyway. Meanwhile it left the
owner's 2/1 Test Subject alive, which it could have killed instead.

**Verdict: confirmed.** That turn the bot had a line that was better than the one it played in every respect that
matters in this game, and it did not take it. The same 2/1 Test Subject also survived a free kill on turn 5. It hit the
bot's face for 2 on each of the owner's turns 5 and 6, and the owner's turn-7 kill needed both hits.

### Turn 6: the position and the bot's line

At the start of the bot's turn 6 (after 35 actions):

| | owner (`a`) | bot (`b`) |
|---|---|---|
| leader | 18/20 | 17/20, 6 PP |
| field | Obsessed Test Subject 2/1 (attacked the bot's face last turn); Test Subject 6/4; evolved Test Subject 7/1 | Skeleton 1/1 |
| hand | Enamored Researcher, Sweet Abomination, Tetra & Ladica ×2, Test Subject | Macmillan ×2, Itsurugi & Taketsumi, Istyndet vs. Mitilykket, Highwire Feline, Baal |

Highwire Feline, 5 PP, 4/5: *"Fanfare: Select an enemy follower on the field and deal it 3 damage. Summon a Skeleton.
Evolve: Replicate the effects of this card's Fanfare ability."*

The bot's line (actions 35–41):

1. It played Highwire Feline. The fanfare's 3 damage killed the evolved 7/1, and a Skeleton came in.
2. It evolved the Feline to 6/7. The evolve's 3 damage hit the **6/4** (now 6/1), and a second Skeleton came in.
3. The Feline attacked the 6/1 and killed it; the Feline went to 6/1.
4. The ready Skeleton hit the owner's face (18 → 17).
5. End Turn. **The owner keeps the 2/1.**

**The better line** (`s13_alt.py` → `alt.txt`): send the evolve's 3 damage to the **2/1**, which dies. Then the 6/7 Feline
attacks the 6/4, which dies, and the Feline goes to 6/1 just as in the actual line. The Skeleton goes face as played.
Both lines are legal, and the replay checks them.

- **Resulting boards:** identical (bot: Skeleton, Feline 6/1, Skeleton, Skeleton; both leaders at 17), except the owner
  has **no** follower instead of the 2/1.
- **Full game states:** they differ only in that, plus where the 2/1 ends up. In the alternative it is in the owner's
  cemetery: the owner's shadows go 9 → 10, and `destroyed_history` gets one more entry.
- **Who reads that difference:** one card does. The owner's Wills United (mode 2, "Reanimate (2)") picks from the
  owner's destroyed history.
  - Every cost-2 follower there is already an Obsessed Test Subject in both lines, so it would summon the same card.
    Only the random tie pick could differ.
  - The owner played Wills United on turn 7, in its other mode.
  - No other card in either deck reads the owner's shadows, cemetery or destroyed history. The bot's Necromancy and
    Reanimate use its own.
- **Other ways to clear the board, short of the better line:** the 6/4 target did not have to leave the 2/1 alive.
  - After it, Skeleton → the 6/1 and then Feline → the 2/1 also empties the owner's board, ending with the bot on
    Feline 6/5 + 2 Skeletons and the owner at 18. That is a trade-off against the better line (Feline 6/1 + 3
    Skeletons, owner at 17), not dominated by it.
  - And at the very end the Skeleton could have traded into the 2/1 instead of going face.
  - The bot took none of these.

### Turn 5: the same 2/1 survived a free kill

- At the bot's turn-5 End Turn (after 27 actions), it had an evolved 6/7 Highwire Feline that could still attack.
  The owner had this same Test Subject on board as a 2/1 (instance id 84).
- Attacking it would have killed it with the Feline surviving. The bot ended its turn instead.
- The held-back scan flags this End Turn (game 1, ply 27, a free kill). The owner then killed the Feline with one attack:
  its evolved 7/7 Test Subject (action 31).

### What the 2/1 did

The 2/1 hit the bot's face on each of the owner's next two turns:

| owner's turn | bot's leader |
|---|---|
| 5 | 19 → 17 |
| 6 | 16 → 14 |
| 7 | Tetra & Ladica 9/8 and a Test Subject 6/2: 14 → 5 → **0** |

- **The turn-7 kill had 1 point to spare:** 9 + 6 into 14.
- **Without the 2/1's 4 damage,** the same attacks leave the bot at 3; without either one of its hits, at 1.
- **Not computed:** whether the owner had another winning route without the 2/1.

### What the bot's values say: at the served seeds

Each served decision is replayed exactly (`explain.txt`). `seeds.txt` and `deal.txt` hold the choice counts over other
seeds.

| decision | at the served seed (as played) | other seeds | `info=all`, served seed |
|---|---|---|---|
| turn 5 End Turn (after 27) | **End Turn −2.54** vs Feline → the 2/1 −4.58 | Feline → the 2/1 at **32 / 32** of seeds 1–32 | Feline → the 2/1 1.28 vs End Turn −9.26 |
| turn 6 evolve target (after 38) | **the 6/4 −16.07** vs the 2/1 −20.95 | the 2/1 17, the 6/4 15 of seeds 1–32 | the 2/1 −2.28 vs the 6/4 −3.09 |
| turn 6 attack (after 39) | **Feline → the 6/1 −10.78**; Skeleton face −11.69; Skeleton → the 6/1 −19.44; Feline → the 2/1 −20.01 | — | Skeleton face −11.60 first |
| turn 6 Skeleton (after 40) | **face −15.25**; End Turn −17.54; Skeleton → the 2/1 −22.24 | face 24, the 2/1 8 of seeds 1–32 | face −16.33 vs the 2/1 −24.45 |

The k sampled worlds of a decision are shared by all its root candidates, in both `deal` modes, so each row is a paired
comparison.

- **Turn 5: the served seed's sampled worlds were the exception.**
  - There End Turn edged out the free kill by 2 points.
  - All 32 other seeds kill the 2/1. So does the served seed with full information, by 10.5 points.
  - So does the served seed under `deal=block`, which deals the opponent's unknown hand to the k worlds from one shared
    shuffle, without replacement across worlds. The 16 other seeds tried under `deal=block` also kill it.
- **Turn 6: the evolve target splits 17 / 15 over 32 seeds.**
  - `deal=block` moves it to the 6/4 at 11 of 16 seeds and at the served seed.
  - The 2/1 target leads to the better line above. But the two targets also differ in the 6/4 vs the 6/1 and in their
    follow-ups, so the target choice itself is not a dominance test.
  - Why the bot finds it this close is not established here. `after.txt` compares the next decision after either target;
    those positions differ in more than the 2/1.
- **A free removal of the 2/1 is worth only a few points to the bot in the turn-6 position** (`s13_endturn.py` →
  `endturn.txt`).
  - The test takes the two positions just before the Skeleton goes face. Their results differ only in the owner's 2/1
    and where it ends up.
  - The root value of "Skeleton face" (then End Turn, a forced move) is higher without the 2/1:
    - by +1.65 on average with full information, 8 / 8 seeds, from +0.33 to +3.20;
    - by +4.05 on average with the served information, 7 / 8 seeds, from −2.65 to +10.78.
  - In the same position the Skeleton prefers face over trading into the 2/1, even with full information (face −16.33
    vs −24.45). The served spec trades at 8 of 32 seeds.
  - Turn 5 shows the opposite: with full information the free kill of the same 2/1 was worth 10.5 points over End Turn.

This is one game, not a measurement. In this position the bot valued removing a 2/1 rush follower at only a few points,
and it passed up three ways to clear the board on turn 6. On turn 5, its sampled worlds at the served seed hid a free kill
that every other seed and full information found.

## Restarts: none

- The owner reported no redone game.
- serve.py's log agrees: each game's bot-request count equals its bot actions in the capture (217 in all), turns never go
  backwards within a `game_id`, and every hash check is `ok`. A rewind across a bot turn would have added request lines.
- The exact replay above is consistent with this. It cannot detect a rewind on its own, and a rewind within the owner's
  own turn leaves no trace anywhere.
- All 6 seeds differ, all 6 captures are `final`, and there is no `-2.json` rollover.

## Preview, not a deciding measurement: the held-back scan

`holdback_audit.py scan` (holdback1's script, unchanged) ran on these 6 captures, bot seat only, on the `ed574a4`
bindings: `holdback_scan_preview.jsonl`.

- It finds **2 HB moments**: 0.33 per game, 0.039 per End Turn. review12 had 0.83 per game and review11 0.46.
- The script's printed `VERDICT` line is holdback1's rule and does not apply here.

| game # | ply | turn | primary trade X -> Y (card ids) | X next turn | can the owner kill X |
|---|---|---|---|---|---|
| 1 | 27 | 5 | 10752110 (Highwire Feline, evolved 6/7) -> 10931110 (the 2/1 above) | died | kill (sure 4/4) |
| 5 | 86 | 9 | 90051140 (Rotting Zombie) -> 10931110 (not a free kill) | game over | kill (sure 4/4) |

Report totals: W 1.0 (n = 1), D 1.0 (n = 1), K 1.0 (n = 2), primary trades that are free kills 1 of 2, the HB side's
other followers that died next turn 1/2. The bot lost both games that hold a moment.

Bot record against the owner:

| review | bot | bot record |
|---|---|---|
| review9 | with hbcheck | 5/12 |
| review10 | with tkill, tkroll and hbcheck, Abyss mirror | 7/12 |
| review11 | same spec | 3/13 (Rune Test Subject mirror 1/7, Abyss mirror 2/6) |
| review12 | same + `fuseguard=1`, Rune Test Subject mirror | 2/6 |
| review13 | same, v4 leaf (`ed574a4`); the bot's Abyss Midrange against the owner's Rune Test Subject | 1/6 |

## `game1/`

Run the scripts from the repo root. They read `results/games/<game_id>.json`, so copy the capture from `review13/games/`
there first.

`s13_g1t6.py` and `s13_seeds.py` pass `net=results/sweep43/leaf-v3-e3.json`. That file is byte-identical to the built-in
v4 leaf (`engine/models/h0-linear-v4.json`), so the values are the served spec's. Copy it to `results/sweep43/` as well,
or drop the `net=` override; the output is the same.

- `s13_replay.py`: the exact replay of every bot decision at the served seed (the check above; its output for all six
  games is `../replay.txt`).
- `s13_g1t6.py` → `explain.txt`: the root values at the four decisions, at the served seed, seeds 1–3, and `info=all`.
- `s13_seeds.py` → `seeds.txt`: the choice counts over seeds 1–32.
- `s13_deal.py` → `deal.txt`: `deal=indep` (served) against `deal=block`, served seed and seeds 1–16.
- `s13_alt.py` → `alt.txt`: the actual and the better turn-6 line, and the bot's leader HP afterwards.
- `s13_after.py` → `after.txt`: the next decision after either evolve target.
- `s13_endturn.py` → `endturn.txt`: the two near-identical positions with and without the 2/1.
- `show_state.py`: a compact board printer.
