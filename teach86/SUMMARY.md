# teach86: teacher data from the served search with the reply keys (runbook 86)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8601
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.22 h in 5 segment(s), 1316.6 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 579957 of 579957 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3174240 of 3174240 (100.00 %)
- decisions: 758519; with two or more candidates: 589224
- paths: {'search': 585664, 'single_legal': 115408, 'take_kill': 33131, 'mulligan': 16384, 'take_kill_roll': 4202, 'holdback_trade': 2413, 'holdback_resample': 1147, 'consensus_lethal': 170}
- candidates per decision (2+): {'n': 589224, 'mean': 5.387, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 183357 of 3023879 (6.06 %)
- gap between the best and second-best aggregate: {'n': 589224, 'mean': 4.936, 'median': 1.7, 'p90': 12.697, 'p99': 46.087, 'max': 160.0}
- hbk ran 6131 times, flipped 1147; hbcheck ran 4984 times; lostrank ran 5707 times (played another move than the first best-scored one: 4370; left out of the search check)
- world line ends over all candidates (25393920; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8583613, 'O': 7764124, 'C': 3951360, 'L': 2113097, 'D': 1462339, 't': 1436870, 'T': 48990, 'c': 33210, 'd': 317}
- seconds per decision, one core: {'n': 758519, 'mean': 0.817, 'median': 0.199, 'p90': 2.481, 'p99': 5.795, 'max': 21.458}
- seconds per search decision, one core: {'n': 585664, 'mean': 1.047, 'median': 0.477, 'p90': 2.852, 'p99': 6.186, 'max': 18.21}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142511, 'mean': 4.331, 'median': 1.521, 'p90': 12.58, 'p99': 30.9, 'max': 89.314}
- decisions per game: {'n': 8192, 'mean': 92.593, 'median': 90, 'p90': 129, 'p99': 171, 'max': 231}; turns: {'n': 8192, 'mean': 8.978, 'median': 9, 'p90': 12, 'p99': 14, 'max': 22}
- first-seat win rate: 0.5436 over 8192 decisive games
- data: 182.4 MB in 8192 files; archive teach86-games.jsonl.gz 177.8 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.664 |
| meta-haven-evo | 0.627 |
| meta-portal-cutthroat | 0.560 |
| meta-sword-rally | 0.558 |
| meta-portal-evo | 0.550 |
| meta-abyss-midrange | 0.547 |
| meta-portal-af | 0.526 |
| meta-sword-loot | 0.482 |
| meta-dragon-ramp | 0.480 |
| meta-haven-kukishiro | 0.477 |
| meta-forest-combo | 0.475 |
| meta-abyss-aggro | 0.468 |
| meta-rune-spell | 0.425 |
| meta-haven-amulet | 0.422 |
| meta-dragon-aggro | 0.400 |
| meta-rune-crystal | 0.339 |
