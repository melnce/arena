# teach71: teacher data from the served search with the reply keys (runbook 71)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 7101
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.02 h in 1 segment(s), 1355.2 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 288188 of 288188 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1566059 of 1566059 (100.00 %)
- decisions: 378211; with two or more candidates: 293118
- paths: {'search': 291456, 'single_legal': 58426, 'take_kill': 16544, 'mulligan': 8192, 'take_kill_roll': 1861, 'holdback_trade': 1085, 'holdback_resample': 577, 'consensus_lethal': 70}
- candidates per decision (2+): {'n': 293118, 'mean': 5.343, 'median': 5, 'p90': 10, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91684 of 1490830 (6.15 %)
- gap between the best and second-best aggregate: {'n': 293118, 'mean': 4.784, 'median': 1.652, 'p90': 12.236, 'p99': 45.13, 'max': 160.0}
- hbk ran 2882 times, flipped 577; hbcheck ran 2305 times; lostrank ran 3268 times (played another move than the first best-scored one: 2456; left out of the search check)
- world line ends over all candidates (12528472; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4306346, 'O': 3789021, 'C': 1921746, 'L': 1040545, 't': 732110, 'D': 702199, 'T': 22343, 'c': 14014, 'd': 148}
- seconds per decision, one core: {'n': 378211, 'mean': 0.802, 'median': 0.187, 'p90': 2.449, 'p99': 5.81, 'max': 28.172}
- seconds per search decision, one core: {'n': 291456, 'mean': 1.034, 'median': 0.458, 'p90': 2.827, 'p99': 6.197, 'max': 17.63}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71599, 'mean': 4.237, 'median': 1.463, 'p90': 12.37, 'p99': 30.652, 'max': 76.622}
- decisions per game: {'n': 4096, 'mean': 92.337, 'median': 90, 'p90': 127, 'p99': 168, 'max': 218}; turns: {'n': 4096, 'mean': 9.025, 'median': 9, 'p90': 12, 'p99': 15, 'max': 21}
- first-seat win rate: 0.5518 over 4096 decisive games
- data: 89.9 MB in 4096 files; archive teach71-games.jsonl.gz 87.6 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.644 |
| meta-rune-test-subject | 0.633 |
| meta-portal-evo | 0.580 |
| meta-portal-af | 0.562 |
| meta-portal-cutthroat | 0.537 |
| meta-abyss-midrange | 0.535 |
| meta-haven-kukishiro | 0.514 |
| meta-sword-loot | 0.512 |
| meta-sword-rally | 0.512 |
| meta-dragon-ramp | 0.494 |
| meta-forest-combo | 0.457 |
| meta-abyss-aggro | 0.430 |
| meta-dragon-aggro | 0.426 |
| meta-rune-spell | 0.426 |
| meta-haven-amulet | 0.418 |
| meta-rune-crystal | 0.320 |
