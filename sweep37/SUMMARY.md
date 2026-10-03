# Sweep `sweep37`

- tag: `sweep37`
- baseline: `h0`
- seed: 1
- engine: `1c551eba7078b79d5207b9862336bdf7bec9478b`
- wall: 7764.1s
- stages: screen 1728.6s, final 6035.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/sweep37/leaf-all4-race-e2-l2x10-s1.json` | 0.510 [0.467, 0.553] | 512 | 1.42 / 1.37 | finalist |
| 2 | `h0:net=results/sweep37/leaf-all4-race-e2-l2x10-s2.json` | 0.504 [0.461, 0.547] | 512 | 1.60 / 1.37 | finalist |
| 3 | `h0:net=results/sweep37/leaf-all4-e2-l2x10-s2.json` | 0.488 [0.445, 0.532] | 512 | 1.43 / 1.37 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep37/leaf-all4-race-e2-l2x10-s1.json` | 0.498 [0.476, 0.520] | 0.510 [0.479, 0.540] | 0.502 [0.484, 0.520] |
| 2 | `h0:net=results/sweep37/leaf-all4-race-e2-l2x10-s2.json` | 0.484 [0.463, 0.506] | 0.469 [0.438, 0.499] | 0.479 [0.462, 0.497] |
| 3 | `h0:net=results/sweep37/leaf-all4-e2-l2x10-s2.json` | 0.467 [0.445, 0.488] | 0.481 [0.451, 0.512] | 0.472 [0.454, 0.489] |

verdict: h0:net=results/sweep37/leaf-all4-race-e2-l2x10-s1.json coin flip — main 0.498 [0.476, 0.520], reverse 0.510 [0.479, 0.540], pooled 0.502 [0.484, 0.520] (not gated)
verdict: h0:net=results/sweep37/leaf-all4-race-e2-l2x10-s2.json unclear (main, reverse) — main 0.484 [0.463, 0.506], reverse 0.469 [0.438, 0.499], pooled 0.479 [0.462, 0.497] (not gated)
verdict: h0:net=results/sweep37/leaf-all4-e2-l2x10-s2.json worse — main 0.467 [0.445, 0.488], reverse 0.481 [0.451, 0.512], pooled 0.472 [0.454, 0.489] (not gated)

best: h0:net=results/sweep37/leaf-all4-race-e2-l2x10-s1.json (coin flip)
