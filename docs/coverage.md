# Coverage — Rotation pool (572 cards + 43 crests)

Judged from printed text in `/tmp/old` at `8f491f9` (`cards/all.json`, `cards/token_details.json`, `specific_effects[].skill_text` with markup stripped). v1 ops were not used as evidence.

Status:

- `expressible` — the closed schema can write the printed text (one construction per meaning).
- `needs: <construct>` — a named construct is missing; do not guess.
- `question: <line>` — behaviour is unclear; listed in the PR for the owner.

## Cards

| id | name | triggers | ops/combinators | conditions | status |
|---|---|---|---|---|---|
| `10001110` | Indomitable Fighter | — | mode:enhance | — | expressible |
| `10001120` | Leah, Bellringer Angel | lastWords, evolve | draw | — | expressible |
| `10001130` | Quake Goliath | — | — | — | expressible |
| `10001210` | Detective's Lens | engage | destroy | — | expressible |
| `10002110` | Arriet, Luxminstrel | evolve, superEvolve | if/else, restore | — | expressible |
| `10002120` | Caravan Mammoth | — | — | — | expressible |
| `10002210` | Adventurers' Guild | fanfare, engage | draw, destroy | — | expressible |
| `10011110` | Fairy Tamer | fanfare | — | — | expressible |
| `10011120` | Stray Beastman | fanfare | — | — | expressible |
| `10011130` | Gentle Treant | fanfare, strike | restore | combo | expressible |
| `10011210` | Wild Profusion | fanfare, enter/when | damage | — | expressible |
| `10012110` | May, Journey Elf | fanfare | damage | combo | expressible |
| `10012120` | Selwyn, Sonic Archer | superEvolve | — | — | expressible |
| `10012310` | Bug Alert | — | damage | — | expressible |
| `10021110` | Flashstep Quickblader | — | — | — | expressible |
| `10021120` | Arms Peddler | lastWords | draw | — | expressible |
| `10021130` | Centaur Centurion | — | — | — | expressible |
| `10021310` | Way of the Maid | — | draw | — | expressible |
| `10022110` | Royal Coachwoman | lastWords | summon | — | expressible |
| `10022120` | Rusty, Luxcard Trickster | superEvolve | draw | — | expressible |
| `10022210` | Ancestral Crown | enter/when | — | — | expressible |
| `10031110` | Dazzling Runeknight | fanfare | choose, pay, spellboostHand | — | expressible |
| `10031210` | Witch's New Brew | fanfare, engage | draw | — | expressible |
| `10031310` | Foresight | — | draw | — | expressible |
| `10031320` | Truth Summons | — | summon | — | expressible |
| `10032110` | Remi & Rami, Two-Faced Witch | fanfare, superEvolve | summon, pay | — | expressible |
| `10032120` | Blaze Destroyer | spellboost | — | — | expressible |
| `10032310` | Arcane Eruption | — | draw, damage, pay | — | expressible |
| `10041110` | Searing Firenewt | fanfare | damage | — | expressible |
| `10041120` | Axe-Wielding Dragonslayer | — | — | — | expressible |
| `10041130` | Warrior of the Deep | fanfare | damage | — | expressible |
| `10041310` | Strike of the Dragonewt | — | if/else, damage | overflow | expressible |
| `10042110` | Draconic Berserker | evolve, superEvolve | if/else, damage | — | expressible |
| `10042120` | Battleforged Dragon Keeper | — | summon, mode:enhance | — | expressible |
| `10042310` | Dragonsign | — | draw | — | expressible |
| `10051110` | Mistress of the Fanged | — | — | — | expressible |
| `10051120` | Night Fiend | fanfare | damage | — | expressible |
| `10051130` | Devious Lesser Mummy | fanfare | pay | — | expressible |
| `10051310` | Chaos Cyclone | — | choose, reanimate, draw | — | expressible |
| `10052110` | Lilith, Enchanting Succubus | lastWords | — | — | expressible |
| `10052120` | Amorous Necromancer | evolve, superEvolve | summon | — | expressible |
| `10052310` | Soul Predation | — | draw, destroy | — | expressible |
| `10061110` | Soulcure Sister | fanfare | restore | — | expressible |
| `10061120` | Fox of Purity | — | — | — | expressible |
| `10061130` | Winged Warrior | fanfare, evolve | replicate | — | expressible |
| `10061210` | Avian Statue | lastWords, engage | summon | — | expressible |
| `10062110` | Ironfist Priest | evolve, superEvolve | if/else, banish | — | expressible |
| `10062120` | Sacred Griffon | engage | — | — | expressible |
| `10062210` | Winged Statue | lastWords, engage | summon | — | expressible |
| `10071110` | Kitty Cannoneer | fanfare | — | — | expressible |
| `10071120` | Puppet Lancer | fanfare | mode:enhance | — | expressible |
| `10071130` | Mechanized Beast | — | — | — | expressible |
| `10071310` | Bullet from Beyond | — | destroy | — | expressible |
| `10072110` | Electric Whip Lass | fanfare | — | — | expressible |
| `10072120` | Mecha Cavalier | evolve, superEvolve | if/else, summon | — | expressible |
| `10072210` | Puppet Theater | fanfare, endOfTurn | — | — | expressible |
| `10131320` | Stormy Blast | spellboost | damage | — | expressible |
| `10401110` | Katalina, Sky's Protector | fanfare | damage | skyboundArt | expressible |
| `10401120` | Vyrn, Bestest Pal | fanfare | — | — | expressible |
| `10402110` | Yuni, Cosmic Legacy | endOfTurn | restore | — | expressible |
| `10403110` | Gran & Djeeta, Valiant Skyfarers | fanfare | choose, draw, damage | skyboundArt | expressible |
| `10403120` | Lyria, Skydestined | — | draw, mode:enhance | — | expressible |
| `10404110` | Sandalphon, Primarch Successor | fanfare, invoke/invoked, startOfTurn, zone:deck | repeat, damage, crest | skyboundArt, evolved | expressible |
| `10411110` | Kou & You, Love and Hatred | strike | restore | — | expressible |
| `10411120` | Manamel, Super Cutest | anyEvolve, endOfTurn | damage | — | expressible |
| `10411310` | Comet Drive | — | draw, damage | evolved | expressible |
| `10412110` | Chloe, What a Gal | — | summon, mode:enhance | — | expressible |
| `10412120` | Anthuria, Toe-Tapping Torch | fanfare | — | — | expressible |
| `10412310` | Starry Sky | — | damage, crest | — | expressible |
| `10413110` | Cupitan, Iridescent Archer | fanfare, anyEvolve | damage | skyboundArt | expressible |
| `10413310` | Alfheimr | — | choose, if/else, draw, restore | skyboundArt | expressible |
| `10414110` | Ewiyar, Wind Personified | fanfare | — | skyboundArt | expressible |
| `10414120` | Yuel & Societte, Dancing Duo | fanfare, superEvolve | repeat, damage, crest | — | expressible |
| `10421110` | Randall, Feet Fighter | — | mode:enhance | — | expressible |
| `10421120` | Arthur, Staunch Dragon | evolve | summon | — | expressible |
| `10421130` | Mordred, Illusory Lion | evolve | summon | — | expressible |
| `10422110` | Aglovale, Lord of Frost | fanfare | damage | — | expressible |
| `10422120` | Feather, Bombastic Brawler | — | — | — | expressible |
| `10422130` | Fiorito, Muscles in Bloom | — | — | — | expressible |
| `10423110` | Golden Knight, True King's Blade | fanfare | choose, if/else, damage, restore, mode:enhance | — | expressible |
| `10423310` | Knightly Ardor | — | choose, restore | — | expressible |
| `10424110` | Zeta & Bea, Crimson and Blue | fanfare | summon, mode:enhance | — | expressible |
| `10424120` | Seofon, Leader of the Eternals | fanfare | if/else | skyboundArt, evolved | expressible |
| `10431110` | Philosophia, Cryptic Sophist | fanfare | draw | — | expressible |
| `10431120` | Suframare, Wandering Tutor | evolve, endOfTurn | — | — | expressible |
| `10431310` | Rune Portal | — | damage, restore | — | expressible |
| `10432110` | Ezecrain, Portent of Vengeance | fanfare | damage | — | expressible |
| `10432120` | Mireille & Risette, Penitent Duo | fanfare | summon, pay | — | expressible |
| `10432310` | Unleashed | — | choose, draw, damage | — | expressible |
| `10433110` | Elmott, Remembrance Aflame | fanfare, superEvolve | damage, crest | — | expressible |
| `10433310` | Alchemic Flare | — | damage | skyboundArt | expressible |
| `10434110` | Wamdus, Water Personified | fanfare, superEvolve, spellboost | choose, damage, spellboostHand | — | expressible |
| `10434120` | Cagliostro, Genius Alchemist | fanfare | crest | skyboundArt | expressible |
| `10441110` | Joel, Wave Chaser | — | — | — | expressible |
| `10441120` | Mari, Meg's Bestie | endOfTurn, zone:hand | — | evolved | expressible |
| `10441310` | Crescent Tube Ride | endOfTurn | crest | — | expressible |
| `10442110` | Izmir, Frigid Fate | fanfare, anyEvolve | damage | — | expressible |
| `10442120` | Mugen, Steel-Bodied Honesty | fanfare, superEvolve | destroy | skyboundArt | expressible |
| `10442310` | Maximum Love Bomb | — | damage | — | expressible |
| `10443110` | Meg, Girl Next Door | fanfare, enter/when | — | skyboundArt | expressible |
| `10443310` | Primal Beast Absorption | — | banish | — | expressible |
| `10444110` | Wilnas, Flame Personified | fanfare, evolve | damage, replicate | — | expressible |
| `10444120` | Zooey, Ally of the World | fanfare | mode:enhance | — | expressible |
| `10451110` | Almeida, Headstrong Miner | — | mode:enhance | — | expressible |
| `10451120` | Vaseraga, Unyielding Scythe | lastWords | summon, damage | — | expressible |
| `10451310` | Valiant Edge | — | damage, crest | — | expressible |
| `10452110` | Nezha, Soaring War God | endOfTurn | damage | — | expressible |
| `10452120` | Satyr, Open-Hearted Rover | fanfare | — | evolved | expressible |
| `10452130` | Baal, Elemental Resonance | fanfare | choose, damage | — | expressible |
| `10453110` | Nehan, Dispenser of Samsara | fanfare | damage | evolved | expressible |
| `10453310` | Corruption | — | destroy, crest | skyboundArt | expressible |
| `10454110` | Fediel, Darkness Personified | fanfare, endOfTurn | reanimate, pay | — | expressible |
| `10454120` | Belial, Archangel of Cunning | fanfare, superEvolve | damage, crest | skyboundArt | expressible |
| `10461110` | Troue, Heroic Visionary | engage | — | — | expressible |
| `10461120` | Lamretta, Sisterly Shepherd | evolve, endOfTurn | damage | evolved | expressible |
| `10461210` | Awed and Inspired | engage | draw, destroy, transform | — | expressible |
| `10462110` | Sara, Graphos's Chosen | evolve | destroy, mode:enhance | — | expressible |
| `10462120` | Sophia, Zeyen Priestess | fanfare, superEvolve | summon | — | expressible |
| `10462210` | Skyfaring Vessel | engage, zone:hand | destroy | evolved | expressible |
| `10463110` | Tikoh, Asclepian Surgeon | evolve, engage | damage, restore | — | expressible |
| `10463210` | De La Fille's Gleaming Gems | engage | choose, draw, destroy | — | expressible |
| `10464110` | Galleon, Earth Personified | endOfTurn | — | evolved | expressible |
| `10464120` | Vira, Luminous Primal Knight | fanfare | banish | skyboundArt | expressible |
| `10471110` | Sho, Reborn Night King | fanfare | — | — | expressible |
| `10471120` | Tsubasa, Blazing Gearcyclist | fanfare | — | skyboundArt | expressible |
| `10471130` | Isaac, Congenial Engineer | lastWords | — | — | expressible |
| `10472110` | Eustace, Howl of Thunder | fanfare, clash | damage | skyboundArt, evolved | expressible |
| `10472120` | Ilsa, Brutal Drill Sergeant | fanfare | repeat, choose, damage | — | expressible |
| `10472310` | Stone Breaker | — | repeat, damage | — | expressible |
| `10473110` | Cassius, Sky-Yearning Arrival | fanfare, lastWords | damage | — | expressible |
| `10473310` | Chaos Legion | — | if/else, damage | skyboundArt | expressible |
| `10474110` | Lu Woh, Light Personified | fanfare | repeat, damage, crest | skyboundArt | expressible |
| `10474120` | Beelzebub, Supreme King | fanfare | damage | — | expressible |
| `10501110` | Monster Litterateur | fanfare | — | — | expressible |
| `10502110` | Goddess of Starlight | evolve | — | — | expressible |
| `10502120` | Behemoth General | evolve | destroy | — | expressible |
| `10503210` | World of Games | lastWords | draw | — | expressible |
| `10503310` | Fate of the World | — | draw, damage, destroy, mode:enhance | — | expressible |
| `10504110` | Getenou, Eightfold Glory | fanfare | choose, draw | — | expressible |
| `10511110` | Prudent Tanuki | evolve | draw | — | expressible |
| `10511120` | Battledore Woodsmaiden | fanfare, evolve, enter/when | summon, damage, replicate | — | expressible |
| `10511310` | Flight of the Swarmpetal | — | damage | — | expressible |
| `10512110` | Flowering Friendship | fanfare | summon | combo | expressible |
| `10512120` | Fairy Beastwhisperer | — | — | — | expressible |
| `10512310` | Quiet Encouragement | — | if/else, damage | combo | expressible |
| `10513110` | Spirited Skipper | fanfare, evolve, superEvolve | summon, replicate | — | expressible |
| `10513310` | Grace of the Swarmpetal | — | draw | — | expressible |
| `10514110` | Wolfraud, Skybound Hanged Man | fanfare, evolve | — | — | expressible |
| `10514120` | Miroku, Swarmpetal | fanfare, evolve | choose, damage, replicate | — | expressible |
| `10521110` | Altruistic Aristocrat | fanfare | if/else, restore | — | expressible |
| `10521120` | Smoke-Shrouded Beauty | fanfare, evolve | — | — | expressible |
| `10521310` | Extravagance of the Goldbloom | — | repeat, damage | — | expressible |
| `10522110` | Swift Staffmaster | fanfare | draw, restore | — | expressible |
| `10522120` | Amphibian Goldmuncher | evolve, endOfTurn | damage | — | expressible |
| `10522310` | Serenity's Shield | — | if/else, summon, mode:enhance | — | expressible |
| `10523110` | Unmoving Tactician | superEvolve, endOfTurn | summon | — | expressible |
| `10523310` | Splendor of the Goldbloom | — | if/else, mode:enhance | — | expressible |
| `10524110` | Oluon, Raging Chariot | endOfTurn | damage | evolved | expressible |
| `10524120` | Unkei, Goldbloom | fanfare, superEvolve | banish, crest | — | expressible |
| `10531110` | Terraforming Wizard | fanfare, superEvolve | summon | — | expressible |
| `10531120` | Waterbending Charmwielder | fanfare | damage, spellboostHand | — | expressible |
| `10531310` | Metamorphosis of the Dawnblossom | — | draw | — | expressible |
| `10532110` | Insomniac Witch | fanfare, evolve | destroy, crest | — | expressible |
| `10532120` | Woodsong Haikumaster | fanfare, lastWords, spellboost | draw, spellboostHand | — | expressible |
| `10532310` | Kitty Cunning | — | choose, summon, restore, pay | — | expressible |
| `10533110` | Emperor of Elements | fanfare, enter/when | summon, pay | — | expressible |
| `10533310` | Grandeur of the Dawnblossom | — | transform | — | expressible |
| `10534110` | Lhynkal, Wandering Fool | fanfare, superEvolve | crest | — | expressible |
| `10534120` | Ara, Dawnblossom | fanfare, evolve, spellboost | damage, transform | — | expressible |
| `10541110` | Springwell Steward | evolve, superEvolve | if/else, damage | — | expressible |
| `10541120` | Stormy Shamisen Shredder | fanfare, enter/when | summon, restore | — | expressible |
| `10541310` | Blade of the Crestpetal | — | draw, damage | — | expressible |
| `10542110` | Ironmace Dragoon | fanfare | summon, damage | — | expressible |
| `10542120` | Jellyfish Dancer | fanfare, enter/when | — | — | expressible |
| `10542310` | Roar of Prominence | — | damage | — | expressible |
| `10543110` | Ruinbringer | superEvolve | damage, banish | — | expressible |
| `10543310` | Sloth of the Crestpetal | — | repeat, damage | overflow | expressible |
| `10544110` | Erntz, Governing Justice | evolve, endOfTurn | damage, restore | evolved | expressible |
| `10544120` | Yube, Crestpetal | fanfare, evolve | summon, crest | — | expressible |
| `10551110` | Support Wolf | — | mode:enhance | — | expressible |
| `10551120` | Crimson Soulmancer | fanfare, evolve | reanimate, replicate | — | expressible |
| `10551310` | Valor of the Nightblossom | — | damage | — | expressible |
| `10552110` | Fickle Necromancer | fanfare, evolve | summon | — | expressible |
| `10552120` | Friendly Blue Ogre | fanfare, evolve | draw, replicate | — | expressible |
| `10552310` | Tyrannical Fists | — | damage | — | expressible |
| `10553110` | Lifestealer | fanfare, evolve | damage, restore, destroy, transform | — | expressible |
| `10553310` | Rigor of the Nightblossom | — | crest | — | expressible |
| `10554110` | Milteo & Luzen | fanfare, anyEvolve, anySuperEvolve | reanimate, destroy, crest | — | expressible |
| `10554120` | Shakdoh, Nightblossom | fanfare, superEvolve | repeat, draw, damage, replicate | — | expressible |
| `10561110` | Prescient Priestess | fanfare, evolve | damage, replicate | — | expressible |
| `10561120` | Bouquet Believer | — | draw, mode:enhance | turnOwner | expressible |
| `10561310` | Malice of the Mistbloom | — | draw | — | expressible |
| `10562110` | Immovable Paladin | fanfare | summon | — | expressible |
| `10562120` | Desperate Shrinemouse | fanfare, evolve | draw, damage, replicate | turnOwner | expressible |
| `10562210` | Protective Shell | engage | draw, destroy | — | expressible |
| `10563110` | Saint of Rehabilitation | fanfare, evolve, superEvolve | summon, restore, replicate | turnOwner | expressible |
| `10563210` | Resolve of the Mistbloom | fanfare, engage | draw, damage, destroy | — | expressible |
| `10564110` | Sofina, Inspiring Strength | fanfare, endOfTurn | choose | evolved | expressible |
| `10564120` | Kukishiro, Mistbloom | fanfare | draw, crest | — | question: Enemy Fox/Falcon summons: opponent controls them and they count as the opponent's Rally? |
| `10571110` | Marionette Master | fanfare | summon, mode:enhance | — | expressible |
| `10571120` | Flowering Artisan | fanfare | draw, damage | — | expressible |
| `10571310` | Light of the Dewdrop | — | draw | — | expressible |
| `10572110` | New-Age Cartographer | fanfare, superEvolve | summon | — | expressible |
| `10572120` | Lunar Bunny | — | — | — | expressible |
| `10572310` | Resurrection Tuner | — | destroy | — | expressible |
| `10573110` | Neuron Disrupter | fanfare, lastWords | draw | — | expressible |
| `10573310` | Sincerity of the Dewdrop | — | transform | — | expressible |
| `10574110` | Slaus, Revolving Wheel of Fortune | startOfTurn, endOfTurn | restore, banish, crest | evolved | question: Provisional: after 3 unused abilities, does the body still roll on later turns? |
| `10574120` | Imari, Dewdrop | fanfare, superEvolve | summon, draw | evolved | expressible |
| `10601110` | Muddled Onlooker | lastWords | damage | — | expressible |
| `10601120` | Disrupted Commoner | fanfare | destroy | — | expressible |
| `10602210` | Encroached World | engage | transform | — | expressible |
| `10603110` | Beast Lost to the Dark | fanfare | — | — | expressible |
| `10603210` | Dark Dimensions | endOfTurn | damage | — | expressible |
| `10604110` | Omegotep, the Dreaded One | fanfare, superEvolve | choose, damage, destroy, replicate | — | question: Re-entrant Fanfare (option 4 / Super-Evolve replicate): what is the resolution-depth cap? |
| `10611110` | Motherly Forestdweller | fanfare, evolve | summon | — | expressible |
| `10611120` | Monkey of Paradise | fanfare | — | combo | expressible |
| `10611310` | Advent of the Eld Lance | — | summon, damage | — | expressible |
| `10612110` | Kindly Executor | fanfare | — | — | expressible |
| `10612120` | Howling Wolfman | — | — | — | expressible |
| `10612310` | Floral Offering | zone:hand | draw | — | expressible |
| `10613110` | Merciful Attendant | fanfare | summon, restore | — | expressible |
| `10613310` | Nurturing Eld Lance | — | choose, summon, destroy | — | expressible |
| `10614110` | Althenia, Nurturing Bloom | fanfare, superEvolve | summon, destroy | — | expressible |
| `10614120` | Sathanid, Eld Lance | fanfare | damage | — | expressible |
| `10621110` | Fearless Soldier | — | mode:enhance | — | expressible |
| `10621120` | Idle Maid | fanfare | draw | — | expressible |
| `10621310` | Advent of the Eld Sword | — | summon, mode:enhance | — | expressible |
| `10622110` | Loyal Guard | evolve | damage, mode:enhance | — | expressible |
| `10622120` | Navy Cat | fanfare | destroy | — | expressible |
| `10622310` | Majestic Conquest | — | crest, mode:enhance | — | expressible |
| `10623110` | Heartless Strategist | fanfare | destroy, mode:enhance | — | expressible |
| `10623310` | Ruthless Eld Sword | — | choose, if/else, draw, damage, mode:enhance | — | expressible |
| `10624110` | Noel IV, Ruthless Warlord | fanfare, superEvolve | summon, mode:enhance | — | expressible |
| `10624120` | Yidmetra, Eld Sword | fanfare, evolve | mode:enhance | — | expressible |
| `10631110` | Crystalspawn | — | — | — | expressible |
| `10631120` | Daydream Librarian | fanfare, superEvolve | summon | — | expressible |
| `10631310` | Advent of the Eld Crystals | — | summon | — | expressible |
| `10632110` | Enraptured Student | fanfare, enter/when | summon, restore | — | expressible |
| `10632120` | Adventurous Grimoire | lastWords | summon, spellboostHand, mode:enhance | — | expressible |
| `10632310` | Reaved Order | — | draw, destroy | — | expressible |
| `10633110` | Spellbound Professor | fanfare, evolve | summon | — | expressible |
| `10633310` | Bewitching Eld Crystals | — | choose, if/else, summon, mode:enhance | — | expressible |
| `10634110` | Shymm, Love Bewitched | fanfare, superEvolve | summon, crest | — | expressible |
| `10634120` | Calge-Danthla, Eld Crystals | fanfare, evolve, enter/when, zone:hand | summon | — | expressible |
| `10641110` | Resolute Dragonewt | fanfare, lastWords | draw | — | expressible |
| `10641120` | Fruitfish | lastWords | summon, restore, mode:enhance | — | expressible |
| `10641310` | Advent of the Eld Blades | discarded | — | — | expressible |
| `10642110` | Decisive Swordmaster | fanfare | — | — | expressible |
| `10642120` | Spiked Dragon | anyEvolve, endOfTurn | damage | — | expressible |
| `10642310` | Spilling Red | — | destroy | — | expressible |
| `10643110` | Impeding Pugilist | fanfare, evolve | damage, replicate | — | expressible |
| `10643310` | Beheading Eld Blades | discarded | damage | — | expressible |
| `10644110` | Sagatsumatsu, Fair Beheader | fanfare | — | — | expressible |
| `10644120` | Vorlalai, Eld Blades | evolve, superEvolve, discarded | if/else, summon | — | expressible |
| `10651110` | Reverent Demon | lastWords | draw, damage | — | expressible |
| `10651120` | Ghost Dodger | lastWords | — | — | expressible |
| `10651310` | Advent of the Eld Sight | — | draw, restore, pay | — | expressible |
| `10652110` | Yearnful Necromancer | — | reanimate, mode:enhance | — | expressible |
| `10652120` | Devilish Heartbreaker | evolve | damage, mode:enhance | — | expressible |
| `10652310` | Allure of the Mightiest | — | summon, banish | — | expressible |
| `10653110` | Deprived Destroyer | fanfare, anyEvolve | summon, destroy | — | expressible |
| `10653310` | Depletive Eld Sight | — | choose, damage | — | expressible |
| `10654110` | Armes, Depletive Demon | superEvolve, clash | destroy | — | expressible |
| `10654120` | Bibatii, Eld Sight | fanfare, anyEvolve | pay | — | expressible |
| `10661110` | Prostrating Coward | lastWords, enter/when | summon, restore, mode:crystallize | — | expressible |
| `10661210` | Unholy Water | lastWords, engage | draw, destroy | — | expressible |
| `10661310` | Advent of the Eld Tome | — | draw | — | expressible |
| `10662110` | Venerating Dyer | lastWords | summon, mode:crystallize | — | expressible |
| `10662120` | Pegasus Rider | fanfare, evolve | summon, replicate | — | expressible |
| `10662210` | Scripture of Salvation | lastWords | draw, damage, restore | — | expressible |
| `10663110` | Worshipful Crusader | fanfare, lastWords, evolve | summon, destroy, mode:crystallize | — | expressible |
| `10663210` | Sublime Eld Tome | fanfare, lastWords | summon, destroy | — | expressible |
| `10664110` | Kandima, Sublime Hatred | fanfare, lastWords, superEvolve | summon, damage, destroy | — | expressible |
| `10664120` | Lyanthoth, Eld Tome | fanfare, endOfTurn | destroy | — | expressible |
| `10671110` | Shoddy Plaything | fanfare | summon, draw, mode:accelerate | — | expressible |
| `10671120` | Brilliant Inventor | fanfare | summon | — | expressible |
| `10671310` | Advent of the Eld Axe | — | draw, damage | — | expressible |
| `10672110` | Substandard Puppet | fanfare | summon, mode:accelerate | — | expressible |
| `10672120` | Timid Pioneer | fanfare | banish | — | expressible |
| `10672310` | Myriad Designs | — | summon | — | expressible |
| `10673110` | Ludicrous Ordnance | fanfare, evolve, endOfTurn | summon, damage, mode:accelerate | — | expressible |
| `10673310` | Unfeeling Eld Axe | enter/when, zone:hand | damage | evolved | expressible |
| `10674110` | Camiscilla, Unfeeling Heart | fanfare, superEvolve, enter/when | summon, damage | — | expressible |
| `10674120` | Yog-Zentha, Eld Axe | fanfare | — | — | expressible |
| `10701110` | Altaro Superfan | evolve | draw | — | expressible |
| `10701310` | Tears of Degradation | — | banish | — | expressible |
| `10702110` | Intrepid Newshound | lastWords, superEvolve | summon, draw | — | expressible |
| `10703110` | Hedonistic Socialite | fanfare | damage | — | expressible |
| `10703210` | City of Babelon | engage, endOfTurn | damage, restore, destroy | — | question: Engage with an empty hand: unplayable (Select forced) or Delay still fires? |
| `10704110` | Illamrita, Designated Target | lastWords, strike, followerStrike, endOfTurn | banish, crest | — | expressible |
| `10704120` | Altaro, Mayor of Babelon | endOfTurn | draw | — | expressible |
| `10711110` | Macrobear | fanfare | summon | — | expressible |
| `10711120` | Elven Trapper | fanfare | — | — | expressible |
| `10711310` | Cognitive Shift | — | draw | — | expressible |
| `10712110` | Virewind Fencer | fanfare | — | combo | expressible |
| `10712120` | Hawkeyed Tactician | fanfare, evolve | damage, replicate | — | expressible |
| `10712310` | Minimized Anxiety | — | restore, crest | combo | expressible |
| `10713110` | Frostbow Sniper | fanfare, endOfTurn | draw, damage | combo | expressible |
| `10713310` | Magnified Malice | — | damage, crest | combo | expressible |
| `10714110` | Thestae, Anathema of Distortion | fanfare, evolve | crest | — | expressible |
| `10714120` | Great Hart of the Glacial Realm | fanfare, superEvolve, endOfTurn | damage, crest | — | expressible |
| `10721110` | Bombastic Bombardier | fanfare, zone:hand | damage | — | expressible |
| `10721120` | High-Strung Liaison | fanfare | summon | — | expressible |
| `10721310` | Measured Attunement | — | damage | rally | expressible |
| `10722110` | Sharp-Eared Operative | lastWords, evolve | summon, damage | — | expressible |
| `10722120` | Metronomic Medic | fanfare, evolve | summon, draw, damage | — | expressible |
| `10722310` | Orchestrated Silence | — | if/else | rally | expressible |
| `10723110` | Knellclaw Lieutenant | fanfare | choose, summon | — | expressible |
| `10723310` | Caesura al Fine | — | damage | rally | expressible |
| `10724110` | Gildaria, Anathema of Attunement | fanfare, anyEvolve, enter/when | summon, crest | rally, turnOwner | expressible |
| `10724120` | Cesar, Accordant Major | fanfare, superEvolve | summon, destroy | — | expressible |
| `10731110` | Dainty Horror | fanfare | pay | — | expressible |
| `10731120` | Little Beastie | fanfare, evolve | damage, replicate | — | expressible |
| `10731310` | Heel, My Dearie | zone:hand | draw, pay | — | expressible |
| `10732110` | Charming Monster | lastWords, superEvolve | summon, pay | — | expressible |
| `10732120` | Pretty Predator | fanfare | — | — | expressible |
| `10732310` | Haphazard Snacking | — | choose, damage, pay | — | expressible |
| `10733110` | Sweet Abomination | fanfare, evolve | choose, draw, damage, pay, replicate | — | expressible |
| `10733310` | Bottomless Gluttony | zone:hand | destroy, pay | — | expressible |
| `10734110` | Lilanthim, Anathema of Predation | fanfare, evolve | destroy, crest, pay | — | expressible |
| `10734120` | Beloved Masterpiece | fanfare, lastWords, superEvolve | summon, damage, pay | — | expressible |
| `10741110` | Dragonewt Promoter | — | summon, mode:enhance | — | expressible |
| `10741120` | Carrier Wyvern | fanfare, evolve | — | — | expressible |
| `10741310` | Apathetic Gaze | — | transform | — | expressible |
| `10742110` | Gallant Gatekeeper | fanfare | destroy | — | expressible |
| `10742120` | Draconic Part-Timer | fanfare, endOfTurn | if/else, restore | evolved | expressible |
| `10742310` | Lazing Flame | — | draw, restore | overflow | expressible |
| `10743110` | Dragonewt Pathfinder | fanfare | choose, destroy | — | expressible |
| `10743310` | Blackflame Deluge | — | damage | — | expressible |
| `10744110` | Burnite, Anathema of Ash | fanfare, superEvolve | damage, crest | — | expressible |
| `10744120` | Dragon's Vale Elder | fanfare, superEvolve | summon, crest | — | expressible |
| `10751110` | Lulumi, Vamp on the Keys | lastWords | — | — | expressible |
| `10751120` | Raz, Demon on the Drums | lastWords, evolve | summon, damage | — | expressible |
| `10751310` | Soul Tuning | — | draw | — | expressible |
| `10752110` | Highwire Feline | fanfare, evolve | summon, damage, replicate | — | expressible |
| `10752120` | Juggler Corvid | fanfare | reanimate, destroy | — | expressible |
| `10752310` | Harmony of Youth | — | summon | — | expressible |
| `10753110` | Beastmaster Bones | fanfare, superEvolve, enter/when | summon, destroy | — | expressible |
| `10753310` | Hark to the Night Song | — | damage, pay | — | expressible |
| `10754110` | Adahime, Anathema of Death | fanfare, superEvolve, enter/when | summon | — | expressible |
| `10754120` | Macmillan, Reaper of Ceremonies | fanfare, enter/when | summon, damage, pay | turnOwner | expressible |
| `10761110` | Reverend of Finance | fanfare, lastWords | summon, draw | — | expressible |
| `10761120` | Missionary of Recruitment | fanfare, evolve | draw, damage | — | expressible |
| `10761210` | Earrings of Sunlight | fanfare, engage | draw, destroy, replicate | — | expressible |
| `10762110` | Sister of Strategic Development | fanfare | damage | — | expressible |
| `10762120` | Holy Hawk of Communications | fanfare | restore, banish | — | expressible |
| `10762210` | Timepiece of Perfection | engage | damage, destroy, mode:enhance | — | expressible |
| `10763110` | Deacon of Security | fanfare, lastWords | restore | — | expressible |
| `10763210` | Trident of Eroding Tides | fanfare, engage | damage, destroy | — | expressible |
| `10764110` | Rodeo, Anathema of Adjudication | fanfare, evolve | summon | — | expressible |
| `10764120` | Initia, Chief Ordination Officer | fanfare, superEvolve | restore, banish, replicate | — | expressible |
| `10771110` | Brusque Barkeep | evolve, enter/when | summon, restore | — | expressible |
| `10771120` | Beat Breaker | fanfare | if/else, summon | — | expressible |
| `10771310` | Freerunning | — | choose, if/else | — | expressible |
| `10772110` | Cool Courier | fanfare, evolve | replicate | — | expressible |
| `10772120` | Audacious Artist | fanfare | summon, destroy | — | expressible |
| `10772310` | Blink Step | — | — | — | expressible |
| `10773110` | Brazen Broadcaster | fanfare, enter/when | summon, mode:enhance | — | expressible |
| `10773310` | Warp Slash | — | damage | — | expressible |
| `10774110` | Scarlet, Anathema of Dislocation | fanfare | damage | — | expressible |
| `10774120` | Myuu, Hot on His Heels | evolve, superEvolve, enter/when | summon, damage | — | expressible |
| `10801110` | Hamsa, Sculpted Divinity | evolve | — | — | expressible |
| `10801120` | Reina, Timeless Wanderer | fanfare | — | — | expressible |
| `10802110` | Alfied, Squire of Joy | fanfare, evolve | damage | — | expressible |
| `10802310` | Legacy of the Brave | — | draw | — | expressible |
| `10803110` | Aika, Elegy of Loss | fanfare, evolve | destroy, replicate | — | expressible |
| `10803310` | Wills United | — | choose, reanimate, damage | — | expressible |
| `10804110` | Alabaster Bahamut | fanfare | choose, banish | — | expressible |
| `10804120` | Olivia, Proud Dark Angel | fanfare | — | — | expressible |
| `10811110` | Marlone, Scales of the Past | fanfare | destroy | — | expressible |
| `10811120` | Citrus, Heretical Hermit | fanfare, evolve | summon, replicate | — | expressible |
| `10811130` | Moelle, Gloomy Maiden | fanfare | draw | — | expressible |
| `10812110` | Ruflet, Primeval Fairy | lastWords | summon | — | expressible |
| `10812120` | Lycoris, Barbs of Passion | fanfare | summon | — | expressible |
| `10812310` | Peaceful Solitude | — | restore, destroy | — | expressible |
| `10813110` | Michelle, Kind Mindreader | fanfare | summon | — | expressible |
| `10813310` | Curiosity Abounds | — | summon | — | expressible |
| `10814110` | Setus & Maisha, Bladerights | fanfare | destroy | — | expressible |
| `10814120` | Tia, Eternal Crystalian | — | mode:enhance | — | expressible |
| `10821110` | Naht & Vince, Force and Order | fanfare, superEvolve | summon, damage, replicate | — | expressible |
| `10821120` | Shaili, Prowling Assassin | evolve | — | — | expressible |
| `10821130` | Sasha, Knight Everlasting | fanfare, evolve | summon | — | expressible |
| `10822110` | Katze, Magical Thief | evolve | damage | — | expressible |
| `10822120` | Oda Nobunaga | fanfare | damage | — | expressible |
| `10822310` | Shared Existence | — | summon, mode:enhance | — | expressible |
| `10823110` | Okita Souji | fanfare, strike, followerStrike | repeat, if/else, damage | evolved | expressible |
| `10823310` | Slice of Domesticity | — | choose, if/else, damage, restore | — | expressible |
| `10824110` | Bunny & Baron, Fate's Bullet | fanfare, evolve | summon, damage | rally | expressible |
| `10824120` | Mars, Conflagrant Commander | fanfare, superEvolve, enter/when | summon | — | expressible |
| `10831110` | Meowskers, Roly-Poly Mk II & Djeana | fanfare, evolve | damage, replicate, spellboostHand | — | expressible |
| `10831120` | Poppy, Mysterian Secretary | fanfare | — | — | expressible |
| `10831310` | Stormy Blast | spellboost | damage | — | expressible |
| `10832110` | Sammy & Marie, Flowers of Joy | fanfare, spellboost | draw | — | expressible |
| `10832310` | Harmonious Meal | — | restore, spellboostHand | — | expressible |
| `10832320` | Earth-Shattering Bolt | — | damage, pay | — | expressible |
| `10833110` | Tico, Mysterian Spellcrafter | fanfare, evolve, superEvolve | crest | — | expressible |
| `10833310` | Amethyst's Naptime | spellboost | draw, restore | — | expressible |
| `10834110` | Tetra & Ladica, Forest BFFs | fanfare, spellboost | — | — | expressible |
| `10834120` | Ginger, Disastrous Word | fanfare, evolve, enter/when | summon | — | expressible |
| `10841110` | Gido, Leader of the Pack | fanfare | — | — | expressible |
| `10841120` | Sandstorm Watchdragon | lastWords | draw | — | expressible |
| `10841130` | Spirit of Wadatsumi | fanfare, evolve | crest | — | expressible |
| `10842110` | Reef & Lolo, Serene Sirens | fanfare, endOfTurn | summon | — | expressible |
| `10842120` | Kimika, Cook of Happiness | fanfare, evolve | draw, restore, replicate | — | expressible |
| `10842310` | Art of Decay | — | damage | — | expressible |
| `10843110` | Giada, Peerless Flame of War | strike | — | — | expressible |
| `10843310` | Ephemeral Foxfire | — | draw, damage | overflow | expressible |
| `10844110` | Drache & Aluzard, Burning Blood | fanfare, lastWords | crest | — | expressible |
| `10844120` | Lumiore & Argente, Shining Wings | fanfare, superEvolve | draw, damage, mode:accelerate | — | expressible |
| `10851110` | Anisage, Clear Resolve | — | — | — | expressible |
| `10851120` | Lilith, Devilish Cutie | lastWords, strike | damage | — | expressible |
| `10851130` | Limil, Devilish Bunny | fanfare, evolve | summon, replicate | — | expressible |
| `10852110` | Fiole, Devilish Matriarch | fanfare, enter/when | summon | — | expressible |
| `10852120` | Marsha, Dark Knight | fanfare, evolve | damage, replicate | — | expressible |
| `10852310` | Bittersweet Departures | — | damage, restore | — | expressible |
| `10853110` | Suzy, Sincere Hexcaster | fanfare, evolve | — | — | expressible |
| `10853310` | Ebb and Flow | — | draw | — | expressible |
| `10854110` | Itsurugi & Taketsumi, Brothers | fanfare, evolve | choose, draw, damage, restore | — | expressible |
| `10854120` | Ceres, Liminal Rose | fanfare, clash, endOfTurn | damage, restore, pay | — | expressible |
| `10861110` | Lilium, Witch of the Tomes | lastWords, evolve | draw, damage | — | expressible |
| `10861120` | Theresa, Ergon Priestess | fanfare | summon | — | expressible |
| `10861130` | Grant, Hunter of Undeath | fanfare, evolve | destroy, replicate | — | expressible |
| `10862110` | Edeth, Voice of Heaven | lastWords, superEvolve | summon, destroy | — | expressible |
| `10862120` | Viche, Abyssal Researcher | zone:hand | — | — | expressible |
| `10862310` | Lingering Threat | — | if/else, banish, mode:enhance | — | expressible |
| `10863110` | Colette, Holy Exorcist | fanfare, anyEvolve | damage | evolved | expressible |
| `10863210` | Academy Hijinks | endOfTurn | if/else, draw, restore | evolved | expressible |
| `10864110` | Verdilia & Castelle, Sisters | fanfare, superEvolve | summon, crest | — | expressible |
| `10864120` | Zoe, Dazzling Hope | fanfare, evolve | choose, damage, restore, crest | — | expressible |
| `10871110` | Kratos, Everyday Joy | lastWords | summon | — | expressible |
| `10871120` | Leona, Overbearing Guardian | superEvolve | — | — | expressible |
| `10871130` | Zerk, Artifact Manipulator | fanfare | destroy | — | expressible |
| `10872110` | Layla, Artificial Gift of Life | lastWords | — | — | expressible |
| `10872120` | Lazuli, Gateway Connector | fanfare | — | — | expressible |
| `10872310` | Unsullied Days | — | draw, restore | — | expressible |
| `10873110` | Miriam, Reciprocator | fanfare | summon, destroy | — | expressible |
| `10873310` | The Journey Ahead | — | damage | — | expressible |
| `10874110` | Asher & Lydia, Paths Beyond | fanfare, anyEvolve | destroy, mode:enhance | — | expressible |
| `10874120` | Eudie, Your Dependable Mentor | fanfare, evolve | — | evolved | expressible |
| `10901110` | Jailor of Antiquity | fanfare | damage, mode:accelerate | — | expressible |
| `10901310` | Initiation of Rebirth | — | draw, destroy | — | question: Does 'destroyed this match' still see a follower that was later reanimated or removed from cemetery? |
| `10902110` | Blade Angel | fanfare | — | — | expressible |
| `10903110` | Warden of Selflessness | fanfare, evolve | damage | — | expressible |
| `10903210` | Azvaldt, Penitentiary of Chaos | lastWords, endOfTurn | summon, destroy | — | expressible |
| `10904110` | Zerael, Sundered Rebirth | fanfare, invoke/invoked, endOfTurn, zone:deck | damage | — | expressible |
| `10911110` | Sprouting Initiate | fanfare | draw | combo | expressible |
| `10911120` | Jungle Youth | — | — | — | expressible |
| `10911210` | Trap in the Woods | enter/when | destroy | — | expressible |
| `10912110` | Leafshadow Assassin | fanfare | — | combo | expressible |
| `10912120` | Primate Plotters | fanfare, evolve | replicate | — | expressible |
| `10912310` | Verdant Ring Kindred | — | choose, if/else, damage | combo | expressible |
| `10913110` | Virid Lieutenant | fanfare, evolve | draw | combo | expressible |
| `10913310` | Crimson Incense | endOfTurn, zone:hand | draw, destroy | combo | expressible |
| `10914110` | Magachiyo, Aromatic Convict | fanfare, superEvolve | if/else, damage | combo | expressible |
| `10914120` | Hien, Redolent Revenant | fanfare, lastWords, zone:hand | summon, damage | — | expressible |
| `10921110` | Open-Sea Scout | fanfare, evolve | summon | — | expressible |
| `10921120` | Kindred Cavalrywoman | — | — | — | expressible |
| `10921310` | Phalanx | — | if/else, summon, mode:enhance | — | expressible |
| `10922110` | Whirlpool Gunner | fanfare | summon | — | expressible |
| `10922120` | Ferocious Commander | fanfare | summon, mode:enhance | — | expressible |
| `10922310` | Severed Ties | — | damage | — | expressible |
| `10923110` | Roughwater First Mate | fanfare, evolve, superEvolve | summon, damage, replicate | — | expressible |
| `10923310` | L'Age d'Or | — | if/else, summon, damage, mode:enhance | — | expressible |
| `10924110` | Barbaros, Rebellious Convict | fanfare | summon | — | expressible |
| `10924120` | Beltezore, Valorous Revenant | — | — | — | expressible |
| `10931110` | Obsessed Test Subject | enter/when | — | — | expressible |
| `10931120` | Key Spirit | fanfare, evolve, spellboost | damage | — | expressible |
| `10931310` | Miscalculated Experiment | — | summon | — | expressible |
| `10932110` | Enamored Researcher | fanfare, evolve | if/else, summon, mode:enhance | — | expressible |
| `10932120` | Noble Philosopher | fanfare | draw | — | expressible |
| `10932310` | Humane Love | — | summon | — | expressible |
| `10933110` | Ecstatic Scholar | fanfare, superEvolve, fused | summon, draw | — | expressible |
| `10933310` | Obsidian Raven | — | repeat, summon, damage | — | expressible |
| `10934110` | Sephie, Maven Convict | fanfare, superEvolve, fused | summon, crest, pay | — | expressible |
| `10934120` | Phylene, Cleansing Revenant | spellboost | — | — | expressible |
| `10941110` | Ripper-Clawed Thief | fanfare, lastWords | — | — | expressible |
| `10941120` | Cave Dragon | fanfare | — | overflow | expressible |
| `10941310` | Drake Whelp's Tantrum | — | summon, damage, mode:enhance | — | expressible |
| `10942110` | High-Spirited Marauder | strike | — | — | expressible |
| `10942120` | Dragonfolk Butler | fanfare, evolve | restore | — | expressible |
| `10942310` | Parting Jaws | — | damage | — | expressible |
| `10943110` | Barren-Earth Tyrant | fanfare, evolve | repeat, if/else, summon, damage | — | expressible |
| `10943310` | Artiglio | — | if/else, damage | — | expressible |
| `10944110` | Antemaria, Piercing Convict | fanfare | — | — | expressible |
| `10944120` | Normagdala, Ravening Revenant | fanfare, evolve | choose, draw, restore, replicate | — | expressible |
| `10951110` | Ruthless Blitzer | fanfare | damage | — | expressible |
| `10951120` | Netherworld Lieutenant | lastWords | summon | — | expressible |
| `10951310` | Spooky Surprise | — | summon | — | expressible |
| `10952110` | Void Colonel | lastWords | summon, restore, destroy, mode:crystallize | — | expressible |
| `10952120` | Sparkly Demoness | fanfare | damage, destroy | — | expressible |
| `10952310` | Chains of the Past | — | repeat, if/else, damage, mode:enhance | — | expressible |
| `10953110` | Rampaging Commander | fanfare, superEvolve | draw, damage | — | expressible |
| `10953310` | Reaper's Due | lastWords | summon | — | expressible |
| `10954110` | Istyndet vs. Mitilykket | fanfare, superEvolve | repeat, reanimate, damage, crest | — | expressible |
| `10954120` | Garodeth vs. Zeth | fanfare, endOfTurn, zone:hand | choose, damage | — | expressible |
| `10961110` | Follower of the Tenets | — | restore | turnOwner | expressible |
| `10961120` | Peryton | fanfare | summon | — | expressible |
| `10961210` | Roaring Basilica | lastWords, engage | summon, damage | — | expressible |
| `10962110` | Agent of the Testaments | fanfare, endOfTurn | restore | — | expressible |
| `10962120` | Miraculous Al-mi'raj | lastWords, engage | summon, mode:crystallize | — | expressible |
| `10962310` | Vow of Devotion | — | choose, if/else, damage, restore | — | expressible |
| `10963110` | Executor of the Vow | fanfare, superEvolve | restore, destroy, replicate | turnOwner | expressible |
| `10963210` | Juratio | fanfare, engage | draw, restore, destroy | — | expressible |
| `10964110` | Erralde, Signet Convict | fanfare, evolve | destroy, crest | — | expressible |
| `10964120` | Omerio, Winged Revenant | evolve | summon, damage, restore, destroy | — | expressible |
| `10971110` | Bluerust Underling | fanfare | damage | — | expressible |
| `10971120` | Twindrone Engineer | fanfare, evolve | summon, replicate | — | expressible |
| `10971310` | Dimensional Selection | — | choose, summon, damage | — | expressible |
| `10972110` | Ironwork Bodyguard | fanfare | damage, restore | — | expressible |
| `10972120` | Blade Puppeteer | fanfare, enter/when | summon, damage, mode:enhance | — | expressible |
| `10972310` | Disgraceful Banishment | — | if/else, draw | — | expressible |
| `10973110` | Steelforged Right Hand | fanfare | destroy | — | expressible |
| `10973310` | Soulforge | — | if/else, destroy | — | expressible |
| `10974110` | Cutthroat, Fluxblade Convict | evolve | banish, crest | — | expressible |
| `10974120` | Aizeden, Killshot Revenant | fanfare, superEvolve, enter/when | summon, destroy, replicate | — | expressible |
| `90011110` | Fairy | — | — | — | expressible |
| `90011120` | Springbloom Fairy | endOfTurn | — | — | expressible |
| `90011310` | Deepwood Bounty | — | restore | — | expressible |
| `90014110` | Eve, Blade of Crystalia | — | — | — | expressible |
| `90014330` | Depths of the Eld Lance | — | — | evolved | expressible |
| `90021110` | Knight | — | — | — | expressible |
| `90021120` | Steelclad Knight | — | — | — | expressible |
| `90021130` | Naht's Henchman | — | — | — | expressible |
| `90021210` | Dread Pirate's Flag | lastWords | damage | — | expressible |
| `90021310` | Gilded Blade | — | damage | — | expressible |
| `90021320` | Gilded Goblet | — | restore | — | expressible |
| `90021330` | Gilded Boots | — | — | — | expressible |
| `90021340` | Gilded Necklace | — | — | — | expressible |
| `90021350` | Glittering Gold | — | choose, draw, damage | — | expressible |
| `90022110` | Wretch | — | — | — | expressible |
| `90024320` | Depths of the Eld Sword | — | if/else, damage, mode:enhance | — | expressible |
| `90024330` | Desperados' Shot | — | repeat, damage | — | expressible |
| `90031110` | Clay Golem | — | — | — | expressible |
| `90031120` | Guardian Golem | — | — | — | expressible |
| `90031210` | Magic Sediment | engage | — | — | expressible |
| `90031310` | Mysterian Missile | — | damage | — | expressible |
| `90034320` | Ars Magna | — | damage, restore | — | expressible |
| `90034330` | Depths of the Eld Crystals | — | summon, damage, restore | — | expressible |
| `90034340` | Delta Cannon | — | damage | — | expressible |
| `90034350` | Send 'Em Packing | — | — | — | expressible |
| `90041110` | Fire Drake Whelp | — | — | — | expressible |
| `90041120` | Vastwing Dragon | — | — | — | expressible |
| `90041130` | Majestic Megalorca | — | — | — | expressible |
| `90044330` | Depths of the Eld Blades | discarded | damage, restore | — | expressible |
| `90051110` | Skeleton | — | — | — | expressible |
| `90051120` | Bat | — | — | — | expressible |
| `90051130` | Ghost | endOfTurn | banish | — | expressible |
| `90051140` | Rotting Zombie | lastWords | summon | — | expressible |
| `90054330` | Depths of the Eld Sight | — | draw | — | expressible |
| `90061110` | Holy Falcon | — | — | — | expressible |
| `90061120` | Holyflame Tiger | — | — | — | expressible |
| `90061130` | Regal Falcon | — | — | — | expressible |
| `90064210` | Rings of Moonlight | endOfTurn | damage | — | expressible |
| `90064320` | Depths of the Eld Tome | — | damage, destroy | — | expressible |
| `90071110` | Puppet | — | destroy | — | expressible |
| `90071120` | Enhanced Puppet | — | destroy | — | expressible |
| `90071130` | Analyzing Artifact | enter/when | draw | — | expressible |
| `90071140` | Ancient Artifact | — | — | — | expressible |
| `90071150` | Mystic Artifact | — | — | — | expressible |
| `90071160` | Radiant Artifact | — | — | — | expressible |
| `90071210` | Gear of Ambition | fused | transform | — | question: Author from ruling (gears only) or printed 'Fuse: Artifact amulets'? |
| `90071220` | Gear of Remembrance | fused | transform | — | expressible |
| `90072110` | Striker Artifact | fused | transform | — | expressible |
| `90072120` | Fortifier Artifact | fused | transform | — | expressible |
| `90073110` | Ominous Artifact α | fused, endOfTurn | restore, transform | — | expressible |
| `90073120` | Ominous Artifact β | endOfTurn | damage | — | expressible |
| `90073130` | Ominous Artifact γ | endOfTurn | damage | — | expressible |
| `90074110` | Masterwork Artifact Ω | fanfare | damage, restore | — | expressible |
| `90074140` | Imari's Little Buddies | — | — | — | expressible |
| `90074150` | Warden of the Trigger | lastWords | restore | — | expressible |
| `90074320` | Depths of the Eld Axe | — | — | — | expressible |

## Crests

| id | name | triggers | ops/combinators | conditions | status |
|---|---|---|---|---|---|
| `crest:10404110` | Crest: Sandalphon, Primarch Successor | endOfTurn | restore | — | expressible |
| `crest:10412310` | Crest: Starry Sky | lastWords | damage | — | expressible |
| `crest:10414120` | Crest: Yuel & Societte, Dancing Duo | — | — | — | expressible |
| `crest:10433110` | Crest: Elmott, Remembrance Aflame | startOfTurn | damage | — | expressible |
| `crest:10434120` | Crest: Cagliostro, Genius Alchemist | startOfTurn | pay | — | expressible |
| `crest:10441310` | Crest: Crescent Tube Ride | endOfTurn | — | — | expressible |
| `crest:10451310` | Crest: Valiant Edge | endOfTurn | damage, restore | — | expressible |
| `crest:10453310` | Crest: Corruption | endOfTurn | damage | — | expressible |
| `crest:10454120` | Crest: Belial, Archangel of Cunning | lastWords | damage | — | expressible |
| `crest:10474110` | Crest: Lu Woh, Light Personified | — | — | — | expressible |
| `crest:10524120` | Crest: Unkei, Goldbloom | endOfTurn | — | — | expressible |
| `crest:10532110` | Crest: Insomniac Witch | lastWords | damage | — | expressible |
| `crest:10534110` | Crest: Lhynkal, Wandering Fool | enter/when | — | — | expressible |
| `crest:10544120` | Crest: Yube, Crestpetal | — | — | — | expressible |
| `crest:10553310` | Crest: Rigor of the Nightblossom | endOfTurn | summon, draw | — | expressible |
| `crest:10554110` | Crest: Milteo & Luzen | — | mode:enhance | — | needs: suppressTriggers |
| `crest:10564120` | Crest: Kukishiro, Mistbloom | — | summon, draw | turnOwner | expressible |
| `crest:10574110` | Crest: Slaus, Revolving Wheel of Fortune | startOfTurn | damage | — | expressible |
| `crest:10622310` | Crest: Majestic Conquest | — | summon, mode:enhance | — | expressible |
| `crest:10634110` | Crest: Shymm, Love Bewitched | — | — | — | expressible |
| `crest:10704110` | Crest: Illamrita, Designated Target | lastWords | summon | — | expressible |
| `crest:10712310` | Crest: Minimized Anxiety | lastWords | — | — | expressible |
| `crest:10713310` | Crest: Magnified Malice | lastWords | — | — | expressible |
| `crest:10714110` | Crest: Thestae, Anathema of Distortion | endOfTurn | — | combo | expressible |
| `crest:10714120` | Crest: Great Hart of the Glacial Realm | endOfTurn | — | combo | expressible |
| `crest:10724110` | Crest: Gildaria, Anathema of Attunement | enter/when | damage | turnOwner | expressible |
| `crest:10734110` | Crest: Lilanthim, Anathema of Predation | — | summon | — | expressible |
| `crest:10744110` | Crest: Burnite, Anathema of Ash | startOfTurn | damage, restore | — | expressible |
| `crest:10744120` | Crest: Dragon's Vale Elder | endOfTurn | summon | — | expressible |
| `crest:10833110` | Crest: Tico, Mysterian Spellcrafter | — | damage | — | expressible |
| `crest:10841130` | Crest: Spirit of Wadatsumi | enter/when | — | — | expressible |
| `crest:10844110` | Crest: Drache & Aluzard, Burning Blood | lastWords | — | — | expressible |
| `crest:10864110` | Crest: Verdilia & Castelle, Sisters | — | — | evolved | expressible |
| `crest:10864120` | Crest: Zoe, Dazzling Hope | lastWords | summon | — | expressible |
| `crest:10934110` | Crest: Sephie, Maven Convict | enter/when | — | — | expressible |
| `crest:10954110` | Crest: Istyndet vs. Mitilykket | lastWords, endOfTurn | destroy | — | expressible |
| `crest:10964110` | Crest: Erralde, Signet Convict | endOfTurn | damage, restore | — | expressible |
| `crest:10974110` | Crest: Cutthroat, Fluxblade Convict | — | — | — | expressible |
| `faith:10614120` | Faith: Sathanid, Eld Lance | — | — | — | expressible |
| `faith:10624120` | Faith: Yidmetra, Eld Sword | — | mode:enhance | — | expressible |
| `faith:10634120` | Faith: Calge-Danthla, Eld Crystals | enter/when | — | — | question: Is this the same Faith instance as faith:90034330 (duplicate official SE text)? |
| `faith:10664120` | Faith: Lyanthoth, Eld Tome | — | destroy | — | expressible |
| `faith:90034330` | Faith: Depths of the Eld Crystals | enter/when | — | — | question: Is this the same Faith instance as faith:10634120, or a second Faith the token grants? |

## Totals

- rows: 615 (cards 572, crests 43)
- expressible: 606
- needs: 1
- question: 8

## Regenerating the id list

The pool is not committed (the old repo is read-only reference). From a checkout of `melnce/Practice-Tool` at `8f491f9`:

```bash
python3 tools/list-pool.py --meta /path/to/Practice-Tool/cards/official-meta.json
```

Expected: 516 rotation + 56 reachable tokens + 38 Crest + 5 Faith = 572 + 43. Measured at authoring time: those exact counts.
