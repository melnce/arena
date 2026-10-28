# teach70: teacher data from the served search with the reply keys (runbook 70)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 7001
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 6.31 h in 5 segment(s), 1299.1 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 573049 of 573049 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 3109641 of 3109641 (100.00 %)
- decisions: 752024; with two or more candidates: 583013
- paths: {'search': 579723, 'single_legal': 115877, 'take_kill': 33135, 'mulligan': 16384, 'take_kill_roll': 3458, 'holdback_trade': 2236, 'holdback_resample': 1054, 'consensus_lethal': 157}
- candidates per decision (2+): {'n': 583013, 'mean': 5.334, 'median': 5, 'p90': 10, 'p99': 16, 'max': 38}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 182584 of 2957935 (6.17 %)
- gap between the best and second-best aggregate: {'n': 583013, 'mean': 4.769, 'median': 1.641, 'p90': 12.239, 'p99': 44.631, 'max': 160.0}
- hbk ran 5749 times, flipped 1054; hbcheck ran 4695 times; lostrank ran 6674 times (played another move than the first best-scored one: 5076; left out of the search check)
- world line ends over all candidates (24877128; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8550213, 'O': 7496436, 'C': 3830146, 'L': 2087774, 't': 1437512, 'D': 1399540, 'T': 47796, 'c': 27217, 'd': 494}
- seconds per decision, one core: {'n': 752024, 'mean': 0.835, 'median': 0.196, 'p90': 2.549, 'p99': 6.018, 'max': 22.228}
- seconds per search decision, one core: {'n': 579723, 'mean': 1.076, 'median': 0.481, 'p90': 2.94, 'p99': 6.443, 'max': 22.035}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 142338, 'mean': 4.411, 'median': 1.534, 'p90': 12.764, 'p99': 31.93, 'max': 93.894}
- decisions per game: {'n': 8192, 'mean': 91.8, 'median': 89, 'p90': 128, 'p99': 171, 'max': 243}; turns: {'n': 8192, 'mean': 8.976, 'median': 9, 'p90': 12, 'p99': 15, 'max': 22}
- first-seat win rate: 0.5598 over 8192 decisive games
- data: 178.5 MB in 8192 files; archive teach70-games.jsonl.gz 173.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.640 |
| meta-haven-evo | 0.622 |
| meta-portal-evo | 0.600 |
| meta-portal-af | 0.573 |
| meta-sword-rally | 0.564 |
| meta-abyss-midrange | 0.535 |
| meta-portal-cutthroat | 0.522 |
| meta-sword-loot | 0.519 |
| meta-haven-kukishiro | 0.496 |
| meta-dragon-ramp | 0.483 |
| meta-forest-combo | 0.445 |
| meta-dragon-aggro | 0.435 |
| meta-abyss-aggro | 0.414 |
| meta-haven-amulet | 0.414 |
| meta-rune-spell | 0.412 |
| meta-rune-crystal | 0.326 |
