# teach83: teacher data from the served search with the reply keys (runbook 83)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8301
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.48 h in 5 segment(s), 1264.7 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 583828 of 583828 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3207515 of 3207515 (100.00 %)
- decisions: 763405; with two or more candidates: 593426
- paths: {'search': 589777, 'single_legal': 115919, 'take_kill': 33580, 'mulligan': 16384, 'take_kill_roll': 3964, 'holdback_trade': 2525, 'holdback_resample': 1124, 'consensus_lethal': 132}
- candidates per decision (2+): {'n': 593426, 'mean': 5.405, 'median': 5, 'p90': 10, 'p99': 16, 'max': 40}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 188421 of 3054613 (6.17 %)
- gap between the best and second-best aggregate: {'n': 593426, 'mean': 4.897, 'median': 1.675, 'p90': 12.603, 'p99': 45.924, 'max': 160.0}
- hbk ran 6221 times, flipped 1124; hbcheck ran 5097 times; lostrank ran 5949 times (played another move than the first best-scored one: 4565; left out of the search check)
- world line ends over all candidates (25660120; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8664359, 'O': 7805976, 'C': 4023502, 'L': 2132776, 'D': 1494226, 't': 1446658, 'T': 47331, 'c': 44843, 'd': 449}
- seconds per decision, one core: {'n': 763405, 'mean': 0.845, 'median': 0.209, 'p90': 2.562, 'p99': 6.001, 'max': 21.009}
- seconds per search decision, one core: {'n': 589777, 'mean': 1.082, 'median': 0.494, 'p90': 2.944, 'p99': 6.416, 'max': 21.009}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 143082, 'mean': 4.49, 'median': 1.58, 'p90': 13.052, 'p99': 31.807, 'max': 89.853}
- decisions per game: {'n': 8192, 'mean': 93.189, 'median': 90, 'p90': 129, 'p99': 168, 'max': 233}; turns: {'n': 8192, 'mean': 9.015, 'median': 9, 'p90': 12, 'p99': 14, 'max': 22}
- first-seat win rate: 0.553 over 8192 decisive games
- data: 183.9 MB in 8192 files; archive teach83-games.jsonl.gz 179.3 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.643 |
| meta-rune-test-subject | 0.636 |
| meta-abyss-midrange | 0.560 |
| meta-portal-evo | 0.535 |
| meta-portal-af | 0.531 |
| meta-sword-rally | 0.524 |
| meta-dragon-ramp | 0.507 |
| meta-portal-cutthroat | 0.498 |
| meta-haven-kukishiro | 0.488 |
| meta-forest-combo | 0.484 |
| meta-sword-loot | 0.482 |
| meta-rune-spell | 0.451 |
| meta-dragon-aggro | 0.436 |
| meta-abyss-aggro | 0.435 |
| meta-haven-amulet | 0.416 |
| meta-rune-crystal | 0.372 |
