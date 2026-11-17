# teach91: teacher data from the served search with the reply keys (runbook 91)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run89/leaf89q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9101
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.3 h in 5 segment(s), 1301.0 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 576561 of 576561 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3157124 of 3157124 (100.00 %)
- decisions: 755230; with two or more candidates: 586220
- paths: {'search': 582608, 'single_legal': 115500, 'take_kill': 33090, 'mulligan': 16384, 'take_kill_roll': 3896, 'holdback_trade': 2475, 'holdback_resample': 1137, 'consensus_lethal': 140}
- candidates per decision (2+): {'n': 586220, 'mean': 5.386, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 181185 of 3005673 (6.03 %)
- gap between the best and second-best aggregate: {'n': 586220, 'mean': 4.893, 'median': 1.685, 'p90': 12.584, 'p99': 45.711, 'max': 160.0}
- hbk ran 6130 times, flipped 1137; hbcheck ran 4993 times; lostrank ran 6047 times (played another move than the first best-scored one: 4673; left out of the search check)
- world line ends over all candidates (25256992; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8550583, 'O': 7718124, 'C': 3925933, 'L': 2105256, 'D': 1441351, 't': 1426464, 'T': 51433, 'c': 37591, 'd': 257}
- seconds per decision, one core: {'n': 755230, 'mean': 0.831, 'median': 0.207, 'p90': 2.52, 'p99': 5.876, 'max': 34.697}
- seconds per search decision, one core: {'n': 582608, 'mean': 1.064, 'median': 0.486, 'p90': 2.895, 'p99': 6.258, 'max': 31.446}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142146, 'mean': 4.393, 'median': 1.546, 'p90': 12.792, 'p99': 31.17, 'max': 89.72}
- decisions per game: {'n': 8192, 'mean': 92.191, 'median': 89, 'p90': 128, 'p99': 171, 'max': 219}; turns: {'n': 8192, 'mean': 8.962, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5552 over 8192 decisive games
- data: 181.5 MB in 8192 files; archive teach91-games.jsonl.gz 176.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.634 |
| meta-rune-test-subject | 0.627 |
| meta-sword-rally | 0.567 |
| meta-portal-af | 0.564 |
| meta-portal-evo | 0.537 |
| meta-abyss-midrange | 0.533 |
| meta-portal-cutthroat | 0.519 |
| meta-dragon-ramp | 0.511 |
| meta-sword-loot | 0.503 |
| meta-haven-kukishiro | 0.494 |
| meta-forest-combo | 0.465 |
| meta-abyss-aggro | 0.436 |
| meta-haven-amulet | 0.432 |
| meta-rune-spell | 0.428 |
| meta-dragon-aggro | 0.398 |
| meta-rune-crystal | 0.352 |
