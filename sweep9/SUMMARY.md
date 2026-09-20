# Sweep `sweep9`

- tag: `sweep9`
- baseline: `h0`
- seed: 1
- engine: `81c8d37f2a58308d14ad1ccf11460312b0da0960`
- wall: 8995.0s
- stages: screen 2771.8s, final 6223.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:oevo=0` | 0.508 [0.478, 0.539] | 1024 | 1.86 / 1.89 | finalist |
| 3 | `h0:info=draws` | 0.506 [0.475, 0.536] | 1024 | 1.81 / 1.89 | finalist |
| 1 | `h0:oevo=0,info=draws` | 0.501 [0.470, 0.532] | 1024 | 1.95 / 1.89 | not in top 2 |
| 4 | `h0:odepth=1` | 0.344 [0.315, 0.373] | 1024 | 1.85 / 1.89 | skipped: interval high 0.37 < 0.50 |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:oevo=0` | 0.485 [0.470, 0.501] | 0.511 [0.490, 0.533] | 0.494 [0.481, 0.506] |
| 3 | `h0:info=draws` | 0.494 [0.479, 0.510] | 0.506 [0.484, 0.527] | 0.498 [0.486, 0.511] |

verdict: h0:oevo=0 coin flip — main 0.485 [0.470, 0.501], reverse 0.511 [0.490, 0.533], pooled 0.494 [0.481, 0.506] (not gated)
verdict: h0:info=draws coin flip — main 0.494 [0.479, 0.510], reverse 0.506 [0.484, 0.527], pooled 0.498 [0.486, 0.511] (not gated)

best: h0:info=draws (coin flip)
