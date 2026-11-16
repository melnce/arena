# teach90: teacher data from the served search with the reply keys (runbook 90)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run89/leaf89q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9001
- games: 7168 of 7168 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 5.51 h in 4 segment(s), 1299.9 games/h
- replay check (every game replayed from its seed and actions): 7168 of 7168 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 504465 of 504465 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2763010 of 2763010 (100.00 %)
- decisions: 660424; with two or more candidates: 512995
- paths: {'search': 509922, 'single_legal': 100713, 'take_kill': 29119, 'mulligan': 14336, 'take_kill_roll': 3122, 'holdback_trade': 2090, 'holdback_resample': 983, 'consensus_lethal': 139}
- candidates per decision (2+): {'n': 512995, 'mean': 5.386, 'median': 5, 'p90': 10, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 159860 of 2627947 (6.08 %)
- gap between the best and second-best aggregate: {'n': 512995, 'mean': 4.905, 'median': 1.682, 'p90': 12.557, 'p99': 46.194, 'max': 160.0}
- hbk ran 5319 times, flipped 983; hbcheck ran 4336 times; lostrank ran 5457 times (played another move than the first best-scored one: 4205; left out of the search check)
- world line ends over all candidates (22104080; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 7449722, 'O': 6762557, 'C': 3441241, 'L': 1859236, 'D': 1264270, 't': 1252945, 'T': 42771, 'c': 30933, 'd': 405}
- seconds per decision, one core: {'n': 660424, 'mean': 0.835, 'median': 0.207, 'p90': 2.527, 'p99': 5.934, 'max': 30.348}
- seconds per search decision, one core: {'n': 509922, 'mean': 1.069, 'median': 0.488, 'p90': 2.903, 'p99': 6.324, 'max': 24.459}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 124469, 'mean': 4.407, 'median': 1.564, 'p90': 12.796, 'p99': 31.011, 'max': 99.422}
- decisions per game: {'n': 7168, 'mean': 92.135, 'median': 89, 'p90': 128, 'p99': 169, 'max': 215}; turns: {'n': 7168, 'mean': 8.971, 'median': 9, 'p90': 12, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5601 over 7168 decisive games
- data: 158.6 MB in 7168 files; archive teach90-games.jsonl.gz 154.5 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.671 |
| meta-haven-evo | 0.614 |
| meta-abyss-midrange | 0.578 |
| meta-portal-af | 0.555 |
| meta-portal-cutthroat | 0.543 |
| meta-portal-evo | 0.540 |
| meta-sword-rally | 0.536 |
| meta-sword-loot | 0.508 |
| meta-dragon-ramp | 0.491 |
| meta-forest-combo | 0.489 |
| meta-haven-kukishiro | 0.473 |
| meta-rune-spell | 0.441 |
| meta-abyss-aggro | 0.439 |
| meta-dragon-aggro | 0.401 |
| meta-haven-amulet | 0.381 |
| meta-rune-crystal | 0.342 |
