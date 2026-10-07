# Sweep `sweep44`

- tag: `sweep44`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=engine/models/h0-linear-v3.json`
- seed: 1
- engine: `3742a1bea63c93b73556f7077c71fc002d3f13d0`
- wall: 47292.4s
- stages: screen 11684.0s, final 35608.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json` | 0.502 [0.459, 0.545] | 512 | 0.08 / 0.09 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json` | 0.498 [0.476, 0.519] | 0.495 [0.465, 0.526] | 0.497 [0.479, 0.514] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json coin flip — main 0.498 [0.476, 0.519], reverse 0.495 [0.465, 0.526], pooled 0.497 [0.479, 0.514] (not gated)

early stop: stopped after 2048/2048 main and 1024/1024 reverse games — not better, pooled ≥ 0.49 settled at P = 1.000 (γ 0.02, thresholds 0.48, 0.49)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json (coin flip)
