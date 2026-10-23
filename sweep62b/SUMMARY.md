# Sweep `sweep62b`

- tag: `sweep62b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40`
- seed: 2
- engine: `75f087f95e767f72ff7d8fb8ae630431be990efe`
- wall: 21030.5s
- stages: screen 3595.6s, final 17434.9s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40` | 0.512 [0.468, 0.555] | 512 | 0.28 / 0.27 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40` | 0.499 [0.482, 0.515] | 0.503 [0.478, 0.528] | 0.500 [0.486, 0.513] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40 coin flip — main 0.499 [0.482, 0.515], reverse 0.503 [0.478, 0.528], pooled 0.500 [0.486, 0.513] (not gated)

early stop: stopped after 3584/4096 main and 1536/2048 reverse games — not better, pooled < 0.505 settled at P = 0.996 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40 (coin flip)

## combined with sweep62

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40` | 0.503 [0.492, 0.515] | 0.497 [0.481, 0.514] | 0.502 [0.492, 0.511] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=2,onext=40 coin flip — main 0.503 [0.492, 0.515], reverse 0.497 [0.481, 0.514], pooled 0.502 [0.492, 0.511] (not gated)
