# teach56: teacher data from the search ordered by leaf53 (runbook 56)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=builtin,oorder=1`
- engine: `7151818ab8d012b669c87d247a5131aa1bb9d73c`, seed 5601
- games: 2048 of 2048 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 8.0 h in 1 segment(s), 256.1 games/h
- replay check (every game replayed from its seed and actions): 2048 of 2048 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 133903 of 133903 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 702234 of 702234 (100.00 %)
- decisions: 178886; with two or more candidates: 135959
- paths: {'search': 135159, 'single_legal': 29440, 'take_kill': 8678, 'mulligan': 4096, 'take_kill_roll': 688, 'holdback_trade': 516, 'holdback_resample': 284, 'consensus_lethal': 25}
- candidates per decision (2+): {'n': 135959, 'mean': 5.165, 'median': 4, 'p90': 9, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 40734 of 671547 (6.07 %)
- gap between the best and second-best aggregate: {'n': 135959, 'mean': 4.651, 'median': 1.547, 'p90': 11.923, 'p99': 44.019, 'max': 160.0}
- hbk ran 1381 times, flipped 284; hbcheck ran 1097 times; lostrank ran 1256 times (played another move than the first best-scored one: 963; left out of the search check)
- seconds per decision, one core: {'n': 178886, 'mean': 4.47, 'median': 0.61, 'p90': 14.802, 'p99': 35.789, 'max': 111.025}
- seconds per search decision, one core: {'n': 135159, 'mean': 5.884, 'median': 1.725, 'p90': 17.19, 'p99': 38.652, 'max': 111.025}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 34565, 'mean': 23.133, 'median': 5.93, 'p90': 69.418, 'p99': 188.668, 'max': 537.169}
- decisions per game: {'n': 2048, 'mean': 87.347, 'median': 84, 'p90': 122, 'p99': 168, 'max': 209}; turns: {'n': 2048, 'mean': 8.725, 'median': 8, 'p90': 11, 'p99': 15, 'max': 21}
- first-seat win rate: 0.5581 over 2048 decisive games
- data: 39.5 MB in 2048 files; archive teach56-games.jsonl.gz 38.4 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-portal-af | 0.676 |
| meta-abyss-midrange | 0.660 |
| meta-rune-test-subject | 0.637 |
| meta-haven-evo | 0.621 |
| meta-sword-rally | 0.594 |
| meta-portal-evo | 0.559 |
| meta-sword-loot | 0.535 |
| meta-portal-cutthroat | 0.492 |
| meta-forest-combo | 0.469 |
| meta-dragon-ramp | 0.457 |
| meta-abyss-aggro | 0.449 |
| meta-dragon-aggro | 0.434 |
| meta-haven-kukishiro | 0.391 |
| meta-rune-spell | 0.375 |
| meta-haven-amulet | 0.352 |
| meta-rune-crystal | 0.301 |
