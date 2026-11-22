# teach97: teacher data from the served search with the reply keys (runbook 97)

- verdict: DATA OK
- teacher: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256`
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`, seed 9701
- games: 16384 of 16384 planned (0 stopped at 1000 decisions, 0 raised errors, 0 unreadable files moved aside); 12.74 h in 13 segment(s), 1285.7 games/h
- replay check (every game replayed from its seed and actions): 16384 of 16384 (100.00 %)
- check: search decisions that played a candidate with the best aggregate: 1158963 of 1158963 (100.00 %)
- check: candidates whose aggregate is the mean of their worlds: 6345481 of 6345481 (100.00 %)
- decisions: 1517601; with two or more candidates: 1178322
- paths: {'search': 1171325, 'single_legal': 231819, 'take_kill': 66651, 'mulligan': 32768, 'take_kill_roll': 7761, 'holdback_trade': 4770, 'holdback_resample': 2227, 'consensus_lethal': 280}
- candidates per decision (2+): {'n': 1178322, 'mean': 5.385, 'median': 5, 'p90': 10, 'p99': 16, 'max': 39}
- duplicate candidates (same values, line ends and lengths in every world as an earlier candidate): 369141 of 6039550 (6.11 %)
- gap between the best and second-best aggregate: {'n': 1178322, 'mean': 4.953, 'median': 1.708, 'p90': 12.752, 'p99': 46.501, 'max': 160.0}
- hbk ran 11961 times, flipped 2227; hbcheck ran 9734 times; lostrank ran 12362 times (played another move than the first best-scored one: 9447; left out of the search check)
- world line ends over all candidates (50763848; O = opp_open, N = opp_next, W = own_next, X = next_leaf, ? = unknown): {'o': 17135249, 'O': 15518167, 'C': 7901926, 'L': 4251596, 'D': 2912479, 't': 2861447, 'T': 102966, 'c': 79058, 'd': 960}
- seconds per decision, one core: {'n': 1517601, 'mean': 0.836, 'median': 0.205, 'p90': 2.538, 'p99': 5.955, 'max': 32.946}
- seconds per search decision, one core: {'n': 1171325, 'mean': 1.071, 'median': 0.486, 'p90': 2.919, 'p99': 6.356, 'max': 26.236}
- seconds per turn (all of one player's decisions in a turn), one core: {'n': 285380, 'mean': 4.424, 'median': 1.545, 'p90': 12.825, 'p99': 31.737, 'max': 98.618}
- decisions per game: {'n': 16384, 'mean': 92.627, 'median': 90, 'p90': 129, 'p99': 172, 'max': 248}; turns: {'n': 16384, 'mean': 8.995, 'median': 9, 'p90': 12, 'p99': 14, 'max': 19}
- first-seat win rate: 0.553 over 16384 decisive games
- data: 364.2 MB in 16384 files; archive teach97-games.jsonl.gz 354.9 MB

## Win rate by deck (both seats, mirrors included)

| deck | win rate |
|---|---|
| meta-rune-test-subject | 0.652 |
| meta-haven-evo | 0.638 |
| meta-portal-evo | 0.547 |
| meta-portal-af | 0.545 |
| meta-sword-rally | 0.545 |
| meta-abyss-midrange | 0.525 |
| meta-portal-cutthroat | 0.524 |
| meta-sword-loot | 0.518 |
| meta-dragon-ramp | 0.497 |
| meta-forest-combo | 0.477 |
| meta-haven-kukishiro | 0.473 |
| meta-haven-amulet | 0.441 |
| meta-abyss-aggro | 0.440 |
| meta-dragon-aggro | 0.422 |
| meta-rune-spell | 0.408 |
| meta-rune-crystal | 0.347 |
