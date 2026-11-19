# teach94: teacher data from the served search with the reply keys (runbook 94)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9401
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.11 h in 1 segment(s), 1315.6 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 291657 of 291657 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1600974 of 1600974 (100.00 %)
- decisions: 381035; with two or more candidates: 296900
- paths: {'search': 295057, 'single_legal': 57690, 'take_kill': 16306, 'mulligan': 8192, 'take_kill_roll': 1873, 'holdback_trade': 1282, 'holdback_resample': 561, 'consensus_lethal': 74}
- candidates per decision (2+): {'n': 296900, 'mean': 5.392, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92794 of 1522972 (6.09 %)
- gap between the best and second-best aggregate: {'n': 296900, 'mean': 4.893, 'median': 1.674, 'p90': 12.554, 'p99': 46.182, 'max': 160.0}
- hbk ran 3161 times, flipped 561; hbcheck ran 2600 times; lostrank ran 3400 times (played another move than the first best-scored one: 2584; left out of the search check)
- world line ends over all candidates (12807792; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4316045, 'O': 3924289, 'C': 2005311, 'L': 1060519, 'D': 734975, 't': 723383, 'T': 25005, 'c': 18034, 'd': 231}
- seconds per decision, one core: {'n': 381035, 'mean': 0.821, 'median': 0.203, 'p90': 2.491, 'p99': 5.812, 'max': 28.514}
- seconds per search decision, one core: {'n': 295057, 'mean': 1.048, 'median': 0.48, 'p90': 2.853, 'p99': 6.198, 'max': 28.514}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71554, 'mean': 4.351, 'median': 1.528, 'p90': 12.69, 'p99': 30.691, 'max': 83.436}
- decisions per game: {'n': 4096, 'mean': 93.026, 'median': 90, 'p90': 129, 'p99': 171, 'max': 222}; turns: {'n': 4096, 'mean': 9.019, 'median': 9, 'p90': 12, 'p99': 15, 'max': 23}
- first-seat win rate: 0.5491 over 4096 decisive games
- data: 91.8 MB in 4096 files; archive teach94-games.jsonl.gz 89.5 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.685 |
| meta-haven-evo | 0.660 |
| meta-portal-af | 0.551 |
| meta-abyss-midrange | 0.541 |
| meta-portal-evo | 0.541 |
| meta-sword-rally | 0.539 |
| meta-dragon-ramp | 0.521 |
| meta-sword-loot | 0.508 |
| meta-portal-cutthroat | 0.502 |
| meta-haven-kukishiro | 0.496 |
| meta-rune-spell | 0.459 |
| meta-forest-combo | 0.441 |
| meta-abyss-aggro | 0.402 |
| meta-haven-amulet | 0.393 |
| meta-dragon-aggro | 0.381 |
| meta-rune-crystal | 0.379 |
