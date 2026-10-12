# Sweep `sweep50`

- tag: `sweep50`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,net=results/sweep43/leaf-v3-e3.json`
- seed: 1
- engine: `d1bbc531306180b3382905da3ee08669effc6a7a`
- wall: 59482.1s
- stages: screen 20513.7s, final 38968.3s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,net=results/sweep43/leaf-v3-e3.json` | 0.520 [0.476, 0.563] | 512 | 0.04 / 0.06 | finalist |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,net=results/sweep43/leaf-v3-e3.json` | 0.504 [0.479, 0.529] | 0.510 [0.467, 0.553] | 0.505 [0.484, 0.527] |

verdict: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,net=results/sweep43/leaf-v3-e3.json coin flip — main 0.504 [0.479, 0.529], reverse 0.510 [0.467, 0.553], pooled 0.505 [0.484, 0.527] (not gated)

early stop: stopped after 1536/2048 main and 512/1024 reverse games — not better, pooled ≥ 0.49 settled at P = 0.990 (γ 0.02, thresholds 0.48, 0.49)

best: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,net=results/sweep43/leaf-v3-e3.json (coin flip)
