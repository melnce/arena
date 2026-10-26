# Sweep `sweep67`

- tag: `sweep67`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40,net=results/run66/leaf66q.json`
- seed: 1
- engine: `d49783c10f1027531eb8cca665cff828ac439b8b`
- wall: 18187.1s
- stages: screen 3159.2s, final 15027.9s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json` | 0.527 [0.484, 0.570] | 512 | 0.37 / 0.27 | finalist |

## final

final games per pair: 2 … 17 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json` | 0.530 [0.513, 0.548] | 0.542 [0.517, 0.566] | 0.534 [0.520, 0.548] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json better — main 0.530 [0.513, 0.548], reverse 0.542 [0.517, 0.566], pooled 0.534 [0.520, 0.548] (not gated)

early stop: stopped after 3072/4096 main and 1536/2048 reverse games — better, pooled ≥ 0.51 settled at P = 0.999 (γ 0.02, thresholds 0.5, 0.51)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,net=results/run66/leaf66q.json (better)
