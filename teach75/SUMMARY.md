# teach75: teacher data from the served search with the reply keys (runbook 75)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7501
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.25 h in 1 segment(s), 1261.9 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290210 of 290210 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1582691 of 1582691 (100.00 %)
- decisions: 380170; with two or more candidates: 295230
- paths: {'search': 293417, 'single_legal': 58380, 'take_kill': 16437, 'mulligan': 8192, 'take_kill_roll': 1845, 'holdback_trade': 1231, 'holdback_resample': 582, 'consensus_lethal': 86}
- candidates per decision (2+): {'n': 295230, 'mean': 5.361, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92262 of 1505972 (6.13 %)
- gap between the best and second-best aggregate: {'n': 295230, 'mean': 4.878, 'median': 1.68, 'p90': 12.475, 'p99': 46.309, 'max': 160.0}
- hbk ran 3073 times, flipped 582; hbcheck ran 2491 times; lostrank ran 3207 times (played another move than the first best-scored one: 2496; left out of the search check)
- world line ends over all candidates (12661528; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4274816, 'O': 3877017, 'C': 1962298, 'L': 1071121, 't': 722600, 'D': 714241, 'T': 22566, 'c': 16627, 'd': 242}
- seconds per decision, one core: {'n': 380170, 'mean': 0.857, 'median': 0.211, 'p90': 2.602, 'p99': 6.1, 'max': 24.192}
- seconds per search decision, one core: {'n': 293417, 'mean': 1.098, 'median': 0.499, 'p90': 2.988, 'p99': 6.523, 'max': 18.169}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71681, 'mean': 4.524, 'median': 1.597, 'p90': 13.08, 'p99': 32.42, 'max': 102.573}
- decisions per game: {'n': 4096, 'mean': 92.815, 'median': 90, 'p90': 130, 'p99': 171, 'max': 229}; turns: {'n': 4096, 'mean': 9.03, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5459 over 4096 decisive games
- data: 90.9 MB in 4096 files; archive teach75-games.jsonl.gz 88.6 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.639 |
| meta-portal-evo | 0.606 |
| meta-rune-test-subject | 0.602 |
| meta-portal-af | 0.562 |
| meta-portal-cutthroat | 0.557 |
| meta-abyss-midrange | 0.537 |
| meta-sword-rally | 0.533 |
| meta-dragon-ramp | 0.508 |
| meta-haven-kukishiro | 0.494 |
| meta-sword-loot | 0.494 |
| meta-forest-combo | 0.463 |
| meta-dragon-aggro | 0.432 |
| meta-haven-amulet | 0.416 |
| meta-rune-spell | 0.416 |
| meta-abyss-aggro | 0.404 |
| meta-rune-crystal | 0.338 |
