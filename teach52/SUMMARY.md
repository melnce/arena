# teach52: teacher data (runbook 52)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1`
- engine: `99e07753ecc3c7e513d0f3be2668a73af810b0f7`, seed 5201
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 13.44 h in 1 segment(s), 304.8 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 267319 of 267319 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1369869 of 1369869 (100.00 %)
- decisions: 355852; with two or more candidates: 269277
- paths: {'search': 267319, 'single_legal': 60379, 'take_kill': 16562, 'mulligan': 8192, 'take_kill_roll': 1381, 'holdback_trade': 1348, 'holdback_resample': 610, 'consensus_lethal': 61}
- candidates per decision (2+): {'n': 269277, 'mean': 5.087, 'median': 4, 'p90': 9, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 81868 of 1301641 (6.29 %)
- gap between the best and second-best aggregate: {'n': 269277, 'mean': 4.772, 'median': 1.526, 'p90': 12.356, 'p99': 46.249, 'max': 160.0}
- hbk ran 3253 times, flipped 610; hbcheck ran 2643 times
- seconds per decision, one core: {'n': 355852, 'mean': 3.782, 'median': 0.523, 'p90': 12.677, 'p99': 29.744, 'max': 118.887}
- seconds per search decision, one core: {'n': 267319, 'mean': 4.998, 'median': 1.446, 'p90': 14.694, 'p99': 32.238, 'max': 118.887}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 68620, 'mean': 19.614, 'median': 5.426, 'p90': 58.138, 'p99': 154.236, 'max': 544.118}
- decisions per game: {'n': 4096, 'mean': 86.878, 'median': 83, 'p90': 122, 'p99': 168, 'max': 200}; turns: {'n': 4096, 'mean': 8.656, 'median': 8, 'p90': 11, 'p99': 15, 'max': 18}
- first-seat win rate: 0.5483 over 4096 decisive games
- data: 76.9 MB in 4096 files; archive teach52-games.jsonl.gz 74.6 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.652 |
| meta-portal-evo | 0.641 |
| meta-rune-test-subject | 0.637 |
| meta-portal-af | 0.611 |
| meta-sword-rally | 0.602 |
| meta-abyss-midrange | 0.594 |
| meta-sword-loot | 0.531 |
| meta-portal-cutthroat | 0.514 |
| meta-forest-combo | 0.490 |
| meta-abyss-aggro | 0.469 |
| meta-dragon-aggro | 0.435 |
| meta-dragon-ramp | 0.430 |
| meta-rune-spell | 0.400 |
| meta-haven-kukishiro | 0.385 |
| meta-haven-amulet | 0.316 |
| meta-rune-crystal | 0.293 |
