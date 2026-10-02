# Oracle card overlay — 2026-09-29 balance patch

Traces under `oracle/traces/` were recorded against pre-patch card data. When Cygames changes cost, stats, or printed text, replay would diverge for a **data** reason even if rules are unchanged.

These seven files are byte-identical to `git show 0276688:<path>` for the cards changed on 2026-09-29 (UTC):

| id | card |
|---|---|
| `10423110` | Golden Knight, True King's Blade |
| `10633310` | Bewitching Eld Crystals |
| `10921110` | Open-Sea Scout |
| `10922110` | Whirlpool Gunner |
| `10972310` | Disgraceful Banishment |
| `10973310` | Soulforge |
| `10974110` | Cutthroat, Fluxblade Convict |

Only the oracle gate (`engine/tests/oracle.rs`, `arena-replay`, `arena-oracle-audit`) loads this overlay after `cards/**`, replacing those ids. Nothing else uses it.
