# teach88: teacher data from the served search with the reply keys (runbook 88)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run87/leaf87q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8801
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.49 h in 5 segment(s), 1261.7 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 584892 of 584892 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3214458 of 3214458 (100.00 %)
- decisions: 764324; with two or more candidates: 594771
- paths: {'search': 591192, 'single_legal': 115867, 'take_kill': 33154, 'mulligan': 16384, 'take_kill_roll': 3989, 'holdback_trade': 2397, 'holdback_resample': 1182, 'consensus_lethal': 159}
- candidates per decision (2+): {'n': 594771, 'mean': 5.405, 'median': 5, 'p90': 10, 'p99': 16, 'max': 34}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 185892 of 3060116 (6.07 %)
- gap between the best and second-best aggregate: {'n': 594771, 'mean': 4.86, 'median': 1.675, 'p90': 12.509, 'p99': 45.425, 'max': 160.0}
- hbk ran 6094 times, flipped 1182; hbcheck ran 4912 times; lostrank ran 6300 times (played another move than the first best-scored one: 4908; left out of the search check)
- world line ends over all candidates (25715664; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8657625, 'O': 7849693, 'C': 4036085, 'L': 2157626, 'D': 1477041, 't': 1451780, 'T': 50908, 'c': 34488, 'd': 418}
- seconds per decision, one core: {'n': 764324, 'mean': 0.847, 'median': 0.211, 'p90': 2.568, 'p99': 5.987, 'max': 31.243}
- seconds per search decision, one core: {'n': 591192, 'mean': 1.083, 'median': 0.496, 'p90': 2.946, 'p99': 6.376, 'max': 31.243}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 143435, 'mean': 4.492, 'median': 1.595, 'p90': 13.031, 'p99': 31.772, 'max': 94.083}
- decisions per game: {'n': 8192, 'mean': 93.301, 'median': 90, 'p90': 130, 'p99': 173, 'max': 222}; turns: {'n': 8192, 'mean': 9.034, 'median': 9, 'p90': 12, 'p99': 15, 'max': 22}
- first-seat win rate: 0.5431 over 8192 decisive games
- data: 184.3 MB in 8192 files; archive teach88-games.jsonl.gz 179.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.658 |
| meta-haven-evo | 0.655 |
| meta-portal-evo | 0.564 |
| meta-sword-rally | 0.562 |
| meta-portal-af | 0.535 |
| meta-abyss-midrange | 0.533 |
| meta-dragon-ramp | 0.532 |
| meta-sword-loot | 0.508 |
| meta-portal-cutthroat | 0.489 |
| meta-haven-kukishiro | 0.482 |
| meta-forest-combo | 0.457 |
| meta-haven-amulet | 0.435 |
| meta-rune-spell | 0.420 |
| meta-abyss-aggro | 0.403 |
| meta-dragon-aggro | 0.402 |
| meta-rune-crystal | 0.363 |
