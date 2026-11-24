# teach100: Abysscraft exploration games against the 16 meta lists, round 1 (runbook 100)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run100/leaf100q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 10001
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 2.11 h in 1 segment(s), 1945.7 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 239955 of 239955 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1294399 of 1294399 (100.00 %)
- decisions: 325109; with two or more candidates: 245407
- paths: {'search': 243898, 'single_legal': 54358, 'take_kill': 15659, 'mulligan': 8192, 'take_kill_roll': 1447, 'holdback_trade': 1022, 'holdback_resample': 487, 'consensus_lethal': 46}
- candidates per decision (2+): {'n': 245407, 'mean': 5.274, 'median': 4, 'p90': 10, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 62074 of 1223084 (5.08 %)
- gap between the best and second-best aggregate: {'n': 245407, 'mean': 3.693, 'median': 0.811, 'p90': 9.577, 'p99': 40.76, 'max': 160.0}
- hbk ran 2535 times, flipped 487; hbcheck ran 2048 times; lostrank ran 3943 times (played another move than the first best-scored one: 3067; left out of the search check)
- world line ends over all candidates (10355192; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 3627650, 'O': 3210391, 'C': 1387672, 'L': 882411, 't': 723197, 'D': 495378, 'T': 21005, 'c': 7447, 'd': 41}
- seconds per decision, one core: {'n': 325109, 'mean': 0.649, 'median': 0.148, 'p90': 1.994, 'p99': 5.278, 'max': 24.015}
- seconds per search decision, one core: {'n': 243898, 'mean': 0.852, 'median': 0.315, 'p90': 2.4, 'p99': 5.697, 'max': 16.843}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 66973, 'mean': 3.127, 'median': 0.89, 'p90': 9.187, 'p99': 24.955, 'max': 76.004}
- decisions per game: {'n': 4096, 'mean': 79.372, 'median': 75, 'p90': 113, 'p99': 154, 'max': 208}; turns: {'n': 4096, 'mean': 8.439, 'median': 8, 'p90': 11, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5225 over 4096 decisive games
- data: 75.1 MB in 4096 files; archive teach100-games.jsonl.gz 73.0 MB

## Exploration

- the exploring seat (abysscraft): win rate 0.1567 over 4096 games; rebuilt decks 0.2383, scratch decks 0.0752; going first 0.1792, second 0.1343
- exploration-pool cards played in a deck: 113 of 113; decks per card: {'n': 113, 'mean': 873.681, 'median': 668, 'p90': 1387, 'p99': 2633, 'max': 2837}; fewest: [(563, '10603110'), (567, '10704110'), (568, '10404110'), (568, '10453110'), (573, '10551120')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the exploring seat's win rate (games) |
|---|---|
| meta-abyss-aggro | 0.2305 (256) |
| meta-abyss-midrange | 0.1328 (256) |
| meta-dragon-aggro | 0.25 (256) |
| meta-dragon-ramp | 0.2227 (256) |
| meta-forest-combo | 0.1484 (256) |
| meta-haven-amulet | 0.2148 (256) |
| meta-haven-evo | 0.0703 (256) |
| meta-haven-kukishiro | 0.1289 (256) |
| meta-portal-af | 0.1094 (256) |
| meta-portal-cutthroat | 0.125 (256) |
| meta-portal-evo | 0.1016 (256) |
| meta-rune-crystal | 0.2305 (256) |
| meta-rune-spell | 0.1758 (256) |
| meta-rune-test-subject | 0.1094 (256) |
| meta-sword-loot | 0.1367 (256) |
| meta-sword-rally | 0.1211 (256) |

## Win rate by deck (both seats; the exploration decks together as one row)

| deck | win rate |
|---|---|
| meta-haven-evo | 0.930 |
| meta-portal-evo | 0.898 |
| meta-portal-af | 0.891 |
| meta-rune-test-subject | 0.891 |
| meta-sword-rally | 0.879 |
| meta-portal-cutthroat | 0.875 |
| meta-haven-kukishiro | 0.871 |
| meta-abyss-midrange | 0.867 |
| meta-sword-loot | 0.863 |
| meta-forest-combo | 0.852 |
| meta-rune-spell | 0.824 |
| meta-haven-amulet | 0.785 |
| meta-dragon-ramp | 0.777 |
| meta-abyss-aggro | 0.769 |
| meta-rune-crystal | 0.769 |
| meta-dragon-aggro | 0.750 |
| abyss-explore | 0.157 |
