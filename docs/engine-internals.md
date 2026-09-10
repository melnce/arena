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

Enhance that does not `replacesBase` (Zeta & Bea) appends to the Fanfare list of the same resolution, so an `as` bound by the base Fanfare is visible to the tier. `replacesBase` (Splendor, L'Age d'Or, Ruthless Eld Sword) swaps the list; `choose pick: all optionsFrom: fanfare` then copies the Fanfare's `options` and runs them in printed order with no player choice.

## This card's cost

`Condition.costEq` is the played instance's current cost. Spells read the cemetery corpse written at play (`cost = paid`). Severed Ties "If this card's cost is 3" therefore sees a cost-set copy as 1 and does not chain. This is not Filter `costEq` (pool / event subject).

## Turn-boundary abilities

`ability_boundary` matches `StartOfTurn` only when `start` and `EndOfTurn` only when `!start`, and only when `a.zone()` matches the scan (field / hand / deck). `queue_turn_boundary` calls the same matcher for both boundaries with that flag, so an `endOfTurn { whose: opponent }` (Enhanced Puppet) does not fire at the opponent's start, and an `endOfTurn { whose: own }` (Puppet Theater, Dark Dimensions) does not fire at the owner's start. `enqueue_boundary` looks up abilities by index (no `Ability` clones) and also scans hand (and deck) so a `zone: hand` endOfTurn (Garodeth) fires; `when` conditions are evaluated at enqueue.

## When events

`Ability::When` is dispatched from game events (`raise_when`). Matching `When` abilities on both players' field cards **and crests** (plus hand/deck when `zone` says so) enqueue into the trigger queue: subject `filter` and `when` conditions at enqueue, `oncePerTurn` honoured, crests before board (active crest 3, active board 4, opponent crest 5, opponent board 6), entry order within a side. Play reactions (`whenever you play`) are flushed onto `pending_work` above Fanfare / spell text (E39). Other cards' enter reactions stay on the queue until the play completes, including across a Fanfare choice (E34). The entrant's own `on:enter` is queued with those reactions and sorts by board age (oldest first) — it is not a priority over older cards' reactions to the same event (E38, pending owner). `pick: entering` reads `State.event_subject`.

`CardDb` builds a static `when` index at load: for each `(EventName, AbilityZone)`, the card ids (and crest ids) that print at least one `When` for that pair. `enqueue_when_on` does not clone zones; it walks field instances whose card id is in the index for `(event, Field)` or whose `granted_whens` count is non-zero (grants are dynamic and rare), crests in the crest index, and hand/deck only when the index has any entry for that `(event, zone)` — today's cards have no deck `When` for most events, so those scans cost nothing. Categories, entry order, `oncePerTurn` marks, and `filter`/`when` evaluation are unchanged.

`CardDb` also indexes start/end-of-turn abilities by `(AbilityZone, start)`. `enqueue_boundary` skips the hand and deck scans when `zone_has_boundary` is empty, so a deck without Sandalphon-style `zone: deck` `startOfTurn` costs nothing.

All 15 `EventName`s are raised where the engine produces them (enter, destroy, play, attack, evolve, draw, earth-rite spend, engage, leader restore, self-buff). None are a silent no-op.

## Field transform

`op:transform` replaces the targeted field instance in its slot with `CardInstance::from_card` of the destination (new instance id). The original is dropped — no Last Words, no shadow, no cemetery, no leave triggers, no compact. The new card is a fresh print (base stats, unevolved, summoning-sick, printed keywords). It is not an enter: no Rally, no `enter_counts` / `enteredThisMatch`, no `on:enter` / `ally_enter`. Rush/Storm on the new card still allow attacking that turn (owner 2026-09-10). In-hand fuse `recipes.transformInto` is the same replacement on the host hand index and is unchanged. Sincerity (`10573310`) targets `any:any` on both boards; `choose {slot}` carries `player` on every option when the pool spans both boards (trace-format / Practice-Tool PR #392). `{slot}` alone means a one-board pool. Old traces that omit `player` prefer the enemy board.

## Invoke

`op:invoke` moves the sourced deck instance onto the field if there is a slot and `State.invoked_ids` does not already contain that card id this boundary window (one copy per name). No RNG pick. Full field: the card stays in the deck and `on:invoked` does not fire. A successful Invoke increments Rally and `enter_counts`, raises enter triggers, then enqueues `on:invoked`. `invoked_ids` is cleared at the start of each start-of-turn boundary.

## One evolve per follower, once per turn

`can_evolve` rejects an already-evolved instance. `rules/official-glossary.md` Evolution: "An evolved follower can't be evolved again" (owner 2026-09-10: cannot EP then SEP later). `legal_actions` therefore offers no super-evolve on a normally evolved follower even with SEP available and the turn unlocked. `PlayerState.evolved_this_turn` is set on a manual EP/SEP evolve and cleared at `begin_turn`; `can_evolve` also rejects both `evolve` and `evolve {super}` for the rest of that player's turn. Effect-granted evolves (`granted: true`) do not set the flag. Camiscilla's "evolve it" on an already-evolved Puppet is a no-op (no stats, no `evolves_used`, no evolve abilities) — same glossary sentence.

## Rally on play

A **played** follower's Rally increment is deferred until the play sequence is quiet (after Fanfare, including any Fanfare choice). Summons and reanimates still increment at entry. A `Fanfare: if Rally (N)` therefore sees the pre-entry count, as does the snapshot during that Fanfare's choice node.

## Resolution

`apply` runs the action then `drain_until_quiet`:

1. Drain the current trigger-queue wave (8-category order, entry-order then printed-order; the whole wave is flushed onto `pending_work` so LIFO still resolves active side first) **unless** the next frame is an index-0 list (Fanfare, a nested body, a freshly flushed trigger) or a trigger wave is still in flight (`RestoreBindings` still on the stack). A trigger raised while a queued item resolves goes to the back of the queue (E32 / Grimnir: (2) and (3) before the Last Words (4) that (1) just queued). **E40:** when a flushed trigger (`When`, turn-boundary, enter — not Last Words / Leave / Strike / Clash) comes up for resolution, skip it if its `SourceRef` is a field instance that is no longer on the field (destroyed, banished, bounced, transformed — transform is a new instance), a crest no longer at that index, or a hand/deck instance that is in neither zone (deck-zone start-of-turn is stored as `SourceRef::Hand`). Last Words are `SourceRef::Spell` and Leave is raised because the source left, so they are exempt. Same skip applies to play-reaction items flushed onto `pending_work` (E39). Source: Shadowverse 効果処理 wiki, https://w.atwiki.jp/svkoukasyori/pages/16.html — 「ラストワード・「場を離れる時」以外の効果は、解決前に効果を持ったカードが場を離れた場合解決されない。」 plus 「ひとつの効果の解決中に他の誘発効果は割り込まない」. Owner 2026-09-10: Trap in the Woods vs three Knights kills only the first.
2. Pop newly pushed effect lists and aftermaths (combat damage, turn-boundary step 7/8). Nested bodies sit on top of the enclosing remainder.
3. Settle 0-defense deaths (by instance id).

Reactions to an op of an in-flight list that is *not* inside a flushed wave (`ally_draw` after `draw count: N`) still run before the next op of that list (E28). Countdown expiry captures doomed amulets/crests by instance id / `granted_order` before any destroy, so compact cannot retarget a neighbour. A `countdown delta` over a selector (Barbaros) likewise captures each hit by instance id before applying, so a Flag that reaches 0 and is destroyed does not steal the next Flag's slot. Fuse partner `legal` is `choose {card}` like every other hand choice.

That order is what makes play reactions (`whenever you play`) resolve before Fanfare (E39), other cards' enter reactions wait until the play completes (E34), the entrant's own `on:enter` sort with those reactions by board age (E38), Strike/Clash precede combat damage, and the start-of-turn draw happen at step 8 after the queued boundary abilities.

`apply_attack` writes `State.combat_opposing` (the attack target as a `TargetOpt`) before queuing Strike / Follower Strike / Clash, and clears it after combat damage. `pick: opposing` reads that slot (Okita's "the opposing follower"). If Strike or Clash has already reduced a combatant to 0 defense, `combat_damage` does not exchange hits.

`grantTraits.until` (`endOfTurn` / `endOfOpponentTurn`) is caster-relative: `endOfOpponentTurn` expires when the caster's opponent's turn ends, even if the grant sits on an enemy follower (Measured Attunement / Shaili). Grants are stored on `CardInstance.temp_traits` and `merge_remove`'d at that boundary.

A `countdown` selector with `zone: crests` adjusts `CrestInstance.countdown` in place (Majestic Conquest "Delay the count of your Crest … by 2"). `filter.card` `10622310` matches crest id `crest:10622310`.

`summon { from }` **moves** the selected instance onto the field (Chloe "summon it"); a full field leaves the card where it is. `summon { copyOf }` always copies (exact or printed) regardless of zone — the hand/deck/field original stays (Cartographer). `addToHand { copyOf }` of several deck targets copies each resolved instance and does not remove the originals (Wolfraud exact copies).

Hand-zone `when ally_draw` fires only on the drawn instance (`note_draw` takes the last same-id in hand). Other copies of the same card already in hand stay quiet (Swift Staffmaster).

A super-evolved follower on its owner's turn is still a legal `destroy` candidate; `destroy_by_ability` fizzles via `cantBeDestroyedByAbilities` / own-turn SE protection (E31). Lethal 0-defense still settles. The candidate pool is unchanged so `random_target` picks still match.

Lethal **damage** marks a follower destroyed (`defense <= 0`) and it stays in its slot — not a candidate, not attackable — until pending work is quiet, when deaths settle together and Last Words queue (rulebook Meteor / simultaneous destruction). Explicit `destroy` / `banish` remove at once (Last Words still wait in the queue). An op's targets are selected when that op is reached (after previous ops in the list), then captured by instance id for that op's applications only. A nested body (`repeat`, `if`/`else`, `seq`, `choose` options) is pushed on top of the enclosing remainder and resolves completely before the next enclosing op.

Super-evolve knockback (1 to the enemy leader when the SE attacker destroys the defender on its owner's turn) keys off the defender's instance id captured at attack declaration — not the follower that compacted into that slot after deaths.

`arena-replay` (and library replay) compare only `phase` and `winner` when both snapshots are `phase: terminal`; the rest of the state and `legal` are post-mortem.

When an effect list pauses for a player choice, the reactive queue is **not** drained first; it drains when the list completes (owner ruling 2026-09-10: Vorlalai discarded by Spilling Red summons after the destroy, not before the second selection). Choice-node snapshots therefore do not show reactions to earlier clauses of the same list.

A completed old-engine `fuse { host_pos, partner_pos }` line is applied by `apply_neutral`: start the fuse, map each `partner_pos` (pre-action hand position) to the index in `options`, then Confirm. Confirm with no partners is not legal. `Choose` with an out-of-range index is `NotLegal`.

## Skybound Art

The per-hand `skybound` counter is the number of allied evolves (player EP and effect-granted, ruling 2026-08-10) witnessed while **that copy** was in hand. It is stored only on hand instances whose printed text has a `skyboundArt` condition, starts at 0 for a newly added copy, and is omitted from the snapshot at 0. Evaluation adds the current round (`State.turn`, equal to the acting player's `turns_taken`) so the gauge is `turn + skybound`; Skybound Art fires at ≥ 10, Super Skybound Art at ≥ 15. The turn is not stored on the instance.

## Faith

At `new_game`, after decks and opening hands are dealt, every player whose starting deck or opening hand contains a card that carries a Faith gains that Faith crest (`faith:<id>`, no countdown, `faith: 0`) — the Sham-Nacha / engine-api rule. The crest's `when ally_evolve` increments `PlayerState.faith`. `pay faith N` spends only if the value is ≥ N, else the wrapped body fizzles. `grantAbility` onto `zone: crests, kind: faith` appends to `CrestInstance.granted` (not visible in CanonicalState; it fires when the event hits). Faith counts toward the five-icon cap but not toward "the number of crests" (rulings 2026-09-05/06).

## E39 — play reactions before Fanfare / spell text

Play reactions (`ally_card_played` / `ally_follower_played` / `ally_spell_played`, "whenever you play …", crests included) resolve when the card is played, **before** its Fanfare or spell body, crest-then-board. Enter reactions ("whenever … enters the field") still wait until the play sequence finishes, including a Fanfare choice (E34).

Official Cygames Q&A, World of Games (`10503210`): _"The only card on my field is a World of Games with a count of 5, and the only enemy card on the field is a Quake Goliath. If I play Divine Thunder, what will World of Games's count be?"_ → _"Its count will be 4."_ Divine Thunder (`10103310`) destroys the 4-cost Goliath; the count still advances, so the play reaction resolved before the spell. Owner 2026-09-10: _"WoG says 'whenever you play' so it should die on play and make space."_ Implementation: take the play-reaction queue items after `raise_when(ally_card_played)` (and `ally_spell_played`), place the card / push Fanfare or spell body, then flush those items onto `pending_work` above Fanfare; Last Words they cause (`FlushPlayLastWords`) still run before the summons. `next_frame_allows_queue_drain` keeps waiting while Fanfare is index-0 so leftover enter reactions wait.

The same Q&A is why World of Games counts a same-base-cost card on either side (`WORLD_OF_GAMES_COUNTS_EITHER_SIDE`; owner _"yes any card"_). The old engine counted allied cards only.

## E37 — evolve reactions before the evolving follower's Evolve list

An evolve has no Fanfare-style exception, so the general same-timing rule applies: crests first, then board abilities (turn boundaries: start-of-turn crests are step 2, board abilities step 3; same-timing crests resolve in grant order).

The Faith's "Whenever an allied follower evolves, increase this faith's value by 1" and the evolving follower's printed `Evolve:` / `Super-Evolve:` are both triggered by the evolve. The crest resolves first, so at the Evolve ability's choice node the faith already shows +1. Implementation: `raise_when(ally_evolve)` is flushed onto `pending_work` **above** the evolving follower's Evolve/Super-Evolve list; reactions raised *during* that list still wait (A2 / enter-play E34). **Owner 2026-09-10:** _"evolve comes first so the faith ticks up first."_ No official Cygames Q&A exists for this ordering.

## E36 — `random_target` among surviving board cards

A `side: any` random pool (Oluon) qualifies keys with the player (`a:slot:0`, `leader:b`) so two leaders or two `slot:0`s do not collapse on scripted replay. Single-side pools still emit `slot:N` / `leader`.

A `side: any` random pool (Oluon) qualifies keys with the player (`a:slot:0`, `leader:b`) so two leaders or two `slot:0`s do not collapse on scripted replay. Single-side pools still emit `slot:N` / `leader`.

Recorded `chose.slot` is the 0-based index among **surviving** cards on that player's field at roll time (followers at 0 defense / marked for destruction and amulets at countdown 0 are skipped; order preserved), not the raw field slot. In a `randomDistinct` wave the first chosen slot is also skipped at later rolls (the old engine applies that destroy before the next roll). A non-distinct `random` wave does not skip a follower that survived the first pick — it stays in the numbering. Live play still picks by index into the candidate list. Scripted replay matches the survivor-index label first; if that misses, a raw field slot is accepted as an alias when that label is not already a survivor key of another candidate (M1 ramp traces numbered by raw slot). Aliases are not added for live RNG.

## Questions for the owner

Asked; arena follows the rulebook/text until he says otherwise.

1. **World of Games counting itself.** A later 1-cost play sees World of Games (base 1) as "a card on the field other than it". The printed "other than it" excludes only the played card.
2. **Granted `attacksPerTurn: 2` after attacks already made this turn.** Rulebook is silent. Implemented as `attacks_left = attacks_left.max(n)`.
3. **Duplicate-id draw after `returnToDeck`.** A draw of an id that has both a modified copy and a just-returned printed copy takes the oldest (first in vec; return appends).
4. **E38 — entrant's own `on:enter` vs older `ally_enter`.** Implemented as same-timing board enter triggers, oldest first (not a jump onto `pending_work` above Fanfare). Pending owner.
5. **E40 — source must still be in its zone.** See Resolution §1. Trap in the Woods vs a 3-Knight summon: three `enemy_follower_enter` items queue (no mid-effect interrupt); the first destroys the first Knight and the trap; the other two skip.

## Defense debuff and `max_defense`

A stat debuff lowers `max_defense` by the same amount; current defense drops by the same amount; healing restores up to the new maximum. Example: a 7/5 (max 7) given −0/−4 becomes 7/1 (max 3). Owner ruling 2026-09-10: modifications define the max (Azurifrit's "fully restore" goes to the buffed maximum).

## State notes

- Field slots are entry order, compacted on leave.
- Deck is an unordered `Vec` treated as a multiset; draws pick uniformly via the state's RNG. When several copies of an id differ (Thestae's crest +1/+1 on a deck follower vs a copy just returned from hand), a recorded `draw` of that id takes the first copy in vec order. `returnToDeck position: random` appends (the old emitter's shuffle is `raw` and ignored) so that copy is the one that has been in the deck longest.
- `enter_counts` is every follower entry this match (play, summon, reanimate), keyed by card id. Obsessed Test Subject's "5 other allied copies have entered" subtracts this copy. Reanimate is an enter (glossary: it summons a copy onto the field).
- `Condition.deckHasNoDuplicates` is a deck-multiset uniqueness check (Cutthroat / Bluerust). `pick: lowest` over leaders returns **all ties** (Tyrannical Fists Q&A). `grantTraits.until` stores instance `temp_traits` (caster-relative; see above) and `merge_remove`s at the matching end-of-turn. `leaderModifier.damageTakenBonus` applies to the **selected** leader and stacks (Beelzebub Q&A). Crest Last Words fire when countdown hits 0 (`expire_crest`). `enemy_follower_destroyed` is raised on the opponent of the destroyed follower's owner (Lifestealer). `addToDeck` appends (`position: random` is a multiset). Multi-choose binds **append** across remaining picks of the same op (`bind_append`) so a later `bound` sees every pick. `op: select` opens a choice and binds (`as`) with no other effect — an empty pool binds nothing, so a later `Amount.stat` is 0 (Cassius `10473110`).
- Earth sigils: a counter plus `earth_slot` (which amulet holds the stack). When an Earth Sigil amulet enters, every other allied Earth Sigil is **banished** (no shadow, no Last Words) and the new amulet takes their counts (official glossary Earth Sigil; owner 2026-09-10 "yes banish them instead"). "Gain X earth sigils" increments the holder on the field, else summons one Magic Sediment with count X; no holder and a full board loses the sigil. A full board still blocks *playing* an Earth Sigil amulet (ruling 2026-09-10).
- `hash` is FNV-1a 64 of the sorted-key canonical snapshot JSON.

## Tests

Integration tests under `engine/tests/` load the repo's `cards/` plus fixtures. Soak is opt-in: `ARENA_SOAK_GAMES=N cargo test --release soak`.
