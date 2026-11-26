# teach105: Forestcraft and Portalcraft exploration games against the 16 meta lists (runbook 105)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run100/leaf100q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 10501
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 5.24 h in 1 segment(s), 1563.7 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 516612 of 516612 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2785595 of 2785595 (100.00 %)
- decisions: 688233; with two or more candidates: 527317
- paths: {'search': 523470, 'single_legal': 107725, 'take_kill': 34135, 'mulligan': 16384, 'holdback_trade': 2688, 'take_kill_roll': 2575, 'holdback_resample': 1159, 'consensus_lethal': 97}
- candidates per decision (2+): {'n': 527317, 'mean': 5.283, 'median': 5, 'p90': 9, 'p99': 15, 'max': 34}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 132499 of 2653875 (4.99 %)
- gap between the best and second-best aggregate: {'n': 527317, 'mean': 3.947, 'median': 1.209, 'p90': 10.151, 'p99': 38.272, 'max': 160.0}
- hbk ran 6374 times, flipped 1159; hbcheck ran 5215 times; lostrank ran 6858 times (played another move than the first best-scored one: 5381; left out of the search check)
- world line ends over all candidates (22284760; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 7706429, 'O': 7232309, 'C': 3113859, 'L': 1675908, 't': 1378046, 'D': 1135010, 'T': 30305, 'c': 12706, 'd': 188}
- seconds per decision, one core: {'n': 688233, 'mean': 0.765, 'median': 0.194, 'p90': 2.34, 'p99': 5.604, 'max': 27.258}
- seconds per search decision, one core: {'n': 523470, 'mean': 0.992, 'median': 0.426, 'p90': 2.727, 'p99': 6.003, 'max': 19.982}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 136886, 'mean': 3.822, 'median': 1.334, 'p90': 11.023, 'p99': 27.698, 'max': 90.965}
- decisions per game: {'n': 8192, 'mean': 84.013, 'median': 80, 'p90': 118, 'p99': 161, 'max': 227}; turns: {'n': 8192, 'mean': 8.619, 'median': 8, 'p90': 11, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5223 over 8192 decisive games
- data: 164.7 MB in 8192 files; archive teach105-games.jsonl.gz 160.4 MB

## Exploration

- the exploring seat: win rate 0.1322 over 8192 games; forestcraft 0.1265 (4096), portalcraft 0.1379 (4096); rebuilt decks 0.1985, scratch decks 0.0659; going first 0.1545, second 0.1099
- exploration-pool cards played in a deck: 179 of 179; decks per card: {'n': 179, 'mean': 1141.419, 'median': 1046, 'p90': 1904, 'p99': 3053, 'max': 3744}; fewest: [(524, '10971120'), (536, '10812120'), (554, '10011130'), (555, '10812310'), (557, '10571110')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the exploring seat's win rate (games) | forestcraft | portalcraft |
|---|---|---|---|
| meta-abyss-aggro | 0.3066 (512) | 0.3438 | 0.2695 |
| meta-abyss-midrange | 0.0703 (512) | 0.0586 | 0.082 |
| meta-dragon-aggro | 0.291 (512) | 0.2852 | 0.2969 |
| meta-dragon-ramp | 0.1445 (512) | 0.1641 | 0.125 |
| meta-forest-combo | 0.1543 (512) | 0.1484 | 0.1602 |
| meta-haven-amulet | 0.1387 (512) | 0.0977 | 0.1797 |
| meta-haven-evo | 0.0508 (512) | 0.0547 | 0.0469 |
| meta-haven-kukishiro | 0.0898 (512) | 0.0586 | 0.1211 |
| meta-portal-af | 0.0664 (512) | 0.0547 | 0.0781 |
| meta-portal-cutthroat | 0.0938 (512) | 0.0938 | 0.0938 |
| meta-portal-evo | 0.0586 (512) | 0.0273 | 0.0898 |
| meta-rune-crystal | 0.2246 (512) | 0.2031 | 0.2461 |
| meta-rune-spell | 0.1484 (512) | 0.1445 | 0.1523 |
| meta-rune-test-subject | 0.0527 (512) | 0.0391 | 0.0664 |
| meta-sword-loot | 0.127 (512) | 0.1445 | 0.1094 |
| meta-sword-rally | 0.0977 (512) | 0.1055 | 0.0898 |

## Win rate by deck (both seats; the exploration decks together as one row)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.949 |
| meta-rune-test-subject | 0.947 |
| meta-portal-evo | 0.941 |
| meta-portal-af | 0.934 |
| meta-abyss-midrange | 0.930 |
| meta-haven-kukishiro | 0.910 |
| meta-portal-cutthroat | 0.906 |
| meta-sword-rally | 0.902 |
| meta-sword-loot | 0.873 |
| meta-haven-amulet | 0.861 |
| meta-dragon-ramp | 0.856 |
| meta-rune-spell | 0.852 |
| meta-forest-combo | 0.846 |
| meta-rune-crystal | 0.775 |
| meta-dragon-aggro | 0.709 |
| meta-abyss-aggro | 0.693 |
| fp-explore | 0.132 |
