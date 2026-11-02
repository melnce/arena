# teach76: teacher data from the served search with the reply keys (runbook 76)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7601
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.0 h in 1 segment(s), 1363.9 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 287448 of 287448 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1575752 of 1575752 (100.00 %)
- decisions: 377344; with two or more candidates: 292494
- paths: {'search': 290762, 'single_legal': 58227, 'take_kill': 16565, 'mulligan': 8192, 'take_kill_roll': 1787, 'holdback_trade': 1156, 'holdback_resample': 576, 'consensus_lethal': 79}
- candidates per decision (2+): {'n': 292494, 'mean': 5.387, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 90859 of 1497451 (6.07 %)
- gap between the best and second-best aggregate: {'n': 292494, 'mean': 4.852, 'median': 1.682, 'p90': 12.482, 'p99': 44.688, 'max': 160.0}
- hbk ran 3016 times, flipped 576; hbcheck ran 2440 times; lostrank ran 3314 times (played another move than the first best-scored one: 2555; left out of the search check)
- world line ends over all candidates (12606016; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4259045, 'O': 3843722, 'C': 1966440, 'L': 1066159, 't': 721733, 'D': 708737, 'T': 24792, 'c': 15229, 'd': 159}
- seconds per decision, one core: {'n': 377344, 'mean': 0.799, 'median': 0.192, 'p90': 2.425, 'p99': 5.683, 'max': 22.912}
- seconds per search decision, one core: {'n': 290762, 'mean': 1.025, 'median': 0.464, 'p90': 2.789, 'p99': 6.086, 'max': 21.656}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71394, 'mean': 4.202, 'median': 1.457, 'p90': 12.206, 'p99': 30.298, 'max': 84.348}
- decisions per game: {'n': 4096, 'mean': 92.125, 'median': 90, 'p90': 128, 'p99': 172, 'max': 226}; turns: {'n': 4096, 'mean': 8.999, 'median': 9, 'p90': 12, 'p99': 15, 'max': 18}
- first-seat win rate: 0.5532 over 4096 decisive games
- data: 90.4 MB in 4096 files; archive teach76-games.jsonl.gz 88.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.643 |
| meta-rune-test-subject | 0.633 |
| meta-portal-af | 0.572 |
| meta-portal-evo | 0.559 |
| meta-abyss-midrange | 0.557 |
| meta-sword-rally | 0.539 |
| meta-portal-cutthroat | 0.537 |
| meta-dragon-ramp | 0.504 |
| meta-haven-kukishiro | 0.480 |
| meta-haven-amulet | 0.477 |
| meta-sword-loot | 0.477 |
| meta-forest-combo | 0.443 |
| meta-dragon-aggro | 0.441 |
| meta-rune-spell | 0.424 |
| meta-abyss-aggro | 0.408 |
| meta-rune-crystal | 0.307 |
