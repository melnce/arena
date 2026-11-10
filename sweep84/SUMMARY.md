# Sweep `sweep84`

- tag: `sweep84`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run83/leaf83rq.json,mull=leaf,mulls=256`
- seed: 84
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 21774.7s
- stages: screen 5066.7s, final 16708.0s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run84/leaf84mq.json,mull=leaf,mulls=256` | 0.521 [0.478, 0.564] | 512 | 0.35 / 0.36 | finalist |
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run84/leaf84q.json,mull=leaf,mulls=256` | 0.514 [0.470, 0.557] | 512 | 0.28 / 0.36 | not in top 1 |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run84/leaf84mq.json,mull=leaf,mulls=256` | 0.495 [0.479, 0.510] | 0.496 [0.474, 0.517] | 0.495 [0.482, 0.507] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run84/leaf84mq.json,mull=leaf,mulls=256 coin flip — main 0.495 [0.479, 0.510], reverse 0.496 [0.474, 0.517], pooled 0.495 [0.482, 0.507] (not gated)

early stop: stopped after 4096/4096 main and 2048/2048 reverse games — not better, pooled < 0.5, < 0.51 settled at P = 1.000 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run84/leaf84mq.json,mull=leaf,mulls=256 (coin flip)
