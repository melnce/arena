# Sweep `sweep35`

- tag: `sweep35`
- baseline: `h0:nodes=32000,horizon=3,k=8`
- seed: 1
- engine: `d920fe153ecc858619a5d0bc42cb4348012c1ad1`
- wall: 37363.7s
- stages: screen 10544.1s, final 26819.6s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8,hbcheck=2000` | 0.512 [0.468, 0.555] | 512 | 0.11 / 0.08 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8,hbcheck=2000` | 0.522 [0.500, 0.544] | 0.508 [0.477, 0.538] | 0.517 [0.500, 0.535] |

verdict: h0:nodes=32000,horizon=3,k=8,hbcheck=2000 unclear (reverse) — main 0.522 [0.500, 0.544], reverse 0.508 [0.477, 0.538], pooled 0.517 [0.500, 0.535] (not gated)

best: h0:nodes=32000,horizon=3,k=8,hbcheck=2000 (unclear (reverse))
