# teach81: teacher data from the served search with the reply keys (runbook 81)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8101
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.06 h in 1 segment(s), 1337.6 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 289195 of 289195 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1581012 of 1581012 (100.00 %)
- decisions: 379072; with two or more candidates: 294271
- paths: {'search': 292551, 'single_legal': 58129, 'take_kill': 16414, 'mulligan': 8192, 'take_kill_roll': 2000, 'holdback_trade': 1173, 'holdback_resample': 547, 'consensus_lethal': 66}
- candidates per decision (2+): {'n': 294271, 'mean': 5.373, 'median': 5, 'p90': 10, 'p99': 16, 'max': 43}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 90898 of 1503065 (6.05 %)
- gap between the best and second-best aggregate: {'n': 294271, 'mean': 4.898, 'median': 1.686, 'p90': 12.598, 'p99': 45.687, 'max': 160.0}
- hbk ran 3018 times, flipped 547; hbcheck ran 2471 times; lostrank ran 3356 times (played another move than the first best-scored one: 2576; left out of the search check)
- world line ends over all candidates (12648096; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4266305, 'O': 3894027, 'C': 1941633, 'L': 1065478, 'D': 722212, 't': 721284, 'T': 21999, 'c': 14962, 'd': 196}
- seconds per decision, one core: {'n': 379072, 'mean': 0.81, 'median': 0.197, 'p90': 2.462, 'p99': 5.733, 'max': 20.564}
- seconds per search decision, one core: {'n': 292551, 'mean': 1.038, 'median': 0.472, 'p90': 2.828, 'p99': 6.106, 'max': 20.564}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71507, 'mean': 4.273, 'median': 1.494, 'p90': 12.451, 'p99': 30.535, 'max': 99.156}
- decisions per game: {'n': 4096, 'mean': 92.547, 'median': 90, 'p90': 129, 'p99': 169, 'max': 228}; turns: {'n': 4096, 'mean': 9.013, 'median': 9, 'p90': 12, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5593 over 4096 decisive games
- data: 90.8 MB in 4096 files; archive teach81-games.jsonl.gz 88.5 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.662 |
| meta-haven-evo | 0.646 |
| meta-sword-rally | 0.553 |
| meta-portal-evo | 0.551 |
| meta-portal-af | 0.547 |
| meta-abyss-midrange | 0.535 |
| meta-sword-loot | 0.506 |
| meta-haven-kukishiro | 0.500 |
| meta-dragon-ramp | 0.494 |
| meta-forest-combo | 0.475 |
| meta-portal-cutthroat | 0.475 |
| meta-rune-spell | 0.441 |
| meta-dragon-aggro | 0.426 |
| meta-abyss-aggro | 0.424 |
| meta-haven-amulet | 0.414 |
| meta-rune-crystal | 0.352 |
