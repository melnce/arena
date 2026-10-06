# Sweep `sweep40`

- tag: `sweep40`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000`
- seed: 1
- engine: `5a453256d573ae26297f814c3a97786b65b62871`
- wall: 47420.1s
- stages: screen 13337.8s, final 34082.3s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,wseed=turn` | 0.512 [0.468, 0.555] | 512 | 0.06 / 0.09 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,wseed=turn` | 0.499 [0.477, 0.521] | 0.485 [0.455, 0.516] | 0.494 [0.477, 0.512] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,wseed=turn coin flip — main 0.499 [0.477, 0.521], reverse 0.485 [0.455, 0.516], pooled 0.494 [0.477, 0.512] (not gated)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,wseed=turn (coin flip)
