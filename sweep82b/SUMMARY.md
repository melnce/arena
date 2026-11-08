# Sweep `sweep82b`

- tag: `sweep82b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- seed: 9
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 10881.1s
- stages: screen 2696.3s, final 8184.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run82/leaf82rq.json,mull=leaf,mulls=256` | 0.471 [0.428, 0.514] | 512 | 0.37 / 0.39 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run82/leaf82rq.json,mull=leaf,mulls=256` | 0.490 [0.468, 0.511] | 0.484 [0.454, 0.515] | 0.488 [0.470, 0.506] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run82/leaf82rq.json,mull=leaf,mulls=256 coin flip — main 0.490 [0.468, 0.511], reverse 0.484 [0.454, 0.515], pooled 0.488 [0.470, 0.506] (not gated)

early stop: stopped after 2048/4096 main and 1024/2048 reverse games — not better, pooled < 0.505 settled at P = 0.982 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run82/leaf82rq.json,mull=leaf,mulls=256 (coin flip)

## combined with sweep82

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run82/leaf82rq.json,mull=leaf,mulls=256` | 0.500 [0.487, 0.513] | 0.498 [0.480, 0.516] | 0.499 [0.489, 0.510] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run82/leaf82rq.json,mull=leaf,mulls=256 coin flip — main 0.500 [0.487, 0.513], reverse 0.498 [0.480, 0.516], pooled 0.499 [0.489, 0.510] (not gated)
