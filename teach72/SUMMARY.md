# teach72: teacher data from the served search with the reply keys (runbook 72)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json,mull=leaf,mulls=256`
- engine: `c30242d841dcfd7e00925d0751bf39dfef253599`, seed 7201
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.74 h in 5 segment(s), 1216.2 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 572902 of 572902 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3120588 of 3120588 (100.00 %)
- decisions: 752210; with two or more candidates: 583097
- paths: {'search': 579903, 'single_legal': 116190, 'take_kill': 32628, 'mulligan': 16384, 'take_kill_roll': 3762, 'holdback_trade': 2146, 'holdback_resample': 1048, 'consensus_lethal': 149}
- candidates per decision (2+): {'n': 583097, 'mean': 5.352, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 180502 of 2965829 (6.09 %)
- gap between the best and second-best aggregate: {'n': 583097, 'mean': 4.765, 'median': 1.65, 'p90': 12.239, 'p99': 44.399, 'max': 160.0}
- hbk ran 5700 times, flipped 1048; hbcheck ran 4652 times; lostrank ran 7001 times (played another move than the first best-scored one: 5383; left out of the search check)
- world line ends over all candidates (24964704; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8497981, 'O': 7553210, 'C': 3880939, 'L': 2119463, 't': 1426421, 'D': 1406597, 'T': 47257, 'c': 32464, 'd': 372}
- seconds per decision, one core: {'n': 752210, 'mean': 0.893, 'median': 0.217, 'p90': 2.696, 'p99': 6.507, 'max': 26.129}
- seconds per search decision, one core: {'n': 579903, 'mean': 1.145, 'median': 0.507, 'p90': 3.116, 'p99': 6.968, 'max': 26.129}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142826, 'mean': 4.678, 'median': 1.601, 'p90': 13.575, 'p99': 34.214, 'max': 114.807}
- decisions per game: {'n': 8192, 'mean': 91.823, 'median': 89, 'p90': 128, 'p99': 170, 'max': 225}; turns: {'n': 8192, 'mean': 9.004, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5583 over 8192 decisive games
- data: 179.2 MB in 8192 files; archive teach72-games.jsonl.gz 174.6 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.651 |
| meta-rune-test-subject | 0.618 |
| meta-portal-af | 0.576 |
| meta-portal-evo | 0.573 |
| meta-abyss-midrange | 0.546 |
| meta-sword-rally | 0.546 |
| meta-portal-cutthroat | 0.525 |
| meta-sword-loot | 0.494 |
| meta-dragon-ramp | 0.485 |
| meta-haven-kukishiro | 0.483 |
| meta-forest-combo | 0.454 |
| meta-abyss-aggro | 0.442 |
| meta-dragon-aggro | 0.435 |
| meta-rune-spell | 0.427 |
| meta-haven-amulet | 0.412 |
| meta-rune-crystal | 0.331 |
