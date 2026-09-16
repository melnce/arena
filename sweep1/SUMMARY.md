# Sweep `sweep1`

- tag: `sweep1`
- baseline: `h0`
- seed: 1
- engine: `f8fd977668dd82662de98fdfccc6c5a927b42c54`
- wall: 9907.5s
- stages: screen 4984.4s, final 4923.1s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=4000` | 0.570 [0.540, 0.600] | 1024 | 1.33 / 2.10 | finalist |
| 2 | `h0:depth=4,beam=8` | 0.533 [0.503, 0.564] | 1024 | 2.37 / 2.10 | finalist |
| 3 | `h0:depth=8,beam=3` | 0.509 [0.478, 0.539] | 1024 | 2.15 / 2.10 | not in top 2 |
| 4 | `h0:k=8` | 0.509 [0.478, 0.539] | 1024 | 1.78 / 2.10 | not in top 2 |
| 8 | `h0:osteps=6` | 0.506 [0.475, 0.536] | 1024 | 2.42 / 2.10 | not in top 2 |
| 5 | `h0:k=2` | 0.501 [0.470, 0.532] | 1024 | 2.70 / 2.10 | not in top 2 |
| 7 | `h0:beam=6` | 0.495 [0.465, 0.526] | 1024 | 2.55 / 2.10 | not in top 2 |
| 6 | `h0:odepth=1` | 0.400 [0.371, 0.431] | 1024 | 1.54 / 2.10 | skipped: interval high 0.43 < 0.50 |

## final

| index | spec | main | reverse |
|---|---|---:|---:|
| 1 | `h0:nodes=4000` | 0.544 [0.529, 0.560] | 0.540 [0.518, 0.561] |
| 2 | `h0:depth=4,beam=8` | 0.529 [0.514, 0.544] | 0.521 [0.500, 0.543] |

verdict: h0:nodes=4000 better — main 0.544 [0.529, 0.560], reverse 0.540 [0.518, 0.561]
verdict: h0:depth=4,beam=8 unclear (reverse) — main 0.529 [0.514, 0.544], reverse 0.521 [0.500, 0.543]

best: h0:nodes=4000 (better)
