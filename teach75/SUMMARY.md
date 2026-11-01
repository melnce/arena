# teach75: teacher data from the served search with the reply keys (runbook 75)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7501
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.56 h in 5 segment(s), 1248.8 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 579543 of 579543 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3161012 of 3161012 (100.00 %)
- decisions: 759479; with two or more candidates: 589600
- paths: {'search': 585932, 'single_legal': 116815, 'take_kill': 32755, 'mulligan': 16384, 'take_kill_roll': 3777, 'holdback_trade': 2485, 'holdback_resample': 1183, 'consensus_lethal': 148}
- candidates per decision (2+): {'n': 589600, 'mean': 5.361, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 183961 of 3008415 (6.11 %)
- gap between the best and second-best aggregate: {'n': 589600, 'mean': 4.896, 'median': 1.695, 'p90': 12.56, 'p99': 46.179, 'max': 160.0}
- hbk ran 6199 times, flipped 1183; hbcheck ran 5016 times; lostrank ran 6389 times (played another move than the first best-scored one: 4902; left out of the search check)
- world line ends over all candidates (25288096; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8556909, 'O': 7731328, 'C': 3928146, 'L': 2125582, 't': 1443187, 'D': 1424408, 'T': 45789, 'c': 32227, 'd': 520}
- seconds per decision, one core: {'n': 759479, 'mean': 0.861, 'median': 0.214, 'p90': 2.606, 'p99': 6.156, 'max': 33.91}
- seconds per search decision, one core: {'n': 585932, 'mean': 1.103, 'median': 0.502, 'p90': 2.996, 'p99': 6.579, 'max': 26.77}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 143409, 'mean': 4.539, 'median': 1.606, 'p90': 13.08, 'p99': 32.466, 'max': 132.742}
- decisions per game: {'n': 8192, 'mean': 92.71, 'median': 90, 'p90': 129, 'p99': 170, 'max': 229}; turns: {'n': 8192, 'mean': 9.036, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5521 over 8192 decisive games
- data: 181.6 MB in 8192 files; archive teach75-games.jsonl.gz 177.0 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.642 |
| meta-rune-test-subject | 0.617 |
| meta-portal-evo | 0.582 |
| meta-sword-rally | 0.557 |
| meta-portal-af | 0.554 |
| meta-abyss-midrange | 0.538 |
| meta-portal-cutthroat | 0.536 |
| meta-haven-kukishiro | 0.516 |
| meta-dragon-ramp | 0.507 |
| meta-sword-loot | 0.493 |
| meta-forest-combo | 0.464 |
| meta-rune-spell | 0.417 |
| meta-dragon-aggro | 0.415 |
| meta-abyss-aggro | 0.413 |
| meta-haven-amulet | 0.410 |
| meta-rune-crystal | 0.340 |
