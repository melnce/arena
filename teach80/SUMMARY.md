# teach80: teacher data from the served search with the reply keys (runbook 80)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8001
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.04 h in 1 segment(s), 1347.0 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 287130 of 287130 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1571230 of 1571230 (100.00 %)
- decisions: 376272; with two or more candidates: 292115
- paths: {'search': 290290, 'single_legal': 57722, 'take_kill': 16356, 'mulligan': 8192, 'take_kill_roll': 1807, 'holdback_trade': 1243, 'holdback_resample': 582, 'consensus_lethal': 80}
- candidates per decision (2+): {'n': 292115, 'mean': 5.379, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 90885 of 1495721 (6.08 %)
- gap between the best and second-best aggregate: {'n': 292115, 'mean': 4.842, 'median': 1.657, 'p90': 12.507, 'p99': 45.199, 'max': 160.0}
- hbk ran 3100 times, flipped 582; hbcheck ran 2518 times; lostrank ran 3160 times (played another move than the first best-scored one: 2354; left out of the search check)
- world line ends over all candidates (12569840; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4248830, 'O': 3836758, 'C': 1953341, 'L': 1051729, 'D': 719711, 't': 719352, 'T': 23180, 'c': 16757, 'd': 182}
- seconds per decision, one core: {'n': 376272, 'mean': 0.811, 'median': 0.2, 'p90': 2.456, 'p99': 5.796, 'max': 32.406}
- seconds per search decision, one core: {'n': 290290, 'mean': 1.039, 'median': 0.471, 'p90': 2.821, 'p99': 6.191, 'max': 32.406}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71073, 'mean': 4.273, 'median': 1.504, 'p90': 12.447, 'p99': 30.395, 'max': 87.434}
- decisions per game: {'n': 4096, 'mean': 91.863, 'median': 89, 'p90': 127, 'p99': 168, 'max': 225}; turns: {'n': 4096, 'mean': 8.967, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5686 over 4096 decisive games
- data: 90.2 MB in 4096 files; archive teach80-games.jsonl.gz 87.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.643 |
| meta-haven-evo | 0.639 |
| meta-portal-evo | 0.594 |
| meta-abyss-midrange | 0.562 |
| meta-sword-rally | 0.560 |
| meta-dragon-ramp | 0.541 |
| meta-portal-cutthroat | 0.533 |
| meta-portal-af | 0.525 |
| meta-haven-kukishiro | 0.496 |
| meta-sword-loot | 0.480 |
| meta-forest-combo | 0.465 |
| meta-dragon-aggro | 0.432 |
| meta-abyss-aggro | 0.414 |
| meta-rune-spell | 0.395 |
| meta-haven-amulet | 0.391 |
| meta-rune-crystal | 0.330 |
