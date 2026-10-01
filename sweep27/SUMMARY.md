# Sweep `sweep27`

- tag: `sweep27`
- baseline: `h0:nodes=12000`
- seed: 1
- engine: `8e66bcf253219f64300c3fb26e2918cf72b2ed81`
- wall: 14175.3s
- stages: screen 3681.1s, final 10494.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=24000,k=8` | 0.547 [0.504, 0.589] | 512 | 0.20 / 0.42 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=24000,k=8` | 0.527 [0.506, 0.549] | 0.513 [0.482, 0.543] | 0.522 [0.505, 0.540] |

verdict: h0:nodes=24000,k=8 unclear (reverse) — main 0.527 [0.506, 0.549], reverse 0.513 [0.482, 0.543], pooled 0.522 [0.505, 0.540] (not gated)

best: h0:nodes=24000,k=8 (unclear (reverse))
