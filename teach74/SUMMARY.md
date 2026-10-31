# teach74: teacher data from the served search with the reply keys (runbook 74)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run73/leaf73q.json,mull=leaf,mulls=256`
- engine: `a3daa7bef9fd5e9d11b31e1898cdb18d64ffaedd`, seed 7401
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.62 h in 1 segment(s), 1130.9 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 287285 of 287285 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1567898 of 1567898 (100.00 %)
- decisions: 376537; with two or more candidates: 292278
- paths: {'search': 290598, 'single_legal': 57659, 'take_kill': 16433, 'mulligan': 8192, 'take_kill_roll': 1884, 'holdback_trade': 1133, 'holdback_resample': 547, 'consensus_lethal': 91}
- candidates per decision (2+): {'n': 292278, 'mean': 5.364, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 90879 of 1492055 (6.09 %)
- gap between the best and second-best aggregate: {'n': 292278, 'mean': 4.857, 'median': 1.678, 'p90': 12.412, 'p99': 45.327, 'max': 160.0}
- hbk ran 2959 times, flipped 547; hbcheck ran 2412 times; lostrank ran 3313 times (played another move than the first best-scored one: 2524; left out of the search check)
- world line ends over all candidates (12543184; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4241293, 'O': 3843054, 'C': 1955169, 'L': 1048862, 't': 712407, 'D': 703262, 'T': 23488, 'c': 15502, 'd': 147}
- seconds per decision, one core: {'n': 376537, 'mean': 0.965, 'median': 0.241, 'p90': 2.918, 'p99': 6.92, 'max': 30.532}
- seconds per search decision, one core: {'n': 290598, 'mean': 1.236, 'median': 0.556, 'p90': 3.367, 'p99': 7.403, 'max': 25.548}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71403, 'mean': 5.063, 'median': 1.751, 'p90': 14.71, 'p99': 36.625, 'max': 105.929}
- decisions per game: {'n': 4096, 'mean': 91.928, 'median': 89, 'p90': 129, 'p99': 169, 'max': 229}; turns: {'n': 4096, 'mean': 9.0, 'median': 9, 'p90': 12, 'p99': 14, 'max': 20}
- first-seat win rate: 0.5552 over 4096 decisive games
- data: 90.1 MB in 4096 files; archive teach74-games.jsonl.gz 87.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.682 |
| meta-haven-evo | 0.664 |
| meta-sword-rally | 0.562 |
| meta-portal-evo | 0.560 |
| meta-portal-af | 0.557 |
| meta-portal-cutthroat | 0.533 |
| meta-abyss-midrange | 0.510 |
| meta-haven-kukishiro | 0.510 |
| meta-dragon-ramp | 0.496 |
| meta-sword-loot | 0.473 |
| meta-forest-combo | 0.451 |
| meta-dragon-aggro | 0.430 |
| meta-abyss-aggro | 0.418 |
| meta-rune-spell | 0.410 |
| meta-haven-amulet | 0.397 |
| meta-rune-crystal | 0.348 |
