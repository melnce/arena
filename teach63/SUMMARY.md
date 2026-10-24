# teach63: teacher data from the served search with the reply keys (runbook 63)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6301
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.89 h in 1 segment(s), 1053.9 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 279985 of 279985 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1469765 of 1469765 (100.00 %)
- decisions: 371424; with two or more candidates: 284956
- paths: {'search': 283514, 'single_legal': 60587, 'take_kill': 16030, 'mulligan': 8192, 'take_kill_roll': 1620, 'holdback_trade': 815, 'holdback_resample': 627, 'consensus_lethal': 39}
- candidates per decision (2+): {'n': 284956, 'mean': 5.158, 'median': 4, 'p90': 9, 'p99': 15, 'max': 33}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 84649 of 1398263 (6.05 %)
- gap between the best and second-best aggregate: {'n': 284956, 'mean': 4.711, 'median': 1.657, 'p90': 12.088, 'p99': 43.433, 'max': 160.0}
- hbk ran 2532 times, flipped 627; hbcheck ran 1905 times; lostrank ran 3529 times (played another move than the first best-scored one: 2574; left out of the search check)
- world line ends over all candidates (11758120; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 3472384, 'C': 3283758, 'O': 2058463, 'L': 960504, 'N': 926391, 't': 589597, 'D': 435361, 'T': 20357, 'c': 11147, 'd': 158}
- seconds per decision, one core: {'n': 371424, 'mean': 1.051, 'median': 0.403, 'p90': 3.04, 'p99': 7.14, 'max': 21.184}
- seconds per search decision, one core: {'n': 283514, 'mean': 1.37, 'median': 0.755, 'p90': 3.527, 'p99': 7.618, 'max': 21.184}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 70844, 'mean': 5.51, 'median': 2.149, 'p90': 15.662, 'p99': 37.613, 'max': 102.755}
- decisions per game: {'n': 4096, 'mean': 90.68, 'median': 87, 'p90': 128, 'p99': 173, 'max': 230}; turns: {'n': 4096, 'mean': 8.926, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.5461 over 4096 decisive games
- data: 83.8 MB in 4096 files; archive teach63-games.jsonl.gz 81.6 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.639 |
| meta-portal-evo | 0.619 |
| meta-sword-rally | 0.607 |
| meta-rune-test-subject | 0.606 |
| meta-portal-af | 0.604 |
| meta-abyss-midrange | 0.559 |
| meta-sword-loot | 0.521 |
| meta-portal-cutthroat | 0.494 |
| meta-dragon-ramp | 0.477 |
| meta-forest-combo | 0.477 |
| meta-dragon-aggro | 0.443 |
| meta-abyss-aggro | 0.435 |
| meta-haven-kukishiro | 0.435 |
| meta-rune-spell | 0.410 |
| meta-haven-amulet | 0.344 |
| meta-rune-crystal | 0.330 |
