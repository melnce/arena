# teach79: teacher data from the served search with the reply keys (runbook 79)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7901
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.15 h in 5 segment(s), 1332.2 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 576408 of 576408 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3150910 of 3150910 (100.00 %)
- decisions: 755811; with two or more candidates: 586122
- paths: {'search': 582554, 'single_legal': 116251, 'take_kill': 33081, 'mulligan': 16384, 'take_kill_roll': 3831, 'holdback_trade': 2443, 'holdback_resample': 1125, 'consensus_lethal': 142}
- candidates per decision (2+): {'n': 586122, 'mean': 5.376, 'median': 5, 'p90': 10, 'p99': 16, 'max': 41}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 182836 of 3000985 (6.09 %)
- gap between the best and second-best aggregate: {'n': 586122, 'mean': 4.897, 'median': 1.69, 'p90': 12.6, 'p99': 45.782, 'max': 160.0}
- hbk ran 6121 times, flipped 1125; hbcheck ran 4996 times; lostrank ran 6146 times (played another move than the first best-scored one: 4729; left out of the search check)
- world line ends over all candidates (25207280; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8520023, 'O': 7750056, 'C': 3893538, 'L': 2095738, 'D': 1436535, 't': 1433654, 'T': 47385, 'c': 29971, 'd': 380}
- seconds per decision, one core: {'n': 755811, 'mean': 0.812, 'median': 0.197, 'p90': 2.468, 'p99': 5.78, 'max': 22.821}
- seconds per search decision, one core: {'n': 582554, 'mean': 1.042, 'median': 0.471, 'p90': 2.839, 'p99': 6.169, 'max': 19.534}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142778, 'mean': 4.279, 'median': 1.494, 'p90': 12.405, 'p99': 30.836, 'max': 94.645}
- decisions per game: {'n': 8192, 'mean': 92.262, 'median': 90, 'p90': 129, 'p99': 168, 'max': 243}; turns: {'n': 8192, 'mean': 8.999, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.5566 over 8192 decisive games
- data: 181.1 MB in 8192 files; archive teach79-games.jsonl.gz 176.5 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.646 |
| meta-rune-test-subject | 0.641 |
| meta-sword-rally | 0.567 |
| meta-portal-evo | 0.551 |
| meta-portal-af | 0.533 |
| meta-portal-cutthroat | 0.532 |
| meta-abyss-midrange | 0.525 |
| meta-dragon-ramp | 0.504 |
| meta-sword-loot | 0.499 |
| meta-forest-combo | 0.487 |
| meta-haven-kukishiro | 0.459 |
| meta-abyss-aggro | 0.432 |
| meta-rune-spell | 0.429 |
| meta-haven-amulet | 0.423 |
| meta-dragon-aggro | 0.417 |
| meta-rune-crystal | 0.354 |
