# Sweep `sweep13`

- tag: `sweep13`
- baseline: `h0`
- seed: 1
- engine: `2e40d4de0a5697678afcb8e7713d5932098f01bb`
- wall: 7858.1s
- stages: screen 1559.2s, final 6298.9s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:mull=results/mull1/table.json` | 0.535 [0.505, 0.566] | 1024 | 1.74 / 1.64 | finalist |
| 2 | `h0:info=open` | 0.488 [0.458, 0.519] | 1024 | 1.72 / 1.64 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:mull=results/mull1/table.json` | 0.521 [0.505, 0.536] | 0.523 [0.501, 0.545] | 0.521 [0.509, 0.534] |
| 2 | `h0:info=open` | 0.497 [0.482, 0.512] | 0.502 [0.481, 0.524] | 0.499 [0.486, 0.511] |

verdict: h0:mull=results/mull1/table.json better — main 0.521 [0.505, 0.536], reverse 0.523 [0.501, 0.545], pooled 0.521 [0.509, 0.534] (not gated)
verdict: h0:info=open coin flip — main 0.497 [0.482, 0.512], reverse 0.502 [0.481, 0.524], pooled 0.499 [0.486, 0.511] (not gated)

best: h0:mull=results/mull1/table.json (better)
