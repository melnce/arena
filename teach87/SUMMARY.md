# teach87: teacher data from the served search with the reply keys (runbook 87)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run87/leaf87q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8701
- games: 7168 of 7168 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 5.75 h in 4 segment(s), 1246.8 games/h
- replay check (every game replayed from its seed and actions): 7168 of 7168 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 505328 of 505328 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2771001 of 2771001 (100.00 %)
- decisions: 661688; with two or more candidates: 513951
- paths: {'search': 510764, 'single_legal': 100729, 'take_kill': 29127, 'mulligan': 14336, 'take_kill_roll': 3428, 'holdback_trade': 2194, 'holdback_resample': 993, 'consensus_lethal': 117}
- candidates per decision (2+): {'n': 513951, 'mean': 5.392, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 160390 of 2637791 (6.08 %)
- gap between the best and second-best aggregate: {'n': 513951, 'mean': 4.881, 'median': 1.684, 'p90': 12.537, 'p99': 45.584, 'max': 160.0}
- hbk ran 5378 times, flipped 993; hbcheck ran 4385 times; lostrank ran 5436 times (played another move than the first best-scored one: 4151; left out of the search check)
- world line ends over all candidates (22168008; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 7493079, 'O': 6774017, 'C': 3470280, 'L': 1837280, 'D': 1268798, 't': 1251940, 'T': 42603, 'c': 29685, 'd': 326}
- seconds per decision, one core: {'n': 661688, 'mean': 0.868, 'median': 0.217, 'p90': 2.626, 'p99': 6.191, 'max': 26.308}
- seconds per search decision, one core: {'n': 510764, 'mean': 1.112, 'median': 0.505, 'p90': 3.023, 'p99': 6.606, 'max': 20.662}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 124572, 'mean': 4.59, 'median': 1.624, 'p90': 13.35, 'p99': 32.798, 'max': 104.898}
- decisions per game: {'n': 7168, 'mean': 92.311, 'median': 89, 'p90': 129, 'p99': 171, 'max': 213}; turns: {'n': 7168, 'mean': 8.97, 'median': 9, 'p90': 12, 'p99': 14, 'max': 21}
- first-seat win rate: 0.5465 over 7168 decisive games
- data: 159.1 MB in 7168 files; archive teach87-games.jsonl.gz 155.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.636 |
| meta-rune-test-subject | 0.634 |
| meta-portal-evo | 0.576 |
| meta-sword-rally | 0.536 |
| meta-abyss-midrange | 0.533 |
| meta-portal-af | 0.531 |
| meta-sword-loot | 0.521 |
| meta-portal-cutthroat | 0.511 |
| meta-dragon-ramp | 0.503 |
| meta-forest-combo | 0.490 |
| meta-haven-kukishiro | 0.487 |
| meta-rune-spell | 0.442 |
| meta-abyss-aggro | 0.436 |
| meta-haven-amulet | 0.424 |
| meta-dragon-aggro | 0.394 |
| meta-rune-crystal | 0.345 |
