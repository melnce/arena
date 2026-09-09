# Design

`arena` is a from-scratch rebuild of a practice tool that grew for a year without a schema. The same effect was authored five ways; six top-level keys plus a `triggers` array expressed one concept. This milestone freezes the replacement: a closed card schema, a headless engine contract, and a shared trace so the old engine can stay the differential oracle.

The end goal (2026-09-05) is matchup simulation statistics — mulligan win rates, drawn-card win rates, whether tech cards matter — without underrating decision-heavy decks versus top-human play. That goal is throughput-bound and search-bound. Flat cloneable state, a multiset deck, a seeded RNG inside the state, and statically enumerable actions are not style; they are the search budget.

## Why a closed schema

Measured on the old authored sets at `8f491f9` (`python3` walk of every object with an `op` key under `cards/sets/*.json`):

- The `damage` op has **18 distinct field names** across **339** uses (`op`, `target`, `amount`, `select`, `count`, `distribution`, `printed`, `amount_source`, `can_target_leader`, `add_amount`, `exclude_selected`, `stat`, `condition`, `spill_to_leader`, `pick`, `include_leader`, `rank`, `filter`).
- **34** distinct ops; **90** distinct field names across those ops.
- One concept (a triggered ability) is spread across `fanfare`, `evolve`, `superevolve`, `spell`, `keywords`, `crest`, plus a `triggers` array.

Reviewers spent a fortnight writing gates that a type system would have refused. The new schema uses `additionalProperties: false` on every object and `oneOf` + a `const` discriminator. One effect, one construction. A value with no card in the 572+43 pool is deleted.

## Traits, abilities, modes

- **Traits** are a flat struct of combat-rule modifiers (`ward`, `storm`, `damageCap`, …). No effects, no nesting. Granting or removing them at runtime is `grantTraits` / `removeTraits`.
- **Abilities** are one uniform list. `on` is a closed tagged union. Evolve is a trigger and nothing else: the EP/SEP resource, the +2/+2, and the turn thresholds are rules.
- **Modes** (Enhance / Accelerate / Crystallize) are data. Which mode applies is a rule (Enhance is never declinable; Accelerate / Crystallize are never available when the normal play is affordable; when several alternates are payable the highest payable one applies). A card in hand yields at most one play action.

## Printed literals

Every ability, mode, choose option, and clause-root effect carries `printed`. Ability/mode `printed` is a whitespace-normalised substring of the card's `text`. Clause-root `printed` is a substring of its ability. Two clause roots under one ability may not share a sentence. `tools/validate.mjs` enforces the relation; the schema enforces presence.

A separate printed sentence is an independent clause root (owner ruling 2026-09-08). "… and if you did so" is a `seq` with `did`.

## Flat state, multiset deck, RNG in state

Followers and amulets occupy slots 0..4 per player in entry order. The deck is an unordered multiset of card instances; a draw is a uniform random pick, never "the top card" (measured: no card in the pool refers to the top or bottom of a deck). That makes determinization cheap for the known-decklist bot and lets the differential harness replay draws as recorded `Pick`s.

The RNG lives inside the state (seedable, cloneable). A clone is a complete fork. `ScriptedRng` replays recorded outcomes. No global RNG.

## Headless

The engine crate has no `web-sys` / `js-sys` / `wasm-bindgen` dependency. CI asserts it. A `wasm/` crate (M4) and a `py/` crate (M5) wrap it later.

## Differential oracle

The old repo stays running. Both engines emit [`trace-format.md`](trace-format.md). First divergence wins. Adjudication: owner ruling > printed text as clarified by official Q&A > rulebook synthesis.

## Rotation scope

M0–M3 cover Rotation only: 516 rotation-legal collectibles, 56 tokens reachable by walking `related_card_ids` from those 516, and 43 crest/faith entries (38 Crest + 5 Faith). Unlimited-only cards are out of scope except `10131320` Stormy Blast (set 10001, rotation-legal). Fuse is in scope: the Artifact chain is reachable from five rotation cards and used by the Artifact Portalcraft meta deck.

## Second-system risk

A rewrite that authors 572 cards into a new shape can silently change meaning. The milestones defend against that:

- M0 freezes the schema before any engine logic.
- M1 is Basic (56) plus one meta-deck mirror, with a differential gate against the old engine on that deck, plus an explicit go/no-go on Rust.
- M2 is the 17 meta decks (199 distinct cards) before any UI.
- M3 is the full Rotation pool.
- Honest `needs:` / `question:` rows beat fifty rows that mostly work.

## Rules vs data

These stay in the engine. Each cites its source.

| Rule | Source |
|---|---|
| Turn structure; two-phase start/end-of-turn sequences | `rules/rulebook.md` Turn Sequence / Start-of-Turn and End-of-Turn Sequences |
| PP / EP / SEP | `rules/rulebook.md` Play Point (PP) and Evolution Point Rules |
| Bonus PP (second player, two charges, pre-turn-6 and turn-6+; usable PP may reach 11) | `rules/rulebook.md` Play Point rules |
| Evolve thresholds 5/4, super 7/6 by player turn count | `rules/rulebook.md` Evolution Point Rules |
| +2/+2 evolve, +3/+3 super-evolve | `rules/rulebook.md` (stat change is a rule; no authored evo stats — measured: official data has no per-card evo ATK/DEF override in the pool) |
| Super-evolve own-turn protection and 1-damage knockback | `rules/rulebook.md` Super-Evolve |
| Trait semantics (Ward, Storm, Rush, Ambush, Aura, Intimidate, Bane, Drain, Barrier, damage caps) | `rules/rulebook.md` keyword catalogue |
| Trigger queue FIFO append, no interruption; 8-category resolution order; entry-order tie-break; printed-order within one card | `rules/rulebook.md` Timing Windows and Trigger Resolution |
| State-based condition check at trigger creation | `rules/rulebook.md` Timing Windows |
| Combat sequence: Strike/Clash → damage → destruction → knockback → Last Words | `rules/rulebook.md` Combat |
| Play sequence: enter reactions → Fanfare → crest reactions → board reactions | `rules/rulebook.md` / playcard contract orientation |
| Hand limit 9; overflow destroys without Last Words | `rules/owner-rulings.md` Hand overflow destroys without Last Words — 2026-08-10 |
| Field 5; failed summons do not count for Rally | `rules/owner-rulings.md` Rally — 2026-08-12 |
| Deck-out on draw | `rules/rulebook.md` Win/Loss |
| Simultaneous leader lethal → active player loses | `rules/rulebook.md` Win/Loss |
| Sequential multi-summon; each entry judged at its own moment | `rules/owner-rulings.md` Every multi-card summon resolves one card after another — 2026-09-05 |
| Board compaction before a Last Words summon | `rules/owner-rulings.md` Last Words summons never spawn in place — 2026-09-05 |
| Transform is neither leave nor enter; the new card may attack | `rules/owner-rulings.md` Transform — neither leave nor enter — 2026-09-08 |
| Split damage allocation oldest-first; Barrier consumes its allocation | `rules/owner-rulings.md` Barrier inside split damage — 2026-08-29 |
| Enhance / Accelerate / Crystallize gating and permanence (alternate form takes base cost N; alternate costs never move; normal play preferred) | `rules/owner-rulings.md` Accelerate / Crystallize — 2026-09-06; Alternate-form permanence — 2026-09-02; Alternate and Enhance costs are fixed — 2026-09-02; Multi-tier Enhance — 2026-08-15 |
| Fuse: once per turn per instance, legal at 0 PP, partners banished, was-fused flag per-instance, α remembers kinds across turns, gears fuse only with gears | `rules/owner-rulings.md` Fuse mechanics; Fused cards are banished; Artifact fuse chain; official Q&A `90073110` |
| Reanimate: field-destroyed only, highest cost ≤ X, random among ties | `rules/owner-rulings.md` Reanimate only sees field-destroyed followers — 2026-09-02 |
| Rally counting | `rules/owner-rulings.md` Rally — 2026-08-12 |
| Overflow | `rules/rulebook.md` class keywords |
| Skybound gauge: every allied evolve counts | `rules/owner-rulings.md` Skybound gauge — 2026-08-10 |
| Spellboost fires per spell played for cards in hand | `rules/rulebook.md` Spellboost |
| Earth Sigil stack; merge survivor (collectible beats token) | `rules/owner-rulings.md` Witch's New Brew always wins an Earth Sigil merge — 2026-08-30 |
| Combo counts the played card | `rules/official-qa.md` `10012110` May, Journey Elf |
| Necromancy auto-pays | `rules/rulebook.md` Necromancy |
| "Select" is forced; a spell with no legal target is unplayable; followers/amulets fizzle; hand selections need N candidates | `rules/owner-rulings.md` "Select" is forced — 2026-08-16; Playability with no "Select" target — 2026-08-16; A printed "Select" means the player picks — 2026-09-08 |
| Deaths settle before dependent triggers pick targets | `rules/owner-rulings.md` Deaths settle before dependent triggers pick targets — 2026-08-23 |
| `attacksPerTurn` grant = max(current, N) | `rules/rulebook.md` / combat keywords |
| "Takes N more damage" stacks and applies to a 0-damage event | `rules/owner-rulings.md` Beelzebub — 2026-09-07; "Takes N more damage" applies to a 0-damage event — 2026-08-31 |
| Damage +/- modifiers apply before set-damage | `rules/rulebook.md` damage |
| Last Words do not fire on banish/transform | `rules/rulebook.md`; Transform ruling |
| `_destroyed` events fire only on destruction | `rules/owner-rulings.md` `_destroyed` events — destruction only — 2026-09-08 |
| Hidden information is always visible in the practice tool; the engine keeps perfect information; observation masking is M5 | `rules/owner-rulings.md` Hidden information — always visible — 2026-08-13 |
| Crest/Faith cap of five; a duplicate bounces (no refresh); a sixth is ignored; Faith does not count as a crest for "number of crests"; crest Countdown ticks at the owner's start of turn | `rules/owner-rulings.md` Crest and Faith slots — 2026-09-05; A duplicate crest bounces — 2026-09-05; Faith is not a crest for counting — 2026-09-06 |
| `randomDistinct` vs `random`+`repeat` stay different | `rules/owner-rulings.md` "N random followers" = N distinct targets — 2026-08-23 |
| "A card on the field" includes amulets | `rules/owner-rulings.md` "A card on the field" includes amulets — 2026-09-09 |
| Resolution-depth guard on re-entrant `replicate` / Fanfare (Omegotep option 4) | generic step ceiling as a bug guard (same idea as the old engine); no rule cap — option 4 re-runs the Fanfare choose and terminates with probability 1 |
| A Faith crest is granted at match start to every player whose starting deck (or opening hand) contains a card carrying that Faith | rulebook Sham-Nacha / "active while … is in your deck"; no card effect `crest {gain}` of a `faith:` id. `faith:10634120` is the one Calge/Depths Faith (`grantedBy: ["10634120"]`). Sathanid / Yidmetra grant abilities onto that player's Faith (`10614120`, `10624120`) |
| Deck is a multiset; return-to-deck keeps modifiers | `rules/official-qa.md` Way of the Maid; measured: no pool card names top/bottom of deck |

## What I added beyond §4 (forced by a card)

| Construct | Card that forced it |
|---|---|
| `on: static` + `modifier.suppress` | `crest:10554110` Milteo & Luzen — printed names Fanfare and Enhance only, so those two tags, not full silence |
| `Selector` pool `kind: faith` | `10614120` Sathanid / `10624120` Yidmetra "give your faith …" |
| `choose.optionsFrom` | `10633310` Bewitching Enhance ("Activate all of them instead") |
| `Fuse.recipes.requires` | `90073110` α both β and γ |
| `removeAbilities.on` | `90051140` Rotting Zombie ("remove Last Words") |
| `summon.controller` | `crest:10564120` Kukishiro ("summon an enemy Fox…") |
| `Amount.neg` | `10714110` Thestae (−X defense) |
| `Amount.distinctNames` / `enteredThisMatch` | `10773310` Warp Slash / `10774110` Scarlet (pool, not an M0 file) |
| `Filter.costIn` / `baseCostIn` | `crest:10564120` Kukishiro; `10543110` Ruinbringer |
| `Filter.hasLastWords` | `crest:10954110` Istyndet |
| `Filter.destroyedThisMatch` | `10901310` Initiation of Rebirth |
| `Condition.varAtLeast` | `10833310` Amethyst's Naptime (pool) |
| `Condition.enterCountAtLeast` | `10931110` Obsessed Test Subject |
| `Condition.handSameCostAtLeast` | `10554120` Shakdoh |
| `damage.split` | `10434110` Wamdus (pool) |
| `Fuse.recipes.costTotal` / `costTotalGte` | `90072110` Striker Artifact |
| `leaderModifier.until` | `10444120` Zooey |
| `pay.resource: faith` | `10624120` Yidmetra |

## Owner questions — resolved from existing rulings (pending owner confirmation)

These were listed as open in M0 R1. Each is implemented from a ruling or the printed text; they are not re-asked.

1. **Faith duplicate (`faith:10634120` / `faith:90034330`)** — withdrawn. Identical official text; the Depths token reads the faith counter. One Faith file, `grantedBy: ["10634120"]`. A Faith is bootstrapped by the Sham-Nacha rule at match start, never by `crest {gain}`.
2. **Slaus after three activations (`10574110`)** — the provisional ruling in `rules/owner-rulings.md` ("Slaus … PROVISIONAL, 2026-08-13": nothing happens on a fourth) is what the engine implements.
3. **City of Babelon Engage with an empty hand (`10703210`)** — Engage is activatable with no legal target (rulebook "Targeting Rules": followers, amulets and Engage activate even with no valid targets; the selection fizzles). "Delay the count of this amulet by 1." is a separate printed sentence and resolves independently (owner ruling 2026-09-08 sentence structure). Authored as two clause roots.
4. **Omegotep re-entrant Fanfare (`10604110`)** — no rule cap. Option 4 re-runs the Fanfare's random choose, which picks option 4 again with probability ½ each round, so resolution terminates with probability 1. The engine carries only a generic resolution-step ceiling as a bug guard (as the old engine does).
5. **Initiation of Rebirth "destroyed this match" (`10901310`)** — a history of destruction events, not the cemetery's current contents. Reanimate copies without removing (rulebook "Reanimate"), so later reanimation or removal does not change eligibility. Cite the rulebook and the 2026-08-29 / 2026-09-02 rulings.
6. **Kukishiro's enemy Fox/Falcon (`10564120`)** — controlled by the opponent and counts for the opponent's Rally. The Rally ruling says any successful entry onto that player's field counts, route irrelevant.
7. **Gears' fuse partners (`90071210` / `90071220`)** — owner ruling "Artifact fuse chain (2026-09-05)" says gears fuse only with gears and notes the printed "Fuse: Artifact amulets" is stale text. Ruling wins; authored gears-only; printed literal kept as printed.
8. **Milteo & Luzen crest (`crest:10554110`)** — answered by its own text: Allied followers' Fanfare and Enhance abilities don't activate. Authored `suppress: ["fanfare","enhance"]` over allied field followers. Not full silence.
