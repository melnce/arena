# teach91: teacher data from the served search with the reply keys (runbook 91)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run89/leaf89q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9101
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.16 h in 1 segment(s), 1297.4 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 289608 of 289608 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1586582 of 1586582 (100.00 %)
- decisions: 378909; with two or more candidates: 294452
- paths: {'search': 292650, 'single_legal': 57770, 'take_kill': 16610, 'mulligan': 8192, 'take_kill_roll': 1806, 'holdback_trade': 1231, 'holdback_resample': 571, 'consensus_lethal': 79}
- candidates per decision (2+): {'n': 294452, 'mean': 5.388, 'median': 5, 'p90': 10, 'p99': 16, 'max': 34}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91036 of 1510724 (6.03 %)
- gap between the best and second-best aggregate: {'n': 294452, 'mean': 4.883, 'median': 1.683, 'p90': 12.606, 'p99': 45.54, 'max': 160.0}
- hbk ran 3081 times, flipped 571; hbcheck ran 2510 times; lostrank ran 3042 times (played another move than the first best-scored one: 2327; left out of the search check)
- world line ends over all candidates (12692656; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4293213, 'O': 3878529, 'C': 1977114, 'L': 1053819, 'D': 728332, 't': 717373, 'T': 27071, 'c': 17120, 'd': 85}
- seconds per decision, one core: {'n': 378909, 'mean': 0.837, 'median': 0.207, 'p90': 2.539, 'p99': 5.912, 'max': 29.175}
- seconds per search decision, one core: {'n': 292650, 'mean': 1.071, 'median': 0.488, 'p90': 2.918, 'p99': 6.303, 'max': 18.718}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71160, 'mean': 4.436, 'median': 1.549, 'p90': 12.998, 'p99': 31.119, 'max': 89.72}
- decisions per game: {'n': 4096, 'mean': 92.507, 'median': 89, 'p90': 129, 'p99': 170, 'max': 219}; turns: {'n': 4096, 'mean': 8.972, 'median': 9, 'p90': 12, 'p99': 14, 'max': 17}
- first-seat win rate: 0.5525 over 4096 decisive games
- data: 91.1 MB in 4096 files; archive teach91-games.jsonl.gz 88.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.641 |
| meta-rune-test-subject | 0.621 |
| meta-abyss-midrange | 0.551 |
| meta-portal-af | 0.541 |
| meta-sword-rally | 0.535 |
| meta-portal-evo | 0.527 |
| meta-dragon-ramp | 0.525 |
| meta-sword-loot | 0.510 |
| meta-portal-cutthroat | 0.492 |
| meta-haven-kukishiro | 0.490 |
| meta-forest-combo | 0.477 |
| meta-abyss-aggro | 0.443 |
| meta-rune-spell | 0.441 |
| meta-haven-amulet | 0.440 |
| meta-dragon-aggro | 0.416 |
| meta-rune-crystal | 0.350 |
