# teach100: Abysscraft exploration games against the 16 meta lists, round 1 (runbook 100)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run100/leaf100q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 10001
- games: 5120 of 5120 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 2.66 h in 2 segment(s), 1923.0 games/h
- replay check (every game replayed from its seed and actions): 5120 of 5120 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 300317 of 300317 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1621628 of 1621628 (100.00 %)
- decisions: 406949; with two or more candidates: 307197
- paths: {'search': 305285, 'single_legal': 67891, 'take_kill': 19871, 'mulligan': 10240, 'take_kill_roll': 1692, 'holdback_trade': 1296, 'holdback_resample': 616, 'consensus_lethal': 58}
- candidates per decision (2+): {'n': 307197, 'mean': 5.279, 'median': 4, 'p90': 10, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 77982 of 1532117 (5.09 %)
- gap between the best and second-best aggregate: {'n': 307197, 'mean': 3.711, 'median': 0.816, 'p90': 9.622, 'p99': 41.09, 'max': 160.0}
- hbk ran 3210 times, flipped 616; hbcheck ran 2594 times; lostrank ran 4968 times (played another move than the first best-scored one: 3851; left out of the search check)
- world line ends over all candidates (12973024; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4543282, 'O': 4020894, 'C': 1739355, 'L': 1106635, 't': 906042, 'D': 622062, 'T': 25817, 'c': 8869, 'd': 68}
- seconds per decision, one core: {'n': 406949, 'mean': 0.653, 'median': 0.149, 'p90': 2.005, 'p99': 5.308, 'max': 24.015}
- seconds per search decision, one core: {'n': 305285, 'mean': 0.857, 'median': 0.318, 'p90': 2.41, 'p99': 5.73, 'max': 16.843}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 83677, 'mean': 3.152, 'median': 0.898, 'p90': 9.265, 'p99': 25.07, 'max': 76.004}
- decisions per game: {'n': 5120, 'mean': 79.482, 'median': 75, 'p90': 114, 'p99': 152, 'max': 208}; turns: {'n': 5120, 'mean': 8.435, 'median': 8, 'p90': 11, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5221 over 5120 decisive games
- data: 94.0 MB in 5120 files; archive teach100-games.jsonl.gz 91.4 MB

## Exploration

- the exploring seat (abysscraft): win rate 0.1572 over 5120 games; rebuilt decks 0.2379, scratch decks 0.0766; going first 0.1793, second 0.1352
- exploration-pool cards played in a deck: 113 of 113; decks per card: {'n': 113, 'mean': 1092.814, 'median': 840, 'p90': 1721, 'p99': 3277, 'max': 3561}; fewest: [(697, '10404110'), (706, '10603110'), (708, '10453110'), (717, '10551120'), (721, '10901110')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the exploring seat's win rate (games) |
|---|---|
| meta-abyss-aggro | 0.2281 (320) |
| meta-abyss-midrange | 0.1313 (320) |
| meta-dragon-aggro | 0.2625 (320) |
| meta-dragon-ramp | 0.2219 (320) |
| meta-forest-combo | 0.1437 (320) |
| meta-haven-amulet | 0.2094 (320) |
| meta-haven-evo | 0.075 (320) |
| meta-haven-kukishiro | 0.1344 (320) |
| meta-portal-af | 0.1 (320) |
| meta-portal-cutthroat | 0.1344 (320) |
| meta-portal-evo | 0.0969 (320) |
| meta-rune-crystal | 0.2281 (320) |
| meta-rune-spell | 0.1938 (320) |
| meta-rune-test-subject | 0.1031 (320) |
| meta-sword-loot | 0.1437 (320) |
| meta-sword-rally | 0.1094 (320) |

## Win rate by deck (both seats; the exploration decks together as one row)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.925 |
| meta-portal-evo | 0.903 |
| meta-portal-af | 0.900 |
| meta-rune-test-subject | 0.897 |
| meta-sword-rally | 0.891 |
| meta-abyss-midrange | 0.869 |
| meta-haven-kukishiro | 0.866 |
| meta-portal-cutthroat | 0.866 |
| meta-forest-combo | 0.856 |
| meta-sword-loot | 0.856 |
| meta-rune-spell | 0.806 |
| meta-haven-amulet | 0.791 |
| meta-dragon-ramp | 0.778 |
| meta-abyss-aggro | 0.772 |
| meta-rune-crystal | 0.772 |
| meta-dragon-aggro | 0.738 |
| abyss-explore | 0.157 |
