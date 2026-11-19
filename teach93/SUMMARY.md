# teach93: teacher data from the served search with the reply keys (runbook 93)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9301
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.11 h in 1 segment(s), 1318.1 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290857 of 290857 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1600560 of 1600560 (100.00 %)
- decisions: 380328; with two or more candidates: 295734
- paths: {'search': 293915, 'single_legal': 57820, 'take_kill': 16583, 'mulligan': 8192, 'take_kill_roll': 1929, 'holdback_trade': 1225, 'holdback_resample': 594, 'consensus_lethal': 70}
- candidates per decision (2+): {'n': 295734, 'mean': 5.412, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 93239 of 1523892 (6.12 %)
- gap between the best and second-best aggregate: {'n': 295734, 'mean': 4.894, 'median': 1.686, 'p90': 12.644, 'p99': 45.216, 'max': 160.0}
- hbk ran 3084 times, flipped 594; hbcheck ran 2490 times; lostrank ran 3058 times (played another move than the first best-scored one: 2346; left out of the search check)
- world line ends over all candidates (12804480; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4302757, 'O': 3904743, 'C': 2025469, 'L': 1069136, 'D': 732509, 't': 721468, 'T': 26651, 'c': 21542, 'd': 205}
- seconds per decision, one core: {'n': 380328, 'mean': 0.819, 'median': 0.199, 'p90': 2.482, 'p99': 5.83, 'max': 25.976}
- seconds per search decision, one core: {'n': 293915, 'mean': 1.047, 'median': 0.476, 'p90': 2.854, 'p99': 6.221, 'max': 21.494}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71427, 'mean': 4.34, 'median': 1.508, 'p90': 12.511, 'p99': 31.562, 'max': 104.435}
- decisions per game: {'n': 4096, 'mean': 92.854, 'median': 90, 'p90': 129, 'p99': 170, 'max': 223}; turns: {'n': 4096, 'mean': 9.001, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5474 over 4096 decisive games
- data: 91.7 MB in 4096 files; archive teach93-games.jsonl.gz 89.4 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.693 |
| meta-rune-test-subject | 0.648 |
| meta-portal-evo | 0.570 |
| meta-portal-af | 0.543 |
| meta-sword-loot | 0.539 |
| meta-sword-rally | 0.525 |
| meta-abyss-midrange | 0.514 |
| meta-portal-cutthroat | 0.514 |
| meta-haven-kukishiro | 0.496 |
| meta-dragon-ramp | 0.475 |
| meta-forest-combo | 0.461 |
| meta-rune-spell | 0.441 |
| meta-haven-amulet | 0.420 |
| meta-dragon-aggro | 0.410 |
| meta-abyss-aggro | 0.397 |
| meta-rune-crystal | 0.353 |
