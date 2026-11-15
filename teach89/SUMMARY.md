# teach89: teacher data from the served search with the reply keys (runbook 89)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run89/leaf89q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8901
- games: 7168 of 7168 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 5.58 h in 4 segment(s), 1284.7 games/h
- replay check (every game replayed from its seed and actions): 7168 of 7168 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 503575 of 503575 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2763075 of 2763075 (100.00 %)
- decisions: 659815; with two or more candidates: 512022
- paths: {'search': 508866, 'single_legal': 100663, 'take_kill': 29223, 'mulligan': 14336, 'take_kill_roll': 3429, 'holdback_trade': 2122, 'holdback_resample': 1034, 'consensus_lethal': 142}
- candidates per decision (2+): {'n': 512022, 'mean': 5.396, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 158799 of 2630567 (6.04 %)
- gap between the best and second-best aggregate: {'n': 512022, 'mean': 4.867, 'median': 1.689, 'p90': 12.505, 'p99': 45.812, 'max': 160.0}
- hbk ran 5310 times, flipped 1034; hbcheck ran 4276 times; lostrank ran 5291 times (played another move than the first best-scored one: 4091; left out of the search check)
- world line ends over all candidates (22104600; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 7451215, 'O': 6757363, 'C': 3463400, 'L': 1855994, 't': 1254600, 'D': 1252199, 'T': 40858, 'c': 28597, 'd': 374}
- seconds per decision, one core: {'n': 659815, 'mean': 0.845, 'median': 0.207, 'p90': 2.562, 'p99': 6.049, 'max': 49.253}
- seconds per search decision, one core: {'n': 508866, 'mean': 1.083, 'median': 0.491, 'p90': 2.946, 'p99': 6.464, 'max': 49.253}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 124333, 'mean': 4.463, 'median': 1.549, 'p90': 12.952, 'p99': 31.975, 'max': 125.952}
- decisions per game: {'n': 7168, 'mean': 92.05, 'median': 89, 'p90': 129, 'p99': 170, 'max': 242}; turns: {'n': 7168, 'mean': 8.96, 'median': 9, 'p90': 12, 'p99': 14, 'max': 20}
- first-seat win rate: 0.5587 over 7168 decisive games
- data: 158.7 MB in 7168 files; archive teach89-games.jsonl.gz 154.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.666 |
| meta-haven-evo | 0.614 |
| meta-portal-af | 0.568 |
| meta-sword-rally | 0.564 |
| meta-portal-evo | 0.561 |
| meta-abyss-midrange | 0.521 |
| meta-portal-cutthroat | 0.520 |
| meta-sword-loot | 0.509 |
| meta-dragon-ramp | 0.499 |
| meta-forest-combo | 0.473 |
| meta-haven-kukishiro | 0.465 |
| meta-dragon-aggro | 0.442 |
| meta-abyss-aggro | 0.435 |
| meta-rune-spell | 0.424 |
| meta-haven-amulet | 0.416 |
| meta-rune-crystal | 0.321 |
