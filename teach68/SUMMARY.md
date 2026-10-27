# teach68: teacher data from the served search with the reply keys (runbook 68)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6801
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.29 h in 5 segment(s), 1302.7 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 572697 of 572697 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3107351 of 3107351 (100.00 %)
- decisions: 752181; with two or more candidates: 582770
- paths: {'search': 579533, 'single_legal': 116447, 'take_kill': 32909, 'mulligan': 16384, 'take_kill_roll': 3519, 'holdback_trade': 2098, 'holdback_resample': 1139, 'consensus_lethal': 152}
- candidates per decision (2+): {'n': 582770, 'mean': 5.332, 'median': 5, 'p90': 10, 'p99': 16, 'max': 40}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 183741 of 2953451 (6.22 %)
- gap between the best and second-best aggregate: {'n': 582770, 'mean': 4.757, 'median': 1.649, 'p90': 12.141, 'p99': 44.524, 'max': 160.0}
- hbk ran 5778 times, flipped 1139; hbcheck ran 4639 times; lostrank ran 6836 times (played another move than the first best-scored one: 5209; left out of the search check)
- world line ends over all candidates (24858808; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8530804, 'O': 7492133, 'C': 3842668, 'L': 2084727, 't': 1444050, 'D': 1384909, 'T': 47873, 'c': 31290, 'd': 354}
- seconds per decision, one core: {'n': 752181, 'mean': 0.826, 'median': 0.194, 'p90': 2.523, 'p99': 5.979, 'max': 42.587}
- seconds per search decision, one core: {'n': 579533, 'mean': 1.065, 'median': 0.474, 'p90': 2.909, 'p99': 6.372, 'max': 42.587}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142798, 'mean': 4.351, 'median': 1.521, 'p90': 12.64, 'p99': 30.819, 'max': 87.74}
- decisions per game: {'n': 8192, 'mean': 91.819, 'median': 89, 'p90': 128, 'p99': 170, 'max': 234}; turns: {'n': 8192, 'mean': 8.999, 'median': 9, 'p90': 12, 'p99': 14, 'max': 27}
- first-seat win rate: 0.5524 over 8192 decisive games
- data: 177.8 MB in 8192 files; archive teach68-games.jsonl.gz 173.2 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.644 |
| meta-rune-test-subject | 0.643 |
| meta-portal-evo | 0.596 |
| meta-sword-rally | 0.578 |
| meta-portal-af | 0.570 |
| meta-abyss-midrange | 0.568 |
| meta-portal-cutthroat | 0.515 |
| meta-dragon-ramp | 0.497 |
| meta-sword-loot | 0.495 |
| meta-haven-kukishiro | 0.455 |
| meta-abyss-aggro | 0.448 |
| meta-forest-combo | 0.440 |
| meta-dragon-aggro | 0.420 |
| meta-rune-spell | 0.411 |
| meta-haven-amulet | 0.399 |
| meta-rune-crystal | 0.319 |
