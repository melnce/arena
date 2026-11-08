# teach82: teacher data from the served search with the reply keys (runbook 82)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8201
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.09 h in 1 segment(s), 1323.5 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290633 of 290633 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1585536 of 1585536 (100.00 %)
- decisions: 380608; with two or more candidates: 295573
- paths: {'search': 293815, 'single_legal': 58361, 'take_kill': 16358, 'mulligan': 8192, 'take_kill_roll': 2043, 'holdback_trade': 1169, 'holdback_resample': 589, 'consensus_lethal': 81}
- candidates per decision (2+): {'n': 295573, 'mean': 5.364, 'median': 5, 'p90': 10, 'p99': 16, 'max': 33}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92542 of 1508634 (6.13 %)
- gap between the best and second-best aggregate: {'n': 295573, 'mean': 4.937, 'median': 1.702, 'p90': 12.696, 'p99': 46.354, 'max': 160.0}
- hbk ran 3038 times, flipped 589; hbcheck ran 2449 times; lostrank ran 3182 times (played another move than the first best-scored one: 2401; left out of the search check)
- world line ends over all candidates (12684288; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4284129, 'O': 3885305, 'C': 1958526, 'L': 1073429, 'D': 727420, 't': 715681, 'T': 23643, 'c': 16003, 'd': 152}
- seconds per decision, one core: {'n': 380608, 'mean': 0.815, 'median': 0.2, 'p90': 2.476, 'p99': 5.784, 'max': 29.756}
- seconds per search decision, one core: {'n': 293815, 'mean': 1.044, 'median': 0.476, 'p90': 2.842, 'p99': 6.191, 'max': 19.672}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71755, 'mean': 4.304, 'median': 1.514, 'p90': 12.52, 'p99': 30.833, 'max': 98.811}
- decisions per game: {'n': 4096, 'mean': 92.922, 'median': 90, 'p90': 130, 'p99': 171, 'max': 216}; turns: {'n': 4096, 'mean': 9.043, 'median': 9, 'p90': 12, 'p99': 15, 'max': 20}
- first-seat win rate: 0.5525 over 4096 decisive games
- data: 91.0 MB in 4096 files; archive teach82-games.jsonl.gz 88.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.664 |
| meta-rune-test-subject | 0.648 |
| meta-portal-evo | 0.566 |
| meta-abyss-midrange | 0.551 |
| meta-sword-rally | 0.545 |
| meta-dragon-ramp | 0.519 |
| meta-portal-af | 0.508 |
| meta-portal-cutthroat | 0.502 |
| meta-haven-kukishiro | 0.492 |
| meta-sword-loot | 0.478 |
| meta-abyss-aggro | 0.463 |
| meta-rune-spell | 0.447 |
| meta-haven-amulet | 0.424 |
| meta-forest-combo | 0.420 |
| meta-dragon-aggro | 0.414 |
| meta-rune-crystal | 0.357 |
