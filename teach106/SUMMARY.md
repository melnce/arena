# teach106: Forestcraft and Portalcraft exploration games against the 16 meta lists, round 1 (runbook 106)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run106/leaf106rq.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 10601
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 2.63 h in 1 segment(s), 1559.8 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 259467 of 259467 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1402631 of 1402631 (100.00 %)
- decisions: 345829; with two or more candidates: 264682
- paths: {'search': 262670, 'single_legal': 54226, 'take_kill': 17421, 'mulligan': 8192, 'holdback_trade': 1377, 'take_kill_roll': 1251, 'holdback_resample': 635, 'consensus_lethal': 57}
- candidates per decision (2+): {'n': 264682, 'mean': 5.299, 'median': 5, 'p90': 9, 'p99': 16, 'max': 34}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 66723 of 1337131 (4.99 %)
- gap between the best and second-best aggregate: {'n': 264682, 'mean': 3.188, 'median': 0.699, 'p90': 8.128, 'p99': 34.922, 'max': 160.0}
- hbk ran 3327 times, flipped 635; hbcheck ran 2692 times; lostrank ran 3203 times (played another move than the first best-scored one: 2539; left out of the search check)
- world line ends over all candidates (11221048; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 3836419, 'O': 3686695, 'C': 1615944, 'L': 829868, 't': 671091, 'D': 552263, 'T': 18206, 'c': 10363, 'd': 199}
- seconds per decision, one core: {'n': 345829, 'mean': 0.76, 'median': 0.188, 'p90': 2.328, 'p99': 5.585, 'max': 16.833}
- seconds per search decision, one core: {'n': 262670, 'mean': 0.987, 'median': 0.422, 'p90': 2.729, 'p99': 6.007, 'max': 16.833}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 68906, 'mean': 3.793, 'median': 1.281, 'p90': 10.951, 'p99': 28.148, 'max': 82.604}
- decisions per game: {'n': 4096, 'mean': 84.431, 'median': 80, 'p90': 119, 'p99': 163, 'max': 220}; turns: {'n': 4096, 'mean': 8.68, 'median': 8, 'p90': 11, 'p99': 14, 'max': 21}
- first-seat win rate: 0.5276 over 4096 decisive games
- data: 81.6 MB in 4096 files; archive teach106-games.jsonl.gz 79.5 MB

## Exploration

- the exploring seat: win rate 0.1331 over 4096 games; forestcraft 0.1445 (2048), portalcraft 0.1216 (2048); rebuilt decks 0.1978, scratch decks 0.0684; going first 0.1606, second 0.1055
- exploration-pool cards played in a deck: 179 of 179; decks per card: {'n': 179, 'mean': 569.436, 'median': 539, 'p90': 932, 'p99': 1531, 'max': 1830}; fewest: [(253, '10813310'), (266, '10071310'), (267, '10411120'), (268, '10812310'), (274, '10071130')]
- cards in the exploring decks of the games that raised errors (most often first): none

| opponent list | the exploring seat's win rate (games) | forestcraft | portalcraft |
|---|---|---|---|
| meta-abyss-aggro | 0.3672 (256) | 0.4375 | 0.2969 |
| meta-abyss-midrange | 0.082 (256) | 0.1016 | 0.0625 |
| meta-dragon-aggro | 0.3047 (256) | 0.3516 | 0.2578 |
| meta-dragon-ramp | 0.168 (256) | 0.2031 | 0.1328 |
| meta-forest-combo | 0.1094 (256) | 0.1016 | 0.1172 |
| meta-haven-amulet | 0.1641 (256) | 0.1719 | 0.1562 |
| meta-haven-evo | 0.0625 (256) | 0.0859 | 0.0391 |
| meta-haven-kukishiro | 0.082 (256) | 0.0625 | 0.1016 |
| meta-portal-af | 0.0625 (256) | 0.0547 | 0.0703 |
| meta-portal-cutthroat | 0.0977 (256) | 0.0938 | 0.1016 |
| meta-portal-evo | 0.0469 (256) | 0.0625 | 0.0312 |
| meta-rune-crystal | 0.1953 (256) | 0.1875 | 0.2031 |
| meta-rune-spell | 0.1289 (256) | 0.1719 | 0.0859 |
| meta-rune-test-subject | 0.0586 (256) | 0.0391 | 0.0781 |
| meta-sword-loot | 0.1133 (256) | 0.1094 | 0.1172 |
| meta-sword-rally | 0.0859 (256) | 0.0781 | 0.0938 |

## Win rate by deck (both seats; the exploration decks together as one row)

| deck | win rate |
|---|---|
| meta-portal-evo | 0.953 |
| meta-rune-test-subject | 0.941 |
| meta-haven-evo | 0.938 |
| meta-portal-af | 0.938 |
| meta-abyss-midrange | 0.918 |
| meta-haven-kukishiro | 0.918 |
| meta-sword-rally | 0.914 |
| meta-portal-cutthroat | 0.902 |
| meta-forest-combo | 0.891 |
| meta-sword-loot | 0.887 |
| meta-rune-spell | 0.871 |
| meta-haven-amulet | 0.836 |
| meta-dragon-ramp | 0.832 |
| meta-rune-crystal | 0.805 |
| meta-dragon-aggro | 0.695 |
| meta-abyss-aggro | 0.633 |
| fp-explore | 0.133 |
