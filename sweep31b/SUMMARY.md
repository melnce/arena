# Sweep `sweep31b`

- tag: `sweep31b`
- baseline: `h0:nodes=16000,horizon=3,net=engine/models/h0-linear-v2.json`
- seed: 2
- engine: `d5f4310421b433cadf8711d24e7ebc726102c1c3`
- wall: 29182.3s
- stages: screen 7294.1s, final 21888.1s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,net=engine/models/h0-linear-v2.json` | 0.541 [0.498, 0.584] | 512 | 0.10 / 0.22 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,net=engine/models/h0-linear-v2.json` | 0.529 [0.508, 0.551] | 0.527 [0.497, 0.558] | 0.529 [0.511, 0.546] |

verdict: h0:nodes=32000,horizon=3,k=8,net=engine/models/h0-linear-v2.json unclear (reverse) — main 0.529 [0.508, 0.551], reverse 0.527 [0.497, 0.558], pooled 0.529 [0.511, 0.546] (not gated)

best: h0:nodes=32000,horizon=3,k=8,net=engine/models/h0-linear-v2.json (unclear (reverse))
