# Sweep `sweep28b`

- tag: `sweep28b`
- baseline: `h0`
- seed: 3
- engine: `8e66bcf253219f64300c3fb26e2918cf72b2ed81`
- wall: 4017.1s
- stages: screen 593.0s, final 3424.0s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/sweep28/leaf-all4-e2-l2x10.json` | 0.510 [0.467, 0.553] | 512 | 1.71 / 1.67 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep28/leaf-all4-e2-l2x10.json` | 0.518 [0.503, 0.533] | 0.521 [0.499, 0.542] | 0.519 [0.506, 0.531] |

verdict: h0:net=results/sweep28/leaf-all4-e2-l2x10.json unclear (reverse) — main 0.518 [0.503, 0.533], reverse 0.521 [0.499, 0.542], pooled 0.519 [0.506, 0.531] (not gated)

best: h0:net=results/sweep28/leaf-all4-e2-l2x10.json (unclear (reverse))
