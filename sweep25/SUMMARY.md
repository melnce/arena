# Sweep `sweep25`

- tag: `sweep25`
- baseline: `h0`
- seed: 1
- engine: `15b845c5df90eab1734d4f17cf5aabc1d506fe42`
- wall: 7348.5s
- stages: screen 1518.1s, final 5830.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/sweep25/leaf-all4-e2.json` | 0.537 [0.494, 0.580] | 512 | 1.69 / 1.57 | finalist |
| 2 | `h0:net=results/sweep25/leaf-all4-e4.json` | 0.531 [0.488, 0.574] | 512 | 1.78 / 1.57 | finalist |
| 3 | `h0:net=results/sweep25/leaf-all4-e8.json` | 0.496 [0.453, 0.539] | 512 | 1.74 / 1.57 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep25/leaf-all4-e2.json` | 0.507 [0.485, 0.528] | 0.523 [0.493, 0.554] | 0.512 [0.495, 0.530] |
| 2 | `h0:net=results/sweep25/leaf-all4-e4.json` | 0.503 [0.481, 0.525] | 0.491 [0.461, 0.522] | 0.499 [0.481, 0.517] |
| 3 | `h0:net=results/sweep25/leaf-all4-e8.json` | 0.480 [0.459, 0.502] | 0.474 [0.443, 0.504] | 0.478 [0.461, 0.496] |

verdict: h0:net=results/sweep25/leaf-all4-e2.json coin flip — main 0.507 [0.485, 0.528], reverse 0.523 [0.493, 0.554], pooled 0.512 [0.495, 0.530] (not gated)
verdict: h0:net=results/sweep25/leaf-all4-e4.json coin flip — main 0.503 [0.481, 0.525], reverse 0.491 [0.461, 0.522], pooled 0.499 [0.481, 0.517] (not gated)
verdict: h0:net=results/sweep25/leaf-all4-e8.json coin flip — main 0.480 [0.459, 0.502], reverse 0.474 [0.443, 0.504], pooled 0.478 [0.461, 0.496] (not gated)

best: h0:net=results/sweep25/leaf-all4-e2.json (coin flip)
