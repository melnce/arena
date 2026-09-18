# Sweep `sweep4`

- tag: `sweep4`
- baseline: `h0`
- seed: 4
- engine: `a17c6c08922faca6943e7b5f5d674d7e13dbcdd6`
- wall: 7379.0s
- stages: screen 2283.2s, final 5095.8s, summary 0.0s

## screen

| index | spec | rate | games | g/s vs baseline | decision |
|---|---:|---|---:|---|---|
| 2 | `h0:alloc=fair,nodes=6000` | 0.605 [0.575, 0.635] | 1024 | 1.31 / 2.53 | finalist |
| 1 | `h0:alloc=fair` | 0.587 [0.556, 0.617] | 1024 | 2.89 / 2.53 | finalist |
| 3 | `h0:alloc=fair,depth=4` | 0.560 [0.529, 0.590] | 1024 | 3.05 / 2.53 | not in top 2 |
| 4 | `h0:alloc=fair,k=8` | 0.560 [0.529, 0.590] | 1024 | 2.46 / 2.53 | not in top 2 |

## final

| index | spec | main | reverse |
|---|---|---:|---:|
| 1 | `h0:alloc=fair` | 0.563 [0.548, 0.579] | 0.538 [0.516, 0.559] |
| 2 | `h0:alloc=fair,nodes=6000` | 0.583 [0.568, 0.598] | 0.552 [0.531, 0.574] |

verdict: h0:alloc=fair better — main 0.563 [0.548, 0.579], reverse 0.538 [0.516, 0.559]
verdict: h0:alloc=fair,nodes=6000 better — main 0.583 [0.568, 0.598], reverse 0.552 [0.531, 0.574]

best: h0:alloc=fair,nodes=6000 (better)
