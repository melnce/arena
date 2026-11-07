# teach81: teacher data from the served search with the reply keys (runbook 81)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8101
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.15 h in 5 segment(s), 1331.8 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 580323 of 580323 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3176560 of 3176560 (100.00 %)
- decisions: 760649; with two or more candidates: 590298
- paths: {'search': 586796, 'single_legal': 116803, 'take_kill': 32884, 'mulligan': 16384, 'take_kill_roll': 4143, 'holdback_trade': 2377, 'holdback_resample': 1125, 'consensus_lethal': 137}
- candidates per decision (2+): {'n': 590298, 'mean': 5.381, 'median': 5, 'p90': 10, 'p99': 16, 'max': 43}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 184833 of 3021961 (6.12 %)
- gap between the best and second-best aggregate: {'n': 590298, 'mean': 4.882, 'median': 1.683, 'p90': 12.556, 'p99': 45.59, 'max': 160.0}
- hbk ran 6062 times, flipped 1125; hbcheck ran 4937 times; lostrank ran 6473 times (played another move than the first best-scored one: 4987; left out of the search check)
- world line ends over all candidates (25412480; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8568922, 'O': 7821723, 'C': 3923076, 'L': 2133028, 'D': 1451021, 't': 1437209, 'T': 46469, 'c': 30664, 'd': 368}
- seconds per decision, one core: {'n': 760649, 'mean': 0.807, 'median': 0.196, 'p90': 2.453, 'p99': 5.731, 'max': 23.179}
- seconds per search decision, one core: {'n': 586796, 'mean': 1.034, 'median': 0.47, 'p90': 2.818, 'p99': 6.113, 'max': 23.179}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 143409, 'mean': 4.26, 'median': 1.481, 'p90': 12.407, 'p99': 30.423, 'max': 99.156}
- decisions per game: {'n': 8192, 'mean': 92.853, 'median': 90, 'p90': 129, 'p99': 170, 'max': 228}; turns: {'n': 8192, 'mean': 9.037, 'median': 9, 'p90': 12, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5576 over 8192 decisive games
- data: 182.3 MB in 8192 files; archive teach81-games.jsonl.gz 177.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.657 |
| meta-rune-test-subject | 0.653 |
| meta-sword-rally | 0.551 |
| meta-portal-evo | 0.543 |
| meta-abyss-midrange | 0.542 |
| meta-portal-af | 0.537 |
| meta-portal-cutthroat | 0.519 |
| meta-haven-kukishiro | 0.501 |
| meta-sword-loot | 0.499 |
| meta-dragon-ramp | 0.487 |
| meta-forest-combo | 0.483 |
| meta-haven-amulet | 0.434 |
| meta-dragon-aggro | 0.415 |
| meta-rune-spell | 0.415 |
| meta-abyss-aggro | 0.410 |
| meta-rune-crystal | 0.353 |
