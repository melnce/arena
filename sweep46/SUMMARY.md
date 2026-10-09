# Sweep `sweep46`

- tag: `sweep46`
- baseline: `h0`
- seed: 1
- engine: `ed574a41fc9b6e49aeb008a4b835510c15c0c739`
- wall: 7910.1s
- stages: screen 634.2s, final 7275.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:mull=results/sweep46/mull-bayes.json` | 0.500 [0.457, 0.543] | 512 | 1.60 / 1.63 | finalist |

## final

final games per pair: 2 … 33 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:mull=results/sweep46/mull-bayes.json` | 0.508 [0.497, 0.520] | 0.512 [0.496, 0.528] | 0.509 [0.500, 0.519] |

verdict: h0:mull=results/sweep46/mull-bayes.json coin flip — main 0.508 [0.497, 0.520], reverse 0.512 [0.496, 0.528], pooled 0.509 [0.500, 0.519] (not gated)

early stop: stopped after 7168/8192 main and 3584/4096 reverse games — not better, pooled ≥ 0.505 settled at P = 0.982 (γ 0.02, thresholds 0.5, 0.505)

best: h0:mull=results/sweep46/mull-bayes.json (coin flip)
