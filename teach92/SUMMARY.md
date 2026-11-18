# teach92: teacher data from the served search with the reply keys (runbook 92)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9201
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.1 h in 1 segment(s), 1322.8 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 290005 of 290005 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1594283 of 1594283 (100.00 %)
- decisions: 379483; with two or more candidates: 294797
- paths: {'search': 293009, 'single_legal': 57753, 'take_kill': 16589, 'mulligan': 8192, 'take_kill_roll': 2064, 'holdback_trade': 1224, 'holdback_resample': 564, 'consensus_lethal': 88}
- candidates per decision (2+): {'n': 294797, 'mean': 5.408, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 92190 of 1518394 (6.07 %)
- gap between the best and second-best aggregate: {'n': 294797, 'mean': 4.895, 'median': 1.681, 'p90': 12.574, 'p99': 46.383, 'max': 160.0}
- hbk ran 3002 times, flipped 564; hbcheck ran 2438 times; lostrank ran 3004 times (played another move than the first best-scored one: 2289; left out of the search check)
- world line ends over all candidates (12754264; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4292967, 'O': 3896299, 'C': 2025052, 'L': 1051346, 'D': 730051, 't': 713359, 'T': 25339, 'c': 19647, 'd': 204}
- seconds per decision, one core: {'n': 379483, 'mean': 0.819, 'median': 0.2, 'p90': 2.487, 'p99': 5.779, 'max': 27.06}
- seconds per search decision, one core: {'n': 293009, 'mean': 1.05, 'median': 0.48, 'p90': 2.862, 'p99': 6.146, 'max': 27.06}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71541, 'mean': 4.327, 'median': 1.511, 'p90': 12.63, 'p99': 30.628, 'max': 92.683}
- decisions per game: {'n': 4096, 'mean': 92.647, 'median': 90, 'p90': 129, 'p99': 169, 'max': 235}; turns: {'n': 4096, 'mean': 9.018, 'median': 9, 'p90': 12, 'p99': 14, 'max': 21}
- first-seat win rate: 0.5569 over 4096 decisive games
- data: 91.5 MB in 4096 files; archive teach92-games.jsonl.gz 89.2 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.676 |
| meta-rune-test-subject | 0.658 |
| meta-sword-rally | 0.586 |
| meta-portal-evo | 0.545 |
| meta-portal-cutthroat | 0.541 |
| meta-abyss-midrange | 0.529 |
| meta-portal-af | 0.523 |
| meta-sword-loot | 0.506 |
| meta-dragon-ramp | 0.492 |
| meta-forest-combo | 0.449 |
| meta-haven-amulet | 0.449 |
| meta-haven-kukishiro | 0.449 |
| meta-dragon-aggro | 0.418 |
| meta-abyss-aggro | 0.408 |
| meta-rune-spell | 0.408 |
| meta-rune-crystal | 0.361 |
