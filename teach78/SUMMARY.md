# teach78: teacher data from the served search with the reply keys (runbook 78)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7801
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.16 h in 5 segment(s), 1330.4 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 577793 of 577793 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3159172 of 3159172 (100.00 %)
- decisions: 757485; with two or more candidates: 587771
- paths: {'search': 584110, 'single_legal': 116270, 'take_kill': 33142, 'mulligan': 16384, 'take_kill_roll': 3770, 'holdback_trade': 2509, 'holdback_resample': 1152, 'consensus_lethal': 148}
- candidates per decision (2+): {'n': 587771, 'mean': 5.375, 'median': 5, 'p90': 10, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 185031 of 3007105 (6.15 %)
- gap between the best and second-best aggregate: {'n': 587771, 'mean': 4.868, 'median': 1.681, 'p90': 12.505, 'p99': 45.897, 'max': 160.0}
- hbk ran 6170 times, flipped 1152; hbcheck ran 5018 times; lostrank ran 6317 times (played another move than the first best-scored one: 4800; left out of the search check)
- world line ends over all candidates (25273376; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8561276, 'O': 7755559, 'C': 3900272, 'L': 2104906, 'D': 1441747, 't': 1429608, 'T': 45201, 'c': 34371, 'd': 436}
- seconds per decision, one core: {'n': 757485, 'mean': 0.811, 'median': 0.199, 'p90': 2.463, 'p99': 5.756, 'max': 27.431}
- seconds per search decision, one core: {'n': 584110, 'mean': 1.04, 'median': 0.474, 'p90': 2.825, 'p99': 6.152, 'max': 25.797}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142963, 'mean': 4.277, 'median': 1.492, 'p90': 12.425, 'p99': 30.348, 'max': 75.385}
- decisions per game: {'n': 8192, 'mean': 92.466, 'median': 90, 'p90': 128, 'p99': 168, 'max': 209}; turns: {'n': 8192, 'mean': 9.013, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.5598 over 8192 decisive games
- data: 181.4 MB in 8192 files; archive teach78-games.jsonl.gz 176.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.652 |
| meta-rune-test-subject | 0.647 |
| meta-portal-af | 0.557 |
| meta-portal-evo | 0.557 |
| meta-sword-rally | 0.550 |
| meta-abyss-midrange | 0.527 |
| meta-portal-cutthroat | 0.507 |
| meta-sword-loot | 0.493 |
| meta-dragon-ramp | 0.487 |
| meta-forest-combo | 0.483 |
| meta-haven-kukishiro | 0.466 |
| meta-abyss-aggro | 0.443 |
| meta-rune-spell | 0.439 |
| meta-haven-amulet | 0.429 |
| meta-dragon-aggro | 0.414 |
| meta-rune-crystal | 0.349 |
