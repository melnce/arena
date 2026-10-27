# Sweep `sweep69b`

- tag: `sweep69b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json`
- seed: 2
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`
- wall: 16236.4s
- stages: screen 2637.4s, final 13599.0s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run69/leaf69q.json` | 0.520 [0.476, 0.563] | 512 | 0.39 / 0.39 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run69/leaf69q.json` | 0.509 [0.493, 0.526] | 0.514 [0.489, 0.539] | 0.511 [0.497, 0.524] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run69/leaf69q.json coin flip — main 0.509 [0.493, 0.526], reverse 0.514 [0.489, 0.539], pooled 0.511 [0.497, 0.524] (not gated)

early stop: stopped after 3584/4096 main and 1536/2048 reverse games — not better, pooled ≥ 0.505 settled at P = 0.998 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run69/leaf69q.json (coin flip)

## combined with sweep69

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run69/leaf69q.json` | 0.507 [0.496, 0.518] | 0.518 [0.501, 0.534] | 0.510 [0.501, 0.520] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run69/leaf69q.json unclear (main) — main 0.507 [0.496, 0.518], reverse 0.518 [0.501, 0.534], pooled 0.510 [0.501, 0.520] (not gated)
