# `h0-linear-v1`

The `h0-linear-v1` value model — linear on the 545 standardized features plus one per-card weight table per zone over a 243-id vocab; `scale` 60. The format is documented in `docs/engine-api.md` "Learned value".

## Provenance

Data: `py/matchup.py --policy h0 --games 24 --seed 201 --export …` (6 144 games, 523 739 rows, ε = 0) and `--seed 202 --export-epsilon 0.1` (6 144 games, 505 648 rows), engine `main` @ `e866b79` (`h0` there = v0 leaf, `tt=1`, 2 000 nodes). Label balance +1 484 832 / −1 441 631 / 0 800 (train).

Training: `py/train_value.py --data data-e0 data-e10 --model linear` with defaults (holdout 0.1 split by game = 1 229 games / 102 124 rows, 30 epochs, seed 0, l2 1e-4), 39 s CPU.

## Holdout

Sign acc 0.802 / AUC 0.887 / MSE 0.545 (≤ 3: 0.765 / 0.847; 4–6: 0.800 / 0.884; ≥ 7: 0.828 / 0.911) vs `value_v0` 0.649 / 0.738 (0.479 / 0.588; 0.677 / 0.743; 0.725 / 0.800).

## Yardstick

Owner's i7-14700F, 28 threads: `h0:value=net,net=linear.json` vs `h0`: 0.536 [0.521, 0.551] over 4 096 games (16 decks, `first=alternate`, seed 1), reverse seating (net as B) 0.529 [0.507, 0.550] over 2 048 — both intervals above 0.50, the same edge the transposition table gave (+3.6 pt); craft mirrors ×200: basic-forest 0.610, abyss-p8rfn 0.595, rune-mach15 0.560, afnm-minatodao 0.545, elf-neanisu2 0.505, royal-nattui 0.445 (±7 %); vs `random` 100–0; throughput 2.42 vs 2.80 g/s (−14 %; the linear leaf costs ~6 µs against v0's 0.15 µs). The MLP from the same data (0.801 / 0.887 pointwise) was 0.523 [0.508, 0.538] with the reverse run at 0.500 — not shipped.

sha256 `a29910999d5b9529a45406f3ec7259b6c607155596e5f061dcafe8a469cca774`

## Rule

Model files are immutable measured artifacts — a retrained model is a new file with a new name (`h0-linear-v2`, …) and becomes the default only after it beats the current default at ±1.5 % on the 4 096-game yardstick with the reverse seating agreeing; never regenerate a committed file in place. Unknown card ids (cards added after training) map to the net's index 0 — a deck-set change is a reason to retrain, not to edit the file.

# `h0-linear-v2`

The `h0-linear-v2` value model — linear on the 567 standardized features (encoding 2: 18 stat-bonus and cost-reduction features for the bot's own hand and deck, 4 flags for both players' Extra PP charges, and the strict opponent pool) plus one per-card weight table per zone over a 245-id vocab; `scale` 60. The format is documented in `docs/engine-api.md` "Learned value".

## Provenance

Command: `python py/iterate.py --tag net5-v2 --encoding 2 --seed 12 --bot h0 --target outcome --models linear --publish` on engine `1e41763`, results branch `c029fcb`.

Data: 6 144 self-play games at ε = 0 (515 889 samples) + 6 144 at ε = 0.1 (507 559); train 920 762 rows, holdout 102 686 (split by game); `train_value.py --model linear`, 45 s.

## Holdout

Sign acc 0.730 / AUC 0.814 / MSE 0.707 (`h0-linear-v1` on the v1 block of the same rows: 0.655 / 0.699 / 0.967).

## Yardstick

`h0:value=net,net=<net5-v2 linear.json>` vs `h0`: main 0.562 [0.547, 0.578] (4 096 games), reverse 0.573 [0.552, 0.595] (2 048), pooled 0.566 [0.554, 0.578] = +46.2 Elo; sanity vs random 1.000; royal-nattui mirror 0.540; throughput 1.64 vs 1.87 g/s (the candidate's games are longer: 89.2 vs 82.2 actions).

sha256 `3262c437bb628c03edc9e013e0904732ee5d5c4fbac9c5f1a9b50a98e544f216`

This was the default H0 leaf from the `h0-linear-v2` default PR until `h0-linear-v3` shipped. Reach it with `h0:net=engine/models/h0-linear-v2.json`; the previous leaf with `h0:net=engine/models/h0-linear-v1.json`.

# `h0-linear-v3`

The `h0-linear-v3` value model — linear on the 567 standardized features (encoding 2: 18 stat-bonus and cost-reduction features for the bot's own hand and deck, 4 flags for both players' Extra PP charges, and the strict opponent pool) plus one per-card weight table per zone over a 245-id vocab; `scale` 60. The format is documented in `docs/engine-api.md` "Learned value".

## Provenance

Data: `results/net5-v2`, `results/net6`, and `results/net7`, each `data-e0` + `data-e10` (49 152 games, 4 219 106 rows).

Training: `py/train_value.py --model linear --target outcome --holdout 0 --epochs 2 --l2 1e-3 --seed 1` (Adam), 114 s CPU. The file's `trained_on` does not record `--l2`; it was `1e-3`.

## Yardstick

`h0:net=<this file>` vs `h0` (= v2), two independent samples on different deals:

- sweep 28 (seed 1; results `9ecf4bb`): main 0.5181 (2 048), reverse 0.5283 (1 024), pooled 0.5215;
- sweep 28b (seed 3; results `6d3e271`, combination `16ee62c`): main 0.5181 (4 096), reverse 0.5205 (2 048), pooled 0.5189;

**Combined per arm: main 0.5181 [0.5056, 0.5305] (6 144 games), reverse 0.5231 [0.5054, 0.5407] (3 072), pooled 0.5197 [0.5095, 0.5299] = +13.7 Elo [+6.6, +20.8]** — both arms' lower bounds above 0.50, the project's `better` rule. Throughput unchanged (1.71 vs 1.67 games per second in sweep 28b's self-play).

sha256 `926ce6021b27f94a49da965d9b7cd3b1b04283acf92d0c372a76b28b85e117b2`

This is the default H0 leaf since this PR. Reach the previous leaf with `h0:net=engine/models/h0-linear-v2.json`.

## Optional race block

Linear models may include an optional `"race"` block (twelve HP / board-attack
inputs; see `docs/engine-api.md` "Optional race block"). The committed defaults
(`h0-linear-v1` … `h0-linear-v3`) have no race block. Train one with
`py/train_value.py --race` on a linear export; gate in play comes after merge.
