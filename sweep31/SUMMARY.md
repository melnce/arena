# Sweep `sweep31`

- tag: `sweep31`
- baseline: `h0:nodes=16000,horizon=3`
- seed: 1
- engine: `a69b248eec99325b639198314c811cf89c84630c`
- wall: 28588.9s
- stages: screen 7803.4s, final 20785.5s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=32000,horizon=3,k=8` | 0.529 [0.486, 0.572] | 512 | 0.09 / 0.22 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=32000,horizon=3,k=8` | 0.526 [0.505, 0.548] | 0.518 [0.487, 0.548] | 0.523 [0.506, 0.541] |

verdict: h0:nodes=32000,horizon=3,k=8 unclear (reverse) — main 0.526 [0.505, 0.548], reverse 0.518 [0.487, 0.548], pooled 0.523 [0.506, 0.541] (not gated)

best: h0:nodes=32000,horizon=3,k=8 (unclear (reverse))
