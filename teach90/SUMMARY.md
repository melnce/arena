# teach90: teacher data from the served search with the reply keys (runbook 90)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run89/leaf89q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9001
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.12 h in 1 segment(s), 1314.6 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 286968 of 286968 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1569875 of 1569875 (100.00 %)
- decisions: 375822; with two or more candidates: 291710
- paths: {'search': 289940, 'single_legal': 57446, 'take_kill': 16632, 'mulligan': 8192, 'take_kill_roll': 1762, 'holdback_trade': 1186, 'holdback_resample': 584, 'consensus_lethal': 80}
- candidates per decision (2+): {'n': 291710, 'mean': 5.382, 'median': 5, 'p90': 10, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 90817 of 1493243 (6.08 %)
- gap between the best and second-best aggregate: {'n': 291710, 'mean': 4.918, 'median': 1.681, 'p90': 12.615, 'p99': 46.37, 'max': 160.0}
- hbk ran 3065 times, flipped 584; hbcheck ran 2481 times; lostrank ran 2972 times (played another move than the first best-scored one: 2314; left out of the search check)
- world line ends over all candidates (12559000; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4231462, 'O': 3849277, 'C': 1946216, 'L': 1064647, 'D': 713033, 't': 712015, 'T': 23358, 'c': 18714, 'd': 278}
- seconds per decision, one core: {'n': 375822, 'mean': 0.832, 'median': 0.206, 'p90': 2.516, 'p99': 5.919, 'max': 24.459}
- seconds per search decision, one core: {'n': 289940, 'mean': 1.066, 'median': 0.489, 'p90': 2.89, 'p99': 6.312, 'max': 24.459}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 70902, 'mean': 4.388, 'median': 1.57, 'p90': 12.745, 'p99': 30.701, 'max': 88.813}
- decisions per game: {'n': 4096, 'mean': 91.753, 'median': 89, 'p90': 127, 'p99': 168, 'max': 215}; turns: {'n': 4096, 'mean': 8.939, 'median': 9, 'p90': 11, 'p99': 15, 'max': 17}
- first-seat win rate: 0.5515 over 4096 decisive games
- data: 90.1 MB in 4096 files; archive teach90-games.jsonl.gz 87.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.680 |
| meta-haven-evo | 0.606 |
| meta-abyss-midrange | 0.576 |
| meta-portal-af | 0.572 |
| meta-sword-rally | 0.560 |
| meta-portal-evo | 0.537 |
| meta-portal-cutthroat | 0.533 |
| meta-sword-loot | 0.502 |
| meta-forest-combo | 0.494 |
| meta-haven-kukishiro | 0.494 |
| meta-dragon-ramp | 0.469 |
| meta-abyss-aggro | 0.426 |
| meta-rune-spell | 0.424 |
| meta-dragon-aggro | 0.408 |
| meta-haven-amulet | 0.379 |
| meta-rune-crystal | 0.340 |
