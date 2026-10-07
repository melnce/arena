# review12 — 6 owner games vs the served bot, Rune Test Subject mirror (2026-10-07)

- Engine **`3742a1b`** (v3 default leaf), the build sweep 44 ran on.
- Every capture's `policy` and `strong` is `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1`:
  review11's spec plus `fuseguard=1`, the served spec since sweep 42.
- serve.py was started by the runner session for these games (20:24 UTC) and stopped after them.
- The owner is on side `a`, against the fair bot only (no cheater games), in the **Rune Test Subject** mirror
  (`meta-rune-test-subject`, game ids `…-104cf827`).
- On the box at the same time, sweep 44's main final arm was running: its last chunk ended at 22:02 UTC. The reverse arm
  had finished at 20:04 UTC, before serve.py started. The search is node-limited, so this changed thinking time only, not
  moves.

**Bot won 2 of 6.** The owner asked for emphasis on game 6. There the bot ended its last turn with Sephie 7/7 still
alive, and the owner won with a 20-damage turn. Killing her would have removed every winning turn-9 line. That analysis
comes first, below the table.

| # | game | first | winner | actions | finished (UTC) | bot think: median / max |
|---|---|---|---|---|---|---|
| 1 | `10688971521922649840-104cf827` | owner | owner | 96 | 2026-10-07T20:42:43Z | 6.8 s / 53.1 s |
| 2 | `469609294969511015-104cf827` | owner | owner | 122 | 2026-10-07T21:05:16Z | 1.9 s / 83.1 s |
| 3 | `5277947071364447533-104cf827` | owner | owner | 103 | 2026-10-07T21:17:38Z | 0.7 s / 47.4 s |
| 4 | `4812917870626254794-104cf827` | owner | **bot** | 67 | 2026-10-07T21:20:57Z | 0.6 s / 13.3 s |
| 5 | `16150004444303745212-104cf827` | bot | **bot** | 87 | 2026-10-07T21:29:22Z | 0.9 s / 40.4 s |
| 6 | `4280399497766595080-104cf827` | owner | owner | 96 | 2026-10-07T21:50:23Z | 5.2 s / 69.3 s |

`games/<game_id>.json`: the raw serve.py captures, byte for byte.

- All 6 are `final`, `humanSide` `a`, engine `3742a1b`, `first` `coin`, no `reseed`.
- Each replays to a terminal state with the recorded winner. The replays ran on the box's `ed574a4` bindings, after its
  pull. Between `3742a1b` and `ed574a4`, `engine/src` changes only the default leaf (`engine/src/policy`); the rest of
  the diff is the v4 model file, its README and tests. The rules are unchanged.
- "first" is the player to act after the mulligans, the same method as review10 and review11.
- Think times come from serve.py's log, which writes one line per bot request.
  - The log has 288 bot lines against 287 bot actions in the captures, mulligans included.
  - The one extra line is in game 6, turn 8 (10 lines, 9 actions). The box cannot tell which request was repeated.
  - Game 6's median and max include that line.

## Game 6: the bot left Sephie 7/7 alive on its last turn and lost to a 20-damage turn

**Verdict: a real blunder.** Killing Sephie was available and would have left the owner no winning line on turn 9.

- **If the bot kills Sephie** with its two fresh rush Test Subjects, the owner has no win on turn 9.
  - The owner's turn was searched exhaustively once for each of the 13 distinct cards the owner could draw.
  - None of those draws gives the owner a win.
  - Whether the bot then wins is not computed. The owner would be at 4 HP, facing 16 attack on board and two cards in the
    bot's hand.
- **If the bot ends its turn**, the owner wins with a 6-action line: 20 damage into 18.
  - It did end the turn in the game.
  - Replayed from this position, the served bot ends with Sephie alive in 4 of 6 seeds.
- **Cause: the opponent model never finds the owner's kill.**
  - It misses it in every sampled world, and even with the owner's real hand shown (`info=all`).
  - The leaf is not the cause. The v4 leaf, served since `ed574a4`, also leaves Sephie alive: End Turn in seeds 1 and 2,
    the Lyria trade in seed 3.
- **Of the opponent-model knobs tried, only `olsolve` (off by default) ever finds the kill**, and only in some worlds.
  With `olsolve=4000` the bot kills Sephie in 4 of 6 seeds, against 2 of 6 for the served bot.

`game6/` holds the scripts and their outputs.

### The position: the bot's End Turn on turn 8 (after 89 actions)

The bot's turn 8, from 8 PP plus its bonus PP (9 spent):

1. It played Obsessed Test Subject (2 PP), which got Storm from the crest.
2. It hit face with Enamored Researcher and that Test Subject: the owner went 14 → 4.
3. It took the bonus PP and played Humane Love (3 PP). That Test Subject attacked Lyria, which only popped her Barrier.
4. It played two more Test Subjects (Rush, 2 PP each), then ended the turn without using them.

| | owner (`a`) | bot (`b`) |
|---|---|---|
| leader | **4**/20 | 18/20 |
| PP next turn | 9 | — |
| field | Lyria, Skydestined 1/1 (Barrier gone); **Sephie, Maven Convict 7/7** (super-evolved, can attack next turn) | Enamored Researcher 4/4; Test Subject 6/5 (attacked); Test Subject 6/4 (attacked); **Test Subject 6/5 and 5/5, Rush, one attack each left** |
| hand | Ecstatic Scholar, Sweet Abomination, Tetra & Ladica, Humane Love, Sephie (draws Alchemic Flare) | Obsidian Raven, Tico |

Both sides have Sephie's crest: *"Once on each of your turns, when an allied Obsessed Test Subject enters the field,
give it Storm."*

The bot's legal moves at that point were End Turn, or either rush Test Subject attacking Lyria or Sephie. Two attacks on
Sephie (6 + 5 ≥ 7) kill her, and both Test Subjects die to her 7 attack.

### What the owner did (turn 9)

1. Tetra & Ladica: 6 PP, 5/5 Storm.
2. Humane Love: 3 PP. Its Test Subject enters 6/5 and gets Storm from the crest.
3. Evolve that Test Subject to 8/7.
4. Attack face: **8 + 5 + 7 (Sephie) = 20 ≥ 18.**

### With Sephie dead, the owner has no win on turn 9

`s12_last_turn.py` uses the same depth-first search as `lethal_dfs.py`. It makes both Sephie attacks and ends the turn,
then searches every owner action sequence of turn 9 for a win. The search runs with full information, the game's own RNG
and a transposition set on the state hash, and reached no cap.

- **Actual draw (Alchemic Flare): no win**, 220,404 states.
- **Other draws:** the game RNG is reseeded before End Turn, so the owner draws something else.
  - `draws`, reseeds 1–15, covers 9 more distinct cards: Tico, Witch's New Brew, Ecstatic Scholar, Obsidian Raven, Humane
    Love, Lyria, Wills United, Test Subject and Sweet Abomination. Reseed 8 draws Alchemic Flare again.
  - `cover`, reseeds 18–20, covers the last 3 of the 13 distinct cards left in the owner's 27-card deck: Enamored
    Researcher, Tetra & Ladica and Miscalculated Experiment.
- **No win in any of the 19 runs**: 40k–541k states each.
- Each distinct draw was searched under one RNG state, so other random effects within the turn were not varied.
- After that, the owner would sit at 4 HP facing Researcher 4/4 and Test Subjects 6/5 (Storm) and 6/4, with Raven and
  Tico in the bot's hand.

### Why the bot ended the turn

Root values at this decision, served spec (v3 leaf), three bot seeds:

| seed | path | chosen | → Lyria (slot 3 / slot 4) | → Sephie (slot 3 / slot 4) | End Turn |
|---|---|---|---|---|---|
| 1 | search | **End Turn** | 21.93 / 19.22 | 33.48 / 33.79 | **40.99** |
| 2 | search | **End Turn** | 31.05 / 30.63 | 36.27 / 36.45 | **37.81** |
| 3 | holdback_trade | slot 3 → Lyria | 37.27 / 29.55 | 38.20 / 38.43 | 45.00 |

These are re-runs with seeds 1–3, not the game's own decision.
- The capture records the served End Turn's value as 35.41.
- serve.py's seed for that decision is not recorded.
- The same explain on `3742a1b`, run before the box pulled, printed the same values as these `ed574a4` runs with the v3
  leaf pinned. That earlier output was not kept as a file.

- **No sampled world sees the owner's kill.**
  - Each candidate gets 8 determinized worlds.
  - No world of any candidate ends in `opp_lethal` or `opp_solver`. Every one ends `opp_reply` (the greedy opponent line
    ran out) or `tt`.
  - Seed 3's holdback trade hits Lyria, which does not stop the kill either.
- **With full information (`info=all`, the owner's real hand) it kills Sephie in all three seeds**, but not because it
  sees lethal.
  - Sephie scores 56.13 / 50.99 / 50.88 against End Turn's 23.76.
  - End Turn still scores +23.76 there, and all 8 of its worlds end `opp_reply`, not in a loss. Even with the owner's
    real cards, the default opponent model does not find this kill. That model is the greedy line plus the `olethal`
    sweep with one evolve.
  - The kill takes two plays from hand plus an evolve.
  - The full-information bot kills Sephie on leaf value, not on lethal.
  - Hidden information makes End Turn look better still: 40.99 / 37.81 / 45.00, against 23.76 with full information.
- **The leaf is not the cause.**
  - The v4 leaf (`h0-linear-v4`, the bot served since `ed574a4`) leaves Sephie alive too.
  - It ends the turn in seeds 1 and 2 (43.39 / 40.69 against Sephie ≤ 39.60) and makes the Lyria holdback trade in
    seed 3.

### Opponent-model knobs tried

Each row adds a knob set to the served spec (v3 leaf), seeds 1–3. The last column counts End Turn's worlds that found
the owner's kill. Outputs are in `knobs.txt` (the first five rows) and `knobs2.txt` (the `odepth` rows).

| added knobs | seed 1 | seed 2 | seed 3 | End Turn value | End Turn worlds with the kill |
|---|---|---|---|---|---|
| none (served) | End Turn | End Turn | slot 3 → Lyria | 40.99 / 37.81 / 45.00 | 0 / 0 / 0 of 8 |
| `okill=7` | identical to served | | | identical | 0 / 0 / 0 |
| `omacro=1` | End Turn | End Turn | End Turn | 36.84 / 35.82 / 45.14 | 0 / 0 / 0 |
| **`olsolve=4000`** | **slot 4 → Sephie** | slot 3 → Lyria | **slot 4 → Sephie** | 3.48 / 36.87 / 20.17 | **3 / 1 / 2** (`opp_solver`) |
| `okill=7,olsolve=4000,omacro=1` | same as `olsolve=4000` alone | | | | |
| `info=all` + the three | slot 3 → Lyria | slot 3 → Lyria | slot 3 → Lyria | 53.57 / −13.22 / 20.17 | 0 / 4 / 2 |
| `odepth=1` | End Turn | End Turn | End Turn | 42.66 / 39.77 / 42.84 | 0 / 0 / 0 (`opp_search`) |
| `odepth=2` | End Turn | End Turn | End Turn | 20.59 / 21.72 / 19.85 | 0 / 0 / 0 (`opp_search`) |
| `odepth=1,obeam=6` | End Turn | End Turn | End Turn | 39.81 / 39.77 / 39.05 | 0 / 0 / 0 (`opp_search`) |

- **`olsolve`** is the bounded `forced_lethal` run after an `olethal` sweep miss.
  - It is the only knob set tried that finds the kill, and only in some worlds.
  - At the root it picks a Sephie attack in seeds 1 and 3 only.
  - Its budget is `min(olsolve, cap − nodes)` (`engine/src/policy/h0.rs`), so it gets only what the pair's node cap has
    left.
- **The `info=all` + three row:** End Turn's worlds still miss the kill in 8, 4 and 6 of 8.
  - The first move is the Lyria trade in all three seeds. In seed 1 that comes from the `holdback_trade` path, since
    End Turn (53.57) outscores Lyria (52.83) there.
  - In seeds 2 and 3, Lyria (52.83) outscores the best Sephie attack (51.08).
- **The `odepth` opponent search** (which replaces the greedy line and the `olethal` sweep) misses the kill in every
  world, with or without a wider beam.

**Played to the end of the turn** (`s12_playout.py`, seeds 1–6):

1. The bot finishes its turn 8 one decision at a time, with the given spec and seed.
2. The owner's turn 9 is then checked:
   - first the owner's actual line, by card;
   - otherwise the full search, which is exhaustive when it finds no win.

| spec | the bot's line, by seed | owner wins on turn 9 |
|---|---|---|
| served | End Turn (1, 2, 4); Lyria, then End Turn (3); kill Sephie (5, 6) | **4 of 6** |
| + `olsolve=4000` | kill Sephie (1, 3, 4, 5); Lyria, then End Turn (2); End Turn (6) | **2 of 6** |

- Every line that kills Sephie leaves the owner without a win (exhaustive, 220,404 states each).
- Every line that leaves her alive loses to the owner's actual line.
- So the served bot finds the save in 2 of 6 seeds, and `olsolve=4000` in 4 of 6.

`olsolve` was measured only at budgets 200 and 1000 on older specs: sweeps 18 and 19, and 24 and 26 under `info=all`.
Sweep 19's `h0:nodes=6000,olsolve=200` was worse, pooled 0.473 [0.456, 0.491]. It was never measured on today's served
spec (`nodes=32000`, horizon 3, `tkill` / `tkroll` / `hbcheck` / `fuseguard`). This is one position, not a measurement.

The held-back scan below also flags this End Turn (ply 89).
- Its primary trade is the free kill on Lyria.
- The scan models single-attack kills only, so it cannot see the two-attack Sephie kill.

## Restarts: none

- The owner reported no redone game this time.
- serve.py's log (one line per bot request, with the turn number and a state-hash check) agrees.
  - Within each `game_id` the turn number never goes backwards.
  - Every hash check is `ok`.
- All 6 seeds differ, all 6 captures are `final`, and there is no `-2.json` rollover.

## Preview, not a deciding measurement: the held-back scan

`holdback_audit.py scan` (holdback1's script, unchanged) ran on these 6 captures, bot seat only:
`holdback_scan_preview.jsonl`. It also ran on the `ed574a4` bindings.

- It finds **5 HB moments**: 0.83 per game, 0.102 per End Turn. review11 had 0.46 per game, review10 0.33 and review9
  0.42.
- The script's printed `VERDICT` line is holdback1's rule and does not apply here.

| game # | ply | turn | primary trade X -> Y (card ids) | X next turn | can the owner kill X |
|---|---|---|---|---|---|
| 1 | 89 | 8 | 10931110 -> 10931110 (not a free kill) | game over | kill (sure 4/4) |
| 2 | 85 | 8 | 10931110 -> 10403120 | died | kill (sure 4/4) |
| 2 | 103 | 9 | 10931110 -> 10403120 | died | kill (sure 2/4) |
| 3 | 34 | 4 | 10931110 -> 10931110 | died | kill (sure 4/4) |
| 6 | 89 | 8 | 10931110 -> 10403120 | game over | kill (sure 4/4) |

10931110 is Obsessed Test Subject and 10403120 is Lyria, Skydestined.

Report totals:

| measure | value |
|---|---|
| W (wasted) | 1.0 (n = 3) |
| D (X died next turn) | 1.0 (n = 3) |
| K (the owner had a sure kill line on X, 4/4 reseeds) | 0.8 (n = 5; game 2 at ply 103 was 2/4) |
| primary trades that are free kills | 4 of 5 |
| the HB side's other followers that died next turn | 7/9 |

The bot lost all four games that hold a moment (games 1, 2, 3 and 6). Two moments end with the owner winning on the
next turn: games 1 and 6, both at ply 89.

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
| review12 | same + `fuseguard=1`, Rune Test Subject | 2/6 |

## `game6/`

Run the scripts from the repo root. They read the capture at `results/games/<game_id>.json`, so copy
`review12/games/4280399497766595080-104cf827.json` there first.

- `lethal_dfs.py`: the exhaustive win search for the side to move.
- `s12_last_turn.py`:
  - `draws` writes `draws.txt` (the actual draw and reseeds 1–15);
  - `cover` writes `cover.txt` (the 3 distinct cards `draws` did not reach);
  - `explain` gives the root values.
- `s12_worlds.py` and `worlds.txt`: the root values with the v3 and v4 leaves and with `info=all`.
- `s12_knobs.py`:
  - `knobs.txt`: the default five knob sets;
  - `knobs2.txt`: `python s12_knobs.py odepth=1 odepth=2 odepth=1,obeam=6`.
- `s12_playout.py` and `playout.txt`: the bot's turn played to its end per seed, then the owner's search.

The v3 runs used `ed574a4` bindings with `net=engine/models/h0-linear-v3.json` pinned.
