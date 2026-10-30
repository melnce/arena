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

Promoted under the owner's "not worse" bar (pooled ≥ 0.49, registered before the leaf existed), not the `better` rule v2 and v3 passed. This is the default H0 leaf since this PR. `net=builtin` and `net=h0-linear-v4` reach it; `py/serve.py`'s defaults and the browser's Full-strength bot score their lines with [`h0-value-v73`](#h0-value-v73) since brief YP (`h0-value-v64` from brief YC, `h0-value-v65` from brief YE, `h0-value-v66` from brief YF, `h0-value-v69` from brief YJ). Reach the previous leaf with `h0:net=engine/models/h0-linear-v3.json`.

# `h0-value-v64`

The `h0-value-v64` value model (the leaf) — `arch: residual` on encoding 3 (`docs/engine-api.md` "Learned value"):
`h0-linear-v4`'s linear part unchanged (same features, standardization, vocab and zones) plus a ReLU MLP residual
with 32 hidden units on the 961 standardized features and an 8-dimensional embedding per card and zone (13 zones),
`scale` 60. Its residual is the trained network's halved — the pre-activation is `h0-linear-v4`'s plus ½ × the network's
residual (`w2`, `b2` scaled by 0.5: `trained_on.shrink` 0.5); sweep 64 screened the full network too (0.549
against this file's 0.578) and sent this one to the final.

## Provenance

Data: every search decision with two or more scored candidates of the box's teach52 / 54 / 56 / 57 / 59 / 60 / 61 /
62 games (32 768 games of the served search, 2.2 M states from both seats), each encoded from the decider's side
with the decider's result (won / lost) as the target, split by game (crc32: training 1 774 682, validation 227 736,
test 222 743 states). Training: the squared error of `60·tanh(pre)` against `60 × result` (`h0-linear-v4`'s own
target), AdamW lr 1e-4, weight decay 1e-3, a moving average of the weights (one epoch), best epoch 3 of 4 by
validation (bots thread probe `rb64`, trainer `train64.py` — the file's `trained_on.probe` reads `train64` — model
`O8-h32`). On the test games (3 279): at turn starts (where the search uses the leaf) this file's Brier
score is 0.1982, the full network's 0.1934, `h0-linear-v4`'s 0.2170.

## Yardstick

Sweep 64 (engine `d49783c`, results `sweep64/` at `19da5cf`), all 16 meta decks: `net=<this file>` vs
the built-in `h0-linear-v4`, both on
`h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40`:

- screen 296 / 512 = 0.578 (the full network 281 / 512 = 0.549);
- final main 1 410 / 2 560 = 0.551, reverse 826 / 1 536 = 0.538, **pooled 2 236 / 4 096 = 0.546 [0.531, 0.561], +32.0 Elo** (adopted by the registered rule, ≥ 0.51);
- self-play throughput × 0.816 (longer games: 94.3 actions against 89.2).

sha256 `df4dd3ccdfa582e7cddaa6ee1fd831751661d7b2e61d6f31a1a44a8cc83846bb`

Reach it with `net=h0-value-v64` (embedded like `h0-linear-v4`). `py/serve.py`'s defaults and the browser's
Full-strength bot scored their lines with it from brief YC to brief YE ([`h0-value-v65`](#h0-value-v65) since).

# `h0-value-v65`

The `h0-value-v65` value model (the leaf) — `arch: residual` on encoding 3, the same architecture as
[`h0-value-v64`](#h0-value-v64): `h0-linear-v4`'s linear part unchanged plus a ReLU MLP residual with 32 hidden
units on the 961 standardized features and an 8-dimensional embedding per card and zone (13 zones), `scale` 60.
Its residual is three quarters of the trained network's (`w2`, `b2` scaled by 0.75: `trained_on.shrink` 0.75);
sweep 65 screened the halved form too (269 / 512 against this file's 270 / 512) and sent this one to the final.

## Provenance

Data: every search decision with two or more scored candidates of the box's teach52 / 54 / 56 / 57 / 59 / 60 / 61 /
62 / 63 games (36 864 games of the served search, 2.0 M training states from both seats), each encoded from the
decider's side, split by game (crc32) as for `h0-value-v64`. Targets: TD(λ 0.95) returns instead of the bare
result — four networks of `h0-value-v64`'s recipe, each trained on three quarters of the training games
(cross-fitted folds), score the states of the quarter they never saw, and each state's target mixes those values
of the game's later states with the result at the end. Training: `h0-value-v64`'s recipe (the squared error of
`60·tanh(pre)`, AdamW lr 1e-4, weight decay 1e-3, a moving average of the weights over one epoch) for 10 epochs on
those targets (bots thread probe `rb65`, trainer `train65.py`). On the 3 695 held-out games of those sets, at turn
starts (where the search uses the leaf), Brier score: this file 0.1890, the full network 0.1875, `h0-value-v64`
0.1973, `h0-linear-v4` 0.2162.

## Yardstick

Sweep 65 (engine `d49783c`, results `sweep65/` at `3fa4c97`), all 16 meta decks: `net=<this file>` vs `h0-value-v64`, both on
`h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40`:

- screen 270 / 512 = 0.527 (the halved network 269 / 512 = 0.525);
- final main 2 106 / 4 096 = 0.514, reverse 1 047 / 2 048 = 0.511, **pooled 3 153 / 6 144 = 0.513 [0.502, 0.524], +9.2 Elo** (adopted by the registered rule, ≥ 0.51);
- self-play throughput × 0.997.

sha256 `954ca78066262b462aab433ccd09a702958557e92106cd632e416549d27cbaf0`

Reach it with `net=h0-value-v65` (embedded like `h0-linear-v4`). `py/serve.py`'s defaults and the browser's
Full-strength bot scored their lines with it from brief YE to brief YF ([`h0-value-v66`](#h0-value-v66) since).

# `h0-value-v66`

The `h0-value-v66` value model (the leaf) — `arch: residual` on encoding 3, the same architecture as
[`h0-value-v65`](#h0-value-v65): `h0-linear-v4`'s linear part unchanged plus a ReLU MLP residual with 32 hidden
units on the 961 standardized features and an 8-dimensional embedding per card and zone (13 zones), `scale` 60.
Its residual is three quarters of the trained network's (`w2`, `b2` scaled by 0.75: `trained_on.shrink` 0.75);
sweep 66 screened the full form too (266 / 512 against this file's 268 / 512) and sent this one to the final.

## Provenance

Trained on the owner's box inside runbook 66 (`fit66.py`), with `h0-value-v65`'s recipe exactly: every search
decision with two or more scored candidates of the box's teach52 / 54 / 56 / 57 / 59 / 60 / 61 / 62 / 63 / 64 / 65
games (45 056 games of the served search — teach64 / 65 are the first a network leaf played —, 2.48 M states from
both seats), each encoded from the decider's side, split by game (crc32). Targets: TD(λ 0.95) returns from four
cross-fitted networks of `h0-value-v64`'s recipe; then 10 epochs of the squared error of `60·tanh(pre)` (AdamW lr
1e-4, weight decay 1e-3, a moving average of the weights over one epoch; best epoch 8). On the 4 527 held-out games
of those sets, at turn starts (where the search uses the leaf), Brier score: this file 0.1890, the full network
0.1868, `h0-value-v65` 0.1897, `h0-value-v64` 0.1982, `h0-linear-v4` 0.2179.

## Yardstick

Sweeps 66 and 66b (engine `d49783c`, results `sweep66/` at `cc9df7e`, `sweep66b/` at `950a567`), all 16 meta decks:
`net=<this file>` vs `h0-value-v65`, both on
`h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=40`:

- screens 268 / 512 = 0.523 (sweep 66; the full network 266 / 512) and 282 / 512 = 0.551 (66b);
- finals main 4 121 / 8 192 = 0.503, reverse 2 094 / 4 096 = 0.511, **pooled 6 215 / 12 288 = 0.506 [0.498, 0.514],
  +4.0 Elo** (sweep 66 alone 0.505 → the confirm run; adopted by the registered rule, both runs combined ≥ 0.505);
- self-play throughput × 0.958 / × 1.031.

sha256 `facd1ed275de891b5ef6f683aa4c217d945bd3503a70d16b5ccf84763b0f7975`

Reach it with `net=h0-value-v66` (embedded like `h0-linear-v4`). `py/serve.py`'s defaults and the browser's
Full-strength bot scored their lines with it from brief YF to brief YJ ([`h0-value-v69`](#h0-value-v69) since).

# `h0-value-v69`

The `h0-value-v69` value model (the leaf) — `arch: residual` on encoding 3, the same architecture as
[`h0-value-v66`](#h0-value-v66): `h0-linear-v4`'s linear part unchanged plus a ReLU MLP residual with 32 hidden
units on the 961 standardized features and an 8-dimensional embedding per card and zone (13 zones), `scale` 60.
Its residual is three quarters of the trained network's (`w2`, `b2` scaled by 0.75: `trained_on.shrink` 0.75);
sweep 69 screened the full form too (236 / 512 against this file's 244 / 512) and sent this one to the final.

## Provenance

Trained on the owner's box inside runbook 69 (`fit69.py`, `fit66.py`'s code), with `h0-value-v66`'s recipe
exactly: every search decision with two or more scored candidates of the box's teach52 / 54 / 56 / 57 / 59 / 60 /
61 / 62 / 63 / 64 / 65 / 66 / 67 / 68 games (61 440 games of the served search — teach67 / 68 are the `onext=0`
bot's —, 4.27 M states from both seats, 3.41 M of them for training), each encoded from the decider's side, split
by game (crc32). Targets: TD(λ 0.95) returns from four cross-fitted networks of `h0-value-v64`'s recipe; then 10
epochs of the squared error of `60·tanh(pre)` (AdamW lr 1e-4, weight decay 1e-3, a moving average of the weights
over one epoch; best epoch 8). On the 6 185 held-out games of those sets, at turn starts (where the search uses
the leaf), Brier score: this file 0.1865, the full network 0.1847, `h0-value-v66` 0.1880, `h0-linear-v4` 0.2175.

## Yardstick

Sweeps 69 and 69b (engine `d49783c`, results `sweep69/` at `823f36b`, `sweep69b/` at `50bd78e`), all 16 meta decks:
`net=<this file>` vs `h0-value-v66`, both on `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0`:

- screens 244 / 512 = 0.477 (sweep 69; the full network 236 / 512) and 266 / 512 = 0.520 (69b);
- finals main 3 893 / 7 680 = 0.507, reverse 1 855 / 3 584 = 0.518, **pooled 5 748 / 11 264 = 0.510 [0.502, 0.519], +7.2 Elo** (sweep 69 alone 0.510 → the confirm run; adopted by the registered rule, both runs combined ≥ 0.505);
- self-play throughput × 1.028 / × 1.007.

sha256 `8c30ae8632d9161444fda5b65c637d60977a02d40105a7b7ccf670d8e1f230a3`

Reach it with `net=h0-value-v69` (embedded like `h0-linear-v4`). `py/serve.py`'s defaults and the browser's
Full-strength bot scored their lines with it from brief YJ to brief YP ([`h0-value-v73`](#h0-value-v73) since).

# `h0-value-v73`

The `h0-value-v73` value model (the leaf) — `arch: residual` on encoding 3, the same architecture as
[`h0-value-v69`](#h0-value-v69): `h0-linear-v4`'s linear part unchanged plus a ReLU MLP residual with 32 hidden
units on the 961 standardized features and an 8-dimensional embedding per card and zone (13 zones), `scale` 60.
Its residual is three quarters of the trained network's (`w2`, `b2` scaled by 0.75: `trained_on.shrink` 0.75).

## Provenance

Trained on the owner's box inside runbook 73 (`fit73.py`, `fit66.py`'s code), with `h0-value-v69`'s recipe
exactly: every search decision with two or more scored candidates of the box's eighteen teach archives teach52 …
teach72 (94 208 games of the served search — teach72, 8 192 of them, played on `main`'s play-reaction order of
10-28 and with the leaf's own mulligan —, 6.59 M states from both seats, 5.25 M of them for training), each encoded
from the decider's side, split by game (crc32). Targets: TD(λ 0.95) returns from four cross-fitted networks of
`h0-value-v64`'s recipe; then 10 epochs of the squared error of `60·tanh(pre)` (AdamW lr 1e-4, weight decay 1e-3,
a moving average of the weights over one epoch; best epoch 10). On the 9 489 held-out games of those sets, at turn
starts (where the search uses the leaf), Brier score: this file 0.1853, the full network 0.1832, `h0-value-v69`
0.1873, `h0-linear-v4` 0.2208.

## Yardstick

Sweep 73 (engine `c30242d`, results `sweep73/` at `30f93be`), all 16 meta decks: `net=<this file>` vs
`h0-value-v69`, both on `h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=15,hbk=16,alloc=world,threads=1,lostrank=4000,order=h0-order-v59,oorder=h0-order-v53,oopen=1,onext=0,deal=block,mull=leaf,mulls=256`
(the leaf in both roles: the search's and the mulligan's):

- screen 252 / 512 = 0.492;
- final main 1 873 / 3 584 = 0.523, reverse 1 052 / 2 048 = 0.514, **pooled 2 925 / 5 632 = 0.519 [0.506, 0.532], +13.5 Elo** (adopted directly by the registered rule, ≥ 0.51; the early stop settled at look 11);
- self-play throughput × 1.04.

sha256 `5df505c67336e0c25bb9d8b24b3713e6d215b216ec80d926dc264eb1a9700efd`

Reach it with `net=h0-value-v73` (embedded like `h0-linear-v4`). `py/serve.py`'s defaults and the browser's
Full-strength bot score their lines with it since brief YP.

# `h0-order-v53`

The `h0-order-v53` move-ordering model — not a value leaf; same `net.json` format as `h0-linear-v4` (linear on 961 encoding-3 features, 245-id vocab, 13 zones). Used only where search ranks one-step values (`order=`); line scores still use the built-in value leaf.

The engine also accepts `arch: residual` on encoding 3 (linear pre-activation plus a small ReLU MLP residual on zone-wise embeddings; see `docs/engine-api.md` "Learned value"). The committed residual order file is [`h0-order-v59`](#h0-order-v59); train others offline and pass `order=<path>` like a linear order net.

## Provenance

Data: teach52 (the served search's per-candidate scores). Training: listwise softmax on each decision's candidates, τ 3, λ 0.001 (results `run53`, commit `1350405`). On teach52's held-out games vs v4 as order net: top-1 0.592 (v4 0.536), recall@4 0.872 (0.804), regret 2.52 (4.18) (`run53/FIT.md`).

## Yardstick

`order=<this file>,oorder=1` on the sweep-53 served spec (`h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1`) vs that spec alone, all 16 meta decks (engine `99e0775`, results `8954a03`):

- main 831 / 1 536 = 0.541, reverse 551 / 1 024 = 0.538;
- **pooled 1 382 / 2 560 = 0.540 [0.520, 0.559], +27.7 Elo**;
- throughput × 0.933 (ordering costs ≈ 7 % per game).

As a value leaf (`net=`) on the same yardstick it screened 0.492 at × 0.74 speed — not shipped for play strength.

sha256 `4b3988c9187ae0d5782ab0600b8e2308760fb530a0fb7012a84a7a1d1ca9328c`

Reach it with `order=builtin` or `order=h0-order-v53` (embedded like `h0-linear-v4`). `py/serve.py` defaults and the browser's Full-strength bot use `oorder=h0-order-v53` for the opponent's replies since brief YA; the bot's own turn uses [`h0-order-v59`](#h0-order-v59).

# `h0-order-v59`

The `h0-order-v59` move-ordering model — `arch: residual` on encoding 3 (`docs/engine-api.md` "Learned value"):
`h0-order-v53`'s linear part unchanged (same features, standardization, vocab and zones) plus a ReLU MLP residual
with 64 hidden units on the 961 standardized features and an 8-dimensional embedding per card and zone (245-id
vocab, 13 zones), `scale` 60. Used only where search ranks one-step values; line scores still use the built-in
value leaf.

## Provenance

Data: the search's per-candidate scores in teach52 (4 096 games), teach54 (2 048) and teach56 (2 048): 2 693 793
rows, split by game (crc32: training 2 152 770, validation 273 470, test 267 553 rows). Training: listwise softmax on
each decision's candidates, τ 3, from `h0-order-v53`, 20 epochs, seed 2, best epoch 10 by validation regret (bots
thread probe `cap57`, trainer `mlp57b.py` — the file's `trained_on.probe` reads `cap57b` — model `M525456-s2`).
On the test games: regret × 0.797 of `h0-order-v53`'s (teach52 0.809, teach54 0.760, teach56 0.805), top-1 0.624
(v53 0.589), recall@4 0.894 (0.869).

## Yardstick

Sweep 59 (engine `2e529fb`, results `sweep59/` at `cf0b817`), all 16 meta decks: `order=<this file>,oorder=<h0-order-v53>`
vs `order=<h0-order-v53>,oorder=1`, both on
`h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,okill=8,hbk=16,alloc=world,threads=1,lostrank=4000,oopen=1,onext=40`:

- main 1 039 / 2 048 = 0.507, reverse 540 / 1 024 = 0.527;
- **pooled 1 579 / 3 072 = 0.514 [0.496, 0.532], +9.7 Elo** (adopted by the registered rule, ≥ 0.51 on the full
  final);
- self-play throughput × 0.937.

On both sides (`oorder=1`) a sibling network (sweep 57, trained without teach56) measured −7.9 Elo: it orders the
bot's own turn only.

sha256 `fdba85d155738881310d714c24ef987cc253fe21fc6d31ca08066d21600ba346`

Reach it with `order=h0-order-v59` (embedded like `h0-order-v53`). `py/serve.py`'s defaults and the browser's
Full-strength bot use `order=h0-order-v59,oorder=h0-order-v53` since brief YA.

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

Yardstick: sweep 46 pooled 0.5094 [0.4999, 0.5188] / +6.5 Elo vs `mulligan-v1` on the 16-deck meta pool (10 752 final games). This was the built-in table until `mulligan-v3` shipped. Reach it with `h0:mull=engine/models/mulligan-v2.json`.

sha256 `efbb96a0efac5cc9e9298e2e1bdbcb84fd0605cc47f13d29fa1f4f7374c6dde5`

## Rule

Committed files are immutable measured artifacts — a new table is a new file (`mulligan-v3`, …) and becomes the default only after it beats the current default on the registered sweep rule; never regenerate a committed file in place.

# `mulligan-v3`

The `mulligan-v3` opening keep table (engine format version 1). Documented in `docs/engine-api.md` "Learned mulligan".

## Provenance

Data: `mull1` + `mull2` + `mull3` pooled via the shrinkage learner (`py/mulligan.py shrink --tags mull1 mull2 mull3 --fresh-only meta-haven-amulet meta-haven-evo meta-haven-kukishiro`, empirical Bayes by rule group).

Yardstick: sweep 51 pooled 0.5060 [0.4944, 0.5176] / +4.2 Elo vs `mulligan-v2` on the 16-deck meta pool (7 168 final games). This is the built-in table since this PR.

sha256 `6bb27768999e896443672ce6f82098ae2a76947b07be5fc94fb42fd7dbce01e7`

## Rule

Committed files are immutable measured artifacts — a new table is a new file (`mulligan-v4`, …) and becomes the default only after it beats the current default on the registered sweep rule; never regenerate a committed file in place.

## Optional race block

Linear models may include an optional `"race"` block (twelve HP / board-attack
inputs; see `docs/engine-api.md` "Optional race block"). The committed defaults
(`h0-linear-v1` … `h0-linear-v4`) have no race block. Train one with
`py/train_value.py --race` on a linear export; gate in play comes after merge.
