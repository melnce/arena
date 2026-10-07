# Sweep `sweep41`

- tag: `sweep41`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000`
- seed: 1
- engine: `9f187a3faacb63c556268be47b0e87e4fd7bd4db`
- wall: 53781.4s
- stages: screen 20241.9s, final 33539.5s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,hread=on` | 0.498 [0.455, 0.541] | 512 | 0.08 / 0.09 | finalist |
| 2 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,deal=block` | 0.494 [0.451, 0.537] | 512 | 0.08 / 0.09 | not in top 1 |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,hread=on` | 0.491 [0.469, 0.512] | 0.499 [0.468, 0.530] | 0.493 [0.476, 0.511] |

verdict: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,hread=on coin flip — main 0.491 [0.469, 0.512], reverse 0.499 [0.468, 0.530], pooled 0.493 [0.476, 0.511] (not gated)

best: h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,hread=on (coin flip)
