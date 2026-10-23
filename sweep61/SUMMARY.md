# Sweep `sweep61`

- tag: `sweep61`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40`
- seed: 1
- engine: `75f087f95e767f72ff7d8fb8ae630431be990efe`
- wall: 23473.0s
- stages: screen 3896.1s, final 19576.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run61/order61.json,oorder=results/run53/leaf53.json,oopen=1,onext=40` | 0.500 [0.457, 0.543] | 512 | 0.25 / 0.29 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run61/order61.json,oorder=results/run53/leaf53.json,oopen=1,onext=40` | 0.485 [0.469, 0.501] | 0.515 [0.493, 0.536] | 0.496 [0.483, 0.509] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run61/order61.json,oorder=results/run53/leaf53.json,oopen=1,onext=40 coin flip — main 0.485 [0.469, 0.501], reverse 0.515 [0.493, 0.536], pooled 0.496 [0.483, 0.509] (not gated)

early stop: stopped after 3584/4096 main and 2048/2048 reverse games — not better, pooled < 0.5, < 0.51 settled at P = 0.995 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run61/order61.json,oorder=results/run53/leaf53.json,oopen=1,onext=40 (coin flip)
