# teach96: teacher data from the served search with the reply keys (runbook 96)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9601
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.1 h in 1 segment(s), 1320.9 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290159 of 290159 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1596198 of 1596198 (100.00 %)
- decisions: 379906; with two or more candidates: 295099
- paths: {'search': 293327, 'single_legal': 58058, 'take_kill': 16419, 'mulligan': 8192, 'take_kill_roll': 2059, 'holdback_trade': 1228, 'holdback_resample': 544, 'consensus_lethal': 79}
- candidates per decision (2+): {'n': 295099, 'mean': 5.409, 'median': 5, 'p90': 10, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92943 of 1518085 (6.12 %)
- gap between the best and second-best aggregate: {'n': 295099, 'mean': 4.887, 'median': 1.675, 'p90': 12.558, 'p99': 45.927, 'max': 160.0}
- hbk ran 3029 times, flipped 544; hbcheck ran 2485 times; lostrank ran 3168 times (played another move than the first best-scored one: 2386; left out of the search check)
- world line ends over all candidates (12769584; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4300303, 'O': 3890910, 'C': 2004538, 'L': 1076758, 't': 725929, 'D': 725894, 'T': 23719, 'c': 21315, 'd': 218}
- seconds per decision, one core: {'n': 379906, 'mean': 0.819, 'median': 0.201, 'p90': 2.481, 'p99': 5.806, 'max': 26.597}
- seconds per search decision, one core: {'n': 293327, 'mean': 1.049, 'median': 0.481, 'p90': 2.844, 'p99': 6.2, 'max': 24.175}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71474, 'mean': 4.332, 'median': 1.517, 'p90': 12.509, 'p99': 31.217, 'max': 91.925}
- decisions per game: {'n': 4096, 'mean': 92.75, 'median': 90, 'p90': 129, 'p99': 168, 'max': 226}; turns: {'n': 4096, 'mean': 9.002, 'median': 9, 'p90': 12, 'p99': 15, 'max': 20}
- first-seat win rate: 0.5398 over 4096 decisive games
- data: 91.4 MB in 4096 files; archive teach96-games.jsonl.gz 89.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.629 |
| meta-haven-evo | 0.619 |
| meta-portal-af | 0.598 |
| meta-sword-rally | 0.555 |
| meta-portal-evo | 0.553 |
| meta-portal-cutthroat | 0.549 |
| meta-dragon-ramp | 0.529 |
| meta-sword-loot | 0.516 |
| meta-abyss-midrange | 0.504 |
| meta-haven-kukishiro | 0.504 |
| meta-forest-combo | 0.451 |
| meta-rune-spell | 0.428 |
| meta-haven-amulet | 0.416 |
| meta-abyss-aggro | 0.406 |
| meta-rune-crystal | 0.377 |
| meta-dragon-aggro | 0.367 |
