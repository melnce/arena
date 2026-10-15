# run56: the ranking fit on teach56 (the ordering model, trained again)

- verdict: NO SWEEP
- rows: training 553092, validation 71031, test 66638
- picked on validation: tau 3.0, lam 0.1
- test, v4: {'decisions': 12740, 'rows': 66638, 'top1': 0.515, 'recall@4': 0.7953, 'decisions_over_4': 6274, 'regret': 4.1781, 'regret_unsat': 3.2732, 'decisions_unsat': 11318, 'ce': 1.7254}
- test, leaf53 (the base, order=builtin): {'decisions': 12740, 'rows': 66638, 'top1': 0.5799, 'recall@4': 0.8634, 'decisions_over_4': 6274, 'regret': 2.48, 'regret_unsat': 1.9718, 'decisions_unsat': 11318, 'ce': 1.31627}
- test, leaf56: {'decisions': 12740, 'rows': 66638, 'top1': 0.5793, 'recall@4': 0.8653, 'decisions_over_4': 6274, 'regret': 2.4921, 'regret_unsat': 1.9748, 'decisions_unsat': 11318, 'ce': 1.31332}
- gate: test regret at most 0.97 x leaf53's (2.4056) and recall@4 at least leaf53's minus 0.005 (0.8584): fail
- engine: leaf56.json through net= against numpy, 839 test afterstates: max |diff| 2.20e-05; h0:nodes=300,net=results/run56/leaf56.json: ok; h0:nodes=300,order=results/run56/leaf56.json: ok; h0:nodes=300,order=results/run56/leaf56.json,oorder=1: ok
- leaf56.json sha256 61a2858880061c078ddad84698ff3d8fbd6644c7fe083ba3373f4c2a92a2876e

## Grid (validation)

| tau | lam | steps | stop | top1 | recall@4 | regret | regret without +-80 | ce |
|---|---|---|---|---|---|---|---|---|
| 1.0 | 1 | 149 | line search | 0.5901 | 0.8577 | 2.5395 | 2.0163 | 1.22657 |
| 1.0 | 0.1 | 300 | max steps | 0.5962 | 0.863 | 2.4753 | 1.9717 | 1.21192 |
| 1.0 | 0.01 | 300 | max steps | 0.5948 | 0.863 | 2.4647 | 1.9696 | 1.20946 |
| 1.0 | 0.001 | 300 | max steps | 0.5942 | 0.8634 | 2.5042 | 1.9901 | 1.21 |
| 1.0 | 0.0001 | 300 | max steps | 0.5952 | 0.8632 | 2.4721 | 1.9588 | 1.20871 |
| 1.0 | 1e-05 | 300 | max steps | 0.5946 | 0.8636 | 2.4989 | 1.9775 | 1.20932 |
| 3.0 | 1 | 65 | line search | 0.5923 | 0.8581 | 2.4347 | 1.9585 | 1.2915 |
| 3.0 | 0.1 | 125 | line search | 0.5933 | 0.8617 | 2.4213 | 1.95 | 1.29047 |
| 3.0 | 0.01 | 300 | max steps | 0.593 | 0.8637 | 2.4214 | 1.9436 | 1.29025 |
| 3.0 | 0.001 | 300 | max steps | 0.5924 | 0.8645 | 2.4261 | 1.9362 | 1.29062 |
| 3.0 | 0.0001 | 300 | max steps | 0.5916 | 0.8645 | 2.4519 | 1.9446 | 1.2911 |
| 3.0 | 1e-05 | 300 | max steps | 0.5924 | 0.866 | 2.4383 | 1.9519 | 1.29075 |
| leaf53 | | | | 0.5902 | 0.8563 | 2.4517 | 1.9412 | 1.29411 |
| v4 | | | | 0.5172 | 0.7934 | 4.4745 | 3.3375 | 1.74249 |
