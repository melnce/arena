# teach87: teacher data from the served search with the reply keys (runbook 87)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run87/leaf87q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8701
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.1 h in 1 segment(s), 1319.4 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 289484 of 289484 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1589382 of 1589382 (100.00 %)
- decisions: 378779; with two or more candidates: 294436
- paths: {'search': 292628, 'single_legal': 57453, 'take_kill': 16599, 'mulligan': 8192, 'take_kill_roll': 2021, 'holdback_trade': 1236, 'holdback_resample': 572, 'consensus_lethal': 78}
- candidates per decision (2+): {'n': 294436, 'mean': 5.398, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92181 of 1512423 (6.09 %)
- gap between the best and second-best aggregate: {'n': 294436, 'mean': 4.89, 'median': 1.691, 'p90': 12.55, 'p99': 45.816, 'max': 160.0}
- hbk ran 3090 times, flipped 572; hbcheck ran 2518 times; lostrank ran 3144 times (played another move than the first best-scored one: 2400; left out of the search check)
- world line ends over all candidates (12715056; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4290879, 'O': 3891136, 'C': 1993304, 'L': 1053991, 'D': 725261, 't': 718438, 'T': 23710, 'c': 18113, 'd': 224}
- seconds per decision, one core: {'n': 378779, 'mean': 0.822, 'median': 0.202, 'p90': 2.495, 'p99': 5.791, 'max': 26.308}
- seconds per search decision, one core: {'n': 292628, 'mean': 1.053, 'median': 0.478, 'p90': 2.871, 'p99': 6.17, 'max': 19.269}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71257, 'mean': 4.35, 'median': 1.534, 'p90': 12.642, 'p99': 31.049, 'max': 92.13}
- decisions per game: {'n': 4096, 'mean': 92.475, 'median': 89, 'p90': 130, 'p99': 172, 'max': 213}; turns: {'n': 4096, 'mean': 8.979, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.543 over 4096 decisive games
- data: 91.2 MB in 4096 files; archive teach87-games.jsonl.gz 88.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.644 |
| meta-rune-test-subject | 0.627 |
| meta-portal-evo | 0.570 |
| meta-sword-rally | 0.551 |
| meta-abyss-midrange | 0.543 |
| meta-portal-af | 0.533 |
| meta-sword-loot | 0.525 |
| meta-dragon-ramp | 0.494 |
| meta-haven-kukishiro | 0.494 |
| meta-portal-cutthroat | 0.490 |
| meta-forest-combo | 0.480 |
| meta-abyss-aggro | 0.451 |
| meta-haven-amulet | 0.434 |
| meta-rune-spell | 0.430 |
| meta-dragon-aggro | 0.391 |
| meta-rune-crystal | 0.342 |
