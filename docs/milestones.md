# Milestones

| | delivers | gate |
|---|---|---|
| **M0** | this design PR | owner reads and approves the schema |
| M1 | engine core + Basic set (56 cards) + one meta-deck mirror, headless; benchmark | differential harness green vs the old engine on that deck; explicit go/no-go on Rust |
| M2 | all 17 meta decks (199 cards), headless, soak | differential green; matchup statistics possible here, before any UI |
| M3 | all 516 rotation cards + tokens + crests | differential green on the full pool |
| M4 | thin web client over the WASM engine, ported look | e2e parity with the old UI's flows |
| M5 | bot: action encoding, determinized search, policy hooks | mulligan / drawn-card / matchup / tech-card win rates |

## M1 card list

**Source (fetched at M1, not copied from the old repo):** the live rotation meta via the WBArts feed (`GET https://sva.hypd.asia/api/decks?format=rotation&sort=recommended&days=7`, per-deck official Cygames deck-code hash) and Cygames' deck-code endpoint (`POST https://shadowverse-wb.com/web/DeckCode/getDeck`). Card ids are then resolved through `cards/official/catalog.json`.

WBArts from this sandbox (2026-09-09, one GET): **HTTP 403**, Cloudflare challenge HTML (`cf-mitigated: challenge`). Shape unverified here. Cygames `DeckCode/getDeck` was not probed (one-request budget). Re-check at M1.

**Basic:** every catalog id with `set == 10000` and `rotation` (measured **56**).

**Mirror deck name: Aggro Abysscraft.** The choice of archetype may stand. Its card list is re-derived from the source above when M1 starts — do not copy `decks/aggro_abysscraft.json` from the old repo.

Measured (historical counts only, not a source): several metas had 14 distinct cards (the minimum); Aggro Abysscraft had the most Basic-set overlap of those (2 names), so fewest new authorings for a real matchup.

M1 also authors every token/crest those Basic + mirror cards can produce (same `related_card_ids` walk as the pool).

**Gate:** differential green vs the old engine on seeded Aggro Abysscraft mirrors; benchmark games/second for both engines; go/no-go on Rust.

## M2 card list

**Source:** same WBArts + Cygames deck-code path as M1. Do not copy `decks/*.json` from the old repo.

Historical distinct-count measurements (17 named archetypes, **199** distinct rotation ids as of the M0 survey). Lists are fetched at M2.

| archetype | distinct (measured) |
|---|---|
| Aggro Abysscraft | 14 |
| Amulet Havencraft | 14 |
| Antemaria Dragoncraft | 14 |
| Artifact Portalcraft | 16 (includes the fuse chain) |
| Barbaros Swordcraft | 15 |
| Buff Forestcraft | 17 |
| Cutthroat Portalcraft | 38 (largest) |
| Evolution Forestcraft | 16 |
| Evolution Havencraft | 14 |
| Kukishiro Havencraft | 15 |
| Lhynkal Runecraft | 15 |
| Midrange Abysscraft | 15 |
| Rally Swordcraft | 16 |
| Ramp Dragoncraft | 14 |
| Sephie Runecraft | 15 |
| Spell Runecraft | 14 |
| Thestae Forestcraft | 15 |

## M3 card list

All 516 rotation collectibles + 56 reachable tokens + 43 crest/faith entries (38 Crest + 5 Faith). Same derivation as M0 from `cards/official/catalog.json`. Tokens: walk `related_card_ids` from the 516, keep `token == true`.

**Gate:** differential green on the full pool.

## M4 / M5

M4 is a thin web client over a `wasm/` wrapper. M5 is the bot (action encoding, determinized search, policy hooks). Neither starts before the owner approves M0 and the M1 Rust go/no-go.
