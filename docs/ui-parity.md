# UI parity audit — old Practice-Tool client vs arena `ui/`

Audit of every user-visible feature of the old practice tool’s **client** (`$HOME/practice-tool`, `origin/main` `bf8a9123`) against the new client (`ui/` at `8dfcb15`, plus this doc). Rules, card facts, and engine computation are out of scope.

**Visible result, not a code copy** (owner, 2026-09-11): *“make sure you don't copy it blindly. you can improve the code but make sure the result is the same. same glows, counters, colors etc.”* A row is `ported` only when the player sees the same thing (what, where, colour/token, when, how long). `partial` means it looks or behaves differently, even if the new code is “equivalent.” Gap specs describe that result — old CSS token names, timings, and corners — so the next agent can write its own code. File:line cites are evidence, not a paste target.

**Old paths** are relative to `$HOME/practice-tool`. **New paths** are relative to this repo.

**Verification.** Every row was read in both trees. Rows tagged `(code-read)` had no matching on-screen state in this pass (Barrier, Skybound, fuse/choice, game-over, F8, etc.). Untagged rows were also confirmed on a running preview (`wasm-pack` + `ui` production build + Chromium at `http://127.0.0.1:4173`). Screenshots: `/opt/cursor/artifacts/parity_*.png`.

**Round 3.** Items already being fixed on `cursor/m4-feedback-3` are `in progress (round 3)` with an empty gap spec — do not re-implement from this list.

**Status** ∈ `ported` | `partial` | `missing` | `dropped (by design)` | `in progress (round 3)`.

Dropped-by-design (brief RZ / `ui/README.md:50`): blackbox, sparring-line scripts, puzzles, coverage banner, god-mode. Adjacent drops I believe were intentional (confirm with owner): Consistency Trainer, `?test=1` QA bridge, UNIMPL/UNKNOWN OP face badges, burn-preview box, deck *paste* import panel (JSON file import was ported instead).

---

## 1. Layout & zones

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Fixed `100dvh` `#gameShell`, no page scroll, `user-scalable=no` | `index.html:7`, `index.html:151`; `css/phase3-layout.css:71-83` | `ui/index.html:5-8,80`; `ui/css/phase3-layout.css:65-81,703-706` | ported | 
| Safe-area insets on the shell | `css/phase3-layout.css:81-82` | `ui/css/phase3-layout.css:79-81` | ported | 
| Left gutter 68px (hamburger + log), right gutter 232px (`#turnControls`) | `css/phase3-layout.css:7-8` | `ui/css/phase3-layout.css:7-10` | ported | 
| Play area max width 1920px, centered | `css/design-tokens.css:230`; `css/modern-theme.css:18-22` | `ui/css/design-tokens.css:230`; `ui/css/modern-theme.css:18-22` | ported | 
| Red top (`order:1`) / blue bottom (`order:2`) by default | `index.html:617-680`; `css/phase3-layout.css:249-255` | `ui/index.html:237-264`; `ui/css/phase3-layout.css:226-232` | ported | 
| Red stack: hand → leader strip → board; blue: board → leader strip → hand | `index.html:617-680`; `css/phase3-layout.css:136-178` | `ui/index.html:237-264`; `ui/css/phase3-layout.css:116-155` | ported | 
| Board followers align toward the centre line; reverses with flip | `css/phase3-layout.css:196-210` | `ui/css/phase3-layout.css:173-187` | ported | 
| Empty board label “Play zone” | `css/layout.css:234-247` | `ui/css/layout.css:236-249` | ported | 
| Per-side board tint (red/blue dashed zone) | `css/layout.css:249-267`; `--color-board-*-tint` | `ui/css/layout.css:219-269`; same tokens | ported | 
| Occupied slots only, engine order, row centred | (old rendered occupied cards in a flex row) | `ui/src/render.ts:226-232` (R1) | ported | 
| Active-on-bottom: `body.active-on-bottom` + `active-second` swaps side `order` (code-read) | `src/boot/perspectiveFlip.ts:4-14`; `src/ui/render.ts:31,44-66`; `css/phase3-layout.css:257-263` | `ui/index.html:181-184`; `ui/src/main.ts:695-699`; `ui/css/phase3-layout.css:234-240` | partial | After reload, the checkbox and the board match last time: if `localStorage.svwb.activeOnBottom` is `"1"`, the active side is already on the bottom (same `order` swap as during play). |
| Bottom hand full size, top hand `--hand-scale-top` 0.72–0.82 | `css/phase3-layout.css:30-45,266-289` | `ui/css/phase3-layout.css:30,243-249` (both hands `--hand-local-scale: 1`) | dropped (by design) | Equal hand sizes were an explicit R1/R2 ask (`--card-height` clamp for both hands). |
| Card footprint 108×162 (114×171 @1440, 118×177 @1900) | `css/design-tokens.css:233-234,284-298` | `ui/css/design-tokens.css:235-237` (`clamp(110px, (100vh-48px)/6.4, 260px)`, width = 2/3 height) | ported | Viewport-scaled cards were R2; same tokens, different formula. |
| Board cards × `--board-card-scale` 1.12 (1.18 @2560) | `css/phase3-layout.css:501-504`; `css/design-tokens.css` | `ui/css/design-tokens.css:237`; `ui/css/phase3-layout.css:253-257,484-485` | ported | 
| Hand overlap: loose→tight, min ~60px strip, 7/8/9-card container queries | `css/design-tokens.css:241-243`; `css/phase3-layout.css:302-341` | `ui/css/design-tokens.css:247-250`; `ui/css/phase3-layout.css:263-336` (overlap only when needed, cap 35% width) | ported | R2 tightened the fit rule; player still sees a fan that overlaps only when the row would overflow. |
| Hand z-index fan 1–9; hover `z-index:20`, `translateY(-6px) scale(1.03)` (code-read) | `css/layout.css:136-167` | `ui/css/layout.css:136-167`; `ui/css/phase3-layout.css:259-261` | ported | 
| Active-side hand green inset ring `rgba(57,217,138,0.55)` | `css/modern-theme.css:955-960` | `ui/css/modern-theme.css:946-951` | ported | 
| Active-side leader `--glow-success` ring | `css/modern-theme.css:962-968` | `ui/css/modern-theme.css:953-959` | ported | 
| Active-side background gradient (`body.active-first` / `active-second`) (code-read) | `css/modern-theme.css:449-467` | `ui/css/modern-theme.css:449-467` | ported | 
| Leader attack strip 32px, Evo / HP pill / Super, hit-pad `::before` | `index.html:629-668`; `css/phase3-layout.css:15-19,412-481` | `ui/index.html:242-258`; `ui/css/phase3-layout.css:15-19,365-464` | ported | 
| Right rail: red stats+crests top, core (PP / End Turn / boost) centre, blue bottom | `index.html:685-908`; `css/phase3-layout.css` | `ui/index.html:266-330`; `ui/css/phase3-layout.css:555-592` | ported | 
| Settings hamburger 42×42 top-left | `index.html:166-173`; `css/settings-drawer.css:6-48` | `ui/index.html:82-84`; `ui/css/settings-drawer.css:6-48` | ported | 
| Battle-log button 46×46 left, above the bottom hand | `index.html:557-587`; `css/phase3-layout.css:522-570` | `ui/index.html:207-212`; `ui/css/phase3-layout.css:505-553` | ported | 
| Live turn / ply readout on the rail | — (no live turn on the board; only on saved-position labels) | `ui/index.html:289-290`; `ui/src/render.ts:86-99` (`Turn N · A or B` / `Winner x`) | ported | New-only extra. Round 3 is rewriting the terminal label; see §6. |

---

## 2. Card rendering — hand and board

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Card frame `--card-width/height`, `--color-card-frame`, hover lift | `css/cards.css:1-25` | `ui/css/cards.css:1-25` | ported | 
| `.hidpi` contrast/saturation on face-up art (code-read) | `src/ui/zones/dom.ts:44`; `css/modern-theme.css:751-754` | `ui/src/render/card.ts:109`; `ui/css/modern-theme.css:742-745` | ported | 
| Cost badge top-left, `--color-stat-cost-bg` `#1e5c32` | `css/cards.css:71-76`; `src/ui/zones/dom.ts:140-143` | `ui/css/cards.css:71-76`; `ui/src/render/card.ts:117-120` | ported | 
| Cost badge shows PP actually paid (hand preview / `handInfo.cost`) (code-read) | `src/ui/zones/viewModel.ts:72-91` | `ui/src/info.ts:55-63`; `ui/src/render.ts:190` | ported | 
| ATK badge bottom-left `#123a6e` (followers only) | `css/cards.css:78-83`; `src/ui/zones/dom.ts:157-159` | `ui/css/cards.css:78-83`; `ui/src/render/card.ts:187-191` | ported | 
| DEF badge bottom-right `#6e1a24` (followers only) | `css/cards.css:85-90`; `src/ui/zones/dom.ts:157-159` | `ui/css/cards.css:85-90`; `ui/src/render/card.ts:192-196` | ported | 
| `.stat-buffed` text `--color-stat-buffed` `#39d98a` + green glow (ATK if buffed; DEF if buffed and not damaged) (code-read) | `css/cards.css:577-581`; `src/ui/zones/dom.ts:163-168`; `src/ui/zones/viewModel.ts:122-151` | `ui/css/cards.css:586-590` (CSS only; never applied) | in progress (round 3) | 
| `.stat-damaged` text `--color-stat-damaged` `#ffb366` (DEF below max; ATK if debuffed) (code-read) | `css/cards.css:583-588`; `src/ui/zones/dom.ts:163-168` | `ui/css/cards.css:593-597`; `ui/src/render/card.ts:161,195` (DEF only) | in progress (round 3) | 
| Evolved: gold ring `--color-evolved-ring` + top-right **E** (code-read) | `css/cards.css:517-534`; `src/ui/zones/dom.ts:66-67` | `ui/css/cards.css:519-543`; `ui/src/render/card.ts:173-174` | ported | 
| Super-evolved: purple pulse `pulseSuper` 1.4s + **SE** (code-read) | `css/cards.css:536-574`; `src/ui/zones/dom.ts:66` | `ui/css/cards.css:545-583`; `ui/src/render/card.ts:173` | ported | 
| Evolved / super art (evo hash / evo image) (code-read) | `src/ui/zones/dom.ts:76-80` (`card.base_image` or `card.image`) | `ui/src/images.ts:5-9,115`; `ui/src/render/card.ts:115` (`evoCard` when evolved/super) | ported | 
| `updateCard` on reused DOM must refresh countdown, overlays, evo art — not just cost/ATK/DEF (code-read) | old replaces the node when the VM changes (`src/ui/zones/index.ts:115-137`) | `ui/src/render/card.ts:130-164` updates cost/ATK/DEF/form only | in progress (round 3) | 
| Alternate-form badge cyan `{formLabel} {cost}` (e.g. `Accelerate 1`) at top of art (code-read) | `css/cards.css:623-637`; `src/ui/zones/dom.ts:145-151` | `ui/src/info.ts:46-53`; `ui/src/render/card.ts:205-214` (letter **E/A/C** only) | in progress (round 3) | 
| No E/A/C trinity badges on the cost chip; yellow = enhance/alternate or a met gate, green = playable, none = unplayable (code-read) | `src/ui/helpers/glow.ts:237-260`; `css/cards.css:590-662` | `ui/src/render/card.ts:56-72` still emits E/A/C + `enhance-ready` / `playable-glow` | in progress (round 3) | 
| `.playable-glow` green `playPulse` 1s — base-cost playable, no special form | `css/cards.css:654-662`; `src/ui/helpers/glow.ts:260` | `ui/css/cards.css:144-157,667-692`; `ui/src/render/card.ts:66` | ported | Same tokens; acting-player-only is round 3. |
| `.enhance-ready` yellow `enhPulse` 1s — Enhance or a met special gate (code-read) | `css/cards.css:590-598`; `src/ui/helpers/glow.ts:241-257` | `ui/css/cards.css:599-611`; `ui/src/render/card.ts:62-65` | in progress (round 3) | 
| `.alternate-ready` amber `altPulse` 1s — Accelerate / Crystallize payable (code-read) | `css/cards.css:600-608`; `src/ui/helpers/glow.ts:237-238` | `ui/css/cards.css:613-634`; `ui/src/render/card.ts:70` | in progress (round 3) | 
| Hand/board glow only on the acting player’s cards (code-read) | `src/ui/helpers/glow.ts:146-148`; `src/ui/zones/viewModel.ts:98,163` | `ui/src/render.ts:176-187,247` uses `handInfo`/`boardInfo` (mirrors acting-player legality) | in progress (round 3) | 
| `.can-attack` green outline — follower can attack (not rush-restricted) (code-read) | `css/cards.css:144-148`; `src/ui/zones/dom.ts:69-72` | `ui/css/cards.css:144-157`; `ui/src/render/card.ts:69` | ported | Rush-vs-green split is round 3. |
| `.rush-glow` yellow — Rush on the summon turn (`hasRush && justPlayed && !hasStorm`) (code-read) | `src/ui/zones/viewModel.ts:183`; `src/ui/zones/dom.ts:70`; `css/cards.css:201-205` | `ui/src/render.ts:255-257` (any rush **or** storm + `canAttack` gets yellow **and** green) | in progress (round 3) | 
| `.engage-ready` yellow `engagePulse` 1s on an amulet that can Engage (code-read) | `css/cards.css:687-705`; `src/ui/zones/viewModel.ts:187-206`; `src/ui/zones/dom.ts:60-63` | `ui/css/cards.css:723-741`; `ui/src/render.ts:254` | ported | 
| `.fuse-ready` gold ring on a fuse-capable hand card (code-read) | — (no on-card fuse chrome; fuse is click/tooltip only) | `ui/css/client.css:96-98`; `ui/src/render.ts:195` | in progress (round 3) | Fuse *chip* (not just the ring) is the round-3 item. |
| `.selectable` dashed green (mulligan) / yellow (targets); `.selected` solid green (code-read) | `css/buttons.css:325-328`; `css/cards.css:739-778`; `src/ui/zones/dom.ts:56-57` | `ui/css/buttons.css:325-328`; `ui/css/cards.css:797-815,1130-1133` | ported | 
| Selected ✓ checkmark overlay (top-right, `#2ecc71`) (code-read) | `src/ui/zones/dom.ts:123-137` | — | missing | When a card is `.selected` (mulligan or multi-target), append a pointer-events-none ✓ at top-right, 24px extrabold `#2ecc71` with the old text-shadow. |
| `body.select-mode`: non-targets 50% + greyscale 0.55 (code-read) | `css/cards.css:732-737`; `css/modern-theme.css:767-769`; `src/ui/render.ts:168-172` | `ui/css/cards.css:767-815`; `ui/css/modern-theme.css:758-778`; `ui/src/render.ts:65` | ported | 
| Ward overlay (board): white shield SVG, `ward-pulse` 2s (code-read) | `src/ui/overlays.ts:27-31`; `css/cards.css` ward block | `ui/src/render/card.ts:219-223`; `ui/css/cards.css:230-271` | ported | 
| Ambush overlay (board): smoke SVGs 4s/5s (code-read) | `src/ui/overlays.ts:32-36` | `ui/src/render/card.ts:224-228`; `ui/css/cards.css:817-904` | ported | 
| Aura overlay (board): purple field + dashed ring 2.2s/6s (code-read) | `src/ui/overlays.ts:37-51` | `ui/src/render/card.ts:229-233`; `ui/css/cards.css:970-1042` | ported | 
| Intimidate overlay (board): amber diamond, `intimidate-flicker` 1.2s (code-read) | `src/ui/overlays.ts:62-66` | `ui/src/render/card.ts:234-238`; `ui/css/cards.css:273-308` | ported | 
| Can’t-attack overlay (board): crossed chains when the follower cannot attack (code-read) | `src/ui/overlays.ts:52-61` (`hasCantAttack` **or** `cantAttack` / `cantAttackUntilOpponentEOT`) | `ui/src/render/card.ts:239-243` (only if **both** `cantAttackFollowers` **and** `cantAttackLeader`) | partial | Show `.cant_attack-overlay` when the follower cannot attack *at all* (either “can’t attack” flag, or both follower+leader locks). Do not require both traits if the card is simply locked. |
| Barrier overlay + 250ms flash / 350ms pop (code-read) | `src/ui/overlays.ts:118-138`; `css/cards.css:443-487` | `ui/css/cards.css:310-501` (CSS only) | missing | On a follower with Barrier, mount `.barrier-overlay` (cyan ring + particles). Add `.barrier-flash` 250ms on a new charge and `.barrier-pop` 350ms when it breaks. |
| Can’t-be-destroyed overlay + 5 gold particles (code-read) | `src/ui/overlays.ts:88-109`; `css/cards.css` CBD block | `ui/css/cards.css:1135-1293` (CSS only) | missing | When the follower can’t be destroyed, mount `.cant-be-destroyed-overlay` and five `.cant-be-destroyed-particle` children. |
| Bane / Drain / Last Words PNG icons, bottom-centre 26×26 stack (code-read) | `src/ui/overlays.ts:67-83`; `images/icon_*.png` | `ui/src/render/card.ts:244-258`; `ui/public/images/icon_*.png` | ported | 
| Ongoing icon `images/icon_ongoing.png` (code-read) | `src/ui/overlays.ts:84-86` | — (`.ongoing-icon` CSS only, `ui/css/cards.css:504`) | missing | If the card shows an Ongoing effect, push `images/icon_ongoing.png` with class `ongoing-icon` into `.keyword-icon-stack`. |
| Icon stack swap animation `.swap-2/3/4` (2s/3s cycle) when 2+ icons (code-read) | `src/ui/overlays.ts:111-115`; `css/cards.css:1008-1090` | `ui/css/cards.css:1045-1126` (CSS only; JS never sets the class) | missing | When 2 / 3 / 4+ keyword icons share the stack, they cycle (old `.swap-2` 2s, `.swap-3`/`.swap-4` 3s keyframes). A single icon stays static. |
| Spellboost badge (blue circle, white count) under the cost (code-read) | `src/ui/zones/dom.ts:207-216`; `css/cards.css:871-897` | `ui/css/cards.css:907-933` (CSS only); `inst.spellboost_count` unused | missing | When `spellboost_count > 0`, render `.spellboost-badge` with the integer under the cost chip. |
| Countdown number, large white + black stroke, bottom-right (amulets) (code-read) | `src/ui/zones/dom.ts:172-176`; `css/cards.css:708-724` | `ui/src/render/card.ts:197-202`; `ui/css/cards.css:744-760` | ported | Refresh on `updateCard` is round 3. |
| Amulet named-counter fallback (first numeric `counters` entry as the same badge) (code-read) | `src/ui/zones/dom.ts:184-199` | — | missing | If there is no countdown but `counters` has a numeric entry, show that value in `.countdown-badge`. |
| Icarus `!` gold 18px badge (top-left of art) (code-read) | `src/ui/zones/dom.ts:83-104` | — | missing | When the instance is Icarus-buffed, draw an 18px gold circle `!` at `top:28px; left:6px`. |
| Face-down hand: `.card.card-back`, striped back, no uid/name leak, `pointer-events: none` (code-read) | `src/ui/zones/dom.ts:28-39`; `css/cards.css:102-142` | `ui/src/render/card.ts:92-105`; `ui/css/cards.css:101-142` | ported | Used for vs-bot hide-hand (not sparring-line). |
| Art fallback: name + cost/stats if the image 404s (code-read) | old uses CDN then `.webp` (`src/ui/render.ts:626-646` for crests) | `ui/src/images.ts:28-52`; `ui/css/client.css:49-70` | ported | 
| UNIMPL / UNKNOWN OP outline + face badge (code-read) | `src/ui/zones/dom.ts:106-119`; `css/modern-theme.css:1028-1048` | `ui/css/modern-theme.css:1019-1039` (CSS only) | dropped (by design) | Coverage badges; same family as the coverage banner. |
| Placeholder / unimplemented card frame (code-read) | `css/cards.css:92-99` | unused | dropped (by design) | 

---

## 3. Leader panel

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| HP number in the 32px pill on the attack strip (default “20”) | `index.html:642,666`; `src/ui/render.ts:76-77`; `css/phase3-layout.css:487-491` | `ui/index.html:244-257`; `ui/src/render.ts:73-74` | ported | 
| No class portrait / leader art — HP pill only | (none) | (none; `bannerImageUrl` unused) | ported | 
| EP / SEP as **button labels** `Evo (N)` / `Super (N)`, not orbs | `src/ui/evo.ts:18-21` | `ui/src/render.ts:414-415` | ported | Tokens `--color-ep-*` exist on both sides and are unused. |
| Evo / Super disabled when off-turn, no charges, already evo’d this turn, or round-locked (red N≥4 / blue N≥5 / red SE≥6 / blue SE≥7) (code-read) | `src/ui/evo.ts:23-54` | `ui/src/render.ts:412-415` (`disabled` when `legal` has no evolve) | ported | Engine `legal` replaces the old round table; the player still sees a grey button until evolve is legal. |
| `.normal-evo` gold / `.super-evo` purple; disabled grey `--color-ep-used` opacity 0.55 | `css/buttons.css:71-93` | `ui/css/buttons.css:71-93` | ported | 
| Drag Evo/Super onto an unevolved allied follower (code-read) | `src/ui/drag.ts:146-206`; `src/ui/evo.ts:56-59` | `ui/src/input.ts:121-131`; `ui/src/render.ts:316-317,422-433` | ported | 
| Click Evo/Super does not count as a leader attack (code-read) | `src/ui/evo.ts:61-66` | `ui/css/phase3-layout.css:377-403`; `ui/src/input.ts:56-61` (pending evolve) | ported | 
| PP box `PP: current/max` (usable includes pending Bonus PP) | `index.html:729-745`; `src/ui/render.ts:78-79` | `ui/src/render.ts:71-76`; `ui/src/info.ts:12-14` | ported | 
| Bonus PP `+1 PP Boost` on the **second** player; early `1–5` / late `6+` pips; `.used` strikethrough; `.active-tier` warn glow | `index.html:708-728`; `src/ui/render.ts:243-286`; `css/modern-theme.css:846-870,1106-1141` | `ui/index.html:301-307`; `ui/src/render.ts:502-526`; `ui/src/main.ts:680-688` | ported | 
| Crests: red 3+2 slots, blue 2+3; empty dashed ring; filled green glow; circular art | `index.html:693-702,890-899`; `src/ui/render.ts:614-651`; `css/modern-theme.css:662-698` | `ui/index.html:275-321`; `ui/src/render/card.ts:261-281` | ported | 
| Crest countdown badge bottom-right (code-read) | `src/ui/render.ts:654-660`; `css/layout.css:480-497` | `ui/src/render/card.ts:282-287`; `ui/css/layout.css:482-504` | ported | 
| Faith value badge **top-left** on the Faith crest (same `.crest-countdown` class, repositioned) (code-read) | `src/ui/render.ts:663-673` | — | in progress (round 3) | 
| 🃏 hand / 📦 deck / 💀 cemetery / ☠️ shadows, both sides, always public | `index.html:688-691,902-905`; `src/ui/counts.ts:25-33` | `ui/index.html:270-327`; `ui/src/render.ts:77-84` | ported | 
| No combo / earth / rally / faith / banished on the rail (code-read) | (not shown) | (not shown) | ported | 
| Leader Barrier cyan ring on the HP host (`.has-leader-barrier`) (code-read) | `src/ui/render.ts:704-721`; `css/modern-theme.css:911-929` | `ui/src/render.ts:397-398` toggles `has-barrier`; CSS expects `.has-leader-barrier` (`ui/css/modern-theme.css:896-920`) | partial | Toggle `has-leader-barrier` on the HP host (or change the CSS selector). Show the cyan `--glow-barrier` ring while a damage-cap / barrier mod is live. |
| Enemy leader `.selectable` red pulse while it is a legal **pending** attack target (code-read) | `src/ui/render.ts:123-146`; `css/buttons.css:149-203` | `ui/src/render.ts:373-396` also outlines whenever *any* leader attack is legal | in progress (round 3) | 

---

## 4. Tooltips

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Single `#cardTooltip`, dark panel, `pointer-events: none`, z 9999, max ~360px | `index.html:940-957`; `css/tooltip.css:3-9` | `ui/index.html:333`; `ui/css/tooltip.css:3-8`; `ui/css/modern-theme.css:748-755` | ported | 
| Order: **name** → class/tribes + set → **counter block** → description → crest panels → fused loot → Rally line → Skybound line → buff delta (code-read) | `src/ui/tooltips.ts:169-176` | `ui/src/tooltip.ts:42-47` (name → cost/ATK/DEF/kind → description → gates) | partial | Restore old order after the name: class/tribes + set, then progress/gate lines, then formatted description, then extras (fused / Rally / Skybound / `+A/+D`). Keep the new Cost line (paid + grey base). Round-3 gate generalisation covers the progress lines themselves. |
| Name `.tooltip-header-name` lg bold | `src/ui/tooltips.ts:170`; `css/tooltip.css` | `ui/src/tooltip.ts:43`; `ui/css/tooltip.css:11-15` | ported | 
| Meta: `Class` or `Class/Tribe1, Tribe2`; set badge `#9aa3b2` in-rotation / `#7a8494` older (code-read) | `src/ui/tooltips.ts:98-99,34-47` | — (meta is `Cost N · A/D · kind`) | missing | After the name, a muted meta line `Class` or `Class/Tribes` plus an optional set label in those two greys. |
| Cost line with strikethrough / grey base when paid ≠ printed | (cost lives on the card, not in the tooltip) | `ui/src/tooltip.ts:21-28` | ported | New-only extra from R1; keep it. |
| Spell/amulet tooltips omit `0/0` | n/a (no ATK/DEF in old tooltip) | `ui/src/tooltip.ts:31-38` (R2) | ported | 
| Keyword lines bold `--color-warn` (`Fanfare:`, `Enhance (N):`, `Accelerate (N):`, …) | `src/ui/tooltipFormat.ts:11-12,86-93` | `ui/src/tooltip.ts:6-10,83-91` | ported | 
| Card-name highlights italic blue inside description (code-read) | `src/ui/tooltipFormat.ts:69-79` | — | missing | Italic/blue known card names inside tooltip description text. |
| “Gain crest:” muted lead + crest icon/name/text panels after the description (code-read) | `src/ui/tooltipFormat.ts:96-97,117-147`; `src/ui/tooltips.ts:104-107` | — | missing | After the description, for each referenced crest: icon + name + formatted text (`.tooltip-crest-panel`). |
| Dynamic counter block **before** the description: `(Label: value/threshold)` (code-read) | `src/ui/tooltips.ts:101-102,172`; `src/ui/tooltipCounters.ts:407-411` | `ui/src/tooltip.ts:100-111` (gates **after** description) | in progress (round 3) | 
| Enhance / Necromancy / Combo / Rally / Earth Rite / Overflow / Spellboost / Accelerate / Crystallize progress lines (code-read) | `src/ui/tooltipCounters.ts:72-98`; `src/ui/info`-equivalent in counters | `ui/src/info.ts:22-43` | in progress (round 3) | 
| Dedicated blue `Rally: current / req` line (code-read) | `src/ui/tooltips.ts:125-141` | Rally only if a `rally` gate exists (`ui/src/info.ts:33-34`) | partial | If the printed text has Rally but the engine sent no gate, still show `Rally: have / need` in `#7af` after the description. |
| Dedicated gold `Skybound Art: current / req` (code-read) | `src/ui/tooltips.ts:143-162` | — | missing | When the card has Skybound Art, show a gold `.skybound-line`: `Skybound Art: (turn + witnessed) / req`. |
| Orange `Fused Loot (unique): N` + names, or grey `Fused: N cards` (code-read) | `src/ui/tooltips.ts:109-123` | — | missing | If the instance is fused, append those lines (orange unique names, else grey count). |
| Buff delta `+A/+D` green if non-negative, `#ff6666` if any negative (code-read) | `src/ui/tooltips.ts:19-31,164-176` | `ui/css/tooltip.css:116-126` (`.buff-delta` unused) | missing | For followers with a non-zero buff pair, append `+A/+D` in `#66ff66` / `#ff6666`. |
| Red `Cannot play: {reason}` when the hand card is blocked (code-read) | `src/ui/tooltips.ts:265-269`; `src/ui/zones/dom.ts:48-52` | — | missing | If the engine/handInfo says unplayable on the active turn, append a red `.tooltip-play-blocked` line `Cannot play: {reason}` (e.g. “Not enough PP.”). |
| Hover shows; mousemove repositions; mouseleave hides | `src/ui/tooltips.ts:330-361` | `ui/src/render.ts:789-814` | ported | 
| Smart vertical anchor: bottom half of the viewport grows **up**; top half grows **down**; 12px from cursor, clamped (code-read) | `src/ui/tooltips.ts:337-356` | `ui/src/render.ts:803-807` (always `+16px` down-right) | missing | Use the old bottom-half / top-half rule so a bottom-hand tooltip does not cover the card. |
| No large art in the tooltip (text only) | `src/ui/tooltips.ts` (no preview node) | R1 removed `#cardPreview` | ported | 
| Drag: pin the same tooltip at `(12px, 12px)` for the gesture (code-read) | `src/ui/tooltips.ts:199-215` | — | missing | While a pointer-drag of a card is active, show `formatCardTooltip` fixed at `top:12px; left:12px`; hide on release. |
| Live refresh on every `render()` (counters / gates stay current) (code-read) | `src/ui/render.ts:230`; `src/ui/tooltips.ts:278-318` | — (HTML built once on `mouseover`) | missing | After each paint, if a tooltip is open, rebuild it from the live instance + gates (drag session wins over hover). |
| Face-down cards never show a tooltip (code-read) | `src/ui/zones/index.ts:71-77` | `ui/src/render.ts:791` | ported | 
| Crest hover: name + description; Faith `Faith — {count}`; blue crests above cursor, red below (code-read) | `src/ui/render.ts:676-698` | `ui/src/tooltip.ts:50-58`; `ui/src/render.ts:458-469` | partial | Prefixed Faith line is round 3. Placement already matches. |
| Long-press (touch) opens the same tooltip (code-read) | — (hover + drag only) | — | in progress (round 3) | 

---

## 5. Interaction flows

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Mulligan: dashed green selectable hand cards; click toggles mark | `src/ui/zones/viewModel.ts:260-261`; `src/ui/zones/handlers.ts:27-35` | `ui/src/render.ts:188-189`; `ui/src/input.ts:77-81` | ported | 
| Mulligan selected: green ✓ + `.selected` (code-read) | `src/ui/zones/dom.ts:123-137` | `.selected` only (`ui/src/render.ts:188`) | partial | Add the ✓ overlay (see §2). |
| Mulligan marks + confirm on **both** hands (sequential acting player) (code-read) | per-side confirm `index.html:619-625,673-679`; each side’s cards marked independently | `ui/src/session.ts:50` one `mulliganSwap[4]`; confirm only for `acting` (`ui/src/render.ts:473-480`) | in progress (round 3) | 
| Confirm Mulligan (Red/Blue) in that side’s hand row | `index.html:619-625,673-679` | `ui/index.html:239-263`; `ui/src/render.ts:473-480` | ported | 
| No hand drag / no End Turn during mulligan (code-read) | `src/ui/zones/handlers.ts:104-105`; `src/ui/render.ts:200` | `ui/src/render.ts:208,489-494` | ported | 
| **Drag** a hand card onto the matching board zone to play (code-read) | `src/ui/drag.ts:115-143` | `ui/src/render.ts:201-208,271-285` | ported | 
| **Right-click** an active-turn hand card to play (code-read) | `src/ui/zones/handlers.ts:52-56` | — (right-click is fuse) | missing | On `contextmenu` of an active-turn playable hand card, prevent default and `play` that `handPos` (old year-long habit). |
| **Left-click / tap** a fuse-capable hand card to open Fuse (does not play) (code-read) | `src/ui/zones/handlers.ts:59-101`; `src/ui/zones/dragClickGuard.ts:50-56` | left-click **plays** (`ui/src/input.ts:83-85`); fuse is right-click (`109-115`) | missing | Left-click/tap on a `fuse-ready` hand card opens Fuse. Do not play on that click. Play remains drag and (if ported) right-click. |
| Fuse via releasing a drag **inside the hand** (no valid drop) (code-read) | `src/ui/pointerDragSession.ts:324-334` | — | missing | If a hand-card drag ends inside the same hand zone with no drop target, run the fuse gesture when the card is fuse-capable. |
| Off-turn hand: not draggable, no play (silent) (code-read) | `src/ui/zones/handlers.ts:104-105` | `ui/src/render.ts:208` (`playable && phase !== "mulligan"`) | ported | 
| 8px drag threshold; source opacity 0.45; preview clone scale 1.05 (code-read) | `src/ui/pointerDragSession.ts:38`; `css/cards.css:184-193` | `ui/src/drag.ts:14,89-105`; `ui/css/cards.css:193-202` | ported | 
| Drop highlight green `.pointer-drop-highlight` on **legal** targets only (code-read) | `src/ui/pointerDragSession.ts:148-157`; `css/cards.css:195-199` | `ui/src/drag.ts:78-86`; `ui/src/render.ts:306-312` (any enemy card / any allied evo drop) | in progress (round 3) | 
| Click-after-drag suppressed (~100ms) (code-read) | `src/ui/pointerDragSession.ts:235-243` | `ui/src/drag.ts:61-64,157-161` | ported | 
| Attack: drag own `canAttack` follower onto an enemy follower or the enemy leader strip (code-read) | `src/ui/drag.ts:26-66,98-112,209-256` | `ui/src/render.ts:258-265,286-318`; `ui/src/input.ts:103-105` | ported | 
| Attack: click attacker then click target (new two-step) (code-read) | — | `ui/src/input.ts:95-97,103-105`; `ui/src/render.ts:754-769` | ported | New-only extra; keep. Pending look/cancel is round 3. |
| Pending attack / evolve: highlight legal targets; Cancel chip; tap elsewhere cancels (code-read) | old targeting is engine-driven select-mode; no pending-attack chip | `ui/src/input.ts:64-70` (empty click does **not** cancel); Escape does (`117-119`) | in progress (round 3) | 
| Evolve: click Evo/Super then click an allied follower (code-read) | click is a no-op (drag only) (`src/ui/evo.ts:61-66`) | `ui/src/input.ts:56-61,91-93` | ported | New-only extra. |
| Engage: **right-click** an `.engage-ready` amulet (code-read) | `src/ui/zones/handlers.ts:137-148` | **left-click** (`ui/src/input.ts:99-101`) | partial | Keep left-click if you want, but right-click on `.engage-ready` must also `engage` (and must not open the browser menu). |
| End Turn (Blue/Red) visible only for the active player; hidden in mulligan / game over | `src/ui/counts.ts:46-49`; `src/ui/render.ts:393-418` | `ui/src/render.ts:489-498` | ported | 
| Bonus PP click toggles / commits on the second player | `index.html:708-728` | `ui/src/main.ts:680-688` | ported | 
| Choice / mode modal: dim overlay, title **“Choose an effect:”**, buttons `label` or `name`, yellow Earth Rite sub-line (code-read) | `src/ui/choiceModal.ts:2-38`; `css/buttons.css:96-145` | `ui/src/render.ts:571-584` title **“Choose”**; `Mode ${m}` 0-based; no Earth Rite line | in progress (round 3) | 
| Choice buttons: card names, 1-based mode text, no dead buttons (code-read) | `src/ui/choiceModal.ts:16-18` | `ui/src/render.ts:650-668` | in progress (round 3) | 
| Stale choice click listeners on reused cards (code-read) | n/a (modal torn down) | `ui/src/render.ts:604-612` `{ once: true }` on highlighted cards | in progress (round 3) | 
| Fuse Confirm sits under the modal (code-read) | fuse UI is engine-driven | confirm is `#targetingConfirmation` “Confirm” (`ui/src/render.ts:562-566`) | in progress (round 3) | 
| Multi-target confirm bar: `{text} ({count})` bottom-centre green (code-read) | `src/ui/targeting.ts:16-28`; `index.html:152-163` | `ui/src/render.ts:562-566` label **“Confirm”** only | partial | Button text `{prompt} ({selectedCount})` using `--color-success` (`.confirm-targets-btn`). |
| Modal `.processing` then remove on click (code-read) | `src/ui/choiceModal.ts:29-33` | — | missing | On option click, add `.processing` and remove the modal before dispatching. |
| Choice-phase prompt + Undo chip (code-read) | — | — | in progress (round 3) | 
| Undo symmetry inside a choice (one undo = one pick, not the whole choice) (code-read) | engine history (not in UI files) | `ui/src/session.ts:206-212` undoes the entire choice | in progress (round 3) | 
| Vs-bot hide opponent hand (face-down) (code-read) | sparring “Hide line hand” (`src/ui/scriptPanel.ts:74-86`) — dropped | `ui/index.html:143-146`; `ui/src/render.ts:101-102,186-189` | dropped (by design) | Hide-hand survives as vs-bot only; sparring-line hide is out of scope. |

---

## 6. Feedback & animation

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| FCT toggle in settings, default on, persist `svwb.floatingCombatText` (code-read) | `index.html:366-371`; `src/ui/floatingCombatText.ts:13-56`; `src/boot/floatingCombatText.ts` | `ui/index.html:185-188`; `ui/src/main.ts:193-196` (checkbox only, not persisted) | partial | Persist `localStorage.svwb.floatingCombatText` (`"1"`/`"0"`) and restore the checkbox on boot. |
| FCT `−N` red `--color-danger`, `+N` green `--color-success`, 1.35s rise, 130ms stagger (code-read) | `css/floating-combat-text.css:5-61`; `src/ui/floatingCombatText.ts:16-17,136-139` | `ui/css/floating-combat-text.css:10-38`; `ui/src/fct.ts:4,13-35` | ported | 
| Max 4 floaters per target; stack `--float-stack-index` (code-read) | `src/ui/floatingCombatText.ts:17,130` | stack index always 0; no cap | missing | Cap 4 per host; increment `--float-stack-index` (18px) per live floater. |
| Leader FCT from the leader strip centre (code-read) | `src/ui/floatingCombatText.ts:62-66,203-245` | `ui/src/fct.ts:43-45` (`#blueLeader` / `#redLeader`) | ported | 
| Follower FCT on the card (code-read) | `src/ui/floatingCombatText.ts:68-70,147-151` | `ui/src/fct.ts:46-48` looks up `#blue-board-{slot}` but cards are `#blue-board-{id}` | in progress (round 3) | 
| Follower brightness flash on damage (`.floating-combat-flash`) (code-read) | `src/ui/floatingCombatText.ts:147-151`; `css/floating-combat-text.css:89-103` | CSS only (`ui/css/floating-combat-text.css:89-103`) | missing | On follower damage, add `.floating-combat-flash` for the CSS flash duration. |
| Suppress FCT on undo/redo / position load (code-read) | `src/ui/floatingCombatText.ts:304-314` | `ui/src/session.ts:175,193`; `ui/src/main.ts:180-183` | ported | 
| No enter/leave/destroy CSS on old zone reconcile (code-read) | `src/ui/zones/index.ts:83-147` | `.card-enter` 140ms / `.card-leave` 120ms (`ui/css/animation.css:3-48`; `ui/src/render.ts:349-357`) | ported | New-only extra; keep. |
| `.dying` 0.6s fade/scale (class rarely applied) (code-read) | `css/cards.css:150-156` | `ui/css/cards.css:159-165` (unused; leave uses `card-leave`) | ported | Same unused CSS. |
| No attack lunge / slash (code-read) | — | — | ported | 
| `.spell-cast` 0.5s brightness/scale (code-read) | `css/animation.css:1-14` | `ui/css/animation.css` has enter/leave/press, no `.spell-cast` | missing | If a spell is played, add `.spell-cast` on the card/board for 0.5s (old `animation.css`). |
| Stat-change flash 150–160ms scale/brightness (code-read) | — (colour change only) | `ui/css/animation.css:24-58`; `ui/src/render/card.ts:142-159` | ported | New-only extra; keep. |
| Pressed card `translateY(2px) scale(0.97)` (code-read) | — | `ui/css/animation.css:11-15`; `ui/src/input.ts:31-45` | ported | New-only extra; keep. |
| No “Turn N” banner; turn change is the hand/leader glow + End Turn swap (code-read) | (none) | (none) | ported | 
| Terminal overlay: “First wins” / “Second wins” / “Draw”; reason “Deck-out” or “Lethal”; Rematch same seed / new seed (code-read) | `src/ui/render.ts:289-342` | `ui/src/render.ts:707-738` “Blue (A) wins” / “Red (B) wins” / “Draw”; reason **“Match over”**; **New Game** + **Rematch (swap sides)** | partial | Title may stay Blue/Red. Set `#gameOverReason` to `Deck-out` or `Lethal` from the engine winner reason. Keep New Game + swap-sides if desired; also offer Rematch (same seed) like the old buttons. |
| Rail terminal readout label (code-read) | — | `ui/src/render.ts:94-98` (`Winner a` or `Winner b`) | in progress (round 3) | 
| Action toast bottom-centre, 0.18s fade, default 1800ms (code-read) | `src/ui/toast.ts:19-27`; `css/modern-theme.css:976-998` | CSS `#actionToast` unused; errors go to `#errBanner` inside the settings drawer (`ui/src/main.ts:198-201`) | in progress (round 3) | Engine-error toast is round 3; the visual is the old `#actionToast` (not a drawer line). |
| Board/leaders 72% opacity, no pointer events on game over (code-read) | `css/modern-theme.css:1050-1104` | `ui/css/modern-theme.css:961-965` | ported | 

---

## 7. History / log drawer

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Left battle-log button; drawer 360px, `transform 0.18s`; scrim click closes | `index.html:557-611,968-1002` | `ui/index.html:207-233`; `ui/src/main.ts:463-484` | ported | 
| Opening history closes settings (and vice versa) (code-read) | `index.html:969`; `src/boot/settingsDrawer.ts:41` | `ui/src/main.ts:443,471` | ported | 
| Escape closes the history drawer (code-read) | `index.html:988-990` | — (Escape only closes settings / cancels pending) | missing | On Escape, if `#historyDrawer` is open, close it (same as settings). |
| Sections: Red — Played, Red — Destroyed, Blue — Played, Blue — Destroyed | `index.html:590-611` | `ui/index.html:216-232` | ported | 
| Each row: cost badge + `Name ×count`, grouped by name+cost, sorted cost then name (code-read) | `src/ui/render.ts:451-507` | `ui/src/render.ts:535-551` (flat name list, no cost, no count) | missing | Group by name+cost; show `.cost-badge` + `Name ×N`; sort by cost then name. Use the old inline `.cost-badge` style from `index.html`. |
| Optional set badge on the row (muted `#7a8494` if out of rotation) (code-read) | `src/ui/render.ts:494-503` | — | missing | After the name, a `.hist-set` span; `data-older="1"` when out of rotation. |
| Hover a row: 198px card-art preview follows the cursor (+18px, clamped) (code-read) | `src/ui/render.ts:534-580`; `index.html:144-147` | — | missing | `#historyImgPreview` with `img` width 198px; show on `.hist-item` mouseover using `data-img`. |
| Rows are not clickable (hover only) (code-read) | (hover only) | (not clickable) | ported | 
| Destroyed list is **that side’s lost cards**, not “destroyed by” (code-read) | `src/ui/render.ts:205-211` (`players.*.destroyedHistory`) | `ui/src/session.ts:144-147` attributes destroy to `game.active()` | in progress (round 3) | 

---

## 8. Settings drawer and top bar

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Hamburger; title “Settings (Ctrl+Shift+M)” | `index.html:166-173` | `ui/index.html:82` | ported | 
| Drawer 360px left slide 0.18s; scrim click / Escape / Ctrl+Shift+M | `src/boot/settingsDrawer.ts:70-88`; `css/settings-drawer.css` | `ui/src/main.ts:438-460`; `ui/css/settings-drawer.css:50-68` | ported | 
| Blue / Red deck `<select>` | `index.html:196-207` | `ui/index.html:99-107` | ported | 
| Import Deck | `index.html:210-216` (paste community list) | `ui/index.html:109-110`; `ui/src/main.ts:611-624` (JSON `{id:count}` file) | partial | Keep JSON import. If you also want old paste (`Nx Name`), that is a separate panel — treat as dropped unless the owner wants it. |
| Export List (Blue deck as `Nx Name` + clipboard) (code-read) | `index.html:217-223`; `src/ui/deckImportPanel.ts:363-365` | — | missing | Button “Export List” dumps the Blue deck as paste text into the clipboard (and a small panel). |
| Seed input, “random if empty” | `index.html:231-240` | `ui/index.html:112-122` placeholder/value **`1`** | partial | Empty seed = roll a random u64 (old placeholder). Defaulting to `1` every boot is a habit break. |
| Game seed panel + Copy; button flashes “Copied” 1200ms | `src/ui/seedDisplay.ts:13-61` | `ui/index.html:123-126`; `ui/src/main.ts:701-707` (copy, no flash) | partial | After a successful copy, set the button label to `Copied` for 1200ms. |
| First-player control | (implicit / start options, not a drawer select) | `ui/index.html:128-133` coin/A/B | ported | New-only extra. |
| Mode: hotseat / vs-bot / watch | — | `ui/index.html:93-98` | ported | New-only extra. |
| Vs-bot: human side, bot policy, hide bot hand (code-read) | — | `ui/index.html:134-146` | ported | New-only extra. |
| Watch: Bot A/B policies, auto-play, Step/Play/Pause, speed 1–20 (code-read) | — | `ui/index.html:148-156,197-205`; `ui/src/main.ts:325-361,627-647` | ported | New-only extra. |
| Start Game | `index.html:259` | `ui/index.html:158` | ported | 
| Undo / Redo buttons; disabled from stack; titles Ctrl+Z / Ctrl+Y | `index.html:266-275` | `ui/index.html:159-160`; `ui/src/render.ts:747-751` | ported | 
| Share URL write on start: `?seed=&a=&b=` (code-read) | `src/boot/shareUrl.ts:4-57` | `ui/src/share.ts:26-38` `?seed=&deckA=&deckB=&mode=`; **reads** `a`/`b` aliases | partial | Keep `deckA`/`deckB`/`mode`. Also write `a`/`b` (or accept both forever). Optionally encode `first` / human side. Auto-start if seed+decks present (`ui/src/main.ts:726-729`) already matches. |
| Consistency Trainer link (code-read) | `index.html:260-265` | — | dropped (by design) | Separate tool, not the M4 client. |
| God Mode checkbox + PP/EP/combo/shadows cheats (code-read) | `index.html:250-257,747-886`; `src/boot/godMode.ts` | CSS leftovers only | dropped (by design) | 
| Save Pos (prompt name) | `src/ui/positionPanel.ts:106-134` | `ui/src/main.ts:536-541` | ported | Snapshot format differs (action log vs board); see save/load row. |
| Position `<select>` `Name · T{n} · time` (code-read) | `src/ui/positionPanel.ts:49-51` | `ui/src/main.ts:603-607` (name only) | missing | Option text `{name} · T{turn} · {localeTime}`. |
| Load / Export / Import JSON (code-read) | `src/ui/positionPanel.ts:137-245` | `ui/src/main.ts:543-576` | ported | 
| Rename + Del (code-read) | `index.html:305-317` | — | missing | Rename (prompt) and Del for the selected in-memory position. |
| Checkpoint button + F6 | `index.html:334`; `src/boot/hotkeysBoot.ts:7-9` | `ui/index.html:178`; `ui/src/main.ts:513-518,577-581` | ported | 
| Restore CP | `index.html:338-343` | `ui/index.html:179`; `ui/src/main.ts:520-525,582-587` | ported | 
| Restore CP key **F7** | — (button only) | `ui/src/main.ts:520-525` | ported | New-only extra. |
| Reroll + **F8** (new RNG branch from the checkpoint) (code-read) | `index.html:344-350`; `src/boot/hotkeysBoot.ts:7-9` | — | missing | Button “Reroll” + F8: restore the checkpoint with a new seed branch; status `Checkpoint: T{n} · rerolls N`. |
| Checkpoint status `Checkpoint: none` or `T{n} · rerolls N` (code-read) | `index.html:351-356` | `ui/index.html:180` `Checkpoint: none/set` | partial | Show turn + reroll depth, not just “set”. |
| Save/load still correct after the 200-step undo ring drops old actions (code-read) | old position store is a full board snapshot | `ui/src/session.ts:11-12,106-111`; `ui/README.md:38` (log of `NeutralAction[]`) | in progress (round 3) | 
| Active on bottom checkbox | `index.html:357-364` | `ui/index.html:181-184` | partial | Persistence — see §1. |
| Floating combat text checkbox (default on) | `index.html:366-371` | `ui/index.html:185-188` | partial | Persistence — see §6. |
| Diagnostics / blackbox (Copy Trace, Export Trace, heap warning) (code-read) | `index.html:375-411`; `src/ui/blackbox.ts` | — | dropped (by design) | 
| Line / sparring script (Rec, Export/Import, Hide line hand, diverge banner) (code-read) | `index.html:413-465`; `src/ui/scriptPanel.ts` | CSS `#scriptDivergeBanner` only | dropped (by design) | 
| Puzzle section (save/play/retry, result overlay) (code-read) | `index.html:466-535`; `src/ui/puzzlePanel.ts` | CSS leftovers only | dropped (by design) | 
| Coverage banner after Start Game (code-read) | `src/ui/coverageBanner.ts:45-104` | CSS `#coverageBanner` only | dropped (by design) | 
| No theme / scale user toggle (fixed dark theme; layout scales) | (none) | (none) | ported | 
| Bundle meta line (`arena {version} · {N} cards`) | — | `ui/index.html:191`; `ui/src/main.ts:718-724` | ported | New-only extra. |

---

## 9. Keyboard & pointer

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Ctrl+Shift+M toggles settings (code-read) | `src/boot/settingsDrawer.ts:86-88` | `ui/src/main.ts:456-459` | ported | 
| Escape closes settings (code-read) | `src/boot/settingsDrawer.ts:81-83` | `ui/src/main.ts:455` | ported | 
| Escape closes history (code-read) | `index.html:988` | — | missing | See §7. |
| Escape / click-outside does **not** cancel old targeting (no UI cancel) (code-read) | (none in UI files) | Escape cancels pending (`ui/src/input.ts:117-119`) | ported | New-only extra; tap-elsewhere + Cancel chip is round 3. |
| Ctrl/Cmd+Z undo; Ctrl+Y / Ctrl+Shift+Z redo (ignore when focus is an input) (code-read) | `index.html:266-275`; `src/boot/hotkeysBoot.ts:4-5` | `ui/src/main.ts:486-511` | ported | 
| F6 set checkpoint (code-read) | `src/boot/hotkeysBoot.ts:7-9` | `ui/src/main.ts:513-518` | ported | 
| F7 restore checkpoint (code-read) | — | `ui/src/main.ts:520-525` | ported | New-only extra. |
| F8 reroll checkpoint (code-read) | `src/boot/hotkeysBoot.ts:7-9` | — | missing | See §8. |
| Unified pointer drag (mouse + touch), pointer capture (code-read) | `src/ui/pointerDragSession.ts` | `ui/src/drag.ts` | ported | 
| `touch-action: none` on draggables (code-read) | `css/cards.css` drag block | `ui/css/cards.css:175-184` | in progress (round 3) | 
| Browser context menu blocked on `.card`, `.zone`, `.leader`, `.evo-btn` (code-read) | `src/boot/contextMenu.ts:3-17` | only `fuse-ready` cards (`ui/src/input.ts:109-115`) | missing | `contextmenu` preventDefault on `.card`, `.zone`, `.leader`, `.evo-btn` (capture). Right-click play / engage / fuse then work without the OS menu. |
| Long-press tooltip (code-read) | — | — | in progress (round 3) | 
| Cancel drag on `pointercancel`, tab blur, `visibilitychange` (code-read) | `src/ui/pointerDragSession.ts:341-367` | `ui/src/drag.ts:284-287` | ported | 

---

## 10. Performance (user-noticeable)

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Zone reconcile by `data-uid`; reuse DOM when the fingerprint matches (code-read) | `src/ui/zones/index.ts:83-147`; `src/ui/zones/memoization.ts:165-225` | `ui/src/render.ts:336-371`; `ui/src/render/card.ts:74-87` | ported | 
| No full-page re-render of unrelated chrome (code-read) | `src/ui/render.ts` updates lists/crests in place | `ui/src/render.ts:535-540` history signature cache | ported | 
| rAF paint coalescing (code-read) | — (sync `render()` on every change, `src/ui/render.ts:70`) | `ui/src/main.ts:148-154` | ported | New-only extra; keep. `window.__arena.paintMs` budget is tested (`ui/tests/feedback.spec.ts:378-387`). |
| CSS-driven FCT (no JS animation loop) (code-read) | `css/floating-combat-text.css:2-3` | `ui/css/floating-combat-text.css:1-2` | ported | 
| Cancel in-flight image loads before tearing down a node (code-read) | `src/ui/releaseImageLoads.ts:13-35` | — | missing | Before `innerHTML = ""` / removing a card or crest, clear `img.src` and load handlers so stale decodes do not hitch the next paint. |
| Catalog / deck JSON cached at boot; images use the browser cache (`referrerPolicy: no-referrer`) (code-read) | CDN + `.webp` fallback | `ui/src/catalog.ts:6-27`; `ui/src/images.ts:25-52` | ported | 
| No explicit image preload queue (code-read) | (none) | (none) | ported | 

---

## Gap list

Partial and missing rows only, **every-turn first**. Round-3 items stay here so the next brief can skip them, but they are not re-specified.

### Every turn

1. **Hand/board glow only for the acting player** — in progress (round 3).
2. **Yellow vs green glow + no E/A/C badges** — in progress (round 3).
3. **Stat colours buffed / damaged / debuffed** — in progress (round 3).
4. **`updateCard` countdown / overlays / evolved art** — in progress (round 3).
5. **Tooltip order + class/tribes/set + extras** — After the name: class/tribes + set; then gate/progress lines; then keyword-formatted description; then crest panels, fused loot, Rally, Skybound, `+A/+D`, `Cannot play:`. Keep the Cost line. Smart-anchor: bottom half of the viewport grows up (`src/ui/tooltips.ts:337-356`).
6. **Play / fuse click map** — Left-click/tap fuse-capable hand card → Fuse. Right-click playable hand card → play. Drag to own board → play. (Today the new client is the inverse: left-click plays, right-click fuses.)
7. **Rush-turn yellow vs green attack glow** — in progress (round 3).
8. **Pending attack/evolve cancel + Cancel chip** — in progress (round 3).
9. **Long-press tooltip + `touch-action: none`** — in progress (round 3).
10. **Tooltip live refresh + drag-pinned panel** — Rebuild an open tooltip on every paint; while dragging a card, pin it at `(12px, 12px)`.

### Common (most games)

11. **Follower floating combat text** — in progress (round 3). Also: cap 4 per host, `--float-stack-index` * 18px, `.floating-combat-flash` on follower damage, persist the FCT checkbox as `svwb.floatingCombatText`.
12. **Mulligan marks on both hands + ✓** — in progress (round 3). Still add the old ✓ overlay on `.selected`.
13. **History rows** — Cost badge + `Name ×count`, group by name+cost, set badge, 198px hover art (`src/ui/render.ts:451-580`).
14. **History destroyed-owner attribution** — in progress (round 3).
15. **Choice modal labels + Fuse Confirm + stale listeners + prompt/Undo** — in progress (round 3). Also: title “Choose an effect:”; Earth Rite sub-line in `--color-warn`; confirm `{text} ({count})`.
16. **Engine-error toast** — in progress (round 3). Use `#actionToast` (bottom-centre, 1800ms, 0.18s fade), not `#errBanner` in the drawer.
17. **Undo symmetry inside a choice** — in progress (round 3).
18. **Enemy leader outline only while pending** — in progress (round 3).
19. **Drag highlights ignore legality** — in progress (round 3).
20. **Fuse chip** — in progress (round 3).
21. **Engage right-click** — `contextmenu` on `.engage-ready` calls `engage` (and suppress the OS menu on `.card`/`.zone`/`.leader`/`.evo-btn`).
22. **Can’t-attack overlay** — Show chains when the follower cannot attack, not only when both `cantAttackFollowers` and `cantAttackLeader` are set.
23. **Keyword icon swap + Ongoing icon** — 2+ icons cycle (`.swap-2` 2s / `.swap-3`/`.swap-4` 3s); Ongoing shows `images/icon_ongoing.png` in the same bottom-centre stack.
24. **Spellboost badge** — Blue `.spellboost-badge` with `spellboost_count` under the cost when `> 0`.
25. **Leader barrier class** — Toggle `has-leader-barrier` so the cyan ring CSS applies.
26. **Escape closes history** — Same as the old drawer (`index.html:988`).
27. **Copy seed “Copied”** — 1200ms label flash (`src/ui/seedDisplay.ts:42-47`).
28. **Active-on-bottom persist** — `localStorage.svwb.activeOnBottom`.

### Occasional

29. **Faith crest badge** — in progress (round 3).
30. **Barrier overlay + flash/pop** — Mount `.barrier-overlay`; `.barrier-flash` 250ms / `.barrier-pop` 350ms.
31. **Can’t-be-destroyed overlay** — `.cant-be-destroyed-overlay` + 5 particles.
32. **Card-name highlights + crest panels + fused loot + Skybound + buff delta + play-blocked** in the tooltip — see §4 gap specs.
33. **Amulet named-counter badge** — First numeric `counters` entry as `.countdown-badge` when countdown is null.
34. **`.spell-cast` 0.5s** on a played spell (`css/animation.css:1-14`).
35. **Terminal overlay reason + rematch-same-seed** — `Deck-out` / `Lethal`; offer Rematch (same seed) next to swap-sides.
36. **Terminal readout label** — in progress (round 3).
37. **Empty seed = random** — Old placeholder “random if empty”.
38. **Share URL `a`/`b` aliases on write** — Keep `deckA`/`deckB`/`mode`; also emit `a`/`b` so old bookmarks work both ways.
39. **Context-menu suppression** on the whole play surface (`src/boot/contextMenu.ts:3-17`).
40. **Fuse via drag-release in hand** — `src/ui/pointerDragSession.ts:324-334`.
41. **Choice `.processing`** then tear down (`src/ui/choiceModal.ts:29-33`).
42. **Image-load release** on node teardown (`src/ui/releaseImageLoads.ts:13-35`).

### Rare (practice drawer)

43. **Save/load after the 200-step ring** — in progress (round 3).
44. **Position option `Name · T{n} · time`; Rename; Del** — `src/ui/positionPanel.ts:49-51`; `index.html:305-317`.
45. **Reroll + F8** — Restore checkpoint with a new RNG branch; status `T{n} · rerolls N`.
46. **Export List** — Blue deck as `Nx Name` + clipboard.
47. **Icarus `!` badge** — Gold 18px circle at `top:28px; left:6px` when Icarus-buffed.

---

## Status counts

| status | rows |
|---|---|
| ported | 121 |
| partial | 18 |
| missing | 37 |
| dropped (by design) | 10 |
| in progress (round 3) | 31 |
| **total** | **217** |

Counts are feature rows in §§1–10 (the gap list is a reordering, not extra rows). The 31 round-3 rows are the brief’s list split across the tables (e.g. yellow/green glow appears as several glow-class rows).

## Please confirm with the owner

These look dropped on purpose (brief RZ or replaced by a new control). I will not put them on the next fix brief unless you say they are still wanted:

- blackbox, sparring-line scripts, puzzles, coverage banner, god-mode (RZ)
- Consistency Trainer, `?test=1` QA bridge, UNIMPL/UNKNOWN OP badges, burn-preview box
- Deck *paste* import panel (JSON file import is what shipped)
- Smaller top-hand scale (R1/R2 asked for equal hands)
- Rematch (same seed) / (new seed) vs today’s New Game + Rematch (swap sides)
- Right-click-to-play / left-click-to-fuse (year of muscle memory — I listed it as a gap; confirm before the next brief treats it as optional)
