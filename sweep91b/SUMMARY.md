# Sweep `sweep91b`

- tag: `sweep91b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run89/leaf89q.json,mull=leaf,mulls=256`
- seed: 18
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 19676.7s
- stages: screen 2877.9s, final 16798.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run91/leaf91rq.json,mull=leaf,mulls=256` | 0.520 [0.476, 0.563] | 512 | 0.34 / 0.36 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run91/leaf91rq.json,mull=leaf,mulls=256` | 0.509 [0.493, 0.524] | 0.497 [0.475, 0.518] | 0.505 [0.492, 0.517] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run91/leaf91rq.json,mull=leaf,mulls=256 coin flip — main 0.509 [0.493, 0.524], reverse 0.497 [0.475, 0.518], pooled 0.505 [0.492, 0.517] (not gated)

early stop: stopped after 4096/4096 main and 2048/2048 reverse games — not better, pooled < 0.505 settled at P = 1.000 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run91/leaf91rq.json,mull=leaf,mulls=256 (coin flip)

## combined with sweep91

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run91/leaf91rq.json,mull=leaf,mulls=256` | 0.504 [0.493, 0.514] | 0.506 [0.490, 0.521] | 0.504 [0.495, 0.513] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run91/leaf91rq.json,mull=leaf,mulls=256 coin flip — main 0.504 [0.493, 0.514], reverse 0.506 [0.490, 0.521], pooled 0.504 [0.495, 0.513] (not gated)
