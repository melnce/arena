# teach99: Abysscraft exploration games against the 16 meta lists (runbook 99)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9901
- games: 15360 of 15360 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 7.72 h in 12 segment(s), 1989.9 games/h
- replay check (every game replayed from its seed and actions): 15360 of 15360 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 890724 of 890724 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 4795461 of 4795461 (100.00 %)
- decisions: 1205451; with two or more candidates: 911285
- paths: {'search': 905862, 'single_legal': 199879, 'take_kill': 58414, 'mulligan': 30720, 'take_kill_roll': 4914, 'holdback_trade': 3637, 'holdback_resample': 1786, 'consensus_lethal': 239}
- candidates per decision (2+): {'n': 911285, 'mean': 5.262, 'median': 4, 'p90': 9, 'p99': 16, 'max': 37}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 233347 of 4533164 (5.15 %)
- gap between the best and second-best aggregate: {'n': 911285, 'mean': 4.452, 'median': 1.329, 'p90': 11.652, 'p99': 43.11, 'max': 160.0}
- hbk ran 9628 times, flipped 1786; hbcheck ran 7842 times; lostrank ran 15138 times (played another move than the first best-scored one: 11935; left out of the search check)
- world line ends over all candidates (38363688; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 13698247, 'O': 11737757, 'C': 5033620, 'L': 3210420, 't': 2734927, 'D': 1847961, 'T': 80263, 'c': 20259, 'd': 234}
- seconds per decision, one core: {'n': 1205451, 'mean': 0.633, 'median': 0.143, 'p90': 1.941, 'p99': 5.241, 'max': 22.231}
- seconds per search decision, one core: {'n': 905862, 'mean': 0.83, 'median': 0.302, 'p90': 2.327, 'p99': 5.663, 'max': 22.231}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 247220, 'mean': 3.065, 'median': 0.863, 'p90': 9.054, 'p99': 24.71, 'max': 83.662}
- decisions per game: {'n': 15360, 'mean': 78.48, 'median': 74, 'p90': 112, 'p99': 153, 'max': 224}; turns: {'n': 15360, 'mean': 8.313, 'median': 8, 'p90': 11, 'p99': 14, 'max': 21}
- first-seat win rate: 0.5231 over 15360 decisive games
- data: 282.2 MB in 15360 files; archive teach99-games.jsonl.gz 274.1 MB

## Exploration

- the exploring seat (abysscraft): win rate 0.159 over 15360 games; rebuilt decks 0.2398, scratch decks 0.0783; going first 0.1822, second 0.1359
- exploration-pool cards played in a deck: 113 of 113; decks per card: {'n': 113, 'mean': 3276.779, 'median': 2519, 'p90': 5210, 'p99': 9714, 'max': 10756}; fewest: [(2148, '10452110'), (2151, '10752120'), (2153, '10404110'), (2158, '10453110'), (2176, '10701310')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the exploring seat's win rate (games) |
|---|---|
| meta-abyss-aggro | 0.2531 (960) |
| meta-abyss-midrange | 0.1177 (960) |
| meta-dragon-aggro | 0.2281 (960) |
| meta-dragon-ramp | 0.2146 (960) |
| meta-forest-combo | 0.1542 (960) |
| meta-haven-amulet | 0.25 (960) |
| meta-haven-evo | 0.1135 (960) |
| meta-haven-kukishiro | 0.15 (960) |
| meta-portal-af | 0.0833 (960) |
| meta-portal-cutthroat | 0.125 (960) |
| meta-portal-evo | 0.0917 (960) |
| meta-rune-crystal | 0.2448 (960) |
| meta-rune-spell | 0.1833 (960) |
| meta-rune-test-subject | 0.0958 (960) |
| meta-sword-loot | 0.1396 (960) |
| meta-sword-rally | 0.1 (960) |

## Win rate by deck (both seats; the exploration decks together as one row)

| deck | win rate |
|---|---|
| meta-portal-af | 0.917 |
| meta-portal-evo | 0.908 |
| meta-rune-test-subject | 0.904 |
| meta-sword-rally | 0.900 |
| meta-haven-evo | 0.886 |
| meta-abyss-midrange | 0.882 |
| meta-portal-cutthroat | 0.875 |
| meta-sword-loot | 0.860 |
| meta-haven-kukishiro | 0.850 |
| meta-forest-combo | 0.846 |
| meta-rune-spell | 0.817 |
| meta-dragon-ramp | 0.785 |
| meta-dragon-aggro | 0.772 |
| meta-rune-crystal | 0.755 |
| meta-haven-amulet | 0.750 |
| meta-abyss-aggro | 0.747 |
| abyss-explore | 0.159 |
