# teach83: teacher data from the served search with the reply keys (runbook 83)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8301
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.15 h in 1 segment(s), 1301.2 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 293037 of 293037 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1608779 of 1608779 (100.00 %)
- decisions: 382906; with two or more candidates: 297780
- paths: {'search': 295994, 'single_legal': 58207, 'take_kill': 16618, 'mulligan': 8192, 'take_kill_roll': 2038, 'holdback_trade': 1214, 'holdback_resample': 572, 'consensus_lethal': 71}
- candidates per decision (2+): {'n': 297780, 'mean': 5.403, 'median': 5, 'p90': 10, 'p99': 16, 'max': 40}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 93803 of 1531326 (6.13 %)
- gap between the best and second-best aggregate: {'n': 297780, 'mean': 4.924, 'median': 1.69, 'p90': 12.654, 'p99': 46.166, 'max': 160.0}
- hbk ran 3064 times, flipped 572; hbcheck ran 2492 times; lostrank ran 2957 times (played another move than the first best-scored one: 2281; left out of the search check)
- world line ends over all candidates (12870232; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4339007, 'O': 3917821, 'C': 2012265, 'L': 1077188, 'D': 748474, 't': 727812, 'T': 24746, 'c': 22722, 'd': 197}
- seconds per decision, one core: {'n': 382906, 'mean': 0.826, 'median': 0.202, 'p90': 2.506, 'p99': 5.84, 'max': 20.536}
- seconds per search decision, one core: {'n': 295994, 'mean': 1.057, 'median': 0.485, 'p90': 2.877, 'p99': 6.239, 'max': 20.536}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 71702, 'mean': 4.393, 'median': 1.552, 'p90': 12.75, 'p99': 31.156, 'max': 89.853}
- decisions per game: {'n': 4096, 'mean': 93.483, 'median': 91, 'p90': 130, 'p99': 168, 'max': 233}; turns: {'n': 4096, 'mean': 9.036, 'median': 9, 'p90': 12, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5525 over 4096 decisive games
- data: 92.2 MB in 4096 files; archive teach83-games.jsonl.gz 89.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.641 |
| meta-haven-evo | 0.639 |
| meta-abyss-midrange | 0.565 |
| meta-portal-af | 0.549 |
| meta-dragon-ramp | 0.521 |
| meta-sword-rally | 0.516 |
| meta-portal-cutthroat | 0.498 |
| meta-portal-evo | 0.498 |
| meta-sword-loot | 0.498 |
| meta-haven-kukishiro | 0.496 |
| meta-forest-combo | 0.477 |
| meta-dragon-aggro | 0.463 |
| meta-rune-spell | 0.440 |
| meta-abyss-aggro | 0.432 |
| meta-haven-amulet | 0.416 |
| meta-rune-crystal | 0.353 |
