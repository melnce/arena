# Sweep `sweep26`

- tag: `sweep26`
- baseline: `h0:nodes=6000,info=all`
- seed: 1
- engine: `8e66bcf253219f64300c3fb26e2918cf72b2ed81`
- wall: 16695.2s
- stages: screen 4051.7s, final 12643.5s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:nodes=6000` | 0.514 [0.470, 0.557] | 512 | 0.72 / 0.57 | finalist |
| 2 | `h0:nodes=6000,info=all,olsolve=200` | 0.508 [0.465, 0.551] | 512 | 0.24 / 0.57 | finalist |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:nodes=6000` | 0.507 [0.485, 0.528] | 0.498 [0.467, 0.529] | 0.504 [0.486, 0.522] |
| 2 | `h0:nodes=6000,info=all,olsolve=200` | 0.493 [0.471, 0.514] | 0.467 [0.436, 0.497] | 0.484 [0.466, 0.502] |

verdict: h0:nodes=6000 coin flip — main 0.507 [0.485, 0.528], reverse 0.498 [0.467, 0.529], pooled 0.504 [0.486, 0.522] (not gated)
verdict: h0:nodes=6000,info=all,olsolve=200 unclear (main, reverse) — main 0.493 [0.471, 0.514], reverse 0.467 [0.436, 0.497], pooled 0.484 [0.466, 0.502] (not gated)

best: h0:nodes=6000 (coin flip)
