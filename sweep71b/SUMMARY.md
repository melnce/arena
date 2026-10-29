# Sweep `sweep71b`

- tag: `sweep71b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run69/leaf69q.json`
- seed: 2
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`
- wall: 18853.6s
- stages: screen 2669.8s, final 16183.7s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run71/leaf71q.json` | 0.521 [0.478, 0.564] | 512 | 0.38 / 0.38 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run71/leaf71q.json` | 0.499 [0.483, 0.514] | 0.509 [0.488, 0.531] | 0.502 [0.490, 0.515] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run71/leaf71q.json coin flip — main 0.499 [0.483, 0.514], reverse 0.509 [0.488, 0.531], pooled 0.502 [0.490, 0.515] (not gated)

early stop: stopped after 4096/4096 main and 2048/2048 reverse games — not better, pooled < 0.505 settled at P = 1.000 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run71/leaf71q.json (coin flip)

## combined with sweep71

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run71/leaf71q.json` | 0.500 [0.489, 0.511] | 0.511 [0.496, 0.526] | 0.504 [0.495, 0.513] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run71/leaf71q.json coin flip — main 0.500 [0.489, 0.511], reverse 0.511 [0.496, 0.526], pooled 0.504 [0.495, 0.513] (not gated)
