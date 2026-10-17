# Sweep `sweep57`

- tag: `sweep57`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40`
- seed: 1
- engine: `6aab0a827974840945cfd0b6f0078f56968e79ea`
- wall: 102229.8s
- stages: screen 27478.3s, final 74751.5s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run57/order57.json,oorder=1,oopen=1,onext=40` | 0.502 [0.459, 0.545] | 512 | 0.04 / 0.04 | finalist |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run57/order57.json,oorder=1,oopen=1,onext=40` | 0.484 [0.459, 0.509] | 0.496 [0.466, 0.527] | 0.489 [0.469, 0.508] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run57/order57.json,oorder=1,oopen=1,onext=40 coin flip — main 0.484 [0.459, 0.509], reverse 0.496 [0.466, 0.527], pooled 0.489 [0.469, 0.508] (not gated)

early stop: stopped after 1536/2048 main and 1024/1024 reverse games — not better, pooled < 0.5, < 0.51 settled at P = 0.998 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run57/order57.json,oorder=1,oopen=1,onext=40 (coin flip)
