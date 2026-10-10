# Sweep `sweep47`

- tag: `sweep47`
- baseline: `h0`
- seed: 1
- engine: `ed574a41fc9b6e49aeb008a4b835510c15c0c739`
- wall: 1830.4s
- stages: screen 605.9s, final 1224.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:net=results/sweep47/leaf-v3-e3-net11.json` | 0.523 [0.480, 0.566] | 512 | 1.64 / 1.68 | finalist |

## final

final games per pair: 2 … 9 (after the screen's 2 games per pair)

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:net=results/sweep47/leaf-v3-e3-net11.json` | 0.498 [0.473, 0.523] | 0.508 [0.465, 0.551] | 0.500 [0.479, 0.522] |

verdict: h0:net=results/sweep47/leaf-v3-e3-net11.json coin flip — main 0.498 [0.473, 0.523], reverse 0.508 [0.465, 0.551], pooled 0.500 [0.479, 0.522] (not gated)

early stop: stopped after 1536/2048 main and 512/1024 reverse games — not better, pooled < 0.515 settled at P = 0.980 (γ 0.02, thresholds 0.515)

best: h0:net=results/sweep47/leaf-v3-e3-net11.json (coin flip)
