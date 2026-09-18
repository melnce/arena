# Unprinted nesting audit — singleton "in your hand" phrases

Scope: phrase-index rows with **count 1** whose phrase contains `in your hand`.
Question (owner ruling 2026-09-08): does the JSON nest an effect inside a
condition where the printed text has that effect as an **independent sentence**?

> "because draw a card is a different sentence. if it said something like
> 'discard a card and if you did so draw a card' then it wouldnt draw"

| printed shape | correct encoding |
|---|---|
| "Select X and do A. **Do B.**" — B its own sentence | B is a **sibling**, never nested under a condition about A |
| "Select X. **If you selected one**, do B." / "…and if you did so…" / "Then, if…" | B **nested** — `did`, `then:`, or a nested `if` |

Verdicts: `ok` · `unprinted nesting` · `missing gate` · `unsure — owner question`.

Base: `cf2ac8a` (PR #50 already unwrapped Aristocrat `10521110`).

## List derivation

Re-derived from `docs/phrase-index.md` (not from the brief):

```
python3 -c '… rows with count==1 and "in your hand" in phrase …'
```

**63 rows, 36 card ids** — same cards as the brief. Two extra attachments are
crests (`crest:10553310`, `crest:10574110`); they resolve to cards already
in the 36 (`10574110`) or to a crest file outside the card-id list
(`crest:10553310` is granted by `10553310`, which is not in the 36). The
36 collectible ids match the brief exactly.

```
10412110 10471120 10473110 10502110 10502120 10521110 10521120 10521310 10522120 10531310
10544120 10554120 10572110 10572310 10574110 10574120 10602210 10642310 10643110 10644110
10703110 10703210 10711120 10711310 10741310 10761120 10772310 10833110 10842120 10844120
10853310 10854120 10903110 10931120 10942310 10972310
```

Printed text in the table is the catalog `text` field from
`cards/official/catalog.json` (already markup-stripped). Newlines collapsed
to ` / ` for the table only.

## Table

| id | name | printed text (verbatim from `cards/official/catalog.json`) | JSON nesting | verdict |
|---|---|---|---|---|
| 10412110 | Chloe, What a Gal | Enhance (8): Select a follower in your hand and summon it. Return this card to hand. | Enhance: `summon` (choose hand follower) **sibling** of `returnToHand`. No `if`/`did`. | ok |
| 10471120 | Tsubasa, Blazing Gearcyclist | Fanfare: Increase the Skybound Art gauges of all cards in your hand by 1. / Rush | Fanfare is a single `counter` op. No selection, no nest. | ok |
| 10473110 | Cassius, Sky-Yearning Arrival | Fanfare: Select an Artifact follower in your hand and deal X damage to all enemy followers. X is the selected follower's attack. / Last Words: Add a Fortifier Artifact to your hand. | One printed sentence: `seq[select as=a, damage]`. Damage is not an independent sentence. Empty bind → X=0 (schema / official Q&A). Last Words is a separate ability. | ok |
| 10502110 | Goddess of Starlight | Evolve: Select 3 cards in your hand and discard them. Add an exact copy each of the 3 leftmost cards in your hand to your hand without revealing them. | `discard` (choose, count 3) **sibling** of `addToHand` (leftmost 3, exact). No `if`/`did`. | ok |
| 10502120 | Behemoth General | Evolve: If the sum of the 3 highest base costs in your hand is higher than that of your opponent's, destroy all enemy followers. | Printed `If` → `if amountAtLeast` then `destroy`. The destroy is the gated sentence. | ok |
| 10521110 | Altruistic Aristocrat | Fanfare: Select a card in your hand and discard it. Restore 3 defense to your leader. If you selected a spell, restore 6 defense instead. | `seq[discard as=d, if countAtLeast(bound d is spell) then restore 6 else restore 3]`. Restore is **not** under `did`. The `if` is the printed "If you selected a spell … instead". Empty bind takes the else (restore 3) — PR #50. | ok |
| 10521120 | Smoke-Shrouded Beauty | Fanfare: If you have at least 2 spells in your hand, give this follower +1/+1 and Ward. / Evolve: Add a Glittering Gold to your hand. | Printed `If` → `if handHas` then buff+Ward. Evolve is a separate ability. | ok |
| 10521310 | Extravagance of the Goldbloom | Select a spell in your hand and discard it. Do this 2 times: "Deal 3 damage to a random enemy follower." | `discard` (choose spell) **sibling** of `repeat×2[damage]`. No `if`/`did`. Playability with no spell in hand is the 2026-09-18 ruling (unplayable); not a nesting question. | ok |
| 10522120 | Amphibian Goldmuncher | At the end of your turn, if you have at least 2 spells in your hand, deal 5 damage to all enemy followers. / Evolve: Add 2 copies of Glittering Gold to your hand. | Printed `if` → `if handHas` then damage. | ok |
| 10531310 | Metamorphosis of the Dawnblossom | Select a card in your hand and discard it. Draw 2 cards. | `discard` **sibling** of `draw`. No `if`/`did`. | ok |
| 10544120 | Yube, Crestpetal | Fanfare: Summon a Majestic Megalorca. / Evolve: Select a card in your hand and discard it. Gain Crest: Yube, Crestpetal. | Evolve: `discard` **sibling** of `crest`. No `if`/`did`. | ok |
| 10554120 | Shakdoh, Nightblossom | Fanfare: Do this 2 times: "Return your hand to deck. Draw X cards. X is the number of cards you returned. Then, if you have at least 4 cards with the same cost in your hand, deal 4 damage to all enemies." / Super-Evolve: Replicate the effects of this card's Fanfare ability. | Printed `Then, if` → `if handSameCostAtLeast` inside the `repeat` body (schema: this is the reference, not `did`). | ok |
| 10572110 | New-Age Cartographer | Fanfare: Add an Ominous Artifact β to your hand. / Super-Evolve: Select an Artifact follower in your hand that costs 5 or less and summon an exact copy of it. | Super-Evolve is one sentence: `summon` with choose-from-hand. No later independent sentence. | ok |
| 10572310 | Resurrection Tuner | Select a card in your hand and discard it. Add a copy each of 2 random differently named allied followers destroyed this match to your hand without revealing them. | `discard` **sibling** of `addToHand`. No `if`/`did`. | ok |
| 10574110 | Slaus, Revolving Wheel of Fortune | Ambush / At the end of your turn, if this follower is evolved, give your opponent Crest: Slaus, Revolving Wheel of Fortune and banish this card. / At the start of your turn, activate a random ability that hasn't been activated yet from the following. / 1. Reduce the cost of all cards in your hand by 1 until the end of the turn. / 2. Give all allied followers on the field +2/+2. / 3. Restore 3 defense to your leader. | End-of-turn printed `if this follower is evolved` is ability `when: {evolved: true}`, not a `did` wrap of an independent sentence. Start-of-turn option 1 is a bare `cost` (no nest). The phrase that put this card in scope is option 1, not a Select+later-sentence. | ok |
| 10574120 | Imari, Dewdrop | Fanfare: Select a card in your hand and discard it. Draw a spell. / Whenever you play a spell, if this follower is evolved, summon an Imari's Little Buddies. / Super-Evolve: Draw 2 differently named 1-cost spells. | Fanfare: `discard` **sibling** of `draw`. The evolve-gate is ability `when: {evolved: true}` on a different ability. | ok |
| 10602210 | Encroached World | Engage: Select a card in your hand and transform it into an exact copy of a random card in your opponent's deck. | One sentence: `transform` with choose-from-hand. | ok |
| 10642310 | Spilling Red | Select a card in your hand and discard it. Select an enemy follower on the field and destroy it. | `discard` **sibling** of `destroy`. Two independent selections, no `did`. Playability is official Q&A (both selections required). | ok |
| 10643110 | Impeding Pugilist | Fanfare: Select a card in your hand and discard it. Deal 6 damage to a random enemy follower. / Ward / Barrier / Evolve: Replicate the effects of this card's Fanfare ability. | `discard` **sibling** of `damage`. No `if`/`did`. | ok |
| 10644110 | Sagatsumatsu, Fair Beheader | Fanfare: Select a card in your hand and discard it. Add 2 copies of Spilling Red to your hand. / Storm / Bane / Aura | `discard` **sibling** of `addToHand`. No `if`/`did`. | ok |
| 10703110 | Hedonistic Socialite | Fanfare: Select a card in your hand and discard it. Deal 2 damage to all enemy followers. | `discard` **sibling** of `damage`. No `if`/`did`. | ok |
| 10703210 | City of Babelon | Countdown (1) / At the end of your turn, activate an ability in sequence from the following. / 1. Deal 2 damage to a random enemy follower. / 2. Restore 2 defense to your leader. / 3. Deal 2 damage to the enemy leader. Destroy this card. / Engage (1): Select a card in your hand and discard it. Delay the count of this amulet by 1. | Engage: `discard` **sibling** of `countdown`. Two clause roots (design.md City of Babelon). No `if`/`did`. | ok |
| 10711120 | Elven Trapper | Fanfare: Select a card in your hand and return it to deck. Add 2 copies of Fairy to your hand. | `returnToDeck` **sibling** of `addToHand`. No `if`/`did`. | ok |
| 10711310 | Cognitive Shift | Select 2 cards in your hand and return them to deck. Draw 2 cards. | `returnToDeck` **sibling** of `draw`. No `if`/`did`. | ok |
| 10741310 | Apathetic Gaze | Gain 1 max play point. Transform all copies of Apathetic Gaze in your hand and deck into copies of Lazing Flame. | `pp` **sibling** of `seq[transform hand, transform deck]` (one printed transform sentence, two zones). No selection nest. | ok |
| 10761120 | Missionary of Recruitment | Fanfare: Draw 2 amulets. / Evolve: Deal X damage to all enemy followers. X is the number of amulets in your hand. | Evolve is a single `damage` with `amount: {count: hand amulets}`. No nest. | ok |
| 10772310 | Blink Step | Give all allied followers on the field +1/+0. If you've unlocked super-evolution, give all followers in your hand +1/+0. | Field buff **sibling** of printed `If you've unlocked` → `if superEvolutionUnlocked` then hand buff. | ok |
| 10833110 | Tico, Mysterian Spellcrafter | Fanfare: Add 2 copies of Mysterian Missile to your hand. / Evolve: Reduce the cost of all Mysteria spells in your hand by 1. / Super-Evolve: Gain Crest: Tico, Mysterian Spellcrafter. | Evolve is a bare `cost`. No nest. | ok |
| 10842120 | Kimika, Cook of Happiness | Fanfare: Select a card in your hand and discard it. Draw a card. Restore 1 defense to your leader. / Evolve: Replicate the effects of this card's Fanfare ability. | `discard` **sibling** of `draw` **sibling** of `restore`. The three sentences are independent (PR #50 empty-hand pin: draws and restores). | ok |
| 10844120 | Lumiore & Argente, Shining Wings | Fanfare: Select 2 cards in your hand and discard them. Deal 4 damage to all enemies. / Super-Evolve: Draw 3 cards. | `discard` **sibling** of `damage`. No `if`/`did`. | ok |
| 10853310 | Ebb and Flow | Select a card in your hand and return it to deck. Draw 2 cards. If you've unlocked super-evolution, reduce their costs by 1. | `returnToDeck` **sibling** of `draw as=d` **sibling** of printed `If you've unlocked` → `if superEvolutionUnlocked` then `cost` on bound d. | ok |
| 10854120 | Ceres, Liminal Rose | Fanfare: Necromancy (20) - Reduce the cost of all Abysscraft cards in your hand by 2. / Clash: Deal 4 damage to the opposing follower. / At the end of your turn, restore 4 defense to your leader. | Fanfare is `pay shadows` wrapping `cost`. Necromancy is a resource cost, not a selection-succeeded gate on a later sentence. | ok |
| 10903110 | Warden of Selflessness | Fanfare: Add a Jailor of Antiquity to your hand. Deal X damage to 2 random enemy followers. X is the number of Neutral cards in your hand. / Evolve: Recover 1 play point. | `addToHand` **sibling** of `damage`. Two independent sentences, no `if`/`did`. | ok |
| 10931120 | Key Spirit | Fanfare: Select an enemy follower on the field and deal it 7 damage. Deal 4 damage to the enemy leader. / Evolve: Select a card in your hand with On Spellboost and spellboost it 4 times. | Evolve is one sentence: `spellboostHand` with choose. Fanfare's leader damage is a sibling of the field damage (independent; not a hand-selection nest). | ok |
| 10942310 | Parting Jaws | Select 2 cards in your hand and discard them. Deal 3 damage to a random enemy follower and the enemy leader. | `discard` **sibling** of `seq[damage follower, damage leader]`. No `if`/`did`. | ok |
| 10972310 | Disgraceful Banishment | Select a card in your hand and discard it. Draw a card. If there are no duplicates in your deck, draw 3 instead. | `discard` **sibling** of printed `If there are no duplicates` → `if deckHasNoDuplicates` then draw 3 else draw 1. The instead-draw is not gated on the discard. | ok |

**Honest result: all 36 are `ok`.** PR #50 already removed the only `did` wrapper in this set (Aristocrat). Every later independent sentence is a sibling. Every `if` that remains matches a printed `If` / `Then, if` / `If you've` / `If there` / `If this` / `If you selected a spell` (type check, not `did`).

Cards that do not print Select-then-another-sentence were still opened and are marked `ok` because they have no place to hide the Aristocrat defect — they are not "needs a second look"; they are out of the defect's shape.

## Goddess pin (measured, not a nesting defect)

`goddess_evolve_fewer_than_three_matches_main` now drives the choice:

| n in hand | choice | discarded | second sentence ("3 leftmost … exact copy") |
|---|---|---|---|
| 0 | no node | 0 | runs against empty hand; adds 0 |
| 1 | one pick | 1 (cemetery + 1 shadow) | hand empty after the discard; adds 0 |
| 2 | two sequential picks | 2 | hand empty after both discards; adds 0 |

This is what `main` does. It matches the printed sentences in order: after
"select as many as you can and discard them", the hand has nothing left to
copy. Not changed. Not reported as wrong.

## outside scope — seen in passing

`cond.did` appears on exactly two authored cards, both outside the 36, both
printing "If you selected one":

- `10653110` Deprived Destroyer — `seq[destroy as=t, if did:t then evolve]`
- `10753110` Beastmaster Bones — Super-Evolve `seq[destroy as=t, if did:t then destroy random enemy]`

These are the legitimate nests the gate must not flag. No other `did` in
`cards/**`. Nothing else in passing looked like the Aristocrat defect; cards
outside the 36 were not audited.

## Gate

`python3 tools/phrase_index.py --check-unprinted-nesting` (also run from
`--check` and `--check-divergences`).

A card is flagged when an `op: if` with `cond.did` sits under a `printed`
that has no selection-**succeeded** marker. Markers derived from
`cards/official/catalog.json` (measured 2026-09-18):

| marker | catalog hits | authorizes `did`? |
|---|---|---|
| `If you selected one` | 4 | **yes** |
| `if you did so` | 0 (kept; 2026-09-08 ruling wording) | **yes** |
| sentence-start `If you do` | 0 (kept; same ruling) | **yes** |
| `If you selected a spell` | 1 (Aristocrat) | **no** — type check → `countAtLeast` / `boundHas` |
| `If you selected an allied amulet` | 3 | **no** — side/kind check → `boundHas` |
| `Then, if` / `If this` / `If you've` / `If you have` / `If there` | many | not `did`; they authorize other `cond` keys |

`If you selected` as a prefix is **not** used as a `did` marker. That is the
discrimination: pre-#50 Aristocrat prints "If you selected a spell" and still
wrapped restore 3 in `did`. A gate that treated any "If you selected" as
licence would miss the bug this brief generalises.
