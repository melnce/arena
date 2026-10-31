# teach73: teacher data from the served search with the reply keys (runbook 73)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run73/leaf73q.json,mull=leaf,mulls=256`
- engine: `c30242d841dcfd7e00925d0751bf39dfef253599`, seed 7301
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.63 h in 5 segment(s), 1234.8 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 580494 of 580494 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3164750 of 3164750 (100.00 %)
- decisions: 760455; with two or more candidates: 590676
- paths: {'search': 587113, 'single_legal': 116497, 'take_kill': 32972, 'mulligan': 16384, 'take_kill_roll': 3778, 'holdback_trade': 2401, 'holdback_resample': 1162, 'consensus_lethal': 148}
- candidates per decision (2+): {'n': 590676, 'mean': 5.358, 'median': 5, 'p90': 10, 'p99': 16, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 184163 of 3009330 (6.12 %)
- gap between the best and second-best aggregate: {'n': 590676, 'mean': 4.815, 'median': 1.677, 'p90': 12.338, 'p99': 45.121, 'max': 160.0}
- hbk ran 6067 times, flipped 1162; hbcheck ran 4905 times; lostrank ran 6619 times (played another move than the first best-scored one: 5041; left out of the search check)
- world line ends over all candidates (25318000; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8580746, 'O': 7732095, 'C': 3921312, 'L': 2141078, 't': 1432546, 'D': 1421685, 'T': 53172, 'c': 34907, 'd': 459}
- seconds per decision, one core: {'n': 760455, 'mean': 0.869, 'median': 0.215, 'p90': 2.636, 'p99': 6.166, 'max': 21.376}
- seconds per search decision, one core: {'n': 587113, 'mean': 1.113, 'median': 0.507, 'p90': 3.03, 'p99': 6.577, 'max': 21.376}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 143774, 'mean': 4.574, 'median': 1.626, 'p90': 13.256, 'p99': 32.221, 'max': 96.963}
- decisions per game: {'n': 8192, 'mean': 92.829, 'median': 90, 'p90': 129, 'p99': 170, 'max': 224}; turns: {'n': 8192, 'mean': 9.062, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5564 over 8192 decisive games
- data: 181.7 MB in 8192 files; archive teach73-games.jsonl.gz 177.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.654 |
| meta-rune-test-subject | 0.643 |
| meta-portal-evo | 0.564 |
| meta-portal-af | 0.555 |
| meta-sword-rally | 0.545 |
| meta-abyss-midrange | 0.537 |
| meta-haven-kukishiro | 0.513 |
| meta-portal-cutthroat | 0.512 |
| meta-dragon-ramp | 0.496 |
| meta-sword-loot | 0.491 |
| meta-forest-combo | 0.476 |
| meta-dragon-aggro | 0.446 |
| meta-abyss-aggro | 0.431 |
| meta-haven-amulet | 0.398 |
| meta-rune-spell | 0.397 |
| meta-rune-crystal | 0.345 |
