# teach105: Forestcraft and Portalcraft exploration games against the 16 meta lists (runbook 105)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run100/leaf100q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 10501
- games: 14336 of 14336 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 9.16 h in 7 segment(s), 1564.4 games/h
- replay check (every game replayed from its seed and actions): 14336 of 14336 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 902972 of 902972 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 4869097 of 4869097 (100.00 %)
- decisions: 1203636; with two or more candidates: 921949
- paths: {'search': 915218, 'single_legal': 188306, 'take_kill': 59670, 'mulligan': 28672, 'take_kill_roll': 4862, 'holdback_trade': 4666, 'holdback_resample': 2065, 'consensus_lethal': 177}
- candidates per decision (2+): {'n': 921949, 'mean': 5.281, 'median': 5, 'p90': 9, 'p99': 15, 'max': 42}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 230626 of 4636188 (4.97 %)
- gap between the best and second-best aggregate: {'n': 921949, 'mean': 3.924, 'median': 1.2, 'p90': 10.13, 'p99': 37.921, 'max': 160.0}
- hbk ran 11169 times, flipped 2065; hbcheck ran 9104 times; lostrank ran 12246 times (played another move than the first best-scored one: 9556; left out of the search check)
- world line ends over all candidates (38952776; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 13482868, 'O': 12636147, 'C': 5424409, 'L': 2942330, 't': 2407160, 'D': 1980755, 'T': 55879, 'c': 22902, 'd': 326}
- seconds per decision, one core: {'n': 1203636, 'mean': 0.761, 'median': 0.193, 'p90': 2.326, 'p99': 5.577, 'max': 27.258}
- seconds per search decision, one core: {'n': 915218, 'mean': 0.986, 'median': 0.424, 'p90': 2.713, 'p99': 5.961, 'max': 19.982}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 239428, 'mean': 3.8, 'median': 1.336, 'p90': 10.949, 'p99': 27.58, 'max': 90.965}
- decisions per game: {'n': 14336, 'mean': 83.959, 'median': 80, 'p90': 118, 'p99': 160, 'max': 229}; turns: {'n': 14336, 'mean': 8.616, 'median': 8, 'p90': 11, 'p99': 14, 'max': 22}
- first-seat win rate: 0.5233 over 14336 decisive games
- data: 287.7 MB in 14336 files; archive teach105-games.jsonl.gz 280.2 MB

## Exploration

- the exploring seat: win rate 0.1321 over 14336 games; forestcraft 0.125 (7168), portalcraft 0.1392 (7168); rebuilt decks 0.1992, scratch decks 0.065; going first 0.1554, second 0.1088
- exploration-pool cards played in a deck: 179 of 179; decks per card: {'n': 179, 'mean': 1997.335, 'median': 1840, 'p90': 3327, 'p99': 5366, 'max': 6564}; fewest: [(956, '10971120'), (978, '10011130'), (984, '10812310'), (988, '10812120'), (998, '10972120')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the exploring seat's win rate (games) | forestcraft | portalcraft |
|---|---|---|---|
| meta-abyss-aggro | 0.3013 (896) | 0.3371 | 0.2656 |
| meta-abyss-midrange | 0.0748 (896) | 0.0603 | 0.0893 |
| meta-dragon-aggro | 0.2801 (896) | 0.2656 | 0.2946 |
| meta-dragon-ramp | 0.1473 (896) | 0.1652 | 0.1295 |
| meta-forest-combo | 0.1496 (896) | 0.1362 | 0.1629 |
| meta-haven-amulet | 0.1473 (896) | 0.1272 | 0.1674 |
| meta-haven-evo | 0.048 (896) | 0.0446 | 0.0513 |
| meta-haven-kukishiro | 0.0915 (896) | 0.0647 | 0.1183 |
| meta-portal-af | 0.0603 (896) | 0.0446 | 0.0759 |
| meta-portal-cutthroat | 0.1027 (896) | 0.1004 | 0.1049 |
| meta-portal-evo | 0.0658 (896) | 0.0491 | 0.0826 |
| meta-rune-crystal | 0.2455 (896) | 0.2277 | 0.2634 |
| meta-rune-spell | 0.1362 (896) | 0.1362 | 0.1362 |
| meta-rune-test-subject | 0.0469 (896) | 0.0357 | 0.058 |
| meta-sword-loot | 0.1239 (896) | 0.1228 | 0.125 |
| meta-sword-rally | 0.0926 (896) | 0.0826 | 0.1027 |

## Win rate by deck (both seats; the exploration decks together as one row)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.953 |
| meta-haven-evo | 0.952 |
| meta-portal-af | 0.940 |
| meta-portal-evo | 0.934 |
| meta-abyss-midrange | 0.925 |
| meta-haven-kukishiro | 0.908 |
| meta-sword-rally | 0.907 |
| meta-portal-cutthroat | 0.897 |
| meta-sword-loot | 0.876 |
| meta-rune-spell | 0.864 |
| meta-dragon-ramp | 0.853 |
| meta-haven-amulet | 0.853 |
| meta-forest-combo | 0.850 |
| meta-rune-crystal | 0.754 |
| meta-dragon-aggro | 0.720 |
| meta-abyss-aggro | 0.699 |
| fp-explore | 0.132 |
