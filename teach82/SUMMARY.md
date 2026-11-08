# teach82: teacher data from the served search with the reply keys (runbook 82)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8201
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.21 h in 5 segment(s), 1319.5 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 580486 of 580486 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3170504 of 3170504 (100.00 %)
- decisions: 760761; with two or more candidates: 590610
- paths: {'search': 587084, 'single_legal': 116790, 'take_kill': 32698, 'mulligan': 16384, 'take_kill_roll': 4130, 'holdback_trade': 2361, 'holdback_resample': 1165, 'consensus_lethal': 149}
- candidates per decision (2+): {'n': 590610, 'mean': 5.368, 'median': 5, 'p90': 10, 'p99': 16, 'max': 34}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 184361 of 3015631 (6.11 %)
- gap between the best and second-best aggregate: {'n': 590610, 'mean': 4.899, 'median': 1.688, 'p90': 12.6, 'p99': 46.006, 'max': 160.0}
- hbk ran 6118 times, flipped 1165; hbcheck ran 4953 times; lostrank ran 6598 times (played another move than the first best-scored one: 5026; left out of the search check)
- world line ends over all candidates (25364032; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8569170, 'O': 7783934, 'C': 3900378, 'L': 2139393, 'D': 1455279, 't': 1434293, 'T': 50271, 'c': 30973, 'd': 341}
- seconds per decision, one core: {'n': 760761, 'mean': 0.814, 'median': 0.2, 'p90': 2.474, 'p99': 5.766, 'max': 34.833}
- seconds per search decision, one core: {'n': 587084, 'mean': 1.043, 'median': 0.475, 'p90': 2.839, 'p99': 6.171, 'max': 34.833}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 143469, 'mean': 4.296, 'median': 1.517, 'p90': 12.459, 'p99': 30.567, 'max': 98.811}
- decisions per game: {'n': 8192, 'mean': 92.866, 'median': 90, 'p90': 130, 'p99': 172, 'max': 230}; turns: {'n': 8192, 'mean': 9.042, 'median': 9, 'p90': 12, 'p99': 15, 'max': 20}
- first-seat win rate: 0.5552 over 8192 decisive games
- data: 182.0 MB in 8192 files; archive teach82-games.jsonl.gz 177.4 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.667 |
| meta-rune-test-subject | 0.657 |
| meta-portal-evo | 0.571 |
| meta-abyss-midrange | 0.553 |
| meta-sword-rally | 0.543 |
| meta-portal-af | 0.527 |
| meta-portal-cutthroat | 0.510 |
| meta-dragon-ramp | 0.501 |
| meta-haven-kukishiro | 0.488 |
| meta-sword-loot | 0.482 |
| meta-forest-combo | 0.443 |
| meta-abyss-aggro | 0.440 |
| meta-rune-spell | 0.430 |
| meta-haven-amulet | 0.426 |
| meta-dragon-aggro | 0.406 |
| meta-rune-crystal | 0.354 |
