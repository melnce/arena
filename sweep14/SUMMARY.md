# Sweep `sweep14`

- tag: `sweep14`
- baseline: `h0`
- seed: 1
- engine: `b5b822c6838a8ed731120c0b91d94cdf51a11e83`
- wall: 10353.3s
- stages: screen 3187.7s, final 7165.5s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 3 | `h0:bppv=2.4` | 0.521 [0.490, 0.551] | 1024 | 1.74 / 1.65 | finalist |
| 4 | `h0:bppv=6` | 0.512 [0.481, 0.542] | 1024 | 1.60 / 1.65 | finalist |
| 1 | `h0:bpp1=4` | 0.509 [0.478, 0.539] | 1024 | 1.59 / 1.65 | not in top 2 |
| 2 | `h0:bpp1=3` | 0.508 [0.477, 0.538] | 1024 | 1.64 / 1.65 | not in top 2 |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 3 | `h0:bppv=2.4` | 0.509 [0.494, 0.524] | 0.495 [0.473, 0.516] | 0.504 [0.492, 0.517] |
| 4 | `h0:bppv=6` | 0.500 [0.485, 0.515] | 0.493 [0.471, 0.514] | 0.498 [0.485, 0.510] |

verdict: h0:bppv=2.4 coin flip — main 0.509 [0.494, 0.524], reverse 0.495 [0.473, 0.516], pooled 0.504 [0.492, 0.517] (not gated)
verdict: h0:bppv=6 coin flip — main 0.500 [0.485, 0.515], reverse 0.493 [0.471, 0.514], pooled 0.498 [0.485, 0.510] (not gated)

best: h0:bppv=2.4 (coin flip)
