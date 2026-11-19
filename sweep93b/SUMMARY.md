# Sweep `sweep93b`

- tag: `sweep93b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run92/leaf92q.json,mull=leaf,mulls=256`
- seed: 20
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 8633.7s
- stages: screen 2802.5s, final 5831.1s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run93/leaf93q.json,mull=leaf,mulls=256` | 0.512 [0.468, 0.555] | 512 | 0.37 / 0.36 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run93/leaf93q.json,mull=leaf,mulls=256` | 0.498 [0.473, 0.523] | 0.467 [0.424, 0.510] | 0.490 [0.469, 0.512] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run93/leaf93q.json,mull=leaf,mulls=256 coin flip — main 0.498 [0.473, 0.523], reverse 0.467 [0.424, 0.510], pooled 0.490 [0.469, 0.512] (not gated)

early stop: stopped after 1536/4096 main and 512/2048 reverse games — not better, pooled < 0.505 settled at P = 0.981 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run93/leaf93q.json,mull=leaf,mulls=256 (coin flip)

## combined with sweep93

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run93/leaf93q.json,mull=leaf,mulls=256` | 0.501 [0.488, 0.514] | 0.493 [0.474, 0.512] | 0.499 [0.488, 0.509] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run93/leaf93q.json,mull=leaf,mulls=256 coin flip — main 0.501 [0.488, 0.514], reverse 0.493 [0.474, 0.512], pooled 0.499 [0.488, 0.509] (not gated)
