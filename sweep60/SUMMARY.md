# Sweep `sweep60`

- tag: `sweep60`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40`
- seed: 1
- engine: `75f087f95e767f72ff7d8fb8ae630431be990efe`
- wall: 22754.9s
- stages: screen 3603.6s, final 19151.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run60/order60.json,oorder=results/run53/leaf53.json,oopen=1,onext=40` | 0.514 [0.470, 0.557] | 512 | 0.28 / 0.28 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run60/order60.json,oorder=results/run53/leaf53.json,oopen=1,onext=40` | 0.489 [0.473, 0.506] | 0.497 [0.475, 0.518] | 0.492 [0.479, 0.505] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run60/order60.json,oorder=results/run53/leaf53.json,oopen=1,onext=40 coin flip — main 0.489 [0.473, 0.506], reverse 0.497 [0.475, 0.518], pooled 0.492 [0.479, 0.505] (not gated)

early stop: stopped after 3584/4096 main and 2048/2048 reverse games — not better, pooled < 0.5, < 0.51 settled at P = 1.000 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run60/order60.json,oorder=results/run53/leaf53.json,oopen=1,onext=40 (coin flip)
