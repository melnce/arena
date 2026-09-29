# Sweep `sweep17`

- tag: `sweep17`
- baseline: `h0`
- seed: 1
- engine: `dc868a47b84865c3a811c89dfc03a0ebb3bd019f`
- wall: 7698.5s
- stages: screen 2436.7s, final 5261.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 5 | `h0:net=results/sweep17/leaf-v2data-seed1.json` | 0.502 [0.459, 0.545] | 512 | 1.70 / 1.51 | finalist |
| 4 | `h0:net=results/sweep17/leaf-union-mix.json` | 0.479 [0.436, 0.522] | 512 | 1.73 / 1.51 | finalist |
| 3 | `h0:net=results/sweep17/leaf-v2data-mix.json` | 0.477 [0.434, 0.520] | 512 | 1.72 / 1.51 | finalist |
| 2 | `h0:net=results/sweep17/leaf-net6-mix.json` | 0.455 [0.412, 0.498] | 512 | 1.65 / 1.51 | skipped: interval high 0.50 < 0.50 |
| 1 | `h0:net=results/sweep17/leaf-net6-search.json` | 0.438 [0.395, 0.481] | 512 | 1.66 / 1.51 | skipped: interval high 0.48 < 0.50 |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 3 | `h0:net=results/sweep17/leaf-v2data-mix.json` | 0.477 [0.455, 0.498] | 0.471 [0.440, 0.501] | 0.475 [0.457, 0.492] |
| 4 | `h0:net=results/sweep17/leaf-union-mix.json` | 0.459 [0.437, 0.481] | 0.454 [0.424, 0.485] | 0.457 [0.440, 0.475] |
| 5 | `h0:net=results/sweep17/leaf-v2data-seed1.json` | 0.481 [0.459, 0.503] | 0.464 [0.434, 0.494] | 0.475 [0.458, 0.493] |

verdict: h0:net=results/sweep17/leaf-v2data-mix.json worse — main 0.477 [0.455, 0.498], reverse 0.471 [0.440, 0.501], pooled 0.475 [0.457, 0.492] (not gated)
verdict: h0:net=results/sweep17/leaf-union-mix.json worse — main 0.459 [0.437, 0.481], reverse 0.454 [0.424, 0.485], pooled 0.457 [0.440, 0.475] (not gated)
verdict: h0:net=results/sweep17/leaf-v2data-seed1.json unclear (main, reverse) — main 0.481 [0.459, 0.503], reverse 0.464 [0.434, 0.494], pooled 0.475 [0.458, 0.493] (not gated)

best: h0:net=results/sweep17/leaf-v2data-seed1.json (unclear (main, reverse))
