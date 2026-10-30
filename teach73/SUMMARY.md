# teach73: teacher data from the served search with the reply keys (runbook 73)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run73/leaf73q.json,mull=leaf,mulls=256`
- engine: `c30242d841dcfd7e00925d0751bf39dfef253599`, seed 7301
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.42 h in 1 segment(s), 1198.6 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290997 of 290997 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1586990 of 1586990 (100.00 %)
- decisions: 380932; with two or more candidates: 296129
- paths: {'search': 294370, 'single_legal': 58409, 'take_kill': 16384, 'mulligan': 8192, 'take_kill_roll': 1741, 'holdback_trade': 1192, 'holdback_resample': 567, 'consensus_lethal': 77}
- candidates per decision (2+): {'n': 296129, 'mean': 5.359, 'median': 5, 'p90': 10, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91620 of 1509852 (6.07 %)
- gap between the best and second-best aggregate: {'n': 296129, 'mean': 4.809, 'median': 1.679, 'p90': 12.34, 'p99': 44.412, 'max': 160.0}
- hbk ran 2993 times, flipped 567; hbcheck ran 2426 times; lostrank ran 3373 times (played another move than the first best-scored one: 2592; left out of the search check)
- world line ends over all candidates (12695920; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4296864, 'O': 3882585, 'C': 1973653, 'L': 1069377, 'D': 716329, 't': 711841, 'T': 26992, 'c': 18024, 'd': 255}
- seconds per decision, one core: {'n': 380932, 'mean': 0.9, 'median': 0.226, 'p90': 2.727, 'p99': 6.343, 'max': 21.376}
- seconds per search decision, one core: {'n': 294370, 'mean': 1.151, 'median': 0.532, 'p90': 3.127, 'p99': 6.751, 'max': 21.376}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71997, 'mean': 4.739, 'median': 1.699, 'p90': 13.711, 'p99': 33.13, 'max': 96.963}
- decisions per game: {'n': 4096, 'mean': 93.001, 'median': 90, 'p90': 130, 'p99': 170, 'max': 224}; turns: {'n': 4096, 'mean': 9.077, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5588 over 4096 decisive games
- data: 91.2 MB in 4096 files; archive teach73-games.jsonl.gz 88.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.658 |
| meta-rune-test-subject | 0.648 |
| meta-portal-evo | 0.562 |
| meta-abyss-midrange | 0.547 |
| meta-portal-af | 0.547 |
| meta-sword-rally | 0.535 |
| meta-haven-kukishiro | 0.514 |
| meta-sword-loot | 0.502 |
| meta-portal-cutthroat | 0.500 |
| meta-forest-combo | 0.486 |
| meta-dragon-ramp | 0.478 |
| meta-dragon-aggro | 0.465 |
| meta-abyss-aggro | 0.408 |
| meta-haven-amulet | 0.408 |
| meta-rune-spell | 0.397 |
| meta-rune-crystal | 0.344 |
