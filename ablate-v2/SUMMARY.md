# Sweep `ablate-v2`

- tag: `ablate-v2`
- baseline: `h0:net=results/net5-v2/linear.json`
- seed: 1
- engine: `063bdd4b6d4c1061c4533d390be217779a3c1b48`
- wall: 6887.3s
- stages: screen 1537.8s, final 5349.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/ablate/v2-no18.json` | 0.510 [0.467, 0.553] | 512 | 1.71 / 1.60 | finalist |
| 3 | `h0:net=results/ablate/v2-nobpp.json` | 0.508 [0.465, 0.551] | 512 | 1.63 / 1.60 | finalist |
| 2 | `h0:net=results/ablate/v2-nocost.json` | 0.502 [0.459, 0.545] | 512 | 1.69 / 1.60 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/ablate/v2-no18.json` | 0.489 [0.468, 0.511] | 0.514 [0.483, 0.544] | 0.497 [0.480, 0.515] |
| 2 | `h0:net=results/ablate/v2-nocost.json` | 0.487 [0.465, 0.508] | 0.504 [0.473, 0.534] | 0.493 [0.475, 0.510] |
| 3 | `h0:net=results/ablate/v2-nobpp.json` | 0.496 [0.474, 0.518] | 0.515 [0.484, 0.545] | 0.502 [0.485, 0.520] |

verdict: h0:net=results/ablate/v2-no18.json coin flip — main 0.489 [0.468, 0.511], reverse 0.514 [0.483, 0.544], pooled 0.497 [0.480, 0.515] (not gated)
verdict: h0:net=results/ablate/v2-nocost.json coin flip — main 0.487 [0.465, 0.508], reverse 0.504 [0.473, 0.534], pooled 0.493 [0.475, 0.510] (not gated)
verdict: h0:net=results/ablate/v2-nobpp.json coin flip — main 0.496 [0.474, 0.518], reverse 0.515 [0.484, 0.545], pooled 0.502 [0.485, 0.520] (not gated)

best: h0:net=results/ablate/v2-nobpp.json (coin flip)
