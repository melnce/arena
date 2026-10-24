# Sweep `sweep63`

- tag: `sweep63`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40`
- seed: 1
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`
- wall: 23660.9s
- stages: screen 8030.9s, final 15629.9s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=40` | 0.490 [0.447, 0.533] | 512 | 0.40 / 0.31 | finalist |
| 3 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run63/leaf63.json` | 0.475 [0.432, 0.518] | 512 | 0.26 / 0.31 | not in top 1 |
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,onextleaf=1` | 0.420 [0.378, 0.463] | 512 | 0.32 / 0.31 | skipped: interval high 0.46 < 0.50 |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=40` | 0.492 [0.476, 0.509] | 0.496 [0.471, 0.521] | 0.493 [0.480, 0.507] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=40 coin flip — main 0.492 [0.476, 0.509], reverse 0.496 [0.471, 0.521], pooled 0.493 [0.480, 0.507] (not gated)

early stop: stopped after 3584/4096 main and 1536/2048 reverse games — not better, pooled < 0.5, < 0.51 settled at P = 0.987 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=40 (coin flip)
