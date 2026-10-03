# Sweep `sweep36`

- tag: `sweep36`
- baseline: `h0`
- seed: 1
- engine: `33de0a942a2776863a9416ffab2397f1b9de96a1`
- wall: 7131.4s
- stages: screen 1575.4s, final 5555.9s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:net=results/sweep36/leaf-net98-e2-l2x10.json` | 0.490 [0.447, 0.533] | 512 | 1.57 / 1.54 | finalist |
| 3 | `h0:net=results/sweep36/leaf-net8-e2-l2x10.json` | 0.473 [0.430, 0.516] | 512 | 1.64 / 1.54 | finalist |
| 1 | `h0:net=results/sweep36/leaf-net9-e2-l2x10.json` | 0.457 [0.414, 0.500] | 512 | 1.76 / 1.54 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep36/leaf-net9-e2-l2x10.json` | 0.454 [0.432, 0.475] | 0.459 [0.429, 0.490] | 0.455 [0.438, 0.473] |
| 2 | `h0:net=results/sweep36/leaf-net98-e2-l2x10.json` | 0.471 [0.449, 0.492] | 0.475 [0.444, 0.505] | 0.472 [0.454, 0.490] |
| 3 | `h0:net=results/sweep36/leaf-net8-e2-l2x10.json` | 0.463 [0.442, 0.485] | 0.485 [0.455, 0.516] | 0.471 [0.453, 0.488] |

verdict: h0:net=results/sweep36/leaf-net9-e2-l2x10.json worse — main 0.454 [0.432, 0.475], reverse 0.459 [0.429, 0.490], pooled 0.455 [0.438, 0.473] (not gated)
verdict: h0:net=results/sweep36/leaf-net98-e2-l2x10.json worse — main 0.471 [0.449, 0.492], reverse 0.475 [0.444, 0.505], pooled 0.472 [0.454, 0.490] (not gated)
verdict: h0:net=results/sweep36/leaf-net8-e2-l2x10.json worse — main 0.463 [0.442, 0.485], reverse 0.485 [0.455, 0.516], pooled 0.471 [0.453, 0.488] (not gated)

best: h0:net=results/sweep36/leaf-net98-e2-l2x10.json (worse)
