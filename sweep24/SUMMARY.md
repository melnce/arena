# Sweep `sweep24`

- tag: `sweep24`
- baseline: `h0:nodes=6000`
- seed: 1
- engine: `15b845c5df90eab1734d4f17cf5aabc1d506fe42`
- wall: 2082.8s
- stages: screen 2082.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:nodes=6000,info=all,olsolve=200` | 0.438 [0.395, 0.481] | 512 | 0.57 / 0.79 | skipped: interval high 0.48 < 0.50 |
| 1 | `h0:nodes=6000,info=all` | 0.428 [0.386, 0.471] | 512 | 2.04 / 0.79 | skipped: interval high 0.47 < 0.50 |

## final

| index | spec | main | reverse | pooled |
|---|---|---:|---:|---:|

best: none
