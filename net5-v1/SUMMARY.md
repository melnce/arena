# Iteration `net5-v1`

- tag: `net5-v1`
- seeds: 12, 13
- bot: `h0:net=engine/models/h0-linear-v1.json`
- baseline: `h0:net=engine/models/h0-linear-v1.json`
- engine: `515995da3468a36be0c2da42760a985cc46539dc`
- wall: 10409.8s
- stages: data 6636.8s, train 47.8s, yard 3725.2s, summary 0.0s

## data

| run | games | samples | samples/game | first-player | g/s |
|---|---:|---:|---:|---:|---:|
| `data-e0` | 6144 | 515889 | 84.0 | 0.536 | 1.87 |
| `data-e10` | 6144 | 507559 | 82.6 | 0.523 | 1.83 |

## holdout

### `linear`

| source | sign_acc | auc | mse | ≤3 sign/auc/mse | 4–6 sign/auc/mse | ≥7 sign/auc/mse |
|---|---:|---:|---:|---|---|---|
| net | 0.729 | 0.813 | 0.709 | 0.683/0.755/0.804 | 0.730/0.817/0.701 | 0.761/0.847/0.648 |
| v0 | 0.626 | 0.698 | 1058.252 | 0.523/0.598/58.068 | 0.646/0.713/1036.743 | 0.681/0.747/1842.089 |
| search_v | 0.687 | 0.788 | 1775.600 | 0.622/0.682/947.133 | 0.671/0.771/1558.114 | 0.741/0.854/2492.540 |
| eval h0-linear-v1.json | 0.655 | 0.700 | 0.967 | 0.606/0.628/1.073 | 0.661/0.709/0.954 | 0.685/0.738/0.902 |

## yardstick

| model | main | reverse | pooled | sanity | g/s vs h0 | royal-nattui |
|---|---:|---:|---:|---:|---:|---:|
| linear | 0.552 [0.537, 0.567] | 0.550 [0.528, 0.571] | 0.551 [0.539, 0.564] | 1.000 [0.963, 1.000] | 1.67 / 1.91 | 0.560 [0.491, 0.627] |

## verdict

verdict: linear better — main 0.552 [0.537, 0.567], reverse 0.550 [0.528, 0.571], pooled 0.551 [0.539, 0.564] (not gated)
