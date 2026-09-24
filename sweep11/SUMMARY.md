# Sweep `sweep11`

- tag: `sweep11`
- baseline: `h0`
- seed: 1
- engine: `bdcb69ef331fef377d78509214ca48576b91f7a2`
- wall: 16239.8s
- stages: screen 3559.7s, final 12680.1s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 5 | `h0:clip=8` | 0.511 [0.480, 0.541] | 1024 | 1.84 / 1.69 | finalist |
| 1 | `h0:lcap=1` | 0.498 [0.467, 0.529] | 1024 | 1.79 / 1.69 | finalist |
| 3 | `h0:fusemacro=0` | 0.493 [0.463, 0.524] | 1024 | 1.81 / 1.69 | finalist |
| 4 | `h0:clip=3` | 0.479 [0.448, 0.509] | 1024 | 1.82 / 1.69 | finalist |
| 2 | `h0:clip=0` | 0.469 [0.438, 0.499] | 1024 | 1.81 / 1.69 | skipped: interval high 0.50 < 0.50 |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:lcap=1` | 0.487 [0.472, 0.503] | 0.493 [0.472, 0.515] | 0.489 [0.477, 0.502] |
| 3 | `h0:fusemacro=0` | 0.490 [0.475, 0.506] | 0.496 [0.474, 0.517] | 0.492 [0.480, 0.505] |
| 4 | `h0:clip=3` | 0.490 [0.474, 0.505] | 0.516 [0.494, 0.538] | 0.499 [0.486, 0.511] |
| 5 | `h0:clip=8` | 0.490 [0.474, 0.505] | 0.490 [0.468, 0.511] | 0.490 [0.477, 0.502] |

verdict: h0:lcap=1 coin flip — main 0.487 [0.472, 0.503], reverse 0.493 [0.472, 0.515], pooled 0.489 [0.477, 0.502] (not gated)
verdict: h0:fusemacro=0 coin flip — main 0.490 [0.475, 0.506], reverse 0.496 [0.474, 0.517], pooled 0.492 [0.480, 0.505] (not gated)
verdict: h0:clip=3 coin flip — main 0.490 [0.474, 0.505], reverse 0.516 [0.494, 0.538], pooled 0.499 [0.486, 0.511] (not gated)
verdict: h0:clip=8 coin flip — main 0.490 [0.474, 0.505], reverse 0.490 [0.468, 0.511], pooled 0.490 [0.477, 0.502] (not gated)

best: h0:fusemacro=0 (coin flip)
