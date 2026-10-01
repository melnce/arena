# Sweep `sweep30`

- tag: `sweep30`
- baseline: `h0`
- seed: 1
- engine: `a69b248eec99325b639198314c811cf89c84630c`
- wall: 7821.0s
- stages: screen 1748.5s, final 6072.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:omacro=1` | 0.521 [0.478, 0.564] | 512 | 1.48 / 1.53 | finalist |
| 1 | `h0:okill=7` | 0.504 [0.461, 0.547] | 512 | 1.40 / 1.53 | finalist |
| 3 | `h0:okill=7,omacro=1` | 0.504 [0.461, 0.547] | 512 | 1.38 / 1.53 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:okill=7` | 0.490 [0.469, 0.512] | 0.487 [0.457, 0.518] | 0.489 [0.472, 0.507] |
| 2 | `h0:omacro=1` | 0.501 [0.480, 0.523] | 0.507 [0.476, 0.537] | 0.503 [0.486, 0.521] |
| 3 | `h0:okill=7,omacro=1` | 0.485 [0.464, 0.507] | 0.488 [0.458, 0.519] | 0.486 [0.469, 0.504] |

verdict: h0:okill=7 coin flip — main 0.490 [0.469, 0.512], reverse 0.487 [0.457, 0.518], pooled 0.489 [0.472, 0.507] (not gated)
verdict: h0:omacro=1 coin flip — main 0.501 [0.480, 0.523], reverse 0.507 [0.476, 0.537], pooled 0.503 [0.486, 0.521] (not gated)
verdict: h0:okill=7,omacro=1 coin flip — main 0.485 [0.464, 0.507], reverse 0.488 [0.458, 0.519], pooled 0.486 [0.469, 0.504] (not gated)

best: h0:omacro=1 (coin flip)
