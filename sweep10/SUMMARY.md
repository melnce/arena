# Sweep `sweep10`

- tag: `sweep10`
- baseline: `h0`
- seed: 1
- engine: `5920dbf1673b4fa3c0be25d84f1b582e47122780`
- wall: 29816.9s
- stages: screen 4873.6s, final 24943.3s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 5 | `h0:lcap=0.5,clip=5,fusemacro=1` | 0.537 [0.506, 0.567] | 1024 | 0.99 / 1.69 | finalist |
| 3 | `h0:clip=5` | 0.527 [0.496, 0.557] | 1024 | 1.10 / 1.69 | finalist |
| 1 | `h0:lcap=0.5` | 0.515 [0.484, 0.545] | 1024 | 1.81 / 1.69 | finalist |
| 4 | `h0:fusemacro=1` | 0.508 [0.478, 0.539] | 1024 | 1.12 / 1.69 | finalist |
| 2 | `h0:lcap=0.25` | 0.507 [0.476, 0.537] | 1024 | 1.69 / 1.69 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:lcap=0.5` | 0.495 [0.480, 0.511] | 0.519 [0.497, 0.540] | 0.503 [0.491, 0.516] |
| 2 | `h0:lcap=0.25` | 0.499 [0.484, 0.515] | 0.524 [0.502, 0.545] | 0.507 [0.495, 0.520] |
| 3 | `h0:clip=5` | 0.529 [0.514, 0.544] | 0.535 [0.514, 0.557] | 0.531 [0.519, 0.543] |
| 4 | `h0:fusemacro=1` | 0.490 [0.475, 0.505] | 0.516 [0.494, 0.537] | 0.499 [0.486, 0.511] |
| 5 | `h0:lcap=0.5,clip=5,fusemacro=1` | 0.543 [0.528, 0.558] | 0.547 [0.525, 0.568] | 0.544 [0.532, 0.557] |

verdict: h0:lcap=0.5 coin flip — main 0.495 [0.480, 0.511], reverse 0.519 [0.497, 0.540], pooled 0.503 [0.491, 0.516] (not gated)
verdict: h0:lcap=0.25 unclear (main) — main 0.499 [0.484, 0.515], reverse 0.524 [0.502, 0.545], pooled 0.507 [0.495, 0.520] (not gated)
verdict: h0:clip=5 better — main 0.529 [0.514, 0.544], reverse 0.535 [0.514, 0.557], pooled 0.531 [0.519, 0.543] (not gated)
verdict: h0:fusemacro=1 coin flip — main 0.490 [0.475, 0.505], reverse 0.516 [0.494, 0.537], pooled 0.499 [0.486, 0.511] (not gated)
verdict: h0:lcap=0.5,clip=5,fusemacro=1 better — main 0.543 [0.528, 0.558], reverse 0.547 [0.525, 0.568], pooled 0.544 [0.532, 0.557] (not gated)

best: h0:lcap=0.5,clip=5,fusemacro=1 (better)
