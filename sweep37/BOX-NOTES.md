# sweep37 — box notes (pull, rebuild, leaves, guard, pairings)

## Step 1: build

- `git pull --ff-only`: 33de0a9 -> **1c551eb**; `git merge-base --is-ancestor 1c551eb HEAD` printed 0.
- `maturin develop --release`: module rebuilt 2026-10-03 19:43 local.
- `pytest py/tests -q`: **161 passed, 6 skipped, 1 xfailed**. No failures; `test_race.py` 7 passed. The skips and the
  xfail are this box's usual local ones: before the pull it had 154 passed, 6 skipped, 1 xfailed, and the 7 new tests
  account for the difference. CI: 163 passed, 5 skipped.

## Step 2: leaves

v3's data in v3's order (net5-v2, net6, net7; `-e0`, `-e10`), `--epochs 2 --l2 1e-3`, one at a time.

| leaf | rows_train | epochs_run | train_seconds | peak working set / private |
|---|---:|---:|---:|---|
| leaf-all4-race-e2-l2x10-s1 | 4219106 | 2 | 136.7 | 13.0 / 13.8 GB |
| leaf-all4-race-e2-l2x10-s2 | 4219106 | 2 | 138.3 | 13.0 / 13.8 GB |
| leaf-all4-e2-l2x10-s2 | 4219106 | 2 | 125.3 | 13.0 / 13.8 GB |
| leaf-all4-e2-l2x10-s1 | 4219106 | 2 | 124.6 | 13.0 / 13.8 GB |

**Plain s1 against v3** (`engine/models/h0-linear-v3.json`). Dense = `linear.w`; card = `linear.zone_w`, flattened.

| leaf | dense cosine | card cosine |
|---|---|---|
| plain s1 | **0.999988** (max abs diff 2.6e-3) | **1.000000** (max abs diff 5.2e-4) |
| plain s2 | 0.990175 | 0.975241 |
| race s1 | 0.980056 | 0.993244 |
| race s2 | 0.973601 | 0.969380 |

For plain s1, the bias is 0.0044033 vs 0.0044040 and `feat_mean` / `feat_std` are identical, so the box reproduces
v3. All four leaves list the same data order as v3.

**Race weights** (version 1):

| input | s1 | s2 |
|---|---:|---:|
| lo5_me | +0.003 | −0.006 |
| lo10_me | −0.124 | −0.134 |
| threat_me | −0.001 | +0.006 |
| near_me | −0.002 | +0.002 |
| margin_me | +0.072 | +0.071 |
| inter_me | −0.097 | −0.103 |
| lo5_opp | +0.047 | +0.042 |
| lo10_opp | +0.140 | +0.141 |
| threat_opp | +0.009 | +0.017 |
| near_opp | −0.003 | +0.009 |
| margin_opp | −0.021 | −0.032 |
| inter_opp | +0.091 | +0.088 |

## Step 3: guard

`race_guard.py`: sha256 over LF `cb26818b04b5eef8bda6f4d2c77acc318f85c4570800ea9d6e9533225ebfcf9f`. Exit 0.

```
1. race leaf loads and plays: yes
2. malformed race block refused: yes (unknown policy h0:net=results/sweep37-guard/bad.json; available ["random","first-legal","h0"]: results/sweep37-guard/bad.json: bad field 'race.version' (99))
   position 0: turn 4, value with race 7.73039436340332, without 9.006847381591797
   position 1: turn 4, value with race 15.511435508728027, without 10.818319320678711
   position 2: turn 4, value with race -15.943215370178223, without -4.3590192794799805
   position 3: turn 4, value with race -9.636502265930176, without 7.5515217781066895
3. race block changes values: yes (4 of 4 positions)
RACE BLOCK ACTIVE
```

## Step 4: pairings

Paired over the 256 deck pairs: the same deals as sweep 36 (seed 1, same decks), pooled finals, bootstrap 95 %.

| sweep 37 | sweep 36 | difference | sweep 37 vs sweep 36 pooled |
|---|---|---|---|
| c01 race s1 | c01 net9 | +4.65 pts [+2.60, +6.77] | 0.5020 vs 0.4554 |
| c01 race s1 | c02 net9+net8 | +2.99 pts [+0.94, +5.08] | 0.5020 vs 0.4720 |
| c01 race s1 | c03 net8 | +3.12 pts [+1.24, +5.08] | 0.5020 vs 0.4707 |
| c02 race s2 | c01 net9 | +2.38 pts [+0.42, +4.33] | 0.4792 vs 0.4554 |
| c02 race s2 | c02 net9+net8 | +0.72 pts [−1.24, +2.70] | 0.4792 vs 0.4720 |
| c02 race s2 | c03 net8 | +0.85 pts [−1.04, +2.83] | 0.4792 vs 0.4707 |
| c03 plain s2 | c01 net9 | +1.63 pts [−0.42, +3.65] | 0.4717 vs 0.4554 |
| c03 plain s2 | c02 net9+net8 | −0.03 pts [−1.89, +1.89] | 0.4717 vs 0.4720 |
| c03 plain s2 | c03 net8 | +0.10 pts [−1.99, +2.18] | 0.4717 vs 0.4707 |

Within sweep 37:

| comparison | difference |
|---|---|
| c02 − c03 | +0.75 pts [−0.94, +2.47] |
| c01 − c03 | +3.03 pts [+0.98, +5.01] |

**On the box during the sweep:** the owner's review10 games on serve.py, from the start until about 19:12 UTC. The
search is node-limited, so the win rates are unaffected; the tp numbers carry that load.
