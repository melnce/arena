# Sweep `sweep32`

- tag: `sweep32`
- baseline: `h0`
- seed: 1
- engine: `d5f4310421b433cadf8711d24e7ebc726102c1c3`
- wall: 3317.2s
- stages: screen 875.6s, final 2441.6s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:tkill=2000` | 0.527 [0.484, 0.570] | 512 | 1.01 / 1.39 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:tkill=2000` | 0.511 [0.490, 0.533] | 0.502 [0.471, 0.533] | 0.508 [0.490, 0.526] |

verdict: h0:tkill=2000 coin flip — main 0.511 [0.490, 0.533], reverse 0.502 [0.471, 0.533], pooled 0.508 [0.490, 0.526] (not gated)

best: h0:tkill=2000 (coin flip)
