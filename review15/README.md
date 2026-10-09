# review15 — 4 owner games vs the served bot: the owner's Rune Test Subject against the bot's Sword Rally (2026-10-09)

- Engine **`ed574a4`** (v4 default leaf), the same build and spec as review13 and review14: `policy` and `strong` are
  `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1` in every capture.
- serve.py was started by the runner session at 21:39 UTC on 2026-10-09 and stopped after the games. Its log is
  `results\serve-review15.log` on the box.
- The owner is on side `a` with **Rune Test Subject** (`meta-rune-test-subject`); the bot plays **Sword Rally**
  (`meta-sword-rally`). All four games are that pairing (game ids `…-b43e6214`).
- **One abandoned start is not in the table:** `5315619802755928396-b8ab69bb`, with the decks the other way round, 0
  actions and not final. serve.py logged one bot request for it.
- On the box at the same time, sweep 46's final was running. The search is node-limited, so this changed thinking time
  only, not moves.

**Bot won 1 of 4** (game 1). The owner flagged nothing specific. The lethal audit below found the notable part: four bot
turns ended handing the owner a forced kill.

- Two of them were avoidable with full information: a Mars board clear the bot's search did not find.
- Found, the clear would not clearly have changed the bot's move. Against the hands the bot could not see, it rated the
  clear no better than what it played.

| # | game | first | winner | actions | finished (UTC) | bot think: median / max |
|---|---|---|---|---|---|---|
| 1 | `6065548863054576576-b43e6214` | bot | **bot** | 80 | 2026-10-09T21:49:05Z | 0.8 s / 29.3 s |
| 2 | `12757697801983742963-b43e6214` | owner | owner | 104 | 2026-10-09T21:57:51Z | 1.1 s / 46.9 s |
| 3 | `2374435823510868837-b43e6214` | owner | owner | 139 | 2026-10-09T22:25:35Z | 3.9 s / 86.6 s |
| 4 | `1068176807631359990-b43e6214` | bot | owner | 104 | 2026-10-09T22:40:14Z | 1.8 s / 58.4 s |

`games/<game_id>.json`: the raw serve.py captures, byte for byte.

- All 4 are `final`, `humanSide` `a`, engine `ed574a4`, `first` `coin`, no `reseed`.
- Each replays to a terminal state with the recorded winner.
- serve.py's log has one line per bot request: 179 for these four games, each game's count equal to its bot actions, plus
  the abandoned start's one. Turns never go backwards within a `game_id`, and every hash check is `ok`.

**All 179 bot decisions replay exactly** at the served seed (review13's method; `scripts/s13_replay.py` → `replay.txt`):

- 138 of 138 with a recorded `bot_value` give the same action and value (to 1e-6).
- 41 of 41 without one give the same action.

## The lethal audit: four bot turns handed the owner a forced kill

`py/lethal_audit.py --records games/ --bot-seat b` (the repo's own audit, budget 50 000; `lethal_audit.json`) asks the
exhaustive within-turn solver, `forced_lethal`, two questions. It works with full information and the game's own RNG.

- **Missed kills** (at each bot decision, a kill the bot then did not take): 175 decisions asked, 171 decided, 4 unknown.
  - A kill was available at 5 decisions, all on game 1's last turn: turn 9, plies 75–79, the kill line shortening from
    5 actions to 1.
  - The bot took it. **0 missed.**
- **Handed kills** (after each bot End Turn, the owner has a kill): 37 End Turns asked, 33 decided, 4 unknown.
  - **4 kills found: 12.1% of decided End Turns.**
  - 2 are deterministic and 2 depend on the RNG (the kill line draws on random effects).

In all four, **End Turn was the bot's only legal move** (explain: `path single_legal`). So whatever handed the kill
happened earlier in the turn. Four scripts took it from there:

- `s15_safe.py` → `safe_search.txt` searched the bot's whole turn:
  - every distinct end-of-turn position it could reach from its turn's first decision, depth-first in legal-move (hand)
    order, with a transposition set and a cap of 4 000 positions (a few more are added as the search unwinds);
  - then `forced_lethal` for the owner from each (budget 20 000).
- `s15_turnstart.py` → `turnstart.txt` explained the bot's first decision of the turn at the exact served seed (review13's
  replay method), and re-checked the safe line at budget 200 000.
- `s15_inside.py` → `inside.txt` valued the safe line from inside it, and replayed it under four reseeded owner draws.
- `s15_after_clear.py` → `after_clear.txt` measured the owner's best possible next turn after it.

| game | bot's turn | leaders: bot / owner | the owner's kill | owner took it | a bot turn that leaves no kill (full information)? |
|---|---|---|---|---|---|
| 4 | 8 | 12 / 13 | deterministic, 6 actions | **no** (won a turn later) | none in 4 009 lines searched (cap reached: not exhaustive) |
| 4 | 9 | 9 / 11 | deterministic, 6 actions | yes | **yes**: 4 of 4 003 lines searched (cap reached) |
| 2 | 9 | 16 / 13 | RNG-dependent, 9 actions | yes | **no**: all 266 possible bot turns leave the kill (exhaustive) |
| 3 | 11 | 8 / 10 | RNG-dependent, 6 actions | yes | **yes**: 1 of 4 005 lines searched (cap reached) |

### The two avoidable ones: a Mars board clear the bot's search did not find

Both safe lines are the same idea. Play **Mars, Conflagrant Commander**: 8 PP, 1/5, Storm and Bane, with *"Fanfare: Summon
3 copies of Knight"* and *"Whenever an allied Officer follower enters the field, give it +2/+0 and Rush and give this
follower +1/+0"*. Then play a Steelclad Knight and trade everything into the owner's board.

**Game 4, turn 9** (bot at 9 HP; the owner has Obsessed Test Subject 2/1, Sephie 7/7, Test Subject 5/5)

- **The clear:** Mars, Steelclad Knight, then five attacks. Both fields end empty.
- **The owner's best reply after it** (`after_clear.txt`):
  - The owner has 13 HP, 9 PP and 1 super-evolve point. In hand: Sephie ×2, Alchemic Flare, Tetra & Ladica, Ecstatic
    Scholar, Enamored Researcher ×2.
  - Alchemic Flare cannot be cast: it needs an enemy follower, so its Skybound Art's 2 to the face is unavailable.
  - The most face damage that turn is **8 into the bot's 9** (exhaustive). Example: Sephie, then the storm Test Subject
    she summons, super-evolved; Tetra & Ladica super-evolved also gives 8.
  - The clear also held under four reseeded owner draws (`inside.txt`).
- **What the bot did:** it played Yidmetra, Eld Sword, and lost on the owner's turn.
- **Its view at the served seed** (`turnstart.txt`):
  - Every first move scored −80.00 with all 8 sampled worlds ending in an opponent kill. That includes "play Mars" (both
    copies) and one of its two Yidmetra copies.
  - The other Yidmetra copy, the one it played, scored −65.78 with 3 of 8.
  - So the bot saw it was in trouble, and its search did not reach the clear behind "play Mars".
- **Inside the line** (`inside.txt`), the bot does find the clear and finishes it, but scores it only −60 to −69. Across
  three seeds, 1–4 of its 8 sampled worlds still end in an owner kill: it cannot see that the owner's kills both end with
  an Alchemic Flare that needs a target. That is about the same as Yidmetra's −65.78.
- **So against the owner's real hand the clear wins the turn**, by exactly 1 HP. Against the hands the bot samples, it is a
  coin flip with what it played.

**Game 3, turn 11** (bot at 8 HP; the owner has Witch's New Brew, Ecstatic Scholar 5/6, three Test Subjects)

- **The clear:** Mars, Steelclad Knight, an attack, a second Knight, then five attacks. It leaves the owner only Witch's
  New Brew 0/0, and with the owner's actual draw the owner's best reply is 5 damage into 8 (`after_clear.txt`).
- **Its safety rests on the realized draw** (`inside.txt`). Reseeded at the turn start (seed 12345), the owner draws
  Tetra & Ladica and has a deterministic kill after the clear. Reseeds 1, 2 and 3 give none.
- **What the bot did:** it played Cesar, Accordant Major. At the served seed Cesar scored 17.19 with 0 of 8 worlds ending
  in an owner kill, and "play Mars" −80.00 with 8 of 8.
- **Inside the line** the bot finds the clear itself, but scores it only −48 to −51 (0 of 8 worlds end in a kill), far below
  Cesar's 17.19. So even a search that reached the clear would have played Cesar.
- **The kill Cesar left was RNG-dependent,** so the bot's sampled worlds not containing it is not by itself an error.

### The unavoidable one, and the undetermined one

- **Game 2, turn 9:** an exhaustive search of all 266 of the bot's possible turns finds none that removes the owner's
  RNG-dependent kill.
- **Game 4, turn 8:** the search reached its cap (4 000 end-of-turn positions; it stopped at 4 009), all leaving the
  deterministic kill. It runs in hand order, and Mars was the fourth card in hand there, so lines starting with Mars may
  not have been reached. Whether a safe line exists is not known.

### For the owner: a missed kill

- **Game 4, turn 8:** the owner had a deterministic kill after the bot's End Turn.
  - The line: an attack to face, Tetra & Ladica, super-evolve, a second attack to face, then Alchemic Flare, whose
    Skybound Art adds 2 to the face.
  - The owner did not take it and won a turn later, on the next handed kill.
- `review15-replays.zip` holds that position as an importable replay (Settings → Practice & Positions → Import).

### Reading

This is a 4-game preview, not a measurement.

- **In both avoidable cases the bot's root search did not reach a 7–9-action Mars board clear,** and scored its first
  move as a certain loss. Both are the same idea in one matchup.
- **Finding it would not by itself have changed the move.**
  - Game 3: the bot rates the clear −48 to −51 against Cesar's 17.19.
  - Game 4: it rates it −60 to −69, about equal to Yidmetra.
- **In both, the deciding factor was hidden information,** not search length alone:
  - game 4: the owner's Alchemic Flare having no target after the clear;
  - game 3: the owner's RNG-dependent kill after Cesar, and the draw that would have beaten the clear.

## Preview, not a deciding measurement: the held-back scan

`holdback_audit.py scan` (holdback1's script, unchanged) ran on these 4 captures, bot seat only, on the `ed574a4`
bindings (`holdback_scan_preview.jsonl`). It finds **no HB moments** in 37 bot End Turns. review14 had 1.25 per game
against Haven Evo.

Bot record against the owner (recent reviews):

| review | bot | bot record |
|---|---|---|
| review13 | v4 (`ed574a4`); the bot's Abyss Midrange against the owner's Rune Test Subject | 1/6 |
| review14 | same; the bot's Haven Evo | 1/4 |
| review15 | same; the bot's Sword Rally | 1/4 |

## Files

- `games/`: the four captures.
- `replay.txt`: the exact replay of every bot decision.
- `lethal_audit.json`: the lethal audit, per game and per decision.
- `handed.txt` (`scripts/s15_handed.py`): each handed kill, the End Turn's legal moves and the owner's kill line.
- `safe_search.txt` (`scripts/s15_safe.py`): the bot's whole-turn search.
- `safe_lines.json`: the two safe lines, as actions.
- `turnstart.txt` (`scripts/s15_turnstart.py`): the bot's first decision of the two avoidable turns at the served seed,
  and the safe lines re-checked.
- `inside.txt` (`scripts/s15_inside.py`): the clear valued from inside it, and replayed under reseeded draws.
- `after_clear.txt` (`scripts/s15_after_clear.py`): the owner's best next turn after the clear.
- `review15-replays.zip`: importable replays for the arena UI (README inside): the three positions above and all four
  games.
- `holdback_scan_preview.jsonl`: the held-back scan.
- `scripts/s13_replay.py`: the replay check.

Run the scripts from the repo root (they load `cards/`).

- `s13_replay.py` takes capture paths as arguments.
- The `s15_*` scripts take `GAME_ID:PLY` arguments (and `s15_turnstart.py` first takes `safe_lines.json`). They read
  `results/games/<game_id>.json`, so copy the captures there first.
- `lethal_audit.json` came from `python py/lethal_audit.py --records <folder of the four captures> --bot-seat b`.
