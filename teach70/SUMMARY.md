# teach70: teacher data from the served search with the reply keys (runbook 70)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 7001
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.09 h in 1 segment(s), 1326.2 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 286513 of 286513 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1551467 of 1551467 (100.00 %)
- decisions: 375804; with two or more candidates: 291494
- paths: {'search': 289898, 'single_legal': 57755, 'take_kill': 16519, 'mulligan': 8192, 'take_kill_roll': 1768, 'holdback_trade': 1087, 'holdback_resample': 509, 'consensus_lethal': 76}
- candidates per decision (2+): {'n': 291494, 'mean': 5.322, 'median': 5, 'p90': 9, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91155 of 1476282 (6.17 %)
- gap between the best and second-best aggregate: {'n': 291494, 'mean': 4.732, 'median': 1.623, 'p90': 12.151, 'p99': 44.253, 'max': 160.0}
- hbk ran 2837 times, flipped 509; hbcheck ran 2328 times; lostrank ran 3385 times (played another move than the first best-scored one: 2553; left out of the search check)
- world line ends over all candidates (12411736; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4282821, 'O': 3742543, 'C': 1901820, 'L': 1029852, 't': 720164, 'D': 696968, 'T': 24039, 'c': 13284, 'd': 245}
- seconds per decision, one core: {'n': 375804, 'mean': 0.825, 'median': 0.195, 'p90': 2.522, 'p99': 5.97, 'max': 19.798}
- seconds per search decision, one core: {'n': 289898, 'mean': 1.063, 'median': 0.474, 'p90': 2.905, 'p99': 6.391, 'max': 19.798}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71030, 'mean': 4.365, 'median': 1.526, 'p90': 12.626, 'p99': 31.435, 'max': 93.894}
- decisions per game: {'n': 4096, 'mean': 91.749, 'median': 89, 'p90': 128, 'p99': 172, 'max': 219}; turns: {'n': 4096, 'mean': 8.957, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5569 over 4096 decisive games
- data: 89.1 MB in 4096 files; archive teach70-games.jsonl.gz 86.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.648 |
| meta-haven-evo | 0.635 |
| meta-portal-evo | 0.607 |
| meta-portal-af | 0.576 |
| meta-sword-rally | 0.559 |
| meta-abyss-midrange | 0.525 |
| meta-sword-loot | 0.516 |
| meta-haven-kukishiro | 0.510 |
| meta-portal-cutthroat | 0.506 |
| meta-dragon-ramp | 0.496 |
| meta-abyss-aggro | 0.426 |
| meta-rune-spell | 0.424 |
| meta-dragon-aggro | 0.422 |
| meta-forest-combo | 0.422 |
| meta-haven-amulet | 0.410 |
| meta-rune-crystal | 0.318 |
