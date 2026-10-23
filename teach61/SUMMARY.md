# teach61: teacher data from the served search with the reply keys (runbook 61)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40`
- engine: `75f087f95e767f72ff7d8fb8ae630431be990efe`, seed 6101
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 4.15 h in 1 segment(s), 987.7 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 278929 of 278929 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1485040 of 1485040 (100.00 %)
- decisions: 370273; with two or more candidates: 283306
- paths: {'search': 281779, 'single_legal': 60425, 'take_kill': 16718, 'mulligan': 8192, 'take_kill_roll': 1588, 'holdback_trade': 936, 'holdback_resample': 591, 'consensus_lethal': 44}
- candidates per decision (2+): {'n': 283306, 'mean': 5.242, 'median': 4, 'p90': 9, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 85283 of 1420596 (6.00 %)
- gap between the best and second-best aggregate: {'n': 283306, 'mean': 4.739, 'median': 1.701, 'p90': 12.157, 'p99': 42.753, 'max': 160.0}
- hbk ran 2735 times, flipped 591; hbcheck ran 2144 times; lostrank ran 2850 times (played another move than the first best-scored one: 2092; left out of the search check)
- world line ends over all candidates (11880320; O = opp_open, N = opp_next, ? = unknown): {'o': 3560809, 'C': 2994766, 'O': 2393757, 'N': 957001, 'L': 875083, 't': 604908, 'D': 461264, 'T': 19188, 'c': 13224, 'd': 320}
- seconds per decision, one core: {'n': 370273, 'mean': 1.124, 'median': 0.388, 'p90': 3.258, 'p99': 7.807, 'max': 28.802}
- seconds per search decision, one core: {'n': 281779, 'mean': 1.466, 'median': 0.786, 'p90': 3.79, 'p99': 8.333, 'max': 23.109}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 70236, 'mean': 5.926, 'median': 2.158, 'p90': 17.128, 'p99': 41.906, 'max': 105.932}
- decisions per game: {'n': 4096, 'mean': 90.399, 'median': 87, 'p90': 129, 'p99': 173, 'max': 244}; turns: {'n': 4096, 'mean': 8.857, 'median': 9, 'p90': 12, 'p99': 15, 'max': 22}
- first-seat win rate: 0.5537 over 4096 decisive games
- data: 85.3 MB in 4096 files; archive teach61-games.jsonl.gz 83.0 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-portal-af | 0.609 |
| meta-rune-test-subject | 0.602 |
| meta-sword-rally | 0.596 |
| meta-abyss-midrange | 0.588 |
| meta-portal-evo | 0.588 |
| meta-haven-evo | 0.574 |
| meta-portal-cutthroat | 0.504 |
| meta-sword-loot | 0.504 |
| meta-forest-combo | 0.480 |
| meta-dragon-aggro | 0.473 |
| meta-haven-kukishiro | 0.449 |
| meta-abyss-aggro | 0.440 |
| meta-dragon-ramp | 0.434 |
| meta-rune-spell | 0.410 |
| meta-haven-amulet | 0.393 |
| meta-rune-crystal | 0.357 |
