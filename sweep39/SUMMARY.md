# Sweep `sweep39`

- tag: `sweep39`
- baseline: `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000`
- seed: 1
- engine: `1c551eba7078b79d5207b9862336bdf7bec9478b`
- wall: 62600.8s
- stages: screen 15750.1s, final 46850.6s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000` | 0.537 [0.494, 0.580] | 512 | 0.05 / 0.09 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000` | 0.519 [0.497, 0.541] | 0.521 [0.491, 0.552] | 0.520 [0.502, 0.537] |

verdict: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000 coin flip — main 0.519 [0.497, 0.541], reverse 0.521 [0.491, 0.552], pooled 0.520 [0.502, 0.537] (not gated)

best: h0:nodes=64000,horizon=3,k=16,tkill=10000,tkroll=8,hbcheck=2000 (coin flip)
