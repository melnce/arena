# Shadowverse: Worlds Beyond — Rules Reference

An engine-oriented rules reference for Shadowverse: Worlds Beyond (SVWB). It covers match and turn flow, zones, timing windows, effect resolution, the keyword catalogue, and combat / continuous-effect rules, with test cases for implementation.

> **Owner rulings override this rulebook.** Verbatim owner rulings live in [`owner-rulings.md`](owner-rulings.md). They override both printed card text and every passage here. Consult that file before deciding any card-behaviour question; where a ruling and this rulebook disagree, the ruling wins and this rulebook is what gets corrected.

**Owner ruling — Card text is bible (2026-08-31):**

> "card text is bible."

Printed card text governs over authored JSON wherever they disagree. The authored data is **never itself evidence of intent** — a card can ship wrong for months. (Owner rulings still override printed text when they explicitly disagree with it.)

## Table of Contents

- [Match Flow](#match-flow)
- [Turn Sequence](#turn-sequence)
- [Win/Loss Conditions](#winloss-conditions)
- [Zones and Card States](#zones-and-card-states)
- [Targeting Rules](#targeting-rules)
- [Randomness](#randomness)
- [Hidden Information](#hidden-information)
- [Time and Priority](#time-and-priority)
- [Canonical Terminology and Zones](#canonical-terminology-and-zones)
  - [Zones and Card Locations](#zones-and-card-locations)
  - [Card Types and Terminology](#card-types-and-terminology)
- [Turn Structure and Timing Windows](#turn-structure-and-timing-windows)
  - [Turn Phases](#turn-phases)
  - [Play Point (PP) and Evolution Point Rules](#play-point-pp-and-evolution-point-rules)
- [Timing Windows and Trigger Resolution](#timing-windows-and-trigger-resolution)
  - [Start-of-Turn and End-of-Turn Sequences](#start-of-turn-and-end-of-turn-sequences)
  - [Combat Timing Specifics](#combat-timing-specifics)
  - [Fanfare and Enter-Play Trigger Order](#fanfare-and-enter-play-trigger-order)
- [Effect Types and Resolution Model](#effect-types-and-resolution-model)
  - [Playing Cards from Hand](#playing-cards-from-hand)
  - [Targeting and Selection Effects](#targeting-and-selection-effects)
  - [Resolution Order and Queue](#resolution-order-and-queue)
- [Random Targeting and RNG in Effects](#random-targeting-and-rng-in-effects)
- [Keywords and Mechanic Catalogue](#keywords-and-mechanic-catalogue)
  - [Evergreen Keywords (General)](#evergreen-keywords-general)
  - [Class/Mechanic-Specific Keywords](#classmechanic-specific-keywords)
  - [Damage Events (General)](#damage-events-general)
- [Combat, Damage, and Destruction Rules](#combat-damage-and-destruction-rules)
  - [Combat Procedure Recap](#combat-procedure-recap)
  - [Barrier](#barrier)
  - [Bane, Drain, and Special Damage Effects in Combat](#bane-drain-and-special-damage-effects-in-combat)
- [Destruction vs. Other Removal](#destruction-vs-other-removal)
- [Continuous Effects and Modifiers](#continuous-effects-and-modifiers)
  - [Stat Modification Hierarchy](#stat-modification-hierarchy)
- [Hidden Information and Reveal Mechanics](#hidden-information-and-reveal-mechanics)
- [Copy semantics (exact copy vs copy)](#copy-semantics-exact-copy-vs-copy)

---

## Match Flow

At the start of a match, both leaders begin at 20 defense and use pre-built 40-card decks. A coin flip determines the first-turn player. Each player draws 4 cards for their opening hand (not 3, as in original Shadowverse), then takes a one-time mulligan: any subset of those 4 cards may be shuffled back and replaced with new draws before the first turn begins. **Redraw (official glossary, 2026-09-10):** a redraw never returns the exact same card instance — if you have multiple copies in your deck you may still draw a different copy of the same card. After the mulligan, play proceeds in alternating turns until a win condition is met.

## Turn Sequence

At the start of each turn the active player increases their maximum Play Points (PP) by one (up to 10) and refills all PP orbs, then draws one card from their deck. (In Worlds Beyond this draw occurs on both players' first turns.) During the Main Phase the player may play cards (followers, spells, amulets) and declare attacks in any order — unlike games with a distinct combat phase, SVWB lets you intermix plays and attacks freely. When finished, the player ends their turn, which fires any end-of-turn effects, and passes to the opponent.

## Win/Loss Conditions

The primary win condition is reducing the opponent's leader defense to 0, through combat damage or card effects; that player wins immediately. The secondary loss condition is deck depletion: nothing happens the moment a deck empties, but the next time a player must _draw_ from an empty deck, they lose immediately — **unless** the bottom card of your deck has been transformed into a **Victory Card** (official glossary, 2026-09-10): in that case, drawing from an empty deck **wins** the match instead of losing it. (The default bottom card is the Reaper Card.) Surrendering also causes an immediate loss.

**Simultaneous leader lethal (official Q&A).** When a single instruction deals damage to both leaders (e.g. "deal X damage to both leaders"), both leaders take that damage before the outcome is judged. If both leaders are at 0 or less after that judgement, the **active player loses** (their opponent wins). Separate instructions are still judged one after another — if the enemy leader reaches 0 from an earlier clause, the match ends before later clauses can damage your leader.

- _Balto, Dusk Bounty Hunter (`10153140`):_ Crest at end of your turn with both leaders at 1 — your opponent wins.
- _Aragavy, Eternal Hunter (`10154130`):_ Evolve with both leaders at 3 — your opponent wins.
- _Rage of Serpents (`10153310`):_ Both leaders at 2, enemy leader selected — you win (enemy reaches 0 first; the self-damage clause does not run).

## Zones and Card States

Cards move through distinct zones during play:

- **Deck:** face-down and randomized; order and contents are hidden to both players. Cards are drawn or searched from here. Drawing from an empty deck causes a loss.
- **Hand:** cards drawn but not yet played, hidden from the opponent. The hand limit is 9 — any card drawn beyond this is destroyed (burned to the cemetery as a shadow). _Test case:_ a player holding 9 cards draws one; the drawn card is destroyed and converted to a shadow rather than added to hand. The engine must never allow a 10th card in hand. **Owner ruling (2026-08-10):** a card destroyed by hand overflow triggers **no** Last Words — "converted into a shadow" beats "destroyed."
- **Field (play area):** up to 5 cards per player. Followers and amulets in play occupy the field. If an effect would put a new follower into a full field, the summon simply fails for the excess units (they do not enter and do not queue). _Test case:_ a "summon 2 followers" spell cast with only 1 empty slot fills the one slot and skips the second summon, with no error.
- **Cemetery (graveyard) and shadows:** destroyed followers/amulets and fully-played spells leave to the cemetery and become shadows. A shadow is a counter representing a destroyed or spent card; no card is returned from the graveyard except via special mechanics (e.g. Reanimate, which copies a card without removing it from the cemetery). Each player's shadow count is public and is consumed by certain effects (see Necromancy). Banished or transformed cards do not go to the cemetery and produce no shadows.
- **Banish zone:** a banished card is removed from play entirely, with no death triggers and no shadow. Banished cards are out of the match for good (equivalent to exile in other games).
- **Tokens / outside the game:** there is no sideboard. Token cards generated by effects may be put into play or hand; they are defined by card text but are not part of the deck list.

## Targeting Rules

Many effects require selecting targets. A card or ability can be played only if all its targets are available and valid at the moment of activation — for example, you cannot cast "deal 3 damage to an enemy follower" while the opponent has no followers. If an effect's text does not use the word "target" (e.g. "destroy a random enemy follower", "give all allies +1/+1"), it requires no specific selection and may be played regardless of board state. In practice this targeting restriction applies almost exclusively to spells: you can nearly always play followers and amulets, and activate Engage abilities, even with no valid targets available. When you do select a target, it must meet the stated criteria (ally/enemy, follower/amulet/leader, etc.). Some abilities add restrictions — an Aura follower cannot be selected by enemy targeting, and an Ambush/Intimidate follower cannot be attacked — such followers are simply not valid targets for those actions. Cards that cannot be targeted say so explicitly (see Aura).

**Owner ruling — "Select" is forced (2026-08-16):**

> "In this game if it says 'select' then it's forced. You can't play him [Field Scientist] without discarding unless your hand is empty."

A "Select …" clause is mandatory: while a legal target exists, the selection cannot be declined or cancelled. The only exception is when no legal target exists (empty hand / empty board), in which case the card still plays (for followers/amulets — see below) and the selection clause fizzles. In this codebase the `optional: true` key on hand-selection cards means "fizzle gracefully on empty zone", **not** "player may decline."

**Owner ruling — A printed "Select" opens a target prompt (2026-09-08):**

> "Both say select so yes you should be able to chooose ofc"

When printed card text says _Select_, the player must be offered a target prompt — the engine must not silently pick the leftmost legal candidate. This is distinct from the forced-selection rule above (you cannot decline once the prompt opens); it governs whether a prompt opens at all. Affected cards at ruling time: Titania, Queen of Fairies (`10214110`, evolve), Ara, Dawnblossom (`10534120`, evolve), Sincerity of the Dewdrop (`10573310`, spell). The engine already excludes the source card from a targeted pool, so Ara's _"another follower"_ needs no `exclude_self` — that key is load-bearing only where there is no targeted pool (`10553110` Lifestealer, `distribution: "all"`).

**Owner ruling — Playability with no "Select" target (2026-08-16):**

> "If it's something like select on the field but there is no target on the field, then you can still play amulets and followers without the effect. Spells tho — if it says select on the field you can't play the spell if there is no target. Radiant Rainbow sometimes is such a dead card…"

A **spell** with a mandatory "Select …" clause requires at least one legal target to be playable at all — no target, no play (no Hearthstone-style burning mana on nothing). **Followers and amulets** with the same kind of clause stay playable; the selection clause simply fizzles. Accelerate plays are gated like spells (they resolve as spells). Radiant Rainbow (`10131310`) is the canonical dead-card case when no Spellboost card is in hand.

**Hand selections (official Q&A, 2026-09-06):** The same rule applies when a spell must **select from your hand** — every mandatory hand-selection step needs enough legal candidates (the card being played is never a candidate for its own hand selection, and `select: N` needs N candidates). Spilling Red (`10642310`) cannot be played unless you can both discard a different hand card and destroy an enemy follower; Doomwright Resurgence (`10172320`) cannot be played without two Artifact followers in hand that cost 5 or less. See `docs/official-qa.md` for the published answers.

Once an ability is activated with chosen targets, those targets are locked in; there is no interrupt or response window between selection and resolution. If a target becomes invalid mid-resolution (e.g. destroyed by an earlier part of a multi-step effect), the action on that target simply does nothing while the rest of the effect resolves on its other targets. Spells never "fizzle" entirely — each instruction does as much as it can. For example, "destroy an enemy follower, then draw a card": if the follower is already gone when the spell resolves, the destroy does nothing but the player still draws. _Official Q&A (Supplicant of Destruction):_ "If you selected one, destroy it and …" clauses run the post-selection effect when a target was **selected**, even if the destroy was prevented (e.g. "Can't be destroyed by abilities") — selection satisfies the "if you selected one" branch, not a successful destruction.

## Randomness

Random effects ("deal damage to a random enemy", "summon a random follower from your deck") are resolved by the engine's pseudorandom generator. For determinism this should be a seeded RNG, so the same seed and game state reproduce the same outcomes. Unless stated otherwise, random selection is uniform over the valid options, and each independent random instruction uses a fresh draw. Multiple random selections in one effect resolve sequentially. _Test case:_ "destroy two random enemy followers" against 3 followers — pick one at random and remove it, then pick from the remaining two; never pick the same follower twice, and never pick both at once without accounting for the first removal. Likewise, "deal 3 damage split randomly among enemies" would be three separate random 1-damage instances, each re-evaluating the board.

## Hidden Information

Each player's hand contents and deck order are hidden from the opponent. Hand size is public (the UI shows the opponent's card count), and deck count is shown as well — so decking-out the opponent is plannable, and deck count must be accessible for win-condition checks. Card identities in hand and deck are private unless an effect reveals them; "reveal" typically means both players see the card, and revealed hand cards return to hidden afterward unless continuously revealed. The cemetery exposes only the shadow count, not distinct cards, so its contents are effectively hidden. Both players can consult the Battle Log, which lists actions taken (cards played, deaths, etc.).

**Owner ruling — Hidden information always visible (2026-08-13):** In this practice tool the player is playing against themselves, so card text that says "without revealing them/it" does **not** hide anything. Exact copies added to hand (and any effect that would read the opponent's hand or deck under a reveal restriction) remain fully visible to the player, and effects may freely inspect both hands and both decks. Do **not** build a hidden-copy variant or gate this on the sparring hidden-hand UI toggle — the owner judged a bot/scripted-opponent hidden mode not worth the complexity. Cards unblocked by this ruling include Goddess of Starlight, Wolfraud, Skybound Hanged Man, Legacy of the Brave, and Behemoth General.

## Time and Priority

There are no real-time prompts for opponent interaction during your turn; all effects resolve fully in trigger order. The game is governed by an action queue (see Turn Structure and Timing Windows) rather than a stack — once an action is initiated it resolves to completion, including all triggered sub-effects, before the next player action. The only "interruptions" are automatic triggered effects, which the engine handles between player actions.

## Canonical Terminology and Zones

This section defines the fundamental terms and zones of SVWB, using official rule language where possible.

### Zones and Card Locations

- **Deck:** a 40-card deck constructed before the match and randomized at start. Players draw from their own deck; order and contents are hidden. Follow any search/reveal effect's text precisely. Drawing from an empty deck causes a loss.
- **Hand:** the cards a player has drawn but not played. Maximum size is 9; a card drawn beyond the limit is immediately destroyed (sent to the cemetery with no effect). The opponent sees your hand count but not the cards. You may generally play any card in hand given enough PP and valid targets.
- **Field / play area:** where followers and amulets reside in play, 5 slots per player. Extras beyond the slot limit are destroyed with no effect. Left-to-right position can matter for ordering (e.g. some simultaneous-trigger resolutions; see the entry-order rule under Timing Windows). A played follower/amulet occupies an open slot; with all 5 filled, further followers/amulets cannot be played (a play requiring a slot is illegal, while a multi-summon spell resolves partially). Removing a card from play (destroy, banish, bounce, etc.) frees its slot.
- **Cemetery (graveyard) and shadows:** the game keeps no visible list of specific graveyard cards. Instead, each time a card leaves play or hand by being destroyed or spent, it produces a shadow for its owner. A follower destroyed (defense 0) or an amulet destroyed (countdown 0) becomes a shadow; a spell becomes a shadow after it resolves; a discarded card (by effect or hand limit) also produces a shadow. Banished or transformed cards produce no shadow. Each player has their own shadow count, used mainly by Abysscraft (Necromancy). The cemetery is not directly interactable except via mechanics such as Reanimate, which summons copies of followers that died.
- **Banished zone:** a banished card is removed from the game entirely. Banishing can affect cards in play, hand, or deck depending on the effect. Banished cards do not count as destroyed, do not enter the cemetery, and produce no shadows or Last Words — they simply cease to exist in the match (equivalent to exile).
- **Leader (character) and leader area:** the leader represents the player (an avatar with a defense value). A leader is not a card and cannot be removed; only its defense changes. The leader area holds Crest/Faith icons — persistent abilities or statuses attached to the leader (similar to enchantments on the player), up to 5 active icons. Crests are gained via card effects (e.g. a card granting "Crest of Glory"), are not cards, occupy no field slots, and cannot be removed except by effects that say so. Rules-wise, a Crest is a continuous effect tied to the leader (see Continuous Effects). **Faith icons share the same five-slot cap but do not count toward "the number of crests you have"** (official Q&A for Shining Disenchantment `10363210`: _"Do faiths count as crests?"_ → _"No, they don't."_; owner ruling 2026-09-06).

**Owner ruling — Crest and Faith slots are capped at five (2026-09-05):** The crest/faith area holds at most five active icons, and **Faith counts toward the five**. A sixth distinct crest or faith gain is a silent no-op — the crest never exists, so neither its Countdown nor its Last Words ever apply. _Worked example:_ Maddening Benison (`10263310`) at five crests is a drawback-free 10 heal — the restore resolves, the crest gain is ignored, and the self-damage Last Words never triggers.

**Owner ruling — A duplicate crest bounces off the active one (2026-09-05):** Re-gaining a crest you already have does nothing — it does not replace the instance and does not reset its Countdown. A crest at Countdown 1, re-gained, stays at 1 with exactly one instance. This differs from original Shadowverse, whose default replaces the instance.

### Card Types and Terminology

- **Leader:** the player's character avatar with a defense value (starting at 20). Not a card, but effects can reference "your leader" (to restore defense or deal damage). Leaders can gain abilities via crests/faith (e.g. "your leader has: at end of turn, draw a card"). At 0 defense, that player loses.
- **Follower:** a unit card that becomes a creature on the field with Attack and Defense stats. Attack is the damage it deals; Defense is its health. At 0 or below defense it is destroyed. Followers enter play with summoning sickness and cannot attack the turn they are played unless they have Rush or Storm (see Keywords). Each follower normally attacks once per turn. Followers may also have a class, traits, and various keyword or text abilities.
- **Amulet:** a card that occupies a field slot and provides a continuous effect or delayed trigger. Amulets have no Attack or Defense and cannot attack or be attacked. Many have a Countdown — a number that decreases at the start of the owner's turns and, at 0, destroys the amulet (usually firing a Last Words). Others have no countdown and persist until removed. Amulets may have Engage (activated) abilities; see Keywords. Followers and amulets share the 5-slot field. A destroyed amulet leaves play to the cemetery (unless banished) and triggers its Last Words. (Some crests also carry Last Words that fire on their destruction.)
- **Spell:** a one-time-effect card. You pay its cost, resolve its text, then it goes to the cemetery (creating a shadow). Spells occupy no field slots and usually require valid targets if any. After resolution a spell is no longer active.
- **Artifact and other traits:** some cards (especially Portalcraft) carry traits such as Artifact, or Officer for Swordcraft. A trait is a sub-classification that matters for deck-building and certain effects but does not change base rules.
- **Token:** a card not in the original deck, generated by effects. Tokens may be followers, amulets, or spells and behave like normal cards once created. Last Words and spells often "summon a [token]" into the field or hand as instructed. **Owner ruling — Tokens live in set "Basic A" (90000) (2026-08-29):** "the tokens are all in the set basic A for god knows what reason. you need cursor to get them from there. they are all starting with numbers with 9000" — Token cards are published under set **[90000] Basic A** with ids beginning `9000…`; any token a card references must be ingested from there (e.g. Dread Pirate's Flag `90021210`, Warden of the Trigger).
- **Allied vs enemy:** always from the perspective of the effect's controller. "Destroy an allied follower" means one currently under your control; a follower you have taken control of counts as allied to you while you control it.
- **Other (as in "other followers"):** excludes the card itself. "Give all other allied followers +1/+0" does not buff the source.
- **This follower / itself:** "this follower" or "it" always refers to the card bearing the text.
- **Attack / Defense (stats):** Attack (ATK) is combat damage dealt; Defense (HP) is health. Unmodified, a follower enters with its base (printed or evolved) stats. Damage persists as reduced defense — there is no end-of-turn restoration; a follower holds its current defense until restored or further damaged, and is destroyed at 0 or below. Restoration cannot raise a follower above its current maximum (the highest it has been set to via base or buffs); leaders likewise cannot restore above their maximum (default 20, or higher if increased).
- **Evolve (verb):** to spend an Evolution Point to transform a follower into its evolved form, typically granting +2/+2 and possibly an Evolve ability. **Super-Evolve** is a stronger version using a Super Evolution Point (SEP), granting +3/+3 and extra perks. Details under Turn Structure and Keywords.

---

## Turn Structure and Timing Windows

SVWB has a structured turn sequence with specific sub-steps. The game uses no interrupt-driven stack; it processes actions in discrete timing windows. This section covers the turn phases, resource rules, and the timing rules for triggers and actions.

### Turn Phases

Each turn breaks into the following phases. (Not all are labeled in-game, but they occur in order. For the exact, queue-accurate step order of the boundaries, see Start-of-Turn and End-of-Turn Sequences below.)

**Start of turn:**

- **Max PP increase:** on turns 1–10 the active player gains +1 maximum PP orb (cap 10). Beyond turn 10 max PP stays at 10 unless an effect raises it. (The second player's Bonus PP can push _usable_ PP to 11 for one turn — shown as 11/10 — but max PP remains 10.)
- **PP refill:** all PP orbs refill; available PP resets to max each turn.
- **Draw step:** draw 1 card. (Unlike original Shadowverse, where the second player drew 2 on turn 1, both players draw only 1 on their first turn in WB.) Drawing from an empty deck loses the game immediately.
- **Countdown reduction:** every allied amulet with Countdown loses 1, essentially simultaneously with the draw. Any amulet reaching 0 is destroyed immediately, queuing its Last Words (if any) to resolve before the main phase.
- **Start-of-turn triggers:** all "at the start of your turn" abilities (on cards or crests) now activate and resolve in priority order. They are checked simultaneously at the very first tick; if a trigger's condition is not met at that tick, it will not fire that turn even if the condition becomes true later during the start sequence.

After all automatic start effects resolve, play proceeds to the main phase.

**Main phase (action phase):**

The active player may take any number of actions in any order — play cards, attack, evolve, use abilities — subject to these constraints:

- **Playing cards:** pay the card's PP cost (you need PP ≥ cost), which is deducted from your available PP. The effect then occurs: followers/amulets enter the field (followers summoning-sick; amulets activating any Fanfare/continuous effects), and spells resolve their text then go to the cemetery. You may keep playing cards while you have PP.
- **Attacking:** ready followers may attack, once per turn by default (**Attack**, official glossary, 2026-09-10: starting from the turn **after** an allied follower enters the field, you may attack with it once per turn; when attacking an enemy follower, the attacker takes damage equal to the defender's attack). A follower played this turn cannot attack unless it has Rush or Storm, and a follower that has already attacked cannot attack again unless an effect refreshes it (rare "can attack twice" effects). To attack, pick a ready follower and a valid target:
  - _Attacking an enemy follower:_ the target must not have Ambush or Intimidate, and if the opponent controls any Ward followers you must target one of them. A follower played this turn without Storm cannot attack at all; with Rush it may attack an enemy follower but not the leader.
  - _Attacking the enemy leader:_ no enemy Wards may be present, and the attacker must have Storm (or have been in play since a previous turn). Rush does not permit a leader attack on the turn the follower was played.
  - Declaring an attack runs the full combat sequence (see Combat Procedure Recap); the main phase pauses to resolve it — all attack triggers and damage — then resumes.
- **Evolving:** from a turn threshold, the active player may spend an Evolution Point (EP) to evolve a follower. Eligibility is by the player's own turn count, not the round number, so the first player is effectively one turn behind:
  - _Normal evolve:_ first player turn 5+, second player turn 4+.
  - _Super-evolve:_ first player turn 7+, second player turn 6+.
  - The follower must currently be unevolved (no follower evolves twice). Pay one EP (or one SEP for super-evolution) and the evolve resolves (see Evolve / Super-Evolve under Continuous Effects). You may evolve any time in your main phase, including immediately after playing a follower — and doing so lets that follower attack enemy followers this turn (evolving lifts summoning sickness for follower attacks only, like Rush), but not the leader unless it has Storm. **Engine note:** this attack permission is modeled via the evolved flag (`hasEvolved`), not by granting the Rush keyword — per official Q&A (Achim `10272120`: evolved copies inherit "the same abilities given to followers evolved using EP"; Olivia `10104110`: super-evolve bundles are enumerated separately from keywords). Each player may evolve at most 4 times per game (2 normal + 2 super), at most once per follower. **A player may evolve a follower only once per turn, of either kind** — spending an EP or an SEP that turn makes both `evolve` and `evolve {super}` illegal until that player's next turn. Effect-granted evolves do not consume the turn's evolution. (old engine `canEvolve` per-turn limit; owner confirmation pending 2026-09-10). Evolving costs no PP — it uses the EP/SEP resource.
- **Activated (Engage) abilities:** some field cards — especially amulets with an [Engage] ability — can be manually activated in the main phase, normally once per turn. Pay any activation cost, follow the usual targeting rules, and the ability queues and resolves immediately. Unlike a freshly played follower, an Engage ability may be used the same turn its amulet is played (no summoning sickness unless stated), but only once per turn even if you have spare PP.
- **Bonus PP and other actions:** the second player has two Bonus PP in WB, activated via a button (not a card) for an immediate temporary +1 PP that turn. One may be used on any turn before their 6th and one on any turn from their 6th onward; the pre-turn-6 charge is use-it-or-lose-it (if unused before turn 6, only the later charge remains). **Bonus Play Point (official glossary, 2026-09-10):** the button refreshes at the **start of your sixth turn** if you already used the first charge, so you may use Bonus PP up to **twice per match**. Using a Bonus PP is a main-phase action and costs no card or EP.

The main phase has no action limit beyond resources; the player continues until done or out of options, then ends the turn.

**End of turn:**

On End Turn, every "at the end of your turn" / "at the end of this turn" effect fires and one-turn buffs wear off. All end-of-turn triggers are treated as simultaneous at the moment of turn end and resolved in priority order (see the sequence below). End-of-turn conditions are checked once, at turn end: if a card's requirement isn't satisfied then (e.g. "at end of turn, if this follower attacked this turn, draw a card" when it did not attack), it gets no trigger. After all end-of-turn effects resolve, the turn passes to the opponent's start-of-turn phase.

**Priority and the action queue:** with no interactive stack, the active player always holds priority during their main phase. The opponent cannot interrupt; it only reacts through automatic triggers (Ambush blocking selection, or an amulet "whenever your opponent plays a card, do X"), which the engine resolves between player actions. Simultaneous triggers resolve in the fixed deterministic order below, active side before reactive side.

**Passing and inaction:** a player who cannot or will not act ends their turn. Online play has a per-turn timer (a ~90-second bar) that auto-ends the turn on expiry; a simulation engine can ignore the timer but should guard against infinite loops.

### Play Point (PP) and Evolution Point Rules

- **Play Points (PP):** the resource for playing cards. Players begin turn 1 at 0/0. Each turn max PP rises by 1 (cap 10) and PP refills to max. Effects can also grant max PP (e.g. a Dragoncraft "gain 1 max play point" can reach 10 before turn 10); **Overflow** is the state of having at least 7 max PP. Gaining max PP outside the normal turn progression does not auto-fill those orbs unless the effect says so. "Recover"/"refill" fills existing empty orbs; "gain" increases the number of max orbs. Unused PP does not carry over — it is wasted at end of turn.
- **Bonus PP (second-player advantage):** two opportunities, via a button — one usable before the second player's 6th turn and one from their 6th turn on, each once; the button **refreshes at the start of your sixth turn** if the first charge was spent (official glossary, 2026-09-10). A Bonus PP does not raise max PP; it is one extra filled orb for that turn (comparable to the "coin" in other card games) and is gone after use. At 10 max PP it can temporarily push usable PP to 11. **Accounting:** usable PP = regular orbs (≤ max) + bonus orb (0 or 1). Paying costs spends regular orbs first; the bonus orb is spent only when regular orbs are exhausted. Recovery ("recover", "fully recover", turn refill) refills regular orbs up to max; an unspent bonus orb remains on top (cap max + 1 for that turn). _Official Q&A:_ at 9 max with bonus active, after a 3-cost play (7 PP left), Baby Carbuncle Super-Evolve "Recover 3" yields **10** PP (9 regular + 1 bonus). After bonus then Kuon Enhance (10) (0 PP left), Dimension Climb "Fully recover" yields **9** PP (bonus orb was spent with the Enhance payment).
- **Evolution Points (EP):** each player has 2 (down from 3 for the second player in original Shadowverse, since both now have equal EP and the second player instead gets Bonus PP). EP are not replenished. They are granted at match start but locked until the eligible turn (turn 4 for the second player, turn 5 for the first), shown grayed-out until then. _Test case:_ before the threshold turn the evolve action is disallowed even though EP exist; on the threshold turn it becomes enabled.
- **Super-Evolution Points (SEP):** new to WB, each player has 2 (purple pips, separate from EP's yellow). Like EP they are finite and never refreshed, usable from turn 6 (second player) or turn 7 (first). Super-evolution is a distinct action; when both EP and SEP are available and conditions are met, you choose which to spend. A follower can be in an evolved state only once: you cannot evolve an already-evolved or super-evolved follower, and you cannot upgrade an evolved follower into a super-evolved one. **Owner 2026-09-10:** _"a card cannot evolve multiple times: only once. also it cant normal evolve and then super evolve later."_ Official glossary (Evolution): _"An evolved follower can't be evolved again."_ _Test case:_ using SEP on a follower already evolved with EP is rejected; `legal_actions` offers no super-evolve. An effect that auto-evolves a follower does not consume your EP but does set its evolved state (so it cannot then be evolved manually); an effect-evolve targeting an already-evolved follower is a **no-op** (no stats, no EP/SEP, no evolve abilities).
- **Active vs reactive player:** track whose turn it is. The active player makes the plays, attacks, and evolves; the non-active player's cards act only through automatic triggers (e.g. an amulet "when your opponent plays a spell, deal 1 damage to their leader" fires during the opponent's main phase at the moment of the play). The engine handles those as they occur.

## Timing Windows and Trigger Resolution

Certain events cause triggered abilities to activate. To stay deterministic, SVWB defines a strict order for resolving simultaneous triggers.

**No stack, but an event queue.** When an event occurs (a follower attacks, a follower is destroyed, the turn ends, etc.), every ability that triggers in response goes into a queue, and the game resolves them one at a time. New triggers raised while another ability is resolving are appended to the **end** of the queue and do not interrupt the ability currently resolving (Japanese effect-processing spec, おんJシャドビヨ部 「シャドバWBの仕様」, confirmed 2026-09-06; see also owner-rulings.md).

**Trigger resolution order by source.** When multiple effects share the same trigger timing, they resolve in this fixed order (active side before reactive side; within a side, hand effects before leader before board):

1. In-hand effects of the active player (rare — e.g. a hand card "whenever an allied follower is destroyed, reduce this card's cost").
2. In-hand effects of the non-active player.
3. Leader (Crest) effects of the active player.
4. In-play card effects of the active player (their field followers/amulets).
5. Leader (Crest) effects of the non-active player.
6. In-play card effects of the non-active player.
7. Invocation / deck effects of the active player.
8. Invocation / deck effects of the non-active player.

**Tie-breaks within a category.** For multiple cards on the field triggering in the same category (e.g. two allied followers both with "whenever an enemy follower is destroyed, do X"), the tie-break is the order the cards entered play (oldest first). Newly summoned followers/amulets always enter to the right of existing ones, so oldest-first equals left-to-right in practice — but the canonical rule the engine should track is entry order, which stays correct even if board positions ever shift. This entry-order tie-break governs all same-side, same-timing ties (simultaneous end-of-turn effects, simultaneous Last Words, etc.). If a single card has multiple triggers on the same event, they resolve in the order printed on the card, top to bottom — e.g. a follower with "Strike: gain +1 attack" written above "Strike: Drain" applies the +1 before the Drain. (This is the same rule that orders a follower's Strike and Clash effects.)

Once all queued triggers from an event are handled, the game returns to normal flow. Triggers often cause further events (damage, destruction) that queue still more triggers; the engine keeps resolving until the queue is empty, then proceeds.

**Owner ruling — evolve reactions before the evolving follower's Evolve list (2026-09-10):** _"evolve comes first so the faith ticks up first."_ Crests and Faith that react to an allied evolve resolve before that follower's printed `Evolve:` / `Super-Evolve:` list, so a Faith increment is visible at the Evolve ability's choice node. No official Cygames Q&A exists for this ordering.

**Owner ruling — Deaths settle before dependent triggers pick targets (2026-08-23):**

> "playing fangs of ardent destruction should kill the 2/1 at the same time as damaging galmieux. galmieux should then deal 3 dmg to the 5/2 ... but currently the 5/1 ward keeps standing. so either galmieux 3 damage didnt trigger or it targeted the corpse of the 2/1."

A follower reduced to 0 or less defense is dead and must never be a legal target for any effect that resolves afterwards — including effects triggered inside the same damage batch (an AoE that kills one follower and damages another). Settle deaths before dependent triggers choose targets.

### Start-of-Turn and End-of-Turn Sequences

Both turn boundaries use a **two-phase** model: first, conditions are checked and triggered abilities are **queued** in a fixed step order; then the queue is **resolved** in that same order. Within a single step, crests that trigger at the same time resolve in the order they were granted (oldest first) and board abilities resolve in entry order (oldest first). If an ability is lost at the same step another ability triggers (e.g. via a Countdown reaching 0), losing it does **not** cancel the other ability.

**Start of your turn:**

1. Max PP increases and PP refills.
2. Your "at the start of your turn" crests trigger; crest Countdowns advance; any crest reaching Countdown 0 is lost and its Last Words is queued. (Anything these trigger is queued.)
3. Your board's "at the start of your turn" abilities trigger; your amulets' Countdowns advance; any amulet reaching Countdown 0 has its Last Words queued.
4. The opponent's "at the start of your opponent's turn" crests trigger.
5. The opponent's board's "at the start of your opponent's turn" abilities trigger.
6. Direct summons occur; any "when directly summoned" abilities are queued.
7. Resolve the queued abilities in the same order (the step-2 queue, then step-3, then 4, 5, 6…). Each resolution may queue further triggers, worked through in self-then-opponent order.
8. Draw 1 card for the turn. The player may now act.

**End of your turn** (after End Turn is pressed; the player can no longer act):

1. Your "at the end of your turn" crests trigger.
2. Your board's "at the end of your turn" abilities trigger.
3. The opponent's "at the end of your opponent's turn" crests trigger.
4. The opponent's board's "at the end of your opponent's turn" abilities trigger.
5. Direct summons occur; "when directly summoned" abilities are queued.
6. Resolve the queue in the same order (step 1, then 2, 3, 4, 5…), working through any further triggers self-then-opponent.
7. The turn ends: "until the end of the turn" abilities on **both** leaders wear off.

An ability worded just "at the end of the turn" — with no "your"/"your opponent's" — triggers on **both** players' turn ends.

**Owner ruling — Turn-boundary triggers are owner-scoped (2026-08-31):** A turn-boundary trigger printed _"at the start/end of your turn"_ fires only on its owner's boundary. `whose_turn: "opponent"` is the explicit opt-out for cards printed _"at the end of your opponent's turn"_ (e.g. Lilanthim `10734110`). Absence of that key must **not** mean "fire for everyone" — that defect made a mirror match add a Fairy to both hands on your own turn start.

_End-of-turn example._ Player A ends the turn with two followers reading "at the end of your turn, deal 1 damage to the enemy leader," while Player B has a card reading "at the end of your opponent's turn, gain 1 defense." All three trigger at once. Resolution: A's older follower deals 1, A's newer follower deals 1, then B's leader effect restores 1 — a net 2−1 = 1 damage to B's leader (active in-play effects before non-active).

_Worked example (the canonical simultaneous case)._ You hold _Crest: Grimnir_, have a super-evolved Celes on board, and the opponent has an evolved 2-defense Funikar & Yavnhar plus a Castle Artifact (granted Icarus's Flight) made by Artifact Catapult. You end your turn. All three end-of-turn abilities trigger at once and queue as **your crest → your follower → opponent's follower**: (1) Grimnir, (2) Celes, (3) Castle Artifact. Resolving (1) deals 2 to all enemy followers, destroying Funikar & Yavnhar, which queues its Last Words (4). Resolving (2) gives Celes a Barrier. Resolving (3) destroys Castle Artifact, queuing the Icarus's-Flight Last Words (5). Resolving (4) would hit Celes, but Celes already has its Barrier (since (2) resolved earlier), so it pops harmlessly. Resolving (5) draws the opponent a card. Queue empty → the turn passes.

_Lethal mid-sequence._ Leader defense is checked continuously: the first time a leader reaches 0 the match ends immediately. The official rules don't specify whether remaining queued triggers cancel, but in practice it rarely matters.

**State-based condition checking.** Conditions for triggered abilities are evaluated at the moment the trigger is created, not at resolution — per the official FAQ, "whether effects activate or not at any particular timing are checked all at once." For example, if an effect requires 8 followers to have left play by end of turn and the 8th leaves at end of turn via another effect, it will not count, because the check was already determined before that destruction resolved. Implementation: snapshot the relevant conditions when the event occurs and do not re-evaluate mid-resolution.

### Combat Timing Specifics

An attack is a multi-trigger event. The detailed damage/destruction rules live under Combat Procedure Recap; here is the trigger order:

1. The attack is declared and a target chosen.
2. **Strike/Clash triggers** fire before damage, in this order:
   - the attacker's own Strike and Clash effects (if it has both, in card-text order — e.g. "Strike: draw a card", "Clash: deal 2 damage to the enemy follower before battle");
   - the defending follower's Clash effects, if any;
   - "whenever an allied follower attacks" effects for the attacker's side — leader/crest first, then the attacker's in-play card effects;
   - "whenever an enemy follower attacks" effects for the defending side — leader/crest first, then that side's other cards.
     These are all triggered abilities queued before the damage step.
3. **Combat damage step:** the attacker deals damage equal to its attack to the target; if the target is a follower, that follower simultaneously deals its attack back. If the target is the leader, only the leader takes damage (leaders do not strike back). Damage is marked on each.
4. **Post-damage destruction:** followers at 0 or less defense are destroyed simultaneously. A follower with a "whenever a follower is destroyed" ability that dies in the same event does not see its own death (its ability is gone); only survivors trigger.
5. **Super-evolution knockback:** if the attacking follower is super-evolved and this combat destroyed its combat target (via a Strike/Clash effect or via combat damage), the enemy leader takes 1 damage now — before that destroyed follower's Last Words resolve (see Super-Evolution).
6. **Last Words / death triggers:** all destroyed followers' Last Words fire, treated as in-play card effects of their controller. If both combatants died, the active player's Last Words resolves first, then the opponent's; multiple deaths on one side resolve in entry order (oldest first).
7. The combat ends and the main phase resumes.

**Owner ruling — "Attacked a leader last turn" counts the attack, not the damage (2026-08-29):** For cards reading _"If an allied follower attacked a leader on your last turn …"_: **"Attack committed counts."** An allied follower satisfies the condition the moment it commits an attack on the enemy leader. Damage is irrelevant — a swing that deals 0 still counts. A Ward-blocked attack does **not** count, because it never reaches the leader at all. Unlocks: Ripper-Clawed Thief, High-Spirited Marauder, Barren-Earth Tyrant, Artiglio, Antemaria.

_Combat test case._ The active player attacks with a follower that has "Strike: deal 1 damage to the enemy leader" and Bane, targeting an enemy follower with "Clash: deal 2 damage to the attacker." Order: the attacker's Strike deals 1 to the leader; the defender's Clash deals 2 to the attacker; combat damage is exchanged; because the attacker has Bane, the defender is destroyed after damage regardless of its remaining defense. If both die (the attacker was already low and took lethal counter-damage), their Last Words trigger simultaneously, active player first. This verifies that Strike resolves before damage, Clash resolves in the right place, Bane applies even if the attacker's attack was reduced to 0 (Bane cares only that damage was dealt, 0 or more), and Last Words favor the active player.

### Fanfare and Enter-Play Trigger Order

When a card is played and/or a follower/amulet enters play, the resulting triggers resolve in this order:

1. **Play reactions** — abilities printed "whenever you play …" (`ally_card_played` / `ally_follower_played`, crests included) are evaluated and resolved **when the card is played**, **before** the played card's own Fanfare or spell text, in the usual crest-then-board order (active crest, active board, opponent crest, opponent board). Official Cygames Q&A, World of Games (`10503210`): _"The only card on my field is a World of Games with a count of 5, and the only enemy card on the field is a Quake Goliath. If I play Divine Thunder, what will World of Games's count be?"_ → _"Its count will be 4."_ Divine Thunder (`10103310`, cost 4) destroys the 4-cost Goliath; the count still advances, so the play reaction resolved before the spell's text. Owner 2026-09-10: _"WoG says 'whenever you play' so it should die on play and make space."_ The same Q&A (and owner _"yes any card"_) is why World of Games counts a same-base-cost card on **either** side.
2. The played card's own **enter** ability (if any) sits above Fanfare.
3. The played card's own **Fanfare** / spell body, and any effects it raises (summons, damage, etc.) in FIFO order with their own triggered abilities. Last Words caused by a play reaction (World of Games dying on play) resolve in this window — still before those summons.
4. Crests that react to **a card entering** ("whenever … enters the field", e.g. Krull) — after the play sequence, including across a Fanfare choice.
5. Board abilities that react to a card entering (e.g. Orchis, Adahime).

Same-timing abilities resolve self → opponent; same-timing crests resolve in the order they were granted. Enter reactions raised by the play stay on the queue until the Fanfare / spell body finishes; play reactions do not wait.

Two enter-play rulings:

- **A keyword granted by Fanfare/enter is not present at the instant of entering.** A follower given Ward by its _own_ Fanfare does **not** satisfy "when a Ward follower enters play" — it had no Ward at the moment it entered.
- **Overflow and once-per-turn:** "when this enters play" abilities do **not** react to cards that overflowed the board (and so never actually entered). A "once per turn, when X enters play" ability reacts only to the **first** of several simultaneous entries.

**Owner ruling — `named_enter_count` / "other" copies and enter-route timing (2026-08-30):** Cards gated on how many **other** allied copies have entered this match must not count the entering card itself when the gate is evaluated on an enter-trigger route. Obsessed Test Subject (`10931110`) — _"if at least 5 other allied copies … have entered"_ — buffs starting at the **6th** copy. The Fanfare route (Drache & Aluzard `10844110`) records the enter _after_ Fanfare and is already correct; a global "exclude self" would break it. Check the route, not just the key.

**E38 (pending owner, 2026-09-10):** an entrant's own "When this card enters the field" (`on:enter`) is a same-timing board enter trigger, ordered with the others by board age — "board abilities resolve in entry order (oldest first)". Item 1 above is play reactions (E39), which do resolve when the card is played, before Fanfare / spell text. Item 2 ("the played card's own enter sits above Fanfare") is the reading E38 does not follow: own `on:enter` waits with other enter reactions until the play completes (E34) and sorts among them by board age, not a jump onto `pending_work` above Fanfare. Example: Aizeden (`10974120`) already on the field, play Analyzing Artifact (`90071130`) — Aizeden's destroy (older) before the Artifact's draw (newest).

---

## Effect Types and Resolution Model

Card effects come from playing cards (a spell's text, a follower's Fanfare, etc.), from triggered abilities (Last Words, Strike, etc.), or from continuous abilities that modify the rules. This section covers how those effects resolve, plus targeting, failure cases, and randomization.

### Playing Cards from Hand

Playing a card follows a fixed sequence:

1. **Pay costs.** The PP cost (or an Enhance cost) is deducted from available PP; if you cannot pay, the play is illegal. Any additional cost (discarding a card, consuming shadows via Necromancy) must also be payable — if not, you cannot take that option (optional cost) or play the card at all (mandatory cost).
2. **Declare targets.** If the card requires choosing targets ("select an enemy follower…"), pick the valid target(s) now; the game highlights legal choices. If no valid target exists, the card generally cannot be played — SVWB blocks the play rather than letting it fizzle. Effects needing several targets require all of them as distinct choices. Randomly-targeted effects ("deal 4 damage to a random enemy follower") need no selection — the engine rolls later — and may be cast even with one, many, or zero valid targets (with zero, the effect simply does nothing).
3. **The card enters play / the effect begins:**
   - _Follower:_ moved from hand to an empty slot and now in play (summoning-sick). Its Fanfare, if any, triggers immediately, and any aura/continuous effect (e.g. "allies have +1 attack") becomes active at once. There is no stack — the Fanfare is an event trigger.
   - _Amulet:_ placed in an empty slot; any Fanfare triggers now (resolving like a follower's), continuous effects begin immediately, Countdown is set as printed, and an Engage ability becomes usable this same turn.
   - _Spell:_ its text executes; it is never "in play," passing through a temporary resolving state straight to the cemetery. Choose/alternate modes prompt before resolution.
4. **Effects resolve:**
   - _Fanfare:_ a triggered ability queued the moment the card enters; in practice it executes right away (nothing else is usually queued). Chosen targets and any RNG are applied now.
   - _Spell text:_ executes step by step, in written order ("Then…", successive sentences), with each conditional ("if X, do Y") checked at the moment it is reached.
   - When a step destroys or damages several entities at once, those destruction/damage events are marked but their triggers wait until the card's effect fully resolves (kill-then-draw still destroys first, but the death triggers wait). _Example:_ "destroy all followers, then deal 2 damage to both leaders" destroys every follower at once, continues to deal the 2 to each leader, and only after the spell finishes do the Last Words fire — card effects resolve atomically, and their triggers resolve afterward in priority order.
   - **A card's own text resolves strictly top-to-bottom**, and the entire started sequence finishes before any triggered abilities it caused resolve. This ordering can decide games:
     - _Deal 3 to the enemy, then 2 to your own leader_ — the enemy is hit first, so this can win at equal life totals (lethal lands before the self-damage could lose you the game).
     - _Add cards A, B, C to your hand_ with a nearly full hand — they are added in text order, so the ones that fit are added first and the later card(s) burn on hand overflow.
     - _Draw 2, then the opponent draws 1_ while you are near deck-out — your draws happen first, so you can deck out and lose before the opponent's draw clause runs.
5. **Aftermath.** The card reaches its final zone: followers/amulets stay in play, spells go to the cemetery (a shadow). A follower/amulet that an effect removes during its own resolution (e.g. a Fanfare that destroys itself) goes to the cemetery after resolving. The engine then processes any resulting triggers (a Fanfare draw, a spell's Last Words), and once the queue is empty the active player resumes.

**Partial resolution, no retargeting, and last-known information.** Because nothing interrupts resolution, a target usually stays valid throughout. If a target is gone before a step that references it, that step simply does nothing — the effect does not retarget to something else and is not cancelled; it skips what it cannot do. Spells never fizzle on a missing target and cannot be countered, so a multi-target effect still affects its remaining valid targets. When an effect needs a value from a card that has left play ("destroy a follower, then deal damage to the enemy leader equal to that follower's attack"), use the card's last-known value (its attack at the moment of destruction). Counts such as "the number of allied followers that died this turn" are read at the moment the effect/trigger resolves.

### Targeting and Selection Effects

Card text specifies the target set (enemy follower, enemy leader, all allies, a random enemy, a card in hand, etc.); the engine enforces it:

- **Enemy / allied:** by controller at the time the effect resolves.
- **Type:** follower, amulet, or leader only, as stated. **Owner ruling — "A card on the field" includes amulets (2026-09-09):** when a target spec is `:any` (any card on the field), amulets are legal candidates — narrowing comes from the target's own kind, not from a hard-coded follower-only pool. Sincerity of the Dewdrop (`10573310`) prints _"Select a card on the field and transform it into an Imari's Little Buddies"_ with `any:any`; the only card in the pool whose board-route transform targets `:any` at ruling time.
- **Trait or condition:** e.g. "the enemy follower with the highest attack" (ties broken randomly) or "a random enemy follower" — compute the valid set, then pick.
- **Random selection:** uniform among valid candidates; with none, that portion does nothing.

Untargetability and combat restrictions:

- **Aura** (cannot be selected by the enemy): excluded from opponent selection effects. If an effect must select an enemy follower and the only one has Aura, there is no valid target and the card cannot be played (a follower/amulet may still be played, but its Fanfare fizzles). Random and global effects still hit Aura followers — "deal 3 to all followers" or "deal 3 to a random follower" is not the opponent _selecting_ it. Aura only blocks explicit enemy selection.
- **Ambush** (stealth): cannot be targeted or attacked by the enemy until it loses Ambush by dealing damage (declaring an attack). While Ambushed it is excluded from opponent targeted effects and from being an attack target.
- **Intimidate:** cannot be attacked by enemy followers, but _can_ be targeted by spells and abilities.
- **Ward:** restricts attack targeting only — you cannot attack any enemy that is not a Ward while a Ward is in play. It does not redirect spells or effects.

Aura and Intimidate are permanent unless removed; Ambush lasts only until the follower attacks. These can combine: an Ambush (or Intimidate) follower that also has Ward does not function as a taunt, because it cannot be attacked anyway; once Ambush is lost (or Intimidate is removed), the Ward becomes active again.

**Multi-target effects resolve partially.** "Return all allied followers to your hand" with 4 followers but room for only 3: the first three (oldest first) return to hand, and the fourth, which cannot (hand limit is 9), is converted to a shadow instead. The engine should mirror this for any move-to-hand effect — overflow cards are destroyed — and for summoning beyond the field limit, where extra summons simply fail (a spell that would summon past 5 summons as many as fit and ignores the rest). **Draw / Add to Hand / Return to Hand (official glossary, 2026-09-10):** if your hand is already full (9 cards), a Draw, Add to Hand, or Return to Hand adds a **shadow** to your cemetery instead of putting the card in hand. **Summon (official glossary, 2026-09-10):** a Summon creates a copy on the field or moves a card from hand/deck to the field; **if your field is full, nothing happens**.

### Resolution Order and Queue

Effects do not use a stack; triggers queue. While a card or ability effect resolves, the engine suspends other actions until it finishes; triggers it causes wait until it is done (unless the effect explicitly acts after a delay). After the effect completes, the queue resolves by priority, with no player input, until empty.

_Complex example._ Player A casts "Meteor": "destroy a random enemy follower; deal 3 damage to all enemies," against 3 enemy followers. Meteor randomly destroys one (queuing its Last Words, which wait), then deals 3 to all enemies — the destroyed follower is already gone, so the leader and the other two take 3 — which may drop more followers to 0 (marked destroyed). Meteor goes to the cemetery, and only then do the death triggers resolve, simultaneously, in priority order (active vs non-active). Any followers summoned or further damage dealt by those triggers are handled in turn, then control returns to the main phase.

Edge cases:

- An Ambush follower destroyed by a global effect without ever attacking still fires its Last Words — Ambush affects targeting, not death triggers.
- If control of a follower changes permanently, it is now on the new controller's side; if it later dies, its Last Words resolve for that controller.
- A simultaneous "destroy all followers" destroys everything together; Last Words resolve active-side first (entry order), then the opponent's. A surviving follower with "whenever a follower is destroyed, …" triggers once per death (5 deaths → 5 triggers), but a follower that dies in the same event does not count its own death. _Test case:_ a follower with "whenever an enemy follower is destroyed, draw a card" while you destroy 2 enemy followers at once triggers twice (drawing 2) — the engine must not collapse the two into one or skip one.

## Random Targeting and RNG in Effects

A recap of random handling:

- Use a seeded pseudorandom generator so a given seed and identical play sequence reproduce identical outcomes.
- Each random decision is independent unless stated. "Randomly deal damage 3 times" is three independent draws with replacement (the same target can be hit repeatedly) unless targets are removed between instances.
- "Different random followers" requires distinct picks: choose one, then another from the remaining pool. If the first random kill removes a candidate for the second pick, the second picks from the survivors.
- No random effect may resolve onto something invalid by the time it executes — if there were 3 candidates and one died to an earlier trigger, the pick is among the remaining 2.
- **"a random … with the highest base cost"** (e.g. Initiation of Rebirth): compute the highest base cost among matching destroyed-this-match candidates, then pick **uniformly at random among those tied at that cost**. Owner ruling 2026-09-02: _"it says random highest cost so that means if some are tied pick randomly between them."_ Printed **random** selects among the tied set; see Card-specific rulings.

**Owner ruling — "N random followers" = distinct; "do this N times" = repeats allowed (2026-08-23):**

> "if the text says 2 random followers it can do max 8 to 1 follower ... if it says 8 to A random follower TWICE it can hit the same target twice."

- **"deal X damage to N random enemy followers"** → N **distinct** followers, each hit once (surplus picks do nothing when fewer than N candidates remain).
- **"Do this N times: deal X damage to a random enemy follower"** → N independent rolls; the same follower can be hit repeatedly (Oluon's ruling generalised; Barren-Earth Tyrant's "do it 2 times instead" is this shape).

In summary, every play and activation produces either an immediate action or queued triggered actions, processed in a well-defined order without interruption; ties are broken by the priority ordering above, yielding deterministic outcomes for identical inputs.

---

## Keywords and Mechanic Catalogue

This section enumerates the keyword abilities and major mechanics of SVWB. Each entry gives a definition, its timing, key interactions, and where useful an example or test case.

### Evergreen Keywords (General)

**Fanfare.** Activates when a follower or amulet is _played from hand_, resolving immediately after the card enters play and before any other action (spells use their own text instead). A follower/amulet put into play by another effect — summoned rather than played — does **not** trigger its Fanfare (e.g. "summon a copy of follower X" does not fire the copy's Fanfare). _Example:_ Little Dragon Nanny — "Fanfare: summon a Fire Drake Whelp." _Test case:_ "Fanfare: deal 2 damage to an enemy follower" with one enemy follower deals 2 on play (and may kill it before you can attack); with no enemy follower the card is still playable and the Fanfare simply fizzles.

**Last Words.** Activates when the card is destroyed (play → cemetery); common on followers, amulets, and crests. It does **not** fire on banish or transform. It triggers at the moment of destruction and queues to resolve after the current effect finishes; simultaneous deaths produce simultaneous Last Words, ordered by the priority rules. A follower silenced or stripped of text before dying has no Last Words. A Last Words that summons a follower places it after the death resolves. _Example:_ Leah, Bellringer Angel — "Last Words: draw a card." _Test case:_ two trading followers each with "Last Words: deal 1 to the enemy leader" resolve active-player-first; a banished follower's Last Words does not fire.

**Strike.** Activates when the follower _attacks_, before combat damage. Subtypes: _Follower Strike_ (only when attacking followers) and _Leader Strike_ (only the leader); a plain Strike fires on any attack. It does not trigger on defense or on non-attack damage, and a Strike trigger queued at declaration still resolves even if the follower is removed mid-attack. _Example:_ "Strike: gain +1 attack" buffs before dealing combat damage. _Test case:_ "Strike: deal 2 to the enemy leader" while attacking a follower deals the 2 first, ending the game immediately if it brings the leader to 0.

**Clash.** Activates when the follower battles another _follower_, as attacker or defender, before damage — never when attacking a leader. If both combatants have Clash, the attacker's resolves first, then the defender's; multiple Clash effects on one card follow its text order. A defender's Clash is queued at declaration and still resolves even if the attacker's Clash kills it first (a queued trigger resolves independently). _Test case:_ attacker "Clash: deal 3 to the follower it's attacking" versus defender "Clash: deal 2 to the attacker" — the attacker's 3 may kill the defender, yet the defender's queued Clash still deals its 2.

**Bane.** If a follower with Bane deals combat damage to a follower (any amount, even 0), that follower is destroyed after damage. It works on both offense and defense, and the Bane follower need not survive or deal nonzero damage. The destruction happens in the combat-damage step, before post-combat triggers, and fires the victim's Last Words. Barrier does **not** save a follower from Bane (the battle still occurred). Bane does not kill leaders and does not apply to non-combat (effect) damage. _Test case:_ a Bane follower trading into a 10/10 destroys it even if the Bane follower also dies; a Bane follower whose damage Barrier reduces to 0 still destroys the defender.

**Drain.** When a follower with Drain deals attack damage to an enemy leader or follower, restore your leader by that amount (life steal). It applies only when the Drain follower is the _attacker_ — counter-damage while defending restores nothing — and only on combat damage, not effect damage. Restoration is immediate and capped at the leader's maximum. _Test case:_ (1) attacking the leader for 5 restores 5 (never above max); (2) the same follower dealing 5 back as a defender restores nothing; (3) its Clash or other effect damage restores nothing.

**Ward.** The opponent must attack your Ward followers before any non-Ward target — they cannot attack your leader or your other followers while a Ward is in play. Multiple Wards must all be cleared first (the attacker chooses which Ward to hit). Ward is a continuous combat rule checked at attack-target selection; it never affects spells or abilities, and it applies symmetrically (your followers likewise cannot attack a warded opponent's leader). _Note:_ Lloyd is modeled in the engine as Ward plus an unofficial "Taunt" (spell/ability taunt) so that nothing but Lloyd can be targeted or attacked. _Official Q&A (Cleric of Crushing vs Lloyd):_ Lloyd's forced targeting applies only when Lloyd is itself a legal target of the ability — an unevolved Lloyd does not block selecting a super-evolved enemy follower for an ability that requires super-evolution. _Test case:_ with one Ward and one non-Ward, the opponent cannot attack the non-Ward; with two Wards they may attack either Ward but nothing beyond.

**Storm.** The follower can attack the enemy leader (or followers) the turn it is played — no summoning sickness, still subject to Ward. Storm supersedes Rush (a follower with both can attack the leader). Evolving a Storm follower keeps Storm, and a follower that gains Storm mid-turn can immediately attack the leader if it hasn't yet attacked. _Test case:_ a Storm follower attacks the leader immediately; a non-Storm follower cannot attack the leader that turn even after evolving (evolving grants only follower attacks, like Rush — the engine does **not** set the Rush keyword on evolve; see Achim `10272120` / Olivia `10104110` Q&A).

**Rush.** The follower can attack enemy _followers_ the turn it is played but not the leader — a weaker Storm. From the next turn it attacks normally. Granting it Storm later in the turn lets it hit the leader; evolving does **not** upgrade Rush to Storm (it still cannot hit the leader without Storm) and does **not** grant the Rush keyword (only the same follower-only attack permission). _Test case:_ a freshly played Rush follower cannot attack the leader but can attack a follower; next turn it can attack the leader.

**Ambush (stealth).** The follower cannot be targeted by enemy spells/effects or attacked by enemy followers until it loses Ambush by attacking or dealing damage. It is granted on entry and lost the moment it attacks (and does not return unless re-granted). Taking AoE/area damage does **not** break Ambush, and random/global effects still affect it — Ambush, like Aura, only blocks explicit enemy selection and attacks. While Ambushed, any Ward it has is moot (it cannot be attacked anyway). _Test case:_ a single-target spell cannot select an Ambush follower; "deal 2 to all enemies" still hits it and it remains Ambushed; once it attacks the leader it loses Ambush and becomes targetable.

**Aura.** The card cannot be _selected_ by the opponent's effects (the untargetable half of Ambush) but can still be attacked. Always on while active; random/AoE effects still affect it. Aura + Intimidate together equal a permanent Ambush (untargetable and unattackable, but not lost on attacking). Aura + Ward: the Ward still works, but the opponent may attack it. _Test case:_ "select and destroy an enemy follower" cannot pick an Aura follower, but you can attack it, and "deal 3 to a random enemy follower" can still hit it.

**Barrier.** A one-time shield that reduces the next instance of damage to 0, then is removed. It does not stop destroy, banish, or stat reduction — only damage (combat or effect) — does not stack, and does not save from Bane. Against an AoE it absorbs that single instance (taking 0) and breaks while other targets take normal damage. It can be granted to a leader too. _Test case:_ with Barrier, "deal 3 damage" deals 0 and the Barrier breaks; a second such spell hits normally; a Barrier follower in combat takes 0 from the enemy's hit, breaks, and still deals its own damage back; a leader with Barrier takes 0 from the next attack and breaks.

**Evolve (ability).** A triggered ability that fires right after the follower evolves (with an EP or SEP): the follower first gains its stat boost (+2/+2, or +3/+3 for super) and then the Evolve text resolves. If a follower has both an Evolve and a Super-Evolve ability, a normal evolve fires only the Evolve line while a super-evolve fires **both simultaneously** — **except** when the Super-Evolve line says "instead" (a replacement), in which case only the Super-Evolve line fires and it does not stack with the Evolve line (e.g. Arriet's super restores 4, not 2+4). _Data note:_ when `evolve[]` and `superevolve[]` are identical in the card JSON (the effect duplicated for tooling), empty the `superevolve[]` array so a super-evolve resolves the effect once (e.g. Leah draws 1 on super, not 2). _Test case:_ "Evolve: deal 3 to the enemy leader" deals 3 on evolving; a follower with both lines fires only Evolve on a normal evolve and both on a super-evolve, unless the Super-Evolve says "instead" (Arriet → restore 4 only).

**Super-Evolve (ability).** Fires when the follower is super-evolved (with a SEP). If both Evolve and Super-Evolve lines exist, both fire on super **unless** the Super-Evolve line contains "instead" (replacement, not stacking). _Example:_ Garyu, Fabled Dragonkin — "Super-Evolve: give all allied Supreme Golden Dragons Storm and all allied Supreme Silver Dragons Barrier." _Test case:_ "Super-Evolve: draw 2 cards" draws nothing on a normal evolve and 2 on a super-evolve; with an added "Evolve: deal 2 to an enemy," a super-evolve does both.

### Class/Mechanic-Specific Keywords

**Necromancy (X)** (Abysscraft). "If you have at least X shadows, spend X of them to activate this effect." It happens only with enough shadows: if you have them the cost is paid automatically and the effect occurs, otherwise it is skipped entirely. It is checked when the card is played, or when the Necromancy-bearing effect would trigger (some Last Words carry Necromancy). Multiple Necromancy effects firing at once draw from the same pool in sequence, an unpayable cost is skipped, and spending is automatic — you cannot choose to save shadows. _Example:_ Devious Lesser Mummy — "Fanfare: Necromancy (4): give this follower Storm." _Test case:_ with 4 shadows it consumes 4 and gains Storm; with 3 it enters normally without Storm.

**Reanimate (X)** (Abysscraft). Summon a copy of one of your highest-cost allied followers that died this match with cost ≤ X (ties at the highest qualifying cost broken randomly, weighted by how many copies of that follower were destroyed — the candidate list is cemetery/destroyed *instances* at that cost, not distinct names). The copy is given the **Departed** trait for the rest of its life (the tribe is on the instance; a later printed copy of the same card does not have it). (official glossary, 2026-09-10): "Reanimate summons a copy of the allied follower with the highest base cost destroyed that match and gives it the Departed trait. … The more copies of a follower have been destroyed, the more likely it is to be chosen." The summon is immediate; the copy is a token (no Fanfare) and enters exhausted, unable to attack unless it has Storm/Rush. With no qualifying death, nothing happens; the original stays in the cemetery (still a shadow) and Reanimate consumes no shadows. _Test case:_ Reanimate (4) with a dead 4-cost and 3-cost summons the 4-cost (random among tied 4-costs), summoning-sick, no Fanfare.

**Owner ruling — Reanimate provenance (2026-09-02):** Reanimate only sees followers that were **destroyed on the field**. Owner, clarifying a live bug where Wills United reanimated a fused Lyria: _"she was never even on the field. not even discarded cards can get reanimated. so its extra preposterous that a fused card got renanimated."_ Discarded, burned, or otherwise non-destruction cemetery entries are invisible to Reanimate. Shadows from discards are unchanged — this is eligibility, not the shadow count.

**Combo** (mainly Forestcraft). Your Combo is the number of cards you have played **this turn** (official glossary, 2026-09-10). Some abilities require Combo to be at or above a threshold; **the card being played counts toward Combo** for its own threshold check.

**Rally (N)** (mainly Swordcraft). Counts the followers that have entered play on your side this match (played or summoned, tokens included). At or above N the Rally ability is active — either a continuous "if Rally ≥ N…" or a one-time threshold trigger. The counter is cumulative and never decreases when followers leave. A "Fanfare: if Rally (N)…" does **not** count the card's own entry; it sees the count from just before it entered. _Test case:_ with 4 followers played, a "Fanfare: if Rally (5), draw 2" card played as the 5th does not draw (it sees 4); a later such card does. A continuous "Rally 10: at end of turn, deal 2 to the enemy leader" fires each end of turn once the count reaches 10.

**Owner ruling — Rally (2026-08-12):** Rally increments if and only if a follower **successfully enters your field**. The route is irrelevant — played from hand, summoned by an effect, created as a token, reanimated, put into play by Last Words, Invoked from deck, or any other successful follower entry. Conversely, anything that does **not** put a follower on the field does not count, and a summon that **fails** (e.g. because the field is already full — excess summons are skipped) must **not** increment Rally. Consequences: (1) a Crystallize play puts an **amulet** on the field and an Accelerate play resolves as a **spell**, so neither increments Rally by itself — but a follower summoned _by_ that alternate form's text, or by a Crystallize amulet's Last Words, does; (2) a control-change that moves an already-in-play follower onto your field is **not** a summon and does **not** increment Rally (secondary reading of this ruling — confirm if a printed take-control card ever appears); (3) the existing Fanfare Rally timing rule still stands — that is about _when_ the count is read (before the card's own entry), not about which route produced the entry.

**Overflow** (Dragoncraft). The state of having at least 7 max PP; once reached it remains true for the rest of the match. "Overflow: do X" is either a continuous bonus or a Fanfare check at play time. It turns on the instant max PP reaches 7, including mid-turn via ramp (a "gain 1 max PP" spell at 6 → 7 enables Overflow immediately). The second player's Bonus PP does **not** count toward Overflow — using it at 6 max PP gives 7/6 usable but does not enable Overflow. _Test case:_ at 6 max PP a "Fanfare: if Overflow, deal 3" does nothing; at 7 it deals 3; ramping 6 → 7 mid-turn enables it at once.

**Spellboost** (Runecraft). Whenever you play a spell, every Spellboost card in your hand applies its Spellboost effect (commonly "cost −1," sometimes "+1 damage," etc.), automatically and with no response window. Only cards in hand at the moment of casting are boosted — cards drawn by that same spell are not, and Spellboost cards generated after the cast missed it. Spellboost reduces _current_ cost (recomputed dynamically), not the printed cost; the engine can simply track a boost counter. _Example:_ Magic Missile boosts other Spellboost cards (e.g. Zeus's Magic) by −1 each. _Test case:_ two "Spellboost: −1 cost" cards at cost 5 drop to 4 after one spell, 3 after the next, and so on. (Some followers also "Spellboost your hand.")

**"Whose cost has been changed"** compares the card's **effective play cost at the moment of playing** with its **printed base cost** — net change, not any intermediate modifier. _Official Q&A (Institute of Truth):_ a Blaze Destroyer reduced to 9 then increased back to 10 by an enemy Whitefrost Whisper does **not** trigger (net equals printed 10); a 0-cost Blaze does; a Quake Goliath raised to 5 does. A card played via Accelerate or Crystallize takes that form's base cost for both sides of the comparison, so it does not count as changed.

**Earth Rite (Earth Sigils & Stack)** (Runecraft). Written as **Earth Rite (N)**: it activates only if the number of Earth Sigils you control is N or greater, and activating it consumes N of them. _Example:_ Earth Rite (3) needs at least 3 Earth Sigils and spends 3; with fewer than N, the Earth Rite portion simply does not happen (the rest of the card still resolves).

- **Earth Sigils are a counted resource, not one-per-amulet.** They are tracked as a _Stack_ counter on an Earth Sigil amulet (e.g. Magic Sediment, Witch's New Brew). **Official glossary, 2026-09-10:** when an Earth Sigil amulet enters the field its sigil count is set to **1**; any other allied Earth Sigil amulets on the field are **banished** (no shadow) and their counts are added to the new amulet. Earth Sigil amulets **cannot be destroyed by abilities** and **cannot be selected for enemy abilities**; they are removed when their sigil count reaches 0. "Gain X earth sigils" adds X to an allied Earth Sigil holder on the field, or **summons a Magic Sediment with sigil count X** if none is present. Sigils merge additively onto a single amulet rather than each taking a board slot (3 on the stack + a card granting 2 = 5).
- **Merge survivor (owner ruling, 2026-08-30):** When Earth Sigils merge, a **collectible** Earth Sigil amulet (from the card pool, e.g. Witch's New Brew) always survives over a **token** Earth Sigil (magically generated, e.g. Magic Sediment). _Example:_ Magic Sediment on the field with stack 2, then you **play** Witch's New Brew — the Sediment is destroyed and replaced by the Brew carrying stack 3 (2 transferred + 1 from the Brew). If Brew is already on the field and an effect **generates** Magic Sediment, no second amulet appears; the Brew's counter simply increases by 1. _(See **Conflicts for the owner** in `rules/official-glossary-audit.md` for Brew-onto-Brew vs the glossary's "new amulet banishes the old ones" wording.)_
- **Consumption happens first**, before the rest of the Earth Rite effect resolves. This matters on a full board: with exactly 1 sigil and a full field, an _Earth Rite (1): summon a Golem_ spends the sigil (removing the now-empty amulet) before summoning, so the Golem still has a slot to enter.
- **Order of play:** sigil-producers and Earth-Rite payoff cards are different cards, so the normal flow is to bank sigils early and spend them mid-to-late game; deck balance between the two matters.
- _Test case:_ with an Earth Sigil stack of 2, play a follower with "Fanfare: Earth Rite (2): summon a 5/5" — it detects 2 sigils, consumes 2, and summons; with 0 or 1 sigils the summon does not happen.

**Enhance (N).** A card may be played at a higher cost, N, to activate additional effects. The printed (original) cost is unchanged for cost-reduction purposes; if you have at least N PP and play the card, the game automatically spends up to N and applies the enhanced effect with no prompt, while paying the base cost ignores the Enhance text. _Test case:_ with 10 PP, Waters of the Orca (base 2, Enhance 10) spends 10 and uses Enhance(10); with 9 or less it spends 2 for the base effect. A cost-increase effect raises the base (e.g. +2 makes it 4) but leaves the Enhance cost at 10; "original cost" / "base cost" references still read the printed base.

**Crystallize (N).** _(Apocalypse Pact onward.)_ An ability that lets you play the card **as an amulet** when you cannot afford its normal play cost. Trigger condition: remaining PP is **strictly below** the card's current (effective) cost to play normally, **and** remaining PP is **at least** the Crystallize cost N. You then pay N, the card enters the field as an amulet carrying the Crystallize ability text (commonly Countdown + Last Words), and the follower form does **not** enter play — no follower Fanfare, no follower keywords from the printed follower side. The Crystallize cost N is a fixed alternate cost (like Enhance): cost-reduction / cost-increase that changes the card's effective play cost does **not** change N. If the card has more than one Crystallize and/or Accelerate ability, the **highest payable** alternate activates (see below). **Original / base cost:** ~~playing via Crystallize does **not** rewrite the card's printed (original) cost to N — "base cost" / "original cost" references stay at the printed follower cost.~~ **Settled 2026-09-06** — the played card takes base cost **N** from that play on (including if bounced to hand). Official Q&A: Zerael, Sundered Rebirth (`10904110`) — accelerated Jailor of Antiquity base cost is 1. _Sources:_ Cygames Apocalypse Pact JP announcement — 「結晶」を持つカードは、自分の残りPPがコスト未満で、「結晶」のコスト以上なら、結晶アミュレットとしてプレイできます; English glossary (secondary corroboration; that page's Clash tooltip was demonstrably wrong, so treat English wording as support, not sole authority). _Test case:_ Venerating Dyer (follower cost 4, Crystallize 1) with 1–3 PP pays 1 and enters as a Countdown (3) amulet with base cost 1; with 4+ PP the normal follower play is used instead.

**Owner ruling — Crystallize / Accelerate alternate-form permanence (2026-09-02):** Crystallize followers played as amulets are only treated as amulets **while on the field** in the sense that hand/deck effects still see the printed follower — quoting the owner-supplied source: _"Crystallize followers played as amulets are only treated as such while on the field. Any effect that would interact with amulets in the hand or deck will not affect Crystallize. Creating/Returning an amulet created by a Crystallize follower to the hand will not return it to being a follower and it will remain an amulet."_ Once played in the Crystallize form, the instance **stays an amulet permanently** (including if returned to hand). The same permanence applies to Accelerate: an Accelerate-played follower stays a **spell** in the cemetery and is **not** a legal Reanimate target. General rule: printed type in hand/deck; alternate form forever after that play. Do not "fix" Reanimate skipping an Accelerate corpse — that is the ruling.

**Accelerate (N).** _(Apocalypse Pact onward.)_ An ability that lets you play the card **as a spell** for a lower cost when you cannot afford its normal play cost. Trigger condition: remaining PP is **strictly below** the card's current (effective) cost to play normally, **and** remaining PP is **at least** the Accelerate cost N. You then pay N; the card resolves as a spell (its Accelerate skill text), goes to the cemetery as a spell would, and generates a shadow — the follower form does **not** enter play. Accelerate cost N is likewise a fixed alternate cost (not modified by the card's cost\*mod / Spellboost reductions). If several Accelerate / Crystallize abilities are present, the **highest payable** one activates. **Original / base cost:** ~~playing via Accelerate does **not** replace the card's printed (original) cost with N.~~ **Settled 2026-09-06** — the played card is a spell with base cost **N** from that play on (graveyard, ladder, and "played base cost X" checks). Official Q&A: Zerael, Sundered Rebirth (`10904110`) — accelerated Jailor of Antiquity base cost is 1. Any follower **summoned by** the Accelerate text is a normal copy of the printed card and reports that printed base cost (not the Accelerate value). _Sources:_ same Cygames JP announcement (アクセラレートスペルとしてプレイ); English glossary (secondary). _Test case:_ Shoddy Plaything (follower cost 6, Accelerate 2) with 2–5 PP pays 2, resolves the Accelerate summon text as a spell, and does not place the follower from hand onto the board; with 6+ PP the normal follower play is preferred. The summoned Shoddy Plaything still has base cost **6**.

**Owner ruling — Accelerate and Spellboost / hand-deck type (2026-09-02):** Quoting the owner-supplied source: _"Accelerate will function properly with Spellboost and other mechanics that interact with spells. However, Accelerate followers will not be affected by cards that would interact with spells while they are in hand or deck, such as Wizardess of Oz."_ So: an Accelerate play **does** Spellboost the hand (and fires other spell-play mechanics); while still in hand or deck the card keeps its **printed** type and is not a spell for type filters. Reachable via Neutral **Jailor of Antiquity** (`10901110`) in a Runecraft Spellboost deck — an earlier claim that no legal deck could contain both Accelerate and Spellboost was wrong.

**Owner ruling — Accelerate / Crystallize original cost (2026-08-12):** ~~In **Shadowverse Worlds Beyond**, playing a card via Accelerate (or Crystallize) does **not** change its original/base cost to the alternate-form value N. The printed cost remains the base cost for every "base cost of X or more / original cost" check.~~ **Superseded 2026-09-06** — see **Accelerate / Crystallize — the played card takes the alternate form's base cost (2026-09-06)** in `docs/owner-rulings.md`. Official Q&A: Zerael, Sundered Rebirth (`10904110`) — accelerated Jailor of Antiquity base cost is 1. The observation here still stands: Evolution Portalcraft with **Shoddy Plaything** (`10671110`, base 6 / Accelerate 2: "Summon a Shoddy Plaything") plus **Yog-Zentha, Eld Axe** / **Advent of the Eld Axe** / **Unfeeling Eld Axe** — the Accelerate **summon** is a base-cost-6 body for 2 PP and therefore turns on the craft's "allied follower with a base cost of 5 or more" package. The 2018 Cygames tweet about original Shadowverse ("when played via Accelerate the original cost becomes the Accelerate value") was right for Worlds Beyond too.

**Highest-payable alternate (Crystallize / Accelerate).** When a card lists more than one Crystallize and/or Accelerate ability, and remaining PP is below the normal effective cost, the game auto-selects the alternate whose cost is the **highest value you can currently pay** (no prompt). This is the inverse shape of Enhance (which picks the highest _higher_ cost you can afford): here the alternates are _lower_ costs gated by "not enough for normal." _Test case:_ with two Accelerate lines at 2 and 4 and 3 PP (below normal cost), Accelerate (2) activates; at 4–(normal−1) PP, Accelerate (4) activates.

**Owner ruling — multi-tier Enhance (2026-08-15):** The parenthetical above ("which picks the highest higher cost you can afford") reads as though only one Enhance tier applies. The owner ruled otherwise: _"If you play Noel for 8 play points, both his Enhance 8 and Enhance 7 abilities will activate."_ — _"And his fanfare also activates no matter what - if played from hand ofc."_ Recorded with that exchange: **Noel IV, Ruthless Warlord** (`10624110`) played from hand at 8 PP summons **three** Fearless Soldiers — Fanfare Bane, Enhance(7) Drain, and Enhance(8) Storm. (A copy summoned by an effect triggers neither Fanfare nor Enhance.) The same ruling states the general rule this refactor implements: _"Enhance never suppresses Fanfare unless a card explicitly says so."_ This engine's `pickEnhanceTiers` returns every affordable tier (cost ≤ PP paid); do not change it to keep only the highest.

**Owner ruling — Alternate and Enhance costs are fixed (2026-09-02):** Cost reduction or increase changes the card's own effective play cost only — it **never** changes the printed **Accelerate**, **Crystallize**, or **Enhance** value N. The alternate-form gate compares available PP against the card's _effective_ cost while reading the alternate's **printed** N, so reducing a follower's cost makes Accelerate/Crystallize **harder** to reach, not cheaper (_e.g._ Shoddy Plaything base 6 / Accelerate 2 at 5 PP plays via Accelerate; reduce its cost to 4 and the same 5 PP plays the printed follower). Enhance is never declinable: L'Age d'Or (printed 4 / Enhance 6) reduced to 1 plays for 1 at 1–5 PP and jumps to Enhance 6 at 6+ PP — the two outcomes are decided by PP, not player choice. See `docs/owner-rulings.md` — **Alternate and Enhance costs are fixed; only the card's own cost moves — 2026-09-02**.

**Normal play preferred.** If remaining PP is enough for the card's normal effective cost, you play the printed type (follower / spell / amulet) at that cost — Crystallize / Accelerate do **not** activate even if their costs are also payable. A full board that blocks a permanent does **not** by itself unlock Accelerate: the PP-below-normal gate must still hold. Accelerate (spell) does not need a board slot; Crystallize (amulet) does.

**Owner questions (sources leave these open — do not invent a ruling):**

1. ~~**Rally:**~~ **Settled** — see **Owner ruling — Rally (2026-08-12)** under Rally. Alternate-form plays do not increment Rally; summons from those texts do.
2. ~~**Spellboost:**~~ **Settled** — see **Owner ruling — Accelerate and Spellboost / hand-deck type (2026-09-02)** above. Accelerate plays do Spellboost; printed type still governs in hand/deck. (An earlier "unreachable" note was wrong — Jailor of Antiquity is Neutral.)
3. ~~**Cost modification of N:**~~ **Settled 2026-09-02** — see **Owner ruling — Alternate and Enhance costs are fixed (2026-09-02)** above and `docs/owner-rulings.md`. Alternate cost N is fixed; only the card's own effective play cost moves.
4. ~~**Original cost under Accelerate / Crystallize:**~~ **Settled 2026-09-06** — see **Accelerate / Crystallize — the played card takes the alternate form's base cost (2026-09-06)** in `docs/owner-rulings.md` and official Q&A Zerael (`10904110`). The played card takes base cost N; summoned bodies keep printed base cost.

**Choose.** Some cards let you pick between two or more options when played; you choose the form at play time, and some options carry added costs.

**Fuse (hand fusion).** Some cards may be fused with other cards in hand. Fuse is a main-phase hand action, not a play.

**Owner ruling — Fuse mechanics, Sephie and Ecstatic Scholar (2026-08-29):**

> "sephie and ecstatic scholar do not need any recipes. … you can fuse (as many cards as you want to ONE sephie ONCE per turn (if you have 2 sephies in hand ofc you can fuse to both of them ONCE) but you need 2 playpoints available for it to do something."

- **No recipe list** — any card may be fused into a Fuse card that accepts unrestricted partners.
- **Once per turn per instance** — with two Sephies in hand you may fuse once to each in the same turn.
- **Fusing is always legal, even at 0 PP.** Owner clarification: _"if you fuse with less than 2 then you just fuse and you dont pay anything and nothing spawns. its important that you CAN fuse even if you dont have playpoints"_. The fused cards always leave hand and the "was fused" flag is always set. Sephie's **on-fuse effect** alone is conditional: with **≥2 PP available, exactly 2 PP are spent** and an Obsessed Test Subject is summoned; with fewer, nothing is spent and nothing spawns. A 0-PP fuse still consumes that Sephie's once-per-turn fuse and still satisfies Ecstatic Scholar's "was fused" requirement.
- **"Was fused while in hand" is a persistent per-card flag** (not a recipe). Ecstatic Scholar's Super-Evolve grants its extra effect (select a Test Subject, give it Drain) **only if she was fused at least once while in hand**; otherwise super-evolution gives only the normal +3/+3. The same flag drives Garden's Allure and Returning Slash. The flag must survive the hand→field transition and belong inside the snapshotted state.
- **Fused partners are banished (2026-09-02).** Owner, after Wills United reanimated a fused Lyria: _"fused cards are banished- gone -fairy dust-fugazi."_ Consumed fuse partners go to the **banish zone** (no shadow, not cemetery). This clarifies the 2026-08-29 ruling that only said fused cards are _"gone from hand"_ — it does not reverse it.

**Owner ruling — Artifact fuse chain (2026-09-05), refined by official Q&A (2026-09-06), partners corrected 2026-09-10:** Gears fuse with any Artifact amulet (printed "Fuse: Artifact amulets"; the 2026-09-05 "gears only" recollection is superseded); the gear fused _into_ decides the body (Ambition → Striker, Remembrance → Fortifier). Striker/Fortifier host any Artifact cards and transform by partners' total cost (1 → Ominous α, 2 → β, 3+ → γ). **Ominous Artifact α** accepts **Ominous Artifact β** and **Ominous Artifact γ** only; each fused partner is banished. Fusing only one kind does not transform α immediately, but α **remembers** which kinds have been fused. When **both** β and γ have been fused to the same α instance — in one fuse action or across turns — it transforms into **Masterwork Artifact Ω**. Already-fused kinds cannot be selected again. β, γ, and Ω cannot initiate fuse. Official Q&A (`90073110`): fusing β one turn and γ to the same α the next turn **does** produce Ω.

**Crests (Countdown / Last Words / end of turn).** Crests use the same timing machinery as amulets. A Countdown (N) on a crest ticks −1 at the start of its owner's turn (the same window as amulet countdowns), and the crest is destroyed at 0. Last Words on a crest fire when it is destroyed (including by countdown reaching 0). "At the end of your turn" on a crest fires at that owner's end of turn (e.g. Corruption: deal 2 to your leader each end of turn while active). _Example Last Words:_ Belial's crest — Countdown (4), Last Words: deal 20 to the enemy leader when it reaches 0.

**Sequence abilities.** When a card activates abilities "in sequence" from a numbered list, each trigger runs the next step in order; after the last step, the sequence **wraps** back to the first step on the next trigger (official Q&A: Omerio, Winged Revenant — after the third ability, the next allied amulet destruction fires step 1 again).

**Owner ruling — Slaus unused-ability pool (PROVISIONAL, 2026-08-13):** For _Slaus, Revolving Wheel of Fortune_ (body and Crest), "activate a random ability that hasn't been activated yet" draws **without replacement** from the printed three options. The unused pool is **not** replenished: at most three activations occur; on a fourth start-of-turn (if the body somehow survived, or if the Crest's Countdown (3) were somehow still present), **nothing happens**. The Crest's Countdown (3) already caps Crest activations at three in normal play. **PROVISIONAL** — the owner flagged uncertainty: the Crest countdown makes the three-cap clear in practice, but the body effect is hard to observe in a real match because followers rarely survive three turns in a row. A future in-game observation may overturn this; keep the provisional mark until confirmed.

**Invoke.** (official glossary, 2026-09-10): "An ability that summons a card from your deck if there's space on your field. If several cards are invoked at once, the order in which they enter the field will be randomized. Only one copy of each card can be invoked at once, but the likelihood a card is invoked increases with the number of copies in your deck." If the field is full, Invoke does nothing — the card stays in the deck and its "When this card is Invoked" ability does not fire. A successful Invoke is a follower entry (Rally counts it — owner ruling 2026-08-12 lists "Invoked from deck") and then the Invoked ability runs. The named card leaves the deck without a `multiset_pick` (trace-format: "Invoke names its card and records nothing").

**Skybound Art / Super Skybound Art.** Evaluated per card in hand when it is played: gauge = the **current turn number** + the number of times an **allied follower evolved while this card was in your hand** (official glossary, 2026-09-10). Skybound Art activates at gauge ≥ **10**, Super Skybound Art at ≥ **15**. Evolves that happened before the card entered your hand do not count.

**Owner ruling — Skybound gauge, every allied evolve counts (2026-08-10):** Every evolve of an allied follower counts for the Skybound gauge, regardless of whether an `Evolve:` script fired or EP was spent (effect-granted evolves count the same as player-spent EP evolves).

**Faith and Modes.** **Mode (official glossary, 2026-09-10):** an ability that lets you pick a specific number of options from a list. You may select options even when you do not meet their printed conditions; those options simply do nothing when they would resolve. Faith is a leader counter on the crest _Faith: Sham-Nacha, Heir to Entwining_ (active while Sham-Nacha is in your deck), starting at 0. It increases by 1 per Modes-selection event (one completed mode-choice resolution), not per individual mode picked — Screaming and Loathing (pick 2 modes at once) adds 1, while a card that selects Modes on both Fanfare and Evolve adds 2. The selectable-mode count = the card's base (usually 1, Screaming 2) + the leader's `modeBonus`. Sham-Nacha's Fanfare spends 10 Faith (`pay_counter` must be ≥ 10 or the pay fizzles; on success `mode_bonus` +1, stacking), and a second successful spend gives +2 total (so Screaming caps at 2 + 2 = 4 picks). Effects that **banish all crests** remove ordinary crests but **not** Faith icons (official Q&A: Alabaster Bahamut mode 3; owner ruling 2026-09-06).

**Effect-granted evolve (when Evolve abilities run).** Effect-granted evolves apply stats and flags only. The `evolve[]` script splits by wording (official Q&A Olivia `10104110`: Evolve abilities activate only when evolved with EP or SEP):

- **"Evolve:"** — the default (82 cards); runs only when the player spends EP.
- **"When this follower evolves"** — not an Evolve ability; runs on **any** evolve, including effect-granted ones (Overflow/Skybound auto-evolve, Necromancy evolve, crest auto-evolve, Olivia Super-Evolve, etc.). Mark the card `evolve_trigger_always: true` when the whole matching `evolve[]` / `superevolve[]` is that trigger (14 Rotation cards in current data), or flag individual effects `on_any_evolve: true` when mixing both wordings in one list (none today).
- **Future:** `on_any_evolve: true` per-effect when a single `evolve[]` mixes an EP-only `Evolve:` line with a When-this-evolves line.

A player's EP evolve always runs the full `evolve[]` / `superevolve[]` script for that mode.

**Card-specific rulings:**

- _Raging Lightning (Overflow leader branch):_ deal 3 damage to every leader whose current defense equals the highest defense among all leaders (both players) — your own leader included; if several tie, all tied leaders take 3.
- _Zooey, Enhance (10):_ (a) setting leader max defense to 1 is a set — current HP is clamped down to the new maximum, repeated sets stay at 1/1, and healing cannot raise current HP above 1; (b) "can't take more than 0 damage at a time" until the opponent's end of turn caps each individual damage instance to the leader at 0 (combat or effect), expiring after the opponent's turn ends (the leader may still be at 1/1).
- _Mari, Meg's Bestie:_ in hand — when a 3 base-cost allied follower super-evolves, Mari's cost becomes 0 until your end of turn (the first such trigger that turn; it stays 0 if more allies super-evolve); on board — at your end of turn, give +1/+1 to one random super-evolved allied follower (any base cost; one super-evolved on a prior turn still qualifies).
- _Azurifrit, Heir to Disdain (`10344110`):_ ~~on your turn, whenever this follower takes damage (including 0) and is not destroyed, deal 1 damage to the enemy leader; each damage instance is a separate trigger, up to 3 activations per turn.~~ **Superseded (owner ruling 2026-08-23).** The trigger fires on **every** qualifying self-damage event during your turn — **no per-turn cap**. Owner: _"No cap — fires every time."_ Playing him (Fanfare 3× AoE hitting himself) and super-evolving him the same turn therefore produces **6** leader pings, not 3. The old `max_per_turn: 3` was an assumed ruling (Fanfare's "Do this 3 times" misread as a trigger cap) and is removed. Separately: his _"Super-Evolve: Fully restore the defense of this follower"_ restores to the **buffed** maximum after the +3/+3 (a 5/7 Azurifrit ends super-evolution at **8/10**, not 8/7).
- _Oluon, Raging Chariot (2026-08-13):_ when evolved, "Deal 7 damage to another random ally or enemy" three times means each hit independently chooses among **all characters except Oluon itself** (both leaders and all other followers on both sides). Previously hit targets remain eligible, so the same follower or leader can be struck more than once, and all three hits can land on the enemy leader (21 total). "Another" exempts only Oluon.
- _Thestae, Anathema of Distortion (2026-08-13):_ the Crest's "Give all followers in your deck +1/+1" applies **only** to follower instances currently in the deck (not cards already in hand or on the field). Those buffs **persist when the card is drawn** (otherwise the Crest would be pointless).
- _Encroached World (2026-08-13):_ Engage and Transform are separate mechanics — Engage activates, and the transform is simply that Engage's effect. There is no special engage-transform interaction to model.
- _Trap in the Woods (`10911210`) (2026-08-29):_ is an **Amulet**, not a Spell. Owner: _"trap in the woods is an amulet not a handtrap spell the fucking site is wrong for over a year now."_ Owner correction beats the scraped source (same class of defect as Azvaldt / Juratio type mistypes).
- _Barren-Earth Tyrant (`10943110`) (2026-08-29):_ when "do it 2 times instead" fires, both hits may land on the same follower — _"Yes — same one can eat 8."_ Authored as independent rolls (`random_hits`), never distinct.
- _Artiglio (`10943310`) / Aragavy (`10154130`) (2026-08-29):_ "Damage split between all enemy followers" is sequential by board age (oldest first, spill onward) — see Split damage. Artiglio deals 3, or **6 instead of 3** (not in addition) when an allied follower attacked a leader last turn; neither spills to the leader.
- _Initiation of Rebirth (`10901310`) (2026-08-29 / tie-break settled 2026-09-02):_ the destroyed-history copy is inserted at a **random** deck index (not top/bottom), from the match RNG — that is what `shuffle: false` on the `deck` add op means (random splice; the rest of the deck order is left undisturbed). The alternative (`shuffle` not false) pushes then shuffles the whole deck. The copy is added _before_ the draw resolves (theoretically drawable by the same spell). With an empty destroyed history, no card is added but **the draw still happens**. ~~Ties on "highest base cost" (assumed, not contested).~~ **Owner ruling 2026-09-02:** _"it says random highest cost so that means if some are tied pick randomly between them"_ — uniformly at random among destroyed allied followers sharing the highest base cost. Printed **random** already answered it (card text is bible); never really open. "Without revealing" is covered by the always-visible practice-tool ruling.
- _Azvaldt, Penitentiary of Chaos (`10903210`) (2026-08-29):_ counts **itself** on its own ladder. Owner: _"azvaldt has to count itself since it says at end of turn if you played. so yes it should count itself at the end of turn."_ The `1…8` ladder can complete the turn Azvaldt lands. Last Words ordering is load-bearing: summon the 4 differently-named followers **first**, then "+3/+3 to all allied followers" catches them too.
- _Elmott, Remembrance Aflame (`10433110`) (2026-08-29):_ card name is **"Elmott, Remembrance Aflame"** (not "Alflame"). Owner: _"elmott exists."_ Name typos break crest / summon / decklist matching keys.
- _Dazzling Runeknight (`10031110`) (2026-08-29):_ costs **3**. Owner: _"dazzling runeknight is 3 cost"_ — local data had been wrong at 2; the scraped source was right.
- _Hien, Redolent Revenant (`10914120`) vs Bayle, Luxglaive Warrior (`10113130`) (2026-08-30):_ same clause shape, different durations. Bayle — _"Whenever an allied follower leaves the field, reduce the cost of this card by 1"_ — **permanent**. Hien — _"Whenever you play a card, reduce the cost of this card by 1 until the end of the turn"_ — **expires each turn** (any allied card played, with `until_eot`). Owner: _"in his case the reduction is permanent in her case it is not permanent and also her reduction is if you just play any card not just if an allied follower leaves the field."_
- _Yube, Crestpetal (`10544120`) / `until_eot` on stat ops (2026-08-29):_ "until the end of the turn" on a stat op must expire; the `until_eot` key is an alias of `until_end_of_turn`. A duration key on an op whose handler ignores it is a silent functional break (Yube's Marine-attack buff was permanent).

### Damage Events (General)

**Split damage (sequential).** Official glossary, Split Damage (2026-09-10): damage is dealt oldest enemy follower to newest, each taking up to its current defense before the next, and leftover is dealt to the enemy leader or, if the ability only targets followers, the last follower. Shares are computed first (`min(pool, current defense)` oldest to newest; the last follower's share also absorbs leftover when the ability does not include the leader), then each share is one damage instance. Barrier reduces the whole share to 0, so a lone 1/1 with Barrier hit by a 6-point follower-only split survives (owner 2026-09-10). Allocated points are still consumed from the pool even when Barrier absorbs them (owner 2026-08-29). Leftover spills to the enemy leader only when the effect sets `includeLeader`.

**Owner ruling — Barrier in split damage (2026-08-29):**

> "if you have 10 points of split damage and a 1/6 with barrier and a 1/5 without barrier: the 1/6 with barrier will take 6 damage reduced to 0 cause of barrier (so still 1/6 but now barrier is gone) and the 1/5 without barrier will now be 1/1 as it takes 4 damage spilled over. oldest to newest"

Rule: split damage allocates against each follower's current defense, oldest to newest; allocated points are consumed from the pool even when Barrier reduces dealt damage to 0 and is consumed. Worked example: 10 split damage — a 1/6 with Barrier (oldest) absorbs an allocation of 6 (stays 1/6, Barrier gone); the 1/5 behind it takes the remaining 4 and becomes 1/1. Do not change without owner sign-off.

A damage instance of **0** still counts as the follower **taking damage** for any "whenever this follower takes damage" trigger, as long as the follower remains on the field and is not destroyed by that event. (Barrier blocking a positive hit also counts as taking damage; see Combat.)

**Owner ruling — Damage prevented by super-evolve still counts as "taking damage" (2026-08-23):**

> "even if superevolved the followers 'take damage' even if it is reduced to 0."

A super-evolved follower's own-turn protection reduces incoming damage to 0 but does **not** cancel the damage event. Every "when this follower takes damage" trigger — Galmieux's 3-damage passive, her crest's Fangs of Ardent Destruction, Azurifrit's leader ping — must fire on such a hit (combat and effect damage alike). The combat path must still call into damage dealing when the attacker is invincible-on-attack; skipping the counter-damage call entirely would starve those triggers.

**Owner ruling — `still_alive` subject is the damage victim (2026-09-08):**

> "ill take galmieux as example here. Galmieux gains the crest that when an ally is damaged and survives the damage then you get a 0 mana spell to hand. ONLY if the unit that was daamged survives -> hence the still alive."

`still_alive` is a property of the **card that took the damage** — the damage event's victim. It is only meaningful where a damage event occurred and the victim remains on the field after that event (not destroyed). It is **trigger-only** — not a pool or card-filter key. For `self_damaged` triggers the subject is the damaged card itself.

**Owner ruling — "Takes N more damage" applies to a 0-damage event (2026-08-31):**

> "id say so yes. since when i attack with a 0 attack in game it deals 0 damage so +1 would be 1. lets keep it until i ever see a situation where that contradicts itself."

A 0-damage event **does** take the bonus (0 + 1 = 1). Healing does not.

---

## Combat, Damage, and Destruction Rules

Combat is straightforward but interacts subtly with abilities. This section covers damage assignment, lethal damage, the aftermath of destruction, and how "destroy," "banish," "transform," and "return to hand" differ.

### Combat Procedure Recap

- **Follower vs follower:** after Strike/Clash triggers, both simultaneously deal damage equal to their attack, marked on each. A combatant reduced to 0 or less defense is destroyed, and combat destruction is processed after the damage event fully completes (including any Bane).
- **Follower vs leader:** only the leader takes damage (it has no attack to retaliate). Leaders are never destroyed by damage — they only lose defense and lose the game at 0.
- **Attacks per turn:** once per turn normally; rare multi-attack abilities allow more. A grant of "Can attack N times per turn" sets the follower's budget to the **maximum** of its current value and N — a lower grant never reduces a higher attacks-per-turn value (official Q&A: Verdilia & Castelle crest and Send 'Em Packing on super-evolved Armes, Depletive Demon).
- **Damage persistence:** damage persists until restored — there is no end-of-turn restoration. A 5/5 that takes 3 becomes 5/2 and stays there until restored or buffed; 2 more damage kills it.
- **Lethal damage:** defense at exactly 0 or below is lethal; there is no negative-defense tracking and no trample/overkill carryover. Followers reduced to 0 in the same event die simultaneously.
- **Destroy vs damage:** a "destroy" effect bypasses damage and kills outright (to the cemetery, firing Last Words), treated like lethal damage — but it is not damage, so it does not interact with Barrier or Drain and does not increment any damage counters.
- **Damage sources:** followers normally deal damage only in combat (some have "Strike: deal X damage"); spells and abilities deal fixed damage. Damage can be buffed or reduced by effects (e.g. "takes 1 less damage from all sources," or an attack buff raising combat damage).
- **Order of damage modifiers vs. set-damage:** effects that **increase or decrease** a damage amount always apply **before** effects that **set** that damage to a fixed value — regardless of the order the effects were granted or how old their sources are on the board. Because the set applies last it overrides the modifiers (e.g. a "+1 damage" buff plus a "deal exactly 3" effect still deals 3).

### Barrier

- **Barrier** is a one-time shield that reduces the next instance of damage to 0, then is removed. It affects only damage (not destroy, banish, or stat reduction), does not stack, and does not save from Bane. (See the keyword entry above for test cases.)
- **Damage floor and no overheal:** defense is not tracked below 0 (0 or less means destroyed), and restoration or shielding never raises defense above full.
- **Setting defense to a value** overwrites both current and max defense to that value, effectively clearing existing damage in the recalculation, so the follower sits at full of its new max. See Stat Modification Hierarchy for how later buffs layer onto a set base.
- **"Lose N defense" / "-N defense" is not damage.** Reducing a follower's defense with a `-X/-Y` or "subtract N defense" effect is a stat (max-defense) reduction, not a damage instance. Consequences:
  - It is **not blocked** by "can't take damage" / damage-immunity effects.
  - It **does not interact with Barrier at all** — Barrier is neither consumed nor does it prevent the reduction. If the reduction brings defense to 0 or below, the follower is destroyed by the normal "0-or-less defense = destroyed" rule (Barrier does not save it); otherwise defense simply drops and Barrier stays.
  - A follower whose defense was lowered this way sits at the **full value of its new (lower) max**, so it is **not in a "damaged" state**, and effects that target/destroy "a damaged follower" cannot select it. (Contrast: a 5/5 reduced to 5/3 by `-0/-2` is _not_ damaged; a 5/5 sitting at 5/3 because it _took_ 2 damage _is_ damaged.)

### Bane, Drain, and Special Damage Effects in Combat

- **Bane:** a Bane follower that deals any combat damage (even 0) to a follower destroys it after damage resolves. The sequence is: exchange damage, then check Bane — a combatant that fought a Bane follower is destroyed (if both had Bane, both die). _Example:_ a 1/1 Bane into a 10/10 — the 1/1 dies to the counter-damage, but its Bane still destroys the 10/10, so both die. A Bane kill is destruction by an ability (not by damage) tied to combat: it fires the victim's Last Words, Barrier does not stop it, and the Bane follower need not survive. A follower that "can't be destroyed by abilities" survives Bane as long as it still has more than 0 defense.
- **Drain:** in combat, after damage, an attacker with Drain restores its leader by the damage it dealt. A _defending_ Drain follower's counter-damage restores nothing, and effect (non-combat) damage from a Drain follower restores nothing.
- **Splash / AoE:** simultaneous damage to several targets is applied to all, then destruction is checked for all at once; the resulting triggers resolve by the queue ordering (active side's Last Words first). A single "deal damage to both leaders" instruction damages both leaders before game-over is judged; if both are at 0 or less, the active player loses.
- **Self-damage:** damaging your own followers or leader works the same way; your leader at 0 loses you the game, and your own units dying are normal deaths (producing your shadows).

## Destruction vs. Other Removal

A card can leave play in several ways:

- **Destroy:** goes to the cemetery (creates a shadow, fires Last Words). **Owner ruling — Engage self-sacrifice is destruction (2026-09-09):** an amulet consumed by its own Engage is **destroyed** and raises `ally_amulet_destroyed`. Owner: _"but i think they should because they say engage destroy"_. Official Cygames Q&A for `10001210`, `10002210`, `10112210`, `10113210`, and `10162220` (can I Engage with no legal target?) — every answer says the amulet is **destroyed** (e.g. _"Yes. Doing so will simply destroy it."_). Consumers at ruling time: Lyanthoth, Eld Tome `10664120` (Faith crest) and Omerio, Winged Revenant `10964120` (sequence trigger); 22 amulets carry `Engage` with `sacrifice: true`.
- **Banish:** removed from play entirely (no shadow, no Last Words).
- **Transform:** the card in play becomes a different card; the original ceases to exist — no Last Words, no shadow, and its continuous effects end — while the new card takes its slot in place. The new card is unrelated to the old one (no stat or ability inheritance). **Transform does not count as leaving play or entering play** — neither "when leaves the field" nor "when enters the field" triggers fire. **Owner ruling (2026-09-08):** transform is not a leave; the transformed-in card is not an enter. **Owner ruling (2026-09-10), reconciling the official glossary** ("A card transformed into a follower cannot attack enemies until the following turn") **with the 2026-09-06 ruling:** the transformed-in card is a fresh printed instance with all of its printed keywords, exactly like a summoned one — summoning-sick by default, but Rush or Storm on the *new* card let it attack that turn (Imari's Little Buddies has Rush → can attack followers), and a printed Ward/Aura/etc. is simply there. Rush and Storm do not override the "not an enter" rule; they only allow attacking the turn the card appears. Enter / ally_enter / Rally / `enteredThisMatch` still do not fire. Owner: _"rush and storm dont override anything. they allow cards to attack the turn they are played/summoned … same with transform. its like you summoned a card (by obliterating another one)."_ **Source:** [Shadowverse 効果処理 wiki — 変身と破壊の違い](https://w.atwiki.jp/svkoukasyori/pages/16.html) — Last Words, "when destroyed", and "when leaving the field" do not activate; the count of "cards that entered the field" does not increase (it is not treated as having entered the field).
- **Return to hand (bounce):** the card goes to its owner's hand; this is not a destroy (no shadow, no Last Words) but does count as leaving play (for conditions like "if 4 allies left play this turn"). A returned follower reverts to base stats unless stated otherwise, frees its field slot, and can cause a hand-overflow destruction if the hand would exceed 9.
- **Return to deck (retreat):** the card is shuffled or placed back into the deck — leaving play without destruction (no Last Words, no shadow). Cards returned to deck from **hand** retain any effects applied to them (official glossary, 2026-09-10). Current effects only move cards from hand back into the deck, and those keep any buffs or nerfs.
- **Leaves-play triggers:** banish, destroy, and bounce count as leaving play; **transform does not** (see Transform above). Triggers that say "destroyed" mean specifically destroyed; "leaves play" means removal other than transform — the engine should distinguish them. Last Words that summon followers place them after a board wipe, so those summons survive (they entered afterward).
- **Forfeit / concede:** an immediate loss for that player (not needed for the engine).

## Continuous Effects and Modifiers

Many effects modify stats or grant abilities, temporarily or permanently. Implementing them requires tracking layers/timestamps. SVWB is not codified as strictly as MTG's layer system, but the following holds.

### Stat Modification Hierarchy

- **Base stats:** each follower/leader has base Attack/Defense. For leaders, base defense is the current maximum (20 by default, can be increased).
- **Permanent buffs/debuffs:** effects that add, subtract, or set stats and are not marked "until end of turn" last for the match (or until the card leaves play), modifying stats on an ongoing basis.
- **Temporary buffs/debuffs:** effects lasting "until end of turn" (or a stated duration) apply on top of the base and wear off at expiry. Card-data keys `until_end_of_turn` and `until_eot` are aliases — a handler that reads only one of them silently ignores the other (see Yube under Card-specific rulings).
- **Set-value effects:** an effect that sets a stat to a specific number overrides other modifications at the moment it applies and persists as the new base. "Set defense to 1" overrides current and max defense to 1 regardless of prior buffs or damage (existing damage is cleared in that recalc, leaving the follower at full of the new max). The set does **not** lock the stat against future changes — a later buff stacks on the set base, so "set to 1" followed by "+0/+2" yields 3 defense — but the follower cannot be healed above its new max (1 becomes its new maximum).
- **Evolution stat bonuses:** a normal evolve adds +2/+2 on top of current stats (unless otherwise specified). A super-evolve adds +3/+3 and, **on its owner's turn only**, makes the follower unable to be destroyed by abilities/effects and reduces any damage it takes to 0. (After the owner ends their turn, it takes damage and can be destroyed like any other follower.) Official glossary, 2026-09-10 (Super-Evolution): "During your turn, this follower can't be destroyed by abilities, and damage it takes is reduced to 0." A super-evolved follower also has a **knockback** effect: when it attacks and destroys the follower it is attacking (its combat target), the enemy leader takes 1 damage. Specifics:
  - It fires only for the **combat target** of the attack — destroying _other_ followers (e.g. via a Strike/attack effect that kills a different follower) does **not** deal the 1.
  - It can fire at either moment the combat target dies: when the attacker's Strike/Clash effect destroys it, or when combat damage destroys it.
  - The 1 damage resolves **before** the destroyed follower's Last Words (e.g. a super-evo attacker kills a follower whose Last Words restores the enemy leader 2 → the leader first takes the 1 knockback, then heals 2).
  - It is tied to the attacking super-evo follower only: a super-evo follower being _attacked_, or killing via a non-combat effect, does not trigger it.
- **Ability buffs/debuffs:** gaining or losing abilities is tracked the same way. Silencing a follower removes its text and keyword flags (Bane, Drain, auras, etc.) and blanks its text, but does not remove numeric stat buffs already applied as values.
- **Buff stacking:** multiple +ATK buffs simply sum.
- **Attack debuffs:** −X attack can take attack to 0 (and the internal value may go negative), but a follower never deals negative damage — minimum 0. A follower at −2 attack deals 0; buffing it with +3/+3 leaves it at 1 attack (−2 + 3). A 0-attack follower can still declare an attack (dealing 0), and with Bane its 0 damage still destroys the target.
- **Temporary vs permanent:** "until end of turn" effects wear off at end-of-turn cleanup; if the follower leaves play or is transformed before then, the buff ends with it.
- **Ability layering:** an effect-granted ability lasts permanently unless given a duration; abilities lost via transform or silence are gone permanently. When two continuous effects conflict (e.g. "this follower cannot attack" vs "this follower can attack twice"), the restrictive one wins — "cannot" effects take precedence.

## Hidden Information and Reveal Mechanics

Hidden information mainly concerns cards in hand and deck:

- **Hand:** only the owner sees their card identities; the opponent sees the count.
- **Deck:** neither player sees the opponent's order or contents (beyond known composition or reveal effects). For training, an open decklist may be used so both bots know each other's decks if feasible.
- **Drawn cards:** only the drawing player learns the identity; the opponent sees only that the hand count rose by one.
- **Created tokens added to hand:** only that player knows which token it is, unless explicitly revealed.
- **Cemetery:** which cards died is visible to the opponent (via the Battle Log), as is the shadow count (relevant for Abysscraft).

**Practice-tool exception:** see **Owner ruling — Hidden information always visible (2026-08-13)** under Hidden Information. "Without revealing" text does not hide copies in this tool.

## Copy semantics (exact copy vs copy)

**Owner ruling — Exact copy vs copy (2026-08-13):** This is a **general** rule for every copy effect in the game, not a per-card note:

- **"an exact copy"** (or "exact copies") copies the **instance**, including its current stats and modifications (buffs, cost mods, granted keywords, evolve state where applicable, and other lasting instance fields), but **not** whether it had attacked or been engaged that turn (official glossary, 2026-09-10). The copy is still a new object (new uid; board combat / summoning-sick / Skybound gauge witnesses are reset when entering hand as appropriate).
- **"a copy"** / **"copies of"** (without "exact") copies the **base/printed card** — a fresh template from the card database (destroyed-history recreations, named token summons, etc.).

When card text says "Summon 2 copies of Fairy," that is base copies of the named card. When it says "summon an exact copy of this card" or "add an exact copy of a random card in your opponent's hand," that is an instance clone of the referenced card.
