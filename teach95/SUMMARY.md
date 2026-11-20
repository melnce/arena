# teach95: teacher data from the served search with the reply keys (runbook 95)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9501
- games: 7168 of 7168 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 5.71 h in 4 segment(s), 1254.4 games/h
- replay check (every game replayed from its seed and actions): 7168 of 7168 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 505572 of 505572 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2776542 of 2776542 (100.00 %)
- decisions: 661644; with two or more candidates: 513860
- paths: {'search': 510728, 'single_legal': 100922, 'take_kill': 29012, 'mulligan': 14336, 'take_kill_roll': 3366, 'holdback_trade': 2108, 'holdback_resample': 1024, 'consensus_lethal': 148}
- candidates per decision (2+): {'n': 513860, 'mean': 5.403, 'median': 5, 'p90': 10, 'p99': 16, 'max': 44}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 159073 of 2643117 (6.02 %)
- gap between the best and second-best aggregate: {'n': 513860, 'mean': 4.897, 'median': 1.684, 'p90': 12.591, 'p99': 46.07, 'max': 160.0}
- hbk ran 5337 times, flipped 1024; hbcheck ran 4313 times; lostrank ran 5156 times (played another move than the first best-scored one: 3959; left out of the search check)
- world line ends over all candidates (22212336; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 7497494, 'O': 6777636, 'C': 3472934, 'L': 1861872, 'D': 1266242, 't': 1257997, 'T': 42572, 'c': 35254, 'd': 335}
- seconds per decision, one core: {'n': 661644, 'mean': 0.861, 'median': 0.213, 'p90': 2.615, 'p99': 6.107, 'max': 30.469}
- seconds per search decision, one core: {'n': 510728, 'mean': 1.103, 'median': 0.504, 'p90': 3.007, 'p99': 6.523, 'max': 21.94}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 124551, 'mean': 4.554, 'median': 1.606, 'p90': 13.203, 'p99': 32.394, 'max': 94.699}
- decisions per game: {'n': 7168, 'mean': 92.305, 'median': 89, 'p90': 129, 'p99': 170, 'max': 222}; turns: {'n': 7168, 'mean': 8.973, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5571 over 7168 decisive games
- data: 159.4 MB in 7168 files; archive teach95-games.jsonl.gz 155.3 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.643 |
| meta-haven-evo | 0.637 |
| meta-portal-evo | 0.569 |
| meta-portal-af | 0.556 |
| meta-abyss-midrange | 0.537 |
| meta-sword-rally | 0.536 |
| meta-portal-cutthroat | 0.519 |
| meta-sword-loot | 0.514 |
| meta-dragon-ramp | 0.507 |
| meta-forest-combo | 0.484 |
| meta-haven-kukishiro | 0.483 |
| meta-rune-spell | 0.431 |
| meta-abyss-aggro | 0.415 |
| meta-haven-amulet | 0.415 |
| meta-dragon-aggro | 0.408 |
| meta-rune-crystal | 0.345 |
