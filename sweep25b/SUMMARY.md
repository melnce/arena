# Sweep `sweep25b`

- tag: `sweep25b`
- baseline: `h0`
- seed: 2
- engine: `8e66bcf253219f64300c3fb26e2918cf72b2ed81`
- wall: 12691.8s
- stages: screen 1583.6s, final 11108.2s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/sweep25/leaf-all4-e2.json` | 0.516 [0.472, 0.559] | 512 | 1.53 / 1.56 | finalist |
| 2 | `h0:net=results/sweep25/leaf-all4-e4.json` | 0.502 [0.459, 0.545] | 512 | 1.57 / 1.56 | finalist |
| 3 | `h0:net=results/sweep25/leaf-all4-e8.json` | 0.490 [0.447, 0.533] | 512 | 1.59 / 1.56 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep25/leaf-all4-e2.json` | 0.510 [0.494, 0.525] | 0.527 [0.506, 0.549] | 0.515 [0.503, 0.528] |
| 2 | `h0:net=results/sweep25/leaf-all4-e4.json` | 0.482 [0.466, 0.497] | 0.512 [0.490, 0.533] | 0.492 [0.479, 0.504] |
| 3 | `h0:net=results/sweep25/leaf-all4-e8.json` | 0.489 [0.474, 0.504] | 0.491 [0.469, 0.512] | 0.490 [0.477, 0.502] |

verdict: h0:net=results/sweep25/leaf-all4-e2.json unclear (main) — main 0.510 [0.494, 0.525], reverse 0.527 [0.506, 0.549], pooled 0.515 [0.503, 0.528] (not gated)
verdict: h0:net=results/sweep25/leaf-all4-e4.json worse — main 0.482 [0.466, 0.497], reverse 0.512 [0.490, 0.533], pooled 0.492 [0.479, 0.504] (not gated)
verdict: h0:net=results/sweep25/leaf-all4-e8.json coin flip — main 0.489 [0.474, 0.504], reverse 0.491 [0.469, 0.512], pooled 0.490 [0.477, 0.502] (not gated)

best: h0:net=results/sweep25/leaf-all4-e2.json (unclear (main))
