# teach84: teacher data from the served search with the reply keys (runbook 84)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8401
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.2 h in 5 segment(s), 1322.2 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 580248 of 580248 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3183432 of 3183432 (100.00 %)
- decisions: 759645; with two or more candidates: 590023
- paths: {'search': 586394, 'single_legal': 116099, 'take_kill': 33331, 'mulligan': 16384, 'take_kill_roll': 3669, 'holdback_trade': 2422, 'holdback_resample': 1207, 'consensus_lethal': 139}
- candidates per decision (2+): {'n': 590023, 'mean': 5.395, 'median': 5, 'p90': 10, 'p99': 16, 'max': 35}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 184512 of 3029170 (6.09 %)
- gap between the best and second-best aggregate: {'n': 590023, 'mean': 4.926, 'median': 1.704, 'p90': 12.666, 'p99': 45.993, 'max': 160.0}
- hbk ran 6099 times, flipped 1207; hbcheck ran 4892 times; lostrank ran 6146 times (played another move than the first best-scored one: 4666; left out of the search check)
- world line ends over all candidates (25467456; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8580585, 'O': 7769213, 'C': 3978198, 'L': 2150173, 'D': 1465975, 't': 1437548, 'T': 50485, 'c': 34913, 'd': 366}
- seconds per decision, one core: {'n': 759645, 'mean': 0.814, 'median': 0.198, 'p90': 2.469, 'p99': 5.794, 'max': 25.059}
- seconds per search decision, one core: {'n': 586394, 'mean': 1.043, 'median': 0.475, 'p90': 2.84, 'p99': 6.191, 'max': 25.059}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142819, 'mean': 4.309, 'median': 1.493, 'p90': 12.572, 'p99': 30.706, 'max': 116.33}
- decisions per game: {'n': 8192, 'mean': 92.73, 'median': 90, 'p90': 129, 'p99': 168, 'max': 224}; turns: {'n': 8192, 'mean': 9.0, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.55 over 8192 decisive games
- data: 182.6 MB in 8192 files; archive teach84-games.jsonl.gz 178.0 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.646 |
| meta-haven-evo | 0.626 |
| meta-portal-evo | 0.551 |
| meta-sword-rally | 0.543 |
| meta-abyss-midrange | 0.540 |
| meta-portal-af | 0.538 |
| meta-portal-cutthroat | 0.537 |
| meta-sword-loot | 0.499 |
| meta-dragon-ramp | 0.498 |
| meta-haven-kukishiro | 0.485 |
| meta-forest-combo | 0.460 |
| meta-abyss-aggro | 0.440 |
| meta-rune-spell | 0.427 |
| meta-dragon-aggro | 0.416 |
| meta-haven-amulet | 0.406 |
| meta-rune-crystal | 0.387 |
