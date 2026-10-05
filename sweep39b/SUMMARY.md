# Sweep `sweep39b`

- tag: `sweep39b`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000`
- seed: 2
- engine: `1c551eba7078b79d5207b9862336bdf7bec9478b`
- wall: 59759.3s
- stages: screen 14946.0s, final 44813.3s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000` | 0.525 [0.482, 0.568] | 512 | 0.06 / 0.09 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000` | 0.516 [0.494, 0.538] | 0.497 [0.467, 0.528] | 0.510 [0.492, 0.527] |

verdict: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000 coin flip — main 0.516 [0.494, 0.538], reverse 0.497 [0.467, 0.528], pooled 0.510 [0.492, 0.527] (not gated)

best: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000 (coin flip)
