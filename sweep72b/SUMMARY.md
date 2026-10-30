# Sweep `sweep72b`

- tag: `sweep72b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json`
- seed: 2
- engine: `c30242d841dcfd7e00925d0751bf39dfef253599`
- wall: 18739.1s
- stages: screen 2662.6s, final 16076.5s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json,mull=leaf,mulls=256` | 0.480 [0.437, 0.524] | 512 | 0.37 / 0.40 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json,mull=leaf,mulls=256` | 0.508 [0.493, 0.524] | 0.512 [0.491, 0.534] | 0.510 [0.497, 0.522] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json,mull=leaf,mulls=256 coin flip — main 0.508 [0.493, 0.524], reverse 0.512 [0.491, 0.534], pooled 0.510 [0.497, 0.522] (not gated)

early stop: stopped after 4096/4096 main and 2048/2048 reverse games — not better, pooled ≥ 0.505 settled at P = 1.000 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json,mull=leaf,mulls=256 (coin flip)

## combined with sweep72

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json,mull=leaf,mulls=256` | 0.505 [0.494, 0.516] | 0.507 [0.491, 0.522] | 0.505 [0.497, 0.514] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json,mull=leaf,mulls=256 coin flip — main 0.505 [0.494, 0.516], reverse 0.507 [0.491, 0.522], pooled 0.505 [0.497, 0.514] (not gated)
