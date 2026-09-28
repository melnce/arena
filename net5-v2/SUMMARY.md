# Iteration `net5-v2`

- tag: `net5-v2`
- seeds: 12, 13
- bot: `h0`
- baseline: `h0`
- engine: `1e417633c8d3813204bee75399d05e1c2c2c2730`
- wall: 10880.4s
- stages: data 7047.0s, train 65.2s, yard 3768.1s, summary 0.0s

## data

| run | games | samples | samples/game | first-player | g/s |
|---|---:|---:|---:|---:|---:|
| `data-e0` | 6144 | 515889 | 84.0 | 0.536 | 1.73 |
| `data-e10` | 6144 | 507559 | 82.6 | 0.523 | 1.76 |

## holdout

### `linear`

| source | sign_acc | auc | mse | ≤3 sign/auc/mse | 4–6 sign/auc/mse | ≥7 sign/auc/mse |
|---|---:|---:|---:|---|---|---|
| net | 0.730 | 0.814 | 0.707 | 0.679/0.755/0.805 | 0.733/0.818/0.698 | 0.765/0.849/0.644 |
| v0 | 0.626 | 0.698 | 1058.252 | 0.523/0.598/58.068 | 0.646/0.713/1036.743 | 0.681/0.747/1842.089 |
| search_v | 0.687 | 0.788 | 1775.599 | 0.622/0.682/947.133 | 0.671/0.771/1558.114 | 0.741/0.854/2492.540 |
| eval h0-linear-v1.json (v1 block of v2 rows) | 0.655 | 0.699 | 0.967 | 0.606/0.628/1.073 | 0.661/0.709/0.954 | 0.684/0.738/0.902 |

## yardstick

| model | main | reverse | pooled | sanity | g/s vs h0 | royal-nattui |
|---|---:|---:|---:|---:|---:|---:|
| linear | 0.562 [0.547, 0.578] | 0.573 [0.552, 0.595] | 0.566 [0.554, 0.578] | 1.000 [0.963, 1.000] | 1.64 / 1.87 | 0.540 [0.471, 0.608] |

## verdict

verdict: linear better — main 0.562 [0.547, 0.578], reverse 0.573 [0.552, 0.595], pooled 0.566 [0.554, 0.578] (not gated)
