# Sweep `sweep19`

- tag: `sweep19`
- baseline: `h0:nodes=6000`
- seed: 1
- engine: `dc868a47b84865c3a811c89dfc03a0ebb3bd019f`
- wall: 11281.2s
- stages: screen 3011.8s, final 8269.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=6000,olsolve=200` | 0.492 [0.449, 0.535] | 512 | 0.23 / 0.74 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=6000,olsolve=200` | 0.475 [0.454, 0.497] | 0.470 [0.439, 0.500] | 0.473 [0.456, 0.491] |

verdict: h0:nodes=6000,olsolve=200 worse — main 0.475 [0.454, 0.497], reverse 0.470 [0.439, 0.500], pooled 0.473 [0.456, 0.491] (not gated)

best: h0:nodes=6000,olsolve=200 (worse)
