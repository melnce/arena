# Sweep `sweep79b`

- tag: `sweep79b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run77/leaf77q.json,mull=leaf,mulls=256`
- seed: 6
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 9718.5s
- stages: screen 2911.6s, final 6806.9s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run79/leaf79q.json,mull=leaf,mulls=256` | 0.496 [0.453, 0.539] | 512 | 0.36 / 0.33 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run79/leaf79q.json,mull=leaf,mulls=256` | 0.484 [0.459, 0.509] | 0.511 [0.480, 0.541] | 0.495 [0.476, 0.514] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run79/leaf79q.json,mull=leaf,mulls=256 coin flip — main 0.484 [0.459, 0.509], reverse 0.511 [0.480, 0.541], pooled 0.495 [0.476, 0.514] (not gated)

early stop: stopped after 1536/4096 main and 1024/2048 reverse games — not better, pooled < 0.505 settled at P = 0.986 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run79/leaf79q.json,mull=leaf,mulls=256 (coin flip)

## combined with sweep79

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run79/leaf79q.json,mull=leaf,mulls=256` | 0.498 [0.485, 0.511] | 0.500 [0.482, 0.517] | 0.499 [0.488, 0.509] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run79/leaf79q.json,mull=leaf,mulls=256 coin flip — main 0.498 [0.485, 0.511], reverse 0.500 [0.482, 0.517], pooled 0.499 [0.488, 0.509] (not gated)
