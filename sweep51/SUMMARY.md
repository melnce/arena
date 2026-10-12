# Sweep `sweep51`

- tag: `sweep51`
- baseline: `h0`
- seed: 1
- engine: `99e07753ecc3c7e513d0f3be2668a73af810b0f7`
- wall: 4795.2s
- stages: screen 599.3s, final 4195.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:mull=results/sweep51/mull-bayes3.json` | 0.539 [0.496, 0.582] | 512 | 1.68 / 1.70 | finalist |

## final

final games per pair: 2 … 33 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:mull=results/sweep51/mull-bayes3.json` | 0.508 [0.494, 0.523] | 0.502 [0.483, 0.521] | 0.506 [0.494, 0.518] |

verdict: h0:mull=results/sweep51/mull-bayes3.json coin flip — main 0.508 [0.494, 0.523], reverse 0.502 [0.483, 0.521], pooled 0.506 [0.494, 0.518] (not gated)

early stop: stopped after 4608/8192 main and 2560/4096 reverse games — not better, pooled ≥ 0.495 settled at P = 0.994 (γ 0.02, thresholds 0.49, 0.495)

best: h0:mull=results/sweep51/mull-bayes3.json (coin flip)
