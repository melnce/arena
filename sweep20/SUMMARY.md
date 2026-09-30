# Sweep `sweep20`

- tag: `sweep20`
- baseline: `h0`
- seed: 1
- engine: `dc868a47b84865c3a811c89dfc03a0ebb3bd019f`
- wall: 11186.0s
- stages: screen 2398.6s, final 8787.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 5 | `h0:net=results/sweep20/leaf-soup-s0-4.json` | 0.527 [0.484, 0.570] | 512 | 1.70 / 1.66 | finalist |
| 4 | `h0:net=results/sweep20/leaf-soup-s1-4.json` | 0.496 [0.453, 0.539] | 512 | 1.62 / 1.66 | finalist |
| 3 | `h0:net=results/sweep20/leaf-v2data-seed4.json` | 0.494 [0.451, 0.537] | 512 | 1.73 / 1.66 | finalist |
| 2 | `h0:net=results/sweep20/leaf-v2data-seed3.json` | 0.482 [0.439, 0.526] | 512 | 1.66 / 1.66 | finalist |
| 1 | `h0:net=results/sweep20/leaf-v2data-seed2.json` | 0.469 [0.426, 0.512] | 512 | 1.74 / 1.66 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep20/leaf-v2data-seed2.json` | 0.456 [0.434, 0.477] | 0.488 [0.458, 0.519] | 0.466 [0.449, 0.484] |
| 2 | `h0:net=results/sweep20/leaf-v2data-seed3.json` | 0.469 [0.448, 0.491] | 0.469 [0.438, 0.499] | 0.469 [0.451, 0.487] |
| 3 | `h0:net=results/sweep20/leaf-v2data-seed4.json` | 0.484 [0.463, 0.506] | 0.510 [0.479, 0.540] | 0.493 [0.475, 0.511] |
| 4 | `h0:net=results/sweep20/leaf-soup-s1-4.json` | 0.489 [0.468, 0.511] | 0.480 [0.450, 0.511] | 0.486 [0.469, 0.504] |
| 5 | `h0:net=results/sweep20/leaf-soup-s0-4.json` | 0.512 [0.491, 0.534] | 0.507 [0.476, 0.537] | 0.510 [0.493, 0.528] |

verdict: h0:net=results/sweep20/leaf-v2data-seed2.json worse — main 0.456 [0.434, 0.477], reverse 0.488 [0.458, 0.519], pooled 0.466 [0.449, 0.484] (not gated)
verdict: h0:net=results/sweep20/leaf-v2data-seed3.json worse — main 0.469 [0.448, 0.491], reverse 0.469 [0.438, 0.499], pooled 0.469 [0.451, 0.487] (not gated)
verdict: h0:net=results/sweep20/leaf-v2data-seed4.json coin flip — main 0.484 [0.463, 0.506], reverse 0.510 [0.479, 0.540], pooled 0.493 [0.475, 0.511] (not gated)
verdict: h0:net=results/sweep20/leaf-soup-s1-4.json coin flip — main 0.489 [0.468, 0.511], reverse 0.480 [0.450, 0.511], pooled 0.486 [0.469, 0.504] (not gated)
verdict: h0:net=results/sweep20/leaf-soup-s0-4.json coin flip — main 0.512 [0.491, 0.534], reverse 0.507 [0.476, 0.537], pooled 0.510 [0.493, 0.528] (not gated)

best: h0:net=results/sweep20/leaf-soup-s0-4.json (coin flip)
