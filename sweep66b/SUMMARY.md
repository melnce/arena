# Sweep `sweep66b`

- tag: `sweep66b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run65/leaf65q.json`
- seed: 2
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`
- wall: 26213.7s
- stages: screen 3859.9s, final 22353.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run66/leaf66q.json` | 0.551 [0.507, 0.593] | 512 | 0.28 / 0.27 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run66/leaf66q.json` | 0.508 [0.492, 0.523] | 0.506 [0.484, 0.527] | 0.507 [0.494, 0.519] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run66/leaf66q.json coin flip — main 0.508 [0.492, 0.523], reverse 0.506 [0.484, 0.527], pooled 0.507 [0.494, 0.519] (not gated)

early stop: stopped after 4096/4096 main and 2048/2048 reverse games — not better, pooled ≥ 0.505 settled at P = 1.000 (γ 0.02, thresholds 0.505)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run66/leaf66q.json (coin flip)

## combined with sweep66

| index | spec | main | reverse | pooled |
|---|---:|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run66/leaf66q.json` | 0.503 [0.492, 0.514] | 0.511 [0.496, 0.527] | 0.506 [0.497, 0.515] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run66/leaf66q.json coin flip — main 0.503 [0.492, 0.514], reverse 0.511 [0.496, 0.527], pooled 0.506 [0.497, 0.515] (not gated)
