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

`as` / `{pick: bound, ref}` names live in `State.bindings` for one **resolution**. The map is cleared at the start of `apply`. Nested `seq` / `if` / `pay` frames share the live map, so an Evolve ability's `as: "g"` is visible to the Super-Evolve ability of the same card when both fire (rulebook: a super-evolve fires both lines unless the super line says `instead`). A reactive-queue drain between ops of the same list saves the map and restores it after the wave (`Aftermath::RestoreBindings`), so an enclosing `as` name survives enter-triggers that run before the next op (Netherworld Lieutenant: summon `as: s`, then buff the bound copy). Queued triggers themselves start from an empty map. A `ref` with no live binding is an empty set — never a runtime error. `CardDb::load` rejects a `ref` that no `as` on the same card could produce (`LoadError::UnboundRef`).

## Turn-boundary abilities

`ability_boundary` matches `StartOfTurn` only when `start` and `EndOfTurn` only when `!start`, and only when `a.zone()` matches the scan (field / hand / deck). `queue_turn_boundary` calls the same matcher for both boundaries with that flag, so an `endOfTurn { whose: opponent }` (Enhanced Puppet) does not fire at the opponent's start, and an `endOfTurn { whose: own }` (Puppet Theater, Dark Dimensions) does not fire at the owner's start. `enqueue_boundary` looks up abilities by index (no `Ability` clones) and also scans hand (and deck) so a `zone: hand` endOfTurn (Garodeth) fires; `when` conditions are evaluated at enqueue.

## When events

`Ability::When` is dispatched from game events (`raise_when`). Matching `When` abilities on both players' field cards **and crests** (plus hand/deck when `zone` says so) enqueue into the trigger queue: subject `filter` and `when` conditions at enqueue, `oncePerTurn` honoured, active side category 4 then opponent 6, entry order within a side. The played card's Fanfare sits on `pending_work`; enter/play reactions stay on the queue until the play completes, including across a Fanfare choice (E34). The entrant's own `on:enter` is queued with those reactions and sorts by board age (oldest first) — it is not a priority over older cards' reactions to the same event (E38, pending owner). `pick: entering` reads `State.event_subject`.

`CardDb` builds a static `when` index at load: for each `(EventName, AbilityZone)`, the card ids (and crest ids) that print at least one `When` for that pair. `enqueue_when_on` does not clone zones; it walks field instances whose card id is in the index for `(event, Field)` or whose `granted_whens` count is non-zero (grants are dynamic and rare), crests in the crest index, and hand/deck only when the index has any entry for that `(event, zone)` — today's cards have no deck `When` for most events, so those scans cost nothing. Categories, entry order, `oncePerTurn` marks, and `filter`/`when` evaluation are unchanged.

`CardDb` also indexes start/end-of-turn abilities by `(AbilityZone, start)`. `enqueue_boundary` skips the hand and deck scans when `zone_has_boundary` is empty, so a deck without Sandalphon-style `zone: deck` `startOfTurn` costs nothing.

All 15 `EventName`s are raised where the engine produces them (enter, destroy, play, attack, evolve, draw, earth-rite spend, engage, leader restore, self-buff). None are a silent no-op.

## Field transform

`op:transform` replaces the targeted field instance in its slot with `CardInstance::from_card` of the destination (new instance id). The original is dropped — no Last Words, no shadow, no cemetery, no leave triggers, no compact. The new card is a fresh print (base stats, unevolved, summoning-sick, printed keywords). It is not an enter: no Rally, no `enter_counts` / `enteredThisMatch`, no `on:enter` / `ally_enter`. Rush/Storm on the new card still allow attacking that turn (owner 2026-09-10). In-hand fuse `recipes.transformInto` is the same replacement on the host hand index and is unchanged. Sincerity (`10573310`) targets `any:any` on both boards; `choose {slot}` includes `player` only when that slot number is occupied on both sides (old traces omit it and prefer the enemy board).

## Invoke

`op:invoke` moves the sourced deck instance onto the field if there is a slot and `State.invoked_ids` does not already contain that card id this boundary window (one copy per name). No RNG pick. Full field: the card stays in the deck and `on:invoked` does not fire. A successful Invoke increments Rally and `enter_counts`, raises enter triggers, then enqueues `on:invoked`. `invoked_ids` is cleared at the start of each start-of-turn boundary.

## One evolve per turn

`PlayerState.evolved_this_turn` is set on a manual EP/SEP evolve and cleared at `begin_turn`. `can_evolve` rejects both `evolve` and `evolve {super}` for the rest of that player's turn. Effect-granted evolves (`granted: true`) do not set the flag.

## Rally on play

A **played** follower's Rally increment is deferred until the play sequence is quiet (after Fanfare, including any Fanfare choice). Summons and reanimates still increment at entry. A `Fanfare: if Rally (N)` therefore sees the pre-entry count, as does the snapshot during that Fanfare's choice node.

## Resolution

`apply` runs the action then `drain_until_quiet`:

1. Drain the current trigger-queue wave (8-category order, entry-order then printed-order; the whole wave is flushed onto `pending_work` so LIFO still resolves active side first) **unless** the next frame is an index-0 list (Fanfare, a nested body, a freshly flushed trigger) or a trigger wave is still in flight (`RestoreBindings` still on the stack). A trigger raised while a queued item resolves goes to the back of the queue (E32 / Grimnir: (2) and (3) before the Last Words (4) that (1) just queued).
2. Pop newly pushed effect lists and aftermaths (combat damage, turn-boundary step 7/8). Nested bodies sit on top of the enclosing remainder.
3. Settle 0-defense deaths (by instance id).

Reactions to an op of an in-flight list that is *not* inside a flushed wave (`ally_draw` after `draw count: N`) still run before the next op of that list (E28). Countdown expiry captures doomed amulets/crests by instance id / `granted_order` before any destroy, so compact cannot retarget a neighbour. Fuse partner `legal` is `choose {card}` like every other hand choice.

That order is what makes Fanfare complete before other cards' enter/play reactions (E34), the entrant's own `on:enter` sort with those reactions by board age (E38), Strike/Clash precede combat damage, and the start-of-turn draw happen at step 8 after the queued boundary abilities.

A super-evolved follower on its owner's turn is still a legal `destroy` candidate; `destroy_by_ability` fizzles via `cantBeDestroyedByAbilities` / own-turn SE protection (E31). Lethal 0-defense still settles. The candidate pool is unchanged so `random_target` picks still match.

Lethal **damage** marks a follower destroyed (`defense <= 0`) and it stays in its slot — not a candidate, not attackable — until pending work is quiet, when deaths settle together and Last Words queue (rulebook Meteor / simultaneous destruction). Explicit `destroy` / `banish` remove at once (Last Words still wait in the queue). An op's targets are selected when that op is reached (after previous ops in the list), then captured by instance id for that op's applications only. A nested body (`repeat`, `if`/`else`, `seq`, `choose` options) is pushed on top of the enclosing remainder and resolves completely before the next enclosing op.

Super-evolve knockback (1 to the enemy leader when the SE attacker destroys the defender on its owner's turn) keys off the defender's instance id captured at attack declaration — not the follower that compacted into that slot after deaths.

`arena-replay` (and library replay) compare only `phase` and `winner` when both snapshots are `phase: terminal`; the rest of the state and `legal` are post-mortem.

When an effect list pauses for a player choice, the reactive queue is **not** drained first; it drains when the list completes (owner ruling 2026-09-10: Vorlalai discarded by Spilling Red summons after the destroy, not before the second selection). Choice-node snapshots therefore do not show reactions to earlier clauses of the same list.

A completed old-engine `fuse { host_pos, partner_pos }` line is applied by `apply_neutral`: start the fuse, map each `partner_pos` (pre-action hand position) to the index in `options`, then Confirm. Confirm with no partners is not legal. `Choose` with an out-of-range index is `NotLegal`.

## Defense debuff and `max_defense`

A stat debuff lowers `max_defense` by the same amount; current defense drops by the same amount; healing restores up to the new maximum. Example: a 7/5 (max 7) given −0/−4 becomes 7/1 (max 3). Owner ruling 2026-09-10: modifications define the max (Azurifrit's "fully restore" goes to the buffed maximum).

## State notes

- Field slots are entry order, compacted on leave.
- Deck is an unordered `Vec` treated as a multiset; draws pick uniformly via the state's RNG.
- Earth sigils: a counter plus `earth_slot` (which amulet holds the stack). Merge: collectible replaces token (ruling 2026-08-30). A full board blocks playing an Earth Sigil amulet; "gain an earth sigil" still increments the existing stack (ruling 2026-09-10).
- `hash` is FNV-1a 64 of the sorted-key canonical snapshot JSON.

## Tests

Integration tests under `engine/tests/` load the repo's `cards/` plus fixtures. Soak is opt-in: `ARENA_SOAK_GAMES=N cargo test --release soak`.
