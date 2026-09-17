# Sweep `sweep2`

- tag: `sweep2`
- baseline: `h0`
- seed: 2
- engine: `f8fd977668dd82662de98fdfccc6c5a927b42c54`
- wall: 10806.8s
- stages: screen 4691.0s, final 6115.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 4 | `h0:nodes=6000` | 0.553 [0.522, 0.583] | 1024 | 1.32 / 2.72 | finalist |
| 1 | `h0:depth=4,beam=8,nodes=4000` | 0.544 [0.513, 0.574] | 1024 | 1.69 / 2.72 | finalist |
| 3 | `h0:nodes=3000` | 0.532 [0.502, 0.563] | 1024 | 1.28 / 2.72 | not in top 2 |
| 8 | `h0:depth=4,beam=8,nodes=3000` | 0.529 [0.499, 0.560] | 1024 | 2.30 / 2.72 | not in top 2 |
| 6 | `h0:depth=3,beam=12` | 0.521 [0.490, 0.551] | 1024 | 3.28 / 2.72 | not in top 2 |
| 2 | `h0:depth=4,beam=8` | 0.511 [0.480, 0.541] | 1024 | 2.50 / 2.72 | not in top 2 |
| 5 | `h0:depth=4,beam=12` | 0.510 [0.479, 0.540] | 1024 | 2.47 / 2.72 | not in top 2 |
| 7 | `h0:depth=5,beam=6` | 0.487 [0.457, 0.518] | 1024 | 2.60 / 2.72 | not in top 2 |

## final

| index | spec | main | reverse |
|---|---|---:|---:|
| 1 | `h0:depth=4,beam=8,nodes=4000` | 0.547 [0.532, 0.563] | 0.550 [0.528, 0.571] |
| 4 | `h0:nodes=6000` | 0.562 [0.547, 0.577] | 0.553 [0.531, 0.574] |

verdict: h0:depth=4,beam=8,nodes=4000 better — main 0.547 [0.532, 0.563], reverse 0.550 [0.528, 0.571]
verdict: h0:nodes=6000 better — main 0.562 [0.547, 0.577], reverse 0.553 [0.531, 0.574]

best: h0:nodes=6000 (better)
