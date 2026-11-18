# teach92: teacher data from the served search with the reply keys (runbook 92)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9201
- games: 6144 of 6144 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 4.73 h in 3 segment(s), 1297.8 games/h
- replay check (every game replayed from its seed and actions): 6144 of 6144 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 436220 of 436220 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2393530 of 2393530 (100.00 %)
- decisions: 570512; with two or more candidates: 443466
- paths: {'search': 440826, 'single_legal': 86638, 'take_kill': 24813, 'mulligan': 12288, 'take_kill_roll': 3177, 'holdback_trade': 1798, 'holdback_resample': 842, 'consensus_lethal': 130}
- candidates per decision (2+): {'n': 443466, 'mean': 5.397, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 137765 of 2278969 (6.05 %)
- gap between the best and second-best aggregate: {'n': 443466, 'mean': 4.906, 'median': 1.691, 'p90': 12.605, 'p99': 46.217, 'max': 160.0}
- hbk ran 4493 times, flipped 842; hbcheck ran 3651 times; lostrank ran 4606 times (played another move than the first best-scored one: 3489; left out of the search check)
- world line ends over all candidates (19148240; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 6450375, 'O': 5858954, 'C': 3023839, 'L': 1580422, 'D': 1096533, 't': 1071296, 'T': 36864, 'c': 29672, 'd': 285}
- seconds per decision, one core: {'n': 570512, 'mean': 0.83, 'median': 0.204, 'p90': 2.513, 'p99': 5.893, 'max': 27.06}
- seconds per search decision, one core: {'n': 440826, 'mean': 1.062, 'median': 0.483, 'p90': 2.891, 'p99': 6.269, 'max': 27.06}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 107422, 'mean': 4.385, 'median': 1.526, 'p90': 12.788, 'p99': 31.282, 'max': 93.399}
- decisions per game: {'n': 6144, 'mean': 92.857, 'median': 90, 'p90': 129, 'p99': 170, 'max': 235}; turns: {'n': 6144, 'mean': 9.027, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.556 over 6144 decisive games
- data: 137.4 MB in 6144 files; archive teach92-games.jsonl.gz 134.0 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.668 |
| meta-rune-test-subject | 0.647 |
| meta-portal-evo | 0.551 |
| meta-sword-rally | 0.549 |
| meta-portal-af | 0.529 |
| meta-portal-cutthroat | 0.526 |
| meta-abyss-midrange | 0.525 |
| meta-sword-loot | 0.519 |
| meta-dragon-ramp | 0.501 |
| meta-forest-combo | 0.458 |
| meta-haven-kukishiro | 0.458 |
| meta-haven-amulet | 0.452 |
| meta-rune-spell | 0.423 |
| meta-abyss-aggro | 0.419 |
| meta-dragon-aggro | 0.408 |
| meta-rune-crystal | 0.366 |
