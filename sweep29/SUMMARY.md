# Sweep `sweep29`

- tag: `sweep29`
- baseline: `h0:nodes=16000,horizon=3`
- seed: 1
- engine: `8e66bcf253219f64300c3fb26e2918cf72b2ed81`
- wall: 6370.7s
- stages: screen 698.4s, final 5672.3s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=16000,horizon=3,info=all,k=1` | 0.425 [0.285, 0.578] | 40 | 0.41 / 0.14 | finalist |
| 2 | `h0:nodes=16000,horizon=3,info=all` | 0.425 [0.285, 0.578] | 40 | 0.20 / 0.14 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=16000,horizon=3,info=all,k=1` | 0.432 [0.385, 0.481] | 0.463 [0.414, 0.511] | 0.448 [0.413, 0.482] |
| 2 | `h0:nodes=16000,horizon=3,info=all` | 0.505 [0.456, 0.554] | 0.450 [0.402, 0.499] | 0.477 [0.443, 0.512] |

verdict: h0:nodes=16000,horizon=3,info=all,k=1 worse — main 0.432 [0.385, 0.481], reverse 0.463 [0.414, 0.511], pooled 0.448 [0.413, 0.482] (not gated)
verdict: h0:nodes=16000,horizon=3,info=all unclear (main, reverse) — main 0.505 [0.456, 0.554], reverse 0.450 [0.402, 0.499], pooled 0.477 [0.443, 0.512] (not gated)

best: h0:nodes=16000,horizon=3,info=all (unclear (main, reverse))
