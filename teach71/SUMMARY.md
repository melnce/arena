# teach71: teacher data from the served search with the reply keys (runbook 71)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 7101
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 5.96 h in 5 segment(s), 1374.5 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 571636 of 571636 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3105279 of 3105279 (100.00 %)
- decisions: 751248; with two or more candidates: 581522
- paths: {'search': 578289, 'single_legal': 116233, 'take_kill': 33261, 'mulligan': 16384, 'take_kill_roll': 3716, 'holdback_trade': 2155, 'holdback_resample': 1078, 'consensus_lethal': 132}
- candidates per decision (2+): {'n': 581522, 'mean': 5.34, 'median': 5, 'p90': 10, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 181555 of 2953932 (6.15 %)
- gap between the best and second-best aggregate: {'n': 581522, 'mean': 4.792, 'median': 1.651, 'p90': 12.285, 'p99': 44.842, 'max': 160.0}
- hbk ran 5740 times, flipped 1078; hbcheck ran 4662 times; lostrank ran 6653 times (played another move than the first best-scored one: 5028; left out of the search check)
- world line ends over all candidates (24842232; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8548924, 'O': 7499251, 'C': 3791135, 'L': 2085348, 't': 1454438, 'D': 1388397, 'T': 46714, 'c': 27600, 'd': 425}
- seconds per decision, one core: {'n': 751248, 'mean': 0.792, 'median': 0.183, 'p90': 2.419, 'p99': 5.739, 'max': 28.172}
- seconds per search decision, one core: {'n': 578289, 'mean': 1.022, 'median': 0.451, 'p90': 2.797, 'p99': 6.12, 'max': 19.777}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142481, 'mean': 4.174, 'median': 1.43, 'p90': 12.137, 'p99': 30.055, 'max': 76.622}
- decisions per game: {'n': 8192, 'mean': 91.705, 'median': 89, 'p90': 127, 'p99': 167, 'max': 219}; turns: {'n': 8192, 'mean': 8.983, 'median': 9, 'p90': 12, 'p99': 15, 'max': 24}
- first-seat win rate: 0.5575 over 8192 decisive games
- data: 178.3 MB in 8192 files; archive teach71-games.jsonl.gz 173.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.646 |
| meta-rune-test-subject | 0.629 |
| meta-portal-evo | 0.585 |
| meta-portal-af | 0.554 |
| meta-abyss-midrange | 0.536 |
| meta-portal-cutthroat | 0.536 |
| meta-sword-rally | 0.532 |
| meta-sword-loot | 0.504 |
| meta-haven-kukishiro | 0.493 |
| meta-dragon-ramp | 0.488 |
| meta-forest-combo | 0.458 |
| meta-abyss-aggro | 0.435 |
| meta-dragon-aggro | 0.434 |
| meta-haven-amulet | 0.425 |
| meta-rune-spell | 0.413 |
| meta-rune-crystal | 0.331 |
