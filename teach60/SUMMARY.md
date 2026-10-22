# teach60: teacher data from the served search with the reply keys (runbook 60)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40`
- engine: `75f087f95e767f72ff7d8fb8ae630431be990efe`, seed 6001
- games: 12288 of 12288 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 11.7 h in 2 segment(s), 1050.0 games/h
- replay check (every game replayed from its seed and actions): 12288 of 12288 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 833781 of 833781 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 4421213 of 4421213 (100.00 %)
- decisions: 1105758; with two or more candidates: 846582
- paths: {'search': 842039, 'single_legal': 179732, 'take_kill': 50147, 'mulligan': 24576, 'take_kill_roll': 4568, 'holdback_trade': 2700, 'holdback_resample': 1843, 'consensus_lethal': 153}
- candidates per decision (2+): {'n': 846582, 'mean': 5.222, 'median': 4, 'p90': 9, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 254903 of 4229448 (6.03 %)
- gap between the best and second-best aggregate: {'n': 846582, 'mean': 4.698, 'median': 1.677, 'p90': 12.126, 'p99': 42.323, 'max': 160.0}
- hbk ran 8203 times, flipped 1843; hbcheck ran 6360 times; lostrank ran 8258 times (played another move than the first best-scored one: 6094; left out of the search check)
- world line ends over all candidates (35369704; O = opp_open, N = opp_next, ? = unknown): {'o': 10653615, 'C': 8805900, 'O': 7156139, 'N': 2868152, 'L': 2602488, 't': 1823216, 'D': 1362079, 'T': 65128, 'c': 32522, 'd': 465}
- seconds per decision, one core: {'n': 1105758, 'mean': 1.064, 'median': 0.364, 'p90': 3.078, 'p99': 7.389, 'max': 25.993}
- seconds per search decision, one core: {'n': 842039, 'mean': 1.386, 'median': 0.744, 'p90': 3.587, 'p99': 7.924, 'max': 25.066}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 210422, 'mean': 5.589, 'median': 2.035, 'p90': 16.071, 'p99': 39.621, 'max': 126.246}
- decisions per game: {'n': 12288, 'mean': 89.987, 'median': 86, 'p90': 127, 'p99': 175, 'max': 265}; turns: {'n': 12288, 'mean': 8.849, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.561 over 12288 decisive games
- data: 253.9 MB in 12288 files; archive teach60-games.jsonl.gz 247.2 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-portal-af | 0.613 |
| meta-abyss-midrange | 0.607 |
| meta-rune-test-subject | 0.606 |
| meta-portal-evo | 0.604 |
| meta-sword-rally | 0.597 |
| meta-haven-evo | 0.587 |
| meta-sword-loot | 0.530 |
| meta-portal-cutthroat | 0.507 |
| meta-abyss-aggro | 0.467 |
| meta-forest-combo | 0.459 |
| meta-dragon-ramp | 0.451 |
| meta-dragon-aggro | 0.441 |
| meta-haven-kukishiro | 0.425 |
| meta-rune-spell | 0.422 |
| meta-haven-amulet | 0.361 |
| meta-rune-crystal | 0.325 |
