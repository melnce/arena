# Milestones

| | delivers | gate |
|---|---|---|
| **M0** | this design PR | owner reads and approves the schema |
| M1 | engine core + Basic set (56 cards) + one meta-deck mirror, headless; benchmark | differential harness green vs the old engine on that deck; explicit go/no-go on Rust |
| M2 | all 17 meta decks (199 cards), headless, soak | differential green; matchup statistics possible here, before any UI |
| M3 | all 516 rotation cards + tokens + crests | differential green on the full pool |
| M4 | thin web client over the WASM engine, ported look | e2e parity with the old UI's flows |
| M5 | bot: action encoding, determinized search, policy hooks | mulligan / drawn-card / matchup / tech-card win rates |

## M1 card list

**Source:** every id in `official-meta.json` with `card_set_id == 10000` and `is_include_rotation` (measured **56**), plus the non-Basic cards of the chosen mirror deck.

**Mirror deck: Aggro Abysscraft** (`decks/aggro_abysscraft.json` in the old repo).

Why: among the 17 meta decks, several have 14 distinct cards (the minimum). Aggro Abysscraft has the **most Basic-set overlap** of those (2 names: Lilith, Enchanting Succubus `10052110`; Devious Lesser Mummy `10051130`). Fewest new authorings for a real matchup.

| id | count | name | in Basic? |
|---|---|---|---|
| 10851120 | 3 | Lilith, Devilish Cutie | no |
| 10052110 | 3 | Lilith, Enchanting Succubus | yes |
| 10501110 | 3 | Monster Litterateur | no |
| 10051130 | 2 | Devious Lesser Mummy | yes |
| 10851130 | 3 | Limil, Devilish Bunny | no |
| 10651110 | 3 | Reverent Demon | no |
| 10951110 | 3 | Ruthless Blitzer | no |
| 10552310 | 3 | Tyrannical Fists | no |
| 10452130 | 3 | Baal, Elemental Resonance | no |
| 10753310 | 3 | Hark to the Night Song | no |
| 10851110 | 2 | Anisage, Clear Resolve | no |
| 10853110 | 3 | Suzy, Sincere Hexcaster | no |
| 10953110 | 3 | Rampaging Commander | no |
| 10954120 | 3 | Garodeth vs. Zeth | no |

M1 also authors every token/crest those 56+12 cards can produce (same reachability walk as the pool).

**Gate:** differential green vs the old engine on seeded Aggro Abysscraft mirrors; benchmark games/second for both engines; go/no-go on Rust.

## M2 card list

**Source:** the 17 files in `decks/manifest.json` with `"category": "deck"` (ignore `0_testing_vanilla`). Measured **199** distinct rotation card ids (command: resolve each `name` through `official-meta.json` rotation ids and union).

### Aggro Abysscraft (14)

10851120, 10052110, 10501110, 10051130, 10851130, 10651110, 10951110, 10552310, 10452130, 10753310, 10851110, 10853110, 10953110, 10954120

### Amulet Havencraft (14)

See `decks/amulet_havencraft.json` in the old repo at `8f491f9`. Names as committed there; all resolved, 0 missing.

### Antemaria Dragoncraft (14)

`decks/antemaria_dragoncraft.json`

### Artifact Portalcraft (16)

`decks/artifact_portalcraft.json` — includes the fuse chain.

### Barbaros Swordcraft (15)

`decks/barbaros_swordcraft.json`

### Buff Forestcraft (17)

`decks/buff_forestcraft.json`

### Cutthroat Portalcraft (38)

`decks/cutthroat_portalcraft.json` — largest distinct count.

### Evolution Forestcraft (16)

`decks/evolution_forestcraft.json`

### Evolution Havencraft (14)

`decks/evolution_havencraft.json`

### Kukishiro Havencraft (15)

`decks/kukishiro_havencraft.json`

### Lhynkal Runecraft (15)

`decks/lhynkal_runecraft.json`

### Midrange Abysscraft (15)

`decks/midrange_abysscraft.json`

### Rally Swordcraft (16)

`decks/rally_swordcraft.json`

### Ramp Dragoncraft (14)

`decks/ramp_dragoncraft.json`

### Sephie Runecraft (15)

`decks/sephie_runecraft.json`

### Spell Runecraft (14)

`decks/spell_runecraft.json`

### Thestae Forestcraft (15)

`decks/thestae_forestcraft.json`

Full per-deck id lists (measured from `/tmp/old/decks/*.json` + rotation name map):

### Aggro Abysscraft (`aggro_abysscraft`, 14 distinct)

`10851120` Lilith, Devilish Cutie ×3, `10052110` Lilith, Enchanting Succubus ×3, `10501110` Monster Litterateur ×3, `10051130` Devious Lesser Mummy ×2, `10851130` Limil, Devilish Bunny ×3, `10651110` Reverent Demon ×3, `10951110` Ruthless Blitzer ×3, `10552310` Tyrannical Fists ×3, `10452130` Baal, Elemental Resonance ×3, `10753310` Hark to the Night Song ×3, `10851110` Anisage, Clear Resolve ×2, `10853110` Suzy, Sincere Hexcaster ×3, `10953110` Rampaging Commander ×3, `10954120` Garodeth vs. Zeth ×3

### Amulet Havencraft (`amulet_havencraft`, 14 distinct)

`10761210` Earrings of Sunlight ×3, `10062210` Winged Statue ×3, `10863210` Academy Hijinks ×3, `10961210` Roaring Basilica ×3, `10662210` Scripture of Salvation ×3, `10762210` Timepiece of Perfection ×2, `10661210` Unholy Water ×3, `10763210` Trident of Eroding Tides ×3, `10664110` Kandima, Sublime Hatred ×3, `10663210` Sublime Eld Tome ×3, `10661110` Prostrating Coward ×3, `10964120` Omerio, Winged Revenant ×3, `10764120` Initia, Chief Ordination Officer ×2, `10664120` Lyanthoth, Eld Tome ×3

### Antemaria Dragoncraft (`antemaria_dragoncraft`, 14 distinct)

`10941310` Drake Whelp's Tantrum ×3, `10941110` Ripper-Clawed Thief ×3, `10943310` Artiglio ×3, `10942110` High-Spirited Marauder ×3, `10542120` Jellyfish Dancer ×3, `10842120` Kimika, Cook of Happiness ×3, `10543310` Sloth of the Crestpetal ×3, `10644120` Vorlalai, Eld Blades ×3, `10641110` Resolute Dragonewt ×3, `10641310` Advent of the Eld Blades ×3, `10741120` Carrier Wyvern ×2, `10943110` Barren-Earth Tyrant ×3, `10944110` Antemaria, Piercing Convict ×3, `10644110` Sagatsumatsu, Fair Beheader ×2

### Artifact Portalcraft (`artifact_portalcraft`, 16 distinct)

`10771310` Freerunning ×3, `10573310` Sincerity of the Dewdrop ×3, `10772110` Cool Courier ×1, `10874120` Eudie, Your Dependable Mentor ×3, `10574120` Imari, Dewdrop ×3, `10471130` Isaac, Congenial Engineer ×1, `10674120` Yog-Zentha, Eld Axe ×3, `10773110` Brazen Broadcaster ×3, `10774120` Myuu, Hot on His Heels ×2, `10874110` Asher & Lydia, Paths Beyond ×3, `10404110` Sandalphon, Primarch Successor ×1, `10671110` Shoddy Plaything ×3, `10974120` Aizeden, Killshot Revenant ×3, `10674110` Camiscilla, Unfeeling Heart ×2, `10673110` Ludicrous Ordnance ×3, `10774110` Scarlet, Anathema of Dislocation ×3

### Barbaros Swordcraft (`barbaros_swordcraft`, 15 distinct)

`10021110` Flashstep Quickblader ×3, `10722310` Orchestrated Silence ×3, `10623310` Ruthless Eld Sword ×2, `10921110` Open-Sea Scout ×3, `10823310` Slice of Domesticity ×2, `10624120` Yidmetra, Eld Sword ×2, `10922310` Severed Ties ×3, `10523310` Splendor of the Goldbloom ×3, `10922110` Whirlpool Gunner ×3, `10923310` L'Age d'Or ×3, `10424110` Zeta & Bea, Crimson and Blue ×3, `10923110` Roughwater First Mate ×3, `10524120` Unkei, Goldbloom ×3, `10924110` Barbaros, Rebellious Convict ×3, `10824120` Mars, Conflagrant Commander ×1

### Buff Forestcraft (`buff_forestcraft`, 17 distinct)

`10511310` Flight of the Swarmpetal ×2, `10403120` Lyria, Skydestined ×3, `10511110` Prudent Tanuki ×1, `10812110` Ruflet, Primeval Fairy ×3, `10814120` Tia, Eternal Crystalian ×3, `10811120` Citrus, Heretical Hermit ×2, `10914110` Magachiyo, Aromatic Convict ×2, `10514120` Miroku, Swarmpetal ×3, `10413110` Cupitan, Iridescent Archer ×3, `10812120` Lycoris, Barbs of Passion ×1, `10813310` Curiosity Abounds ×3, `10513110` Spirited Skipper ×3, `10813110` Michelle, Kind Mindreader ×3, `10404110` Sandalphon, Primarch Successor ×1, `10814110` Setus & Maisha, Bladerights ×3, `10614110` Althenia, Nurturing Bloom ×3, `10904110` Zerael, Sundered Rebirth ×1

### Cutthroat Portalcraft (`cutthroat_portalcraft`, 38 distinct)

`10971110` Bluerust Underling ×1, `10972310` Disgraceful Banishment ×1, `10771310` Freerunning ×1, `10571310` Light of the Dewdrop ×1, `10573310` Sincerity of the Dewdrop ×1, `10503210` World of Games ×1, `10772110` Cool Courier ×1, `10974110` Cutthroat, Fluxblade Convict ×3, `10874120` Eudie, Your Dependable Mentor ×1, `10574120` Imari, Dewdrop ×1, `10471130` Isaac, Congenial Engineer ×1, `10403120` Lyria, Skydestined ×1, `10071120` Puppet Lancer ×1, `10072210` Puppet Theater ×1, `10674120` Yog-Zentha, Eld Axe ×1, `10704120` Altaro, Mayor of Babelon ×1, `10773110` Brazen Broadcaster ×1, `10872120` Lazuli, Gateway Connector ×1, `10471110` Sho, Reborn Night King ×1, `10574110` Slaus, Revolving Wheel of Fortune ×1, `10472310` Stone Breaker ×1, `10771110` Brusque Barkeep ×1, `10403110` Gran & Djeeta, Valiant Skyfarers ×1, `10972110` Ironwork Bodyguard ×1, `10973310` Soulforge ×1, `10874110` Asher & Lydia, Paths Beyond ×1, `10401110` Katalina, Sky's Protector ×1, `10474110` Lu Woh, Light Personified ×1, `10973110` Steelforged Right Hand ×1, `10473310` Chaos Legion ×1, `10672310` Myriad Designs ×1, `10404110` Sandalphon, Primarch Successor ×1, `10671110` Shoddy Plaything ×1, `10974120` Aizeden, Killshot Revenant ×1, `10674110` Camiscilla, Unfeeling Heart ×1, `10673110` Ludicrous Ordnance ×1, `10774110` Scarlet, Anathema of Dislocation ×1, `10474120` Beelzebub, Supreme King ×1

### Evolution Forestcraft (`evolution_forestcraft`, 16 distinct)

`10614120` Sathanid, Eld Lance ×3, `10414110` Ewiyar, Wind Personified ×3, `10511310` Flight of the Swarmpetal ×2, `10612110` Kindly Executor ×3, `10914110` Magachiyo, Aromatic Convict ×2, `10514120` Miroku, Swarmpetal ×3, `10613310` Nurturing Eld Lance ×3, `10413110` Cupitan, Iridescent Archer ×3, `10403110` Gran & Djeeta, Valiant Skyfarers ×3, `10611110` Motherly Forestdweller ×3, `10613110` Merciful Attendant ×2, `10513110` Spirited Skipper ×2, `10714120` Great Hart of the Glacial Realm ×1, `10404110` Sandalphon, Primarch Successor ×3, `10614110` Althenia, Nurturing Bloom ×3, `10804110` Alabaster Bahamut ×1

### Evolution Havencraft (`evolution_havencraft`, 14 distinct)

`10863210` Academy Hijinks ×3, `10861110` Lilium, Witch of the Tomes ×3, `10403120` Lyria, Skydestined ×3, `10763210` Trident of Eroding Tides ×3, `10461110` Troue, Heroic Visionary ×3, `10564110` Sofina, Inspiring Strength ×3, `10864120` Zoe, Dazzling Hope ×3, `10964110` Erralde, Signet Convict ×3, `10963210` Juratio ×3, `10404110` Sandalphon, Primarch Successor ×2, `10862120` Viche, Abyssal Researcher ×3, `10963110` Executor of the Vow ×3, `10864110` Verdilia & Castelle, Sisters ×3, `10804120` Olivia, Proud Dark Angel ×2

### Kukishiro Havencraft (`kukishiro_havencraft`, 15 distinct)

`10561120` Bouquet Believer ×1, `10761210` Earrings of Sunlight ×3, `10962310` Vow of Devotion ×3, `10503210` World of Games ×3, `10863110` Colette, Holy Exorcist ×3, `10463210` De La Fille's Gleaming Gems ×3, `10563210` Resolve of the Mistbloom ×3, `10763210` Trident of Eroding Tides ×3, `10461110` Troue, Heroic Visionary ×3, `10562120` Desperate Shrinemouse ×1, `10503310` Fate of the World ×3, `10661110` Prostrating Coward ×3, `10864120` Zoe, Dazzling Hope ×3, `10902110` Blade Angel ×2, `10564120` Kukishiro, Mistbloom ×3

### Lhynkal Runecraft (`lhynkal_runecraft`, 15 distinct)

`10031310` Foresight ×3, `10534110` Lhynkal, Wandering Fool ×3, `10831310` Stormy Blast ×3, `10503210` World of Games ×3, `10832310` Harmonious Meal ×2, `10531310` Metamorphosis of the Dawnblossom ×3, `10803310` Wills United ×3, `10833310` Amethyst's Naptime ×3, `10833110` Tico, Mysterian Spellcrafter ×3, `10832110` Sammy & Marie, Flowers of Joy ×3, `10834110` Tetra & Ladica, Forest BFFs ×3, `10532120` Woodsong Haikumaster ×1, `10834120` Ginger, Disastrous Word ×2, `10534120` Ara, Dawnblossom ×3, `10934120` Phylene, Cleansing Revenant ×2

### Midrange Abysscraft (`midrange_abysscraft`, 15 distinct)

`10851120` Lilith, Devilish Cutie ×3, `10052110` Lilith, Enchanting Succubus ×1, `10951120` Netherworld Lieutenant ×3, `10751120` Raz, Demon on the Drums ×3, `10452130` Baal, Elemental Resonance ×3, `10654120` Bibatii, Eld Sight ×2, `10552110` Fickle Necromancer ×3, `10753310` Hark to the Night Song ×3, `10752110` Highwire Feline ×3, `10754110` Adahime, Anathema of Death ×3, `10952110` Void Colonel ×3, `10954110` Istyndet vs. Mitilykket ×2, `10954120` Garodeth vs. Zeth ×3, `10854110` Itsurugi & Taketsumi, Brothers ×2, `10754120` Macmillan, Reaper of Ceremonies ×3

### Rally Swordcraft (`rally_swordcraft`, 16 distinct)

`10722310` Orchestrated Silence ×3, `10722110` Sharp-Eared Operative ×3, `10624120` Yidmetra, Eld Sword ×3, `10721120` High-Strung Liaison ×3, `10922310` Severed Ties ×1, `10724110` Gildaria, Anathema of Attunement ×3, `10822310` Shared Existence ×2, `10424110` Zeta & Bea, Crimson and Blue ×3, `10824110` Bunny & Baron, Fate's Bullet ×3, `10723110` Knellclaw Lieutenant ×3, `10722120` Metronomic Medic ×3, `10724120` Cesar, Accordant Major ×3, `10423110` Golden Knight, True King's Blade ×2, `10821110` Naht & Vince, Force and Order ×1, `10824120` Mars, Conflagrant Commander ×2, `10924120` Beltezore, Valorous Revenant ×2

### Ramp Dragoncraft (`ramp_dragoncraft`, 14 distinct)

`10741110` Dragonewt Promoter ×3, `10842120` Kimika, Cook of Happiness ×3, `10403120` Lyria, Skydestined ×3, `10543310` Sloth of the Crestpetal ×3, `10644120` Vorlalai, Eld Blades ×3, `10042310` Dragonsign ×3, `10603210` Dark Dimensions ×2, `10444120` Zooey, Ally of the World ×3, `10944120` Normagdala, Ravening Revenant ×3, `10644110` Sagatsumatsu, Fair Beheader ×3, `10844120` Lumiore & Argente, Shining Wings ×3, `10804110` Alabaster Bahamut ×3, `10744110` Burnite, Anathema of Ash ×2, `10544110` Erntz, Governing Justice ×3

### Sephie Runecraft (`sephie_runecraft`, 15 distinct)

`10031310` Foresight ×3, `10931310` Miscalculated Experiment ×3, `10831310` Stormy Blast ×3, `10403120` Lyria, Skydestined ×1, `10531310` Metamorphosis of the Dawnblossom ×2, `10931110` Obsessed Test Subject ×3, `10803310` Wills United ×3, `10833310` Amethyst's Naptime ×3, `10932310` Humane Love ×3, `10932110` Enamored Researcher ×3, `10933110` Ecstatic Scholar ×3, `10503310` Fate of the World ×2, `10933310` Obsidian Raven ×3, `10834110` Tetra & Ladica, Forest BFFs ×2, `10934110` Sephie, Maven Convict ×3

### Spell Runecraft (`spell_runecraft`, 14 distinct)

`10031310` Foresight ×3, `10931310` Miscalculated Experiment ×3, `10831310` Stormy Blast ×3, `10832310` Harmonious Meal ×3, `10831110` Meowskers, Roly-Poly Mk II & Djeana ×1, `10833310` Amethyst's Naptime ×3, `10833110` Tico, Mysterian Spellcrafter ×3, `10434110` Wamdus, Water Personified ×3, `10832110` Sammy & Marie, Flowers of Joy ×3, `10834110` Tetra & Ladica, Forest BFFs ×3, `10532120` Woodsong Haikumaster ×3, `10834120` Ginger, Disastrous Word ×3, `10534120` Ara, Dawnblossom ×3, `10934120` Phylene, Cleansing Revenant ×3

### Thestae Forestcraft (`thestae_forestcraft`, 15 distinct)

`10711120` Elven Trapper ×3, `10911110` Sprouting Initiate ×3, `10503210` World of Games ×3, `10912110` Leafshadow Assassin ×3, `10403120` Lyria, Skydestined ×3, `10912310` Verdant Ring Kindred ×3, `10712110` Virewind Fencer ×2, `10913110` Virid Lieutenant ×3, `10914110` Magachiyo, Aromatic Convict ×3, `10514120` Miroku, Swarmpetal ×3, `10913310` Crimson Incense ×1, `10714110` Thestae, Anathema of Distortion ×3, `10714120` Great Hart of the Glacial Realm ×3, `10814110` Setus & Maisha, Bladerights ×3, `10914120` Hien, Redolent Revenant ×1

## M3 card list

All 516 rotation collectibles + 56 reachable tokens + 43 crest/faith entries (38 Crest + 5 Faith). Same derivation as M0 § pool. Tokens: walk `related_card_ids` from the 516, keep `is_token == true`.

**Gate:** differential green on the full pool.

## M4 / M5

M4 is a thin web client over a `wasm/` wrapper. M5 is the bot (action encoding, determinized search, policy hooks). Neither starts before the owner approves M0 and the M1 Rust go/no-go.
