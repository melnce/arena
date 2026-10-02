# Sweep `sweep33`

- tag: `sweep33`
- baseline: `h0:tkill=2000`
- seed: 1
- engine: `d5f4310421b433cadf8711d24e7ebc726102c1c3`
- wall: 8099.9s
- stages: screen 1780.3s, final 6319.6s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:tkill=2000,okill=7` | 0.523 [0.480, 0.566] | 512 | 1.02 / 0.95 | finalist |
| 2 | `h0:tkill=2000,okill=3` | 0.516 [0.472, 0.559] | 512 | 0.95 / 0.95 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:tkill=2000,okill=7` | 0.499 [0.477, 0.520] | 0.490 [0.460, 0.521] | 0.496 [0.478, 0.513] |
| 2 | `h0:tkill=2000,okill=3` | 0.504 [0.483, 0.526] | 0.492 [0.462, 0.523] | 0.500 [0.483, 0.518] |

verdict: h0:tkill=2000,okill=7 coin flip — main 0.499 [0.477, 0.520], reverse 0.490 [0.460, 0.521], pooled 0.496 [0.478, 0.513] (not gated)
verdict: h0:tkill=2000,okill=3 coin flip — main 0.504 [0.483, 0.526], reverse 0.492 [0.462, 0.523], pooled 0.500 [0.483, 0.518] (not gated)

best: h0:tkill=2000,okill=3 (coin flip)
