# teach65: teacher data from the served search with the reply keys (runbook 65)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run65/leaf65q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6501
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 4.65 h in 1 segment(s), 880.6 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 299785 of 299785 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1615840 of 1615840 (100.00 %)
- decisions: 391851; with two or more candidates: 305004
- paths: {'search': 303374, 'single_legal': 60605, 'take_kill': 16225, 'mulligan': 8192, 'take_kill_roll': 1764, 'holdback_trade': 947, 'holdback_resample': 683, 'consensus_lethal': 61}
- candidates per decision (2+): {'n': 305004, 'mean': 5.298, 'median': 5, 'p90': 9, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92196 of 1540230 (5.99 %)
- gap between the best and second-best aggregate: {'n': 305004, 'mean': 4.955, 'median': 1.789, 'p90': 12.714, 'p99': 44.882, 'max': 160.0}
- hbk ran 2842 times, flipped 683; hbcheck ran 2159 times; lostrank ran 3589 times (played another move than the first best-scored one: 2683; left out of the search check)
- world line ends over all candidates (12926720; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'C': 3923466, 'o': 3590855, 'O': 2222830, 'L': 1032777, 'N': 1020480, 't': 614619, 'D': 490023, 'T': 20220, 'c': 11319, 'd': 131}
- seconds per decision, one core: {'n': 391851, 'mean': 1.192, 'median': 0.509, 'p90': 3.381, 'p99': 7.554, 'max': 25.82}
- seconds per search decision, one core: {'n': 303374, 'mean': 1.529, 'median': 0.886, 'p90': 3.866, 'p99': 8.005, 'max': 25.82}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 72808, 'mean': 6.414, 'median': 2.867, 'p90': 17.855, 'p99': 40.86, 'max': 98.55}
- decisions per game: {'n': 4096, 'mean': 95.667, 'median': 92, 'p90': 134, 'p99': 180, 'max': 224}; turns: {'n': 4096, 'mean': 9.171, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.5522 over 4096 decisive games
- data: 92.2 MB in 4096 files; archive teach65-games.jsonl.gz 89.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.650 |
| meta-rune-test-subject | 0.613 |
| meta-portal-af | 0.588 |
| meta-portal-evo | 0.586 |
| meta-abyss-midrange | 0.582 |
| meta-sword-rally | 0.560 |
| meta-sword-loot | 0.518 |
| meta-portal-cutthroat | 0.504 |
| meta-dragon-ramp | 0.482 |
| meta-forest-combo | 0.455 |
| meta-abyss-aggro | 0.435 |
| meta-dragon-aggro | 0.430 |
| meta-rune-spell | 0.430 |
| meta-haven-kukishiro | 0.418 |
| meta-haven-amulet | 0.397 |
| meta-rune-crystal | 0.352 |
