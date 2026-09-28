# Sweep `sweep15`

- tag: `sweep15`
- baseline: `h0:nodes=16000`
- seed: 1
- engine: `71127bf5126601fc3b05c65237cc56990a4257f0`
- wall: 29076.6s
- stages: screen 12777.9s, final 16298.7s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 4 | `h0:nodes=16000,horizon=3` | 0.545 [0.502, 0.588] | 512 | 0.24 / 0.38 | finalist |
| 1 | `h0:nodes=16000,beam=2` | 0.541 [0.498, 0.584] | 512 | 0.62 / 0.38 | finalist |
| 3 | `h0:nodes=16000,horizon=2` | 0.512 [0.468, 0.555] | 512 | 0.26 / 0.38 | not in top 2 |
| 5 | `h0:nodes=48000` | 0.506 [0.463, 0.549] | 512 | 0.19 / 0.38 | not in top 2 |
| 2 | `h0:nodes=16000,beam=2,horizon=2` | 0.488 [0.445, 0.532] | 512 | 0.49 / 0.38 | not in top 2 |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=16000,beam=2` | 0.503 [0.481, 0.525] | 0.472 [0.441, 0.502] | 0.493 [0.475, 0.510] |
| 4 | `h0:nodes=16000,horizon=3` | 0.518 [0.496, 0.539] | 0.522 [0.492, 0.553] | 0.519 [0.502, 0.537] |

verdict: h0:nodes=16000,beam=2 coin flip — main 0.503 [0.481, 0.525], reverse 0.472 [0.441, 0.502], pooled 0.493 [0.475, 0.510] (not gated)
verdict: h0:nodes=16000,horizon=3 coin flip — main 0.518 [0.496, 0.539], reverse 0.522 [0.492, 0.553], pooled 0.519 [0.502, 0.537] (not gated)

best: h0:nodes=16000,horizon=3 (coin flip)
