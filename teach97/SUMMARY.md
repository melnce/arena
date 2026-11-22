# teach97: teacher data from the served search with the reply keys (runbook 97)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9701
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.1 h in 1 segment(s), 1320.7 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 289823 of 289823 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1587667 of 1587667 (100.00 %)
- decisions: 379375; with two or more candidates: 294684
- paths: {'search': 292942, 'single_legal': 57960, 'take_kill': 16514, 'mulligan': 8192, 'take_kill_roll': 1956, 'holdback_trade': 1181, 'holdback_resample': 561, 'consensus_lethal': 69}
- candidates per decision (2+): {'n': 294684, 'mean': 5.388, 'median': 5, 'p90': 10, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91820 of 1509687 (6.08 %)
- gap between the best and second-best aggregate: {'n': 294684, 'mean': 4.948, 'median': 1.702, 'p90': 12.764, 'p99': 46.397, 'max': 160.0}
- hbk ran 3004 times, flipped 561; hbcheck ran 2443 times; lostrank ran 3119 times (played another move than the first best-scored one: 2368; left out of the search check)
- world line ends over all candidates (12701336; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4284918, 'O': 3874974, 'C': 1968735, 'L': 1077754, 'D': 730093, 't': 719015, 'T': 26113, 'c': 19499, 'd': 235}
- seconds per decision, one core: {'n': 379375, 'mean': 0.821, 'median': 0.202, 'p90': 2.496, 'p99': 5.776, 'max': 18.095}
- seconds per search decision, one core: {'n': 292942, 'mean': 1.052, 'median': 0.48, 'p90': 2.866, 'p99': 6.163, 'max': 18.095}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71238, 'mean': 4.353, 'median': 1.543, 'p90': 12.696, 'p99': 30.465, 'max': 83.071}
- decisions per game: {'n': 4096, 'mean': 92.621, 'median': 90, 'p90': 129, 'p99': 169, 'max': 216}; turns: {'n': 4096, 'mean': 8.983, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.554 over 4096 decisive games
- data: 91.0 MB in 4096 files; archive teach97-games.jsonl.gz 88.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.674 |
| meta-rune-test-subject | 0.656 |
| meta-portal-evo | 0.560 |
| meta-sword-rally | 0.560 |
| meta-abyss-midrange | 0.553 |
| meta-portal-af | 0.545 |
| meta-portal-cutthroat | 0.533 |
| meta-sword-loot | 0.506 |
| meta-dragon-ramp | 0.480 |
| meta-forest-combo | 0.475 |
| meta-abyss-aggro | 0.445 |
| meta-haven-kukishiro | 0.443 |
| meta-dragon-aggro | 0.430 |
| meta-haven-amulet | 0.430 |
| meta-rune-spell | 0.381 |
| meta-rune-crystal | 0.328 |
