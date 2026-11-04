# teach78: teacher data from the served search with the reply keys (runbook 78)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7801
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.01 h in 1 segment(s), 1362.0 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 287518 of 287518 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1569034 of 1569034 (100.00 %)
- decisions: 377209; with two or more candidates: 292430
- paths: {'search': 290619, 'single_legal': 58076, 'take_kill': 16469, 'mulligan': 8192, 'take_kill_roll': 1968, 'holdback_trade': 1248, 'holdback_resample': 563, 'consensus_lethal': 74}
- candidates per decision (2+): {'n': 292430, 'mean': 5.366, 'median': 5, 'p90': 10, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92115 of 1493633 (6.17 %)
- gap between the best and second-best aggregate: {'n': 292430, 'mean': 4.911, 'median': 1.685, 'p90': 12.625, 'p99': 46.603, 'max': 160.0}
- hbk ran 3089 times, flipped 563; hbcheck ran 2526 times; lostrank ran 3101 times (played another move than the first best-scored one: 2331; left out of the search check)
- world line ends over all candidates (12552272; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4257412, 'O': 3845843, 'C': 1926875, 'L': 1053539, 'D': 717984, 't': 713976, 'T': 20022, 'c': 16368, 'd': 253}
- seconds per decision, one core: {'n': 377209, 'mean': 0.8, 'median': 0.195, 'p90': 2.431, 'p99': 5.695, 'max': 27.431}
- seconds per search decision, one core: {'n': 290619, 'mean': 1.027, 'median': 0.467, 'p90': 2.794, 'p99': 6.084, 'max': 25.797}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71312, 'mean': 4.212, 'median': 1.453, 'p90': 12.315, 'p99': 29.842, 'max': 74.496}
- decisions per game: {'n': 4096, 'mean': 92.092, 'median': 89, 'p90': 128, 'p99': 167, 'max': 209}; turns: {'n': 4096, 'mean': 8.992, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5603 over 4096 decisive games
- data: 90.1 MB in 4096 files; archive teach78-games.jsonl.gz 87.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.676 |
| meta-haven-evo | 0.648 |
| meta-portal-evo | 0.562 |
| meta-sword-rally | 0.562 |
| meta-portal-af | 0.537 |
| meta-portal-cutthroat | 0.516 |
| meta-forest-combo | 0.508 |
| meta-abyss-midrange | 0.502 |
| meta-sword-loot | 0.488 |
| meta-dragon-ramp | 0.480 |
| meta-haven-kukishiro | 0.467 |
| meta-abyss-aggro | 0.459 |
| meta-rune-spell | 0.424 |
| meta-haven-amulet | 0.422 |
| meta-dragon-aggro | 0.410 |
| meta-rune-crystal | 0.338 |
