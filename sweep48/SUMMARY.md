# Sweep `sweep48`

- tag: `sweep48`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json`
- seed: 1
- engine: `470ba212e77c89ee3956bc302239df6e173834fc`
- wall: 52472.8s
- stages: screen 12270.3s, final 40202.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,net=results/sweep43/leaf-v3-e3.json` | 0.512 [0.468, 0.555] | 512 | 0.07 / 0.09 | finalist |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,net=results/sweep43/leaf-v3-e3.json` | 0.496 [0.474, 0.517] | 0.482 [0.452, 0.513] | 0.491 [0.474, 0.509] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,net=results/sweep43/leaf-v3-e3.json coin flip — main 0.496 [0.474, 0.517], reverse 0.482 [0.452, 0.513], pooled 0.491 [0.474, 0.509] (not gated)

early stop: stopped after 2048/2048 main and 1024/1024 reverse games — not better, pooled ≥ 0.49 settled at P = 1.000 (γ 0.02, thresholds 0.48, 0.49)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,net=results/sweep43/leaf-v3-e3.json (coin flip)
