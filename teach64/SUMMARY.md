# teach64: teacher data from the served search with the reply keys (runbook 64)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run64/leaf64h.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6401
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 4.18 h in 1 segment(s), 980.8 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 292412 of 292412 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1562251 of 1562251 (100.00 %)
- decisions: 384754; with two or more candidates: 297469
- paths: {'search': 295904, 'single_legal': 60965, 'take_kill': 16240, 'mulligan': 8192, 'take_kill_roll': 1832, 'holdback_trade': 854, 'holdback_resample': 711, 'consensus_lethal': 56}
- candidates per decision (2+): {'n': 297469, 'mean': 5.252, 'median': 4, 'p90': 9, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91359 of 1489215 (6.13 %)
- gap between the best and second-best aggregate: {'n': 297469, 'mean': 4.9, 'median': 1.773, 'p90': 12.586, 'p99': 43.7, 'max': 160.0}
- hbk ran 2695 times, flipped 711; hbcheck ran 1984 times; lostrank ran 3492 times (played another move than the first best-scored one: 2629; left out of the search check)
- world line ends over all candidates (12498008; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'C': 3713308, 'o': 3547712, 'O': 2176968, 'L': 987074, 'N': 970146, 't': 598638, 'D': 469717, 'T': 22301, 'c': 12043, 'd': 101}
- seconds per decision, one core: {'n': 384754, 'mean': 1.089, 'median': 0.452, 'p90': 3.105, 'p99': 7.014, 'max': 24.038}
- seconds per search decision, one core: {'n': 295904, 'mean': 1.407, 'median': 0.81, 'p90': 3.564, 'p99': 7.447, 'max': 21.902}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 72223, 'mean': 5.803, 'median': 2.476, 'p90': 16.285, 'p99': 38.438, 'max': 98.668}
- decisions per game: {'n': 4096, 'mean': 93.934, 'median': 91, 'p90': 131, 'p99': 178, 'max': 251}; turns: {'n': 4096, 'mean': 9.101, 'median': 9, 'p90': 12, 'p99': 15, 'max': 22}
- first-seat win rate: 0.554 over 4096 decisive games
- data: 89.2 MB in 4096 files; archive teach64-games.jsonl.gz 87.0 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.627 |
| meta-haven-evo | 0.623 |
| meta-sword-rally | 0.615 |
| meta-portal-evo | 0.565 |
| meta-abyss-midrange | 0.562 |
| meta-portal-af | 0.560 |
| meta-portal-cutthroat | 0.504 |
| meta-sword-loot | 0.494 |
| meta-dragon-ramp | 0.488 |
| meta-forest-combo | 0.482 |
| meta-dragon-aggro | 0.461 |
| meta-abyss-aggro | 0.453 |
| meta-haven-kukishiro | 0.451 |
| meta-rune-spell | 0.393 |
| meta-haven-amulet | 0.381 |
| meta-rune-crystal | 0.340 |
