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

This was the default H0 leaf from the `h0-linear-v3` default PR until `h0-linear-v4` shipped. Reach it with `h0:net=engine/models/h0-linear-v3.json`; the previous leaf with `h0:net=engine/models/h0-linear-v2.json`.

# `h0-linear-v4`

The `h0-linear-v4` value model — linear on the 961 standardized features (encoding 3: v3's 567 encoding-2 features plus crest identity, amulet countdowns, enter counts, and the cemetery for both players) plus one per-card weight table per zone over a 245-id vocab; `scale` 60. Thirteen zones: v3's five unchanged, eight stacked rows for the new encoding-3 inputs. The format is documented in `docs/engine-api.md` "Learned value".

## Provenance

Data: `py/iterate.py --tag net10 --encoding 3 --seed 22 --bot h0 --games 48 --only data` (24 576 self-play games, 2 096 983 rows).

Training: `py/train_value.py --data results/net10/data-e0 results/net10/data-e10 --model linear --target outcome --stack-on engine/models/h0-linear-v3.json --seed 1` with penalties crests `1e-6`, amulets `0.1`, entered `1`, cemetery `1` (deck controls during the fit only; not in the file). `trained_on.stack_on_sha256` `c2621ef8e0ea2e98663ca0661b44a3fa4ff59b3b93f807d3f48c04043756a451` is v3's file with CRLF line endings (the owner's Windows checkout); the frozen weights equal v3's exactly.

## Yardstick

`h0:net=<this file>` vs `h0` (= v3), all 16 meta decks (sweep 43, engine `3742a1b`, results `193f48a`):

- main 784 / 1 536 = 0.5104, reverse 496 / 1 024 = 0.4844;
- **pooled 1 280 / 2 560 = 0.5000 [0.4806, 0.5194], +0.0 Elo [−13.5, +13.5]**;
- early stop settled "pooled ≥ 0.49";
- throughput 1.03× (games a little shorter; per action 1.00×).

sha256 `76685509ff4aef49e34ae37793385d40b477d0756b52141299300f4cb85d8c95`

Promoted under the owner's "not worse" bar (pooled ≥ 0.49, registered before the leaf existed), not the `better` rule v2 and v3 passed. This is the default H0 leaf since this PR. Reach the previous leaf with `h0:net=engine/models/h0-linear-v3.json`.

# `mulligan-v1`

The `mulligan-v1` opening keep table (engine format version 1). Documented in `docs/engine-api.md` "Learned mulligan".

## Provenance

Data: `mull1`, the random-keep run behind sweep 13 (`py/mulligan.py data` + `fit`); z ≥ 1.5 departures from the cost ≥ 4 rule.

Yardstick: sweep 13 pooled 0.521 [0.509, 0.534] / +14.9 Elo vs `mull=rule` on the 16-deck meta pool (6 144 games per candidate). This was the built-in table until `mulligan-v2` shipped. Reach it with `h0:mull=engine/models/mulligan-v1.json`.

sha256 `a500fb7b44ca41ca3c5c4733dace9bf7c230ffda5aafc9a8eaa95a71d16e4e68`

## Rule

Committed files are immutable measured artifacts — a new table is a new file (`mulligan-v2`, …) and becomes the default only after it beats the current default on the registered sweep rule; never regenerate a committed file in place.

# `mulligan-v2`

The `mulligan-v2` opening keep table (engine format version 1). Documented in `docs/engine-api.md` "Learned mulligan".

## Provenance

Data: `mull1` + `mull2` pooled via the shrinkage learner (`py/mulligan.py shrink`, empirical Bayes by rule group).

Yardstick: sweep 46 pooled 0.5094 [0.4999, 0.5188] / +6.5 Elo vs `mulligan-v1` on the 16-deck meta pool (10 752 final games). This is the built-in table since this PR.

sha256 `efbb96a0efac5cc9e9298e2e1bdbcb84fd0605cc47f13d29fa1f4f7374c6dde5`

## Rule

Committed files are immutable measured artifacts — a new table is a new file (`mulligan-v3`, …) and becomes the default only after it beats the current default on the registered sweep rule; never regenerate a committed file in place.

## Optional race block

Linear models may include an optional `"race"` block (twelve HP / board-attack
inputs; see `docs/engine-api.md` "Optional race block"). The committed defaults
(`h0-linear-v1` … `h0-linear-v4`) have no race block. Train one with
`py/train_value.py --race` on a linear export; gate in play comes after merge.
