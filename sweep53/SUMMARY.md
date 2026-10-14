# Sweep `sweep53`

- tag: `sweep53`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1`
- seed: 1
- engine: `99e07753ecc3c7e513d0f3be2668a73af810b0f7`
- wall: 74638.9s
- stages: screen 36500.4s, final 38138.5s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,order=results/run53/leaf53.json,oorder=1` | 0.520 [0.476, 0.563] | 512 | 0.07 / 0.08 | finalist |
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,order=results/run53/leaf53.json` | 0.516 [0.472, 0.559] | 512 | 0.07 / 0.08 | not in top 1 |
| 3 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,net=results/run53/leaf53.json` | 0.492 [0.449, 0.535] | 512 | 0.06 / 0.08 | not in top 1 |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,order=results/run53/leaf53.json,oorder=1` | 0.541 [0.516, 0.566] | 0.538 [0.507, 0.568] | 0.540 [0.520, 0.559] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,order=results/run53/leaf53.json,oorder=1 better — main 0.541 [0.516, 0.566], reverse 0.538 [0.507, 0.568], pooled 0.540 [0.520, 0.559] (not gated)

early stop: stopped after 1536/2048 main and 1024/1024 reverse games — better, pooled ≥ 0.51 settled at P = 0.999 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,order=results/run53/leaf53.json,oorder=1 (better)
