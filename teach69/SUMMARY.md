# teach69: teacher data from the served search with the reply keys (runbook 69)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run69/leaf69q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6901
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.09 h in 1 segment(s), 1327.6 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 285158 of 285158 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1546089 of 1546089 (100.00 %)
- decisions: 374657; with two or more candidates: 290179
- paths: {'search': 288476, 'single_legal': 58023, 'take_kill': 16439, 'mulligan': 8192, 'take_kill_roll': 1733, 'holdback_trade': 1089, 'holdback_resample': 614, 'consensus_lethal': 91}
- candidates per decision (2+): {'n': 290179, 'mean': 5.328, 'median': 5, 'p90': 9, 'p99': 16, 'max': 35}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 90548 of 1470484 (6.16 %)
- gap between the best and second-best aggregate: {'n': 290179, 'mean': 4.793, 'median': 1.647, 'p90': 12.286, 'p99': 44.566, 'max': 160.0}
- hbk ran 3014 times, flipped 614; hbcheck ran 2400 times; lostrank ran 3318 times (played another move than the first best-scored one: 2549; left out of the search check)
- world line ends over all candidates (12368712; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4251001, 'O': 3743765, 'C': 1889808, 'L': 1030049, 't': 717812, 'D': 695191, 'T': 24812, 'c': 16098, 'd': 176}
- seconds per decision, one core: {'n': 374657, 'mean': 0.825, 'median': 0.193, 'p90': 2.515, 'p99': 5.98, 'max': 17.22}
- seconds per search decision, one core: {'n': 288476, 'mean': 1.064, 'median': 0.474, 'p90': 2.9, 'p99': 6.413, 'max': 16.632}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71208, 'mean': 4.339, 'median': 1.511, 'p90': 12.603, 'p99': 31.379, 'max': 102.015}
- decisions per game: {'n': 4096, 'mean': 91.469, 'median': 88, 'p90': 128, 'p99': 171, 'max': 229}; turns: {'n': 4096, 'mean': 8.975, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5464 over 4096 decisive games
- data: 88.7 MB in 4096 files; archive teach69-games.jsonl.gz 86.3 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.619 |
| meta-haven-evo | 0.607 |
| meta-portal-evo | 0.600 |
| meta-portal-af | 0.584 |
| meta-sword-rally | 0.568 |
| meta-abyss-midrange | 0.566 |
| meta-sword-loot | 0.523 |
| meta-dragon-ramp | 0.514 |
| meta-haven-kukishiro | 0.496 |
| meta-portal-cutthroat | 0.478 |
| meta-forest-combo | 0.473 |
| meta-abyss-aggro | 0.463 |
| meta-haven-amulet | 0.404 |
| meta-rune-spell | 0.395 |
| meta-dragon-aggro | 0.393 |
| meta-rune-crystal | 0.316 |
