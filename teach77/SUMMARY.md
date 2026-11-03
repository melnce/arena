# teach77: teacher data from the served search with the reply keys (runbook 77)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7701
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.16 h in 1 segment(s), 1296.0 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290801 of 290801 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1586418 of 1586418 (100.00 %)
- decisions: 381417; with two or more candidates: 295700
- paths: {'search': 293894, 'single_legal': 58728, 'take_kill': 16657, 'mulligan': 8192, 'take_kill_roll': 2080, 'holdback_trade': 1211, 'holdback_resample': 595, 'consensus_lethal': 60}
- candidates per decision (2+): {'n': 295700, 'mean': 5.365, 'median': 5, 'p90': 10, 'p99': 16, 'max': 35}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91288 of 1509760 (6.05 %)
- gap between the best and second-best aggregate: {'n': 295700, 'mean': 4.922, 'median': 1.704, 'p90': 12.637, 'p99': 45.691, 'max': 160.0}
- hbk ran 3060 times, flipped 595; hbcheck ran 2465 times; lostrank ran 3093 times (played another move than the first best-scored one: 2366; left out of the search check)
- world line ends over all candidates (12691344; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4289701, 'O': 3894555, 'C': 1954204, 'L': 1073019, 'D': 723751, 't': 718183, 'T': 24009, 'c': 13744, 'd': 178}
- seconds per decision, one core: {'n': 381417, 'mean': 0.831, 'median': 0.204, 'p90': 2.526, 'p99': 5.878, 'max': 20.997}
- seconds per search decision, one core: {'n': 293894, 'mean': 1.066, 'median': 0.486, 'p90': 2.895, 'p99': 6.269, 'max': 18.485}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71800, 'mean': 4.394, 'median': 1.574, 'p90': 12.689, 'p99': 31.084, 'max': 114.203}
- decisions per game: {'n': 4096, 'mean': 93.119, 'median': 91, 'p90': 130, 'p99': 175, 'max': 219}; turns: {'n': 4096, 'mean': 9.048, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5508 over 4096 decisive games
- data: 91.2 MB in 4096 files; archive teach77-games.jsonl.gz 88.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.672 |
| meta-rune-test-subject | 0.637 |
| meta-sword-rally | 0.580 |
| meta-portal-af | 0.555 |
| meta-portal-cutthroat | 0.535 |
| meta-portal-evo | 0.533 |
| meta-dragon-ramp | 0.518 |
| meta-abyss-midrange | 0.516 |
| meta-sword-loot | 0.492 |
| meta-haven-kukishiro | 0.486 |
| meta-forest-combo | 0.465 |
| meta-haven-amulet | 0.441 |
| meta-rune-spell | 0.426 |
| meta-abyss-aggro | 0.418 |
| meta-dragon-aggro | 0.383 |
| meta-rune-crystal | 0.344 |
