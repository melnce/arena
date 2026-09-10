# Schema companion

`schema/cards.schema.json` is draft 2020-12. `additionalProperties: false` on every object. Discriminated unions are `oneOf` + a `const` on `kind` / `on` / `op` / `pick`. No `patternProperties`, no free-form maps.

A value with no card in the 572+43 pool is omitted. Each value below names at least one justifying id.

Regenerate the schema JSON with `python3 tools/gen_schema.py`.

## Official integer maps (from Cygames `common` / `specific_effect_card_info`)

Derived from the live card-list API (`cards/official/catalog.json` `_meta.maps`), then cross-checked against cards whose kind/class/tribe is known. Authored `kind` is still only `follower` / `spell` / `amulet` — official type 2 and type 3 both map to `amulet`. Extra official tribes (`luminous`, `levin`, `shikigami`) appear in the catalog; the closed authored-card enum stays the 12 pool tribes below unless an authored file needs one.

| official int | authored name | evidence |
|---|---|---|
| `type` 1 | `follower` | Knight `90021110` |
| `type` 2 | `amulet` (no Countdown type) | Awed and Inspired `10461210` (Engage amulet) |
| `type` 3 | `amulet` (Countdown family) | City of Babelon `10703210`, World of Games `10503210` |
| `type` 4 | `spell` | Deepwood Bounty `90011310` |
| `class` 0 | `neutral` | Indomitable Fighter `10001110`, Sandalphon `10404110` |
| `class` 1 | `forestcraft` | Deepwood Bounty `90011310` |
| `class` 2 | `swordcraft` | Knight `90021110` |
| `class` 3 | `runecraft` | Dazzling Runeknight `10031110` |
| `class` 4 | `dragoncraft` | Zooey `10444120` |
| `class` 5 | `abysscraft` | Istyndet `10954110` |
| `class` 6 | `havencraft` | Kukishiro `10564120` |
| `class` 7 | `portalcraft` | Slaus `10574110` |
| `rarity` 1 | `bronze` | Knight `90021110` |
| `rarity` 2 | `silver` | Venerating Dyer `10662110` |
| `rarity` 3 | `gold` | City of Babelon `10703210` |
| `rarity` 4 | `legendary` | Slaus `10574110` |
| `tribe` 0 | none (omit from `tribes[]`) | Deepwood Bounty `90011310` (`tribes: [0]`) |
| `tribe` 2 | `officer` | Knight `90021110` |
| `tribe` 3 | `luminous` | catalog `data.tribe_names` (no M0 example) |
| `tribe` 4 | `levin` | catalog `data.tribe_names` (no M0 example) |
| `tribe` 5 | `pixie` | Fairy `90011110` |
| `tribe` 6 | `departed` | Rotting Zombie `90051140` |
| `tribe` 8 | `earth sigil` | Magic Sediment `90031210` |
| `tribe` 11 | `mysteria` | Tico `10833110` (pool) |
| `tribe` 12 | `golem` | Emperor of Elements `10533110` |
| `tribe` 13 | `shikigami` | catalog `data.tribe_names` (no M0 example) |
| `tribe` 14 | `artifact` | Gear of Ambition `90071210` |
| `tribe` 15 | `puppetry` | Puppet `90071110` |
| `tribe` 17 | `marine` | Stormy Shamisen Shredder `10541120` |
| `tribe` 18 | `loot` | Gilded Blade `90021310` |
| `tribe` 19 | `encroacher` | Sathanid `10614120` |
| `tribe` 20 | `anathema` | Gildaria `10724110` |
| SE type 1 | `crest` | Slaus `10574110` / `10412312` |
| SE type 2 | `crystallize` | Venerating Dyer `10662110` cost 1 |
| SE type 3 | `accelerate` | Shoddy Plaything `10671110` cost 2 |
| SE type 4 | `faith` | Sathanid `10614120` |

`evo` carries images and `skill_text` only — no ATK/DEF. `+2/+2` on evolve is a rule. This fetch: `evo.skill_text === common.skill_text` for every pool card with an object `evo` (`_meta.evo_skill_text_mismatches` is `[]`).

Accelerate and Crystallize lines are **not** in `common.skill_text`. They live on `specific_effects`. Card `text` is stripped `skill_text` only; mode `printed` may be a whitespace-normalised substring of the reconstructed `Accelerate (N): {se.text}` / `Crystallize (N): {se.text}` line. Enhance stays in `skill_text`.

## Markup stripping (card `skill_text` and crest / Faith `skill_text`)

Official `common.skill_text` and `specific_effects[].skill_text` use the same HTML-ish markup. Card `text` and crest `text` are that string after, in order:

1. Replace `<hr>` / `<hr/>` with a newline.
2. Remove `<b>`, `</b>`, `<i>`, `</i>`, `<color=…>`, `</color>`, `<ridx=…>`, `</ridx>` (attribute values included).
3. Remove any remaining `<…>` tag.
4. Do not otherwise rewrite whitespace or entities.

Measured: Belial `10454120` and Corruption `10453310` **do** have `specific_effects` named `Crest` (the brief said they did not). Their crest text is taken from `skill_text` like every other crest, not from the v1 `crest` op `description`. Flagged in the PR; the files are not in the M0 example set.

Faith ids: `faith:<card id>`. Crest ids: `crest:<granting card id>`.

## Top-level

`oneOf` **Card** | **Crest**.

### Card

| field | why |
|---|---|
| `id` | Cygames id, 8 digits. Tokens start with `9` (`90051140`). |
| `name` | official name |
| `kind` | `follower` `10574110` · `spell` `10021310` · `amulet` `10703210` |
| `class` | `neutral` `10404110` · `forestcraft` `10714110` · `swordcraft` `10724110` · `runecraft` `10434120` · `dragoncraft` `10944110` · `abysscraft` `10954110` · `havencraft` `10564120` · `portalcraft` `10574110` |
| `set` | integer `card_set_id`. Tokens `90000` (`90051140`) |
| `rarity` | `bronze` `10001110` · `silver` `10002110` · `gold` `10933110` · `legendary` `10574110` |
| `token` | `true` `90051140` · `false` `10574110`. `10631110` Crystalspawn is collectible, not a token |
| `cost` | integer |
| `attack` / `defense` | followers only |
| `countdown` | amulets that print Countdown (`10072210` 2, `10703210` 1) |
| `tribes` | closed, lowercase (below) |
| `text` | printed, verbatim from official stripped `skill_text` (`cards/official/catalog.json`) |
| `traits` | 4.3 |
| `abilities` | 4.4 |
| `modes` | 4.5 |
| `fuse` | cards that print `Fuse:` (`10934110`, `90071210`) |
| `vars` | only `X starts at N` (`10131320` X=2) |

Evolved stats are not authored. Measured: official records have no per-card evo ATK/DEF override field in the pool.

### Crest

`id`, `name`, `grantedBy`, `faith`, `text`, optional `countdown`, `abilities` (same Ability schema). Ids inside files stay `faith:10634120`, `crest:10574110`. Filenames use a hyphen (`cards/crests/crest-10574110.json`, `cards/crests/faith-10634120.json`) so the tree checks out on NTFS.

## Tribes

`anathema` `10724110` · `artifact` `90071210` · `departed` `90051140` · `earth sigil` `90031210` · `encroacher` `10604110` · `golem` (Emperor of Elements `10533110`) · `loot` `90021310` · `marine` `10541120` · `mysteria` `10833110` · `officer` `90021120` · `pixie` `90011110` · `puppetry` `90071110`

## Traits

Booleans / small ints only. No `effects`.

| trait | card |
|---|---|
| `ward` | `10564110` Sofina |
| `bane` | `10662110` Venerating Dyer |
| `rush` | `10564120` Kukishiro |
| `storm` | `90061110` Holy Falcon |
| `ambush` | `10574110` Slaus |
| `aura` | `10654110` Armes |
| `intimidate` | `10904110` Zerael |
| `drain` | `10933110` (granted) |
| `barrier` | `10704110` (granted) |
| `ignoresWard` | `10944110` Antemaria |
| `attacksPerTurn` | `10654110` Super-Evolve "Can attack 3 times per turn" |
| `cantAttackFollowers` / `cantAttackLeader` | `10704110` quoted grant |
| `cantBeDestroyedByAbilities` | `10654110` |
| `cantBePlayed` | `90071210` Gear of Ambition |
| `damageCap` | `10464120` Vira (3) |
Blanket `cantAttack` is omitted (no pool card). Use `cantAttackFollowers` + `cantAttackLeader`.

## Abilities

`{ on, printed, effects, zone?, oncePerTurn?, when?, replaces? }` except `on: static`, which has `modifier` and no `effects`. `effects` is `minItems: 1`. `sequence.steps[]` items require `printed`. `ep` requires `amount`.

| `on` | card |
|---|---|
| `fanfare` | `10434120` (spells/amulets use this for on-play; there is no separate `play` trigger) |
| `lastWords` | `90051140` |
| `evolve` | `10714110` (EP-spent Evolve: line) |
| `superEvolve` | `10954110` |
| `anyEvolve` | `10724110` "When this follower evolves" |
| `anySuperEvolve` | `10554110` "When this follower super-evolves" |
| `strike` | `10851120` Lilith, Devilish Cutie (pool) |
| `followerStrike` | `10704110` |
| `leave` | `90051130` Ghost ("When this card leaves the field, banish it") |
| `clash` | `10654110` |
| `engage` | `{cost, sacrifice}` — `10963210` sacrifice true; `10703210` sacrifice false (Delay, no destroy) |
| `enter` | `10931110` "When this follower enters the field" |
| `discarded` | `90044330` |
| `invoked` | `10404110` |
| `fused` | `10934110` |
| `spellboost` | `10534120` / `10032120` |
| `startOfTurn` `{whose}` | `10574110` `own` |
| `endOfTurn` `{whose}` | `10574110` `own`; `90071110` Puppet `opponent` |
| `when` `{event, filter?}` | see Events |
| `static` | `{on, printed, modifier}` — no `effects`. `crest:10554110` `modifier.suppress: ["fanfare","enhance"]` over allied field followers |

`whose`: `own` `10574110` · `opponent` `90071110`. Turn-boundary triggers are owner-scoped (ruling 2026-08-31); `any` is omitted.

`zone`: `field` default · `hand` `10634120` / `10534120` · `deck` `10404110` / `10904110`

`oncePerTurn`: `10812110`, `10822110`, `crest:10934110`

`when` on the ability: `evolved` `10574110`; `turnOwner` `10724110`; `wasFused` `10933110`; `evolvedCountAtLeast` `10404110`; `playedBaseCostsThisMatch` `10904110`

`replaces: "evolve"`: `10002110` Arriet ("instead")

### Events (`on: when`)

| event | card |
|---|---|
| `ally_follower_enter` | `10724110`, `crest:10724110`, `10754120` |
| `enemy_follower_enter` | `10911210` Trap in the Woods (pool; supporting if referenced) |
| `ally_follower_destroyed` | Lifestealer `10553110` "Whenever a Skeleton is destroyed" + card-id filter |
| `ally_amulet_destroyed` | `10664120` Lyanthoth Faith; `10964120` Omerio |
| `ally_card_played` | `10914120` Hien; `crest:10554110` "play a follower" + filter |
| `ally_spell_played` | `10822110`, `90021210` |
| `ally_follower_attacks` | `10544120` Yube |
| `enemy_follower_attacks` | `10474110` Lu Woh |
| `ally_evolve` | `10613110` Merciful Attendant |
| `ally_super_evolve` | `10721110` Bombastic Bombardier |
| `ally_draw` | `10561120`, `crest:10564120` |
| `ally_earth_rite` | `10731310` Heel, My Dearie |
| `ally_engage` | `10062120` Sacred Griffon |
| `leader_restored` | `10563110` Saint of Rehabilitation; `10963110` Executor of the Vow |
| `self_buffed_up` | `10812110` Ruflet |

Omitted (no pool card): `leaderStrike`, `self_damaged`, `enemy_follower_defense_down`, `ally_follower_leaves_field`, `enemy_follower_destroyed`. Ghost's leave is `on: leave`, not an event. The schema lists exactly the 15 events in the table.

## Effects

Clause roots (ability/mode `effects[]`) require `printed`. Nested effects do not.

Common optional fields on every effect: `printed`, `as` (bind the result set), `when` (leaf-level condition).

### Leaf ops

| op | card |
|---|---|
| `damage` | `10904110`; `split: true` `10434110` Wamdus |
| `restore` | `10002110` |
| `buff` | `10001110`; negative via `Amount.neg` `10714110`; `untilEndOfTurn` `10574110` option 1 (cost, not buff) — buff until EOT: `10474110` Lu Woh |
| `setStats` | omitted — no pool card sets a follower's ATK/DEF to a number (Zooey sets leader max defense via `leaderModifier`) |
| `destroy` | `10963210` |
| `banish` | `10574110`, `10443310` |
| `returnToHand` | `10404110` invoked |
| `returnToDeck` | `position` only `random` — `10021310`, `10564120`. `top`/`bottom` omitted (no pool card) |
| `summon` | `10724110`; `controller: opponent` `crest:10564120` |
| `reanimate` | `10954110`, `10554110` |
| `addToHand` | `10434120` Ars Magna |
| `draw` | `10564120`; `filter` `10021310`; `player: opponent` `10832110` Sammy & Marie |
| `discard` | `10703210` |
| `search` | omitted — measured 0 printed "search" in the pool. `10021310` is `draw` + filter |
| `addToDeck` | `10901310` |
| `evolve` | effect-granted `10724110` Fanfare; `super: true` `10464120` |
| `grantTraits` / `removeTraits` | `10724110` Rush; `10624110` Bane; `until` `10442310` Maximum Love Bomb / `10843110` Giada |
| `grantAbility` | `10704110` quoted end-of-turn banish |
| `removeAbilities` | `90051140` (optional `on: ["lastWords"]` removes only Last Words) |
| `cost` | `delta` `10534120`; `set` `10923110`; `untilEndOfTurn` `10574110` |
| `pp` | `gainMax` `10444120`; `recover` `10604110`; `spend` via `pay` |
| `ep` | `gain` `10854110` "Recover 1 evolution point" |
| `crest` | `gain` + `player` self `10434120` / opponent `10574110` |
| `removeCrests` | `10453310` Corruption Super Skybound (pool) |
| `countdown` | `delta` −1 advance `90021210`; +1 delay `10703210` |
| `counter` | `earth` `10434120`; `combo` `10714110`; `skyboundHand` `10471120`; `shadows` via `pay`; `faith` `faith:10634120`; `{var: X}` `10131320` |
| `pay` | `shadows` `10754120`; `earth` `10031110`; `pp` `10934110`; `faith` `10624120` Yidmetra |
| `transform` | `10534120`, `10573310` |
| `leaderModifier` | Forced `maxDefense: {set: N}` Zooey `10444120` (existing file; was a bare int) or `maxDefense: {delta: N}` Lhynkal `crest:10534110` — same split as `cost`. After either, current defense clamps to the new max. `delta` floors the max at 0; a leader whose max (and therefore defense) is 0 is destroyed. `damageCap` + `until` `10444120` |
| `replicate` | `10604110`, `10923110` |
| `invoke` | `10404110`, `10904110` |
| `spellboostHand` | `10031110`; optional `select` `10931120` Key Spirit ("spellboost it 4 times") |
| `randomSplit` | `90034330` (ruling: one independent draw per faith point) |

### Combinators

| op | card |
|---|---|
| `seq` | `10633310` "summon … and give it"; `as` + later `bound` |
| `if` | `10724110` rally; `10041310` overflow else; `cond` / `then` / `else` |
| `choose` | `pick: 1` `by: player` `10564110`; `pick: 2` `by: random` `10604110`; `pick: all` + `optionsFrom: fanfare` `10633310` Enhance (`options` XOR `optionsFrom`); `by: randomUnused` `10574110` |
| `forEach` | omitted — measured 0 printed "for each" in the pool |
| `repeat` | `10954110`, `10543310`, `10554120` |
| `sequence` | `10703210` City of Babelon (official Q&A wrap-after-last); each `steps[]` item requires `printed` |

## Card source

`{named: id}` `10724110` · `{copyOf, exact}` `10443310` / `10901310` · Grandeur `10533310` `exact: true` clones the rolled deck instance (modifiers included) · `{randomFrom: Filter}` `crest:10564120`

## Amount

integer · `{count: Selector}` `10554120` · `{counter}` `90034330` faith · `{stat: {of, which}}` `10714110` · `{var}` `10131320` · `{add}/{sub}/{max}/{min}` `10811110` Marlone (sub) · `{neg}` `10714110` · `{distinctNames}` `10773310` · `{enteredThisMatch}` `10773310`. `{turn}` omitted (no pool card).

## Selector

`oneOf` two closed shapes.

**Reference picks** — `pick ∈ {self, bound, entering, attacker, defender, opposing, selected}`. No other fields. `bound` requires `ref`. `self` `10001110` · `bound` `10633310` · `entering` `10724110` · `opposing` `10654110` · `selected` `10473110` Cassius.

**Pool picks** — `pick ∈ {all, choose, random, randomDistinct, leftmost, highest, lowest}` with **required** `side`, `zone`, `kind`. Optional `filter`, `count`, `other`, `includeLeader`, `orderBy`.

`side` ally/enemy/any · `zone` field/hand/deck/cemetery/leader/crests · `kind` follower/amulet/card/leader/character/faith (`card` on field includes amulets — ruling 2026-09-09, `10573310`; `faith` is the player's Faith crest — `10614120` / `10624120`) · `filter` · `count` · `other` `10724110` · `includeLeader` `90021310` · `orderBy` `10901310`.

`all` `10963210` · `choose` `10021310` · `random` `10543310` · `randomDistinct` `10554110` / `10564120` · `highest` `10901310` · `lowest` `10552310` Tyrannical Fists · `leftmost` `10502110`. A leader is a pool pick (`pick: all`, `zone: leader`, `kind: leader`), not `pick: self` with extra fields. Omitted: `rightmost`, `lastSummoned`.

## Filter

`all` / `any` / `not` · `tribe` `10754120` · `card` / `cards` / `notCard` (Cygames ids, never names) `10933110` · `kind` · `class` `10021310` · `costEq`/`Lte`/`Gte`/`In` `crest:10564120` · `baseCost*` `10901310` / `10674110` · `attack*`/`defense*` · `evolved`/`unevolved` `10564110` · `damaged` · `hasTrait` enum of trait keys `10564110` Ward · `enhanced` `10622310` Majestic Conquest · `sameCostGroup` `10503210` World of Games · `hasLastWords` `crest:10954110` · `hasSpellboost` `10931120` Key Spirit ("a card in your hand with On Spellboost") · `destroyedThisMatch` `10901310`

## Condition

`all`/`any`/`not` · `countAtLeast` · `counterAtLeast` · `evolved` `10574110` · `superEvolutionUnlocked` `10401120` Vyrn · `combo` `10012110` · `rally` `10724110` · `overflow` `10041310` · `maxPpAtLeast` `10042310` · `skyboundArt` `10434120` · `wasFused` `10933110` / `"both"` `90073110` · `did` `10653110` "If you selected one" · `attackedLeaderLastTurn` `10944110` · `attackingFollower` `10843110` Giada (one Strike; second sentence) · `turnOwner` `10724110` · `evolvedCountAtLeast` `10404110` · `playedBaseCostsThisMatch` `10904110` · `handHas` · `fieldHas` `crest:10954110` · `leaderDefenseLte` `10841110` Gido · `varAtLeast` `10833310` (`key` ∈ {X,Y,Z}) · `enterCountAtLeast` `{card, n}` `10931110` · `handSameCostAtLeast` `10554120`. Omitted: `survived` (ruling exists, no printed card), `ppAtLeast`, `isEvolvedFollowerEntering`.

## Modes

Three closed shapes. `enhance` `{kind, cost, printed, effects, replacesBase?}` `10001110`; multi-tier `10624110` (both tiers + Fanfare — ruling 2026-08-15). `replacesBase` `10633310`. `accelerate` `{kind, cost, printed, effects}` `10671110` (summoned body is base cost 6 — ruling; official `skill_text` is `Fanfare: Draw 3 cards.\n\nWard` — the Accelerate line is the SE). `crystallize` `{kind, cost, printed, countdown?, traits?, abilities}` — no mode-level `effects`; the form's text is amulet abilities (`10662110`; official follower `text` is `Rush\nBane`).

## Fuse

Required `printed` (the literal `Fuse: …` line) plus `partners` Filter. "Cards" → empty filter `10934110`. Artifact cards → `{tribe: artifact}` `90072110` / `90072120`. Artifact amulets → `{tribe: artifact, kind: amulet}` `90071210` / `90071220` (owner ruling 2026-09-10; printed text was right).

What a fuse does to the **host** is `recipes` data (`partners`, cost conditions, `requires: [ids]`, `result`). The action enumerator reads recipes without running effects. `on: fused` exists only for effects *beyond* the host transform (Sephie `10934110` summon). Ability `effects` is `minItems: 1` — no empty fused stub.

`recipes`: `{costTotal}` / `{costTotalGte}` + `{transformInto}` `90072110`; `{requires: ["90073120","90073130"]}` → Ω on α (`90073110`). Across-turns memory of which partners were fused is a rule (official Q&A `90073110`).

## Sentence structure

Independent sentences: `10021310` (return finds nothing → still draws). Dependent: `seq` + `did` / `as` + `{count: bound}`. `10554120` "Then, if …" is `handSameCostAtLeast` inside the `repeat` body, not `did`.

## What I could not express and why

Coverage is `needs: 0`. Owner questions that looked open are recorded as derived from existing rulings in [`design.md`](design.md).

`returnToDeck.position: top|bottom` from §4.6 is omitted (measured: no pool card names top or bottom of a deck). `{var}` / `randomSplit.keys` are the closed enum `X` `Y` `Z`. `crest.gain` is `crest:` only — a Faith is never gained by an effect.
