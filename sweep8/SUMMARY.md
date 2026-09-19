# Sweep `sweep8`

- tag: `sweep8`
- baseline: `h0:olethal=1,osteps=6`
- seed: 9
- engine: `414df531475fdda8b4465b5b6817d96a8e2792f4`
- wall: 2143.0s
- stages: screen 728.6s, final 1414.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:olethal=1,osteps=6,info=fair` | 0.531 [0.461, 0.599] | 196 | 1.20 / 1.19 | finalist |
| 3 | `h0:olethal=1,osteps=6,info=all` | 0.510 [0.441, 0.579] | 196 | 1.51 / 1.19 | finalist |
| 1 | `h0:olethal=1,osteps=6,oevo=1` | 0.500 [0.431, 0.569] | 196 | 1.27 / 1.19 | not in top 2 |
| 4 | `h0:olethal=1,osteps=6,info=fair,k=8` | 0.474 [0.406, 0.544] | 196 | 1.08 / 1.19 | not in top 2 |

## final

| index | spec | main | reverse |
|---|---|---:|---:|
| 2 | `h0:olethal=1,osteps=6,info=fair` | 0.509 [0.474, 0.544] | 0.500 [0.451, 0.549] |
| 3 | `h0:olethal=1,osteps=6,info=all` | 0.490 [0.455, 0.525] | 0.485 [0.436, 0.534] |

verdict: h0:olethal=1,osteps=6,info=fair coin flip — main 0.509 [0.474, 0.544], reverse 0.500 [0.451, 0.549]
verdict: h0:olethal=1,osteps=6,info=all coin flip — main 0.490 [0.455, 0.525], reverse 0.485 [0.436, 0.534]

best: h0:olethal=1,osteps=6,info=fair (coin flip)
