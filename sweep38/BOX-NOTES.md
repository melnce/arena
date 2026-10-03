# sweep38: box notes (leaves, soups, guard, readings)

**Build:** `1c551eb`, as sweep 37, with no pull (HEAD checked by the chain).

## Step 1: leaves

v3's data in v3's order (net5-v2, net6, net7; `-e0`, `-e10`), `--epochs 2 --l2 1e-3`, one at a time.

| leaf | rows_train | epochs_run | train_seconds | peak working set / private |
|---|---:|---:|---:|---|
| leaf-all4-e2-l2x10-s3 | 4219106 | 2 | 113.7 | 13.0 / 13.8 GB |
| leaf-all4-e2-l2x10-s4 | 4219106 | 2 | 114.5 | 13.0 / 13.8 GB |
| leaf-all4-e2-l2x10-s5 | 4219106 | 2 | 114.0 | 13.0 / 13.8 GB |
| leaf-all4-race-e2-l2x10-s3 | 4219106 | 2 | 123.6 | 13.0 / 13.8 GB |
| leaf-all4-race-e2-l2x10-s4 | 4219106 | 2 | 126.3 | 13.0 / 13.8 GB |
| leaf-all4-race-e2-l2x10-s5 | 4219106 | 2 | 124.8 | 13.0 / 13.8 GB |

## Step 2: soups

`soup_same.py`: sha256 over LF `cb63f1776513c8d62ac954095dedc286dbefaf61480235ced424318924c92de1`. Each soup averages
sweep 37's s1 and s2 with this run's s3–s5.

```
wrote results/sweep38/soup5-plain-e2-l2x10.json: 5 leaves averaged, race block: no
wrote results/sweep38/soup5-race-e2-l2x10.json: 5 leaves averaged, race block: yes
```

**Checked on the box:**

- Each soup's `w`, `zone_w`, `b` (and `race.w`) equals the mean of its five members exactly (max abs difference 0.0).
- `trained_on.soup` lists the five members in order.

**Cosine against v3**, dense (`linear.w`) and card (`linear.zone_w`):

| leaf | dense cosine | card cosine |
|---|---|---|
| plain soup | 0.995731 | 0.990041 |
| race soup | 0.977897 | 0.984000 |
| plain s3 | 0.990968 | 0.974904 |
| plain s4 | 0.988504 | 0.975344 |
| plain s5 | 0.988853 | 0.975195 |

## Step 3: guard on the race soup

`race_guard.py`: sha256 over LF `cb26818b04b5eef8bda6f4d2c77acc318f85c4570800ea9d6e9533225ebfcf9f`. Exit 0.

```
1. race leaf loads and plays: yes
2. malformed race block refused: yes (unknown policy h0:net=results/sweep37-guard/bad.json; available ["random","first-legal","h0"]: results/sweep37-guard/bad.json: bad field 'race.version' (99))
   position 0: turn 4, value with race 4.301300525665283, without 6.225229263305664
   position 1: turn 4, value with race 15.178380012512207, without 9.72739028930664
   position 2: turn 4, value with race -18.88373374938965, without -9.74976921081543
   position 3: turn 4, value with race -6.456179618835449, without 8.718278884887695
3. race block changes values: yes (4 of 4 positions)
RACE BLOCK ACTIVE
```

## Readings

Paired over the 256 deck pairs: the same deals as sweeps 36 and 37 (seed 1), pooled finals, bootstrap 95 %.

| quantity | value |
|---|---|
| S = c01 (plain soup) − mean(sweep 37 c03 = order 2, sweep 38 c03 = order 3) | **+2.26 pts [+0.78, +3.79]** |
| R = c02 (race soup) − c01 (plain soup) | **−0.13 pts [−1.92, +1.56]** |
| race soup − mean(sweep 37 race s1, race s2) | +1.17 pts [−0.29, +2.80] |
| order 3 − order 2 (single plain leaves) | +1.86 pts [+0.00, +3.71] |
| plain soup − order 2 / − order 3 | +3.19 [+1.50, +4.95] / +1.33 [−0.49, +3.12] |
| race soup − race s1 / − race s2 | +0.03 [−1.69, +1.79] / +2.31 [+0.46, +4.17] |
| order 3 − sweep 36 net9 / net9+net8 / net8 | +3.48 / +1.82 / +1.95 |
| plain soup / race soup − sweep 36 net9+net8 | +3.16 / +3.03 |

**The lottery.** Pooled against v3, single plain orders scored:

| order | pooled vs v3 | difference |
|---|---|---|
| 1 | — | v3 itself (the reproduction) |
| 2 | 0.4717 | −2.83 pts |
| 3 | 0.4902 | −0.98 pts |

**On the box during the sweep:** nothing else. chain-s39 waited, idle, for this chain to finish. The tp numbers are
unloaded.
