# Sweep `sweep8b`

- tag: `sweep8b`
- baseline: `h0:olethal=1,osteps=6`
- seed: 9
- engine: `57395371b8e4b114479441b1754776f1e976e920`
- wall: 9917.4s
- stages: screen 2824.6s, final 7092.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:olethal=1,osteps=6,oevo=1` | 0.534 [0.503, 0.564] | 1029 | 1.71 / 1.72 | finalist |
| 2 | `h0:olethal=1,osteps=6,info=fair` | 0.505 [0.475, 0.536] | 1029 | 1.83 / 1.72 | finalist |
| 3 | `h0:olethal=1,osteps=6,info=all` | 0.503 [0.473, 0.534] | 1029 | 2.71 / 1.72 | not in top 2 |
| 4 | `h0:olethal=1,osteps=6,info=fair,k=8` | 0.476 [0.446, 0.507] | 1029 | 1.47 / 1.72 | not in top 2 |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|
| 1 | `h0:olethal=1,osteps=6,oevo=1` | 0.532 [0.516, 0.547] | 0.519 [0.497, 0.540] | 0.527 [0.515, 0.540] |
| 2 | `h0:olethal=1,osteps=6,info=fair` | 0.516 [0.500, 0.531] | 0.508 [0.486, 0.529] | 0.513 [0.500, 0.525] |

verdict: h0:olethal=1,osteps=6,oevo=1 unclear (reverse) — main 0.532 [0.516, 0.547], reverse 0.519 [0.497, 0.540], pooled 0.527 [0.515, 0.540] (not gated)
verdict: h0:olethal=1,osteps=6,info=fair unclear (reverse) — main 0.516 [0.500, 0.531], reverse 0.508 [0.486, 0.529], pooled 0.513 [0.500, 0.525] (not gated)

best: h0:olethal=1,osteps=6,oevo=1 (unclear (reverse))
