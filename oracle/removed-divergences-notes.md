# Removed allowlist rows (main → play-time-selection branch)

Main had **84** entries; new allowlist has **147** entries. **38** main rows are absent (different `(trace, i, path)` or fixed).

## Adahime / Divine Thunder error rows — different step index (1)

- `elf-neanisu2-mirror/trace-20260910-13.jsonl` i=24 `error` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts, so the fifth advance destroys World of Games and Last Words draw; the old engine did not count the enemy card and recorded no draw pick

## E39 hand-card ordering — superseded by play-time selection timing (1)

- `elf-neanisu2-mirror/trace-20260910-19.jsonl` i=36 `players.b.hand[3].card` — E39 / owner 2026-09-10: World of Games Last Words from a play-reaction destroy resolve before Fanfare; the old engine ran Leafshadow Assassin 10912110 Fanfare add-Fairy first then the draws

## Other — ruling unchanged but first divergence key moved or trace now green (14)

- `elf-neanisu2-mirror/trace-20260910-18.jsonl` i=131 `players.a.field[1].max_defense` — owner ruling 2026-09-10 (like Baal): Setus & Maisha, Bladerights 10814110 'Give all other allied followers on the field +1/+1' raises max_defense; the old engine left a damaged or evolved follower's max at the pre-buff value
- `ramp-37772-mirror/trace-20260910-15.jsonl` i=57 `players.a.leader_defense` — owner ruling 2026-09-10: Depths of the Eld Blades discarded by Spilling Red ('When this card is discarded, deal 1 damage to the enemy leader and restore 1 defense to your leader') resolves after the destroy, not before the second selection
- `ramp-claywies-mirror/trace-20260910-18.jsonl` i=50 `players.a.leader_defense` — owner ruling 2026-09-10: Depths of the Eld Blades 90044330 discarded by Spilling Red 10642310 ('When this card is discarded, deal 1 damage to the enemy leader and restore 1 defense to your leader') resolves after the destroy, not before the second selection
- `ramp-claywies-mirror/trace-20260910-8.jsonl` i=43 `players.b.field[2]` — owner ruling 2026-09-10: reactions queued during an effect wait until the list completes, including across a player choice; Vorlalai discarded by Lumiore & Argente 10844120 ('Select 2 cards in your hand and discard them') summons after both discards and the damage, not after the first selection
- `royal-nattui-mirror/trace-20260910-26.jsonl` i=63 `players.a.field[0].max_defense` — owner ruling 2026-09-10 (like Baal): Gilded Necklace 90021340 'give it +0/+1 and Ward' raises max_defense; the old engine left Super-Evolved Unkei, Goldbloom 10524120 max at 7
- `rune-mach15-mirror/trace-20260910-11.jsonl` i=107 `players.a.banished.90031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto Magic Sediment 90031210 banishes the older holder (no shadow); the old engine cemeteries it
- `rune-mach15-mirror/trace-20260910-15.jsonl` i=132 `players.a.banished.10031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto another Earth Sigil amulet banishes the older holder (no shadow); the old engine cemeteries it
- `rune-mach15-mirror/trace-20260910-17.jsonl` i=61 `players.b.banished.10031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto another Earth Sigil amulet banishes the older holder (no shadow); the old engine cemeteries it
- `rune-mach15-mirror/trace-20260910-19.jsonl` i=100 `players.a.banished.90031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto Magic Sediment 90031210 banishes the older holder (no shadow); the old engine cemeteries it
- `rune-mach15-mirror/trace-20260910-2.jsonl` i=39 `players.a.banished.90031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto Magic Sediment 90031210 banishes the older holder (no shadow); the old engine cemeteries it
- `rune-mach15-mirror/trace-20260910-23.jsonl` i=32 `players.b.banished.10031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto another Earth Sigil amulet banishes the older holder (no shadow); the old engine cemeteries it
- `rune-mach15-mirror/trace-20260910-24.jsonl` i=52 `players.b.banished.90031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto Magic Sediment 90031210 banishes the older holder (no shadow); the old engine cemeteries it
- `rune-mach15-mirror/trace-20260910-29.jsonl` i=99 `players.a.banished.10031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto another Earth Sigil amulet banishes the older holder (no shadow); the old engine cemeteries it
- `rune-mach15-mirror/trace-20260910-8.jsonl` i=95 `players.a.banished.90031210` — official glossary Earth Sigil + owner 2026-09-10 'yes banish them instead': playing Witch's New Brew 10031210 onto Magic Sediment 90031210 banishes the older holder (no shadow); the old engine cemeteries it

## Spilling Red — divergence moved from field snapshot to cemetery/play-time path (4)

- `ramp-37772-mirror/trace-20260910-13.jsonl` i=52 `players.b.field[1]` — owner ruling 2026-09-10: reactions queued during an effect wait until the list completes, including across a player choice; Vorlalai discarded by Spilling Red summons after the destroy, not before the second selection
- `ramp-37772-mirror/trace-20260910-16.jsonl` i=51 `players.a.field[2]` — owner ruling 2026-09-10: reactions queued during an effect wait until the list completes, including across a player choice; Vorlalai discarded by Spilling Red summons after the destroy, not before the second selection
- `ramp-claywies-mirror/trace-20260910-4.jsonl` i=59 `players.a.field[4]` — owner ruling 2026-09-10: reactions queued during an effect wait until the list completes, including across a player choice; Vorlalai discarded by Spilling Red 10642310 summons after the destroy, not before the second selection
- `ramp-claywies-mirror/trace-20260910-6.jsonl` i=28 `players.a.field[2]` — owner ruling 2026-09-10: reactions queued during an effect wait until the list completes, including across a player choice; Vorlalai discarded by Spilling Red 10642310 summons after the destroy, not before the second selection

## WoG countdown — superseded by play-time selection rows at new `(i, path)` (17)

- `elf-neanisu2-mirror/trace-20260910-10.jsonl` i=15 `players.b.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-14.jsonl` i=160 `players.a.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-15.jsonl` i=92 `players.a.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-21.jsonl` i=6 `players.b.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-23.jsonl` i=22 `players.a.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-24.jsonl` i=38 `players.a.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-26.jsonl` i=89 `players.b.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-27.jsonl` i=40 `players.a.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-28.jsonl` i=6 `players.b.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-29.jsonl` i=16 `players.b.field[1].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-3.jsonl` i=10 `players.a.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-5.jsonl` i=53 `players.b.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-6.jsonl` i=40 `players.a.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `elf-neanisu2-mirror/trace-20260910-7.jsonl` i=109 `players.b.field[0].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `rune-mach15-mirror/trace-20260910-21.jsonl` i=56 `players.b.field[1].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `rune-mach15-mirror/trace-20260910-27.jsonl` i=81 `players.b.field[2].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only
- `rune-mach15-mirror/trace-20260910-28.jsonl` i=48 `players.b.field[1].countdown` — official Q&A (World of Games / Divine Thunder): an enemy card with the same base cost counts; the old engine counted allied cards only

## afnm Exact Copy — same ruling, different trace step after play-time pin (1)

- `afnm-minatodao-mirror/trace-20260910-21.jsonl` i=93 `players.a.field[3].attack` — rules/official-glossary.md Exact Copy: 'An exact copy of a card retains any damage and effects on the original' — a plain copy does not; Depths of the Eld Axe 90074320 'add a copy of it to your hand'; the old engine played the copy as an evolved 7/7 (Asher & Lydia 10874110)

