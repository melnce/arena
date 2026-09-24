# Sweep `sweep12`

- tag: `sweep12`
- baseline: `h0`
- seed: 1
- engine: `bdcb69ef331fef377d78509214ca48576b91f7a2`
- wall: 12504.4s
- stages: screen 2250.9s, final 10253.5s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/net4-meta/linear.json` | 0.498 [0.467, 0.529] | 1024 | 1.98 / 1.61 | finalist |
| 2 | `h0:net=results/net4-meta/mlp.json` | 0.483 [0.453, 0.514] | 1024 | 1.50 / 1.61 | finalist |
| 3 | `h0:net=results/net4-meta/linear.json,clip=0` | 0.471 [0.440, 0.501] | 1024 | 1.86 / 1.61 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/net4-meta/linear.json` | 0.507 [0.492, 0.522] | 0.486 [0.464, 0.507] | 0.500 [0.487, 0.512] |
| 2 | `h0:net=results/net4-meta/mlp.json` | 0.470 [0.455, 0.486] | 0.485 [0.463, 0.507] | 0.475 [0.463, 0.488] |
| 3 | `h0:net=results/net4-meta/linear.json,clip=0` | 0.482 [0.467, 0.498] | 0.479 [0.457, 0.500] | 0.481 [0.469, 0.494] |

verdict: h0:net=results/net4-meta/linear.json coin flip — main 0.507 [0.492, 0.522], reverse 0.486 [0.464, 0.507], pooled 0.500 [0.487, 0.512] (not gated)
verdict: h0:net=results/net4-meta/mlp.json worse — main 0.470 [0.455, 0.486], reverse 0.485 [0.463, 0.507], pooled 0.475 [0.463, 0.488] (not gated)
verdict: h0:net=results/net4-meta/linear.json,clip=0 worse — main 0.482 [0.467, 0.498], reverse 0.479 [0.457, 0.500], pooled 0.481 [0.469, 0.494] (not gated)

best: h0:net=results/net4-meta/linear.json (coin flip)
