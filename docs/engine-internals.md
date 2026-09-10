# Engine internals (M1)

How `arena-engine` is wired. The public contract is `docs/engine-api.md` and `docs/trace-format.md`.

## Crate

`engine/` is the `arena-engine` lib plus three bins:

- `arena-trace` — seeded random-legal self-play → JSONL
- `arena-replay` — ScriptedRng replay + snapshot/legal diff
- `arena-bench` — one JSON line of games/second

`#![forbid(unsafe_code)]`. No async, no wasm/js deps, no trait objects where an enum does.

## Card load

`CardDb::load(root)` walks `cards/**/*.json` except `cards/official/catalog.json`. The catalog is loaded as facts (keys that are 8-digit ids; `_meta` is ignored). Fixture cards live under `engine/tests/fixtures/cards/` and are merged with `load_extra_dir` — they are not authored under `cards/` (RD's partition).

A card whose data uses an M1-unsupported construct fails at `require_supported` with `Unsupported { card, construct }`. `legal_actions` never offers a play of such a card.

## Bonus PP toggle

`Action::BonusPp` is a toggle, matching the old engine (`canToggleSecondPlayerBonusPp`). The second player may activate the current-tier charge (early: their turns ≤ 5; late: from turn 6) and may cancel while the bonus orb is unspent (`pp_bonus > 0`), regardless of regular PP (regular orbs are spent first; the bonus orb last). Once the orb is spent, the toggle is locked for the rest of the turn and cancel is not offered. End of turn (after EOT effects, step 7) commits the charge. Activate → cancel → activate in one turn is legal.

## Bindings

`as` / `{pick: bound, ref}` names live in `State.bindings` for one **resolution**. The map is cleared at the start of `apply` and before each queued trigger. Nested `seq` / `if` / `pay` frames inherit the current map, so an Evolve ability's `as: "g"` is visible to the Super-Evolve ability of the same card when both fire (rulebook: a super-evolve fires both lines unless the super line says `instead`). A `ref` with no live binding is an empty set — never a runtime error. `CardDb::load` rejects a `ref` that no `as` on the same card could produce (`LoadError::UnboundRef`).

## Resolution

`apply` runs the action then `drain_until_quiet`:

1. Continue an in-flight `Effects` frame (`index > 0`) — never interrupt the resolving effect.
2. Drain the FIFO trigger queue (8-category order, entry-order then printed-order).
3. Pop newly pushed effect lists and aftermaths (combat damage, turn-boundary step 7/8).
4. Settle 0-defense deaths.

That order is what makes enter-reactions precede Fanfare, Strike/Clash precede combat damage, and the start-of-turn draw happen at step 8 after the queued boundary abilities.

## State notes

- Field slots are entry order, compacted on leave.
- Deck is an unordered `Vec` treated as a multiset; draws pick uniformly via the state's RNG.
- Earth sigils: a counter plus `earth_slot` (which amulet holds the stack). Merge: collectible replaces token (ruling 2026-08-30).
- `hash` is FNV-1a 64 of the sorted-key canonical snapshot JSON.

## Tests

Integration tests under `engine/tests/` load the repo's `cards/` plus fixtures. Soak is opt-in: `ARENA_SOAK_GAMES=N cargo test --release soak`.
