# teach94: teacher data from the served search with the reply keys (runbook 94)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9401
- games: 7168 of 7168 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 5.62 h in 4 segment(s), 1274.4 games/h
- replay check (every game replayed from its seed and actions): 7168 of 7168 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 507847 of 507847 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2785357 of 2785357 (100.00 %)
- decisions: 664293; with two or more candidates: 516692
- paths: {'search': 513462, 'single_legal': 100974, 'take_kill': 28848, 'mulligan': 14336, 'take_kill_roll': 3325, 'holdback_trade': 2271, 'holdback_resample': 959, 'consensus_lethal': 118}
- candidates per decision (2+): {'n': 516692, 'mean': 5.391, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 161082 of 2651735 (6.07 %)
- gap between the best and second-best aggregate: {'n': 516692, 'mean': 4.892, 'median': 1.679, 'p90': 12.562, 'p99': 45.938, 'max': 160.0}
- hbk ran 5504 times, flipped 959; hbcheck ran 4545 times; lostrank ran 5615 times (played another move than the first best-scored one: 4256; left out of the search check)
- world line ends over all candidates (22282856; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 7516678, 'O': 6826773, 'C': 3501318, 'L': 1835101, 'D': 1274630, 't': 1254776, 'T': 41134, 'c': 32114, 'd': 332}
- seconds per decision, one core: {'n': 664293, 'mean': 0.844, 'median': 0.21, 'p90': 2.551, 'p99': 6.012, 'max': 29.809}
- seconds per search decision, one core: {'n': 513462, 'mean': 1.079, 'median': 0.493, 'p90': 2.93, 'p99': 6.401, 'max': 28.514}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 124955, 'mean': 4.464, 'median': 1.576, 'p90': 12.988, 'p99': 31.728, 'max': 89.516}
- decisions per game: {'n': 7168, 'mean': 92.675, 'median': 90, 'p90': 129, 'p99': 168, 'max': 222}; turns: {'n': 7168, 'mean': 9.002, 'median': 9, 'p90': 12, 'p99': 14, 'max': 23}
- first-seat win rate: 0.552 over 7168 decisive games
- data: 159.9 MB in 7168 files; archive teach94-games.jsonl.gz 155.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.671 |
| meta-haven-evo | 0.651 |
| meta-portal-evo | 0.566 |
| meta-portal-af | 0.556 |
| meta-abyss-midrange | 0.550 |
| meta-sword-rally | 0.523 |
| meta-dragon-ramp | 0.508 |
| meta-portal-cutthroat | 0.501 |
| meta-sword-loot | 0.500 |
| meta-haven-kukishiro | 0.483 |
| meta-forest-combo | 0.464 |
| meta-rune-spell | 0.461 |
| meta-abyss-aggro | 0.418 |
| meta-haven-amulet | 0.411 |
| meta-dragon-aggro | 0.386 |
| meta-rune-crystal | 0.350 |
