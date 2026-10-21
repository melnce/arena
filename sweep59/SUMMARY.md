# Sweep `sweep59`

- tag: `sweep59`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40`
- seed: 1
- engine: `2e529fb9869b2ed698911c8f0a9c8df18ccf6060`
- wall: 137416.0s
- stages: screen 52427.2s, final 84988.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40` | 0.525 [0.482, 0.568] | 512 | 0.03 / 0.04 | finalist |
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run57/order57.json,oorder=results/run53/leaf53.json,oopen=1,onext=40` | 0.521 [0.478, 0.564] | 512 | 0.04 / 0.04 | not in top 1 |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40` | 0.507 [0.486, 0.529] | 0.527 [0.497, 0.558] | 0.514 [0.496, 0.532] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40 coin flip — main 0.507 [0.486, 0.529], reverse 0.527 [0.497, 0.558], pooled 0.514 [0.496, 0.532] (not gated)

early stop: stopped after 2048/2048 main and 1024/1024 reverse games — not better, pooled ≥ 0.51 settled at P = 1.000 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run59/order59.json,oorder=results/run53/leaf53.json,oopen=1,onext=40 (coin flip)
