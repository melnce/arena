# teach69: teacher data from the served search with the reply keys (runbook 69)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run69/leaf69q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6901
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.27 h in 5 segment(s), 1307.3 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 572770 of 572770 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3104229 of 3104229 (100.00 %)
- decisions: 752104; with two or more candidates: 582912
- paths: {'search': 579524, 'single_legal': 116265, 'take_kill': 32638, 'mulligan': 16384, 'take_kill_roll': 3727, 'holdback_trade': 2171, 'holdback_resample': 1217, 'consensus_lethal': 178}
- candidates per decision (2+): {'n': 582912, 'mean': 5.325, 'median': 5, 'p90': 9, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 182322 of 2951327 (6.18 %)
- gap between the best and second-best aggregate: {'n': 582912, 'mean': 4.774, 'median': 1.644, 'p90': 12.225, 'p99': 44.538, 'max': 160.0}
- hbk ran 6049 times, flipped 1217; hbcheck ran 4832 times; lostrank ran 6754 times (played another move than the first best-scored one: 5204; left out of the search check)
- world line ends over all candidates (24833832; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8530847, 'O': 7513510, 'C': 3789401, 'L': 2078552, 't': 1443825, 'D': 1398944, 'T': 49678, 'c': 28735, 'd': 340}
- seconds per decision, one core: {'n': 752104, 'mean': 0.828, 'median': 0.194, 'p90': 2.521, 'p99': 5.985, 'max': 19.254}
- seconds per search decision, one core: {'n': 579524, 'mean': 1.067, 'median': 0.477, 'p90': 2.907, 'p99': 6.417, 'max': 19.254}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142746, 'mean': 4.362, 'median': 1.519, 'p90': 12.619, 'p99': 31.357, 'max': 102.015}
- decisions per game: {'n': 8192, 'mean': 91.81, 'median': 89, 'p90': 129, 'p99': 170, 'max': 229}; turns: {'n': 8192, 'mean': 8.997, 'median': 9, 'p90': 12, 'p99': 15, 'max': 29}
- first-seat win rate: 0.5531 over 8192 decisive games
- data: 177.9 MB in 8192 files; archive teach69-games.jsonl.gz 173.2 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.629 |
| meta-haven-evo | 0.615 |
| meta-portal-evo | 0.608 |
| meta-portal-af | 0.590 |
| meta-sword-rally | 0.556 |
| meta-abyss-midrange | 0.552 |
| meta-sword-loot | 0.515 |
| meta-portal-cutthroat | 0.510 |
| meta-haven-kukishiro | 0.490 |
| meta-dragon-ramp | 0.479 |
| meta-forest-combo | 0.472 |
| meta-abyss-aggro | 0.448 |
| meta-dragon-aggro | 0.413 |
| meta-rune-spell | 0.402 |
| meta-haven-amulet | 0.401 |
| meta-rune-crystal | 0.319 |
