# teach76: teacher data from the served search with the reply keys (runbook 76)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7601
- games: 15360 of 15360 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 11.57 h in 12 segment(s), 1327.4 games/h
- replay check (every game replayed from its seed and actions): 15360 of 15360 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 1077639 of 1077639 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 5885063 of 5885063 (100.00 %)
- decisions: 1414007; with two or more candidates: 1096431
- paths: {'search': 1089753, 'single_legal': 218277, 'take_kill': 61450, 'mulligan': 30720, 'take_kill_roll': 6861, 'holdback_trade': 4567, 'holdback_resample': 2111, 'consensus_lethal': 268}
- candidates per decision (2+): {'n': 1096431, 'mean': 5.367, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 341178 of 5595492 (6.10 %)
- gap between the best and second-best aggregate: {'n': 1096431, 'mean': 4.865, 'median': 1.683, 'p90': 12.511, 'p99': 45.232, 'max': 160.0}
- hbk ran 11501 times, flipped 2111; hbcheck ran 9390 times; lostrank ran 12114 times (played another move than the first best-scored one: 9326; left out of the search check)
- world line ends over all candidates (47080504; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 15942841, 'O': 14368449, 'C': 7272382, 'L': 3982608, 't': 2696405, 'D': 2661292, 'T': 93164, 'c': 62611, 'd': 752}
- seconds per decision, one core: {'n': 1414007, 'mean': 0.813, 'median': 0.198, 'p90': 2.467, 'p99': 5.812, 'max': 42.001}
- seconds per search decision, one core: {'n': 1089753, 'mean': 1.043, 'median': 0.47, 'p90': 2.842, 'p99': 6.221, 'max': 24.775}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 267530, 'mean': 4.275, 'median': 1.487, 'p90': 12.422, 'p99': 30.799, 'max': 106.578}
- decisions per game: {'n': 15360, 'mean': 92.058, 'median': 89, 'p90': 128, 'p99': 171, 'max': 234}; turns: {'n': 15360, 'mean': 8.996, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5573 over 15360 decisive games
- data: 337.9 MB in 15360 files; archive teach76-games.jsonl.gz 329.2 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.647 |
| meta-rune-test-subject | 0.630 |
| meta-portal-evo | 0.576 |
| meta-portal-af | 0.559 |
| meta-sword-rally | 0.557 |
| meta-abyss-midrange | 0.549 |
| meta-portal-cutthroat | 0.536 |
| meta-haven-kukishiro | 0.498 |
| meta-dragon-ramp | 0.494 |
| meta-sword-loot | 0.485 |
| meta-forest-combo | 0.458 |
| meta-dragon-aggro | 0.421 |
| meta-rune-spell | 0.419 |
| meta-haven-amulet | 0.418 |
| meta-abyss-aggro | 0.415 |
| meta-rune-crystal | 0.337 |
