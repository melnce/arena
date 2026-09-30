# Sweep `sweep21`

- tag: `sweep21`
- baseline: `h0:nodes=16000,net=engine/models/h0-linear-v1.json`
- seed: 1
- engine: `dc868a47b84865c3a811c89dfc03a0ebb3bd019f`
- wall: 8441.8s
- stages: screen 3451.0s, final 4990.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=16000,horizon=3` | 0.627 [0.584, 0.668] | 512 | 0.23 / 0.39 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=16000,horizon=3` | 0.609 [0.579, 0.639] | 0.576 [0.533, 0.618] | 0.598 [0.574, 0.623] |

verdict: h0:nodes=16000,horizon=3 better — main 0.609 [0.579, 0.639], reverse 0.576 [0.533, 0.618], pooled 0.598 [0.574, 0.623] (not gated)

best: h0:nodes=16000,horizon=3 (better)
