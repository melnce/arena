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
