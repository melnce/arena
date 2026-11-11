# teach85: teacher data from the served search with the reply keys (runbook 85)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8501
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.38 h in 5 segment(s), 1283.2 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 577036 of 577036 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3166092 of 3166092 (100.00 %)
- decisions: 755446; with two or more candidates: 586843
- paths: {'search': 583201, 'single_legal': 114953, 'take_kill': 33328, 'mulligan': 16384, 'take_kill_roll': 3792, 'holdback_trade': 2491, 'holdback_resample': 1151, 'consensus_lethal': 146}
- candidates per decision (2+): {'n': 586843, 'mean': 5.395, 'median': 5, 'p90': 10, 'p99': 16, 'max': 40}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 183003 of 3013148 (6.07 %)
- gap between the best and second-best aggregate: {'n': 586843, 'mean': 4.882, 'median': 1.679, 'p90': 12.526, 'p99': 45.622, 'max': 160.0}
- hbk ran 6170 times, flipped 1151; hbcheck ran 5019 times; lostrank ran 6165 times (played another move than the first best-scored one: 4785; left out of the search check)
- world line ends over all candidates (25328736; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8565277, 'O': 7711061, 'C': 3963945, 'L': 2106560, 'D': 1450485, 't': 1441089, 'T': 53512, 'c': 36492, 'd': 315}
- seconds per decision, one core: {'n': 755446, 'mean': 0.843, 'median': 0.209, 'p90': 2.554, 'p99': 5.975, 'max': 38.886}
- seconds per search decision, one core: {'n': 583201, 'mean': 1.079, 'median': 0.492, 'p90': 2.939, 'p99': 6.371, 'max': 38.886}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142232, 'mean': 4.455, 'median': 1.581, 'p90': 12.868, 'p99': 32.118, 'max': 120.644}
- decisions per game: {'n': 8192, 'mean': 92.218, 'median': 89, 'p90': 128, 'p99': 172, 'max': 227}; turns: {'n': 8192, 'mean': 8.966, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5499 over 8192 decisive games
- data: 181.7 MB in 8192 files; archive teach85-games.jsonl.gz 177.1 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.631 |
| meta-rune-test-subject | 0.618 |
| meta-portal-evo | 0.559 |
| meta-portal-af | 0.546 |
| meta-abyss-midrange | 0.539 |
| meta-sword-rally | 0.525 |
| meta-dragon-ramp | 0.522 |
| meta-portal-cutthroat | 0.509 |
| meta-sword-loot | 0.496 |
| meta-forest-combo | 0.494 |
| meta-haven-kukishiro | 0.477 |
| meta-haven-amulet | 0.453 |
| meta-abyss-aggro | 0.433 |
| meta-rune-spell | 0.432 |
| meta-dragon-aggro | 0.412 |
| meta-rune-crystal | 0.354 |
