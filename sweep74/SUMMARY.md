# Sweep `sweep74`

- tag: `sweep74`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run73/leaf73q.json,mull=leaf,mulls=256`
- seed: 1
- engine: `a3daa7bef9fd5e9d11b31e1898cdb18d64ffaedd`
- wall: 13589.1s
- stages: screen 5064.7s, final 8524.3s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run74/leaf74wq.json,mull=leaf,mulls=256` | 0.508 [0.465, 0.551] | 512 | 0.35 / 0.34 | finalist |
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run74/leaf74q.json,mull=leaf,mulls=256` | 0.500 [0.457, 0.543] | 512 | 0.35 / 0.34 | not in top 1 |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run74/leaf74wq.json,mull=leaf,mulls=256` | 0.486 [0.465, 0.508] | 0.484 [0.454, 0.515] | 0.486 [0.468, 0.503] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run74/leaf74wq.json,mull=leaf,mulls=256 coin flip — main 0.486 [0.465, 0.508], reverse 0.484 [0.454, 0.515], pooled 0.486 [0.468, 0.503] (not gated)

early stop: stopped after 2048/4096 main and 1024/2048 reverse games — not better, pooled < 0.5, < 0.51 settled at P = 0.987 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,net=results/run74/leaf74wq.json,mull=leaf,mulls=256 (coin flip)
