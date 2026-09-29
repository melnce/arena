# review4: five owner games against the `horizon=3` served bot

- **Played** 2026-09-29, 00:04–00:27 UTC, against `h0:nodes=16000,horizon=3` with the default leaf `h0-linear-v2`, on engine
  **71127bf**. Sweep 15b's final was running at the same time; its tp stages had already finished.
- **Matchup:** the Abyss Midrange mirror. The owner (seat `a`) went first in 4 of the 5 games. **The owner won 5–0.**
- **No deep reference.** This is a bot-side analysis:
  - `bot_action_explain` re-run at the served spec for every bot decision;
  - the exact `forced_lethal` solver (budget 50 000) before every action, with the two `unknown` verdicts re-run at 500 000;
  - an escape check at the bot's last turns, where each first move is completed by a perfect-information bot and then the solver checks the owner's reply.

## Files

| path | contents |
|---|---|
| `games/<id>.json` | the raw captures from `py/serve.py`, byte for byte |
| `botview/<id>.txt` | bot's-eye timelines with explain output and `[SOLVER]` tags |
| `botview/CARDS.txt` | card texts |
| `escape_check.txt`, `escape_detail.txt` | the escape checks |
| `tools/` | the scripts; see `tools/README.md` for how `[N]` maps to positions |

## The two avoidable losses: exact positions

### `14155189002142913913-5ce21003`

**The bot's decision.** Bot turn 6 starts before **[44]**, with the bot on 11 HP and the owner on 18.

- **What the bot played ([44]–[50], from the capture):**
  1. [44] switch on Bonus PP
  2. [45] Lilith, Devilish Cutie
  3. [46] Adahime
  4. [47] Netherworld Lieutenant 1/1 attacks Rotting Zombie 2/2
  5. [48] Netherworld Lieutenant 2/1 attacks the owner's Netherworld Lieutenant 2/1
  6. [49] super-evolve Adahime to 8/7, which then **did not attack**, so the owner's evolved Highwire Feline 6/6 stayed alive
  7. [50] end turn
- **Its values:**
  - The live bot's `bot_value` went from +51.0 at [44] to +32.8 at [50].
  - The served explain, re-run at [44], put its top candidate (Lilith first) at **+53.6**, with every one of its 4 worlds at +46.8 or better.
- **Safe and unresolved alternatives** (the rest of the turn played by a perfect-information bot, then the solver):
  - **Adahime first leaves the owner no forced kill**, with the owner's board empty.
  - Bonus PP first, the same first move as the bot, is `unknown` at the 50 000 budget. That continuation also sends Adahime 8/7 into Highwire Feline 6/6.

**The owner's kill.** From the position before **[51]** (owner to act, turn 7), the solver finds a forced kill.

- At the 500 000 budget: `lethal`, 219 933 nodes, rng-dependent. At 50 000 the verdict is `unknown` at [51] and `lethal` from [52] on.
- The line breaks the bot's Ward first (slot 0 is Void Colonel 6/8, Ward), then goes face:
  `attack s0->slot0 | attack s1->slot0 | play Adahime | attack s3->slot0 (x3) | evolve | play Lilith, Devilish Cutie | attack s2->slot0 | attack s0->leader`.

### `9420046197828951589-5ce21003`

**The bot's decision.** Bot turn 7 starts before **[64]**, with the bot on 12 HP and the owner on 14. The line diverges at **[65]**.

- **What the bot played:**
  1. Hark to the Night Song
  2. super-evolve Raz, Demon on the Drums (choose slot 0)
  3. Depths of the Eld Sight
  4. Raz 6/4 attacks the owner's leader
  5. play a second Raz
  6. end turn
- **The safe line:** the same Hark first, then **Raz 3/1 attacks the owner's super-evolved Highwire Feline 7/2**, then the rest played by a perfect-information bot. It leaves the owner no forced kill.
- **What the bot saw at [65]:**

  | served candidate | value | worst world |
  |---|---|---|
  | play Raz | +24.2 | |
  | super-evolve Raz | +22.3 | |
  | Depths of the Eld Sight | +20.1 | |
  | Raz attacks the leader | +16.5 | |
  | Raz attacks Skeleton | −5.8 | −19.2 |

  All worlds were at −19 or better. "Raz attacks Highwire Feline" is not in the top 5.

**The owner's kill.** From the position before **[71]** (owner to act, turn 8), the solver finds a forced kill:
`attack s0->leader | attack s1->leader | play Garodeth vs. Zeth | choose mode 0 | evolve | attack s3->leader`.
In the game the owner **super-evolved** Garodeth at [73], to 11/11 with Storm.

## All five games

| game | first | turns | bot HP vs owner HP before the kill turn | could the bot's last turn have avoided the kill? |
|---|---|---|---|---|
| 257109342992564242 | owner | 11 | 10 vs 14 | **no.** Every option leaves a forced kill. Lost to the owner's turn-10 setup |
| 9420046197828951589 | owner | 8 | 12 vs 14 | **yes**, see above |
| 11296488206978013884 | bot | 7 | 8 vs 14 | **no.** The owner's turn 6 took the bot from 19 to 8 HP and set up the kill. There was no immediate kill after the bot's turn 6 (500 000: `none`) |
| 4373984720253669473 | owner | 7 | 5 vs 10 | **no.** The owner's turn 6 took the bot from 15 to 5 HP |
| 14155189002142913913 | owner | 7 | 11 vs 18 | **yes**, see above |

## Other findings

- **Solver results:**
  - The only forced kills the solver found in all five games were the owner's, each on the final turn.
  - It found none for the bot.
  - It found no earlier owner kill, so the owner missed none. A few mid-game positions stayed `unknown` at the 50 000 budget.
- **Horizon fix:** lines cut off by depth or the node cap now meet a reply. The explain output shows the new `cap_reply` and `depth_reply` ends. None of the five losses traces to a line cut off before the reply (review3's P1).
- **The two avoidable losses are P3:** the reply model does not produce a Ward-break kill (14155) or a Storm plus super-evolve kill (9420).
- **The other three are two-turn plans.** The one-turn escape check cannot show whether the bot's earlier turn could have prevented them.
- **Caveats:** five games, one matchup, one strong human. Explain re-runs use a different seed from the live game. The perfect-information completions use information the bot did not have.
