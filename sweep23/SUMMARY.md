# Sweep `sweep23`

- tag: `sweep23`
- baseline: `h0`
- seed: 1
- engine: `2d12e25f27ce4276ef6274be8b52e30d1e510f6b`
- wall: 7179.1s
- stages: screen 1648.0s, final 5531.1s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/sweep23/leaf-h0e8-s1.json` | 0.545 [0.502, 0.588] | 512 | 1.54 / 1.46 | finalist |
| 2 | `h0:net=results/sweep23/leaf-h0e8-s2.json` | 0.529 [0.486, 0.572] | 512 | 1.46 / 1.46 | finalist |
| 3 | `h0:net=results/sweep23/leaf-r0.json` | 0.504 [0.461, 0.547] | 512 | 1.69 / 1.46 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep23/leaf-h0e8-s1.json` | 0.500 [0.478, 0.521] | 0.515 [0.484, 0.545] | 0.505 [0.487, 0.522] |
| 2 | `h0:net=results/sweep23/leaf-h0e8-s2.json` | 0.497 [0.475, 0.519] | 0.482 [0.452, 0.513] | 0.492 [0.475, 0.510] |
| 3 | `h0:net=results/sweep23/leaf-r0.json` | 0.496 [0.474, 0.517] | 0.509 [0.478, 0.539] | 0.500 [0.482, 0.518] |

verdict: h0:net=results/sweep23/leaf-h0e8-s1.json coin flip — main 0.500 [0.478, 0.521], reverse 0.515 [0.484, 0.545], pooled 0.505 [0.487, 0.522] (not gated)
verdict: h0:net=results/sweep23/leaf-h0e8-s2.json coin flip — main 0.497 [0.475, 0.519], reverse 0.482 [0.452, 0.513], pooled 0.492 [0.475, 0.510] (not gated)
verdict: h0:net=results/sweep23/leaf-r0.json coin flip — main 0.496 [0.474, 0.517], reverse 0.509 [0.478, 0.539], pooled 0.500 [0.482, 0.518] (not gated)

best: h0:net=results/sweep23/leaf-h0e8-s1.json (coin flip)
