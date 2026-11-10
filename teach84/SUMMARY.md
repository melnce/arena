# teach84: teacher data from the served search with the reply keys (runbook 84)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8401
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.1 h in 1 segment(s), 1323.1 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290980 of 290980 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1594679 of 1594679 (100.00 %)
- decisions: 380789; with two or more candidates: 295796
- paths: {'search': 293990, 'single_legal': 58278, 'take_kill': 16623, 'mulligan': 8192, 'take_kill_roll': 1826, 'holdback_trade': 1231, 'holdback_resample': 575, 'consensus_lethal': 74}
- candidates per decision (2+): {'n': 295796, 'mean': 5.391, 'median': 5, 'p90': 10, 'p99': 16, 'max': 35}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92302 of 1517808 (6.08 %)
- gap between the best and second-best aggregate: {'n': 295796, 'mean': 4.936, 'median': 1.702, 'p90': 12.672, 'p99': 46.315, 'max': 160.0}
- hbk ran 3036 times, flipped 575; hbcheck ran 2461 times; lostrank ran 3010 times (played another move than the first best-scored one: 2272; left out of the search check)
- world line ends over all candidates (12757432; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4297373, 'O': 3891871, 'C': 1988665, 'L': 1082391, 'D': 730929, 't': 721860, 'T': 25212, 'c': 18953, 'd': 178}
- seconds per decision, one core: {'n': 380789, 'mean': 0.816, 'median': 0.2, 'p90': 2.471, 'p99': 5.786, 'max': 22.394}
- seconds per search decision, one core: {'n': 293990, 'mean': 1.045, 'median': 0.479, 'p90': 2.84, 'p99': 6.181, 'max': 22.394}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71520, 'mean': 4.324, 'median': 1.517, 'p90': 12.595, 'p99': 30.774, 'max': 116.33}
- decisions per game: {'n': 4096, 'mean': 92.966, 'median': 90, 'p90': 129, 'p99': 168, 'max': 215}; turns: {'n': 4096, 'mean': 9.011, 'median': 9, 'p90': 12, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5442 over 4096 decisive games
- data: 91.4 MB in 4096 files; archive teach84-games.jsonl.gz 89.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.637 |
| meta-haven-evo | 0.633 |
| meta-sword-rally | 0.551 |
| meta-portal-evo | 0.547 |
| meta-abyss-midrange | 0.539 |
| meta-portal-af | 0.533 |
| meta-portal-cutthroat | 0.529 |
| meta-sword-loot | 0.504 |
| meta-dragon-ramp | 0.486 |
| meta-forest-combo | 0.482 |
| meta-haven-kukishiro | 0.475 |
| meta-abyss-aggro | 0.455 |
| meta-rune-spell | 0.422 |
| meta-dragon-aggro | 0.418 |
| meta-haven-amulet | 0.414 |
| meta-rune-crystal | 0.375 |
