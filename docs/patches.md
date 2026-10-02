# Balance patch log

## 2026-09-29 (UTC)

- **Source:** [Cygames maintenance notice](https://shadowverse-wb.com/en/news/detail/?id=01M1GEHYM6FH0137A65FTXFWBQ)
- **Arena commit:** _(filled at merge)_

| id | card | change |
|---|---|---|
| `10423110` | Golden Knight, True King's Blade | cost 7 → 6; `Enhance (9)` → `Enhance (8)` |
| `10633310` | Bewitching Eld Crystals | cost 3 → 4; `Enhance (5)` → `Enhance (6)` |
| `10921110` | Open-Sea Scout | defense 1 → 2 |
| `10922110` | Whirlpool Gunner | attack 3 → 4, defense 1 → 2 |
| `10973310` | Soulforge | cost 4 → 3 |
| `10974110` | Cutthroat, Fluxblade Convict | cost 2 → 1, 2/2 → 1/1; Evolve text new; `related_card_ids` `[10974110]` → `[]` |
| `10972310` | Disgraceful Banishment | text new (search draw + draw 2) |

## Procedure for the next patch

1. `node tools/fetch-official.mjs` — diff committed `cards/official/catalog.json`; stop if more than expected cards changed.
2. `node tools/apply-official-catalog.mjs` — only touched card files should change.
3. Hand-author changed printed text / modes the catalog tool cannot infer.
4. Update tests (grep card ids in `engine/tests/` and `ui/tests/`).
5. Add or extend `oracle/cards-as-recorded/` for any card present in committed traces.
6. Re-pin only where a game or fixture containing a patched card changes outcome; document each row in the PR.
