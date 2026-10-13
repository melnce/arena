# run53: the ranking fit

- verdict: SWEEP
- rows: training 1067608, validation 133801, test 134167
- picked on validation: tau 3.0, lam 0.001
- test, v4: {'decisions': 26216, 'rows': 134167, 'top1': 0.5357, 'recall@4': 0.8042, 'decisions_over_4': 12578, 'regret': 4.1763, 'regret_unsat': 3.2693, 'decisions_unsat': 23427, 'ce': 1.69595}
- test, leaf53: {'decisions': 26216, 'rows': 134167, 'top1': 0.5921, 'recall@4': 0.8717, 'decisions_over_4': 12578, 'regret': 2.5167, 'regret_unsat': 2.0373, 'decisions_unsat': 23427, 'ce': 1.28554}
- gate: test regret at most 0.90 x v4's (3.7587) and recall@4 at least v4's (0.8042): pass
- engine: leaf53.json through net= against numpy, 781 test afterstates: max |diff| 2.24e-05; h0:nodes=300,net=results/run53/leaf53.json: ok; h0:nodes=300,order=results/run53/leaf53.json: ok; h0:nodes=300,order=results/run53/leaf53.json,oorder=1: ok
- leaf53.json sha256 35db9e7ac511e8c85e9145b5531d9d7369d95625df8af3a3c59a80da9df4a6e5

## Grid (validation)

| tau | lam | steps | stop | top1 | recall@4 | regret | regret without +-80 | ce |
|---|---|---|---|---|---|---|---|---|
| 1.0 | 1 | 92 | line search | 0.5591 | 0.833 | 3.3543 | 2.5889 | 1.33594 |
| 1.0 | 0.1 | 136 | line search | 0.5881 | 0.856 | 2.8866 | 2.2506 | 1.24484 |
| 1.0 | 0.01 | 300 | max steps | 0.604 | 0.8712 | 2.5546 | 2.0187 | 1.19668 |
| 1.0 | 0.001 | 300 | max steps | 0.604 | 0.872 | 2.5564 | 2.0095 | 1.19344 |
| 1.0 | 0.0001 | 300 | max steps | 0.604 | 0.8736 | 2.5551 | 2.0066 | 1.19251 |
| 1.0 | 1e-05 | 300 | max steps | 0.6047 | 0.8724 | 2.5615 | 2.0118 | 1.1926 |
| 3.0 | 1 | 76 | line search | 0.5585 | 0.8304 | 3.4201 | 2.6284 | 1.3829 |
| 3.0 | 0.1 | 162 | line search | 0.5801 | 0.8512 | 2.9336 | 2.3005 | 1.31558 |
| 3.0 | 0.01 | 300 | max steps | 0.5966 | 0.8702 | 2.5842 | 2.037 | 1.28277 |
| 3.0 | 0.001 | 300 | max steps | 0.5979 | 0.8728 | 2.5475 | 2.0039 | 1.27978 |
| 3.0 | 0.0001 | 300 | max steps | 0.5969 | 0.8731 | 2.563 | 2.0167 | 1.27943 |
| 3.0 | 1e-05 | 300 | max steps | 0.5976 | 0.8728 | 2.5642 | 2.017 | 1.27929 |
| v4 | | | | 0.536 | 0.8086 | 4.454 | 3.3894 | 1.70757 |
