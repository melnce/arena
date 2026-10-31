# teach74: teacher data from the served search with the reply keys (runbook 74)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run73/leaf73q.json,mull=leaf,mulls=256`
- engine: `a3daa7bef9fd5e9d11b31e1898cdb18d64ffaedd`, seed 7401
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 7.36 h in 5 segment(s), 1112.8 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 577207 of 577207 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3152037 of 3152037 (100.00 %)
- decisions: 756230; with two or more candidates: 587188
- paths: {'search': 583798, 'single_legal': 115754, 'take_kill': 32872, 'mulligan': 16384, 'take_kill_roll': 3873, 'holdback_trade': 2288, 'holdback_resample': 1102, 'consensus_lethal': 159}
- candidates per decision (2+): {'n': 587188, 'mean': 5.368, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 183106 of 2999281 (6.10 %)
- gap between the best and second-best aggregate: {'n': 587188, 'mean': 4.829, 'median': 1.665, 'p90': 12.35, 'p99': 45.149, 'max': 160.0}
- hbk ran 6061 times, flipped 1102; hbcheck ran 4959 times; lostrank ran 6591 times (played another move than the first best-scored one: 5048; left out of the search check)
- world line ends over all candidates (25216296; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8533142, 'O': 7693891, 'C': 3943365, 'L': 2108218, 't': 1432989, 'D': 1421831, 'T': 48633, 'c': 33845, 'd': 382}
- seconds per decision, one core: {'n': 756230, 'mean': 0.969, 'median': 0.243, 'p90': 2.93, 'p99': 6.957, 'max': 30.532}
- seconds per search decision, one core: {'n': 583798, 'mean': 1.241, 'median': 0.561, 'p90': 3.375, 'p99': 7.441, 'max': 25.548}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 143308, 'mean': 5.089, 'median': 1.773, 'p90': 14.79, 'p99': 36.473, 'max': 146.271}
- decisions per game: {'n': 8192, 'mean': 92.313, 'median': 89, 'p90': 129, 'p99': 171, 'max': 229}; turns: {'n': 8192, 'mean': 9.032, 'median': 9, 'p90': 12, 'p99': 14, 'max': 32}
- first-seat win rate: 0.5536 over 8192 decisive games
- data: 181.0 MB in 8192 files; archive teach74-games.jsonl.gz 176.4 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.675 |
| meta-haven-evo | 0.644 |
| meta-portal-evo | 0.574 |
| meta-sword-rally | 0.560 |
| meta-portal-af | 0.550 |
| meta-abyss-midrange | 0.525 |
| meta-portal-cutthroat | 0.524 |
| meta-dragon-ramp | 0.506 |
| meta-haven-kukishiro | 0.499 |
| meta-sword-loot | 0.488 |
| meta-forest-combo | 0.461 |
| meta-abyss-aggro | 0.434 |
| meta-dragon-aggro | 0.423 |
| meta-rune-spell | 0.405 |
| meta-haven-amulet | 0.399 |
| meta-rune-crystal | 0.332 |
