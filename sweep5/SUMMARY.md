# Sweep `sweep5`

- tag: `sweep5`
- baseline: `h0`
- seed: 6
- engine: `f7b0a614546091244ddedadd9a54f6e967d6e384`
- wall: 13108.6s
- stages: screen 3663.2s, final 9445.4s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 1 | `h0:olethal=1` | 0.521 [0.491, 0.552] | 1024 | 2.46 / 2.90 | finalist |
| 2 | `h0:olethal=1,osteps=6` | 0.517 [0.486, 0.547] | 1024 | 2.40 / 2.90 | finalist |
| 6 | `h0:osteps=6` | 0.517 [0.486, 0.547] | 1024 | 1.33 / 2.90 | not in top 2 |
| 5 | `h0:odepth=2,obeam=3` | 0.373 [0.344, 0.403] | 1024 | 1.35 / 2.90 | skipped: interval high 0.40 < 0.50 |
| 3 | `h0:odepth=1` | 0.366 [0.337, 0.396] | 1024 | 2.15 / 2.90 | skipped: interval high 0.40 < 0.50 |
| 4 | `h0:odepth=1,obeam=5` | 0.357 [0.329, 0.387] | 1024 | 2.04 / 2.90 | skipped: interval high 0.39 < 0.50 |

## final

| index | spec | main | reverse |
|---|---|---:|---:|
| 1 | `h0:olethal=1` | 0.512 [0.497, 0.527] | 0.512 [0.491, 0.534] |
| 2 | `h0:olethal=1,osteps=6` | 0.526 [0.511, 0.541] | 0.538 [0.516, 0.559] |

verdict: h0:olethal=1 coin flip — main 0.512 [0.497, 0.527], reverse 0.512 [0.491, 0.534]
verdict: h0:olethal=1,osteps=6 better — main 0.526 [0.511, 0.541], reverse 0.538 [0.516, 0.559]

best: h0:olethal=1,osteps=6 (better)
