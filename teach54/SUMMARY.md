# teach54: teacher data at sixteen worlds (runbook 54)

- verdict: DATA OK
- teacher: `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000`
- engine: `8093a9e93230d851bdb3f1bf776303d0ba80d789`, seed 5401
- games: 2048 of 2048 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 11.42 h in 2 segment(s), 177.2 games/h
- replay check (every game replayed from its seed and actions): 2048 of 2048 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 130971 of 130971 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 680230 of 680230 (100.00 %)
- decisions: 176224; with two or more candidates: 133258
- paths: {'search': 132372, 'single_legal': 29862, 'take_kill': 8364, 'mulligan': 4096, 'holdback_trade': 705, 'take_kill_roll': 626, 'holdback_resample': 181, 'consensus_lethal': 18}
- candidates per decision (2+): {'n': 133258, 'mean': 5.105, 'median': 4, 'p90': 9, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 36609 of 650789 (5.63 %)
- gap between the best and second-best aggregate: {'n': 133258, 'mean': 4.735, 'median': 1.54, 'p90': 12.066, 'p99': 45.461, 'max': 160.0}
- hbk ran 1570 times, flipped 181; hbcheck ran 1389 times; lostrank ran 1401 times (played another move than the first best-scored one: 1070; left out of the search check)
- seconds per decision, one core: {'n': 176224, 'mean': 6.506, 'median': 1.0, 'p90': 20.781, 'p99': 50.422, 'max': 178.138}
- seconds per search decision, one core: {'n': 132372, 'mean': 8.616, 'median': 2.813, 'p90': 23.929, 'p99': 54.989, 'max': 178.138}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 34240, 'mean': 33.485, 'median': 9.825, 'p90': 98.303, 'p99': 260.946, 'max': 807.473}
- decisions per game: {'n': 2048, 'mean': 86.047, 'median': 83, 'p90': 120, 'p99': 159, 'max': 210}; turns: {'n': 2048, 'mean': 8.637, 'median': 8, 'p90': 11, 'p99': 14, 'max': 19}
- first-seat win rate: 0.542 over 2048 decisive games
- data: 57.8 MB in 2048 files; archive teach54-games.jsonl.gz 56.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-portal-evo | 0.703 |
| meta-haven-evo | 0.668 |
| meta-abyss-midrange | 0.644 |
| meta-sword-rally | 0.621 |
| meta-portal-af | 0.617 |
| meta-rune-test-subject | 0.566 |
| meta-sword-loot | 0.512 |
| meta-abyss-aggro | 0.480 |
| meta-dragon-aggro | 0.480 |
| meta-portal-cutthroat | 0.480 |
| meta-forest-combo | 0.441 |
| meta-dragon-ramp | 0.414 |
| meta-rune-spell | 0.387 |
| meta-haven-kukishiro | 0.344 |
| meta-rune-crystal | 0.332 |
| meta-haven-amulet | 0.309 |
