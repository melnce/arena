# Sweep `sweep7`

- tag: `sweep7`
- baseline: `h0:olethal=1,osteps=6`
- seed: 8
- engine: `737dbf8232afc16f06d16299bc5cfa5995d0ba30`
- wall: 19148.9s
- stages: screen 5266.0s, final 13882.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 5 | `h0:olethal=1,osteps=6,nodes=6000` | 0.523 [0.493, 0.554] | 1024 | 0.54 / 2.17 | finalist |
| 1 | `h0:olethal=1,osteps=6,wv=60` | 0.508 [0.477, 0.538] | 1024 | 1.83 / 2.17 | finalist |
| 4 | `h0:olethal=1,osteps=6,pess=0.5` | 0.496 [0.466, 0.527] | 1024 | 1.16 / 2.17 | not in top 2 |
| 2 | `h0:olethal=1,osteps=6,wv=40` | 0.422 [0.392, 0.452] | 1024 | 1.49 / 2.17 | skipped: interval high 0.45 < 0.50 |
| 3 | `h0:olethal=1,osteps=6,wv=20` | 0.354 [0.325, 0.383] | 1024 | 1.48 / 2.17 | skipped: interval high 0.38 < 0.50 |

## final

| index | spec | main | reverse |
|---|---|---:|---:|
| 1 | `h0:olethal=1,osteps=6,wv=60` | 0.497 [0.482, 0.512] | 0.493 [0.472, 0.515] |
| 5 | `h0:olethal=1,osteps=6,nodes=6000` | 0.521 [0.505, 0.536] | 0.512 [0.491, 0.534] |

verdict: h0:olethal=1,osteps=6,wv=60 coin flip — main 0.497 [0.482, 0.512], reverse 0.493 [0.472, 0.515]
verdict: h0:olethal=1,osteps=6,nodes=6000 unclear (reverse) — main 0.521 [0.505, 0.536], reverse 0.512 [0.491, 0.534]

best: h0:olethal=1,osteps=6,nodes=6000 (unclear (reverse))
