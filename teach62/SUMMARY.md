# teach62: teacher data from the served search with the reply keys (runbook 62)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40`
- engine: `75f087f95e767f72ff7d8fb8ae630431be990efe`, seed 6201
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.74 h in 1 segment(s), 1095.5 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 281651 of 281651 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1479439 of 1479439 (100.00 %)
- decisions: 373465; with two or more candidates: 286564
- paths: {'search': 285088, 'single_legal': 60856, 'take_kill': 16158, 'mulligan': 8192, 'take_kill_roll': 1644, 'holdback_trade': 845, 'holdback_resample': 631, 'consensus_lethal': 51}
- candidates per decision (2+): {'n': 286564, 'mean': 5.163, 'median': 4, 'p90': 9, 'p99': 15, 'max': 33}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 85324 of 1408642 (6.06 %)
- gap between the best and second-best aggregate: {'n': 286564, 'mean': 4.78, 'median': 1.699, 'p90': 12.202, 'p99': 44.008, 'max': 160.0}
- hbk ran 2624 times, flipped 631; hbcheck ran 1993 times; lostrank ran 3437 times (played another move than the first best-scored one: 2453; left out of the search check)
- world line ends over all candidates (11835512; O = opp_open, N = opp_next, ? = unknown): {'o': 3502744, 'C': 3316002, 'O': 2074134, 'L': 952992, 'N': 926808, 't': 592468, 'D': 439859, 'T': 20978, 'c': 9409, 'd': 118}
- seconds per decision, one core: {'n': 373465, 'mean': 1.005, 'median': 0.385, 'p90': 2.906, 'p99': 6.804, 'max': 22.083}
- seconds per search decision, one core: {'n': 285088, 'mean': 1.31, 'median': 0.726, 'p90': 3.376, 'p99': 7.222, 'max': 20.171}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71046, 'mean': 5.283, 'median': 2.068, 'p90': 15.16, 'p99': 36.014, 'max': 92.774}
- decisions per game: {'n': 4096, 'mean': 91.178, 'median': 88, 'p90': 128, 'p99': 179, 'max': 224}; turns: {'n': 4096, 'mean': 8.958, 'median': 9, 'p90': 12, 'p99': 15, 'max': 22}
- first-seat win rate: 0.5562 over 4096 decisive games
- data: 84.4 MB in 4096 files; archive teach62-games.jsonl.gz 82.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-abyss-midrange | 0.625 |
| meta-portal-af | 0.602 |
| meta-haven-evo | 0.594 |
| meta-portal-evo | 0.584 |
| meta-rune-test-subject | 0.582 |
| meta-sword-rally | 0.557 |
| meta-portal-cutthroat | 0.518 |
| meta-sword-loot | 0.502 |
| meta-abyss-aggro | 0.496 |
| meta-forest-combo | 0.469 |
| meta-dragon-ramp | 0.455 |
| meta-haven-kukishiro | 0.455 |
| meta-dragon-aggro | 0.445 |
| meta-haven-amulet | 0.400 |
| meta-rune-spell | 0.391 |
| meta-rune-crystal | 0.326 |
