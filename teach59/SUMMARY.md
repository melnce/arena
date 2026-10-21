# teach59: teacher data from the served search with the reply keys (runbook 59)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40`
- engine: `2e529fb9869b2ed698911c8f0a9c8df18ccf6060`, seed 5901
- games: 2048 of 2048 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 16.09 h in 1 segment(s), 127.3 games/h
- replay check (every game replayed from its seed and actions): 2048 of 2048 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 139783 of 139783 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 741358 of 741358 (100.00 %)
- decisions: 185272; with two or more candidates: 141958
- paths: {'search': 141157, 'single_legal': 29961, 'take_kill': 8394, 'mulligan': 4096, 'take_kill_roll': 840, 'holdback_trade': 495, 'holdback_resample': 306, 'consensus_lethal': 23}
- candidates per decision (2+): {'n': 141958, 'mean': 5.222, 'median': 4, 'p90': 9, 'p99': 16, 'max': 32}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 43011 of 709503 (6.06 %)
- gap between the best and second-best aggregate: {'n': 141958, 'mean': 4.687, 'median': 1.677, 'p90': 12.09, 'p99': 42.164, 'max': 160.0}
- hbk ran 1448 times, flipped 306; hbcheck ran 1142 times; lostrank ran 1374 times (played another move than the first best-scored one: 991; left out of the search check)
- world line ends over all candidates (5930864; O = opp_open, N = opp_next, ? = unknown): {'o': 1792288, 'C': 1474086, 'O': 1204964, 'N': 474877, 'L': 432363, 't': 306321, 'D': 229243, 'T': 10324, 'c': 6325, 'd': 73}
- seconds per decision, one core: {'n': 185272, 'mean': 8.674, 'median': 2.896, 'p90': 25.758, 'p99': 59.341, 'max': 176.098}
- seconds per search decision, one core: {'n': 141157, 'mean': 11.316, 'median': 5.994, 'p90': 29.844, 'p99': 63.355, 'max': 156.997}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 35244, 'mean': 45.596, 'median': 16.754, 'p90': 131.428, 'p99': 319.134, 'max': 749.784}
- decisions per game: {'n': 2048, 'mean': 90.465, 'median': 86, 'p90': 127, 'p99': 175, 'max': 216}; turns: {'n': 2048, 'mean': 8.884, 'median': 9, 'p90': 12, 'p99': 15, 'max': 20}
- first-seat win rate: 0.5479 over 2048 decisive games
- data: 42.7 MB in 2048 files; archive teach59-games.jsonl.gz 41.6 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.672 |
| meta-abyss-midrange | 0.625 |
| meta-rune-test-subject | 0.586 |
| meta-sword-rally | 0.586 |
| meta-portal-evo | 0.582 |
| meta-portal-af | 0.566 |
| meta-portal-cutthroat | 0.547 |
| meta-sword-loot | 0.512 |
| meta-dragon-ramp | 0.473 |
| meta-dragon-aggro | 0.465 |
| meta-forest-combo | 0.461 |
| meta-haven-kukishiro | 0.445 |
| meta-abyss-aggro | 0.406 |
| meta-rune-spell | 0.391 |
| meta-haven-amulet | 0.379 |
| meta-rune-crystal | 0.305 |
