# Sweep `sweep28`

- tag: `sweep28`
- baseline: `h0`
- seed: 1
- engine: `8e66bcf253219f64300c3fb26e2918cf72b2ed81`
- wall: 7291.5s
- stages: screen 1615.7s, final 5675.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:net=results/sweep28/leaf-all4-e2-l2x10.json` | 0.535 [0.492, 0.578] | 512 | 1.57 / 1.51 | finalist |
| 1 | `h0:net=results/sweep28/leaf-all4-e1.json` | 0.512 [0.468, 0.555] | 512 | 1.63 / 1.51 | finalist |
| 3 | `h0:net=results/sweep28/leaf-all4-e8-l2x10.json` | 0.504 [0.461, 0.547] | 512 | 1.48 / 1.51 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep28/leaf-all4-e1.json` | 0.500 [0.478, 0.522] | 0.512 [0.481, 0.542] | 0.504 [0.486, 0.522] |
| 2 | `h0:net=results/sweep28/leaf-all4-e2-l2x10.json` | 0.518 [0.496, 0.540] | 0.528 [0.498, 0.559] | 0.521 [0.504, 0.539] |
| 3 | `h0:net=results/sweep28/leaf-all4-e8-l2x10.json` | 0.497 [0.475, 0.518] | 0.482 [0.452, 0.513] | 0.492 [0.474, 0.510] |

verdict: h0:net=results/sweep28/leaf-all4-e1.json coin flip — main 0.500 [0.478, 0.522], reverse 0.512 [0.481, 0.542], pooled 0.504 [0.486, 0.522] (not gated)
verdict: h0:net=results/sweep28/leaf-all4-e2-l2x10.json coin flip — main 0.518 [0.496, 0.540], reverse 0.528 [0.498, 0.559], pooled 0.521 [0.504, 0.539] (not gated)
verdict: h0:net=results/sweep28/leaf-all4-e8-l2x10.json coin flip — main 0.497 [0.475, 0.518], reverse 0.482 [0.452, 0.513], pooled 0.492 [0.474, 0.510] (not gated)

best: h0:net=results/sweep28/leaf-all4-e2-l2x10.json (coin flip)
