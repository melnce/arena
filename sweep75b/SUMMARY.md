# Sweep `sweep75b`

- tag: `sweep75b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run73/leaf73q.json,mull=leaf,mulls=256`
- seed: 2
- engine: `bcfb6cd2d737d7cb68cc8ef6bc0f1e3a65d17150`
- wall: 16869.2s
- stages: screen 2792.3s, final 14076.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256` | 0.496 [0.453, 0.539] | 512 | 0.36 / 0.37 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256` | 0.496 [0.480, 0.513] | 0.539 [0.514, 0.564] | 0.509 [0.495, 0.523] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256 unclear (main) — main 0.496 [0.480, 0.513], reverse 0.539 [0.514, 0.564], pooled 0.509 [0.495, 0.523] (not gated)

early stop: stopped after 3584/4096 main and 1536/2048 reverse games — not better, pooled ≥ 0.505 settled at P = 0.988 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256 (unclear (main))

## combined with sweep75

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256` | 0.502 [0.491, 0.513] | 0.520 [0.504, 0.537] | 0.508 [0.499, 0.517] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run75/leaf75q.json,mull=leaf,mulls=256 unclear (main) — main 0.502 [0.491, 0.513], reverse 0.520 [0.504, 0.537], pooled 0.508 [0.499, 0.517] (not gated)
