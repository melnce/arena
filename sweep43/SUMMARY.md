# Sweep `sweep43`

- tag: `sweep43`
- baseline: `h0`
- seed: 1
- engine: `3742a1bea63c93b73556f7077c71fc002d3f13d0`
- wall: 2180.3s
- stages: screen 635.5s, final 1544.7s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/sweep43/leaf-v3-e3.json` | 0.525 [0.482, 0.568] | 512 | 1.63 / 1.58 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep43/leaf-v3-e3.json` | 0.510 [0.485, 0.535] | 0.484 [0.454, 0.515] | 0.500 [0.481, 0.519] |

verdict: h0:net=results/sweep43/leaf-v3-e3.json coin flip — main 0.510 [0.485, 0.535], reverse 0.484 [0.454, 0.515], pooled 0.500 [0.481, 0.519] (not gated)

early stop: stopped after 1536/2048 main and 1024/1024 reverse games — not better, pooled ≥ 0.49 settled at P = 0.997 (γ 0.02, thresholds 0.48, 0.49)

best: h0:net=results/sweep43/leaf-v3-e3.json (coin flip)
