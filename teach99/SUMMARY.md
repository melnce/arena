# teach99: Abysscraft exploration games against the 16 meta lists (runbook 99)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9901
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 2.0 h in 1 segment(s), 2050.8 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 235974 of 235974 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1272468 of 1272468 (100.00 %)
- decisions: 319700; with two or more candidates: 241559
- paths: {'search': 240091, 'single_legal': 52980, 'take_kill': 15699, 'mulligan': 8192, 'take_kill_roll': 1217, 'holdback_trade': 1001, 'holdback_resample': 467, 'consensus_lethal': 53}
- candidates per decision (2+): {'n': 241559, 'mean': 5.268, 'median': 4, 'p90': 9, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 62231 of 1203193 (5.17 %)
- gap between the best and second-best aggregate: {'n': 241559, 'mean': 4.429, 'median': 1.332, 'p90': 11.653, 'p99': 42.155, 'max': 160.0}
- hbk ran 2617 times, flipped 467; hbcheck ran 2150 times; lostrank ran 4117 times (played another move than the first best-scored one: 3246; left out of the search check)
- world line ends over all candidates (10179744; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 3634797, 'O': 3122187, 'C': 1327724, 'L': 844671, 't': 732852, 'D': 490903, 'T': 20847, 'c': 5729, 'd': 34}
- seconds per decision, one core: {'n': 319700, 'mean': 0.626, 'median': 0.141, 'p90': 1.923, 'p99': 5.191, 'max': 18.187}
- seconds per search decision, one core: {'n': 240091, 'mean': 0.821, 'median': 0.299, 'p90': 2.308, 'p99': 5.587, 'max': 17.367}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 65690, 'mean': 3.024, 'median': 0.844, 'p90': 8.955, 'p99': 24.609, 'max': 83.662}
- decisions per game: {'n': 4096, 'mean': 78.052, 'median': 74, 'p90': 112, 'p99': 153, 'max': 224}; turns: {'n': 4096, 'mean': 8.28, 'median': 8, 'p90': 11, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5144 over 4096 decisive games
- data: 74.9 MB in 4096 files; archive teach99-games.jsonl.gz 72.7 MB

## Exploration

- the exploring seat (abysscraft): win rate 0.1526 over 4096 games; rebuilt decks 0.228, scratch decks 0.0771; going first 0.167, second 0.1382
- exploration-pool cards played in a deck: 113 of 113; decks per card: {'n': 113, 'mean': 872.699, 'median': 671, 'p90': 1394, 'p99': 2557, 'max': 2856}; fewest: [(543, '10603210'), (546, '10802110'), (555, '10453110'), (566, '10404110'), (568, '10052120')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the exploring seat's win rate (games) |
|---|---|
| meta-abyss-aggro | 0.2148 (256) |
| meta-abyss-midrange | 0.1094 (256) |
| meta-dragon-aggro | 0.2266 (256) |
| meta-dragon-ramp | 0.1992 (256) |
| meta-forest-combo | 0.1484 (256) |
| meta-haven-amulet | 0.2109 (256) |
| meta-haven-evo | 0.1562 (256) |
| meta-haven-kukishiro | 0.1484 (256) |
| meta-portal-af | 0.0859 (256) |
| meta-portal-cutthroat | 0.1289 (256) |
| meta-portal-evo | 0.0859 (256) |
| meta-rune-crystal | 0.2188 (256) |
| meta-rune-spell | 0.1523 (256) |
| meta-rune-test-subject | 0.0977 (256) |
| meta-sword-loot | 0.1602 (256) |
| meta-sword-rally | 0.0977 (256) |

## Win rate by deck (both seats; the exploration decks together as one row)

| deck | win rate |
|---|---|
| meta-portal-af | 0.914 |
| meta-portal-evo | 0.914 |
| meta-rune-test-subject | 0.902 |
| meta-sword-rally | 0.902 |
| meta-abyss-midrange | 0.891 |
| meta-portal-cutthroat | 0.871 |
| meta-forest-combo | 0.852 |
| meta-haven-kukishiro | 0.852 |
| meta-rune-spell | 0.848 |
| meta-haven-evo | 0.844 |
| meta-sword-loot | 0.840 |
| meta-dragon-ramp | 0.801 |
| meta-haven-amulet | 0.789 |
| meta-abyss-aggro | 0.785 |
| meta-rune-crystal | 0.781 |
| meta-dragon-aggro | 0.773 |
| abyss-explore | 0.153 |
