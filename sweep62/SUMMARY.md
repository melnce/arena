# Sweep `sweep62`

- tag: `sweep62`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40`
- seed: 1
- engine: `75f087f95e767f72ff7d8fb8ae630431be990efe`
- wall: 30312.3s
- stages: screen 8973.9s, final 21338.3s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40` | 0.469 [0.426, 0.512] | 512 | 0.25 / 0.29 | finalist |
| 3 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run59/order59.json` | 0.428 [0.386, 0.471] | 512 | 0.28 / 0.29 | skipped: interval high 0.47 < 0.50 |
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run61/order61.json` | 0.398 [0.357, 0.441] | 512 | 0.27 / 0.29 | skipped: interval high 0.44 < 0.50 |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40` | 0.508 [0.492, 0.523] | 0.494 [0.472, 0.515] | 0.503 [0.490, 0.515] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40 coin flip — main 0.508 [0.492, 0.523], reverse 0.494 [0.472, 0.515], pooled 0.503 [0.490, 0.515] (not gated)

early stop: stopped after 4096/4096 main and 2048/2048 reverse games — not better, pooled ≥ 0.5 settled at P = 1.000 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40 (coin flip)
