# UI parity audit — old Practice-Tool client vs arena `ui/`

Audit of every user-visible feature of the old practice tool’s **client** (`$HOME/practice-tool`, `origin/main` `bf8a9123`) against the new client (`ui/` at `8dfcb15`, plus this doc). Rules, card facts, and engine computation are out of scope.

**Visible result, not a code copy** (owner, 2026-09-11): *“make sure you don't copy it blindly. you can improve the code but make sure the result is the same. same glows, counters, colors etc.”* A row is `ported` only when the player sees the same thing (what, where, colour/token, when, how long). Gap specs name the result to reproduce — old CSS tokens, timings, corners — never a function to paste. File:line cites are evidence of the old result, not an implementation.

**Old paths** are relative to `$HOME/practice-tool`. **New paths** are relative to this repo.

**Verification.** Every row was read in both trees. Rows tagged `(code-read)` had no matching on-screen state in this pass (Barrier, Skybound, fuse/choice, game-over, F8, etc.). Untagged rows were also confirmed on a running preview (`wasm-pack` + `ui` production build + Chromium at `http://127.0.0.1:4173`). Screenshots: `/opt/cursor/artifacts/parity_*.png`.

**Round 3 / SF.** Rows that landed on `main` since `c69801c` were verified against this client. Owner 2026-09-11 dropped the form-letter badge and form-gate tooltip lines. Undo-inside-choice is one pick per press (owner 2026-09-05). Do not re-implement a `ported` or owner-dropped row.

**Owner decisions 2026-09-11**

1. **Click map — keep the old tool’s.** Left-click / tap a fuse-capable hand card opens Fuse and does not play. Right-click a playable hand card plays it. Drag to the own board plays. Releasing a hand-card drag inside the hand opens Fuse when the card can fuse. Engage by right-click on an `engage-ready` amulet (left-click may also engage). The OS context menu is suppressed on `.card`, `.zone`, `.leader`, `.evo-btn` and the play surface. The round-3 Fuse chip stays as the visible affordance; on a tablet the tap on the card is the primary path.
2. **Terminal overlay.** Buttons: Rematch (same seed), Rematch (new seed), New Game (opens the drawer). Remove “Rematch (swap sides)”. The first player of a rematch follows the drawer’s first-player setting (A / B / coin). Reason line: `Deck-out` or `Lethal`, not “Match over”.
3. **Dropped for good** (keep as `dropped (by design)`): blackbox, sparring-line scripts, puzzles, coverage banner, god-mode, Consistency Trainer, `?test=1` bridge, UNIMPL/unknown-op badges, burn-preview box, deck paste panel, smaller top-hand scale.

**Status** ∈ `ported` | `missing` | `dropped (by design)` | `dropped (not representable)` | `dropped (owner decision 2026-09-11)`.

Dropped-by-design (owner 2026-09-11 + brief RZ / `ui/README.md:50`): blackbox, sparring-line scripts, puzzles, coverage banner, god-mode, Consistency Trainer, `?test=1` QA bridge, UNIMPL/UNKNOWN OP face badges, burn-preview box, deck *paste* import panel (JSON file import was ported instead), smaller top-hand scale.

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
| Active-on-bottom: `body.active-on-bottom` + `active-second` swaps side `order` (code-read) | `src/boot/perspectiveFlip.ts:4-14`; `src/ui/render.ts:31,44-66`; `css/phase3-layout.css:257-263` | `ui/index.html:80-88`; `ui/src/main.ts:712-723`; `ui/css/phase3-layout.css:234-240` | ported | Persisted as `svwb.activeOnBottom`; applied by an inline boot script before the first paint. |
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
| Live turn / ply readout on the rail | — (no live turn on the board; only on saved-position labels) | `ui/index.html:289-290`; `ui/src/render.ts:118-127` (`Turn N · A` / `Blue (A) wins`) | ported | New-only extra. Terminal label is `Blue (A) wins` / `Red (B) wins` / `Draw`. |

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
| `.stat-buffed` text `--color-stat-buffed` `#39d98a` + green glow (ATK if buffed; DEF if buffed and not damaged) (code-read) | `css/cards.css:577-581`; `src/ui/zones/dom.ts:163-168`; `src/ui/zones/viewModel.ts:122-151` | `ui/css/cards.css:636-640`; `ui/src/render/card.ts:247-256` | ported | ATK above printed → buffed; DEF above printed and not below max → buffed. |
| `.stat-damaged` text `--color-stat-damaged` `#ffb366` (DEF below max; ATK if debuffed) (code-read) | `css/cards.css:583-588`; `src/ui/zones/dom.ts:163-168` | `ui/css/cards.css:643-647`; `ui/src/render/card.ts:247-256` | ported | ATK below printed → damaged; DEF below `max_defense` → damaged. | 
| Evolved: gold ring `--color-evolved-ring` + top-right **E** (code-read) | `css/cards.css:517-534`; `src/ui/zones/dom.ts:66-67` | `ui/css/cards.css:519-543`; `ui/src/render/card.ts:173-174` | ported | 
| Super-evolved: purple pulse `pulseSuper` 1.4s + **SE** (code-read) | `css/cards.css:536-574`; `src/ui/zones/dom.ts:66` | `ui/css/cards.css:545-583`; `ui/src/render/card.ts:173` | ported | 
| Evolved / super art (evo hash / evo image) (code-read) | `src/ui/zones/dom.ts:76-80` (`card.base_image` or `card.image`) | `ui/src/images.ts:5-9,115`; `ui/src/render/card.ts:115` (`evoCard` when evolved/super) | ported | 
| `updateCard` on reused DOM must refresh countdown, overlays, evo art — not just cost/ATK/DEF (code-read) | old replaces the node when the VM changes (`src/ui/zones/index.ts:115-137`) | `ui/src/render/card.ts:34-49,145-171` | ported | Signature includes countdown, named counter, traits, evo flags; `paintWrapper` rebuilds art + overlays on change. |
| Alternate-form badge cyan `{formLabel} {cost}` (e.g. `Accelerate 1`) at top of art (code-read) | `css/cards.css:623-637`; `src/ui/zones/dom.ts:145-151` | `ui/css/cards.css:685-699` (CSS only; no JS node) | dropped (owner decision 2026-09-11) | “you don't need to mark them as A C E, just the cost dynamically adapting is enough.” |
| No E/A/C trinity badges on the cost chip; yellow = enhance/alternate or a met gate, green = playable, none = unplayable (code-read) | `src/ui/helpers/glow.ts:237-260`; `css/cards.css:590-662` | `ui/src/render/card.ts:71-94`; tests assert `.alternate-form-badge` count 0 | ported | Cost chip is the paid PP only. Yellow = enhance / alternate / met gate; green = base playable; none = unplayable. |
| `.playable-glow` green `playPulse` 1s — base-cost playable, no special form | `css/cards.css:654-662`; `src/ui/helpers/glow.ts:260` | `ui/css/cards.css:144-157,667-692`; `ui/src/render/card.ts:85` | ported | Same tokens; acting-player-only via `handInfo`. |
| `.enhance-ready` yellow `enhPulse` 1s — Enhance or a met special gate (code-read) | `css/cards.css:590-598`; `src/ui/helpers/glow.ts:241-257` | `ui/css/cards.css:649-660`; `ui/src/render/card.ts:81-83`; `ui/src/render.ts:208` | ported | Yellow `enhPulse` when a non-form gate is met or the form is not `normal`. |
| `.alternate-ready` amber `altPulse` 1s — Accelerate / Crystallize payable (code-read) | `css/cards.css:600-608`; `src/ui/helpers/glow.ts:237-238` | `ui/css/cards.css:662-670`; `ui/src/render/card.ts:83` | ported | Amber `altPulse` stacked with yellow when form is Accelerate / Crystallize. |
| Hand/board glow only on the acting player’s cards (code-read) | `src/ui/helpers/glow.ts:146-148`; `src/ui/zones/viewModel.ts:98,163` | `ui/src/render.ts:208-218,282-294` uses `handInfo`/`boardInfo` (acting-player legality) | ported | Off-turn hands/boards get no playable / attack glow. |
| `.can-attack` green outline — follower can attack (not rush-restricted) (code-read) | `css/cards.css:144-148`; `src/ui/zones/dom.ts:69-72` | `ui/css/cards.css:144-157`; `ui/src/render/card.ts:88-91` | ported | Green only when `canAttack && !rushOnly`. |
| `.rush-glow` yellow — Rush on the summon turn (`hasRush && justPlayed && !hasStorm`) (code-read) | `src/ui/zones/viewModel.ts:183`; `src/ui/zones/dom.ts:70`; `css/cards.css:201-205` | `ui/css/cards.css:207`; `ui/src/render.ts:282-288`; `ui/src/render/card.ts:90`; `engine/src/info.rs` `rush_only` / `followers_only_this_turn` | ported | Yellow iff `attacksFrom` is non-empty, none of those attacks is the leader, and the follower entered this turn. The moment `legal()` lists no attack from that slot (already attacked, empty enemy board, opponent's turn, or a restriction) both yellow and green are gone. Idle evolved gold ring removed so it cannot impersonate the yellow attack glow. | 
| `.engage-ready` yellow `engagePulse` 1s on an amulet that can Engage (code-read) | `css/cards.css:687-705`; `src/ui/zones/viewModel.ts:187-206`; `src/ui/zones/dom.ts:60-63` | `ui/css/cards.css:723-741`; `ui/src/render.ts:254` | ported | 
| `.fuse-ready` gold ring on a fuse-capable hand card (code-read) | — (no on-card fuse chrome; fuse is click/tooltip only) | `ui/css/client.css:121-123,234-252`; `ui/src/render.ts:226-237` | ported | Gold ring plus a bottom-centre **Fuse** chip on a fuse-capable acting-turn hand card. |
| `.selectable` dashed green (mulligan) / yellow (targets); `.selected` solid green (code-read) | `css/buttons.css:325-328`; `css/cards.css:739-778`; `src/ui/zones/dom.ts:56-57` | `ui/css/buttons.css:325-328`; `ui/css/cards.css:797-815,1130-1133` | ported | 
| Selected ✓ checkmark overlay (top-right, `#2ecc71`) (code-read) | `src/ui/zones/dom.ts:123-137` | `ui/src/render/card.ts:198-206`; `ui/css/cards.css` `.selected-check` | ported | 24px extrabold `#2ecc71` ✓, `0 0 4px` black + `0 0 8px` `#2ecc71`, plus the solid green selected ring. |
| `body.select-mode`: non-targets 50% + greyscale 0.55 (code-read) | `css/cards.css:732-737`; `css/modern-theme.css:767-769`; `src/ui/render.ts:168-172` | `ui/css/cards.css:767-815`; `ui/css/modern-theme.css:758-778`; `ui/src/render.ts:65` | ported | 
| Ward overlay (board): white shield SVG, `ward-pulse` 2s (code-read) | `src/ui/overlays.ts:27-31`; `css/cards.css` ward block | `ui/src/render/card.ts:219-223`; `ui/css/cards.css:230-271` | ported | 
| Ambush overlay (board): smoke SVGs 4s/5s (code-read) | `src/ui/overlays.ts:32-36` | `ui/src/render/card.ts:224-228`; `ui/css/cards.css:817-904` | ported | 
| Aura overlay (board): purple field + dashed ring 2.2s/6s (code-read) | `src/ui/overlays.ts:37-51` | `ui/src/render/card.ts:229-233`; `ui/css/cards.css:970-1042` | ported | 
| Intimidate overlay (board): amber diamond, `intimidate-flicker` 1.2s (code-read) | `src/ui/overlays.ts:62-66` | `ui/src/render/card.ts:234-238`; `ui/css/cards.css:273-308` | ported | 
| Can’t-attack overlay (board): crossed chains when the follower cannot attack (code-read) | `src/ui/overlays.ts:52-61` (`hasCantAttack` **or** `cantAttack` / `cantAttackUntilOpponentEOT`) | `ui/src/render/card.ts:282-290`; `engine/src/info.rs` `cannot_attack_reason` | ported | Chains on any printed lock (`cantAttack*` / engine reason). Not summoning sickness. |
| Barrier overlay + 250ms flash / 350ms pop (code-read) | `src/ui/overlays.ts:118-138`; `css/cards.css:443-487` | `ui/src/render/card.ts:209,327-330,387-398`; `ui/css/cards.css:313-535` | ported | Cyan ring + 5 particles (`.barrier-overlay`, `--glow-barrier`). Gain/loss is a `traits` signature diff in `updateCard`: `.barrier-flash` 250ms, `.barrier-pop` 350ms. |
| Can’t-be-destroyed overlay + 5 gold particles (code-read) | `src/ui/overlays.ts:88-109`; `css/cards.css` CBD block | `ui/src/render/card.ts:327`; `ui/css/cards.css:1196-1354` | ported | Gold field + five particles while `cantBeDestroyedByAbilities` is in `traits` (Earth Sigil / `Game.full()`). |
| Bane / Drain / Last Words PNG icons, bottom-centre 26×26 stack (code-read) | `src/ui/overlays.ts:67-83`; `images/icon_*.png` | `ui/src/render/card.ts:244-258`; `ui/public/images/icon_*.png` | ported | 
| Ongoing icon `images/icon_ongoing.png` (code-read) | `src/ui/overlays.ts:84-86` | `ui/src/render/card.ts:306-313`; `wasm/src/bundle.rs` `tags` | ported | Shown when `cardText.tags` includes `ongoing` (Static ability or printed “Ongoing”). |
| Icon stack swap animation `.swap-2/3/4` (2s/3s cycle) when 2+ icons (code-read) | `src/ui/overlays.ts:111-115`; `css/cards.css:1008-1090` | `ui/src/render/card.ts:314-317`; `ui/css/cards.css:1045-1126` | ported | `.swap-2` 2s; `.swap-3`/`.swap-4` 3s. A single icon stays static. |
| Spellboost badge (blue circle, white count) under the cost (code-read) | `src/ui/zones/dom.ts:207-216`; `css/cards.css:871-897` | `ui/src/render/card.ts:189-196`; `ui/css/cards.css` `.spellboost-badge` | ported | Blue circle, white count, under the cost when the card has Spellboost (`handInfo` gate / `printed_tags` / `cardText.tags`) and `spellboost_count ≥ 1`. Playing a spell still increments every hand card; cards without Spellboost get no badge and no tooltip Spellboost line. |
| Countdown number, large white + black stroke, bottom-right (amulets) (code-read) | `src/ui/zones/dom.ts:172-176`; `css/cards.css:708-724` | `ui/src/render/card.ts:197-202`; `ui/css/cards.css:744-760` | ported | Refresh on `updateCard` is round 3. |
| Amulet named-counter fallback (first numeric `counters` entry as the same badge) (code-read) | `src/ui/zones/dom.ts:184-199` | `ui/src/render/card.ts:22-31,274-283`; `engine/src/info.rs` `named_counter` | ported | No countdown + numeric `vars`: same bottom-right white/black-stroke number. Order is **X, then Y, then Z**. |
| Icarus `!` gold 18px badge (top-left of art) (code-read) | `src/ui/zones/dom.ts:83-104` | — | dropped (not representable) | The engine has no Icarus buff / trait, so the badge cannot be driven from `full()` or `traits`. |
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
| Crests: red 3+2 slots, blue 2+3; empty dashed ring; filled green glow; circular art | `index.html:693-702,890-899`; `src/ui/render.ts:614-651`; `css/modern-theme.css:662-698` | `ui/index.html:275-321`; `ui/src/render/card.ts:261-281`; `ui/css/modern-theme.css:679-705` | ported | Slot box stays `var(--crest-slot-size)` square / `50%`. Filled cue is `drop-shadow` on `.crest-icon-frame` (ogee alpha), not a rectangular gradient. |
| Crest countdown badge bottom-right (code-read) | `src/ui/render.ts:654-660`; `css/layout.css:480-497` | `ui/src/render/card.ts:282-287`; `ui/css/layout.css:482-504` | ported | 
| Faith value badge **top-left** on the Faith crest (same `.crest-countdown` class, repositioned) (code-read) | `src/ui/render.ts:663-673` | `ui/src/render/card.ts:428-432`; `ui/css/client.css:257-269` | ported | `.crest-faith` count at top-left; countdown stays bottom-right. | 
| 🃏 hand / 📦 deck / 💀 cemetery / ☠️ shadows, both sides, always public | `index.html:688-691,902-905`; `src/ui/counts.ts:25-33` | `ui/index.html:270-327`; `ui/src/render.ts:77-84` | ported | 
| No combo / earth / rally / faith / banished on the rail (code-read) | (not shown) | (not shown) | ported | 
| Leader Barrier cyan ring on the HP host (`.has-leader-barrier`) (code-read) | `src/ui/render.ts:704-721`; `css/modern-theme.css:911-929` | `ui/src/render.ts:456-462`; `engine/src/info.rs` `has_leader_barrier`; `ui/css/modern-theme.css:902-920` | ported | Cyan `--glow-barrier` ring on the HP pill while a damage cap is live. |
| Enemy leader `.selectable` red pulse while it is a legal **pending** attack target (code-read) | `src/ui/render.ts:123-146`; `css/buttons.css:149-203` | `ui/src/render.ts:472-496` | ported | Outline only while a pending attack lists that leader as a legal target. | 

---

## 4. Tooltips

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Single `#cardTooltip`, dark panel, `pointer-events: none`, z 9999, max ~360px | `index.html:940-957`; `css/tooltip.css:3-9` | `ui/index.html:333`; `ui/css/tooltip.css:3-8`; `ui/css/modern-theme.css:748-755` | ported | 
| Order: **name** → class/tribes + set → **counter block** → description → crest panels → fused loot → Rally line → Skybound line → buff delta (code-read) | `src/ui/tooltips.ts:169-176` | `ui/src/tooltip.ts:39-61` | ported | Name → class/tribes + set → gates → description → crests / fused / Rally / Skybound / `+A/+D` / `Cannot play:`. Cost/stats line omitted (hotfix). |
| Name `.tooltip-header-name` lg bold | `src/ui/tooltips.ts:170`; `css/tooltip.css` | `ui/src/tooltip.ts:43`; `ui/css/tooltip.css:11-15` | ported | 
| Meta: `Class` or `Class/Tribe1, Tribe2`; set badge `#9aa3b2` in-rotation / `#7a8494` older (code-read) | `src/ui/tooltips.ts:98-99,34-47` | `ui/src/tooltip.ts:43-48,84-88` | ported | `Class` or `Class/Tribes` plus `Set N` in `#9aa3b2` (set ≥ 8) / `#7a8494`. |
| Cost line with strikethrough / grey base when paid ≠ printed | (cost lives on the card, not in the tooltip) | — (omitted; `ui/src/tooltip.ts:47`) | ported | Matches the old tool: no cost/stats line in the tooltip (hotfix). |
| Spell/amulet tooltips omit `0/0` | n/a (no ATK/DEF in old tooltip) | `ui/src/tooltip.ts:31-38` (R2) | ported | 
| Keyword lines bold `--color-warn` (`Fanfare:`, `Enhance (N):`, `Accelerate (N):`, …) | `src/ui/tooltipFormat.ts:11-12,86-93` | `ui/src/tooltip.ts:6-10,83-91` | ported | 
| Card-name highlights italic blue inside description (code-read) | `src/ui/tooltipFormat.ts:69-79` | `ui/src/tooltip.ts:204-211`; `ui/css/tooltip.css:70-74` | ported | Catalog names in the description are italic semibold `--color-accent-blue-bright`. |
| “Gain crest:” muted lead + crest icon/name/text panels after the description (code-read) | `src/ui/tooltipFormat.ts:96-97,117-147`; `src/ui/tooltips.ts:104-107` | `ui/src/tooltip.ts:formatCrestPanels`; `ui/src/crest-icon.ts`; `ui/css/tooltip.css`; `wasm/src/bundle.rs` `crests` | ported | Muted “Gain crest:” lead, then panels from `cardText.crests` (`name` + `text`). Icon is a cropped granting-card headshot behind `crest_frame.png` (56px, `--crest-fx/fy/zoom`). **Until this round the row claimed `ported` while every panel rendered a catalog `specificEffects` id (or a `Gain crest:` capture including the trailing period) with an empty body — no test asserted panel content.** |
| Accelerate / Crystallize form line from `cardText.forms` (Enhance already in `text`) | `src/ui/tooltipFormat.ts` description | `ui/src/tooltip.ts:formatFormLines`; `wasm/src/bundle.rs` `forms` | ported | After the description, a `.tooltip-desc-line` for each `forms[].printed` that is not already in `text` (whitespace-normalised). Adds the 10 Accelerate/Crystallize cards; 43 Enhance cards unchanged. |
| Crest slot long-press (touch) opens the same crest tooltip | — (hover only) | `ui/src/render.ts:bindLongPressTooltip` | ported | 400 ms hold on a filled `.crest-slot`; cancel on >8 px move; dismiss on the next tap outside. Mouse hover unchanged (blue above, red below). |
| Dynamic counter block **before** the description: `(Label: value/threshold)` (code-read) | `src/ui/tooltips.ts:101-102,172`; `src/ui/tooltipCounters.ts:407-411` | `ui/src/tooltip.ts:50-58,239-254` | ported | `.tooltip-counter-block` sits above the description. Lines are `{Label} {have}/{need}`. |
| Enhance / Necromancy / Combo / Rally / Earth Rite / Overflow / Spellboost / Accelerate / Crystallize progress lines (code-read) | `src/ui/tooltipCounters.ts:72-98`; `src/ui/info`-equivalent in counters | `ui/src/info.ts:26-49`; `ui/src/tooltip.ts:239-241` | dropped (owner decision 2026-09-11) | Necromancy / Combo / Rally / Earth Rite / Overflow / Spellboost lines paint. Form gates omitted: “enhance doesn't need the tooltip with have1. it's not progress.” | 
| Dedicated blue `Rally: current / req` line (code-read) | `src/ui/tooltips.ts:125-141` | `ui/src/tooltip.ts:117-124` | ported | `Rally: have / need` in `#7af` whenever the printed text has Rally (gate optional). |
| Dedicated gold `Skybound Art: current / req` (code-read) | `src/ui/tooltips.ts:143-162` | `ui/src/tooltip.ts:127-133` | ported | Gold `#ebd04f` `Skybound Art: have / need` when the text or a gate has Skybound Art. |
| Orange `Fused Loot (unique): N` + names, or grey `Fused: N cards` (code-read) | `src/ui/tooltips.ts:109-123` | `ui/src/tooltip.ts:104-114` | ported | Orange unique loot list, or grey `#aaa` `Fused: N cards` at 0.8em. |
| Buff delta `+A/+D` green if non-negative, `#ff6666` if any negative (code-read) | `src/ui/tooltips.ts:19-31,164-176` | `ui/src/tooltip.ts:136-148` | ported | Bold `+A/+D` in `#66ff66` or `#ff6666`. |
| Red `Cannot play: {reason}` when the hand card is blocked (code-read) | `src/ui/tooltips.ts:265-269`; `src/ui/zones/dom.ts:48-52` | `ui/src/tooltip.ts:97-99`; `engine/src/info.rs` `blocked_reason`; `ui/css/tooltip.css` `.tooltip-play-blocked` | ported | Active-turn unplayable cards append `#ff8888` bold `Cannot play: {reason}` (engine reason). |
| Hover shows; mousemove repositions; mouseleave hides | `src/ui/tooltips.ts:330-361` | `ui/src/render.ts:789-814` | ported | 
| Smart vertical anchor: bottom half of the viewport grows **up**; top half grows **down**; 12px from cursor, clamped (code-read) | `src/ui/tooltips.ts:337-356` | `ui/src/render.ts:1179-1190` | ported | Bottom half grows up; top half grows down; 12px from the cursor, clamped 12px from the edges. |
| No large art in the tooltip (text only) | `src/ui/tooltips.ts` (no preview node) | R1 removed `#cardPreview` | ported | 
| Drag: pin the same tooltip at `(12px, 12px)` for the gesture (code-read) | `src/ui/tooltips.ts:199-215` | `ui/src/render.ts:1193-1218` | ported | Drag-pinned at `top:12px; left:12px`; hides on release unless still hovering a card. |
| Live refresh on every `render()` (counters / gates stay current) (code-read) | `src/ui/render.ts:230`; `src/ui/tooltips.ts:278-318` | `ui/src/render.ts:139,1220-1228` | ported | Open tooltip rebuilds on every paint; drag-pin wins over hover during the gesture. |
| Face-down cards never show a tooltip (code-read) | `src/ui/zones/index.ts:71-77` | `ui/src/render.ts:791` | ported | 
| Crest hover: name + description; Faith `Faith — {count}`; blue crests above cursor, red below (code-read) | `src/ui/render.ts:676-698` | `ui/src/tooltip.ts:131-141`; `ui/src/render.ts:575-598` | ported | Faith line is `Faith — {count}` (em dash). Blue (A) crests above the cursor; red (B) below. |
| Long-press (touch) opens the same tooltip (code-read) | — (hover + drag only) | `ui/src/render.ts:1361-1377` | ported | 400ms touch `pointerdown` opens the same `#cardTooltip`. | 

---

## 5. Interaction flows

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Mulligan: dashed green selectable hand cards; click toggles mark | `src/ui/zones/viewModel.ts:260-261`; `src/ui/zones/handlers.ts:27-35` | `ui/src/render.ts:188-189`; `ui/src/input.ts:77-81` | ported | 
| Mulligan selected: green ✓ + `.selected` (code-read) | `src/ui/zones/dom.ts:123-137` | `ui/src/render/card.ts:198-206`; `ui/src/render.ts:203` | ported | Same 24px `#2ecc71` ✓ plus the solid green selected ring. |
| Mulligan marks + confirm on **both** hands (sequential acting player) (code-read) | per-side confirm `index.html:619-625,673-679`; each side’s cards marked independently | `ui/src/session.ts:59`; `ui/src/render.ts:213,601-608` | ported | Sequential acting: the current mulliganer’s four cards are selectable / marked; Confirm sits in that side’s hand row. | 
| Confirm Mulligan (Red/Blue) in that side’s hand row | `index.html:619-625,673-679` | `ui/index.html:239-263`; `ui/src/render.ts:473-480` | ported | 
| No hand drag / no End Turn during mulligan (code-read) | `src/ui/zones/handlers.ts:104-105`; `src/ui/render.ts:200` | `ui/src/render.ts:208,489-494` | ported | 
| **Drag** a hand card onto the matching board zone to play (code-read) | `src/ui/drag.ts:115-143` | `ui/src/render.ts:201-208,271-285` | ported | 
| **Right-click** an active-turn hand card to play (code-read) | `src/ui/zones/handlers.ts:52-56` | `ui/src/input.ts:192-210` | ported | Right-click a playable hand card plays it. No OS menu. |
| **Left-click / tap** a fuse-capable hand card to open Fuse (does not play) (code-read) | `src/ui/zones/handlers.ts:59-101`; `src/ui/zones/dragClickGuard.ts:50-56` | `ui/src/input.ts:170-172` | ported | Left-click / tap a fuse-capable hand card opens Fuse and does not play. |
| Fuse via releasing a drag **inside the hand** (no valid drop) (code-read) | `src/ui/pointerDragSession.ts:324-334` | `ui/src/drag.ts:228-236`; `ui/src/render.ts:239-240` | ported | Release a fuse-capable hand card inside the same hand → Fuse opens. |
| Off-turn hand: not draggable, no play (silent) (code-read) | `src/ui/zones/handlers.ts:104-105` | `ui/src/render.ts:208` (`playable && phase !== "mulligan"`) | ported | 
| 8px drag threshold; source opacity 0.45; preview clone scale 1.05 (code-read) | `src/ui/pointerDragSession.ts:38`; `css/cards.css:184-193` | `ui/src/drag.ts:14,89-105`; `ui/css/cards.css:193-202` | ported | 
| Drop highlight green `.pointer-drop-highlight` on **legal** targets only (code-read) | `src/ui/pointerDragSession.ts:148-157`; `css/cards.css:195-199` | `ui/src/drag.ts:80-88` | ported | Highlight only when `t.accepts(payload)` is true. | 
| Click-after-drag suppressed (~100ms) (code-read) | `src/ui/pointerDragSession.ts:235-243` | `ui/src/drag.ts:61-64,157-161` | ported | 
| Attack: drag own `canAttack` follower onto an enemy follower or the enemy leader strip (code-read) | `src/ui/drag.ts:26-66,98-112,209-256` | `ui/src/render.ts:258-265,286-318`; `ui/src/input.ts:103-105` | ported | 
| Attack: click attacker then click target (new two-step) (code-read) | — | `ui/src/input.ts:95-97,103-105`; `ui/src/render.ts:1205-1234` | ported | New-only extra; keep. Pending look/cancel — see next row. |
| Pending attack / evolve: highlight legal targets; Cancel chip; tap elsewhere cancels (code-read) | old targeting is engine-driven select-mode; no pending-attack chip | `ui/src/render.ts:1205-1234`; `ui/src/input.ts:137-154` | ported | Legal targets highlighted; `.pending-cancel-chip`; tap elsewhere or Escape cancels. | 
| Evolve: click Evo/Super then click an allied follower (code-read) | click is a no-op (drag only) (`src/ui/evo.ts:61-66`) | `ui/src/input.ts:56-61,91-93` | ported | New-only extra. |
| Engage: **right-click** an `.engage-ready` amulet (code-read) | `src/ui/zones/handlers.ts:137-148` | `ui/src/input.ts:182-184,212-214` | ported | Right-click engages; left-click may also engage. No OS menu. |
| End Turn (Blue/Red) visible only for the active player; hidden in mulligan / game over | `src/ui/counts.ts:46-49`; `src/ui/render.ts:393-418` | `ui/src/render.ts:489-498` | ported | 
| Bonus PP click toggles / commits on the second player | `index.html:708-728` | `ui/src/main.ts:680-688` | ported | 
| Choice / mode modal: dim overlay, title **“Choose an effect:”**, buttons `label` or `name`, yellow Earth Rite sub-line (code-read) | `src/ui/choiceModal.ts:2-38`; `css/buttons.css:96-145` | `ui/src/render.ts:732-740`; `ui/css/buttons.css:176-181` | ported | Title “Choose an effect:” for modes; `#f1c40f` Earth Rite sub-line when the option costs sigils. Round-3 prompt bar kept. | 
| Choice buttons: card names, 1-based mode text, no dead buttons (code-read) | `src/ui/choiceModal.ts:16-18` | `ui/src/render.ts:777-783,990-1007` | ported | Buttons are catalog names or `Mode {n}` (1-based); only legal `choose` actions. |
| Stale choice click listeners on reused cards (code-read) | n/a (modal torn down) | `ui/src/input.ts:119-125`; `ui/src/render.ts:817-861` | ported | In-place picks go through the document click + `chooseActionForElement`; highlights are class-only (no leftover card listeners). |
| Fuse Confirm sits under the modal (code-read) | fuse UI is engine-driven | `ui/src/render.ts:764-767,949-960` | ported | Fuse is in-place; Confirm is on the prompt bar as `{prompt} ({count})`. | 
| Multi-target confirm bar: `{text} ({count})` bottom-centre green (code-read) | `src/ui/targeting.ts:16-28`; `index.html:152-163` | `ui/src/render.ts:850-856,924-926`; `ui/css/buttons.css:240-252` | ported | Prompt-bar confirm is `{prompt} ({count})` in the old green (`#4caf50`→`#45a049`, 12×24, 16px bold white). |
| Modal `.processing` then remove on click (code-read) | `src/ui/choiceModal.ts:29-33` | `ui/src/render.ts:752-759` | ported | Option click adds `.processing` and removes the dim overlay immediately. |
| Choice-phase prompt + Undo chip (code-read) | — | `ui/src/render.ts:864-907,949-960` | ported | Mid-board prompt bar with the choice text and an **Undo** chip. |
| Undo symmetry inside a choice (one undo = one pick, not the whole choice) (code-read) | engine history (not in UI files) | `ui/src/session.ts:262-272,297-302` | ported | Each applied action is one history step. Ctrl+Z after a completed choice reopens the prompt with the last pick removed; another press drops the previous pick or the play/fuse. Ctrl+Y replays the same pick. | 
| Vs-bot hide opponent hand (face-down) (code-read) | sparring “Hide line hand” (`src/ui/scriptPanel.ts:74-86`) — dropped | `ui/index.html:143-146`; `ui/src/render.ts:101-102,186-189` | dropped (by design) | Hide-hand survives as vs-bot only; sparring-line hide is out of scope. |

---

## 6. Feedback & animation

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| FCT toggle in settings, default on, persist `svwb.floatingCombatText` (code-read) | `index.html:366-371`; `src/ui/floatingCombatText.ts:13-56`; `src/boot/floatingCombatText.ts` | `ui/index.html:228-231`; `ui/src/main.ts:712-719,778-781` | ported | Persisted as `svwb.floatingCombatText` (`"1"` / `"0"`). Default on. |
| FCT `−N` red `--color-danger`, `+N` green `--color-success`, 1.35s rise, 130ms stagger (code-read) | `css/floating-combat-text.css:5-61`; `src/ui/floatingCombatText.ts:16-17,136-139` | `ui/css/floating-combat-text.css:10-38`; `ui/src/fct.ts:4,13-35` | ported | 
| Max 4 floaters per target; stack `--float-stack-index` (code-read) | `src/ui/floatingCombatText.ts:17,130` | `ui/src/fct.ts:5,75-85` | ported | Cap 4 per host; `--float-stack-index` × 18px. |
| Leader FCT from the leader strip centre (code-read) | `src/ui/floatingCombatText.ts:62-66,203-245` | `ui/src/fct.ts:43-45` (`#blueLeader` / `#redLeader`) | ported | 
| Follower FCT on the card (code-read) | `src/ui/floatingCombatText.ts:68-70,147-151` | `ui/src/fct.ts:43-55` | ported | Host is `[data-slot]` on that player’s board. | 
| Follower brightness flash on damage (`.floating-combat-flash`) (code-read) | `src/ui/floatingCombatText.ts:147-151`; `css/floating-combat-text.css:89-103` | `ui/src/fct.ts:58-63`; `ui/css/floating-combat-text.css:89-103` | ported | 0.45s brightness 1.35 / saturate 1.1 on follower damage. |
| Suppress FCT on undo/redo / position load (code-read) | `src/ui/floatingCombatText.ts:304-314` | `ui/src/session.ts:175,193`; `ui/src/main.ts:180-183` | ported | 
| No enter/leave/destroy CSS on old zone reconcile (code-read) | `src/ui/zones/index.ts:83-147` | `.card-enter` 140ms / `.card-leave` 120ms (`ui/css/animation.css:3-48`; `ui/src/render.ts:349-357`) | ported | New-only extra; keep. |
| `.dying` 0.6s fade/scale (class rarely applied) (code-read) | `css/cards.css:150-156` | `ui/css/cards.css:159-165` (unused; leave uses `card-leave`) | ported | Same unused CSS. |
| No attack lunge / slash (code-read) | — | — | ported | 
| `.spell-cast` 0.5s brightness/scale (code-read) | `css/animation.css:1-14` | `ui/src/render.ts:382-410`; `ui/css/animation.css:61-78` | ported | Leaving hand spell: brightness 1.5, cyan `rgba(100, 200, 255, 0.8)` drop-shadow, scale 1.1 → 1, 0.5s ease-out. Detached clone at the same rect when the card is removed the same paint. |
| Stat-change flash 150–160ms scale/brightness (code-read) | — (colour change only) | `ui/css/animation.css:24-58`; `ui/src/render/card.ts:142-159` | ported | New-only extra; keep. |
| Pressed card `translateY(2px) scale(0.97)` (code-read) | — | `ui/css/animation.css:11-15`; `ui/src/input.ts:31-45` | ported | New-only extra; keep. |
| No “Turn N” banner; turn change is the hand/leader glow + End Turn swap (code-read) | (none) | (none) | ported | 
| Terminal overlay: “First wins” / “Second wins” / “Draw”; reason “Deck-out” or “Lethal”; Rematch same seed / new seed (code-read) | `src/ui/render.ts:289-342` | `ui/src/render.ts:1054-1093`; `ui/src/main.ts:335-345` | ported | Reason `Deck-out` or `Lethal`. Buttons: Rematch (same seed), Rematch (new seed), New Game. First player of a rematch follows the drawer A/B/coin setting. |
| Rail terminal readout label (code-read) | — | `ui/src/render.ts:118-127` | ported | Live: `Turn N · A` / `Turn N · B`. Terminal: `Blue (A) wins` / `Red (B) wins` / `Draw`. |
| Action toast bottom-centre, 0.18s fade, default 1800ms (code-read) | `src/ui/toast.ts:19-27`; `css/modern-theme.css:976-998` | `ui/src/main.ts:318-335`; `ui/index.html:127`; `ui/css/modern-theme.css:986-1008` | ported | Bottom-centre `#actionToast` dark pill, 0.18s fade, holds 1800ms. |
| Board/leaders 72% opacity, no pointer events on game over (code-read) | `css/modern-theme.css:1050-1104` | `ui/css/modern-theme.css:961-965` | ported | 

---

## 7. History / log drawer

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Left battle-log button; drawer 360px, `transform 0.18s`; scrim click closes | `index.html:557-611,968-1002` | `ui/index.html:207-233`; `ui/src/main.ts:463-484` | ported | 
| Opening history closes settings (and vice versa) (code-read) | `index.html:969`; `src/boot/settingsDrawer.ts:41` | `ui/src/main.ts:443,471` | ported | 
| Escape closes the history drawer (code-read) | `index.html:988-990` | `ui/src/main.ts:497-504,530-533`; `ui/index.html:24-32` | ported | Escape closes the open 360px drawer (0.18s slide + scrim). |
| Sections: Red — Played, Red — Destroyed, Blue — Played, Blue — Destroyed | `index.html:590-611` | `ui/index.html:216-232` | ported | 
| Each row: cost badge + `Name ×count`, grouped by name+cost, sorted cost then name (code-read) | `src/ui/render.ts:451-507` | `ui/src/render.ts:616-668` | ported | Grouped by name+cost, sorted cost then name. 22px `#262c36`→`#1a1f27` cost square + `Name ×N`. |
| Optional set badge on the row (muted `#7a8494` if out of rotation) (code-read) | `src/ui/render.ts:494-503` | `ui/src/render.ts:655-661`; `ui/index.html` `.hist-set` | ported | 11px `Set N` in `#9aa3b2` / `#7a8494` (older). |
| Hover a row: 198px card-art preview follows the cursor (+18px, clamped) (code-read) | `src/ui/render.ts:534-580`; `index.html:144-147` | `ui/src/render.ts:671-705`; `#historyImgPreview` | ported | 198px art follows the cursor (+18px, clamped 12px). `#0c0e12`, 10px radius, `0 8px 18px`. |
| Rows are not clickable (hover only) (code-read) | (hover only) | (not clickable) | ported | 
| Destroyed list is **that side’s lost cards**, not “destroyed by” (code-read) | `src/ui/render.ts:205-211` (`players.*.destroyedHistory`) | `ui/src/session.ts:163-189` | ported | Owner is the side that held the card on the board before the destroy event. | 

---

## 8. Settings drawer and top bar

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Hamburger; title “Settings (Ctrl+Shift+M)” | `index.html:166-173` | `ui/index.html:82` | ported | 
| Drawer 360px left slide 0.18s; scrim click / Escape / Ctrl+Shift+M | `src/boot/settingsDrawer.ts:70-88`; `css/settings-drawer.css` | `ui/src/main.ts:438-460`; `ui/css/settings-drawer.css:50-68` | ported | 
| Blue / Red deck `<select>` | `index.html:196-207` | `ui/index.html:99-107` | ported | 
| Import Deck | `index.html:210-216` (paste community list) | `ui/index.html:109-110`; `ui/src/main.ts:611-624` (JSON `{id:count}` file) | dropped (by design) | JSON file import stays. Community-list paste is dropped (owner 2026-09-11). |
| Export List (Blue deck as `Nx Name` + clipboard) (code-read) | `index.html:217-223`; `src/ui/deckImportPanel.ts:363-365` | `ui/index.html:155-158`; `ui/src/main.ts:813-833` | ported | Copies the started Blue deck as `Nx Name` (catalog names) and shows the text in `#exportListPanel`. |
| Seed input, “random if empty” | `index.html:231-240` | `ui/index.html:157-164`; `ui/src/main.ts:244-245,320-323` | ported | Placeholder “random if empty”; empty field rolls a u64. |
| Game seed panel + Copy; button flashes “Copied” 1200ms | `src/ui/seedDisplay.ts:13-61` | `ui/index.html:166-170`; `ui/src/main.ts:782-796` | ported | Button reads `Copied` for 1200ms, then `Copy`. |
| First-player control | (implicit / start options, not a drawer select) | `ui/index.html:128-133` coin/A/B | ported | New-only extra. |
| Mode: hotseat / vs-bot / watch | — | `ui/index.html:93-98` | ported | New-only extra. |
| Vs-bot: human side, bot policy, hide bot hand (code-read) | — | `ui/index.html:134-146` | ported | New-only extra. |
| Watch: Bot A/B policies, auto-play, Step/Play/Pause, speed 1–20 (code-read) | — | `ui/index.html:148-156,197-205`; `ui/src/main.ts:325-361,627-647` | ported | New-only extra. |
| Start Game | `index.html:259` | `ui/index.html:158` | ported | 
| Undo / Redo buttons; disabled from stack; titles Ctrl+Z / Ctrl+Y | `index.html:266-275` | `ui/index.html:159-160`; `ui/src/render.ts:747-751` | ported | 
| Share URL write on start: `?seed=&a=&b=` (code-read) | `src/boot/shareUrl.ts:4-57` | `ui/src/share.ts:26-40` writes `deckA`/`deckB`/`mode` **and** `a`/`b` | ported | Both alias pairs are written; `a`/`b` still accepted on read so old bookmarks open the same match. |
| Consistency Trainer link (code-read) | `index.html:260-265` | — | dropped (by design) | Separate tool, not the M4 client. |
| God Mode checkbox + PP/EP/combo/shadows cheats (code-read) | `index.html:250-257,747-886`; `src/boot/godMode.ts` | CSS leftovers only | dropped (by design) | 
| Save Pos (prompt name) | `src/ui/positionPanel.ts:106-134` | `ui/src/main.ts:536-541` | ported | Snapshot format differs (action log vs board); see save/load row. |
| Position `<select>` `Name · T{n} · time` (code-read) | `src/ui/positionPanel.ts:49-51` | `ui/src/main.ts:786-809` | ported | Each option is `{name} · T{turn} · {locale time}`. Export JSON keeps `name`, `turn`, `savedAt`. |
| Load / Export / Import JSON (code-read) | `src/ui/positionPanel.ts:137-245` | `ui/src/main.ts:543-576` | ported | 
| Rename + Del (code-read) | `index.html:305-317` | `ui/src/main.ts:747-765`; `ui/index.html:223-224` | ported | Rename (prompt) and Del act on the selected in-memory position. |
| Checkpoint button + F6 | `index.html:334`; `src/boot/hotkeysBoot.ts:7-9` | `ui/index.html:178`; `ui/src/main.ts:513-518,577-581` | ported | 
| Restore CP | `index.html:338-343` | `ui/index.html:179`; `ui/src/main.ts:520-525,582-587` | ported | 
| Restore CP key **F7** | — (button only) | `ui/src/main.ts:520-525` | ported | New-only extra. |
| Reroll + **F8** (new RNG branch from the checkpoint) (code-read) | `index.html:344-350`; `src/boot/hotkeysBoot.ts:7-9` | `ui/src/session.ts:395-410`; `ui/src/main.ts:669-675,777-781`; `engine/src/state.rs` `State::reseed` | ported | F8 / Reroll restores the checkpoint and `reseed(splitmix64(gameSeed XOR (n * GOLDEN)))`. Status `Checkpoint: T{n} · rerolls N`. Log step `{"reseed": <u64>}`. |
| Checkpoint status `Checkpoint: none` or `T{n} · rerolls N` (code-read) | `index.html:351-356` | `ui/src/session.ts:412-415`; `ui/index.html:228` | ported | `Checkpoint: none` or `Checkpoint: T{n} · rerolls N`. |
| Save/load still correct after the 200-step undo ring drops old actions (code-read) | old position store is a full board snapshot | `ui/src/session.ts:15,123-147,412-422` | ported | `s.actions` is the untrimmed log; `toPositionLog` / replay use that full list, not the 200-step `past` ring. | 
| Active on bottom checkbox | `index.html:357-364` | `ui/index.html:224-227`; `ui/src/main.ts:712-723,773-777` | ported | Persistence — see §1. |
| Floating combat text checkbox (default on) | `index.html:366-371` | `ui/index.html:228-231`; `ui/src/main.ts:712-719,778-781` | ported | Persistence — see §6. |
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
| Escape closes history (code-read) | `index.html:988` | `ui/src/main.ts:497-504` | ported | Same result as §7. |
| Escape / click-outside does **not** cancel old targeting (no UI cancel) (code-read) | (none in UI files) | Escape and tap-elsewhere cancel pending (`ui/src/input.ts:137-154`) | ported | New-only extra; same Cancel chip as §5. |
| Ctrl/Cmd+Z undo; Ctrl+Y / Ctrl+Shift+Z redo (ignore when focus is an input) (code-read) | `index.html:266-275`; `src/boot/hotkeysBoot.ts:4-5` | `ui/src/main.ts:486-511` | ported | 
| F6 set checkpoint (code-read) | `src/boot/hotkeysBoot.ts:7-9` | `ui/src/main.ts:513-518` | ported | 
| F7 restore checkpoint (code-read) | — | `ui/src/main.ts:520-525` | ported | New-only extra. |
| F8 reroll checkpoint (code-read) | `src/boot/hotkeysBoot.ts:7-9` | `ui/src/main.ts:669-675` | ported | Same result as §8. |
| Unified pointer drag (mouse + touch), pointer capture (code-read) | `src/ui/pointerDragSession.ts` | `ui/src/drag.ts` | ported | 
| `touch-action: none` on draggables (code-read) | `css/cards.css` drag block | `ui/css/cards.css:176` | ported | `[data-pointer-draggable="true"]` sets `touch-action: none`. | 
| Browser context menu blocked on `.card`, `.zone`, `.leader`, `.evo-btn` (code-read) | `src/boot/contextMenu.ts:3-17` | `ui/src/input.ts:192-197` | ported | Right-click on a card, zone, leader, or Evo/Super never shows the OS menu. |
| Long-press tooltip (code-read) | — | `ui/src/render.ts:1361-1377` | ported | Same 400ms touch long-press as §4. | 
| Cancel drag on `pointercancel`, tab blur, `visibilitychange` (code-read) | `src/ui/pointerDragSession.ts:341-367` | `ui/src/drag.ts:284-287` | ported | 

---

## 10. Performance (user-noticeable)

| feature | old tool (file:line) | new client (file:line or "—") | status | gap spec |
|---|---|---|---|---|
| Zone reconcile by `data-uid`; reuse DOM when the fingerprint matches (code-read) | `src/ui/zones/index.ts:83-147`; `src/ui/zones/memoization.ts:165-225` | `ui/src/render.ts:336-371`; `ui/src/render/card.ts:74-87` | ported | 
| No full-page re-render of unrelated chrome (code-read) | `src/ui/render.ts` updates lists/crests in place | `ui/src/render.ts:535-540` history signature cache | ported | 
| rAF paint coalescing (code-read) | — (sync `render()` on every change, `src/ui/render.ts:70`) | `ui/src/main.ts:148-154` | ported | New-only extra; keep. `window.__arena.paintMs` budget is tested (`ui/tests/feedback.spec.ts:378-387`). |
| CSS-driven FCT (no JS animation loop) (code-read) | `css/floating-combat-text.css:2-3` | `ui/css/floating-combat-text.css:1-2` | ported | 
| Cancel in-flight image loads before tearing down a node (code-read) | `src/ui/releaseImageLoads.ts:13-35` | `ui/src/releaseImages.ts:2-15`; `ui/src/render.ts:382-388`; `ui/src/render/card.ts` crest teardown | ported | Card/crest teardown sets `img.src = ""` / removes the attribute so leftover decodes do not stall the next paint. |
| Catalog / deck JSON cached at boot; images use the browser cache (`referrerPolicy: no-referrer`) (code-read) | CDN + `.webp` fallback | `ui/src/catalog.ts:6-27`; `ui/src/images.ts:25-52` | ported | 
| No explicit image preload queue (code-read) | (none) | (none) | ported | 

---

## Gap list

Every former gap is `ported` or owner-dropped. Owner-dropped form chrome stays listed so it is not re-opened.

### Every turn

1. **Hand/board glow only for the acting player** — **`ported` 2026-09-11** (`ui/src/render.ts:208-218,282-294`).
2. **Yellow vs green glow + no E/A/C badges** — **`ported` 2026-09-11** (`ui/src/render/card.ts:71-94`). Cyan `{formLabel} {cost}` badge is **`dropped (owner decision 2026-09-11)`**.
3. **Stat colours buffed / damaged / debuffed** — **`ported` 2026-09-11** (`ui/src/render/card.ts:247-256`; `ui/css/cards.css:636-647`).
4. **`updateCard` countdown / overlays / evolved art** — **`ported` 2026-09-11** (`ui/src/render/card.ts:34-49,145-171`).
5. **Tooltip order + class/tribes/set + extras** — **`ported` 2026-09-11** (`ui/src/tooltip.ts:39-61`, smart anchor `ui/src/render.ts:1274`). Name → class/tribes + set → gates → description → crests / fused / Rally / Skybound / `+A/+D` / `Cannot play:`. Cost/stats line omitted (hotfix). Form-gate tooltip lines are **`dropped (owner decision 2026-09-11)`**.
6. **Play / fuse click map** — **`ported` 2026-09-11** (`ui/src/input.ts:170-214`; owner: keep the old mapping). Left-click/tap fuse-capable → Fuse (does not play). Right-click playable → play. Drag to own board → play.
7. **Rush-turn yellow vs green attack glow** — **`ported` 2026-09-11** (`ui/src/render.ts:282-288`; `engine/src/info.rs` `followers_only_this_turn`). Yellow requires a legal follower-only attack on the entry turn; clears on the opponent's turn.
8. **Pending attack/evolve cancel + Cancel chip** — **`ported` 2026-09-11** (`ui/src/render.ts:1205-1234`; `ui/src/input.ts:137-154`).
9. **Long-press tooltip + `touch-action: none`** — **`ported` 2026-09-11** (`ui/src/render.ts:1361-1377`; `ui/css/cards.css:176`).
10. **Tooltip live refresh + drag-pinned panel** — **`ported` 2026-09-11** as part of #5 (`ui/src/render.ts:1193-1228`). Open tooltip rebuilds on every paint; drag-pin at `top:12px; left:12px`.

### Common (most games)

11. **Follower floating combat text** — **`ported` 2026-09-11** (`ui/src/fct.ts:43-55`). Host is `[data-slot]` on that player’s board. Extras: cap 4, `--float-stack-index` × 18px, 0.45s flash, persist `svwb.floatingCombatText`.
12. **Mulligan marks on both hands + ✓** — **`ported` 2026-09-11** (`ui/src/render.ts:213,601-608`; `ui/src/render/card.ts:224-235`). Sequential acting player; 24px `#2ecc71` ✓ + solid green ring.
13. **History rows** — **`ported` 2026-09-11** (`ui/src/render.ts:616-705`): 22px cost square + `Name ×N`, grouped by name+cost, set badge, 198px hover art.
14. **History destroyed-owner attribution** — **`ported` 2026-09-11** (`ui/src/session.ts:163-189`). Owner is the side that held the card.
15. **Choice modal labels + Fuse Confirm + stale listeners + prompt/Undo** — **`ported` 2026-09-11** (`ui/src/render.ts:749-807,864-960,990-1007`; `ui/src/input.ts:119-125`). Title “Choose an effect:”; card names / 1-based modes; prompt-bar Confirm + Undo; document-level choose (no leftover card listeners).
16. **Engine-error toast** — **`ported` 2026-09-11** (`ui/src/main.ts:318-335`; `ui/css/modern-theme.css:986-1008`). Bottom-centre `#actionToast` dark pill, 1800ms, 0.18s fade.
17. **Undo symmetry inside a choice** — **`ported` 2026-09-11** (`ui/src/session.ts:262-272,297-302`). One Ctrl+Z = one applied action (one pick).
18. **Enemy leader outline only while pending** — **`ported` 2026-09-11** (`ui/src/render.ts:472-496`).
19. **Drag highlights ignore legality** — **`ported` 2026-09-11** (`ui/src/drag.ts:80-88`). Highlight only when `accepts(payload)`.
20. **Fuse chip** — **`ported` 2026-09-11** (`ui/src/render.ts:226-237`; `ui/css/client.css:234-252`). Gold ring + bottom-centre **Fuse** chip.
21. **Engage right-click** — **`ported` 2026-09-11** as part of #6 (`ui/src/input.ts:182-214`).
22. **Can’t-attack overlay** — **`ported` 2026-09-11** (`ui/src/render/card.ts:282-290`; `engine/src/info.rs` `cannot_attack_reason`). Crossed chains on any printed lock, not sickness.
23. **Keyword icon swap + Ongoing icon** — **`ported` 2026-09-11** (`ui/src/render/card.ts:306-317`). `.swap-2` 2s; `.swap-3`/`.swap-4` 3s; Ongoing via `cardText.tags`.
24. **Spellboost badge** — **`ported` 2026-09-11** (`ui/src/render/card.ts:189-196`). Blue circle, white count, under the cost when the card has Spellboost and `spellboost_count ≥ 1`.
25. **Leader barrier ring** — **`ported` 2026-09-11** (`ui/src/render.ts:456-462`; `engine/src/info.rs` `has_leader_barrier`).
26. **Escape closes history** — **`ported` 2026-09-11** (`ui/src/main.ts:497-533`).
27. **Copy seed “Copied”** — **`ported` 2026-09-11** (`ui/src/main.ts:782-796`). Button reads `Copied` for 1200ms.
28. **Active-on-bottom persist** — **`ported` 2026-09-11** (`ui/index.html` inline boot; `ui/src/main.ts:712-777`). `svwb.activeOnBottom` applied before first paint.

### Occasional

29. **Faith crest badge** — **`ported` 2026-09-11** (`ui/src/render/card.ts:428-432`; tooltip `Faith — {count}` in `ui/src/tooltip.ts:131-141`).
30. **Barrier overlay + flash/pop** — **`ported` 2026-09-11** (`ui/src/render/card.ts:209,327-398`). Cyan ring + particles; 250ms flash on gain; 350ms pop on loss; driven from the `traits` signature diff.
31. **Can’t-be-destroyed overlay** — **`ported` 2026-09-11** (`ui/src/render/card.ts:327`). Gold field + five particles while `cantBeDestroyedByAbilities` is on.
32. **Card-name highlights + crest panels + fused loot + Skybound + buff delta + play-blocked** in the tooltip — **`ported` 2026-09-11** as part of #5 (see §4).
33. **Amulet named-counter badge** — **`ported` 2026-09-11** (`ui/src/render/card.ts:22-31,274-283`; `engine/src/info.rs` `named_counter`). First of **X, then Y, then Z**.
34. **Spell-cast flash** — **`ported` 2026-09-11** (`ui/src/render.ts:382-410`). Brightness 1.5, cyan drop-shadow, scale 1.1 → 1, 0.5s ease-out; detached clone if the hand card is removed the same paint.
35. **Terminal overlay reason + rematch-same-seed** — **`ported` 2026-09-11** (`ui/src/render.ts:1054-1093`). Reason `Deck-out` or `Lethal`; Rematch (same seed) / (new seed) / New Game. Swap-sides dropped (owner 2026-09-11).
36. **Terminal readout label** — **`ported` 2026-09-11** (`ui/src/render.ts:118-127`). `Turn N · A` / `Blue (A) wins` / `Red (B) wins` / `Draw`.
37. **Empty seed = random** — **`ported` 2026-09-11** (`ui/src/main.ts:244-245,320-323`). Placeholder “random if empty”; empty field rolls a u64.
38. **Share URL `a`/`b` aliases on write** — **`ported` 2026-09-11** (`ui/src/share.ts:26-40`). Writes `deckA`/`deckB`/`mode` and `a`/`b`.
39. **No OS menu on the play surface** — **`ported` 2026-09-11** as part of #6 (`ui/src/input.ts:192-197`).
40. **Fuse via drag-release in hand** — **`ported` 2026-09-11** as part of #6 (`ui/src/drag.ts:228-236`).
41. **Choice dismisses on click** — **`ported` 2026-09-11** as part of #15 (`ui/src/render.ts:752-759`).
42. **No hitch on card/crest teardown** — **`ported` 2026-09-11** (`ui/src/releaseImages.ts:2-15`). In-flight `<img>` loads are cancelled on card/crest teardown.

### Rare (practice drawer)

43. **Save/load after the 200-step ring** — **`ported` 2026-09-11** (`ui/src/session.ts:15,123-147,412-422`). `s.actions` is the untrimmed log.
44. **Position option `Name · T{n} · time`; Rename; Del** — **`ported` 2026-09-11** (`ui/src/main.ts:747-809`). Option `{name} · T{turn} · {locale time}`; Rename / Del; JSON keeps name, turn, timestamp.
45. **Reroll + F8** — **`ported` 2026-09-11** (`ui/src/session.ts:395-415`; `State::reseed`). New RNG branch from the checkpoint; status `Checkpoint: T{n} · rerolls N`; log step `{"reseed": <u64>}`.
46. **Export List** — **`ported` 2026-09-11** (`ui/src/main.ts:813-833`). Blue deck as `Nx Name` lines, clipboard + `#exportListPanel`.
47. **Icarus `!` badge** — **`dropped (not representable)`**. The engine has no Icarus buff, so there is no `traits` / `full()` fact to paint.

---

## Status counts

| status | rows |
|---|---|
| ported | 203 |
| missing | 0 |
| dropped (by design) | 11 |
| dropped (not representable) | 1 |
| dropped (owner decision 2026-09-11) | 2 |
| **total** | **217** |

Counts are feature rows in §§1–10 (the gap list is a reordering, not extra rows). SF (2026-09-11): undo-inside-choice is `ported` (one pick per press); the cyan form-cost badge and form-gate tooltip lines are `dropped (owner decision 2026-09-11)`.

## Owner decided 2026-09-11

See the top-of-doc block. Confirmed and implemented this round:

- Click map: keep the old mapping (left-click fuse-capable → Fuse; right-click playable → play; drag-release in hand → Fuse; engage by right-click).
- Rematch: Rematch (same seed), Rematch (new seed), New Game. No swap-sides; first player follows the drawer A/B/coin setting.
- Import Deck paste, burn-preview, god-mode, `?test=1`, UNIMPL badges, smaller top-hand scale, and the other listed extras stay `dropped (by design)`.
- Icarus badge is not representable (engine has no Icarus buff).
