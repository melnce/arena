# teach72: teacher data from the served search with the reply keys (runbook 72)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json,mull=leaf,mulls=256`
- engine: `c30242d841dcfd7e00925d0751bf39dfef253599`, seed 7201
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.11 h in 1 segment(s), 1316.4 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 285783 of 285783 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1554951 of 1554951 (100.00 %)
- decisions: 375239; with two or more candidates: 290757
- paths: {'search': 289180, 'single_legal': 57889, 'take_kill': 16432, 'mulligan': 8192, 'take_kill_roll': 1901, 'holdback_trade': 1071, 'holdback_resample': 506, 'consensus_lethal': 68}
- candidates per decision (2+): {'n': 290757, 'mean': 5.348, 'median': 5, 'p90': 10, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 90309 of 1478086 (6.11 %)
- gap between the best and second-best aggregate: {'n': 290757, 'mean': 4.798, 'median': 1.662, 'p90': 12.281, 'p99': 44.965, 'max': 160.0}
- hbk ran 2838 times, flipped 506; hbcheck ran 2332 times; lostrank ran 3397 times (played another move than the first best-scored one: 2597; left out of the search check)
- world line ends over all candidates (12439608; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4241846, 'O': 3765692, 'C': 1927269, 'L': 1056361, 't': 710888, 'D': 697583, 'T': 22630, 'c': 17178, 'd': 161}
- seconds per decision, one core: {'n': 375239, 'mean': 0.831, 'median': 0.2, 'p90': 2.524, 'p99': 6.012, 'max': 21.78}
- seconds per search decision, one core: {'n': 289180, 'mean': 1.066, 'median': 0.474, 'p90': 2.914, 'p99': 6.406, 'max': 21.78}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71275, 'mean': 4.353, 'median': 1.483, 'p90': 12.678, 'p99': 31.631, 'max': 83.006}
- decisions per game: {'n': 4096, 'mean': 91.611, 'median': 89, 'p90': 127, 'p99': 167, 'max': 225}; turns: {'n': 4096, 'mean': 8.989, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.562 over 4096 decisive games
- data: 89.3 MB in 4096 files; archive teach72-games.jsonl.gz 87.0 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.672 |
| meta-rune-test-subject | 0.598 |
| meta-portal-af | 0.586 |
| meta-abyss-midrange | 0.570 |
| meta-portal-evo | 0.557 |
| meta-sword-rally | 0.553 |
| meta-portal-cutthroat | 0.529 |
| meta-sword-loot | 0.500 |
| meta-dragon-ramp | 0.492 |
| meta-haven-kukishiro | 0.488 |
| meta-forest-combo | 0.449 |
| meta-rune-spell | 0.445 |
| meta-dragon-aggro | 0.420 |
| meta-abyss-aggro | 0.418 |
| meta-haven-amulet | 0.408 |
| meta-rune-crystal | 0.315 |
