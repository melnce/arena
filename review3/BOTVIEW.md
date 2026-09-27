# h0 vs the owner: bot-side findings from 18 games (report for Cowork)

**Scope.** Five analyst groups each covered part of the 18 games, and a checker verified each group. Refuted claims are left out. Claims the checker weakened are marked **(W)**.

- **Build and spec.** The games ran on engine build a45e2bb. The bot played `h0:nodes=16000`: k=4, depth 6, beam 4, odepth=0, pess=0, lcap=0.5.
- **Indices.** `[N]` is the action index in the botview timeline. The position is the one before action N. Values are from the acting side's view.
- **Short game ids.** Each game is named by its leading digits. `2560m` is the mirror game 2560486491816220020-5ce21003 and `2560f` is the Forest game 2560486491816220020-c9e8a9fb.
- **Files** (all under `review3/` on the `results` branch):
  - `botview/<GAME_ID>.txt`: the bot's-eye timeline of each game; `botview/CARDS.txt`: card texts
  - `games/<GAME_ID>.json`: the raw game records; `<GAME_ID>.json` and `REVIEW.md`: the `py/review.py` output (reference `h0:nodes=200000,k=16`, `--side both`)
  - `tools/replay_at.py` and `tools/cf_skip.py` (`cf_skip` replays a game skipping one action), plus `tools/botview.py`; they need bindings built at a45e2bb (see `tools/README.md`)
- **The owner cannot answer section 6**: the games were played hours before this report, so treat those questions as open.

## 1. Headline

The bot lost 13 of the 18 games: 1-6 in the Abyss mirror and 4-7 against Forest Combo. The checker confirmed a decisive or major bot decision in 7 of the 13 losses:

- **4 of the 7 (P1): truncated lines beat answered lines.** A candidate line that runs out of depth 6 or of its node share before the bot's turn ends gets a plain static score. It never meets the human's reply or the opponent-lethal check, yet it is compared directly against lines that did. Instances: 6711 [106], 7210 [59], 6482 [94], 2557 [57]; 2560m [39] is a further major instance.
- **2-3 of the 7 (P2): the mean over 4 sampled hidden hands (worlds) hides the human's lethal.** On an irreversible mode choice it either never samples the world where the human has lethal, or averages that world away. Instances: 2560m [68], 10951 [70], and 14567 [74] (W).
- **4 of the 7 are the same decision class:** the bot declined Garodeth's Ward + 8-damage-to-all mode.

The other 4 losses show no better bot move. They were lost to human turns that the greedy opponent model never generates: ward-break lethal, Combo-3 setups, super-evolve payoffs (P3). The deep reference shares both P1 and P3, so "the reference agrees with the bot" is not evidence here. Bonus PP burns happened in 6 of the 13 second-seat games but cost no game, and current HEAD already fixes them.

## 2. Patterns (ranked by games affected × severity; n = 13 losses, one human, two matchups)

| # | Pattern | Confirmed decisive/major losses | Other instances | Status |
|---|---|---|---|---|
| P1 | Truncated lines (depth/cap, no reply) beat answered lines | 6711, 7210, 6482, 2557; 2560m [39] | 14567 [74][100], 13238 [99] (W), 7210 [76], 5146 [33] (W) | Confirmed in code and by experiment |
| P2 | k=4 mean dilutes opponent-lethal worlds; worlds redrawn per sub-decision | 2560m, 10951; 14567 (W) | 2560m [69], within-turn value jumps | Confirmed |
| P3 | Greedy opponent reply plus face-only lethal sweep | 6482 ([97], together with P1) | Main story of 11837, 11162, 11472, 2560f, 13238; inflates race worlds in 10951 | Confirmed in code; decision impact mostly unproven |
| P4 | Duplicate root candidates | Amplifier in 14567 [74], 6482 [94] | 5146 [56], reference noise | Code-verified, still in HEAD |
| P5 | Value net over-credits own unspent PP | Mechanism under P1/P3 | All traces | Hypothesis from weights, not instrumented |
| P6 | Bonus PP toggled at the first legal moment | 0 losses | 13692 [49] (cost a forced t7 win in a won game), 7075 [48] | Mostly fixed at HEAD; remainder under sweep14 |
| P7 | Leaf blind to opponent crest/deck buffs and own crest | 0 demonstrated | 2560f [103], 2557 [55] | (W) |

**Order of fixes matters.** Fix P1 before strengthening P2 or P3:

- Blanket pessimism penalises only lines that face the opponent model. At 6711 [106], `pess=0.5` picks the losing move.
- A stronger reply model would lower only the answered lines. That pushes the bot further toward truncated lines, as at 7210 [59].

**Decision class: Garodeth mode.** In 4 losses the decisive error was not taking Garodeth's Ward + 8-damage-to-all mode (or Garodeth vs Istyndet): 14567 [74], 2560m [68], 10951 [70], 6711 [106]. The causes differ:

- 14567: a consistent k=4 preference, cap-truncated lines, and duplicate candidates.
- 2560m and 10951: averaging (P2).
- 6711: the depth horizon (P1).

Why this card keeps showing up:

- Storm mode produces long own turns (a storm attack plus follow-ups), so its line truncates and escapes the human's reply.
- Ward mode ends the turn quickly and gets exposed to the reply and to world noise.
- Play and Choose are separate sub-decisions with redrawn worlds.

In 7075 T8 and 5883 t8 the bot took the defensive or wipe mode and won. These positions make a clean regression probe set (see the end of this section).

### P1. Truncated leaves compete with answered leaves (horizon mixing and node-cap starvation)

**What the bot does.** Some turns take many actions: Garodeth storm plus attacks, Adahime plus super-evolve plus 3-5 attacks, Macmillan zombie chains, Itsurugi modes, runs of 1-PP plays. On those turns a candidate's line uses up depth 6 or its node share before EndTurn. `search_own` then returns `eval.value(state)`. That score masks the opponent's hand, so it is identical in every world, and it comes with no `opponent_reply` and no `opp_lethal_sweep`. Candidates that end the turn quickly pay for the human's turn. The root compares the two kinds directly.

Three details sharpen this:

- **Cap check before the reply.** The cap is checked before the handoff to `opponent_reply`, so a line that has just played EndTurn at the cap gets no reply at all. Example at 2560m [40]: `bonus_pp | evolve | end_turn [end cap]` scored +32.
- **Depth counts every step.** Every Play, Choose, Evolve and attack uses a depth step.
- **Thin budget per pair.** Alloc::Fair gives roughly node_cap/(k×candidates), about 250-500 nodes per (world, candidate) pair at 16k with 7-15 candidates. EndTurns reached late get a cut-down sweep (`sweep_cap = min(nodes+120, cap)`) or none.

Signature in explain output: `ends depth` or `ends cap` with identical per-world values.

**Instances and the bot's numbers:**

- **6711 t10 [106] (decisive).**
  - Storm mode line ends at depth: +28.2 in 4/4 worlds, and in 16/16 worlds at k=16.
  - Ward mode +28.5 to +29.6, with worlds +42/+13/+2/+57.
  - `depth=10`: Storm falls to -26.4 / -27.6; Ward is unchanged.
  - k=16 seed 2: Ward +51.2 (all worlds +47 to +56).
  - `pess=0.5` picks the losing Storm mode (+28.2 vs -2.3).
  - The reference (200k, k=16, depth 6) also scored Storm exactly +28.2 (cost 0.4, not flagged).
  - Real outcome: Great Hart was left alive, and Setus plus a Great Hart 9/9 dealt exactly 9.
- **7210 t7 [59] (decisive, plausible).**
  - At 64k, the cheap-card candidates end at depth with world-identical values around -36. The Void Colonel (VC) lines reach the reply and spread from -42 to +5, so a bad k=4 draw sinks them.
  - 64k: Skeleton→VC -11.2 / -18.0 and VC -14.6 / -21.5 vs Lilith -35.7 / -33.7. The reference agrees: VC -14.2 vs Lilith -34.5.
  - The live game, the explain re-run and seed 1 made three different choices.
  - Caveat: the good line needs the bot's own VC super-evolved to 7/9 so it survives Highwire.
- **6482 t10 [94] (decisive, starvation variant).**
  - Every candidate ends at the cap, so the zombie targets were picked static-vs-static. Rotting Zombie (RZ) into NL scored +14.8 and beat its own duplicate (+13.0) and RZ into Adahime (+13.0 / +12.8).
  - `nodes=200000`: RZ into Adahime -16.9 vs RZ into NL -19.8, with lines reaching opp_reply. Reference cost 52.
  - Killing Adahime leaves at most 13 damage against 19 HP; the game was lost to exactly 19.
- **2557 t7 [57] (major).**
  - The Hark and Lilith lines end `[end cap]` right after EndTurn, so they get no reply.
  - 16k seed 1: Lilith -8.5, Istyndet -9.0, Hark -10.3. The pick changes with the seed.
  - Reference: Istyndet +5.2 vs Hark -21.3 (cost 26.5).
  - Hark paid 3 PP to kill a Great Hart that the bot's own permanent Istyndet crest would have destroyed for free at end of turn.
- **2560m t6 [39]-[41] (major, from a +34 lead).**
  - 16k seed 1: Adahime +33.0, Fickle +29.4, Bonus PP +28.9 (noise level).
  - 64k on the same worlds: Bonus PP into Adahime +34.7, Fickle +5.2 (worst -20.1).
  - The bot's own value went from +35.0 at [40] to -7.2 at [41] with no human move in between. Reference cost 14.9 at [39] and 21.8 at [40].
  - The passive reply model also contributes: in explain, Fickle reached opp_reply in all 4 worlds and still scored +32.
- **Minor, or the game was lost anyway:**
  - 14567 [100]: Itsurugi (`ends depth`, -32.7) beat Garodeth (opp_reply, -58.3). At depth=10, Itsurugi falls to -70.2 with 3 of 4 worlds at -80.
  - 13238 [99] (W): seed-2 world 4 scores Bibatii +21 and Baal +16 at 16k, but every candidate is -80 at 400k with depth 10. Truncation hid the Setus lethal, although every line loses to the real hand.
  - 7210 [76]: at 3 HP facing an 11/11 Storm Garodeth, the top-8 lines all end at depth, TT or cap, valued -29 to -38. The only line that removes the Storm (Garodeth Ward mode, then super-evolved Raz attacks) is not in the top 8.
  - 5146 [33] (W): -6 when capped vs about -30 at depth 10, a 25-point misevaluation; the move gap is only 2.
  - The reference has the same bias: at 14567 [65] (200k, k=16), the Lilith lines sit at a flat +7 in 11 of 16 worlds, ending at depth.

**Helper summary.** `depth=10` or more nodes flips 6711 [106], 6482 [94], 7210 [59] and 2560m [39] in the right direction. k=16 does not help truncated lines, because their worlds are identical. pess makes them worse. At 16k, k=16 runs are budget-starved (about 200-500 nodes per pair), so "all worlds identical" results partly measure starvation.

**Cheap tests:**

- **(a) Existing tools.**
  1. Run `bot_action_explain` over bot-vs-bot decisions (the `lethal_audit --decisions` records, or `review.py` runs).
  2. Bucket the decisions by whether the chosen line and the runner-up end the same way (both answered, or both depth/cap) or differently.
  3. Re-run a sample of each bucket at `h0:nodes=64000,depth=10`.
  - **Confirms:** the mixed-end-type bucket flips far more often (more than 2x) and the flips concentrate where the chosen line was unanswered.
  - **Kills:** equal flip rates.
- **(b) Probe set.** Run the probe set below with `replay_at.py` at `h0:nodes=16000,depth=10` and at `h0:nodes=64000`, seeds 1-3.
- **(c) Code flag, then sweep.** The flag would do three things:
  - At a mid-turn depth or cap leaf, score `min(static, end-turn-now)`, where end-turn-now is `opp_lethal_sweep` plus the greedy reply on a reserved or uncharged budget.
  - Stop counting forced Choose steps toward depth.
  - Run the sweep before the cap check on EndTurn nodes.

  Sweep it with `py/sweep.py --candidates h0:<flag> --baseline h0`, then run a final at `nodes=16000` with `--decks meta-abyss-midrange meta-forest-combo` (plus `--mirrors meta-abyss-midrange`). Pair the result with `py/lethal_audit.py --by-deficit`.
  - **Confirms:** rate above 0.5 and handed_lethal down at equal deficit.
  - **Kills:** a neutral rate and unchanged probe picks.

  Prior hint that budget matters: sweep7 on an older engine, `nodes=6000` vs 2000, final 0.521 [0.505, 0.536]. Note that sweep baselines run at the default `nodes=2000`, where starvation is worse than in these 16k games.

### P2. The mean over k=4 worlds dilutes opponent-lethal tails; worlds are redrawn at every sub-decision

**What the bot does.** On irreversible choices, mostly Garodeth's mode, the root mean over 4 worlds either samples no world where the opponent has lethal, or averages one -80 against race worlds. Those race worlds look good because the passive greedy reply undervalues the human (P3). Worlds are also redrawn between Play and Choose, so one card play can be decided on inconsistent hands.

**Instances:**

- **2560m t8 [68] (decisive).**
  - Live: Storm -11.7 with 0 of 4 worlds lethal. At the very next sub-decision [69], every candidate was -80 (4 of 4).
  - k=4 seed 1: Storm -13.8 (worlds -30 -5 -4 -16) vs Ward -23.8 (-31 +3 -53 -14).
  - k=16 seed 1: Storm -29.2 with 3/16 worlds at -80, vs Ward -32.3 (never lethal, worst -54.3). Still picks Storm.
  - k=16 seed 2: Ward -16.8 vs Storm -37.5, with Storm lethal in 5/16 worlds. So even k=16 flips by seed.
  - The true chance that a Garodeth is among the ~5 hidden cards is about 43%.
  - Applying pess=0.5 to the k=16 seed-1 printout (computed offline, not a separate run) gives Ward (-43.3 vs -54.6). At k=4 seed 1, pess alone does not flip it.
  - Reference cost is only 9.6, and the reference rates the Ward position at -32.5. So Ward was a live game, not a win.
- **10951 t8 [70] (decisive).**
  - Seed 1: Storm -28.7 (worlds -53 +3 +15 -80) vs Ward -35.8 (-43 -25 -28 -48). Seeds 2 and 3 pick Ward.
  - k=16 seed 1 still picks Storm (-21.3, 3/16 worlds at -80).
  - `pess=0.5` seed 1 flips to Ward (-41.8 vs -54.3).
  - The +3 and +15 worlds come from modelled replies like `Leafshadow, attack, evolve, Moelle, end` at 9 PP.
  - Real outcome: Setus destroyed Garodeth, then 17 damage into 16 HP. Reference cost 11.3.
  - **Second chance at [71] (checker-found):** valued at -72.5, the bot still sent Garodeth face. Trading into Miroku would have capped the real hand's damage at about 12. Helper: face -28.7 vs trade -58.1. At k=16, face is lethal in 4/16 worlds and the trade in 3/16, so the sampled worlds treat the lethal as nearly independent of which follower dies.
- **14567 t8 [74] (decisive; attribution W).**
  - k=4 picks Istyndet in 3 of 4 samples: live bot_value -6.8; seed 2 -3.6 vs Garodeth -12.0; seed 3 -4.4 vs Garodeth -7.9 / -11.1.
  - k=16 seed 1: Garodeth +5.4 (worst -1.0) vs Istyndet -8.4 (worst -19).
  - This is a consistent k=4 preference rather than a coin flip. It is mixed with P4 (the two Garodeth copies differ by 3.2 on the same worlds) and P1 (the Garodeth line ends `[end cap]`), and the flat k=16 Garodeth values may themselves be cap-truncated.
  - Board check: Garodeth's 8 damage kills the human's 6/4 super-evolved Istyndet, both Raz and the NL. Reference cost 34.2.
- **When every world says the game is lost, the bot plays the first legal move.** At 2560m [69], with all worlds at -80, the strict `>` kept the first legal action (the un-evolved Garodeth attacked Itsurugi and died). Helper at [69]: super-evolve Garodeth then attack -13.8, vs -50.4 for the recorded move. No effect on that game.
- **Within-turn value jumps with no new information:**
  - 10951: -14.3 at [70] to -72.5 at [71]
  - 2557: -27.6 at [72] to +7.8 at [73]
  - 11162: -4.2 at [90] to +14.1 at [91]
  - 2560f: -15.6 at [89] to -42.5 at [90]

**Prior results (older engines, `nodes=2000`).** Blanket knobs were neutral: sweep7 `pess=0.5` 0.496 [0.466, 0.527]; `k=8` 0.474-0.501 in sweeps 6, 8 and 8b. Do not re-sweep blanket pess or k.

**Cheap tests:**

- **(a) Probes.** Run `replay_at.py` at 2560m [68], 10951 [70], 14567 [74] and 6711 [106] with `h0:nodes=16000,pess=0.25`, `pess=0.5` and `k=16`, seeds 1-3. Expected if P2 is right: pess flips 2560m and 10951 to Ward and leaves 6711 wrong or worse. That confirms pess only helps after P1.
- **(b) Code flags.**
  - Lethal-conditional pessimism or a veto: apply pess only when some candidate has at least one opp_lethal world and another candidate has none.
  - Common worlds across all sub-decisions of one card play (Play, Choose, target), or across a whole turn.
  - When all worlds are -80, re-sample or rank the moves while ignoring the opponent's lethal.

  Sweep against h0 and check with `lethal_audit --by-deficit`.
  - **Confirms:** handed_lethal down at equal deficit and rate at least 0.5.
  - **Kills:** no change.
- **(c) No-code diagnostic.** In explain records from bot-vs-bot games, count decisions where the chosen candidate has an opp_lethal world while an alternative has none. Then check the realized loss rate in those games.

### P3. The greedy opponent model under-plays the human (lethal sweep is face-only)

**What the bot does.** At odepth=0, `opponent_reply` has two parts:

- **`opp_lethal_sweep`.** It tries face attacks only, or one Play (choices resolved in index order) plus at most one evolve, then face. It never attacks a Ward first, never tries two plays and never tries super-evolve buff lines.
- **`greedy_until_end`.** A one-step argmax of the value net, up to osteps=6 with a hard stop at 9, where EndTurn is one of the options.

As a result, the modelled replies are passive:

- `H: end_turn`
- `play Great Hart | end_turn`
- `Bat, two attacks` spending 1 of 8 PP

**Evidence:**

- **6482 [97] (decisive, together with P1).**
  - Ending the turn with the zombie Ward intact scored -44.0, worst world -55.3. The expected reply was `H: attack ward with Adahime … play Garodeth … end_turn`.
  - The candidates that removed the Ward themselves showed -80.
  - Real: NL broke the Ward, then storm Garodeth plus evolve (10) and Adahime (9) made exactly 19.
  - `odepth=2,obeam=6` still finds no lethal (-52 to -53).
- **11837 [38]/[39] (W: no better move shown).**
  - Super-evolved VC +41.8, worst world +31.7, expected reply `H: end_turn`. At k=16 all 16 worlds were positive.
  - Real: 11 face damage through a 7/9 Ward, via a Lilith chip, Bat, and super-evolved Adahime giving +2/+2. Reference at t7H -51.
- **2560m [55].**
  - Default +13.8 over 16 worlds (worst +4.9); expected `Bat, two attacks, end`.
  - `odepth=6` gives -47.9 and `odepth=2` gives -33, against a reference of -51.1.
  - Real: a super-evolved Itsurugi 9/8 killed the super-evolved Adahime 8/7 for free.
- **7210 [59].** The modelled reply spends 0-1 of 8 PP. Real: Garodeth storm plus super-evolve plus attacks for 14 (17 to 3 HP).
- **Great Hart left standing in the model.** At 13238 [100]/[104] and 6711 [111], the best-world reply is `H: end_turn` with an 8-9 attack Great Hart ready. Likely cause: EndTurn credits Great Hart's end-of-turn split immediately, so it beats "attack face" in a one-step comparison. Real: Setus plus Great Hart was lethal both times.
- **Forest combo turns.**
  - 11162 [60]: +42 to +48 in all worlds, expected `Great Hart, end turn`. Real: a Magachiyo Combo-3 wipe.
  - At 11162 [94], h0 playing the human's own seat values its 9-action turn at about +1. Its lines end at depth before any attack; the real swing was to -68.
- **In the bot's wins the model is miscalibrated, but no move changed:** 3883 [84] +52.9 to +22.1 at odepth=2; 9038 [40] 32.7 to 13.9; 5883 [50] 47.9 to 39.7.

**Prior results.** `odepth=1` and `odepth=2` lost badly at `nodes=2000`: sweep5 0.366 / 0.373, sweep9 0.344. `odepth≥1` stops in the middle of the human's turn and scores a human-to-move leaf. At 16k it also starves the bot's own search (7210 [59], 11162 [60] and 11472 [40] all ended at the cap). Do not re-sweep odepth.

**Cheap tests:**

- **(a) Sweep recall, existing tools.** Take the `py/lethal_audit.py` handed_lethal records (EndTurns where `forced_lethal` finds the opponent has lethal). Run `bot_action_explain` at each and compute recall: the share where the bot's worst world was -80. Break the misses down by the shape of the kill: Ward removal first, two or more plays, super-evolve or evolve buff, removal spell.
  - **Confirms:** low recall, with the misses dominated by Ward-first or multi-play shapes.
- **(b) Passivity metric.** Compare the PP spent in the modelled best-world reply (the explain "expected line") with the PP actually spent on the next opponent turn in bot-vs-bot games.
  - **Confirms:** modelled PP far below actual, especially for 7-8 cost cards with a mode choice.
- **(c) Flags, each swept against h0 and re-checked with (a):**
  - Ward-first sweep: kill the Ward(s) with the cheapest sufficient attackers, then run the face line.
  - Sweep with two plays.
  - Greedy check: when EndTurn wins `greedy_index`, compare it first with "all ready attackers face, then EndTurn".
  - Treat Play plus its best Choose as one greedy macro-step.

  Probe expectations: 6482 [97] shows -80 for ending the turn with the Ward intact, and 13238 [104] and 6711 [111] stop predicting `H: end_turn`.

### P4. Duplicate root candidates (checker-found; code-verified; still in HEAD)

**What happens.** The root candidate list is every legal action, filtered only by `useful_action`. Identical copies in hand and identical token attackers become separate candidates, each with its own budget share and its own noisy score. Taking the max over those noisy duplicates biases the top value upward and wastes budget exactly where the cap already bites.

**Instances:**

- 14567 [74]: Garodeth twice (-1.0 / -4.3 in explain; -7.9 / -11.1 at seed 3 on the same worlds).
- 14567 [65], reference run: Istyndet twice (-2.3 / -6.7).
- 5146 [56]: "Skeleton into Bibatii" twice.
- 6482 [93]/[94]: every zombie target twice. The chosen RZ into NL beat its own twin by more than it beat RZ into Adahime.
- Reference noise from the same cause: 3883 [84] Istyndet copies differ by 11.5; 6711 [107] identical Bat plays score +28.2 vs -30.3.

**Test.** Count duplicates per decision, and the budget share they consume, over a bot-vs-bot explain sample. Add a dedupe flag (by card id + target + mode, or by the state hash after applying the action), sweep it, and probe 6482 [94] and 14567 [74].

- **Kills:** duplicates are rare outside zombie/token turns and the sweep is neutral.

### P5. The value net over-credits unspent PP (hypothesis from weights, not instrumented)

**The weights (h0-linear-v1).**

| Feature | Pre-tanh effect |
|---|---|
| Own PP | 0.1843 over a feature std of 2.526 = +0.073 per PP (about 3-4 value points per PP at scale 60) |
| Own leader HP | +0.060 per HP |
| Opponent PP | +0.0155 per PP |

So one PP of your own is worth more than one point of your own leader's HP, while the opponent's PP barely counts.

**Suggested consequences:**

- The greedy reply hoards PP. An 8-drop starts about 0.58 behind before its effect counts, and a card with a mode choice sits in a pending-choice state with its PP already spent.
- Truncated own-turn leaves get credited for PP the bot never spent.
- Leaves capped right after the bot's EndTurn value the human's refilled turn as nearly free.
- It contributes to the 30-68-point swing between consecutive half-turns seen in every value trace, whichever side is ahead.

**Test.**
1. At 2560m [55], log `greedy_index` scores in a world whose hand holds Itsurugi or Garodeth.
2. Evaluate the net on paired states: the same board at EndTurn with PP spent vs unspent.
3. If confirmed, the v2 retrain (needed anyway for the charge flags) should zero or down-weight unspent PP at end of turn. Alternatively, have greedy score states after a forced EndTurn.

- **Kills:** the 8-drop loses in `greedy_index` for reasons other than PP.

### P6. Bonus PP (no game lost to it; mostly fixed at HEAD)

**What happened.** In all 13 bot-second games the bot toggled the early charge on turn 1 and the late charge on turn 6.

- **Early charge wasted, nothing cast (6 games):** 13692 [6], 2560m [4], 10951 [5], 11837 [4], 2560f [4], 6482 [5].
- **Late charge wasted:** 10951 [45], 11162 [61], 13692 [49], 7075 [48], 2560m [41] (W).

**Mechanism on a45e2bb:**

- The v1 encoding has only `bonus_pp.active`.
- `commit_bonus_pp` consumed a charge that was active or locked.
- BonusPp is listed before attacks and EndTurn, and the root uses a strict `>`, so exact ties go to toggling. Example: at 10951 [5], toggling and ending the turn both show +5 +12 +23 +9 across worlds.

**Cost.** No loss is attributable to it.

- At 13692, skipping [49] and asking the bot at [62] (`cf_skip.py`, 16k): `consensus_lethal` finds toggle, then I&T mode 0, in 161 nodes. The burn cost a forced t7 win; the bot won on t8 anyway.
- At 7075 [48], the burn stranded four 8-drops at 7 PP on t7. The bot won that game too.

**Already at HEAD b5b822c (PR #84):**

- 2164e13: an unspent orb is no longer consumed, so the pure burns become no-ops.
- 90197dc: v2 charge flags in the encoding. No v2 net is trained yet, so the built-in v1 net is still blind to charges.
- 295ab6c: `bpp1` / `bpp2` / `bppv` knobs, off by default.
- sweep14 is running `bpp1=4`, `bpp1=3`, `bppv=2.4` and `bppv=6`.

**Open question.** In 7 of 13 games the turn-1 toggle did pay for a 1-2 drop: 5146 [3] Raz, 11162 Lilith DC, 2557 [4] NL, 5883 [5] Lilith DC, 7075 World of Games, 14567 [4] Lilith DC, 7210 Raz. None is shown to be wrong (5146, 14567 and 5883 are all W). The owner, going second, saved the early charge until t4-t5 (3883 [29], 9038 [34]). In the 13692 counterfactual, a saved charge enabled a t4 Highwire plus evolve worth +24 to +45 by the bot's own values (seed-dependent).

**Test.** sweep14 decides. Also read its per-deck and second-seat rows.

### P7. Leaf blind to crest and deck buffs (W; low priority)

The encoder stores crests only as count, countdown and a faith flag. Zone bonuses cover only the bot's own deck.

- **Opponent crest.** The human's Thestae crest was evolved on t5 in all four losses in the first Forest batch (10951, 11162, 2557, 2560f). It makes late Forest followers +2 to +3 over printed stats. At 2560f [103], Setus 7/9 survived Garodeth's 8 AoE.
- **Own crest.** The bot's own Istyndet crest was ignored at 2557 [55] and [57].

No bot decision error is demonstrated. Revisit if a v2 retrain adds opponent deck-buff features.

### Probe set (known better answers; frozen engine)

| Position | Played | Better | Evidence |
|---|---|---|---|
| 6711 [106] | Garodeth Storm | Ward mode | depth=10: Ward +28.5 vs Storm -26.4 |
| 6482 [94] | RZ into NL | RZ into Adahime | 200k: -16.9 vs -19.8; reference cost 52 |
| 7210 [59] | Lilith / cheap line | VC line (super-evolved to 7/9) | 64k: -11.2 vs -35.7 |
| 2557 [57] | Hark | Istyndet #2 | reference: +5.2 vs -21.3 |
| 2560m [39] | Fickle | Bonus PP into Adahime or Istyndet | 64k: +34.7 vs +5.2 |
| 2560m [68] | Storm | Ward | k=16 seed 2: -16.8 vs -37.5 |
| 10951 [70] | Storm | Ward | pess=0.5: -41.8 vs -54.3 |
| 14567 [74] | Istyndet | Garodeth Ward + AoE | k=16: +5.4 vs -8.4 |

**Controls (the played move was right; a fix should not flip them):**

- 11162 [79]: attacking Virid +3.7 vs Magachiyo -11.7.
- 11162 [88]: the Itsurugi, evolve, Void Colonel line.

**Usage.** Run each candidate spec at seeds 1-3 and score the number of correct picks.

**Caveat on HEAD.** 2560m, 10951 and 6482 had Bonus PP burns earlier in the game. Replaying them on a HEAD engine leaves those charges unconsumed, so the positions differ. Probe on the frozen engine, or accept the divergence.

## 3. Per-game one-liners

Bot deck is always Abyss Midrange. "Seat" is who went first.

| Game | Bot | Human deck | Seat | Decisive bot error (or human error in bot wins) |
|---|---|---|---|---|
| 14567320158693562650-5ce21003 | L | Abyss (mirror) | human first | t8 [74]: Istyndet instead of Garodeth Ward + 8 AoE (P2/P1/P4) |
| 5146842333902506308-5ce21003 | L | Abyss (mirror) | human first | None confirmed. t7 [56] trade vs Adahime-first all-face (W): race line +28.6 vs +19.1 at depth 10, k=16, but storm is still lethal at 3 HP unless the Adahime summon is Lilith DC (about 1/4-1/3). The dead t5 (0 of 5 PP) contributed |
| 6482272190127595542-5ce21003 | L | Abyss (mirror) | human first | t10 [94]: zombie damage spread instead of killing Adahime (P1 starvation, P4), then [97] Ward-break lethal not modelled (P3) |
| 11837117393133213561-5ce21003 | L | Abyss (mirror) | human first | None demonstrated. t6 [38]/[39] valued +37 (16/16 worlds positive) into the t7H super-evolved Adahime burst, 11 through a 7/9 Ward (P3) |
| 7210033081212776899-5ce21003 | L | Abyss (mirror) | human first | t7 [59]: cheap plays plus last SEP on Raz, no Void Colonel Ward before the human's 8-PP turn (P1) |
| 2560486491816220020-5ce21003 | L | Abyss (mirror) | human first | t8 [68]: Garodeth Storm instead of Ward (P2). Earlier, t6 [39]-[41] squandered a +34 lead (P1, major) |
| 7075938339537277432-5ce21003 | W | Abyss (mirror) | human first | Human error: t9 [73] missed a forced win (reference cost 27.8); t10 [87]/[91] (59.4 / 48.6). Bot's own t6 [48] Bonus PP burn stranded its 8-drops on t7 |
| 10951419422900500402-c9e8a9fb | L | Forest Combo | human first | t8 [70]: Garodeth Storm instead of Ward (P2), then [71] face instead of a trade |
| 11162066908668332800-c9e8a9fb | L | Forest Combo | human first | None demonstrated. t7H Magachiyo Combo-3 wipe (+39 to +1) and t9H Thestae -0/-5 plus immune super-evolved Thestae (+10 to -68) never modelled (P3) |
| 2557551841517808989-c9e8a9fb | L | Forest Combo | human first | t7 [57]: Hark instead of Istyndet #2; its own crest would have killed the Hart for free (P1, major) |
| 2560486491816220020-c9e8a9fb | L | Forest Combo | human first | None demonstrated. Two super-evolved Adahime boards wiped (t7H, t8H); crest-buffed Setus 7/9 survived Garodeth's 8 AoE at [103] (P3/P7) |
| 11472704200738364811-c9e8a9fb | L | Forest Combo | bot first | None demonstrated. t6H super-evolved Great Hart plus its 8-damage split wiped the board (bot +43.1 vs reference -12); t7H 10-action combo turn (P3) |
| 13238162772055293923-c9e8a9fb | L | Forest Combo | bot first | None confirmed. t10 [99] truncated lines hid the Setus lethal (W; every line loses to the real hand); t9 [88] Macmillan into the combo wipe (W) |
| 6711863184032091189-c9e8a9fb | L | Forest Combo | bot first | t10 [106]: Garodeth Storm instead of Ward; the Storm line is a flat +28.2 ending at depth (P1) |
| 13692073994695065052-c9e8a9fb | W | Forest Combo | human first | Human error: [34] Thestae evolve (reference cost 14.3). Bot's [49] late Bonus PP burn cost a forced t7 win |
| 3883134982689280309-c9e8a9fb | W | Forest Combo | bot first | Human error: Fairy plays [16] (25.4) and [27] (24.6). Bot led throughout |
| 5883843333739085459-c9e8a9fb | W | Forest Combo | human first | No flagged human error. The human's hand clogged (2 Great Hart + 6 Deepwood Bounty) |
| 9038916459751691349-c9e8a9fb | W | Forest Combo | bot first | Human error: t7 Sathanid plays [61] (24.0) and [62] (12.9). Bot won the race by one turn at 6 HP (t13 [145]) |

Record by seat: bot second 3-10, bot first 2-3. Of the 13 losses, 7 have a confirmed decisive or major bot error, 2 have a weakened candidate, and 4 show no demonstrated bot move error.

## 4. What the bot did well

- **Highwire Feline fanfare plus evolve** as double removal on turn 5: 13692, 5883, 9038, 11162, 10951.
- **Istyndet super-evolve crest** as free, recurring removal. It killed a lone super-evolved Great Hart 8/8 at 5883 (end of t7) and 9038 (end of t9), and removed Lyria through Barrier at 2557 t6.
- **Resets at low HP:**
  - 5146 t6 at 9 HP: Istyndet plus super-evolved Raz cleared 5 followers.
  - Itsurugi resets at 2560f t8 and 6711 t8.
  - Itsurugi plus evolve for +2 PP plus Void Colonel in one turn: 9038 t8, 11162 t8, 13238 t8.
- **Correct Garodeth Ward or wipe mode** when it chose it: 7075 T8 and 5883 t8, both won.
- **Super-evolve immunity lines:** 10951 t7 (Raz chip, then super-evolved Adahime into the 9/9 Hart); 6711 t9 (super-evolved Adahime plus Hark through a Ward).
- **Macmillan chains:** about 6 face pings per cast.
- **Lethal finding:** `consensus_lethal` found every final kill (13692, 3883, 5883, 7075, 9038) and would have found the 13692 t7 kill. A "lethal pre-check too shallow" pattern was proposed and refuted.

## 5. Shared blind spots (swings neither the bot nor the reference foresaw)

The reference uses the same net, the same greedy opponent model and the same depth 6, so it misses these too. Human plays the opponent model does not generate:

- **Ward removal before face damage.**
  - 6482 t11H: NL broke the zombie Ward, then storm Garodeth 10 plus Adahime 9 made exactly 19.
  - 11837 t7H: a Lilith DC chip, Bat, and super-evolved Adahime (+2/+2) broke a 7/9 Ward for 11 face.
  - 11472 t7H: a bane Fairy killed the super-evolved Colonel through its Ward.
  - Setus destroying the only Ward or blocker, then storm damage, finished 10951, 11472, 13238, 6711 and 2560f.
- **Multi-card Combo-3 setups** (enablers first, payoff later).
  - Magachiyo AoE: 10951 t8H, 11162 t7H, 13238 t9H (after three Deepwood Bounty), 13692 t7H, 3883 t7H/t9H, 9038 t7H.
  - Leafshadow bane Fairy: 2560f t8H, 11472 t7H.
  - Combo-storm Fencer: 11472 t7H.
- **Super-evolve as the payoff.**
  - Super-evolved Itsurugi 9/8 killing super-evolved Adahime 8/7 for free (2560m T8).
  - Garodeth Storm plus super-evolve for 14 (7210 T8).
  - Super-evolved Great Hart plus its end-of-turn 8 split: at 11472 t6H it killed four bodies (1+3+3+1) exactly; also 2560f t7H, 3883 t8H, 5883 t7H.
  - Super-evolved Thestae, immune on its own turn, after -0/-5 on Itsurugi (11162 t9H).
  - Super-evolved Virid (11162 t7H, 2557 t9H) and super-evolved Sathanid drain (6711 t7H).
- **7-8-cost cards with a mode choice** played at full PP: Garodeth Ward + AoE + 6 face (7075 T8); Itsurugi (2560m T8). The human's Macmillan chain at 14567 t10H was also unmodelled (optimism +21.3).
- **Greedy ends the turn instead of swinging** with a ready Great Hart: 13238 [100]/[104], 6711 [111].
- **Removal and PP tricks.** Crimson Incense on the best body (3883 [39], 9038 [47]); Thestae -0/-X; a 12-action turn built on Miroku PP recovery (9038 t6H).
- **The mirror's turn-5 play.** In all 4 games of the first mirror batch, the human's t5 Highwire Feline plus evolve wiped the bot's t4 development. The bot had spent an EP on a small t4 body each time (14567 [28], 5146 [25], 6482 [23], 11837 [21]) and expected `play Void Colonel` in reply (e.g. 6482 [26], optimism +41).
- **Opponent deck growth** from the Thestae crest (Setus 7/9, 2560f [103]). Nothing in the leaf values it.

## 6. Questions for the owner

1. **Garodeth Ward mode in four losses** (14567 t8, 2560m T8, 10951 T8, 6711 T10). If the bot had chosen Ward + 8 damage to all instead, what was your plan, and did you still have outs? For example: Setus plus Virid in 10951; Miroku plus the drawn Setus in 6711; Istyndet #2 or Macmillan in 14567. *Tells us how many games P1/P2 fixes would actually flip.*
2. **11837 t7 [42]-[48].** Was the line Lilith chip, Bat, super-evolved Adahime, break the 7/9 Ward, 11 face obvious to you? Could any bot t6 setup have stopped it? *Separates "the model's blind spot cost the game" from "unavoidable".*
3. **6482 t11.** Was breaking the Ward with NL and then 10 + 9 = 19 routine? Would three zombie hits killing Adahime have taken lethal off the table? *Validates the Ward-first sweep and the [94] target claim.*
4. **7210 T7 to T8.** Against a super-evolved Void Colonel 7/9 Ward, would you still have gone Garodeth storm, or switched to Itsurugi or Hark burn? *Checks whether [59] was really decisive.*
5. **Forest combo turns** (11472 t7 [56]-[67]; 11162 t9; 9038 t6 [43]-[55]). How far ahead do you plan combo count and PP? Which 2-3 ideas must an opponent model at least try: Ward-first, enablers before payoff, super-evolve as payoff, removal on the biggest body? *Sets the priority for extending the sweep and greedy reply.*
6. **Holding AoE.** Do you hold Magachiyo combo, Great Hart's split or Miroku splits until the bot goes wide? Would a bot that kept 1-2 bodies back (11472 t6 Baal/Lilith; 13238 t9 holding Macmillan's three Ward zombies against a single Setus destroy) have been harder? *The overextension pattern is weak in the data; your view decides whether it is worth a term.*
7. **Bonus PP going second.** When do you spend each charge? Is any turn-1 use correct, or is it always turn-4 Highwire / turn-5 Adahime, with the late charge saved for an 8-cost turn? *Sets the prior for the sweep14 `bpp1`/`bppv` result.*
8. **Your flagged mistakes in the bot's wins:** 7075 T9 [73] ("forced win"), 3883 [16]/[27], 9038 [61]/[62], 13692 [34]. Real mistakes or reference artifacts? *Tells us whether the 5 wins say anything about bot strength.*

## 7. Caveats

- **Sample.** 18 games against one strong human, two matchups, and the bot always on Abyss Midrange. The human went first in all 7 mirrors, so the bot was second in 13 of 18 games and second-seat issues (Bonus PP) are over-represented. Each pattern rests on 2-5 decisive instances.
- **Engine version.** The games ran on a45e2bb. HEAD b5b822c changes the Bonus PP rule, the encoding (v2) and the `bpp` knobs. The helpers use a frozen pre-fix `arena.pyd`, which is valid for these games but diverges on HEAD wherever a charge was burned earlier.
- **Recorded values.**
  - The recorded bot_value sometimes disagrees with explain; prefer explain.
  - Explain re-runs use a different seed from the live game, so many "re-run would choose" differences are seed noise.
  - Most helper experiments are single-seed. Even k=16 flips by seed at 2560m [68].
  - k=16 runs at 16k nodes are budget-starved.
  - lcap caps the lethal pre-check at `min(nodes + 0.5*cap, cap)`; it does not reserve half the budget. Explain runs used about 14.4k of 16k nodes.
- **The reference** (200k nodes, k=16, depth 6) shares P1, P3 and the net:
  - Its "<<< MISTAKE" flags include self-comparisons (same action label, or a duplicate candidate).
  - The decisive 6711 [106] went unflagged (cost 0.4).
  - Treat costs below about 25 as unreliable, and higher costs as unreliable too when the compared lines end in different ways.
  - Values swing 30-68 points between the bot's and the human's half-turns in every trace, whichever side is ahead. "Optimism" gaps under about 40 are not surprises; compare same-side values.
  - The human's flagged mistakes are subject to the same artifacts.
- **Hindsight.** "Better" lines are checked against the real hidden hand, which the bot could not see. Several still lose: 13238 [99], 2560m [68] (reference -32.5 for Ward), 10951 [70] (reference -37.5 for Ward).
- **Sweep context.** Prior sweeps used `nodes=2000` baselines on older engines. The blanket results (pess neutral, k=8 neutral or slightly negative, odepth=1/2 about 0.35) may not transfer to 16k, and P1 starvation is worse at 2k.
- **Dropped as refuted (do not chase):**
  - 7210 [78] (every move lost; the real decision was [76]).
  - 11162 [88] and [79] (the played moves were right).
  - 6711 [53] (super-evolving Fickle was fine).
  - 9038 [40] (a tie).
  - "Lethal pre-check too shallow" (16k finds the 13692 t7 kill).
  - "EP/SE wasted on tokens" (evolving is limited to once per turn, and the bot evolved every late turn).
- **Weakened patterns, not ranked:** Great Hart disrespect (a restatement of P1 and P3 with hindsight), overextension into AoE (gaps 3-10, no outcome change shown), per-card mulligan table (7210 [1], thin evidence).