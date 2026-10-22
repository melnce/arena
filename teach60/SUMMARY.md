# teach60: teacher data from the served search with the reply keys (runbook 60)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40`
- engine: `75f087f95e767f72ff7d8fb8ae630431be990efe`, seed 6001
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.89 h in 1 segment(s), 1054.1 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 276407 of 276407 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1465338 of 1465338 (100.00 %)
- decisions: 366827; with two or more candidates: 280618
- paths: {'search': 279096, 'single_legal': 59670, 'take_kill': 16804, 'mulligan': 8192, 'take_kill_roll': 1496, 'holdback_trade': 897, 'holdback_resample': 625, 'consensus_lethal': 47}
- candidates per decision (2+): {'n': 280618, 'mean': 5.222, 'median': 4, 'p90': 9, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 84279 of 1402142 (6.01 %)
- gap between the best and second-best aggregate: {'n': 280618, 'mean': 4.704, 'median': 1.667, 'p90': 12.165, 'p99': 42.516, 'max': 160.0}
- hbk ran 2734 times, flipped 625; hbcheck ran 2109 times; lostrank ran 2689 times (played another move than the first best-scored one: 1967; left out of the search check)
- world line ends over all candidates (11722704; O = opp_open, N = opp_next, ? = unknown): {'o': 3541024, 'C': 2919270, 'O': 2362542, 'N': 951354, 'L': 858389, 't': 607886, 'D': 450791, 'T': 21331, 'c': 10008, 'd': 109}
- seconds per decision, one core: {'n': 366827, 'mean': 1.064, 'median': 0.363, 'p90': 3.08, 'p99': 7.387, 'max': 23.078}
- seconds per search decision, one core: {'n': 279096, 'mean': 1.388, 'median': 0.744, 'p90': 3.596, 'p99': 7.936, 'max': 23.078}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 69747, 'mean': 5.596, 'median': 2.041, 'p90': 16.116, 'p99': 39.965, 'max': 106.23}
- decisions per game: {'n': 4096, 'mean': 89.557, 'median': 85, 'p90': 126, 'p99': 178, 'max': 265}; turns: {'n': 4096, 'mean': 8.8, 'median': 8, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.5613 over 4096 decisive games
- data: 84.2 MB in 4096 files; archive teach60-games.jsonl.gz 81.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-sword-rally | 0.635 |
| meta-rune-test-subject | 0.623 |
| meta-abyss-midrange | 0.617 |
| meta-portal-af | 0.606 |
| meta-haven-evo | 0.600 |
| meta-portal-evo | 0.580 |
| meta-sword-loot | 0.537 |
| meta-portal-cutthroat | 0.525 |
| meta-abyss-aggro | 0.486 |
| meta-forest-combo | 0.453 |
| meta-dragon-aggro | 0.443 |
| meta-rune-spell | 0.418 |
| meta-haven-kukishiro | 0.408 |
| meta-dragon-ramp | 0.389 |
| meta-haven-amulet | 0.350 |
| meta-rune-crystal | 0.330 |
