# Sweep `sweep45b`

- tag: `sweep45b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json`
- seed: 2
- engine: `ed574a41fc9b6e49aeb008a4b835510c15c0c739`
- wall: 70227.5s
- stages: screen 18647.2s, final 51580.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json` | 0.512 [0.468, 0.555] | 512 | 0.04 / 0.09 | finalist |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json` | 0.504 [0.482, 0.526] | 0.520 [0.489, 0.550] | 0.509 [0.491, 0.527] |

verdict: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json coin flip — main 0.504 [0.482, 0.526], reverse 0.520 [0.489, 0.550], pooled 0.509 [0.491, 0.527] (not gated)

early stop: stopped after 2048/2048 main and 1024/1024 reverse games — not better, pooled < 0.51 settled at P = 1.000 (γ 0.02, thresholds 0.51)

best: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json (coin flip)

## combined with sweep45

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json` | 0.508 [0.493, 0.523] | 0.508 [0.487, 0.530] | 0.508 [0.496, 0.521] |

verdict: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json coin flip — main 0.508 [0.493, 0.523], reverse 0.508 [0.487, 0.530], pooled 0.508 [0.496, 0.521] (not gated)
