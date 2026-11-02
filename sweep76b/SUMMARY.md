# Sweep `sweep76b`

- tag: `sweep76b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256`
- seed: 3
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 8197.7s
- stages: screen 2764.1s, final 5433.6s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run76/leaf76rq.json,mull=leaf,mulls=256` | 0.500 [0.457, 0.543] | 512 | 0.36 / 0.35 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run76/leaf76rq.json,mull=leaf,mulls=256` | 0.479 [0.454, 0.504] | 0.510 [0.467, 0.553] | 0.486 [0.465, 0.508] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run76/leaf76rq.json,mull=leaf,mulls=256 coin flip — main 0.479 [0.454, 0.504], reverse 0.510 [0.467, 0.553], pooled 0.486 [0.465, 0.508] (not gated)

early stop: stopped after 1536/4096 main and 512/2048 reverse games — not better, pooled < 0.505 settled at P = 0.993 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run76/leaf76rq.json,mull=leaf,mulls=256 (coin flip)

## combined with sweep76

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run76/leaf76rq.json,mull=leaf,mulls=256` | 0.498 [0.485, 0.511] | 0.496 [0.476, 0.515] | 0.497 [0.486, 0.508] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run76/leaf76rq.json,mull=leaf,mulls=256 coin flip — main 0.498 [0.485, 0.511], reverse 0.496 [0.476, 0.515], pooled 0.497 [0.486, 0.508] (not gated)
