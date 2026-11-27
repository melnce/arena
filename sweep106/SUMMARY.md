# Sweep `sweep106`

- tag: `sweep106`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run100/leaf100q.json,mull=leaf,mulls=256`
- seed: 106
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 16210.0s
- stages: screen 4881.6s, final 11328.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run106/leaf106rq.json,mull=leaf,mulls=256` | 0.508 [0.465, 0.551] | 512 | 0.37 / 0.36 | finalist |
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run106/leaf106q.json,mull=leaf,mulls=256` | 0.498 [0.455, 0.541] | 512 | 0.38 / 0.36 | not in top 1 |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run106/leaf106rq.json,mull=leaf,mulls=256` | 0.512 [0.492, 0.531] | 0.508 [0.483, 0.533] | 0.510 [0.495, 0.526] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run106/leaf106rq.json,mull=leaf,mulls=256 coin flip — main 0.512 [0.492, 0.531], reverse 0.508 [0.483, 0.533], pooled 0.510 [0.495, 0.526] (not gated)

early stop: stopped after 2560/4096 main and 1536/2048 reverse games — not better, pooled ≥ 0.49 settled at P = 0.995 (γ 0.02, thresholds 0.49)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run106/leaf106rq.json,mull=leaf,mulls=256 (coin flip)
