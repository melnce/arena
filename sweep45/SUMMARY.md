# Sweep `sweep45`

- tag: `sweep45`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json`
- seed: 1
- engine: `ed574a41fc9b6e49aeb008a4b835510c15c0c739`
- wall: 62667.8s
- stages: screen 16461.5s, final 46206.3s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json` | 0.541 [0.498, 0.584] | 512 | 0.04 / 0.08 | finalist |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json` | 0.512 [0.491, 0.534] | 0.497 [0.467, 0.528] | 0.507 [0.489, 0.525] |

verdict: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json coin flip — main 0.512 [0.491, 0.534], reverse 0.497 [0.467, 0.528], pooled 0.507 [0.489, 0.525] (not gated)

early stop: stopped after 2048/2048 main and 1024/1024 reverse games — not better, pooled ≥ 0.505 settled at P = 1.000 (γ 0.02, thresholds 0.505, 0.515)

best: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json (coin flip)
