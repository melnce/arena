# teach67: teacher data from the served search with the reply keys (runbook 67)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6701
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.07 h in 1 segment(s), 1335.5 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 282878 of 282878 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1534776 of 1534776 (100.00 %)
- decisions: 372998; with two or more candidates: 288034
- paths: {'search': 286385, 'single_legal': 58124, 'take_kill': 17052, 'mulligan': 8192, 'take_kill_roll': 1529, 'holdback_trade': 1048, 'holdback_resample': 601, 'consensus_lethal': 67}
- candidates per decision (2+): {'n': 288034, 'mean': 5.328, 'median': 5, 'p90': 10, 'p99': 16, 'max': 34}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 90179 of 1458629 (6.18 %)
- gap between the best and second-best aggregate: {'n': 288034, 'mean': 4.764, 'median': 1.644, 'p90': 12.091, 'p99': 44.94, 'max': 160.0}
- hbk ran 2870 times, flipped 601; hbcheck ran 2269 times; lostrank ran 3507 times (played another move than the first best-scored one: 2607; left out of the search check)
- world line ends over all candidates (12278208; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4235722, 'O': 3699158, 'C': 1884536, 'L': 1026811, 't': 716150, 'D': 678737, 'T': 21767, 'c': 15154, 'd': 173}
- seconds per decision, one core: {'n': 372998, 'mean': 0.826, 'median': 0.189, 'p90': 2.518, 'p99': 6.049, 'max': 19.246}
- seconds per search decision, one core: {'n': 286385, 'mean': 1.069, 'median': 0.469, 'p90': 2.918, 'p99': 6.463, 'max': 18.165}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71001, 'mean': 4.339, 'median': 1.493, 'p90': 12.658, 'p99': 31.525, 'max': 131.204}
- decisions per game: {'n': 4096, 'mean': 91.064, 'median': 88, 'p90': 128, 'p99': 169, 'max': 225}; turns: {'n': 4096, 'mean': 8.952, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5596 over 4096 decisive games
- data: 87.9 MB in 4096 files; archive teach67-games.jsonl.gz 85.6 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.629 |
| meta-haven-evo | 0.611 |
| meta-abyss-midrange | 0.568 |
| meta-sword-rally | 0.568 |
| meta-portal-evo | 0.551 |
| meta-portal-af | 0.547 |
| meta-sword-loot | 0.545 |
| meta-dragon-ramp | 0.518 |
| meta-forest-combo | 0.488 |
| meta-portal-cutthroat | 0.471 |
| meta-abyss-aggro | 0.453 |
| meta-haven-kukishiro | 0.434 |
| meta-dragon-aggro | 0.424 |
| meta-haven-amulet | 0.420 |
| meta-rune-spell | 0.400 |
| meta-rune-crystal | 0.373 |
