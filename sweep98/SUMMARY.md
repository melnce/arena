# Sweep `sweep98`

- tag: `sweep98`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256`
- seed: 98
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 22903.8s
- stages: screen 5727.8s, final 17176.0s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256` | 0.543 [0.500, 0.586] | 512 | 0.25 / 0.35 | finalist |
| 1 | `h0:nodes=48000,horizon=3,k=12,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256` | 0.531 [0.488, 0.574] | 512 | 0.33 / 0.35 | not in top 1 |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256` | 0.498 [0.481, 0.514] | 0.477 [0.452, 0.502] | 0.491 [0.478, 0.505] |

verdict: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256 coin flip — main 0.498 [0.481, 0.514], reverse 0.477 [0.452, 0.502], pooled 0.491 [0.478, 0.505] (not gated)

early stop: stopped after 3584/4096 main and 1536/2048 reverse games — not better, pooled < 0.5, < 0.51 settled at P = 0.999 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=0,onext=0,deal=block,net=results/run97/leaf97q.json,mull=leaf,mulls=256 (coin flip)
