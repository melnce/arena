# Sweep `sweep68b`

- tag: `sweep68b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json`
- seed: 2
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`
- wall: 19290.6s
- stages: screen 2565.8s, final 16724.7s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run68/leaf68q.json` | 0.504 [0.461, 0.547] | 512 | 0.40 / 0.39 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run68/leaf68q.json` | 0.497 [0.482, 0.513] | 0.510 [0.489, 0.532] | 0.502 [0.489, 0.514] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run68/leaf68q.json coin flip — main 0.497 [0.482, 0.513], reverse 0.510 [0.489, 0.532], pooled 0.502 [0.489, 0.514] (not gated)

early stop: stopped after 4096/4096 main and 2048/2048 reverse games — not better, pooled < 0.505 settled at P = 1.000 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run68/leaf68q.json (coin flip)

## combined with sweep68

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run68/leaf68q.json` | 0.500 [0.489, 0.510] | 0.512 [0.496, 0.527] | 0.504 [0.495, 0.512] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run68/leaf68q.json coin flip — main 0.500 [0.489, 0.510], reverse 0.512 [0.496, 0.527], pooled 0.504 [0.495, 0.512] (not gated)
