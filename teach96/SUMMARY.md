# teach96: teacher data from the served search with the reply keys (runbook 96)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9601
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.37 h in 5 segment(s), 1285.8 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 579977 of 579977 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3188525 of 3188525 (100.00 %)
- decisions: 758972; with two or more candidates: 589841
- paths: {'search': 586244, 'single_legal': 115928, 'take_kill': 32673, 'mulligan': 16384, 'take_kill_roll': 3989, 'holdback_trade': 2460, 'holdback_resample': 1137, 'consensus_lethal': 157}
- candidates per decision (2+): {'n': 589841, 'mean': 5.406, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 186426 of 3033152 (6.15 %)
- gap between the best and second-best aggregate: {'n': 589841, 'mean': 4.914, 'median': 1.676, 'p90': 12.639, 'p99': 46.314, 'max': 160.0}
- hbk ran 6140 times, flipped 1137; hbcheck ran 5003 times; lostrank ran 6267 times (played another move than the first best-scored one: 4740; left out of the search check)
- world line ends over all candidates (25508200; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8601149, 'O': 7774282, 'C': 4003684, 'L': 2139917, 't': 1450351, 'D': 1448838, 'T': 50583, 'c': 39036, 'd': 360}
- seconds per decision, one core: {'n': 758972, 'mean': 0.836, 'median': 0.207, 'p90': 2.53, 'p99': 5.959, 'max': 26.597}
- seconds per search decision, one core: {'n': 586244, 'mean': 1.07, 'median': 0.489, 'p90': 2.902, 'p99': 6.359, 'max': 24.175}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142844, 'mean': 4.422, 'median': 1.563, 'p90': 12.797, 'p99': 31.681, 'max': 91.925}
- decisions per game: {'n': 8192, 'mean': 92.648, 'median': 90, 'p90': 129, 'p99': 170, 'max': 226}; turns: {'n': 8192, 'mean': 8.992, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.5299 over 8192 decisive games
- data: 182.6 MB in 8192 files; archive teach96-games.jsonl.gz 178.0 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.649 |
| meta-haven-evo | 0.629 |
| meta-portal-af | 0.564 |
| meta-sword-rally | 0.564 |
| meta-portal-evo | 0.551 |
| meta-portal-cutthroat | 0.544 |
| meta-abyss-midrange | 0.523 |
| meta-dragon-ramp | 0.523 |
| meta-sword-loot | 0.505 |
| meta-haven-kukishiro | 0.488 |
| meta-forest-combo | 0.446 |
| meta-abyss-aggro | 0.426 |
| meta-rune-spell | 0.412 |
| meta-haven-amulet | 0.409 |
| meta-dragon-aggro | 0.397 |
| meta-rune-crystal | 0.370 |
