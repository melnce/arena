# teach77: teacher data from the served search with the reply keys (runbook 77)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 7701
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.43 h in 5 segment(s), 1273.5 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 583407 of 583407 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3186093 of 3186093 (100.00 %)
- decisions: 764340; with two or more candidates: 593286
- paths: {'search': 589648, 'single_legal': 117215, 'take_kill': 33070, 'mulligan': 16384, 'take_kill_roll': 4254, 'holdback_trade': 2477, 'holdback_resample': 1161, 'consensus_lethal': 131}
- candidates per decision (2+): {'n': 593286, 'mean': 5.37, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 184004 of 3033572 (6.07 %)
- gap between the best and second-best aggregate: {'n': 593286, 'mean': 4.901, 'median': 1.696, 'p90': 12.606, 'p99': 45.588, 'max': 160.0}
- hbk ran 6271 times, flipped 1161; hbcheck ran 5110 times; lostrank ran 6241 times (played another move than the first best-scored one: 4776; left out of the search check)
- world line ends over all candidates (25488744; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8602781, 'O': 7841719, 'C': 3944704, 'L': 2129040, 'D': 1450552, 't': 1439322, 'T': 48130, 'c': 32173, 'd': 323}
- seconds per decision, one core: {'n': 764340, 'mean': 0.84, 'median': 0.207, 'p90': 2.548, 'p99': 5.943, 'max': 26.727}
- seconds per search decision, one core: {'n': 589648, 'mean': 1.076, 'median': 0.491, 'p90': 2.931, 'p99': 6.351, 'max': 26.727}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 143816, 'mean': 4.443, 'median': 1.58, 'p90': 12.836, 'p99': 31.641, 'max': 114.203}
- decisions per game: {'n': 8192, 'mean': 93.303, 'median': 90, 'p90': 130, 'p99': 175, 'max': 223}; turns: {'n': 8192, 'mean': 9.061, 'median': 9, 'p90': 12, 'p99': 15, 'max': 19}
- first-seat win rate: 0.5515 over 8192 decisive games
- data: 183.2 MB in 8192 files; archive teach77-games.jsonl.gz 178.5 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.656 |
| meta-haven-evo | 0.649 |
| meta-sword-rally | 0.562 |
| meta-portal-evo | 0.554 |
| meta-portal-cutthroat | 0.536 |
| meta-portal-af | 0.534 |
| meta-haven-kukishiro | 0.506 |
| meta-abyss-midrange | 0.503 |
| meta-dragon-ramp | 0.501 |
| meta-sword-loot | 0.492 |
| meta-forest-combo | 0.463 |
| meta-haven-amulet | 0.443 |
| meta-abyss-aggro | 0.421 |
| meta-rune-spell | 0.421 |
| meta-dragon-aggro | 0.411 |
| meta-rune-crystal | 0.347 |
