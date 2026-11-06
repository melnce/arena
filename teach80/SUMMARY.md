# teach80: teacher data from the served search with the reply keys (runbook 80)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8001
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.2 h in 5 segment(s), 1321.7 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 576351 of 576351 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3154638 of 3154638 (100.00 %)
- decisions: 755465; with two or more candidates: 586387
- paths: {'search': 582753, 'single_legal': 115907, 'take_kill': 32846, 'mulligan': 16384, 'take_kill_roll': 3795, 'holdback_trade': 2444, 'holdback_resample': 1190, 'consensus_lethal': 146}
- candidates per decision (2+): {'n': 586387, 'mean': 5.38, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 183121 of 3002018 (6.10 %)
- gap between the best and second-best aggregate: {'n': 586387, 'mean': 4.844, 'median': 1.666, 'p90': 12.493, 'p99': 45.135, 'max': 160.0}
- hbk ran 6249 times, flipped 1190; hbcheck ran 5059 times; lostrank ran 6402 times (played another move than the first best-scored one: 4818; left out of the search check)
- world line ends over all candidates (25237104; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8505215, 'O': 7723831, 'C': 3929978, 'L': 2126102, 'D': 1441906, 't': 1431030, 'T': 46447, 'c': 32182, 'd': 413}
- seconds per decision, one core: {'n': 755465, 'mean': 0.817, 'median': 0.201, 'p90': 2.474, 'p99': 5.842, 'max': 32.406}
- seconds per search decision, one core: {'n': 582753, 'mean': 1.047, 'median': 0.474, 'p90': 2.845, 'p99': 6.233, 'max': 32.406}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142608, 'mean': 4.308, 'median': 1.51, 'p90': 12.495, 'p99': 30.789, 'max': 87.434}
- decisions per game: {'n': 8192, 'mean': 92.22, 'median': 89, 'p90': 129, 'p99': 168, 'max': 225}; turns: {'n': 8192, 'mean': 8.991, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5596 over 8192 decisive games
- data: 181.1 MB in 8192 files; archive teach80-games.jsonl.gz 176.5 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.652 |
| meta-haven-evo | 0.642 |
| meta-portal-evo | 0.565 |
| meta-sword-rally | 0.559 |
| meta-abyss-midrange | 0.547 |
| meta-portal-cutthroat | 0.542 |
| meta-dragon-ramp | 0.530 |
| meta-portal-af | 0.526 |
| meta-sword-loot | 0.492 |
| meta-forest-combo | 0.478 |
| meta-haven-kukishiro | 0.472 |
| meta-abyss-aggro | 0.433 |
| meta-dragon-aggro | 0.416 |
| meta-rune-spell | 0.413 |
| meta-haven-amulet | 0.396 |
| meta-rune-crystal | 0.337 |
