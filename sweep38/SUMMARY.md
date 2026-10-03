# Sweep `sweep38`

- tag: `sweep38`
- baseline: `h0`
- seed: 1
- engine: `1c551eba7078b79d5207b9862336bdf7bec9478b`
- wall: 6944.9s
- stages: screen 1595.7s, final 5349.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:net=results/sweep38/soup5-race-e2-l2x10.json` | 0.539 [0.496, 0.582] | 512 | 1.48 / 1.52 | finalist |
| 1 | `h0:net=results/sweep38/soup5-plain-e2-l2x10.json` | 0.518 [0.474, 0.561] | 512 | 1.59 / 1.52 | finalist |
| 3 | `h0:net=results/sweep38/leaf-all4-e2-l2x10-s3.json` | 0.486 [0.443, 0.530] | 512 | 1.58 / 1.52 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep38/soup5-plain-e2-l2x10.json` | 0.506 [0.484, 0.527] | 0.499 [0.468, 0.530] | 0.504 [0.486, 0.521] |
| 2 | `h0:net=results/sweep38/soup5-race-e2-l2x10.json` | 0.504 [0.482, 0.526] | 0.499 [0.468, 0.530] | 0.502 [0.485, 0.520] |
| 3 | `h0:net=results/sweep38/leaf-all4-e2-l2x10-s3.json` | 0.488 [0.467, 0.510] | 0.494 [0.464, 0.525] | 0.490 [0.473, 0.508] |

verdict: h0:net=results/sweep38/soup5-plain-e2-l2x10.json coin flip — main 0.506 [0.484, 0.527], reverse 0.499 [0.468, 0.530], pooled 0.504 [0.486, 0.521] (not gated)
verdict: h0:net=results/sweep38/soup5-race-e2-l2x10.json coin flip — main 0.504 [0.482, 0.526], reverse 0.499 [0.468, 0.530], pooled 0.502 [0.485, 0.520] (not gated)
verdict: h0:net=results/sweep38/leaf-all4-e2-l2x10-s3.json coin flip — main 0.488 [0.467, 0.510], reverse 0.494 [0.464, 0.525], pooled 0.490 [0.473, 0.508] (not gated)

best: h0:net=results/sweep38/soup5-plain-e2-l2x10.json (coin flip)
