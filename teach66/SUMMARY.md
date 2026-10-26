# teach66: teacher data from the served search with the reply keys (runbook 66)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run66/leaf66q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6601
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 4.28 h in 1 segment(s), 957.9 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 296152 of 296152 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1595113 of 1595113 (100.00 %)
- decisions: 387679; with two or more candidates: 301198
- paths: {'search': 299448, 'single_legal': 60232, 'take_kill': 16323, 'mulligan': 8192, 'take_kill_roll': 1666, 'holdback_trade': 1044, 'holdback_resample': 706, 'consensus_lethal': 68}
- candidates per decision (2+): {'n': 301198, 'mean': 5.296, 'median': 5, 'p90': 9, 'p99': 16, 'max': 34}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92123 of 1521457 (6.05 %)
- gap between the best and second-best aggregate: {'n': 301198, 'mean': 5.006, 'median': 1.8, 'p90': 12.875, 'p99': 45.552, 'max': 160.0}
- hbk ran 2982 times, flipped 706; hbcheck ran 2276 times; lostrank ran 3296 times (played another move than the first best-scored one: 2482; left out of the search check)
- world line ends over all candidates (12760904; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'C': 3860719, 'o': 3562289, 'O': 2191375, 'L': 1010046, 'N': 1001239, 't': 610614, 'D': 487271, 'T': 25429, 'c': 11799, 'd': 123}
- seconds per decision, one core: {'n': 387679, 'mean': 1.105, 'median': 0.468, 'p90': 3.16, 'p99': 6.993, 'max': 20.577}
- seconds per search decision, one core: {'n': 299448, 'mean': 1.421, 'median': 0.823, 'p90': 3.608, 'p99': 7.424, 'max': 20.577}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 72152, 'mean': 5.938, 'median': 2.614, 'p90': 16.649, 'p99': 38.125, 'max': 94.848}
- decisions per game: {'n': 4096, 'mean': 94.648, 'median': 91, 'p90': 135, 'p99': 182, 'max': 228}; turns: {'n': 4096, 'mean': 9.087, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5469 over 4096 decisive games
- data: 91.0 MB in 4096 files; archive teach66-games.jsonl.gz 88.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.617 |
| meta-portal-af | 0.607 |
| meta-rune-test-subject | 0.602 |
| meta-portal-evo | 0.580 |
| meta-abyss-midrange | 0.562 |
| meta-sword-rally | 0.553 |
| meta-sword-loot | 0.516 |
| meta-portal-cutthroat | 0.514 |
| meta-dragon-ramp | 0.486 |
| meta-haven-kukishiro | 0.486 |
| meta-dragon-aggro | 0.473 |
| meta-forest-combo | 0.467 |
| meta-abyss-aggro | 0.438 |
| meta-rune-spell | 0.418 |
| meta-haven-amulet | 0.373 |
| meta-rune-crystal | 0.309 |
