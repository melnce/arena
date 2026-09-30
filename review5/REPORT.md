# Cheater games vs the owner (2026-09-30): is it the search or the imagined hand?

**Setup.** 21 games, all Abyss Midrange mirror. Owner = seat a, bot = seat b. Bot spec `h0:nodes=16000,horizon=3,info=all` with leaf h0-linear-v2, engine 2d12e25. Record: owner 13, cheater 8. The cheater went first in 16 of 21.

**Conventions.**
- Games are named by the first four digits of their ID; the full IDs are in the appendix.
- `[N]` is an action index in the timeline. The position is the one before action N.
- (W) means the checker weakened the analyst's claim, and the text gives the checked reading. Refuted claims are left out.
- [new] marks a helper run made for this report on the current engine. Each is a single seed unless stated, and none was checked independently.

## 0. Owner's context (added after the analysis)

The owner points out that the comparison that matters is the **Abyss mirror only**. In review3 the owner's 4 losses
on Forest Combo came from playing Forest without drawing Thestae (not their strongest deck, into a healing,
sticky Abyss Midrange), and the one review3 mirror loss was a missed obvious lethal on the owner's side. Against the
fair bots the owner considers the mirror essentially a sure win; the cheater taking 8 mirror games is a real change.

| bot in the Abyss mirror vs the owner | bot wins | rate [Wilson 95 %] |
|---|---|---|
| fair bots (review3 1-6, review4 0-5) | 1 / 12 | 0.083 [0.015, 0.354] |
| fair bots, the missed-lethal game set aside | 0 / 11 | 0.000 [0.000, 0.259] |
| **cheater** (this report) | **8 / 21** | **0.381 [0.208, 0.591]** |

One-sided Fisher exact: p = 0.071 as played, p = 0.019 with the blunder game set aside. **Seat confound:** the fair
bots were second in 11 of those 12 games, the cheater first in 16 of 21 (first player wins ≈ 0.55 in bot-vs-bot
play, i.e. ≈ 5 points — far too small to explain a ≈ 30-point gap). Section 1's "5-18 vs 8-13, p ≈ 0.20" pools the
Forest games and so understates the difference in the matchup these 21 games were played in. Note also section 3.5:
`info=all` is a partial cheat (current hand and deck contents, not the draw order; k = 1), so the mirror gap is a
lower bound on what hidden information costs the fair bot against this player.

## 1. Answer first

- **The record.** The cheater went 8-13 (38%). The fair bots went 5-18 combined (22%): 0-5 in review4 and 5-13 in review3.
  - That gap cannot be told apart from chance here. Pooled one-sided Fisher p is about 0.20. The 95% intervals overlap widely (21-59% vs 10-42%).
  - The cheater's 16/21 first-player share is worth only about half an expected win. It went 5-11 going first and 3-2 going second.
- **The 13 losses:**
  - 4 were missed one-turn defences with an escape shown: 6889 [45] and 1537 [75] are solver-verified, 1043 [109] is probable, and 2790 [83] is the fourth.
  - 4 were reply-model misses of an Adahime super-evolve swing. The cheater could see that Adahime in the true hand. 4611 [48] has a likely Hark x2 escape; 5366, 1720 and 6727 have no escape shown.
  - 1 was a k=1 sampling outlier: 1550 [47], where 9 of 10 samples choose the right move.
  - 3 were decided by the owner's fresh draws, with no bot error shown: 1509, 1399, 4040.
  - 1 was lost earlier on board: 5951.
- **No horizon-only failures.** Formally, 11 of 13 are "two-turn plans" (the final turn was unstoppable). But wherever a bot error is shown, the damaging human turn was the very reply the bot modelled, with the true hand in view. The reply model generated the wrong turn: its greedy one-step ranking, its face-only lethal sweep (which never breaks a Ward or frees a board slot), and its step cap.
- **What the knobs fixed.** None of these positions was fixed by more nodes (64k), odepth=2, obeam=12 or osteps=10. An exact kill check in the reply does fix 1537 at olsolve=1000 [new]. At olsolve=4000 it flags 1043's end turn as lethal, but within 16k nodes it cannot vet the defending lines [new]. It does not fix 6889 or 2790.
- **The only missed own kill** is 1740 [75], a game the cheater won. The deterministic line is super-evolve, then the Baal buff, then attacks. The own-turn search never puts those actions in that order at any tested seed, budget, beam or depth. It cost nothing.
- **Verdict.** For every loss with a demonstrated bot error, the cause is search, not the imagined hand. But `info=all` is only a partial cheat: it sees the current hand, not draws or random outcomes; the leaf still masks the hand; and it runs k=1 instead of 4 worlds. So this test puts a bound on the imagined-hand cost rather than measuring it.

## 2. Loss causes (13)

| Game | First | Cause class | Mechanism | Decisive [N] | Escape-check evidence |
|---|---|---|---|---|---|
| 6889 | bot | Missed one-turn defence (solver-verified) | SE Adahime kill from the true hand: +2/+2 to Raz and two Skeletons; the SE Adahime hits a follower and pings the leader; 1+5+3+3 = 12. The sweep drops the Adahime play. The greedy reply at [42] did play Adahime + SE as steps 5-6, then hit osteps=6 before the buffed attacks. Lilith's Strike at [43] cost the 13th HP. | Bot t6 [43]-[45]; live ended turn at [45] (+5.1) | Adahime 7/6→Raz at [45], then end turn: solver `none` on the true RNG and in 12/12 reseeds. Plain end turn at 13 HP: `none`. Recorded line: `unknown`. The re-run's [44] Lt→Raz is not a defence (12/12 lethal). |
| 1537 | bot | Missed one-turn defence (solver-verified) (W: the moment is [75], not [74]) | After Macmillan, three Rush+Ward zombies make every human face attack illegal, so the sweep never fires. The real kill breaks the Wards (Baal, Lt, Bat+evolve), then Garodeth Storm 8. Live ended the turn (+21.7); the re-run would attack (+3.3). | Bot t9 [75] | Zombie→Baal, Zombie→Lt, end turn at [75]: solver `none` (complete, 2,613 nodes). Macmillan + end turn: `lethal`. Adahime-first: `none`, but RNG-dependent (needs the Lilith→Bat summon). t10: all lethal. |
| 1043 | bot | Missed one-turn defence (probable) | The human's board was full (5/5), so no Play was legal at the start of their turn ([115]: 27 legal actions, none a Play). Neither sweep nor greedy suicides a follower to free a slot for Garodeth Storm, which was in the true hand at [109]. The only -80 lines were the bot itself killing Void Colonel. Kill: 8+1+1+3 = 13. | Bot t11 [109]-[114]; ended turn at +37.7 | Recorded: `lethal`; 7 alternatives `unknown`. A hand-built defence from [109] caps the human at 11 vs 13 HP; solver `unknown` at 200k. t10: all `none`. |
| 2790 | bot | Missed one-turn defence; the human missed the kill, bot lost a turn later | Macmillan let the human break a zombie Ward, then Garodeth Storm + Hark (solver line at [90]; needs Hark's random split). The human missed it but took the bot 11→4 with zombie trades and pings that the reply model did not generate (W). At [99]-[100] the bot chased a sampled +80 with Garodeth Storm mode and paid 2 HP. | Bot t9 [83] (Macmillan, live +7.2) | Macmillan: `lethal` (rng-dependent). Garodeth mode 1 + crystallised Void Colonel: `none` (bot 11, human 12, human board 2). t10: all lethal. |
| 4611 | human | Reply-model miss (SE swing); likely escape | [54] modelled "play Bat, end turn" (+6.8). The human played the visible Adahime + SE, traded (+1 ping) and took the bot 14→4. The bot drained back to 8; kill at [70] = Raz 3 + Garodeth Storm 8. The leaf-blindness part of the claim is (W). | Bot t6 [48] (Adahime line over Hark x2) | Hark, Hark, bonus PP: `none` (both boards empty, bot 15, human 12, Garodeth costs 8 vs 7 PP; RNG-free, immediate turn only). Recorded: `unknown`. t7: all lethal. |
| 5366 | bot | Reply-model miss (SE swing); no escape shown (W) | [49] modelled Bat/Baal; actual Adahime + SE [51]-[57], bot 14→5 facing a lethal board. The human held one Adahime at [44]; the second was drawn on turn 6. Live also diverged from the re-run at [45]/[46]. | Bot t6 [44] (Fickle vs Hark-first, both +26.5) | Hark-first: `none`, but only at 14 HP, which says little. Recorded: `unknown`. t7: all 7 lethal. |
| 1720 | bot | Reply-model miss (SE swing); no escape shown (W) | [46] modelled "attack, Bat, Lilith, attack, bonus PP"; actual Adahime + SE [47]-[52], bot 18→7. With the threat modelled (odepth=2, obeam=12), the whole position is about -22 and the move is unchanged. | Bot t6 [41]-[46] | Lt-first and end turn: `none` at 18 HP (says little). Recorded: `unknown`. t7: all lethal. |
| 6727 | bot | Reply-model misses (SE swing t7, slot-freeing t9); escape unproven (W) | [47]/[48] modelled "Hark, Bat, Lilith, bonus PP" (Lilith was sampled, not in hand) at +49; actual Adahime + Bat + SE took the value to -22. At [76] the modelled end-turn reply played Macmillan into a near-full board (one zombie, bot at about 6-7 HP). The human first traded both Skeletons, freeing slots for three zombies (13 damage, bot to 2). | Bot t7 [44]-[48]; bot t9 [76] (seed-dependent: end turn vs Bibatii→Feline 7/4) | t9: recorded `none` at 200k (the escape check said `unknown`); all 6 served options `none`. t10: all lethal. Whether killing the Feline survives t10 is untested. |
| 1550 | bot | k=1 sampling outlier; escape untested (W) | The live single-world sample valued Istyndet at +19.7. 9 of 10 other samples choose Garodeth mode 1 (Ward, 8 to all enemy followers, wiping SE Baal, Adahime, Raz and Lilith), by 22-46 points. The human's Adahime + Lilith + Bat + SE took the bot 7→2. The t8 Ward-break kill was valued -28..-58, not -80. | Bot t7 [47] | Garodeth-first: `none` (bot 7, human board 0). Recorded Istyndet: `unknown`, and the human did not kill on t7, so the check does not separate the two lines. t8: all 8 lethal. |
| 1509 | bot | Human draw plus an unavoidable Storm; no bot error shown (W) | Garodeth Storm 8 to face at [72]-[73] could not be stopped (no Ward in hand). The model's mode-1 prediction at [70] has no demonstrated cost. Macmillan was drawn at the start of human t9: three Rush+Ward zombies plus pings left the bot at 8 facing 13 on board. | Human t9 [83] (fresh Macmillan) | t9 [76]: recorded `none`, Baal-first `none`, end turn `lethal` (immediate kill only). t10 [88]: every option lethal. |
| 1399 | bot | Human draw; no bot error shown (W) | Adahime was not in hand at [55]. Depths drew it mid-turn at [61]; Adahime + SE at [62]-[63] turned a fresh Void Colonel into a 6/8 Ward and Lt and Raz into 3/3s, while dealing only 1. The bot had no Ward and could not afford Macmillan at 7 PP. | Human t7 [61]-[63] | t7: recorded `none`, most others `unknown`. t8: all 8 lethal. |
| 4040 | bot | Human draw (random swing) | A sound t7 defence left 6 damage against 10 HP. The human drew a second Adahime: SE +2/+2 on Raz and Bat plus 1 ping = 11. | Human t7 draw, after bot t7 [58]-[64] | t7: all lethal on the true RNG stream. With reseeded human draws: 9/40 lethal (Hark, Baal, Fickle, Adahime), 31/40 `none`. |
| 5951 | human | Lost earlier: board deficit (W) | Highwire's 3+3 killed Baal and Bat on t5, and values were negative from then on. The hand was clunky (2 Istyndet + Garodeth at 5 PP). Human t6 took the bot 16→8. The kill used only the board: VC 6 + Highwire 6 + Skeleton = 13 vs 8. | Bot t5 [43]-[45] (no single error) | t5: recorded, Hark-first and end turn all `none` (immediate turn only, at 16 HP). t6: all 5 lethal. |

**Tally.** 4 missed one-turn defences, 4 reply-model misses of a non-lethal swing, 1 sampling outlier, 3 draws, 1 lost earlier. The bot went first in 11 of the 13 losses.

## 3. Patterns

### 3.1 The reply model misses the Adahime super-evolve swing (W)

**Where it mattered** (Adahime was in the true hand each time):
- 6889: the kill itself.
- 4611: 14→4.
- 5366: 14→5.
- 1720: 18→7.
- 6727: value +49 → -22.

Not instances: 1399 and 4040, where Adahime was drawn that turn.

**Mechanism (checked):**
- **Sweep.** `resolve_play_line` (h0.rs ~2808) only continues after a play that adds a leader attacker or lowers the bot's HP. Adahime's summons have Rush, so the play is dropped. The SE, and the SE Adahime's non-face attack (which pinged the leader for 1 in 6889), are outside `face_line_kills`.
- **Greedy reply.** Each step is ranked by its immediate leaf value, so Bat, Lilith, Hark, Istyndet or end turn outrank "play Adahime", whose value only arrives with the SE and the buffed attacks.
  - At 6889 [42] the greedy did play Adahime + SE as steps 5-6; osteps=6 then forced end turn before the buffed attacks.
  - [new] At 6889 [45] with osteps=10, the end-turn reply still stops by choice: Raz→face, Bat, bonus PP, end turn, leaving 6 PP and Adahime unused. The ranking binds, not only the cap.
- **odepth>0 swaps the model rather than deepening it.** With odepth>0, `opponent_reply` skips the sweep and the greedy completion and scores a static mid-turn leaf. `search_opp_expand` sorts by `eval.value` and truncates to obeam=3, which prunes Adahime at step 1.
- **Position-specific, not structural.** Modelled replies contain "H:play Adahime" 52 times across 13 timelines.

**Experiments:**
- **64k nodes:**
  - 1720 [41] stops at 14,107 nodes with the same reply.
  - 4611 [48] still prefers the Adahime line (+10.7) over Hark x2 (-7.1), using 55k nodes; the modelled reply is Istyndet.
- **odepth=2 (obeam 3):**
  - No Adahime appears in the reply at 5366 [44], 1720 [41] or 4611 [48] (Adahime line -39.0 vs Hark -45.0).
  - At 6889 [45] it picks the defence by only 1.1 (-38.0 vs -39.1), with nothing at -80.
- **odepth=2, obeam=12:** the reply now reaches "H:play Adahime" at 5366 [44] and 1720 [41], but the move does not change.
  - 5366: Fickle -16.4, Hark -29.8.
  - 1720: Adahime -21.5, end turn -21.8, Lt -24.1.
- **olsolve=1000 at 6889 [45]:** still ends turn (+10.7), with Adahime→Raz at -13.3. The check runs out of budget.
- **[new] osteps=10:**
  - 6889 [45] ends turn (+10.7), with Adahime→Raz at -37.1.
  - 4611 [48] unchanged (Adahime +10.7, Hark x2 -7.1).

**Reading.** The miss is real in 5 losses, but its causal share is proven only in 6889 and likely in 4611. In 5366 and 1720, modelling the threat leaves the position at about -16 to -22 with the same move.

**Cheap tests:**
- Replay 6889 [45] and 4611 [48] over seeds 1-5. Pass = the defence is chosen and the losing line scores near -80.
- Try a sweep variant that does not drop a play when an SE on the board would buff attackers: play → SE → all attacks, including one non-face attack.
- Measure with `py/lethal_audit.py --policy <spec> --decks meta-abyss-midrange --by-deficit` (handed_lethal at equal deficit) plus a self-play sweep against the served spec.

### 3.2 Kill shapes the opponent sweep cannot generate: Ward-first and full board (confirmed)

**Instances:**
- 1537 [75]: Ward-first; solver-verified escape.
- 2790 [83]: Ward break + Storm + Hark; RNG-dependent kill, which the human missed.
- 1043 [109]-[114]: full board.
- 6727 [76]: slot-freeing, not lethal. The model played Macmillan into a near-full board and got one zombie; the human traded both Skeletons first and got three.
- 1550 t8: the Ward-break kill was valued -28..-58 instead of -80; the game was already lost.

**Mechanism.**
- `opp_lethal_sweep` (h0.rs ~2920) tries face attacks, then each Play from the unchanged state with its Choose, a face line and (with oevo) one evolve.
- `face_line_kills` only attacks the face, so the sweep never makes a non-face attack.
- Behind Wards no face attack is legal. On a full 5/5 board no Play is legal.
- The greedy reply finds neither the Ward break nor the suicide attack that frees a slot.

**Experiments:**
- **Checked:**
  - 2790 [83], 64k nodes: Macmillan +8.2 using only 18.8k nodes, so the budget does not bind.
  - 2790 [83], olsolve=1000: Macmillan +24.3 (`cap_reply`), Garodeth -35.4. With 64k + olsolve=1000: Macmillan still +24.3.
  - 1043 [109], olsolve=1000: every candidate +35..+43.
- **[new] 1537:**
  - At [74], olsolve=1000 (seed 1) plays Macmillan with the planned line Macmillan | Zombie→Baal | Zombie→(slot 0) | end turn (+39.9).
  - At [75], olsolve=1000 (seed 1) and olsolve=4000 (seeds 1 and 2) score end turn at -80 (`opp_solver`) and choose Zombie→Baal (+39.9).
  - The default spec at [75], seed 1, ends the turn (+24.6), with the reply Macmillan | attack | attack | evolve | end.
  - So the earlier reading that "olsolve=1000 misses 1537 (Macmillan +39.9)" is wrong: +39.9 is the value of Macmillan followed by the escape.
- **[new] 1043 [109], olsolve=4000:** end turn scores -80 (`opp_solver`). Every other candidate ends in `cap_reply` (+7.5..+42.9), so the check never judged them. It picks Zombie→Void Colonel, then Highwire Feline (+42.9), which is unverified. At [114], after the recorded [109]-[113], every option is -80.
- **[new] 2790 [83], olsolve=4000:** still Macmillan +24.3 (`cap_reply`). Garodeth rises to second (+6.2). End turn, Istyndet and Adahime score -80. The kill after Macmillan needs Hark's random split, which the reseeded world need not reproduce.

**Reading.** This is a search-model failure that perfect information cannot cure. An exact check in the reply fixes the clean case (1537). That fits the view that olsolve's self-play Elo cost comes from verdicts in sampled worlds rather than from the check itself; this has not been tested.

**Cheap tests:**
- A 2x2 self-play sweep: served vs served + `olsolve=1000`, each under `info=open` and `info=all`. If olsolve gains under info=all but loses under info=open, the cost is the imagined hand inside the check. Gating the check (for example, trusting only kills that use the board plus known cards) would then be next.
- A sweep extension: before the play/face lines, allow one attack into a Ward and, on a full board, one attack that trades a follower off to free a slot. Probe 1537 [75], 1043 [109], 2790 [83] and 6727 [76].
- At 1043 [109] the pair cap binds (`cap_reply` on every defending line). Check whether a larger node budget lets olsolve vet those lines.

### 3.3 k=1 world with a reseeded RNG: seed-flipped decisions and false certainties (confirmed; W on "random card effects cause misses")

**Decisive:** 1550 [47]. The live sample picked Istyndet (+19.7); 9 of 10 other samples pick Garodeth mode 1.

**Seed-flipped near-ties under a blind reply model:**
- 1537 [75]: live end turn +21.7 vs re-run attack +3.3; [new] default seed 1 ends turn (+24.6).
- 6889 [44]/[45]: live ended the turn; the re-run chose Adahime→Raz.
- 6727 [76]: seeds 1 and 2 end the turn, seed 3 plays Bibatii→Feline. The timeline explain scores end turn at -80 from a sampled draw.

**False certainties:**
- 2790 [99]/[100]: the live value was +80 in the sampled world. The true RNG had no kill; 11/30 reseeds had one, via Baal first. Chasing it cost 2 HP.
- 1740 [75]: seeds 1-4 report +80 for a 50% line.
- 1714 [123]: -80 from a sampled Macmillan the human did not hold.
- 4040 t7: values flip between -80 and -24 depending on whether the sampled draw kills.

**Mechanism.**
- Info::All = `state.clone()` + `rng.reseed(seed)` (determinize.rs 73-76).
- k is forced to 1 under info=all (h0.rs 704-706). The comment there says "every determinization identical", but that premise is false: each root gets its own `rng.next_u64()` (h0.rs 714/721), so k>1 would average future draws and random effects. The cheater gave up the fair bot's k=4 (h0.rs 429).
- Noise size, from the six losses in the second analysis group: over 151 decisions, the live vs re-run value gap for the same action had median 2.9, mean 7.7, and was 10 or more in 17% of cases. Over 157 decisions, the re-run chose a different action at 24.

**Reading.** The hazard is over-trusting coin-flip lines and breaking near-ties at random, not missing kills. It cost at least 1550.

**Cheap test.** Drop the k=1 force for info=all (one line). Then:
- Replay 1550 [47], 1537 [75], 6727 [76] and 1740 [75] with `info=all,k=4` over several seeds.
- Run self-play `info=all` with k=1 vs k=4.

This also gives any future cheater study a cleaner baseline.

### 3.4 Own-turn search orders attacks before buffs: the one missed own kill (W: not caused by randomness)

**Position: 1740 [75].**
- Human: 10 HP, no Ward.
- Bot: Lilith 3/2 rush and Bat 3/2 drain rush on board, SEP 1, Baal in hand, 8 PP.
- Deterministic kill: SE either follower, play Baal mode 0 before any attack, then both followers attack. 6+3+1 = 10, lethal wherever the buff lands.

**What the search did:**
- Live: Istyndet (+53.5).
- Seeds 1-4: +80 via SE Bat→face first, then Baal. That is a 50% line.
- Seed 5: Raz +52.7, no kill.
- nodes=64000, seed 5: +80 via a 1-in-3 line.
- beam=8, seed 5: no kill.
- [new] depth=10, seed 5: Raz +52.7, no kill.

**Mechanism.**
- No run put the buff before the attacks.
- `consensus_lethal` runs at depth 2 (h0.rs 791), too shallow for a 5-action kill.
- Longer own kills were found (4038 [62] Ward-first, 1750 [54], 1740 [89]), so this is a move-ordering bias, not a general blindness.
- It affects the fair bot equally.

**Cheap test.**
- Run `py/lethal_audit.py --policy "h0:nodes=16000,horizon=3" --decks meta-abyss-midrange --games 40` and read `missed_lethal`, split into deterministic and rng-dependent.
- Replay 1740 [75] after any ordering change, for example trying Choose/buff plays before attacks, or a deeper consensus_lethal.

### 3.5 What info=all actually knows (confirmed)

**No deck order.** The cheater sees the current hand and the exact deck contents, but the deck has no order to see: `draw_one` picks at draw time via `rng.pick_index_among` (apply.rs), and the search reseeds the RNG.
- Fresh draws decided 1509 (Macmillan), 1399 (Adahime via Depths), 4040 (Adahime) and the win 1714 (Itsurugi via Depths).
- Modelled replies contain cards the human did not hold: 1509 [80] Istyndet, 1043 [109] Macmillan, 6727 [47] Lilith, 1714 [123] Macmillan.

**The leaf still masks the opponent's hand.** encode gives only hand size and a pool histogram, so the known hand reaches only the one-ply reply model (sweep + greedy).
- Example: 1043 t11 scored +35..+47 at 13 HP against 10 board attack plus Garodeth (Storm 8) in hand.
- Its causal share is not shown (W): 5951's kill used only the board, and in 4611 and 2790 the decisive miss was the next human turn.

**The timeline headers were wrong for this bot** (they claimed knowledge of both deck orders and labelled the human hand "HIDDEN from the bot"); fixed in the published copies under `botview/`.

### 3.6 Not supported by these games

- **Horizon-only two-turn plans (W).** 11 of 13 losses had an unstoppable final turn. But the setups came from:
  - inside the modelled reply: 4611, 5366, 1720, 6727, 1550, 2790;
  - draws: 1509, 1399, 4040;
  - a board deficit: 5951.

  There is only partial support for a leaf problem: at 6727 [76] the leaf rated about 6-7 HP facing a Feline 7/4 (-32) above killing the Feline (-49), and that choice flips by seed.
- **Greedy mode choice (W).** At 1509 [70] the model expected Garodeth mode 1 (wipe); the human took mode 0 (Storm, 8 to face). No cost is shown: without a Ward, the 8 damage could not be stopped anyway.

### Knob matrix

"–" means not tried.

| Position (escape) | 64k nodes | depth=10 | odepth=2 | odepth=2, obeam=12 | osteps=10 | olsolve=1000 | olsolve=4000 |
|---|---|---|---|---|---|---|---|
| 6889 [45] (Adahime→Raz) | – | – | picks it by 1.1, all ≈ -40 | – | no [new] | no (end turn +10.7) | – |
| 1537 [74]/[75] (zombies into Baal and Lt) | – | – | – | – | – | **yes** [new] | **yes**, 2 seeds [new] |
| 1043 [109] (clear VC, human capped at 11) | – | – | – | – | – | no (+35..+43) | partial [new]: end turn -80, chosen line unvetted |
| 2790 [83] (Garodeth Ward + VC) | no (18.8k nodes used) | – | – | – | – | no (+24.3, also at 64k) | no [new] (Garodeth 2nd, +6.2) |
| 4611 [48] (Hark, Hark) | no | – | no | – | no [new] | – | – |
| 5366 [44] / 1720 [41] (no escape shown) | 1720: no (14.1k nodes) | – | no | sees Adahime, same move | – | – | – |
| 1740 [75] (own kill) | 1-in-3 line only | no [new] | – | – | – | – | – |

At 1740 [75], beam=8 also finds no kill.

## 4. What perfect information bought (8 wins)

**Seats.** The cheater won 5 of 16 going first and 3 of 5 going second.

**How the games were won:**
- **Six tempo/race conversions.** Each kill was taken at the first action where the solver saw one: 1829 [82], 1340 [55], 1842 [89], 4038 [62], 1750 [54], 1367 [56].
- **1740:** won by own-turn swings (Bonus PP Istyndet at [46]-[47]; its own Adahime SE wipe on t7), then a missed kill at [75] and a Macmillan-ping kill at [89].
- **1714:** a long attrition game the bot already led (+53) when Depths drew Itsurugi for the kill at [131]. The draw decided when the game ended more than who won it.

**Finishers.**
- Garodeth Storm: 4 (1340, 1842, 1750, 1367).
- Itsurugi mode 0: 2 (1829, 1714).
- Macmillan pings: 1 (1740).
- Hark + Bibatii through a Ward: 1 (4038).

Garodeth's self-damage cost reduction (bot at 12 HP or less) gave early Storm reach in 1340 and 1750; the late Bonus PP paid for it in 1367. The human never had a forced kill in any of the 8 wins.

**Where seeing the hand showed.** The fair served bot (`h0:nodes=16000,horizon=3`, info=open, k=4) chose the same move at every tested position:
- 1340 [39]: same race plan; the cheater's top-2 margin was 0.0.
- 4038 [54]: Istyndet, +45.6.
- 4038 [56]: Istyndet→Adahime, +45.2.
- 1750 [41]: Lilith→Lilith, +49.3.

The information showed up as certainty, not as different moves. "End turn" scored -80:
- at 1829 [55]/[57] (board 9 vs 20 HP; lethal only with the known Garodeth in hand);
- at 1842 [67] (Raz 5 + SE Garodeth 11 + Hark 2 = 18 vs 18);
- at 4038 [54]/[56], where the fair bot scored the same options Hark -34.5 and end turn +27.6 (worst world +1.7).

**Weakened win patterns:**
- **"Remove the SE threat next turn" (W).** It holds at 1829, 1842 and 4038, but 1367's Adahime was a plain 5/4, and the -80 flags came from the known Garodeth.
- **"Race while low against a clogged hand" (W).** Only 1750 fits; in 1340 the human could still cast Adahime or Istyndet.
- **1842 [70] (W).** The explain run under-predicted a non-lethal swing (SE Garodeth 11 face + Hark, bot 18→7). That is greedy under-prediction, not the reply model's missed-kill blind spot (P3).

**In the losses**, the true hand was available but went unused: Adahime in 6889, 4611, 5366, 1720 and 6727; Garodeth in 1043, 1537 and 2790.

**Net.** No checked decision shows the hand producing a different, decisive move. The wins came from tempo, reach and answering public threats, which is search the fair bot shares.

## 5. Probe positions (regression tests)

| Position | Side to move | Escape / correct move | Evidence | Current behaviour |
|---|---|---|---|---|
| 6889 [45] | bot | Adahime 7/6 → Raz 3/3, then end turn | Solver `none` on the true RNG and in 12/12 reseeds | Ends turn (default and olsolve=1000 seed 1 +10.7; osteps=10 [new]). Timeline re-run picks the attack. Negative control: [44] Lt→Raz is **not** a defence (12/12 lethal). |
| 1537 [75] | bot | Rotting Zombie → Baal, Rotting Zombie → Netherworld Lieutenant, end turn | Solver `none`, complete in 2,613 nodes | Default seed 1 ends turn (+24.6) [new]. olsolve=1000/4000 choose the escape [new]. Also [74] with olsolve=1000 plans Macmillan + these attacks. |
| 1043 [109] | bot | Highwire Feline (3 to Void Colonel), Lieutenant→VC, Rotting Zombie→VC (VC's Last Words destroy the bot's Macmillan), Bibatii (evolves)→Istyndet | Hand count: human max 11 vs 13 HP; solver `unknown` at 200k (probable) | Default/olsolve=1000 end the turn (+35..+43). olsolve=4000 flags end turn -80 but picks an unvetted line [new]. |
| 2790 [83] | bot | Garodeth vs. Zeth mode 1 (Ward, 8 to all enemy followers), then crystallise Void Colonel (2 PP) instead of Macmillan | Escape check `none` (bot 11, human 12, human board 2); turn-10 survival untested | Macmillan (live +7.2; olsolve=1000/4000 +24.3). |
| 4611 [48] | bot | Hark to the Night Song ×2, bonus PP, end turn | `none`; both boards empty; human's Garodeth costs 8 vs 7 PP (immediate turn only) | Adahime line +10.7 vs Hark x2 -7.1 (16k, 64k, osteps=10). |
| 1550 [47] | bot | Garodeth vs. Zeth mode 1 instead of Istyndet | Noise probe: 9/10 samples already choose it; turn-8 survival untested | Seed-dependent (about 1 in 6 to 1 in 10 samples pick Istyndet). |
| 1740 [75] | bot (own kill) | Super-evolve Bat or Lilith, play Baal mode 0, then Bat→face and Lilith→face: 10 vs 10, deterministic | Solver `lethal` | No seed or config finds the deterministic order (default, 64k, beam=8, depth=10 [new]). |

## 6. Caveats

- **Sample.** 21 games, one human, one matchup (Abyss Midrange mirror), one day.
  - 8/21 vs 5/23 is not significant (one-sided Fisher p ≈ 0.20). The two fair-bot samples are different bot versions from different sessions.
  - The owner may have adapted across review3 → review4 → this session.
- **First-player share.** 16/21 first is worth about +0.5 expected wins at a 0.55 first-player rate. That rate comes from bot-vs-bot play and may differ against a human. The cheater lost 11 of its 16 first-seat games, and the 3-2 record going second is too small to read.
- **Solver verdicts.**
  - `unknown` = the budget ran out (50k in the escape check, 200k in the helper).
  - `none` rules out a kill only for the fixed RNG stream and only for the immediate turn. At 14-18 HP it says little (5366 Hark-first, 1720 Lt-first/end turn, 5951 t5).
  - `lethal (rng-dependent)` means the solver used rolls it already knew, so "24/24 lethal under reseeds" overstates a human's real chance (2790 [90], 1550 t8).
- **Explain re-runs vs the live seed.** With k=1, re-runs disagree with the live choice at about 15% of decisions. The timeline's "expected line" is truncated to 10 entries. Helper runs use the current engine, which is the served one: PR #94 touched only serve.py, the UI and docs.
- **New runs.** The [new] runs here (osteps=10, olsolve=1000/4000 at 1537, 1043 and 2790, depth=10 at 1740) are one or two seeds each and were not checked independently.
- **One matchup.** Card-specific shapes dominate (Adahime SE, Garodeth Storm, Macmillan zombies), so how far this generalises is unknown.
- **Data quirks.**
  - Timeline headers wrongly claimed deck-order knowledge and labelled the human's hand "HIDDEN"; fixed in the published copies.
  - CARDS.txt omits Void Colonel's Crystallize (2) mode: a Countdown 4 amulet whose Last Words summon Void Colonel. That is the "Void Colonel (amulet, cd N)" seen in the timelines.
  - Super-evolved attackers visibly took no damage when attacking on their own turn (e.g. 6889 [48]), which drives the "answer the SE threat" dynamics.

## Appendix: game IDs and files

| Prefix | Full ID | Result |
|---|---|---|
| 6889 | 6889336734586405565-5ce21003 | loss |
| 1537 | 15374915787985923294-5ce21003 | loss |
| 1043 | 10436817046835498199-5ce21003 | loss |
| 2790 | 2790993798946146787-5ce21003 | loss |
| 4611 | 4611759015070762461-5ce21003 | loss |
| 5366 | 5366400070542567175-5ce21003 | loss |
| 1720 | 17202454263948740065-5ce21003 | loss |
| 6727 | 672758849267719898-5ce21003 | loss |
| 1550 | 15502683015898007302-5ce21003 | loss |
| 1509 | 15092800735689879391-5ce21003 | loss |
| 1399 | 13991699318320519384-5ce21003 | loss |
| 4040 | 4040560043826215616-5ce21003 | loss |
| 5951 | 5951640394299249771-5ce21003 | loss |
| 1829 | 18291913484690914103-5ce21003 | win |
| 1740 | 17408232703009915553-5ce21003 | win |
| 1340 | 13407016906558238790-5ce21003 | win |
| 1714 | 17141743436488616938-5ce21003 | win |
| 1842 | 18420804399337637653-5ce21003 | win |
| 4038 | 4038811095950142129-5ce21003 | win |
| 1750 | 17502911858337506307-5ce21003 | win |
| 1367 | 13672062988649254864-5ce21003 | win |

Files in this folder (`review5/` on the `results` branch):

- **Game records:** `games/<ID>.json` (the raw serve.py captures, byte for byte)
- **Timelines:** `botview/<ID>.txt`, card texts `botview/CARDS.txt`
- **Escape checks:** `escape_check.txt`
- **Helpers:** `tools/replay_now.py` (top 8 candidates) and `tools/replay_all.py` (all candidates), plus `tools/botview3.py`,
  `tools/escape_check3.py`; see `tools/README.md`. Usage: `python <helper> <ID> <N> "<spec>" <seed>`; they call
  `Game.bot_action_explain(spec, seed)` and `arena.forced_lethal(g, 200000)`.
- The checkers' one-off scripts (`try_defence.py`, `chk_g1_defence.py`, `chk_g2_47_seeds.py`) stayed on the owner's box.
