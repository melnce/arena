# search101: Abysscraft search decks against the 16 meta lists, round 1 (runbook 101)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run100/leaf100q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 10101
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 2.39 h in 1 segment(s), 1715.4 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 258149 of 258149 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1431531 of 1431531 (100.00 %)
- decisions: 346533; with two or more candidates: 263309
- paths: {'search': 261626, 'single_legal': 57011, 'take_kill': 16816, 'mulligan': 8192, 'take_kill_roll': 1158, 'holdback_trade': 1143, 'holdback_resample': 540, 'consensus_lethal': 47}
- candidates per decision (2+): {'n': 263309, 'mean': 5.437, 'median': 5, 'p90': 10, 'p99': 17, 'max': 35}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 89284 of 1350307 (6.61 %)
- gap between the best and second-best aggregate: {'n': 263309, 'mean': 5.647, 'median': 1.815, 'p90': 14.724, 'p99': 54.585, 'max': 160.0}
- hbk ran 3159 times, flipped 540; hbcheck ran 2619 times; lostrank ran 3477 times (played another move than the first best-scored one: 2644; left out of the search check)
- world line ends over all candidates (11452248; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4026606, 'O': 3282055, 'C': 1659347, 'L': 1115734, 't': 757484, 'D': 587942, 'T': 17441, 'c': 5624, 'd': 15}
- seconds per decision, one core: {'n': 346533, 'mean': 0.691, 'median': 0.156, 'p90': 2.101, 'p99': 5.627, 'max': 19.393}
- seconds per search decision, one core: {'n': 261626, 'mean': 0.903, 'median': 0.349, 'p90': 2.498, 'p99': 6.058, 'max': 19.393}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 68934, 'mean': 3.455, 'median': 1.035, 'p90': 10.187, 'p99': 27.383, 'max': 81.629}
- decisions per game: {'n': 4096, 'mean': 84.603, 'median': 81, 'p90': 117, 'p99': 150, 'max': 180}; turns: {'n': 4096, 'mean': 8.683, 'median': 8, 'p90': 11, 'p99': 14, 'max': 16}
- first-seat win rate: 0.5322 over 4096 decisive games
- data: 81.6 MB in 4096 files; archive search101-games.jsonl.gz 79.4 MB

## The search seat

- the search seat (abysscraft): win rate 0.4526 over 4096 games; going first 0.4849, second 0.4204
- exploration-pool cards played in a deck: 69 of 113; decks per card: {'n': 113, 'mean': 623.575, 'median': 64, 'p90': 3136, 'p99': 4096, 'max': 4096}; fewest: [(0, '10001130'), (0, '10001210'), (0, '10002110'), (0, '10002210'), (0, '10051110')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the search seat's win rate, all decks (games) |
|---|---|
| meta-abyss-aggro | 0.6211 (256) |
| meta-abyss-midrange | 0.3086 (256) |
| meta-dragon-aggro | 0.7383 (256) |
| meta-dragon-ramp | 0.5391 (256) |
| meta-forest-combo | 0.4336 (256) |
| meta-haven-amulet | 0.4258 (256) |
| meta-haven-evo | 0.207 (256) |
| meta-haven-kukishiro | 0.3711 (256) |
| meta-portal-af | 0.3281 (256) |
| meta-portal-cutthroat | 0.543 (256) |
| meta-portal-evo | 0.2969 (256) |
| meta-rune-crystal | 0.6562 (256) |
| meta-rune-spell | 0.625 (256) |
| meta-rune-test-subject | 0.2617 (256) |
| meta-sword-loot | 0.5 (256) |
| meta-sword-rally | 0.3867 (256) |

## The search decks (each against the 16 lists; Wilson 95 % interval)

| deck | games | win rate | 95 % | first | second |
|---|---:|---:|---|---:|---:|
| s01 | 64 | 0.6406 | [0.5182, 0.7471] | 0.7812 | 0.5 |
| s18 | 64 | 0.6094 | [0.4869, 0.7194] | 0.5938 | 0.625 |
| s35 | 64 | 0.6094 | [0.4869, 0.7194] | 0.5625 | 0.6562 |
| s53 | 64 | 0.6094 | [0.4869, 0.7194] | 0.6562 | 0.5625 |
| s19 | 64 | 0.5938 | [0.4714, 0.7054] | 0.6562 | 0.5312 |
| s02 | 64 | 0.5781 | [0.4561, 0.6913] | 0.5625 | 0.5938 |
| s26 | 64 | 0.5625 | [0.4409, 0.6771] | 0.5938 | 0.5312 |
| c-midrange | 64 | 0.5469 | [0.4257, 0.6627] | 0.625 | 0.4688 |
| s03 | 64 | 0.5469 | [0.4257, 0.6627] | 0.5938 | 0.5 |
| s04 | 64 | 0.5469 | [0.4257, 0.6627] | 0.5625 | 0.5312 |
| s20 | 64 | 0.5469 | [0.4257, 0.6627] | 0.5312 | 0.5625 |
| s24 | 64 | 0.5469 | [0.4257, 0.6627] | 0.4688 | 0.625 |
| s34 | 64 | 0.5469 | [0.4257, 0.6627] | 0.5938 | 0.5 |
| s36 | 64 | 0.5312 | [0.4107, 0.6482] | 0.5625 | 0.5 |
| s49 | 64 | 0.5312 | [0.4107, 0.6482] | 0.6562 | 0.4062 |
| s17 | 64 | 0.5156 | [0.3958, 0.6337] | 0.4688 | 0.5625 |
| s33 | 64 | 0.5156 | [0.3958, 0.6337] | 0.5625 | 0.4688 |
| s38 | 64 | 0.5156 | [0.3958, 0.6337] | 0.4688 | 0.5625 |
| c-aggro | 64 | 0.5 | [0.381, 0.619] | 0.375 | 0.625 |
| s07 | 64 | 0.5 | [0.381, 0.619] | 0.5938 | 0.4062 |
| s22 | 64 | 0.5 | [0.381, 0.619] | 0.5312 | 0.4688 |
| s05 | 64 | 0.4844 | [0.3663, 0.6042] | 0.5312 | 0.4375 |
| s10 | 64 | 0.4844 | [0.3663, 0.6042] | 0.4688 | 0.5 |
| s23 | 64 | 0.4844 | [0.3663, 0.6042] | 0.5 | 0.4688 |
| s29 | 64 | 0.4844 | [0.3663, 0.6042] | 0.5312 | 0.4375 |
| s41 | 64 | 0.4844 | [0.3663, 0.6042] | 0.5312 | 0.4375 |
| s50 | 64 | 0.4844 | [0.3663, 0.6042] | 0.5312 | 0.4375 |
| m01 | 64 | 0.4688 | [0.3518, 0.5893] | 0.5312 | 0.4062 |
| s21 | 64 | 0.4688 | [0.3518, 0.5893] | 0.5312 | 0.4062 |
| s37 | 64 | 0.4688 | [0.3518, 0.5893] | 0.5312 | 0.4062 |
| s40 | 64 | 0.4688 | [0.3518, 0.5893] | 0.4688 | 0.4688 |
| s56 | 64 | 0.4688 | [0.3518, 0.5893] | 0.5 | 0.4375 |
| s28 | 64 | 0.4531 | [0.3373, 0.5743] | 0.5312 | 0.375 |
| s42 | 64 | 0.4531 | [0.3373, 0.5743] | 0.5938 | 0.3125 |
| m04 | 64 | 0.4375 | [0.3229, 0.5591] | 0.4375 | 0.4375 |
| s09 | 64 | 0.4375 | [0.3229, 0.5591] | 0.4688 | 0.4062 |
| s54 | 64 | 0.4375 | [0.3229, 0.5591] | 0.5312 | 0.3438 |
| m02 | 64 | 0.4219 | [0.3087, 0.5439] | 0.5312 | 0.3125 |
| s12 | 64 | 0.4219 | [0.3087, 0.5439] | 0.375 | 0.4688 |
| s27 | 64 | 0.4219 | [0.3087, 0.5439] | 0.5 | 0.3438 |
| s31 | 64 | 0.4219 | [0.3087, 0.5439] | 0.4375 | 0.4062 |
| s06 | 64 | 0.4062 | [0.2946, 0.5286] | 0.3438 | 0.4688 |
| s25 | 64 | 0.4062 | [0.2946, 0.5286] | 0.5312 | 0.2812 |
| s30 | 64 | 0.4062 | [0.2946, 0.5286] | 0.375 | 0.4375 |
| s43 | 64 | 0.4062 | [0.2946, 0.5286] | 0.4062 | 0.4062 |
| s55 | 64 | 0.4062 | [0.2946, 0.5286] | 0.4375 | 0.375 |
| s15 | 64 | 0.3906 | [0.2806, 0.5131] | 0.4375 | 0.3438 |
| s39 | 64 | 0.3906 | [0.2806, 0.5131] | 0.3125 | 0.4688 |
| s48 | 64 | 0.3906 | [0.2806, 0.5131] | 0.4062 | 0.375 |
| s51 | 64 | 0.3906 | [0.2806, 0.5131] | 0.3438 | 0.4375 |
| m03 | 64 | 0.375 | [0.2667, 0.4975] | 0.4688 | 0.2812 |
| s52 | 64 | 0.375 | [0.2667, 0.4975] | 0.3125 | 0.4375 |
| f01 | 64 | 0.3594 | [0.2529, 0.4818] | 0.375 | 0.3438 |
| s13 | 64 | 0.3594 | [0.2529, 0.4818] | 0.4062 | 0.3125 |
| s16 | 64 | 0.3594 | [0.2529, 0.4818] | 0.5312 | 0.1875 |
| s47 | 64 | 0.3594 | [0.2529, 0.4818] | 0.4062 | 0.3125 |
| s14 | 64 | 0.3438 | [0.2392, 0.466] | 0.5 | 0.1875 |
| s08 | 64 | 0.3281 | [0.2257, 0.45] | 0.3125 | 0.3438 |
| s11 | 64 | 0.3281 | [0.2257, 0.45] | 0.375 | 0.2812 |
| s45 | 64 | 0.3281 | [0.2257, 0.45] | 0.4375 | 0.2188 |
| s44 | 64 | 0.2969 | [0.199, 0.4177] | 0.2812 | 0.3125 |
| s46 | 64 | 0.2969 | [0.199, 0.4177] | 0.3438 | 0.25 |
| s32 | 64 | 0.2656 | [0.173, 0.3848] | 0.375 | 0.1562 |
| f02 | 64 | 0.25 | [0.1601, 0.3682] | 0.25 | 0.25 |

## Win rate by deck (both seats; the search decks together as one row)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.793 |
| meta-rune-test-subject | 0.738 |
| meta-portal-evo | 0.703 |
| meta-abyss-midrange | 0.691 |
| meta-portal-af | 0.672 |
| meta-haven-kukishiro | 0.629 |
| meta-sword-rally | 0.613 |
| meta-haven-amulet | 0.574 |
| meta-forest-combo | 0.566 |
| meta-sword-loot | 0.500 |
| meta-dragon-ramp | 0.461 |
| meta-portal-cutthroat | 0.457 |
| abyss-search | 0.453 |
| meta-abyss-aggro | 0.379 |
| meta-rune-spell | 0.375 |
| meta-rune-crystal | 0.344 |
| meta-dragon-aggro | 0.262 |
