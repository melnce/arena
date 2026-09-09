> **This file is canonical.** Owner rulings are recorded here, in the repo, where they are versioned, reviewable in a PR, readable by every agent, and gated by `npm run check:rulings-absorbed`. A ruling that exists only in a chat transcript, a session document, or a code comment is not recorded. Earlier project-side copies of this document are frozen archives and must not be treated as authoritative.

# Owner rulings — Shadowverse Worlds Beyond Practice Tool

Rulings Chris has given during development, with the reasoning and evidence behind each.

**Standing principle: owner rulings override printed card text.** When a card's printed text and a ruling here disagree, the ruling wins. This has already prevented one regression — a fidelity audit flagged Azurifrit's 3-per-turn cap as contradicting its text, and the cap is correct because it is a ruling.

---

## Rally — 2026-08-12

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Rally (2026-08-12):** -->

> "It doesn't matter how, but only if a follower got successfully summoned to the field."

The only trigger is a follower actually arriving on your field. Route is irrelevant: played from hand, summoned by an effect, token, reanimated, put into play by Last Words. Anything that does not put a follower on the field does not count, and a summon that **fails** (field already full) does not count either.

Consequences: a Crystallize play puts down an _amulet_ and an Accelerate play resolves as a _spell_, so neither counts by itself — but a follower summoned by that text, or by a Crystallize amulet's Last Words, does.

**Taking control** of an enemy follower does **not** count — a control change is not a summon. Verified empirically: **zero cards in the entire 735-card pool** take control of a follower, in any class, in either printed text or effect ops. The question is therefore unreachable in practice.

Note the separate, pre-existing timing rule still stands: a "Fanfare: if Rally (N)" check does not count the card's own entry, because the count is read before that card arrived.

## Accelerate — original cost preserved — 2026-08-12

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Accelerate / Crystallize original cost (2026-08-12):** -->

> "In the game there was a puzzle with Accelerate that said the units keep their original cost. Evolution Portalcraft with Shoddy Plaything and Yog-Zentha, Eld Axe abuses this specifically."

~~Playing a card via Accelerate does **not** replace its original/base cost with the Accelerate value. The base cost stays as printed.~~ **Superseded 2026-09-06** — see **Accelerate / Crystallize — the played card takes the alternate form's base cost (2026-09-06)** below. The observation here still stands: the summoned Shoddy Plaything is base cost **6**, and the Eld Axe package turns on off that body.

**This overrides a contradicting official source**, deliberately: a Cygames tweet from 18 Sep 2018 states 「アクセラレート能力でカードをプレイした際、元のコストはアクセラレートの値となります」 — "when played via Accelerate, the original cost becomes the Accelerate value." That tweet is about the **original Shadowverse**, whose service ended June 2026. The **2026-09-06 ruling** restores that tweet's reading for Worlds Beyond too (see official Q&A on Zerael, Sundered Rebirth `10904110`).

The abuse case is also the regression test: Shoddy Plaything is base cost 6 with "Accelerate 2: Summon a Shoddy Plaything", so 2 PP buys a **base-cost-6 body**, switching on Portalcraft's whole "base cost 5 or more" package — Yog-Zentha adds Depths of the Eld Axe, Advent of the Eld Axe draws, Unfeeling Eld Axe drops 1 in hand.

## Accelerate / Crystallize — the played card takes the alternate form's base cost (2026-09-06)

<!-- rulebook: absorbed #classmechanic-specific-keywords » the played card takes base cost **N** from that play on -->

> "yes if you play it in that form it stays that value. That was my mistake. The shoddy plaything accelerate spell has the EFFECT of summoning a shoddy plaything. So obviously it will summon a freshly printed one and go to grave as 2 cost spell."

Official per-card Q&A (Zerael, Sundered Rebirth `10904110`): _"If I play an accelerated Jailor of Antiquity, what will its base cost be?"_ → _"Its base cost will be 1."_

From the moment a card is played via **Accelerate (N)**, it is a **spell with base cost N** (and cost N). It records N — not the printed follower cost — on the Zerael / Azvaldt "played cards with base costs of 1…8" ladder and on every other "played a card with base cost X" check, and it goes to the graveyard as a cost-N spell. The **2026-09-02 permanence ruling** stands: it stays a spell corpse (not reanimatable).

A card played via **Crystallize (N)** is likewise an **amulet with base cost N** from that play on (and stays an amulet if bounced to hand — the hand copy keeps cost N).

Whatever the alternate form's _effect_ summons is a **freshly printed** card: Shoddy Plaything's _"Accelerate (2): Summon a Shoddy Plaything"_ puts a base-cost-**6** follower on the field. Normal plays and Enhance plays are unchanged: printed base cost.

## Hidden information — always visible — 2026-08-13

<!-- rulebook: absorbed #hidden-information » **Owner ruling — Hidden information always visible (2026-08-13):** -->

> "Should be always visible cause it's my practice tool and I'm playing against myself. The only time they should be hidden is if playing against a bot or scripted opponent, but that doesn't really have merit since the bot is going to be terrible."

Card text saying "without revealing them" hides nothing in this tool. Copies are visible, and effects may read both hands freely. Not tied to the hidden-hand toggle — judged not worth the complexity.

Settles four cards at once: Goddess of Starlight (`10502110`), Wolfraud (`10514110`), Legacy of the Brave (`10802310`), Behemoth General (`10502120`).

## Oluon, Raging Chariot (`10524110`) — 2026-08-13

<!-- rulebook: absorbed #classmechanic-specific-keywords » _Oluon, Raging Chariot (2026-08-13):_ when evolved -->

> "'Another' means only Oluon is exempt. You can hit an 8 HP unit twice or the leader 3 times. I've seen clips of Oluon just one-shotting the enemy leader (really cancer card)."

Each of the 3 hits picks independently from all characters except Oluon itself. Already-hit targets stay eligible, so all three can land on the leader for 21.

## Slaus, Revolving Wheel of Fortune (`10574110`) — 2026-08-13 — **PROVISIONAL**

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Slaus unused-ability pool (PROVISIONAL, 2026-08-13):** -->

> "Unsure. The crest is clear — it has a 3 turn CD so only 3 abilities max can happen. His card effect is hard to test in game because usually cards in Shadowverse don't survive 3 turns in a row. So I'd have said max 3 times and then it stops."

The unused-ability pool is not replenished: three activations maximum, and nothing happens on a fourth start-of-turn. **Marked provisional** — the owner flagged it as uncertain and it is hard to observe in a real match, since a card surviving four turns usually means the game is already decided.

## Thestae, Anathema of Distortion (`10714110`) — 2026-08-13

<!-- rulebook: absorbed #classmechanic-specific-keywords » _Thestae, Anathema of Distortion (2026-08-13):_ the Crest's -->

> "It ONLY buffs cards IN deck, and yes they keep the stats otherwise it would be pointless."

The crest buffs cards while in the deck, and the buff **persists after the card is drawn**. It does not touch cards already in hand or on the field.

## Copy semantics — general rule — 2026-08-13

<!-- rulebook: absorbed #copy-semantics-exact-copy-vs-copy » **Owner ruling — Exact copy vs copy (2026-08-13):** -->

> "EXACT copy means it has the same stats and modifications. Just copy means the base card."

Applies to every copy effect in the game, not just the card that raised it:

- **"an exact copy"** → the instance, including current stats and modifications.
- **"a copy"** → the base/printed card.

Also corrected a badly-formed question from a brief: _"Engage is its own thing. Transform is an effect that happens after engaging. Nothing to do with each other."_ There is no Engage-transform interaction to model — Engage activates, the transform is simply its effect, exactly as "Engage: give an ally +1/+1" would be.

## Multi-tier Enhance — ALL affordable tiers activate — 2026-08-15

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — multi-tier Enhance (2026-08-15):** -->

> "If you play Noel for 8 play points, both his Enhance 8 and Enhance 7 abilities will activate." — "And his fanfare also activates no matter what — if played from hand ofc."

When a card with multiple Enhance tiers is played **from hand** at a PP amount covering several tiers, **every tier whose cost is met activates** — they stack, not compete — **and the Fanfare activates as well** (Enhance never suppresses Fanfare unless a card explicitly says so). Noel IV, Ruthless Warlord (`10624110`) played from hand at 8 PP therefore summons THREE soldiers: the Fanfare Bane soldier, the Enhance(7) Drain soldier, AND the Enhance(8) Storm soldier. The from-hand qualifier is the standard rule: summoned-by-effect copies trigger neither Fanfare nor Enhance.

Engine status at ruling time: the engine applied only the highest affordable tier — **wrong**, fix dispatched. The test added in PR #47 asserting "8 PP → Storm only, no Drain" enshrines the wrong behavior and must be rewritten by the fix. Blast radius: Noel IV is the only multi-tier Enhance card in the current 735-card pool (multiset linter scan), but the fix must be general.

## "Select" is forced — 2026-08-16

<!-- rulebook: absorbed #targeting-rules » **Owner ruling — "Select" is forced (2026-08-16):** -->

> "In this game if it says 'select' then it's forced. You can't play him [Field Scientist] without discarding unless your hand is empty."

A "Select …" clause is mandatory: while a legal target exists, the selection cannot be declined or cancelled — the only exception is when no legal target exists (empty hand / empty board), in which case the card still plays and the selection clause fizzles.

Engine status at ruling time: already correct de facto — there is no cancel path for pending selections anywhere in engine or UI, and the `optional: true` key on the four hand-selection cards (Ruby `10101110`, Apprentice Astrologer `10131120`, Arcane Archivist `10231120`, Field Scientist `10372120`) implements only the empty-zone fizzle, not a decline. Note for future readers: in this codebase `optional` means "fizzle gracefully on empty zone", NOT "player may decline". Tests pinning the ruling were dispatched (mandatory selection + empty-hand fizzle + the conditional-clause distinction: "If you selected one, …" effects must NOT run when nothing was selectable).

## Playability with no "Select" target — spells blocked, followers/amulets fizzle — 2026-08-16

<!-- rulebook: absorbed #targeting-rules » **Owner ruling — Playability with no "Select" target (2026-08-16):** -->

> "If it's something like select on the field but there is no target on the field, then you can still play amulets and followers without the effect. Spells tho — if it says select on the field you can't play the spell if there is no target. Radiant Rainbow sometimes is such a dead card: when you don't have an On-Spellboost card in hand you just can't play it at all, compared to Hearthstone or something where you could just spend 2 mana on nothing to unclog hand."

The rule: a **spell** with a mandatory "Select …" clause requires at least one legal target to be playable at all — no target, no play, the card sits dead in hand (no Hearthstone-style burning mana on nothing). **Followers and amulets** with the same kind of clause stay playable; the selection clause simply fizzles (see the "Select is forced" ruling above for the fizzle mechanics).

Engine status at ruling time: already correct — `canPlayCard` (src/logic/core/playCard/preflight.ts) runs target-requirement checks only when `card.type === "Spell"` or the play is an Accelerate (which resolves as a spell), and Radiant Rainbow (`10131310`) has a bespoke preflight requiring a Spellboost card in hand (excluding itself), with a blocked-reason string. Verify-first tests dispatched to pin the boundary (spell blocked / follower fizzles / Radiant Rainbow dead-card case / Accelerate-as-spell gating).

## Azurifrit, Heir to Disdain (`10344110`) — NO per-turn cap — 2026-08-23

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Superseded (owner ruling 2026-08-23).** The trigger fires on **every** qualifying self-damage -->

> Q: "In the real game, can Azurifrit ping the leader more than 3 times in one turn?" — A: **"No cap — fires every time."**

The trigger _"During your turn, whenever this follower takes damage but isn't destroyed, deal 1 damage to the enemy leader"_ fires on **every** qualifying self-damage event during your turn. Playing him (Fanfare 3× AoE, hitting himself 3×) and super-evolving him on the same turn (3 more AoE) therefore produces **6** leader pings, not 3.

**This CORRECTS an earlier assumed ruling.** The card data carried `max_per_turn: 3` on the trigger, and a fidelity audit that flagged it as contradicting the printed text was told the cap was an owner ruling — but no such ruling was ever recorded in Chris's words. It appears the "Do this 3 times" of the Fanfare was mistakenly modelled as a per-turn cap on the trigger. The cap is removed. **Lesson: a "ruling" that exists only as a second-hand note in a roadmap, with no owner quote, is an assumption — verify it before defending it.**

Symptom that exposed it (Chris, live play): super-evolving Azurifrit on the turn he was played produced no leader damage, but super-evolving him a turn later did — the cap was silently consumed by the Fanfare's own self-hits.

Also fixed alongside: _"Super-Evolve: Fully restore the defense of this follower"_ must restore to the **buffed** maximum, so a 5/7 Azurifrit ends super-evolution at **8/10**, not 8/7.

## Damage prevented by super-evolve still counts as "taking damage" — 2026-08-23

<!-- rulebook: absorbed #damage-events-general » **Owner ruling — Damage prevented by super-evolve still counts as "taking damage" (2026-08-23):** -->

> "even if superevolved the followers 'take damage' even if it is reduced to 0."

A super-evolved follower's own-turn protection reduces incoming damage to 0 but does **not** cancel the damage event. Every "when this follower takes damage" trigger — Galmieux's 3-damage passive, her crest's Fangs of Ardent Destruction, Azurifrit's leader ping — must fire on such a hit.

Applies to combat as well as effect damage. Chris hit the combat case live: a super-evolved Galmieux attacking a follower took the counter-damage, was protected to 0, and fired **nothing** — no passive, no Fangs. Root cause found in review: `src/logic/core/combat.ts` skipped the counter-damage call entirely when the attacker was invincible-on-attack, so `dealDamage` (which already treats `preventedBySuper` as "counts as damaged") was never reached.

## Deaths settle before dependent triggers pick targets — 2026-08-23

<!-- rulebook: absorbed #timing-windows-and-trigger-resolution » **Owner ruling — Deaths settle before dependent triggers pick targets (2026-08-23):** -->

> "playing fangs of ardent destruction should kill the 2/1 at the same time as damaging galmieux. galmieux should then deal 3 dmg to the 5/2 ... but currently the 5/1 ward keeps standing. so either galmieux 3 damage didnt trigger or it targeted the corpse of the 2/1."

A follower reduced to 0 or less defense is dead and must never be a legal target for any effect that resolves afterwards — including effects triggered inside the same damage batch (an AoE that kills one follower and damages another). The general rule: settle deaths before dependent triggers choose targets.

## "N random followers" = N distinct targets; "do this N times" = repeats allowed — 2026-08-23

<!-- rulebook: absorbed #random-targeting-and-rng-in-effects » **Owner ruling — "N random followers" = distinct; "do this N times" = repeats allowed (2026-08-23):** -->

> "if the text says 2 random followers it can do max 8 to 1 follower ... if it says 8 to A random follower TWICE it can hit the same target twice."

Two printed phrasings with different semantics, and they must be authored differently:

- **"deal X damage to N random enemy followers"** → N **distinct** followers, each hit once. With fewer than N candidates, the surplus picks simply do nothing.
- **"Do this N times: 'deal X damage to a random enemy follower'"** → N independent rolls; the same follower can be hit repeatedly (this is Oluon's ruling generalised).

**The rulebook already said so** — Randomness: _"destroy two random enemy followers" against 3 followers — pick one at random and remove it, then pick from the remaining two; never pick the same follower twice._ The engine contradicted it: `random_hits` was documented and implemented as "with replacement", so both phrasings resolved identically.

Symptom Chris hit: **Erntz, Governing Justice** (`10544110`, "deal 8 damage to 2 random enemy followers") rolled the same follower twice for 16 and killed an 8/10 Azurifrit that should have survived at 8/2.

Blast radius found by scan — 7 cards were wrong (Erntz, Katalina `10401110`, March of the Brutes `10351310`, Unleashed `10432310`, Elder Sagebrush `10111150`, Band of Battle Princesses `10222310`, Waterbending Charmwielder `10531120`); 11 "do this N times" cards were already correct and must keep with-replacement behaviour.

## "Attacked a leader last turn" — the attack counts, not the damage — 2026-08-29

<!-- rulebook: absorbed #combat-timing-specifics » **Owner ruling — "Attacked a leader last turn" counts the attack, not the damage (2026-08-29):** -->

Ruling given on a direct question, for the five set-10009 cards reading _"If an allied follower attacked a leader on your last turn …"_: **"Attack committed counts."**

An allied follower satisfies the condition the moment it commits an attack on the enemy leader. Damage is irrelevant — a swing that deals 0 still counts. A Ward-blocked attack does **not** count, because it never reaches the leader at all.

Engine: `PlayerState.anyAllyAttackedLeaderThisTurn` / `allyAttackedLeaderLastTurn`, set post-validation when the attack commits, snapshotted at end of turn. The flags live **inside** the snapshotted state so undo/redo and replay stay correct (an out-of-snapshot flag is exactly what silently broke undo on Galmieux's passive). Unlocks: Ripper-Clawed Thief, High-Spirited Marauder, Barren-Earth Tyrant, Artiglio, Antemaria.

## Fuse mechanics — Sephie and Ecstatic Scholar — 2026-08-29

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Fuse mechanics, Sephie and Ecstatic Scholar (2026-08-29):** -->

> "sephie and ecstatic scholar do not need any recipes. sephie needs effect on fuse. you can fuse (as many cards as you want to ONE sephie ONCE per turn (if you have 2 sephies in hand ofc you can fuse to both of them ONCE) but you need 2 playpoints available for it to do something. if you have 2 playpoints and you fuse a card to her she will spawn a obsessed test subject on the field."

**Fuse needs no recipe list.** Any card may be fused. The limit is **once per turn per Sephie instance** — with two Sephies in hand you may fuse once to each in the same turn.

**Fusing is always legal, even at 0 PP** — clarified 2026-08-29:

> "the 2 playpoints are spent if you fuse yes and the test subject gets summoned. if you fuse with less than 2 then you just fuse and you dont pay anything and nothing spawns. its important that you CAN fuse even if you dont have playpoints just nothing happens really other than the cards you fused to it are gone from hand"

So: the fuse action is never gated on play points, the fused cards always leave hand, and the "was fused" flag is always set. Only Sephie's **on-fuse effect** is conditional — with **≥2 PP available, exactly 2 PP are SPENT** and an **Obsessed Test Subject** is summoned; with fewer, nothing is spent and nothing spawns. A 0-PP fuse still consumes that Sephie's once-per-turn fuse, and still satisfies Ecstatic Scholar's "was fused" requirement.

> "ecstatic scholar ... her superevolve only works if you fused a card to her when she was still in hand (similar to garden's allure and returning slash (those 2 are spells but also have an additional effect if you fused them). if ecstatic scholar was fused then if she superevolves she can select a test subject on the field and give it drain. only if she fused at least once. if she didnt nothing happens when superevolving (except for the +3/+3 and other superevo buffs ofc)"

**"Was fused while in hand" is a persistent per-card flag**, not a recipe. Ecstatic Scholar's Super-Evolve grants its extra effect (select a Test Subject on the field, give it Drain) **only if she was fused at least once while in hand**; otherwise super-evolution gives only the normal +3/+3. The same flag drives **Garden's Allure** and **Returning Slash** (spells with an additional effect when fused). The flag must survive the hand→field transition and belongs inside the snapshotted state so undo stays correct.

## Tokens live in set "Basic A" (90000) — 2026-08-29

<!-- rulebook: absorbed #card-types-and-terminology » **Owner ruling — Tokens live in set "Basic A" (90000) (2026-08-29):** -->

> "the tokens are all in the set basic A for god knows what reason. you need cursor to get them from there. they are all starting with numbers with 9000"

Token cards (summoned/added-by-effect cards that are not in any collectible set) are published under set **[90000] Basic A** with ids beginning `9000…`. Any token a card references must be ingested from there — e.g. **Dread Pirate's Flag** `90021210` (Swordcraft Amulet, Countdown (7), "Whenever you play a spell, advance this amulet's count by 1. Last Words: Deal 2 damage to the enemy leader.") and **Warden of the Trigger** (Portalcraft Artifact Follower 3/3, Ward, "Last Words: Restore 2 defense to your leader.").

## Trap in the Woods is an AMULET — the data source is wrong — 2026-08-29

<!-- rulebook: absorbed #classmechanic-specific-keywords » _Trap in the Woods (`10911210`) (2026-08-29):_ is an **Amulet**, not a Spell -->

> "trap in the woods is an amulet not a handtrap spell the fucking site is wrong for over a year now."

`10911210` Trap in the Woods is an **Amulet**, not a Spell. The scraped source has had this wrong for over a year. This is the same class of source defect as the Spell/Amulet mistypes the type linter already catches (Azvaldt, Juratio) — **owner correction beats the source**.

## Set 10009 authoring rulings — 2026-08-29

<!-- rulebook: pending — composite log entry spans multiple card-specific rulebook bullets; no single unique locator for the whole heading -->

Four questions asked before authoring the last set-10009 cards; answers verbatim.

**Barren-Earth Tyrant (`10943110`) — the repeat may hit the same follower twice.**

> Q: When "do it 2 times instead" fires, can both hits land on the same follower? — A: **"Yes — same one can eat 8."**

Confirms the general ruling above ("do this N times" = independent rolls, repeats allowed) against the contrasting "N random followers" = distinct rule. Authored as `distribution: "random_hits"` with `count: 2`, never `random_distinct`. A single enemy follower therefore takes 4 twice.

**"Damage split between all enemy followers" — oldest first, spilling onward.**

> "Aragavy, Eternal Hunter should have the logic on how it works. oldest follower gets hit first until it dies then second oldest follower etc."

The split is **sequential by board age**, not even division and not random: damage pours into the oldest enemy follower until it dies, then the remainder continues into the next-oldest, and so on until the total is spent. `10154130` Aragavy, Eternal Hunter (`"distribution": "split_sequential"`, amount 7) is the reference implementation; **Artiglio** (`10943310`) uses the identical shape at amount 3, or 6 when an allied follower attacked a leader last turn — _instead of_ 3, not in addition. Neither card spills to the leader (`spill_to_leader` stays off): the text says followers.

**Initiation of Rebirth (`10901310`) — the copy is shuffled to a random position.**

> Q: Where in the deck does the copy go? — A: **"Shuffled to a random spot."**

Not top, not bottom — a random position in the deck, drawn from the match RNG so replay and undo stay deterministic. Ordering: the copy is added to the deck _before_ the draw resolves, so it is theoretically drawable by the card's own draw. With an empty destroyed history, no card is added but **the draw still happens** — the spell must not fizzle wholesale.

**`shuffle: false` does not mean "do not shuffle."** On the `deck` add op it means insert the copy at `state.rng.nextInt(deck.length + 1)` — a random index — leaving the rest of the deck order undisturbed. That is exactly this _"Shuffled to a random spot"_ ruling. The alternative branch (`shuffle` not false) pushes then shuffles the whole deck (`src/logic/effects/deck.ts`). Naming reads backwards; do not "fix" it to a no-shuffle append.

~~Ties on "the highest base cost" are broken randomly among the tied cards (assumed, not contested).~~ **Settled 2026-09-02** — see **Initiation of Rebirth highest-base-cost ties** below. The printed word **random** already answered it; the "assumed" note should have prompted a re-read, not an escalation.

Note this card's _"without revealing it"_ is settled by the standing **hidden information — always visible** ruling: nothing is actually concealed in this tool; the only requirement is that the battle log not name the card.

**Azvaldt, Penitentiary of Chaos (`10903210`) — counts itself on its own ladder. CONFIRMED 2026-08-29.**

> Q: Does playing the 8-cost Azvaldt tick the `8` slot? — A: **"never tested, but i assume so yes."**

The owner confirmed it with the reasoning, not just an assumption: _"azvaldt has to count itself since it says at end of turn if you played. so yes it should count itself at the end of turn."_ The check happens at end of turn, by which point Azvaldt has itself been played, so its own base cost 8 is already in the played-this-match history. The ladder `1,2,3,4,5,6,7,8` can therefore complete on the very turn Azvaldt lands. This supersedes the earlier hedge — it is now a settled ruling and no longer needs live retesting.

Its Last Words ordering is also load-bearing: the 4 differently-named followers are summoned **first**, then "+3/+3 to all allied followers on the field" catches them too.

## Elmott, Remembrance Aflame (`10433110`) — name typo in the database — 2026-08-29

<!-- rulebook: engine-internal — card name typo in database; dangling-ref data correction, no rules statement to absorb -->

> "elmott exists." (with the official card page)

The card is **"Elmott, Remembrance Aflame"**. The database had **"Alflame"** (extra L) on the card record while the crest it grants was already named correctly — so the card's own Super-Evolve produced a crest whose name did not match any card, a dangling reference. Everything else on the record matched the source (Runecraft Follower, cost 3, 2/1, Gold, Skybound Dragons 10004; crest: "At the start of your turn, deal 1 damage to the enemy leader"). Card names are matching keys for decklist import, `add_to_hand`/`summon`/crest references and the dangling-reference scan, so a typo here is a silent functional break, not cosmetic. Fixed alongside a permanent gate that fails on any dangling card-name reference.

## Dazzling Runeknight (`10031110`) costs 3 — the source was right, we were wrong — 2026-08-29

<!-- rulebook: engine-internal — printed cost correction in card data; source was right, no rules statement to absorb -->

> "dazzling runeknight is 3 cost"

Local data had cost **2**; the scraped source said **3**. Owner confirmed **3**. Corrected in `cards/sets/10000_basic.json`.

Worth recording because it runs the _opposite_ direction to the two nearby corrections. Trap in the Woods and the 29 Engage-amulet type rows are cases where the source is wrong and local wins; Elmott was our own typo. Here the source was simply right. The lesson is that "the site is often wrong" is not a licence to assume local is correct — each mismatch needs its own verdict, and the ones that cannot be settled from the card image go to the owner rather than being guessed. This card is in the Basic set, so it is legal in every Runecraft deck and the extra play point changes real curve decisions.

## Third instance of the silently-ignored-key species — `until_eot` on stat ops — 2026-08-29

<!-- rulebook: engine-internal — silently-ignored-key audit incident and check:duration-op gate; Yube behaviour is documented in rulebook card-specific but this log entry is engine-data hygiene -->

Found while verifying the set-10009 batch A PR, not by live play. The engine read `until_eot` on **cost** ops (`src/logic/effects/cost.ts:188,224`) and on **attacks_per_turn** ops (`src/logic/effects/attacks.ts:81,105`), but **not** on **stat** ops — `ops/stat/duration.ts` and `ops/stat/core.ts` checked only `until_end_of_turn`.

Scan over every card: ten use `until_eot`. Seven on cost ops and two on attacks_per_turn ops — all fine. Exactly one used it on a stat op:

**`10544120` Yube, Crestpetal** — her Evolve crest reads _"Whenever an allied Marine follower attacks, give it +1/+0 until the end of the turn…"_. The buff was **permanent**, and stacked on every attack, for as long as the crest was out.

This is the same species as **Zeta & Bea** (top-level `name`/`not_self` on a stat op) and **Sara** (`filter:{damaged:true}` on a destroy op): card data expressing an intent through a key no code path reads, with no error and no warning. Fixed by honouring `until_eot` as an alias, with a Yube regression test, and gated permanently by a new `check:duration-op` — a duration key on an op whose handler ignores it is now a CI error, with the allowlist derived from the engine's consumption paths rather than from what the data currently happens to contain.

**Standing lesson, now three-for-three:** every time a new key is introduced in card data, the question "does anything actually read this?" has to be answered from the engine source, not from the fact that tests pass. Tests pass because they assert what the engine does.

## Barrier inside split damage — allocation is spent even when absorbed — 2026-08-29

<!-- rulebook: absorbed #damage-events-general » **Owner ruling — Barrier in split damage (2026-08-29):** -->

> "if you have 10 points of split damage and a 1/6 with barrier and a 1/5 without barrier: the 1/6 with barrier will take 6 damage reduced to 0 cause of barrier (so still 1/6 but now barrier is gone) and the 1/5 without barrier will now be 1/1 as it takes 4 damage spilled over. oldest to newest"

Split damage allocates against each follower's **current defense**, in board order oldest to newest, and the allocation is consumed whether or not the damage actually lands. Barrier reduces the _dealt_ damage to 0 and is itself consumed, but it does **not** hand the allocated points back to the pool.

Worked example, 10 points of split damage:

| Order  | Follower         | Allocated                 | Result                               |
| ------ | ---------------- | ------------------------- | ------------------------------------ |
| oldest | 1/6 with Barrier | 6 (= its current defense) | takes 0, still **1/6**, Barrier gone |
| next   | 1/5 no Barrier   | 4 (the remaining spill)   | **1/1**                              |

So a cheap high-defense Barrier body is a genuine soak against split damage — it eats its full defense worth of the pool and the followers behind it are protected by that much.

**The engine was already correct.** `applySplitSpillover` (`src/logic/effects/ops/damage/primitives.ts:386`) walks recipients in pool order, deals `min(remaining, currentDefense)` to each, and decrements `remaining` by the allocation rather than by the damage actually dealt. This was raised as an open question during set-10009 authoring and pinned by a test _documenting the engine_; that test now pins a **ruling** and its comment should say so. Affects the 9 split-damage cards: Aragavy, Glade, Artiglio, Marwynn, Shining Disenchantment, Flight of the Swarmpetal, Miroku, Ruinbringer, Ludicrous Ordnance, Hark to the Night Song.

## Witch's New Brew always wins an Earth Sigil merge — 2026-08-30

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Merge survivor (owner ruling, 2026-08-30):** When Earth Sigils merge -->

> "witchs new brew is the one that wins when they merge. magic sediment is a 'magically' generated amulet. witchs new brew is an actual amulet you play from hand. so lets say magic sediment is on the field with 2 counters. you play witches new brew: magic sedmient amulet gets destroyed and replaced by the witches new brew that now has 3 counters 2 from before + the one from witches brew. if witches new brew is on the field and you play something that generates magic sediment it will just increase the counter of witches new brew. so witches new brew always wins"

The ruling is **order-independent** — the Brew survives whether it arrives first or second:

| Situation                                                                    | Result                                                                                 |
| ---------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Magic Sediment on field with 2 counters, then **Witch's New Brew is played** | Sediment destroyed; Brew survives carrying **3** (the Sediment's 2 + the Brew's own 1) |
| Witch's New Brew on field, then an effect **generates a Magic Sediment**     | No new amulet appears; the Brew's counter simply goes up by 1                          |

This settles a question raised during the card-name cleanup. The engine had a `findEarthSigilTarget()` that preferred a Brew over a Sediment **by matching card-name strings**, and the obvious "general" replacement — oldest sigil survives — would have been **wrong in both directions**. The owner volunteered the rule before that landed.

The cleanup therefore kept the behaviour and changed only how the ranking is expressed: **a collectible Earth Sigil outranks a token one**, derived from data (Witch's New Brew is `[10000] Basic` in the card pool; Magic Sediment is `[90000] Basic A` in `token_details.json`), which is exactly the distinction in the owner's own words — "an actual amulet you play from hand" versus "magically generated". Ranking by cost, rarity or "has a Fanfare" was explicitly rejected: those are incidental to these two cards and would break on the next printing.

Related and already settled: the stacking itself is correct per the rulebook — _"Earth Sigils are a counted resource, not one-per-amulet ... sigils merge additively onto a single amulet rather than each taking a board slot."_

## Hien vs Bayle — the same clause shape, two different durations — 2026-08-30

<!-- rulebook: absorbed #classmechanic-specific-keywords » _Hien, Redolent Revenant (`10914120`) vs Bayle, Luxglaive Warrior (`10113130`) (2026-08-30):_ -->

> "Bayle, Luxglaive Warrior should already have cost reduction implemented whenever an allied follower leaves the field. in his case the reduction is permanent in her case it is not permanent and also her reduction is if you just play any card not just if an allied follower leaves the field."

Two in-hand cost-reduction cards that must be authored differently:

- **`10113130` Bayle, Luxglaive Warrior** — _"Whenever an allied follower leaves the field, reduce the cost of this card by 1."_ **Permanent.** Trigger `ally_follower_leaves_field`, no duration key.
- **`10914120` Hien, Redolent Revenant** — _"Whenever you play a card, reduce the cost of this card by 1 **until the end of the turn**."_ **Expires each turn.** Trigger `ally_card_played` (any allied card — follower, spell or amulet, not just followers leaving), with `until_eot`.

Hien's entire in-hand line was **unauthored** — she had only a Fanfare and Last Words, so a 9-cost 4/4 whose whole design is ramping herself down sat at 9 forever. She still reported as "implemented" by `check:card-status` because she had _some_ ops, which is exactly how it hid. The load-bearing test is the expiry: play three cards, Hien is at 6; next turn she is back at 9.

## Fourth instance of the silently-ignored-key species — `named_enter_count` counts the entering card — 2026-08-30

<!-- rulebook: absorbed #fanfare-and-enter-play-trigger-order » **Owner ruling — `named_enter_count` / "other" copies and enter-route timing (2026-08-30):** -->

Reported by another session, then reproduced and measured before acting. `10931110` **Obsessed Test Subject** reads _"if at least 5 **other** allied copies ... have entered the field this match, give it +3/+3"_, so the 6th copy should be the first buffed. Measured on the old main: **the 5th was buffed.**

Cause: `src/logic/effects/ops/summon_ops/core.ts` calls `recordFollowerEnter` **before** firing `ally_follower_enter`, so a card driven by an enter trigger is already in its own history when its gate evaluates.

The blast-radius scan is what made the fix non-obvious. Only two cards use `named_enter_count`, they use the _identical_ English phrase, and they behave _differently_ because of the route:

- **`10844110` Drache & Aluzard** — Fanfare route. **Already correct** (measured: 1st play +0/+0, 2nd +1/+1, 3rd +2/+2 then evolve). The play-from-hand path records the enter _after_ Fanfare, which `followerEnterHistory.ts` documents as deliberate.
- **`10931110` Obsessed Test Subject** — enter-trigger route. Off by one.

So a global "exclude self" would have broken Drache. Drache's three measured rows were pinned as a test before the fix went in. Lesson, now four-for-four with Zeta & Bea, Sara and Yube: **the same key can be honoured on one route and ignored on another — check the route, not just the key.**

## Card text is bible — 2026-08-31

<!-- rulebook: absorbed #shadowverse-worlds-beyond-rules-reference » **Owner ruling — Card text is bible (2026-08-31):** -->

> "card text is bible."

Printed card text governs over authored JSON wherever they disagree. The authored data is **never itself evidence of intent** — a card can ship wrong for months.

## Turn-boundary triggers are owner-scoped — 2026-08-31

<!-- rulebook: absorbed #start-of-turn-and-end-of-turn-sequences » **Owner ruling — Turn-boundary triggers are owner-scoped (2026-08-31):** -->

A turn-boundary trigger printed _"at the start/end of **your** turn"_ fires only on its owner's boundary. `whose_turn: "opponent"` is the explicit opt-out for cards printed _"at the end of your opponent's turn"_ (Lilanthim `10734110`). The defect being fixed was that _absence_ of the key meant "fire for everyone", which in a mirror made your own turn start add a Fairy to **both** hands.

## "Takes N more damage" applies to a 0-damage event — 2026-08-31

<!-- rulebook: absorbed #damage-events-general » **Owner ruling — "Takes N more damage" applies to a 0-damage event (2026-08-31):** -->

> "id say so yes. since when i attack with a 0 attack in game it deals 0 damage so +1 would be 1. lets keep it until i ever see a situation where that contradicts itself."

A 0-damage event **does** take the bonus (0 + 1 = 1). Healing does not.

## Skybound gauge — every allied evolve counts — 2026-08-10

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Skybound gauge, every allied evolve counts (2026-08-10):** -->

Every evolve of an allied follower counts for the Skybound gauge, regardless of whether an `Evolve:` script fired or EP was spent.

## Hand overflow destroys without Last Words — 2026-08-10

<!-- rulebook: absorbed #zones-and-card-states » **Owner ruling (2026-08-10):** a card destroyed by hand overflow triggers **no** Last Words -->

A card destroyed by hand overflow triggers **no** Last Words — "converted into a shadow" beats "destroyed".

## Accelerate and spells — 2026-09-02

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Accelerate and Spellboost / hand-deck type (2026-09-02):** -->

Quoting the source the owner supplied:

> "Accelerate will function properly with Spellboost and other mechanics that interact with spells.
> However, Accelerate followers will not be affected by cards that would interact with spells while
> they are in hand or deck, such as Wizardess of Oz."

An Accelerate play **is** a spell play for Spellboost and every other spell-interacting mechanic. While the card is still in hand or deck, its **printed type** governs — an Accelerate follower is a follower there, not a spell, so hand/deck effects that filter on spells do not see it.

**Reachability correction:** an earlier note in this file claimed the Accelerate × Spellboost interaction was unreachable because every Accelerate card was Portalcraft or Dragoncraft and every Spellboost card is Runecraft. That was **wrong**. **`10901110` Jailor of Antiquity** is Neutral (Follower, cost 6, Accelerate (1)), so it is legal in every craft — including Runecraft alongside Spellboost, and Abysscraft alongside Reanimate.

## Alternate-form permanence (Accelerate / Crystallize) — 2026-09-02

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Crystallize / Accelerate alternate-form permanence (2026-09-02):** -->

Asked whether an Accelerate-played follower in the cemetery should be reanimatable. Answer: **no**. The owner reasoned by analogy from Crystallize and quoted:

> "Crystallize followers played as amulets are only treated as such while on the field. Any effect
> that would interact with amulets in the hand or deck will not affect Crystallize. Creating/Returning
> an amulet created by a Crystallize follower to the hand will not return it to being a follower and
> it will remain an amulet."

General rule in both directions: **the printed type governs while the card is in hand or deck; once the card has been played in its alternate form, it stays in that form and never reverts** — including in the cemetery. An Accelerate-played follower is a spell corpse and is invisible to Reanimate. A Crystallize-played card is an amulet for the rest of its life; bouncing it to hand does not restore the follower.

## Fused cards are banished — 2026-09-02

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Fused partners are banished (2026-09-02).** -->

Live-game bug report (owner, verbatim):

> "i fused lyria in turn1 with sephie to create a test subject. fused cards are banished- gone -fairy dust-fugazi. wills united reanimated lyria - massive bug!"

**Fused partners are banished**, not sent to the cemetery. They produce **no shadow**. This clarifies the earlier **Fuse mechanics — Sephie and Ecstatic Scholar — 2026-08-29** ruling, which said only that fused cards are _"gone from hand"_ and did **not** say where they go — it does **not** reverse that ruling.

## Reanimate only sees field-destroyed followers — 2026-09-02

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Reanimate provenance (2026-09-02):** -->

Same report, clarifying (owner, verbatim):

> "she was never even on the field. not even discarded cards can get reanimated. so its extra
> preposterous that a fused card got renanimated"

**Reanimate (and any `source: "graveyard"` follower summon that is Reanimate)** may only pick followers that were **destroyed on the field**. A card that reached the cemetery any other way — discarded from hand, fused (now banished, so not even in the cemetery), burned / hand-overflow, Engage-consumed amulet material, etc. — is **not** a legal Reanimate target. It was never on the field, so it never died there.

This matches the printed Reanimate keyword's "died this match" language and is enforced via `destroyedHistory` (written only at the genuine destruction sites). Discards still generate shadows exactly as before; this ruling is about **eligibility**, not the shadow count.

## Initiation of Rebirth highest-base-cost ties — 2026-09-02

<!-- rulebook: absorbed #random-targeting-and-rng-in-effects » Owner ruling 2026-09-02: _"it says random highest cost so that means if some are tied pick randomly between them."_ -->

Printed text: _"Add a copy of a **random** allied follower destroyed this match with the highest base cost to your deck without revealing it. Draw a card."_

Owner, settling the old "assumed, not contested" open:

> "it says random highest cost so that means if some are tied pick randomly between them"

Uniformly at random among the destroyed allied followers that share the highest base cost. Cheaper corpses are never eligible. This was **never really open** — the printed text answered it, and the "assumed, not contested" note on the 2026-08-29 authoring ruling should have prompted a re-read of the card rather than an escalation. Worked example of **card text is bible** (2026-08-31).

Engine already matched (`pickDestroyedMatchHighestBaseCost` keeps every record at `maxBase`, then `top[state.rng.nextInt(top.length)]`). Behaviour pinned; do not change without a new ruling.

## Alternate and Enhance costs are fixed; only the card's own cost moves — 2026-09-02

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Alternate and Enhance costs are fixed (2026-09-02):** -->

> "you cant reduce the accelerate cost. you can only reduce (or increase) the cost of the follower itself
> if thats what you are asking. same thing for enhance or crystallize"

A cost reduction or increase changes the card's own effective play cost. It **never** touches the
**Accelerate**, **Crystallize** or **Enhance** value N — those are fixed alternate costs as printed.

**The consequence is counter-intuitive and is the part worth testing.** The alternate-form gate compares
available PP against the card's _effective_ cost, so moving that cost moves the threshold: reducing a
follower's cost makes its Accelerate/Crystallize form **harder** to reach, not cheaper. Shoddy Plaything
(`10671110`, base 6 / Accelerate 2) at 5 PP plays via Accelerate; reduce its cost to 4 and the same 5 PP
plays the printed follower instead.

**Enhance is never declinable, and a reduced cost is not an exception:**

> "huh no you cant choose. i just said you only have those 2 option. cost reduced of l'age d'or to 1pp:
> either you would be at 1-5pp and play it for 1 but as soon as oyu hit 6pp available it jumps to 6pp
> cost for the enhance"

The "two options" are the two outcomes decided by your PP, not a player choice. L'Age d'Or (`10923310`,
printed 4 / Enhance 6) reduced to 1 plays for 1 at 1–5 PP and jumps to 6 and enhances at 6+.

Engine: `resolvePlayCost` (`src/logic/core/playCard/cost.ts`) tries `pickEnhanceTiers` first and falls
through to the normal cost only when no tier is affordable; `pickAlternateForm`
(`src/helpers/alternateForm.ts`) compares available PP against the **effective** cost while reading the
alternate's **printed** N. Both already correct.

## Crest and Faith slots are capped at five; a sixth is ignored — 2026-09-05

<!-- rulebook: absorbed #zones-and-card-locations » **Owner ruling — Crest and Faith slots are capped at five (2026-09-05):** -->

> "the crests / faith slots are capped at 5 yes"

> "6th crest just bounces of (gets ignored). In the past meta maddening benison was abused by crest
> haven. When you have 5 crests the maddening benison crest effect deal 10 DMG to your leader doesn't
> happen since the crest had no space."

The crest area holds at most five, and **Faith counts toward the five**. A gain beyond the cap is a
silent no-op: the crest never exists, so neither do its Countdown or its Last Words.

The worked consequence is a real play pattern: Maddening Benison `10263310` (_Restore 10 defense to your
leader. Gain Crest: Maddening Benison_ — crest: Countdown (2), Last Words: deal 10 damage to your leader)
at five crests is a **drawback-free 10 heal** — the restore resolves, the crest gain does nothing, and
the self-damage never comes.

Engine: implemented — `handleGainCrest` (`src/logic/effects/crest.ts`) refuses past `MAX_CREST_SLOTS`
with `reason: "slot_cap"`.

## A duplicate crest bounces off the active one — no refresh — 2026-09-05

<!-- rulebook: absorbed #zones-and-card-locations » **Owner ruling — A duplicate crest bounces off the active one (2026-09-05):** -->

> "if you play a duplicate crest the crest doesn't get replaced it bounces of at the active one. Say 1
> crest has countdown 4 and counts down to 1 and you play it again it will stay at 1 and not go back up
> to 4."

Re-gaining a crest you already have does nothing at all — it does not replace the instance and does not
reset its Countdown. A crest at Countdown 1, re-gained, stays at 1, and there is still exactly one.

Note this differs from SWB-RL, whose default **replaces** the instance; do not port that behaviour.

Engine: implemented — `crest.ts:117` returns early when `crests.some((c) => c.name === crestName)`.

## Last Words summons never spawn in place — 2026-09-05

<!-- rulebook: pending — Last Words board-compaction summon placement not written through to rulebook -->

> "When the board is full and a follower with Last Words 'summon another follower' dies, he won't spawn
> in place. The other older followers to the right of his will shuffle left and the Last Words-summoned
> follower will be the new rightmost — first in is left and last in is right."

A dead follower's slot is not held open for its own Last Words summon. The board **compacts first** —
survivors to the right shift left — and the summoned follower is then **appended on the right**, making
it the newest. Board order is entry order: first in is leftmost, last in is rightmost.

This matters for every position-sensitive effect: "the leftmost allied follower", `distribution:
"leftmost"`, split damage by board age, and anything that reads entry order.

## Every multi-card summon resolves one card after another — 2026-09-05

<!-- rulebook: pending — sequential multi-summon resolution not written through to rulebook -->

> "they never appear at the same time, it's always one after the other (even if it looks instant by eye)"

A `summon` with `count: N` is **N sequential entries**, never one simultaneous arrival. Each entry raises
its own enter trigger, and each trigger's condition is judged at its own moment.

The worked case the owner gave: Sephie's Fanfare with exactly four Obsessed Test Subjects already
entered summons _"a 2/2 and then 5/5"_ — the fifth copy's _"if at least 5 **other** allied copies have
entered"_ is judged as it enters (four others, no buff), and the sixth's as it enters (five others,
+3/+3). Two copies from one "summon 2" are **not** simultaneous.

## Krulle, Heir to Unkilling (`10314110`) — his own Fanfare satisfies his own heal — 2026-09-07

<!-- rulebook: pending — Krulle self-Fanfare heal rule not written through to rulebook -->

> "Krulle when he comes onto the field and gives enemy followers -0/-2 he heals the leader 1 if he is
> successful."

Printed: _"Fanfare: Give all enemy followers on the field -0/-2. | Ambush | Once on each of your turns,
when an enemy follower is given -defense on the field, restore 1 defense to your leader. | Super-Evolve:
Give your opponent Crest: Krulle, Heir to Unkilling."_

**A card's own Fanfare debuff satisfies its own "when an enemy follower is given -defense" clause.**
Playing Krulle with at least one enemy follower on the field restores 1 defense to your leader in the
same resolution.

**The qualifier is load-bearing: "if he is successful."** The heal is conditional on the debuff actually
landing. With no enemy followers on the field the `-0/-2` does nothing, so there is **no heal** — a fix
that heals unconditionally on play is wrong, and that negative case is exactly what a naive fix breaks.

`10314110` is the only card in the pool that listens for `enemy_follower_defense_down`.

Engine: the bug reported on 2026-09-07 (no heal, from either route) is **fixed**; three tests in
`tests/unit/d4a-clause-gap.test.ts` pin it — the self-Fanfare case, an external-debuff discriminator, and
the no-enemy-followers negative — and all three pass on `main`.

## Beelzebub, Supreme King (`10474120`) — "Takes 1 more damage" is permanent and stacks — 2026-09-07

<!-- rulebook: pending — Beelzebub stacking leader damage bonus not written through to rulebook -->

> "if you play him 3 times in a game every damage instance to the enemy leader will deal +3. realistically
> you play him once and everything is +1 to the enemy leader attacks or spells."

Printed: _"Fanfare: Select 2 enemy followers on the field, remove all abilities from them, and deal them
9 damage. **Give the enemy leader 'Takes 1 more damage.'**"_

- **Permanent for the rest of the match** — not per-turn, and it does not expire.
- **Stacks additively per instance played.** Three Beelzebubs = **+1 each**, so +3 on every damage
  instance to that leader.
- **Applies per damage instance, whatever the source.** Five separate 1-damage pings each become 2, not
  one lump +1.
- Interacts with the 2026-08-31 ruling: a **0-damage event still takes the bonus** (0 + 1 = 1). Healing
  does not.

**Confirmed by three official Cygames Q&A**, which settle the general shape beyond the owner's words:

1. Multiple copies _can_ give the debuff multiple times — stacking confirmed by the publisher.
2. Beryl, Nightmare Incarnate's Fanfare into your own leader carrying the debuff deals **4**, not 3 — the
   bonus applies to **self-inflicted** damage from your own card.
3. An enemy super-evolved follower destroying one of your followers deals **2** to your leader — the
   bonus applies to **trigger** damage, not only attacks and spells.

So the bonus is a property of the **damage pipeline into that leader**, not of any source category.

**"Vulnerable" is not a keyword.** The authored JSON grants `keywords: [{"name": "Vulnerable", "value":
1}]`; there is no such keyword in this game and the printed text is the quoted string _"Takes 1 more
damage."_ The name appears in exactly one card's JSON. It is a naming hazard, not a bug — the behaviour
is correct — and renaming it to read like the printed text is optional cleanup, not part of any fix.

Engine: implemented as leader state — `PlayerState.leaderDamageTakenBonus`, granted with `+=` (so
stacking is already correct) and consumed in `applyLeaderDamage`, the single centralised leader-damage
entry point, which is why (2) and (3) hold without special cases. Two defects noted when this was ruled
are both since fixed: the consumer guard is now `if (mod !== 0)`, and the dead
`handleModifyLeaderDamageReceived` has been removed.

## `still_alive` — subject is the damage victim (2026-09-08)

<!-- rulebook: absorbed #damage-events-general » **Owner ruling — `still_alive` subject is the damage victim (2026-09-08):** -->

Asked what the key means, the owner answered with Galmieux, verbatim:

> "ill take galmieux as example here. Galmieux gains the crest that when an ally is damaged and survives the damage then you get a 0 mana spell to hand. ONLY if the unit that was daamged survives -> hence the still alive. What is unclear?"

**The subject is the card that took the damage.** It is a property of a damage event's victim, and it is only meaningful where there _is_ a damage event.

**Consequence:** `still_alive` is **trigger-only** — not a pool/card filter key. Its subject is the damage victim; for `self_damaged` triggers this is implemented in `src/logic/core/triggers/handlers/self.ts`.

## A printed "Select" means the player picks (2026-09-08)

<!-- rulebook: absorbed #targeting-rules » **Owner ruling — A printed "Select" opens a target prompt (2026-09-08):** -->

Asked whether Titania, Queen of Fairies `10214110` and Ara, Dawnblossom `10534120` should open a target
prompt on evolve — measured, they opened none and the engine silently transformed the leftmost
candidate — the owner answered, verbatim:

> "Both say select so yes you should be able to chooose ofc"

**So a card whose printed text says _Select_ must open a target prompt.** Fixed in PR #368: `op:
"transform"` ignored `select` on its board route and sliced the first N of the pool, while `damage` and
`destroy` with the identical `target`/`select` shape on the identical route prompted correctly. Three
cards were affected — Titania `10214110` (evolve), Ara `10534120` (evolve), Sincerity of the Dewdrop
`10573310` (spell); Vier `10272110` uses the hand route and was already correct.

Corollary measured at the same time: the engine already excludes the source card from a targeted pool,
so Ara's _"another follower"_ needs no `exclude_self`. That key is load-bearing only where there is no
targeted pool (`10553110` Lifestealer, `distribution: "all"`).

## "A card on the field" includes amulets (2026-09-09)

<!-- rulebook: absorbed #targeting-and-selection-effects » **Owner ruling — "A card on the field" includes amulets (2026-09-09):** -->

Sincerity of the Dewdrop `10573310` prints _"Select a card on the field and transform it into an Imari's
Little Buddies"_ and targets `any:any`, but the board route of `op:"transform"` narrowed its pool to
`type === "Follower"`, so an allied amulet was never offered. Asked whether it should be selectable, the
owner answered, verbatim:

> "Yes it can"

**So `:any` means any card on the field, amulets included.** The narrowing must come from the target's
own kind, not from a hard-coded type. Sincerity is the only card in the pool whose board-route transform
target is `:any`; the other five board-route transforms all target `:follower`, where the narrowing was
already redundant. Fixed in PR #378.

## An amulet destroyed by its own Engage is destroyed (2026-09-09)

<!-- rulebook: absorbed #destruction-vs-other-removal » **Owner ruling — Engage self-sacrifice is destruction (2026-09-09):** -->

Reported from live play: an Engage-sacrifice amulet did not increase Lyanthoth, Eld Tome `10664120`'s
Faith and did not make Omerio, Winged Revenant `10964120` react. The owner:

> "but i think they should because they say engage destroy"

**Official source, checked at his request.** Cygames' own Q&A answers the "can I engage this with no
legal target?" question for `10001210`, `10002210`, `10112210`, `10113210` and `10162220`, and every
answer says the amulet is **destroyed** — e.g. _"Yes. Doing so will simply destroy it."_ The Japanese
effect-processing spec has no destroy-vs-cost distinction, and no Q&A covers the trigger interaction
directly.

**So an amulet consumed by its own Engage is destroyed and raises `ally_amulet_destroyed`.** Fixed in
PR #381: `engage.ts` had a private removal helper that never fired the trigger and never recorded the
destruction; all three of its call sites — sacrifice, sacrifice-with-selection, and countdown reaching 0
during the engage — now go through the shared `destroyTarget`. 22 amulets carry `Engage` with
`sacrifice: true`; the event's only consumers are Omerio's trigger and Lyanthoth's Faith crest.

## Transform — neither leave nor enter (2026-09-08)

<!-- rulebook: absorbed #destruction-vs-other-removal » **Owner ruling (2026-09-08):** transform is not a leave; the transformed-in card is not an enter. -->

> Transform does **not** count as leaving the field. The transformed-in card does **not** count as entering the field.

**Owner ruling (2026-09-08).** **Source:** [Shadowverse 効果処理 wiki — 変身と破壊の違い](https://w.atwiki.jp/svkoukasyori/pages/16.html), quoted:

- ラストワード・「破壊された時」「**場を離れる時**」効果が発動しない — Last Words, "when destroyed", and **"when leaving the field"** effects do not activate.
- 変身して別のカードになるが、「場に出たカード」の枚数は増えない(**場に出た扱いにもならない**) — it transforms into a different card, but the count of "cards that entered the field" does not increase (**it is not treated as having entered the field**).

Consequences: no Last Words, no shadow, no `leaves_field` / `enter` reactive triggers on transform. The original ceases to exist and continuous effects on it end; the new card takes its slot with no relation to the old one. The engine (`src/logic/effects/ops/transform.ts`) already implements this deliberately — do not fire enter/leave on transform.

There is no official Cygames Q&A on transform vs leave/enter triggers (checked all 904 entries in `cards/official-meta.json`).

## `_destroyed` events — destruction only — 2026-09-08

<!-- rulebook: pending — general destroyed-vs-leaves-play principle is at line 591 but banish/bounce must not raise destroyed-axis events is not written through with this dated ruling -->

> A `_destroyed` event must never fire for something that was not destroyed.

Banishing or bouncing a follower (or amulet) does **not** raise `ally_*_destroyed` / `enemy_*_destroyed` / `ally_amulet_destroyed` — it raises nothing on the destroyed axis. Those events are reserved for genuine destruction (defense ≤ 0 on a follower, countdown 0 on an amulet, etc.), not for any other form of leaving play.

Same principle as the rulebook at line 591 in `docs/svwb_rulebook_formatted.md`: _"Triggers that say 'destroyed' mean specifically destroyed; 'leaves play' means any removal."_ This is the rule `ally_follower_destroyed` / `enemy_follower_destroyed` are built on (Lifestealer `10553110`, PR #352). Context: #353 asked whether banishing or bouncing an amulet should raise `ally_amulet_destroyed` instead of a leave-field event — answer is no, it should raise nothing on the destroyed axis.

## Faith is not a crest for counting (2026-09-06)

<!-- rulebook: absorbed #zones-and-card-locations » Faith icons share the same five-slot cap but do not count toward "the number of crests you have" -->

Owner:

> "yes I guess for certain cards number of crests is important so faiths shouldn't count despite sharing a 'board'."

Official per-card Q&A (Shining Disenchantment `10363210`, Temple of Repose `10362210`, Himeka `10364110`, Marwynn `10364120` — _"Do faiths count as crests?"_): **"No, they don't."**

**Counting:** any effect whose amount is _"the number of crests you have"_ (`crest_count` / `amount_source: "crest_count"`) excludes Faith entries. Faith is still stored as a crest named `Faith: <card name>` with `isFaith: true`.

**Slot cap unchanged:** Faith still occupies one of the five crest/faith slots (`MAX_CREST_SLOTS`, owner ruling 2026-09-05). Four ordinary crests plus one Faith means a sixth distinct crest bounces.

## Depths of the Eld Crystals — how X, Y and Z are drawn — 2026-09-05

<!-- rulebook: pending — random_split X/Y/Z algorithm for Depths of the Eld Crystals not in rulebook -->

Owner-supplied FAQ text (his caveat: "Dont know if this is official but since I personally never seen it hit 0 I think this makes sense"): "To determine the values of X, Y, and Z, the ability first chooses X, Y, or Z at random, with each having an equal 1/3 chance of being chosen. This process is repeated a number of times equal to your faith's value … The number of times each letter is chosen then becomes its final value." So the split is one independent uniform draw per faith point; zeros are legal outcomes (1/27 for X=3,Y=0,Z=0 at faith 3). `random_split` (`src/logic/effects/ops/random_split.ts`) implements exactly this; the faith counter is read, not spent.

## Artifact fuse chain — 2026-09-05

<!-- rulebook: absorbed #classmechanic-specific-keywords » **Owner ruling — Artifact fuse chain (2026-09-05), refined by official Q&A (2026-09-06):** -->

Owner's recollection (caveat: "Its been a while so I'm not 100% sure anymore"): gears fuse only with gears and the gear fused _into_ decides the body (Ambition → Striker 5/1 Rush, Remembrance → Fortifier 1/5 Ward); Striker/Fortifier host any Artifact cards and transform by the partners' total cost (1 → Ominous α, 2 → β, 3+ → γ — which is where γ's many recipes come from); Ominous α needs β **and** γ for Masterwork Ω (only one → consumed, no transform); β, γ, Ω cannot fuse. The engine (`src/logic/effects/ops/fuse/fuse.artifact.ts`) matches all of it and the printed token texts. Open, low priority: the owner "believes" α can fuse with anything except gears — α's printed text says β and γ only, and text wins until the client shows otherwise; and both Gears' printed description in `cards/token_details.json` reads "Fuse: Artifact amulets", which matches neither the owner nor the engine (stale text, not a behaviour bug — do not edit the description in this PR).

**Official Q&A refinement (2026-09-06) — Ominous Artifact α (`90073110`):** _"If I fuse an Ominous Artifact β to an Ominous Artifact α in my hand one turn, then fuse an Ominous Artifact γ to the same Ominous Artifact α the next turn, will it transform into a Masterwork Artifact Ω?"_ → _"Yes, it will."_ A lone β or γ fused to α is still **consumed and banished** with no immediate transform (owner ruling stands), but α **remembers** which partner kinds have been fused (`fusedArtifacts` on the instance). When the second kind arrives — same fuse or a later turn — α transforms into Masterwork Ω. Already-fused kinds are excluded from the partner pool (a second β cannot be selected once β is remembered). Fresh α copies start with empty flags.

---

## Still open — Chris will test in game

~~Whether a **cost reduction moves the Accelerate value N**, or only the normal cost.~~ **Settled 2026-09-02** — see **Alternate and Enhance costs are fixed; only the card's own cost moves — 2026-09-02** above. Alternate cost N is fixed; only the card's own effective play cost moves.

~~Related and **unreachable in practice**: whether an Accelerate play triggers Spellboost.~~ **Settled 2026-09-02 — yes.** Accelerate plays trigger Spellboost (and other spell-play mechanics). The earlier "unreachable in practice" reasoning was wrong: it assumed every Accelerate card is Portalcraft or Dragoncraft, but **Jailor of Antiquity (`10901110`) is Neutral**, so any Runecraft deck can contain both. See **Accelerate and spells — 2026-09-02** and **Alternate-form permanence — 2026-09-02** above.

~~Initiation of Rebirth highest-base-cost ties (assumed, not contested).~~ **Settled 2026-09-02 — randomly among the tied cards.** See **Initiation of Rebirth highest-base-cost ties — 2026-09-02** above.
