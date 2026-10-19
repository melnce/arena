# Sweep `sweep58`

- tag: `sweep58`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40`
- seed: 1
- engine: `6aab0a827974840945cfd0b6f0078f56968e79ea`
- wall: 111786.7s
- stages: screen 51964.7s, final 59822.0s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40` | 0.520 [0.476, 0.563] | 512 | 0.03 / 0.04 | finalist |
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=12,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40` | 0.508 [0.465, 0.551] | 512 | 0.03 / 0.04 | not in top 1 |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40` | 0.510 [0.485, 0.535] | 0.508 [0.465, 0.551] | 0.509 [0.488, 0.531] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40 coin flip — main 0.510 [0.485, 0.535], reverse 0.508 [0.465, 0.551], pooled 0.509 [0.488, 0.531] (not gated)

early stop: stopped after 1536/2048 main and 512/1024 reverse games — not better, pooled ≥ 0.49 settled at P = 0.996 (γ 0.02, thresholds 0.48, 0.49)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40 (coin flip)
