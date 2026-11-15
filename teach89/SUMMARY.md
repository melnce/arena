# teach89: teacher data from the served search with the reply keys (runbook 89)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run89/leaf89q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 8901
- games: 4096 of 4096 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 3.07 h in 1 segment(s), 1333.2 games/h
- replay check (every game replayed from its seed and actions): 4096 of 4096 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 285728 of 285728 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 1565293 of 1565293 (100.00 %)
- decisions: 374836; with two or more candidates: 290592
- paths: {'search': 288827, 'single_legal': 57388, 'take_kill': 16589, 'mulligan': 8192, 'take_kill_roll': 1999, 'holdback_trade': 1174, 'holdback_resample': 591, 'consensus_lethal': 76}
- candidates per decision (2+): {'n': 290592, 'mean': 5.387, 'median': 5, 'p90': 10, 'p99': 16, 'max': 35}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 89534 of 1489808 (6.01 %)
- gap between the best and second-best aggregate: {'n': 290592, 'mean': 4.861, 'median': 1.68, 'p90': 12.479, 'p99': 46.02, 'max': 160.0}
- hbk ran 2955 times, flipped 591; hbcheck ran 2364 times; lostrank ran 3099 times (played another move than the first best-scored one: 2382; left out of the search check)
- world line ends over all candidates (12522344; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 4228973, 'O': 3826147, 'C': 1954385, 'L': 1051598, 't': 715158, 'D': 704199, 'T': 24298, 'c': 17326, 'd': 260}
- seconds per decision, one core: {'n': 374836, 'mean': 0.823, 'median': 0.2, 'p90': 2.502, 'p99': 5.863, 'max': 30.409}
- seconds per search decision, one core: {'n': 288827, 'mean': 1.056, 'median': 0.477, 'p90': 2.883, 'p99': 6.274, 'max': 30.409}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 70724, 'mean': 4.341, 'median': 1.494, 'p90': 12.668, 'p99': 30.895, 'max': 109.587}
- decisions per game: {'n': 4096, 'mean': 91.513, 'median': 88, 'p90': 128, 'p99': 168, 'max': 219}; turns: {'n': 4096, 'mean': 8.916, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.5488 over 4096 decisive games
- data: 90.0 MB in 4096 files; archive teach89-games.jsonl.gz 87.7 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.644 |
| meta-haven-evo | 0.629 |
| meta-sword-rally | 0.572 |
| meta-portal-af | 0.570 |
| meta-portal-evo | 0.557 |
| meta-abyss-midrange | 0.529 |
| meta-portal-cutthroat | 0.523 |
| meta-sword-loot | 0.519 |
| meta-dragon-ramp | 0.502 |
| meta-haven-kukishiro | 0.461 |
| meta-forest-combo | 0.449 |
| meta-dragon-aggro | 0.440 |
| meta-abyss-aggro | 0.432 |
| meta-rune-spell | 0.432 |
| meta-haven-amulet | 0.408 |
| meta-rune-crystal | 0.332 |
