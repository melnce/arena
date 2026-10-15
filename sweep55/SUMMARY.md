# Sweep `sweep55`

- tag: `sweep55`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1`
- seed: 1
- engine: `8093a9e93230d851bdb3f1bf776303d0ba80d789`
- wall: 65630.4s
- stages: screen 35798.8s, final 29831.6s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40` | 0.531 [0.488, 0.574] | 512 | 0.04 / 0.06 | finalist |
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,onext=40` | 0.516 [0.472, 0.559] | 512 | 0.05 / 0.06 | not in top 1 |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40` | 0.485 [0.455, 0.516] | 0.582 [0.539, 0.624] | 0.518 [0.493, 0.542] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40 unclear (main) — main 0.485 [0.455, 0.516], reverse 0.582 [0.539, 0.624], pooled 0.518 [0.493, 0.542] (not gated)

early stop: stopped after 1024/2048 main and 512/1024 reverse games — not better, pooled ≥ 0.49 settled at P = 0.998 (γ 0.02, thresholds 0.48, 0.49)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,order=results/run53/leaf53.json,oorder=1,oopen=1,onext=40 (unclear (main))
