# teach68: teacher data from the served search with the reply keys (runbook 68)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json`
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`, seed 6801
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.1 h in 1 segment(s), 1322.8 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 286332 of 286332 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1550975 of 1550975 (100.00 %)
- decisions: 375973; with two or more candidates: 291477
- paths: {'search': 289857, 'single_legal': 58169, 'take_kill': 16284, 'mulligan': 8192, 'take_kill_roll': 1770, 'holdback_trade': 1054, 'holdback_resample': 566, 'consensus_lethal': 81}
- candidates per decision (2+): {'n': 291477, 'mean': 5.321, 'median': 5, 'p90': 9, 'p99': 16, 'max': 34}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 91227 of 1473514 (6.19 %)
- gap between the best and second-best aggregate: {'n': 291477, 'mean': 4.731, 'median': 1.65, 'p90': 12.047, 'p99': 44.641, 'max': 160.0}
- hbk ran 2831 times, flipped 566; hbcheck ran 2265 times; lostrank ran 3525 times (played another move than the first best-scored one: 2725; left out of the search check)
- world line ends over all candidates (12407800; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4259696, 'O': 3743547, 'C': 1903391, 'L': 1045279, 't': 727208, 'D': 689540, 'T': 23567, 'c': 15423, 'd': 149}
- seconds per decision, one core: {'n': 375973, 'mean': 0.827, 'median': 0.195, 'p90': 2.525, 'p99': 6.012, 'max': 42.587}
- seconds per search decision, one core: {'n': 289857, 'mean': 1.066, 'median': 0.474, 'p90': 2.909, 'p99': 6.396, 'max': 42.587}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71260, 'mean': 4.365, 'median': 1.524, 'p90': 12.687, 'p99': 31.006, 'max': 87.74}
- decisions per game: {'n': 4096, 'mean': 91.79, 'median': 89, 'p90': 128, 'p99': 169, 'max': 218}; turns: {'n': 4096, 'mean': 8.98, 'median': 9, 'p90': 12, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5488 over 4096 decisive games
- data: 88.8 MB in 4096 files; archive teach68-games.jsonl.gz 86.5 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.635 |
| meta-haven-evo | 0.633 |
| meta-portal-evo | 0.604 |
| meta-sword-rally | 0.582 |
| meta-abyss-midrange | 0.572 |
| meta-portal-af | 0.572 |
| meta-dragon-ramp | 0.514 |
| meta-portal-cutthroat | 0.508 |
| meta-sword-loot | 0.504 |
| meta-abyss-aggro | 0.461 |
| meta-haven-kukishiro | 0.455 |
| meta-rune-spell | 0.426 |
| meta-forest-combo | 0.420 |
| meta-haven-amulet | 0.404 |
| meta-dragon-aggro | 0.398 |
| meta-rune-crystal | 0.312 |
