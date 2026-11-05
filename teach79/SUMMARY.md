# teach79: teacher data from the served search with the reply keys (runbook 79)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7901
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.0 h in 1 segment(s), 1366.1 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 286722 of 286722 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1564392 of 1564392 (100.00 %)
- decisions: 376052; with two or more candidates: 291499
- paths: {'search': 289711, 'single_legal': 57950, 'take_kill': 16468, 'mulligan': 8192, 'take_kill_roll': 1887, 'holdback_trade': 1246, 'holdback_resample': 542, 'consensus_lethal': 56}
- candidates per decision (2+): {'n': 291499, 'mean': 5.367, 'median': 5, 'p90': 10, 'p99': 16, 'max': 41}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91830 of 1490094 (6.16 %)
- gap between the best and second-best aggregate: {'n': 291499, 'mean': 4.91, 'median': 1.697, 'p90': 12.633, 'p99': 45.916, 'max': 160.0}
- hbk ran 3070 times, flipped 542; hbcheck ran 2528 times; lostrank ran 2989 times (played another move than the first best-scored one: 2296; left out of the search check)
- world line ends over all candidates (12515136; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4243579, 'O': 3848385, 'C': 1923911, 'L': 1036790, 't': 711386, 'D': 708805, 'T': 25333, 'c': 16744, 'd': 203}
- seconds per decision, one core: {'n': 376052, 'mean': 0.801, 'median': 0.192, 'p90': 2.433, 'p99': 5.701, 'max': 21.201}
- seconds per search decision, one core: {'n': 289711, 'mean': 1.027, 'median': 0.463, 'p90': 2.803, 'p99': 6.095, 'max': 17.703}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71213, 'mean': 4.208, 'median': 1.463, 'p90': 12.213, 'p99': 30.32, 'max': 94.645}
- decisions per game: {'n': 4096, 'mean': 91.81, 'median': 89, 'p90': 127, 'p99': 167, 'max': 243}; turns: {'n': 4096, 'mean': 8.981, 'median': 9, 'p90': 12, 'p99': 14, 'max': 21}
- first-seat win rate: 0.5581 over 4096 decisive games
- data: 89.9 MB in 4096 files; archive teach79-games.jsonl.gz 87.6 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.652 |
| meta-haven-evo | 0.635 |
| meta-sword-rally | 0.582 |
| meta-portal-evo | 0.555 |
| meta-abyss-midrange | 0.539 |
| meta-portal-cutthroat | 0.521 |
| meta-forest-combo | 0.510 |
| meta-portal-af | 0.508 |
| meta-dragon-ramp | 0.498 |
| meta-sword-loot | 0.477 |
| meta-abyss-aggro | 0.443 |
| meta-dragon-aggro | 0.441 |
| meta-rune-spell | 0.438 |
| meta-haven-kukishiro | 0.432 |
| meta-haven-amulet | 0.408 |
| meta-rune-crystal | 0.361 |
