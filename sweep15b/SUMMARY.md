# Sweep `sweep15b`

- tag: `sweep15b`
- baseline: `h0:nodes=16000,net=engine/models/h0-linear-v2.json`
- seed: 2
- engine: `71127bf5126601fc3b05c65237cc56990a4257f0`
- wall: 23988.2s
- stages: screen 3548.0s, final 20440.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=16000,horizon=3,net=engine/models/h0-linear-v2.json` | 0.541 [0.498, 0.584] | 512 | 0.24 / 0.35 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=16000,horizon=3,net=engine/models/h0-linear-v2.json` | 0.520 [0.504, 0.535] | 0.523 [0.502, 0.545] | 0.521 [0.508, 0.533] |

verdict: h0:nodes=16000,horizon=3,net=engine/models/h0-linear-v2.json better — main 0.520 [0.504, 0.535], reverse 0.523 [0.502, 0.545], pooled 0.521 [0.508, 0.533] (not gated)

best: h0:nodes=16000,horizon=3,net=engine/models/h0-linear-v2.json (better)
