# Official glossary audit (2026-09-10)

Source: `rules/official-glossary.md` (78 entries from `GET https://shadowverse-wb.com/web/System/glossaryList`). Compared against `rules/rulebook.md` and `rules/owner-rulings.md`. Owner-rulings.md is **not** edited — conflicts are reported only.

## Summary

| Verdict | Count |
|---|---:|
| agrees | 58 |
| rulebook silent → added | 17 |
| conflict | 3 |

The 17 **rulebook silent** entries received glossary sentences in `rules/rulebook.md` tagged `(official glossary, 2026-09-10)` on 2026-09-10. The 3 **conflict** rows are listed under [Conflicts for the owner](#conflicts-for-the-owner) below.

## Audit table

| Entry | Rulebook / rulings | Verdict |
|---|---|---|
| Follower | [Card Types](rulebook.md#card-types-and-terminology) — enters field, summoning sickness | agrees |
| Spell | [Card Types](rulebook.md#card-types-and-terminology) — resolves then cemetery | agrees |
| Amulet | [Card Types](rulebook.md#card-types-and-terminology) — field, no combat | agrees |
| Crest | [Zones](rulebook.md#zones-and-card-locations) — leader area, one of each (owner 2026-09-05 duplicate bounce) | agrees |
| Token | [Card Types](rulebook.md#card-types-and-terminology); owner 2026-08-29 Basic A set | agrees |
| Class | [Card Types](rulebook.md#card-types-and-terminology) — eight crafts + Neutral | agrees |
| Trait | [Card Types](rulebook.md#card-types-and-terminology) — tribes, no standalone effect | agrees |
| Cost | [Playing Cards](rulebook.md#playing-cards-from-hand) — PP to play | agrees |
| Attack (Stat) | [Card Types](rulebook.md#card-types-and-terminology) — combat damage stat | agrees |
| Defense | [Card Types](rulebook.md#card-types-and-terminology); leader at 0 loses | agrees |
| Match | [Match Flow](rulebook.md#match-flow); leader at 0 ends match | agrees |
| Leader | [Zones](rulebook.md#zones-and-card-locations) — 20 defense, not a card | agrees |
| Deck | [Zones](rulebook.md#zones-and-card-states); deck-out on draw | agrees |
| Field | [Zones](rulebook.md#zones-and-card-states) — 5 slots | agrees |
| Hand | [Zones](rulebook.md#zones-and-card-states) — limit 9 | agrees |
| Cemetery | [Zones](rulebook.md#zones-and-card-states) — shadows | agrees |
| Leader Area | [Zones](rulebook.md#zones-and-card-locations) — 5 crest/faith slots | agrees |
| Play Point | [PP rules](rulebook.md#play-point-pp-and-evolution-point-rules) — +1 max/turn, cap 10 | agrees |
| EP (Evolution Point) | [PP rules](rulebook.md#play-point-pp-and-evolution-point-rules) — 2 per player | agrees |
| SEP (Super-Evolution Point) | [PP rules](rulebook.md#play-point-pp-and-evolution-point-rules) — 2 per player | agrees |
| Play | [Main phase](rulebook.md#turn-sequence) — spend PP from hand | agrees |
| Turn | [Turn Sequence](rulebook.md#turn-sequence) | agrees |
| Attack (Action) | [Main phase / Attacking](rulebook.md#turn-sequence) had summoning sickness but not the explicit once-per-turn-from-next-turn wording | rulebook silent → **added** |
| Redraw | [Match Flow](rulebook.md#match-flow) — mulligan described, not “never same instance” | rulebook silent → **added** |
| Bonus Play Point | [PP rules](rulebook.md#play-point-pp-and-evolution-point-rules); sixth-turn refresh now explicit | rulebook silent → **added** |
| Evolution | [Evolve keyword](rulebook.md#classmechanic-specific-keywords); +2/+2, attack followers same turn | agrees |
| Super-Evolution | [Continuous Effects](rulebook.md#continuous-effects-and-modifiers) — +3/+3, protection, knockback, thresholds | agrees |
| Combo | Not previously defined | rulebook silent → **added** |
| Spellboost | [Spellboost](rulebook.md#classmechanic-specific-keywords) | agrees |
| Earth Rite | [Earth Rite](rulebook.md#classmechanic-specific-keywords) | agrees |
| Earth Sigil | [Earth Rite / Sigils](rulebook.md#classmechanic-specific-keywords) — stack/merge; glossary adds banish-on-entry, immunity, Sediment summon | rulebook silent → **added** (merge vs owner: **conflict**) |
| Overflow | [Overflow](rulebook.md#classmechanic-specific-keywords) — ≥7 max PP | agrees |
| Necromancy | [Necromancy](rulebook.md#classmechanic-specific-keywords) | agrees |
| Reanimate | [Reanimate](rulebook.md#classmechanic-specific-keywords); owner 2026-09-02 field-destroyed only | rulebook silent → **added** (Departed trait, destroy-count weighting) |
| Storm | [Storm](rulebook.md#classmechanic-specific-keywords) | agrees |
| Rush | [Rush](rulebook.md#classmechanic-specific-keywords) | agrees |
| Ward | [Ward](rulebook.md#classmechanic-specific-keywords) | agrees |
| Bane | [Bane](rulebook.md#classmechanic-specific-keywords) | agrees |
| Ambush | [Ambush](rulebook.md#classmechanic-specific-keywords) | agrees |
| Drain | [Drain](rulebook.md#classmechanic-specific-keywords) | agrees |
| Countdown | [Countdown on crests](rulebook.md#classmechanic-specific-keywords); amulet countdown in turn sequence | agrees |
| Intimidate | [Intimidate](rulebook.md#classmechanic-specific-keywords) — not attackable | agrees |
| Aura | [Aura](rulebook.md#classmechanic-specific-keywords) | agrees |
| Barrier | [Barrier](rulebook.md#classmechanic-specific-keywords) | agrees |
| Fanfare | [Fanfare](rulebook.md#classmechanic-specific-keywords) — play from hand only | agrees |
| Last Words | [Last Words](rulebook.md#classmechanic-specific-keywords) — not on banish/transform | agrees |
| Evolve | [Evolve ability](rulebook.md#classmechanic-specific-keywords) | agrees |
| Super-Evolve | [Super-Evolve ability](rulebook.md#classmechanic-specific-keywords) | agrees |
| Strike | [Strike](rulebook.md#classmechanic-specific-keywords) | agrees |
| Clash | [Clash](rulebook.md#classmechanic-specific-keywords) | agrees |
| Enhance | [Enhance](rulebook.md#classmechanic-specific-keywords); owner multi-tier 2026-08-15 | agrees |
| Mode | [Faith and Modes](rulebook.md#classmechanic-specific-keywords) — Faith/Modes economy; not “pick without conditions” | rulebook silent → **added** |
| Fuse | Owner 2026-08-29 once per instance; JA glossary per-card; EN “once per turn” is imprecise | agrees (with owner + JA) |
| Engage | [Engage](rulebook.md#classmechanic-specific-keywords); owner 2026-09-09 destruction | agrees |
| Give +X/+Y | [Stat hierarchy](rulebook.md#stat-modification-hierarchy) | agrees |
| Damage | [Combat / damage](rulebook.md#combat-damage-and-destruction-rules) | agrees |
| Restore | [Combat](rulebook.md#combat-damage-and-destruction-rules) — cap at max | agrees |
| Draw | [Hand limit](rulebook.md#zones-and-card-states) — overflow to shadow; now tagged on Draw op | rulebook silent → **added** |
| Add to Hand | Same hand-overflow rule; now explicit for Add op | rulebook silent → **added** |
| Summon | [Field limit](rulebook.md#zones-and-card-states) partial; full-field “nothing happens” now explicit | rulebook silent → **added** |
| Destroy | [Destruction](rulebook.md#destruction-vs-other-removal) — shadow | agrees |
| Banish | [Destruction](rulebook.md#destruction-vs-other-removal) | agrees |
| Leave the Field | [Destruction](rulebook.md#destruction-vs-other-removal) — transform excluded; owner 2026-09-08 | agrees |
| Split Damage | [Split damage](rulebook.md#damage-events-general) `split_sequential` + owner 2026-08-29 (followers only, no leader spill) | **conflict** |
| Select | [Targeting](rulebook.md#targeting-and-selection-effects); owner 2026-08-16 / 2026-09-08 | agrees |
| Exact Copy | [Copy semantics](rulebook.md#copy-semantics-exact-copy-vs-copy); owner 2026-08-13 | rulebook silent → **added** (attacked/engaged) |
| Return to Deck | [Destruction](rulebook.md#destruction-vs-other-removal) — hand keeps effects; now glossary-tagged | rulebook silent → **added** |
| Return to Hand | [Destruction](rulebook.md#destruction-vs-other-removal) bounce; hand overflow | rulebook silent → **added** (full-hand shadow) |
| Discard | [Cemetery](rulebook.md#zones-and-card-states) — shadow | agrees |
| Transform | [Transform](rulebook.md#destruction-vs-other-removal) leave/enter; `docs/design.md` “new card may attack” vs glossary attack delay | **conflict** |
| Rally | [Rally](rulebook.md#classmechanic-specific-keywords); owner 2026-08-12 | agrees |
| Faith | [Faith and Modes](rulebook.md#classmechanic-specific-keywords); owner/Q&A | agrees |
| Victory Card | [Win/Loss](rulebook.md#winloss-conditions) — deck-out only; Reaper/Victory win now added | rulebook silent → **added** |
| Invoke | Mentioned in Rally ruling only | rulebook silent → **added** |
| Skybound Art | [Skybound Art](rulebook.md#classmechanic-specific-keywords); owner 2026-08-10 | agrees |
| Super Skybound Art | [Skybound Art](rulebook.md#classmechanic-specific-keywords) — gauge ≥15 | agrees |
| Accelerate | [Accelerate](rulebook.md#classmechanic-specific-keywords); owner 2026-09-06 | agrees |
| Crystallize | [Crystallize](rulebook.md#classmechanic-specific-keywords); owner 2026-09-06 | agrees |

---

## Conflicts for the owner

### 1. Transform — attack timing (owner ruling 2026-09-06 / design synthesis)

**Official glossary (EN):**

> Transforming a card changes it into a different card. … A card transformed into a follower cannot attack enemies until the following turn.

**Official glossary (JA):**

> フォロワーに変身した場合、次のターンから、相手のリーダーやフォロワーを攻撃できます。

**Arena position today:** `docs/design.md` and the transform synthesis in `rules/rulebook.md` treat transform as neither leave nor enter; the transformed-in card is **not** an enter and **may attack** if it has Rush/Storm (owner play-testing, 2026-09-06). The glossary imposes summoning-sickness on transformed-in followers.

**Action:** rulebook **not** changed on this point; reported for owner resolution.

---

### 2. Split Damage — leftover spill (rulebook + owner 2026-08-29)

**Official glossary (EN):**

> … Any leftover damage is dealt to the enemy leader or, if the ability only targets followers, the last follower.

**Official glossary (JA):**

> … 割りふったダメージが残った場合、相手のリーダー、または対象がフォロワーのみの場合は最後のフォロワーに与えられます。

**Rulebook / owner:** [Split damage](rulebook.md#damage-events-general) and owner ruling 2026-08-29 (Artiglio / Aragavy) — sequential oldest-first spill among followers; **no** spill to the leader when the text targets followers only (`spill_to_leader` off). Neither source mentions spilling leftover pool onto the **last** follower after the newest is exhausted.

**Action:** rulebook **not** changed; reported for owner resolution (glossary vs printed cards vs engine).

---

### 3. Earth Sigil merge — played Brew onto existing Brew (owner ruling 2026-08-30)

**Official glossary (EN):**

> When an amulet with Earth Sigil enters the field, its sigil count is set to 1 … Any other allied amulets with Earth Sigil on the field will be **banished**, and their sigil counts added on to the **new** amulet.

**Official glossary (JA):**

> 土の印を持つアミュレットが場に出たとき、右下に土の印の数が1として表示されます。他の自分の場の土の印を持つアミュレットは消滅し、土の印の数が新しく出たアミュレットに加算されます。

**Owner ruling (2026-08-30):** collectible Witch's New Brew always wins over token Magic Sediment; generating Sediment while Brew is present only increments Brew. The ruling does **not** address playing a second Brew onto an existing Brew. The glossary says the **newly played** amulet survives and the old one is banished (counts sum on the new card).

**Agreed subset:** “gain X earth sigils” with a holder present adds to the stack; generating Sediment with Brew on field increments Brew (no second amulet).

**Disputed subset:** playing Witch's New Brew from hand while another Brew is already on the field — glossary: first Brew banished, new Brew holds the total; owner ruling’s “Brew always wins” was argued for the **existing** collectible surviving.

**Action:** rulebook records glossary text and flags this case; owner ruling file unchanged.

---

## Engine implications (separate briefs)

One-line items for follow-up engine/oracle work — **not** implemented in this PR:

1. Earth Sigil amulets need `cantBeDestroyedByAbilities` and cannot be selected by enemy abilities.
2. Earth Sigil entry merge must **banish** prior sigil amulets (no shadow), not destroy-to-cemetery.
3. Earth Sigil entry sets count to 1 then adds banished stacks; “gain X sigils” adds to holder or summons Magic Sediment at count X.
4. Reanimate must grant the **Departed** trait on the summoned copy.
5. Reanimate tie-break among equal base costs must weight by **destroyed copy count**, not uniform random.
6. Combo must include the **card being played** in the count for its own threshold.
7. Mode: allow selecting options whose conditions fail; those branches fizzle.
8. Invoke: require field space; randomize simultaneous entry order; at most one copy per card name; weight by deck copies.
9. Victory Card: deck-out draw with transformed bottom card wins instead of losing.
10. Redraw/mulligan: never return the same deck **instance** to hand.
11. Exact copy: clone damage/effects but reset attacked/engaged flags.
12. Transform attack permission: resolve owner vs glossary before implementing Rush/Storm on transform-in.
13. Split damage: resolve whether follower-only spill can hit the last follower after the pool is exhausted.
14. Earth Sigil Brew-on-Brew: resolve survivor identity (new vs existing amulet) per owner vs glossary.
