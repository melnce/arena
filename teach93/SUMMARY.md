# teach93: teacher data from the served search with the reply keys (runbook 93)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9301
- games: 6144 of 6144 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 4.74 h in 3 segment(s), 1296.4 games/h
- replay check (every game replayed from its seed and actions): 6144 of 6144 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 436488 of 436488 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2401062 of 2401062 (100.00 %)
- decisions: 570603; with two or more candidates: 443973
- paths: {'search': 441245, 'single_legal': 86734, 'take_kill': 24579, 'mulligan': 12288, 'take_kill_roll': 2914, 'holdback_trade': 1827, 'holdback_resample': 901, 'consensus_lethal': 115}
- candidates per decision (2+): {'n': 443973, 'mean': 5.408, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 139488 of 2284696 (6.11 %)
- gap between the best and second-best aggregate: {'n': 443973, 'mean': 4.906, 'median': 1.674, 'p90': 12.628, 'p99': 45.945, 'max': 160.0}
- hbk ran 4632 times, flipped 901; hbcheck ran 3731 times; lostrank ran 4757 times (played another move than the first best-scored one: 3660; left out of the search check)
- world line ends over all candidates (19208496; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 6451486, 'O': 5862875, 'C': 3030092, 'L': 1612099, 'D': 1097122, 't': 1084187, 'T': 40343, 'c': 29855, 'd': 437}
- seconds per decision, one core: {'n': 570603, 'mean': 0.829, 'median': 0.205, 'p90': 2.509, 'p99': 5.899, 'max': 27.9}
- seconds per search decision, one core: {'n': 441245, 'mean': 1.06, 'median': 0.483, 'p90': 2.887, 'p99': 6.301, 'max': 27.9}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 107128, 'mean': 4.395, 'median': 1.545, 'p90': 12.704, 'p99': 31.643, 'max': 104.435}
- decisions per game: {'n': 6144, 'mean': 92.872, 'median': 90, 'p90': 130, 'p99': 169, 'max': 223}; turns: {'n': 6144, 'mean': 9.0, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.547 over 6144 decisive games
- data: 137.6 MB in 6144 files; archive teach93-games.jsonl.gz 134.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.677 |
| meta-rune-test-subject | 0.647 |
| meta-portal-evo | 0.573 |
| meta-portal-af | 0.543 |
| meta-sword-rally | 0.526 |
| meta-sword-loot | 0.525 |
| meta-portal-cutthroat | 0.521 |
| meta-abyss-midrange | 0.516 |
| meta-dragon-ramp | 0.488 |
| meta-haven-kukishiro | 0.488 |
| meta-forest-combo | 0.467 |
| meta-rune-spell | 0.445 |
| meta-haven-amulet | 0.422 |
| meta-dragon-aggro | 0.411 |
| meta-abyss-aggro | 0.395 |
| meta-rune-crystal | 0.355 |
