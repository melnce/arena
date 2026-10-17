# teach57: teacher data from the served search with the reply keys (runbook 57)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40`
- engine: `6aab0a827974840945cfd0b6f0078f56968e79ea`, seed 5701
- games: 2048 of 2048 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 14.79 h in 1 segment(s), 138.5 games/h
- replay check (every game replayed from its seed and actions): 2048 of 2048 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 138817 of 138817 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 732068 of 732068 (100.00 %)
- decisions: 184611; with two or more candidates: 140939
- paths: {'search': 140248, 'single_legal': 30386, 'take_kill': 8331, 'mulligan': 4096, 'take_kill_roll': 827, 'holdback_trade': 441, 'holdback_resample': 250, 'consensus_lethal': 32}
- candidates per decision (2+): {'n': 140939, 'mean': 5.194, 'median': 4, 'p90': 9, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 43877 of 699677 (6.27 %)
- gap between the best and second-best aggregate: {'n': 140939, 'mean': 4.703, 'median': 1.663, 'p90': 12.127, 'p99': 43.108, 'max': 160.0}
- hbk ran 1272 times, flipped 250; hbcheck ran 1022 times; lostrank ran 1431 times (played another move than the first best-scored one: 1073; left out of the search check)
- world line ends over all candidates (5856544; O = opp_open, N = opp_next, ? = unknown): {'o': 1722394, 'C': 1460310, 'O': 1196040, 'N': 496099, 'L': 444357, 't': 299211, 'D': 219840, 'T': 9402, 'c': 8797, 'd': 94}
- seconds per decision, one core: {'n': 184611, 'mean': 7.979, 'median': 2.638, 'p90': 23.648, 'p99': 54.588, 'max': 212.401}
- seconds per search decision, one core: {'n': 140248, 'mean': 10.449, 'median': 5.53, 'p90': 27.474, 'p99': 58.668, 'max': 212.401}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 35280, 'mean': 41.754, 'median': 15.767, 'p90': 120.757, 'p99': 288.62, 'max': 741.144}
- decisions per game: {'n': 2048, 'mean': 90.142, 'median': 86, 'p90': 128, 'p99': 174, 'max': 206}; turns: {'n': 2048, 'mean': 8.888, 'median': 9, 'p90': 12, 'p99': 15, 'max': 18}
- first-seat win rate: 0.5381 over 2048 decisive games
- data: 41.9 MB in 2048 files; archive teach57-games.jsonl.gz 40.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-sword-rally | 0.660 |
| meta-abyss-midrange | 0.609 |
| meta-portal-evo | 0.609 |
| meta-haven-evo | 0.586 |
| meta-portal-af | 0.586 |
| meta-rune-test-subject | 0.543 |
| meta-sword-loot | 0.531 |
| meta-portal-cutthroat | 0.523 |
| meta-haven-kukishiro | 0.492 |
| meta-dragon-ramp | 0.465 |
| meta-forest-combo | 0.465 |
| meta-abyss-aggro | 0.441 |
| meta-rune-spell | 0.422 |
| meta-dragon-aggro | 0.410 |
| meta-haven-amulet | 0.383 |
| meta-rune-crystal | 0.273 |
