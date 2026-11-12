# teach86: teacher data from the served search with the reply keys (runbook 86)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8601
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.11 h in 1 segment(s), 1316.5 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290639 of 290639 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1591350 of 1591350 (100.00 %)
- decisions: 380022; with two or more candidates: 295289
- paths: {'search': 293516, 'single_legal': 57809, 'take_kill': 16596, 'mulligan': 8192, 'take_kill_roll': 2050, 'holdback_trade': 1202, 'holdback_resample': 571, 'consensus_lethal': 86}
- candidates per decision (2+): {'n': 295289, 'mean': 5.389, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91463 of 1515374 (6.04 %)
- gap between the best and second-best aggregate: {'n': 295289, 'mean': 4.948, 'median': 1.71, 'p90': 12.652, 'p99': 46.29, 'max': 160.0}
- hbk ran 3088 times, flipped 571; hbcheck ran 2517 times; lostrank ran 2877 times (played another move than the first best-scored one: 2181; left out of the search check)
- world line ends over all candidates (12730800; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4291075, 'O': 3887390, 'C': 1993566, 'L': 1059506, 'D': 736014, 't': 719961, 'T': 24967, 'c': 18163, 'd': 158}
- seconds per decision, one core: {'n': 380022, 'mean': 0.821, 'median': 0.201, 'p90': 2.489, 'p99': 5.808, 'max': 21.458}
- seconds per search decision, one core: {'n': 293516, 'mean': 1.051, 'median': 0.481, 'p90': 2.861, 'p99': 6.21, 'max': 17.399}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71360, 'mean': 4.352, 'median': 1.534, 'p90': 12.663, 'p99': 31.208, 'max': 77.348}
- decisions per game: {'n': 4096, 'mean': 92.779, 'median': 90, 'p90': 129, 'p99': 173, 'max': 209}; turns: {'n': 4096, 'mean': 8.99, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5403 over 4096 decisive games
- data: 91.4 MB in 4096 files; archive teach86-games.jsonl.gz 89.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.643 |
| meta-rune-test-subject | 0.639 |
| meta-sword-rally | 0.572 |
| meta-abyss-midrange | 0.557 |
| meta-portal-evo | 0.557 |
| meta-portal-cutthroat | 0.547 |
| meta-portal-af | 0.512 |
| meta-forest-combo | 0.496 |
| meta-haven-kukishiro | 0.478 |
| meta-dragon-ramp | 0.477 |
| meta-sword-loot | 0.467 |
| meta-abyss-aggro | 0.461 |
| meta-dragon-aggro | 0.430 |
| meta-haven-amulet | 0.412 |
| meta-rune-spell | 0.402 |
| meta-rune-crystal | 0.352 |
