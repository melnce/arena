# Sweep `sweep16`

- tag: `sweep16`
- baseline: `h0`
- seed: 1
- engine: `71127bf5126601fc3b05c65237cc56990a4257f0`
- wall: 11026.0s
- stages: screen 2342.4s, final 8683.6s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 3 | `h0:net=results/sweep16/leaf-v2-t14.json` | 0.510 [0.467, 0.553] | 512 | 1.69 / 1.64 | finalist |
| 2 | `h0:net=results/sweep16/leaf-v2-t071.json` | 0.504 [0.461, 0.547] | 512 | 1.65 / 1.64 | finalist |
| 5 | `h0:net=results/sweep16/leaf-net6-e20.json` | 0.504 [0.461, 0.547] | 512 | 1.76 / 1.64 | finalist |
| 1 | `h0:net=results/sweep16/leaf-net6-t14.json` | 0.479 [0.436, 0.522] | 512 | 1.85 / 1.64 | finalist |
| 4 | `h0:net=results/sweep16/leaf-union.json` | 0.467 [0.424, 0.510] | 512 | 1.84 / 1.64 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep16/leaf-net6-t14.json` | 0.470 [0.448, 0.491] | 0.446 [0.416, 0.477] | 0.462 [0.444, 0.480] |
| 2 | `h0:net=results/sweep16/leaf-v2-t071.json` | 0.500 [0.478, 0.522] | 0.515 [0.484, 0.545] | 0.505 [0.487, 0.523] |
| 3 | `h0:net=results/sweep16/leaf-v2-t14.json` | 0.490 [0.469, 0.512] | 0.512 [0.481, 0.542] | 0.497 [0.480, 0.515] |
| 4 | `h0:net=results/sweep16/leaf-union.json` | 0.471 [0.449, 0.492] | 0.442 [0.412, 0.473] | 0.461 [0.444, 0.479] |
| 5 | `h0:net=results/sweep16/leaf-net6-e20.json` | 0.467 [0.446, 0.489] | 0.457 [0.427, 0.488] | 0.464 [0.446, 0.482] |

verdict: h0:net=results/sweep16/leaf-net6-t14.json worse — main 0.470 [0.448, 0.491], reverse 0.446 [0.416, 0.477], pooled 0.462 [0.444, 0.480] (not gated)
verdict: h0:net=results/sweep16/leaf-v2-t071.json coin flip — main 0.500 [0.478, 0.522], reverse 0.515 [0.484, 0.545], pooled 0.505 [0.487, 0.523] (not gated)
verdict: h0:net=results/sweep16/leaf-v2-t14.json coin flip — main 0.490 [0.469, 0.512], reverse 0.512 [0.481, 0.542], pooled 0.497 [0.480, 0.515] (not gated)
verdict: h0:net=results/sweep16/leaf-union.json worse — main 0.471 [0.449, 0.492], reverse 0.442 [0.412, 0.473], pooled 0.461 [0.444, 0.479] (not gated)
verdict: h0:net=results/sweep16/leaf-net6-e20.json worse — main 0.467 [0.446, 0.489], reverse 0.457 [0.427, 0.488], pooled 0.464 [0.446, 0.482] (not gated)

best: h0:net=results/sweep16/leaf-v2-t071.json (coin flip)
