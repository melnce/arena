# search101: Abysscraft search decks against the 16 meta lists, round 1 (runbook 101)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run100/leaf100q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 10101
- games: 8192 of 8192 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 4.78 h in 3 segment(s), 1712.9 games/h
- replay check (every game replayed from its seed and actions): 8192 of 8192 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 517093 of 517093 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 2869199 of 2869199 (100.00 %)
- decisions: 693279; with two or more candidates: 527211
- paths: {'search': 523971, 'single_legal': 113797, 'take_kill': 33400, 'mulligan': 16384, 'take_kill_roll': 2379, 'holdback_trade': 2146, 'holdback_resample': 1094, 'consensus_lethal': 108}
- candidates per decision (2+): {'n': 527211, 'mean': 5.442, 'median': 5, 'p90': 10, 'p99': 17, 'max': 36}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 178102 of 2707856 (6.58 %)
- gap between the best and second-best aggregate: {'n': 527211, 'mean': 5.64, 'median': 1.802, 'p90': 14.739, 'p99': 54.947, 'max': 160.0}
- hbk ran 6094 times, flipped 1094; hbcheck ran 5000 times; lostrank ran 6878 times (played another move than the first best-scored one: 5275; left out of the search check)
- world line ends over all candidates (22953592; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 8091056, 'O': 6583993, 'C': 3296150, 'L': 2213145, 't': 1547709, 'D': 1173726, 'T': 35611, 'c': 12166, 'd': 36}
- seconds per decision, one core: {'n': 693279, 'mean': 0.691, 'median': 0.157, 'p90': 2.099, 'p99': 5.604, 'max': 20.974}
- seconds per search decision, one core: {'n': 523971, 'mean': 0.902, 'median': 0.351, 'p90': 2.489, 'p99': 6.049, 'max': 19.393}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 137454, 'mean': 3.465, 'median': 1.071, 'p90': 10.181, 'p99': 27.231, 'max': 81.642}
- decisions per game: {'n': 8192, 'mean': 84.629, 'median': 81, 'p90': 117, 'p99': 153, 'max': 184}; turns: {'n': 8192, 'mean': 8.662, 'median': 8, 'p90': 11, 'p99': 14, 'max': 18}
- first-seat win rate: 0.5411 over 8192 decisive games
- data: 163.5 MB in 8192 files; archive search101-games.jsonl.gz 159.1 MB

## The search seat

- the search seat (abysscraft): win rate 0.4562 over 8192 games; going first 0.4973, second 0.415
- exploration-pool cards played in a deck: 69 of 113; decks per card: {'n': 113, 'mean': 1247.15, 'median': 128, 'p90': 6272, 'p99': 8192, 'max': 8192}; fewest: [(0, '10001130'), (0, '10001210'), (0, '10002110'), (0, '10002210'), (0, '10051110')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the search seat's win rate, all decks (games) |
|---|---|
| meta-abyss-aggro | 0.5781 (512) |
| meta-abyss-midrange | 0.3633 (512) |
| meta-dragon-aggro | 0.6699 (512) |
| meta-dragon-ramp | 0.5547 (512) |
| meta-forest-combo | 0.4551 (512) |
| meta-haven-amulet | 0.4395 (512) |
| meta-haven-evo | 0.2129 (512) |
| meta-haven-kukishiro | 0.377 (512) |
| meta-portal-af | 0.373 (512) |
| meta-portal-cutthroat | 0.5098 (512) |
| meta-portal-evo | 0.2871 (512) |
| meta-rune-crystal | 0.6738 (512) |
| meta-rune-spell | 0.5625 (512) |
| meta-rune-test-subject | 0.291 (512) |
| meta-sword-loot | 0.4863 (512) |
| meta-sword-rally | 0.4648 (512) |

## The search decks (each against the 16 lists; Wilson 95 % interval)

| deck | games | win rate | 95 % | first | second |
|---|---:|---:|---|---:|---:|
| s53 | 128 | 0.5781 | [0.4915, 0.6602] | 0.6094 | 0.5469 |
| s05 | 128 | 0.5703 | [0.4837, 0.6528] | 0.5781 | 0.5625 |
| s35 | 128 | 0.5703 | [0.4837, 0.6528] | 0.5625 | 0.5781 |
| s02 | 128 | 0.5625 | [0.476, 0.6454] | 0.6094 | 0.5156 |
| s18 | 128 | 0.5625 | [0.476, 0.6454] | 0.5938 | 0.5312 |
| s19 | 128 | 0.5625 | [0.476, 0.6454] | 0.625 | 0.5 |
| s20 | 128 | 0.5625 | [0.476, 0.6454] | 0.5938 | 0.5312 |
| s49 | 128 | 0.5547 | [0.4682, 0.6379] | 0.6562 | 0.4531 |
| s01 | 128 | 0.5469 | [0.4605, 0.6305] | 0.6719 | 0.4219 |
| c-midrange | 128 | 0.5391 | [0.4528, 0.623] | 0.625 | 0.4531 |
| s26 | 128 | 0.5391 | [0.4528, 0.623] | 0.5781 | 0.5 |
| s24 | 128 | 0.5312 | [0.4452, 0.6155] | 0.4688 | 0.5938 |
| s22 | 128 | 0.5234 | [0.4375, 0.608] | 0.6094 | 0.4375 |
| s34 | 128 | 0.5234 | [0.4375, 0.608] | 0.5469 | 0.5 |
| s03 | 128 | 0.5156 | [0.4299, 0.6005] | 0.5625 | 0.4688 |
| s40 | 128 | 0.5156 | [0.4299, 0.6005] | 0.5781 | 0.4531 |
| s10 | 128 | 0.5078 | [0.4222, 0.5929] | 0.5 | 0.5156 |
| s36 | 128 | 0.5078 | [0.4222, 0.5929] | 0.5938 | 0.4219 |
| s07 | 128 | 0.5 | [0.4147, 0.5853] | 0.6094 | 0.3906 |
| s23 | 128 | 0.4922 | [0.4071, 0.5778] | 0.5312 | 0.4531 |
| s33 | 128 | 0.4922 | [0.4071, 0.5778] | 0.5625 | 0.4219 |
| m01 | 128 | 0.4844 | [0.3995, 0.5701] | 0.5312 | 0.4375 |
| s17 | 128 | 0.4844 | [0.3995, 0.5701] | 0.4219 | 0.5469 |
| s21 | 128 | 0.4844 | [0.3995, 0.5701] | 0.5469 | 0.4219 |
| s29 | 128 | 0.4844 | [0.3995, 0.5701] | 0.5469 | 0.4219 |
| s54 | 128 | 0.4844 | [0.3995, 0.5701] | 0.5469 | 0.4219 |
| s55 | 128 | 0.4844 | [0.3995, 0.5701] | 0.4844 | 0.4844 |
| s56 | 128 | 0.4844 | [0.3995, 0.5701] | 0.4844 | 0.4844 |
| s08 | 128 | 0.4766 | [0.392, 0.5625] | 0.4531 | 0.5 |
| s38 | 128 | 0.4766 | [0.392, 0.5625] | 0.4844 | 0.4688 |
| s42 | 128 | 0.4766 | [0.392, 0.5625] | 0.5625 | 0.3906 |
| m02 | 128 | 0.4688 | [0.3845, 0.5548] | 0.5469 | 0.3906 |
| s04 | 128 | 0.4609 | [0.377, 0.5472] | 0.5 | 0.4219 |
| s37 | 128 | 0.4531 | [0.3695, 0.5395] | 0.4844 | 0.4219 |
| s41 | 128 | 0.4531 | [0.3695, 0.5395] | 0.5312 | 0.375 |
| s52 | 128 | 0.4453 | [0.3621, 0.5318] | 0.4688 | 0.4219 |
| f01 | 128 | 0.4375 | [0.3546, 0.524] | 0.5156 | 0.3594 |
| s06 | 128 | 0.4375 | [0.3546, 0.524] | 0.4062 | 0.4688 |
| s12 | 128 | 0.4375 | [0.3546, 0.524] | 0.3906 | 0.4844 |
| s13 | 128 | 0.4375 | [0.3546, 0.524] | 0.5 | 0.375 |
| s28 | 128 | 0.4375 | [0.3546, 0.524] | 0.5 | 0.375 |
| s09 | 128 | 0.4297 | [0.3472, 0.5163] | 0.5 | 0.3594 |
| s15 | 128 | 0.4297 | [0.3472, 0.5163] | 0.5156 | 0.3438 |
| s27 | 128 | 0.4297 | [0.3472, 0.5163] | 0.5 | 0.3594 |
| c-aggro | 128 | 0.4141 | [0.3325, 0.5007] | 0.3438 | 0.4844 |
| m03 | 128 | 0.4141 | [0.3325, 0.5007] | 0.5156 | 0.3125 |
| m04 | 128 | 0.4141 | [0.3325, 0.5007] | 0.4219 | 0.4062 |
| s39 | 128 | 0.4141 | [0.3325, 0.5007] | 0.3906 | 0.4375 |
| s25 | 128 | 0.3984 | [0.3178, 0.485] | 0.4844 | 0.3125 |
| s31 | 128 | 0.3984 | [0.3178, 0.485] | 0.4375 | 0.3594 |
| s43 | 128 | 0.3906 | [0.3105, 0.4772] | 0.4219 | 0.3594 |
| s50 | 128 | 0.3906 | [0.3105, 0.4772] | 0.3906 | 0.3906 |
| s51 | 128 | 0.3906 | [0.3105, 0.4772] | 0.4062 | 0.375 |
| s11 | 128 | 0.3828 | [0.3032, 0.4693] | 0.4219 | 0.3438 |
| s48 | 128 | 0.3828 | [0.3032, 0.4693] | 0.4062 | 0.3594 |
| s14 | 128 | 0.3672 | [0.2887, 0.4534] | 0.4531 | 0.2812 |
| s47 | 128 | 0.3672 | [0.2887, 0.4534] | 0.4219 | 0.3125 |
| s16 | 128 | 0.3594 | [0.2815, 0.4455] | 0.4688 | 0.25 |
| s45 | 128 | 0.3594 | [0.2815, 0.4455] | 0.4688 | 0.25 |
| s30 | 128 | 0.3516 | [0.2743, 0.4375] | 0.375 | 0.3281 |
| s44 | 128 | 0.3359 | [0.26, 0.4215] | 0.3594 | 0.3125 |
| s46 | 128 | 0.2969 | [0.2246, 0.381] | 0.3438 | 0.25 |
| s32 | 128 | 0.2891 | [0.2176, 0.3728] | 0.3594 | 0.2188 |
| f02 | 128 | 0.2422 | [0.1762, 0.3232] | 0.25 | 0.2344 |

## Win rate by deck (both seats; the search decks together as one row)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.787 |
| meta-portal-evo | 0.713 |
| meta-rune-test-subject | 0.709 |
| meta-abyss-midrange | 0.637 |
| meta-portal-af | 0.627 |
| meta-haven-kukishiro | 0.623 |
| meta-haven-amulet | 0.560 |
| meta-forest-combo | 0.545 |
| meta-sword-rally | 0.535 |
| meta-sword-loot | 0.514 |
| meta-portal-cutthroat | 0.490 |
| abyss-search | 0.456 |
| meta-dragon-ramp | 0.445 |
| meta-rune-spell | 0.438 |
| meta-abyss-aggro | 0.422 |
| meta-dragon-aggro | 0.330 |
| meta-rune-crystal | 0.326 |
