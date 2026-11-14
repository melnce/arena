# teach88: teacher data from the served search with the reply keys (runbook 88)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run87/leaf87q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8801
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.19 h in 1 segment(s), 1282.4 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 291620 of 291620 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1600467 of 1600467 (100.00 %)
- decisions: 381226; with two or more candidates: 296608
- paths: {'search': 294841, 'single_legal': 57890, 'take_kill': 16393, 'mulligan': 8192, 'take_kill_roll': 2077, 'holdback_trade': 1181, 'holdback_resample': 586, 'consensus_lethal': 66}
- candidates per decision (2+): {'n': 296608, 'mean': 5.396, 'median': 5, 'p90': 10, 'p99': 16, 'max': 33}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91993 of 1522961 (6.04 %)
- gap between the best and second-best aggregate: {'n': 296608, 'mean': 4.862, 'median': 1.675, 'p90': 12.535, 'p99': 45.412, 'max': 160.0}
- hbk ran 3044 times, flipped 586; hbcheck ran 2458 times; lostrank ran 3221 times (played another move than the first best-scored one: 2499; left out of the search check)
- world line ends over all candidates (12803736; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4308934, 'O': 3915353, 'C': 2002574, 'L': 1078670, 'D': 731040, 't': 725363, 'T': 25845, 'c': 15742, 'd': 215}
- seconds per decision, one core: {'n': 381226, 'mean': 0.84, 'median': 0.208, 'p90': 2.546, 'p99': 5.959, 'max': 20.932}
- seconds per search decision, one core: {'n': 294841, 'mean': 1.074, 'median': 0.493, 'p90': 2.925, 'p99': 6.346, 'max': 19.71}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71586, 'mean': 4.452, 'median': 1.573, 'p90': 12.833, 'p99': 31.619, 'max': 94.083}
- decisions per game: {'n': 4096, 'mean': 93.073, 'median': 90, 'p90': 130, 'p99': 174, 'max': 222}; turns: {'n': 4096, 'mean': 9.016, 'median': 9, 'p90': 12, 'p99': 15, 'max': 22}
- first-seat win rate: 0.541 over 4096 decisive games
- data: 91.8 MB in 4096 files; archive teach88-games.jsonl.gz 89.5 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.682 |
| meta-haven-evo | 0.658 |
| meta-sword-rally | 0.574 |
| meta-portal-evo | 0.549 |
| meta-abyss-midrange | 0.537 |
| meta-portal-af | 0.529 |
| meta-sword-loot | 0.510 |
| meta-dragon-ramp | 0.506 |
| meta-portal-cutthroat | 0.482 |
| meta-forest-combo | 0.467 |
| meta-rune-spell | 0.457 |
| meta-haven-kukishiro | 0.455 |
| meta-abyss-aggro | 0.418 |
| meta-haven-amulet | 0.418 |
| meta-dragon-aggro | 0.406 |
| meta-rune-crystal | 0.352 |
