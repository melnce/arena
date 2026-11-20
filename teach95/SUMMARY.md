# teach95: teacher data from the served search with the reply keys (runbook 95)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9501
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.3 h in 1 segment(s), 1241.4 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 289397 of 289397 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1590458 of 1590458 (100.00 %)
- decisions: 378689; with two or more candidates: 294071
- paths: {'search': 292281, 'single_legal': 57737, 'take_kill': 16747, 'mulligan': 8192, 'take_kill_roll': 1866, 'holdback_trade': 1201, 'holdback_resample': 589, 'consensus_lethal': 76}
- candidates per decision (2+): {'n': 294071, 'mean': 5.408, 'median': 5, 'p90': 10, 'p99': 16, 'max': 44}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91285 of 1514300 (6.03 %)
- gap between the best and second-best aggregate: {'n': 294071, 'mean': 4.897, 'median': 1.681, 'p90': 12.542, 'p99': 46.067, 'max': 160.0}
- hbk ran 3056 times, flipped 589; hbcheck ran 2467 times; lostrank ran 2884 times (played another move than the first best-scored one: 2231; left out of the search check)
- world line ends over all candidates (12723664; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4285982, 'O': 3885705, 'C': 1989701, 'L': 1071505, 'D': 723426, 't': 722798, 'T': 23101, 'c': 21248, 'd': 198}
- seconds per decision, one core: {'n': 378689, 'mean': 0.874, 'median': 0.216, 'p90': 2.658, 'p99': 6.184, 'max': 26.035}
- seconds per search decision, one core: {'n': 292281, 'mean': 1.12, 'median': 0.511, 'p90': 3.054, 'p99': 6.585, 'max': 16.983}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71218, 'mean': 4.625, 'median': 1.63, 'p90': 13.447, 'p99': 33.104, 'max': 78.273}
- decisions per game: {'n': 4096, 'mean': 92.453, 'median': 89, 'p90': 130, 'p99': 171, 'max': 222}; turns: {'n': 4096, 'mean': 8.979, 'median': 9, 'p90': 12, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5566 over 4096 decisive games
- data: 91.2 MB in 4096 files; archive teach95-games.jsonl.gz 88.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.633 |
| meta-rune-test-subject | 0.633 |
| meta-portal-af | 0.574 |
| meta-portal-evo | 0.568 |
| meta-sword-rally | 0.549 |
| meta-abyss-midrange | 0.525 |
| meta-portal-cutthroat | 0.506 |
| meta-sword-loot | 0.504 |
| meta-dragon-ramp | 0.500 |
| meta-haven-kukishiro | 0.494 |
| meta-forest-combo | 0.469 |
| meta-abyss-aggro | 0.434 |
| meta-rune-spell | 0.428 |
| meta-dragon-aggro | 0.420 |
| meta-haven-amulet | 0.416 |
| meta-rune-crystal | 0.348 |
