# teach85: teacher data from the served search with the reply keys (runbook 85)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8501
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.25 h in 1 segment(s), 1261.2 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290014 of 290014 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1592446 of 1592446 (100.00 %)
- decisions: 379245; with two or more candidates: 294841
- paths: {'search': 293008, 'single_legal': 57647, 'take_kill': 16550, 'mulligan': 8192, 'take_kill_roll': 1936, 'holdback_trade': 1253, 'holdback_resample': 580, 'consensus_lethal': 79}
- candidates per decision (2+): {'n': 294841, 'mean': 5.401, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92701 of 1515014 (6.12 %)
- gap between the best and second-best aggregate: {'n': 294841, 'mean': 4.9, 'median': 1.678, 'p90': 12.555, 'p99': 46.143, 'max': 160.0}
- hbk ran 3121 times, flipped 580; hbcheck ran 2541 times; lostrank ran 2994 times (played another move than the first best-scored one: 2334; left out of the search check)
- world line ends over all candidates (12739568; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4286549, 'O': 3890498, 'C': 1998269, 'L': 1064293, 'D': 734644, 't': 718319, 'T': 27938, 'c': 18925, 'd': 133}
- seconds per decision, one core: {'n': 379245, 'mean': 0.857, 'median': 0.213, 'p90': 2.59, 'p99': 6.057, 'max': 38.886}
- seconds per search decision, one core: {'n': 293008, 'mean': 1.096, 'median': 0.502, 'p90': 2.98, 'p99': 6.465, 'max': 38.886}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71384, 'mean': 4.53, 'median': 1.616, 'p90': 13.011, 'p99': 32.66, 'max': 120.644}
- decisions per game: {'n': 4096, 'mean': 92.589, 'median': 89, 'p90': 128, 'p99': 174, 'max': 227}; turns: {'n': 4096, 'mean': 8.999, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5476 over 4096 decisive games
- data: 91.3 MB in 4096 files; archive teach85-games.jsonl.gz 89.0 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.650 |
| meta-rune-test-subject | 0.611 |
| meta-portal-evo | 0.576 |
| meta-portal-af | 0.549 |
| meta-abyss-midrange | 0.541 |
| meta-sword-rally | 0.525 |
| meta-dragon-ramp | 0.519 |
| meta-portal-cutthroat | 0.519 |
| meta-haven-kukishiro | 0.490 |
| meta-haven-amulet | 0.484 |
| meta-sword-loot | 0.482 |
| meta-forest-combo | 0.480 |
| meta-rune-spell | 0.422 |
| meta-abyss-aggro | 0.414 |
| meta-dragon-aggro | 0.395 |
| meta-rune-crystal | 0.340 |
