# Schema examples

Each worked example is the committed file. `printed` is a whitespace-normalised substring of `text` (crest/mode text for those abilities).

## Listed examples

### `10574110` — Slaus, Revolving Wheel of Fortune

Ambush; endOfTurn when evolved → crest to opponent + banish; startOfTurn choose by randomUnused.

Printed text:

```
Ambush
At the end of your turn, if this follower is evolved, give your opponent Crest: Slaus, Revolving Wheel of Fortune and banish this card.
At the start of your turn, activate a random ability that hasn't been activated yet from the following.
1. Reduce the cost of all cards in your hand by 1 until the end of the turn.
2. Give all allied followers on the field +2/+2.
3. Restore 3 defense to your leader.
```

File: `cards/10005/10574110.json`

```json
{
  "id": "10574110",
  "name": "Slaus, Revolving Wheel of Fortune",
  "kind": "follower",
  "class": "portalcraft",
  "set": 10005,
  "rarity": "legendary",
  "token": false,
  "cost": 3,
  "text": "Ambush\nAt the end of your turn, if this follower is evolved, give your opponent Crest: Slaus, Revolving Wheel of Fortune and banish this card.\nAt the start of your turn, activate a random ability that hasn't been activated yet from the following.\n1. Reduce the cost of all cards in your hand by 1 until the end of the turn.\n2. Give all allied followers on the field +2/+2.\n3. Restore 3 defense to your leader.",
  "attack": 0,
  "defense": 2,
  "traits": {
    "ambush": true
  },
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, if this follower is evolved, give your opponent Crest: Slaus, Revolving Wheel of Fortune and banish this card.",
      "when": {
        "evolved": true
      },
      "effects": [
        {
          "printed": "give your opponent Crest: Slaus, Revolving Wheel of Fortune and banish this card.",
          "op": "seq",
          "effects": [
            {
              "op": "crest",
              "gain": "crest:10574110",
              "player": "opponent"
            },
            {
              "op": "banish",
              "select": {
                "pick": "self"
              }
            }
          ]
        }
      ]
    },
    {
      "on": "startOfTurn",
      "whose": "own",
      "printed": "At the start of your turn, activate a random ability that hasn't been activated yet from the following.\n1. Reduce the cost of all cards in your hand by 1 until the end of the turn.\n2. Give all allied followers on the field +2/+2.\n3. Restore 3 defense to your leader.",
      "effects": [
        {
          "printed": "activate a random ability that hasn't been activated yet from the following.\n1. Reduce the cost of all cards in your hand by 1 until the end of the turn.\n2. Give all allied followers on the field +2/+2.\n3. Restore 3 defense to your leader.",
          "op": "choose",
          "pick": 1,
          "by": "randomUnused",
          "options": [
            {
              "printed": "1. Reduce the cost of all cards in your hand by 1 until the end of the turn.",
              "effects": [
                {
                  "op": "cost",
                  "select": {
                    "side": "ally",
                    "zone": "hand",
                    "kind": "card",
                    "pick": "all"
                  },
                  "delta": -1,
                  "untilEndOfTurn": true
                }
              ]
            },
            {
              "printed": "2. Give all allied followers on the field +2/+2.",
              "effects": [
                {
                  "op": "buff",
                  "select": {
                    "side": "ally",
                    "zone": "field",
                    "kind": "follower",
                    "pick": "all"
                  },
                  "attack": 2,
                  "defense": 2
                }
              ]
            },
            {
              "printed": "3. Restore 3 defense to your leader.",
              "effects": [
                {
                  "op": "restore",
                  "select": {
                    "pick": "all",
                    "side": "ally",
                    "zone": "leader",
                    "kind": "leader"
                  },
                  "amount": 3
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### `10724110` — Gildaria, Anathema of Attunement

if rally 20 gates two sentences; when ally_follower_enter + turnOwner + other; anyEvolve summon 2.

Printed text:

```
Fanfare: Rally (20) - Gain Crest: Gildaria, Anathema of Attunement. Evolve this follower.
During your turn, whenever another allied follower enters the field, give it Rush.
When this follower evolves, summon 2 copies of Steelclad Knight.
```

File: `cards/10007/10724110.json`

```json
{
  "id": "10724110",
  "name": "Gildaria, Anathema of Attunement",
  "kind": "follower",
  "class": "swordcraft",
  "set": 10007,
  "rarity": "legendary",
  "token": false,
  "cost": 4,
  "text": "Fanfare: Rally (20) - Gain Crest: Gildaria, Anathema of Attunement. Evolve this follower.\nDuring your turn, whenever another allied follower enters the field, give it Rush.\nWhen this follower evolves, summon 2 copies of Steelclad Knight.",
  "attack": 4,
  "defense": 4,
  "tribes": [
    "anathema"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Rally (20) - Gain Crest: Gildaria, Anathema of Attunement. Evolve this follower.",
      "effects": [
        {
          "printed": "Rally (20) - Gain Crest: Gildaria, Anathema of Attunement. Evolve this follower.",
          "op": "if",
          "cond": {
            "rally": {
              "n": 20
            }
          },
          "then": [
            {
              "op": "crest",
              "gain": "crest:10724110",
              "player": "self"
            },
            {
              "op": "evolve",
              "select": {
                "pick": "self"
              },
              "super": false
            }
          ]
        }
      ]
    },
    {
      "on": "when",
      "event": "ally_follower_enter",
      "printed": "During your turn, whenever another allied follower enters the field, give it Rush.",
      "when": {
        "turnOwner": "self"
      },
      "effects": [
        {
          "printed": "give it Rush.",
          "op": "grantTraits",
          "select": {
            "pick": "entering"
          },
          "traits": {
            "rush": true
          }
        }
      ]
    },
    {
      "on": "anyEvolve",
      "printed": "When this follower evolves, summon 2 copies of Steelclad Knight.",
      "effects": [
        {
          "printed": "summon 2 copies of Steelclad Knight.",
          "op": "summon",
          "card": {
            "named": "90021120"
          },
          "count": 2
        }
      ]
    }
  ]
}
```

### `10434120` — Cagliostro, Genius Alchemist

Two independent Fanfare sentences; two Skybound thresholds; crest.

Printed text:

```
Fanfare: Gain 2 earth sigils. Add an Ars Magna to your hand.
Skybound Art - Evolve this follower.
Super Skybound Art - Gain Crest: Cagliostro, Genius Alchemist.
```

File: `cards/10004/10434120.json`

```json
{
  "id": "10434120",
  "name": "Cagliostro, Genius Alchemist",
  "kind": "follower",
  "class": "runecraft",
  "set": 10004,
  "rarity": "legendary",
  "token": false,
  "cost": 4,
  "text": "Fanfare: Gain 2 earth sigils. Add an Ars Magna to your hand.\nSkybound Art - Evolve this follower.\nSuper Skybound Art - Gain Crest: Cagliostro, Genius Alchemist.",
  "attack": 5,
  "defense": 3,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Gain 2 earth sigils. Add an Ars Magna to your hand.\nSkybound Art - Evolve this follower.\nSuper Skybound Art - Gain Crest: Cagliostro, Genius Alchemist.",
      "effects": [
        {
          "printed": "Gain 2 earth sigils.",
          "op": "counter",
          "key": "earth",
          "how": "add",
          "amount": 2
        },
        {
          "printed": "Add an Ars Magna to your hand.",
          "op": "addToHand",
          "card": {
            "named": "90034320"
          },
          "count": 1
        },
        {
          "printed": "Skybound Art - Evolve this follower.",
          "op": "if",
          "cond": {
            "skyboundArt": {
              "n": 10
            }
          },
          "then": [
            {
              "op": "evolve",
              "select": {
                "pick": "self"
              },
              "super": false
            }
          ]
        },
        {
          "printed": "Super Skybound Art - Gain Crest: Cagliostro, Genius Alchemist.",
          "op": "if",
          "cond": {
            "skyboundArt": {
              "n": 15
            }
          },
          "then": [
            {
              "op": "crest",
              "gain": "crest:10434120",
              "player": "self"
            }
          ]
        }
      ]
    }
  ]
}
```

### `10954110` — Istyndet vs. Mitilykket

repeat around reanimate; independent AoE; superEvolve crest.

Printed text:

```
Fanfare: Do this 3 times: "Reanimate (2)." Deal 2 damage to all enemy followers.
Super-Evolve: Gain Crest: Istyndet vs. Mitilykket.
```

File: `cards/10009/10954110.json`

```json
{
  "id": "10954110",
  "name": "Istyndet vs. Mitilykket",
  "kind": "follower",
  "class": "abysscraft",
  "set": 10009,
  "rarity": "legendary",
  "token": false,
  "cost": 7,
  "text": "Fanfare: Do this 3 times: \"Reanimate (2).\" Deal 2 damage to all enemy followers.\nSuper-Evolve: Gain Crest: Istyndet vs. Mitilykket.",
  "attack": 3,
  "defense": 3,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Do this 3 times: \"Reanimate (2).\" Deal 2 damage to all enemy followers.",
      "effects": [
        {
          "printed": "Do this 3 times: \"Reanimate (2).\"",
          "op": "repeat",
          "times": 3,
          "effects": [
            {
              "op": "reanimate",
              "maxCost": 2
            }
          ]
        },
        {
          "printed": "Deal 2 damage to all enemy followers.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "all"
          },
          "amount": 2
        }
      ]
    },
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: Gain Crest: Istyndet vs. Mitilykket.",
      "effects": [
        {
          "printed": "Gain Crest: Istyndet vs. Mitilykket.",
          "op": "crest",
          "gain": "crest:10954110",
          "player": "self"
        }
      ]
    }
  ]
}
```

### `10404110` — Sandalphon, Primarch Successor

zone deck + evolvedCountAtLeast; invoke; on invoked; Super Skybound; crest countdown + endOfTurn.

Printed text:

```
Activates in deck. At the start of your turn, if allied followers have evolved at least 6 times this match, Invoke this card.
When this card is Invoked, gain Crest: Sandalphon, Primarch Successor and return this card to hand
Fanfare: Super Skybound Art - Do this 5 times: "Deal 2 damage to a random enemy."
```

File: `cards/10004/10404110.json`

```json
{
  "id": "10404110",
  "name": "Sandalphon, Primarch Successor",
  "kind": "follower",
  "class": "neutral",
  "set": 10004,
  "rarity": "legendary",
  "token": false,
  "cost": 6,
  "text": "Activates in deck. At the start of your turn, if allied followers have evolved at least 6 times this match, Invoke this card.\n\nWhen this card is Invoked, gain Crest: Sandalphon, Primarch Successor and return this card to hand.\nFanfare: Super Skybound Art - Do this 5 times: \"Deal 2 damage to a random enemy.\"",
  "attack": 7,
  "defense": 6,
  "abilities": [
    {
      "on": "startOfTurn",
      "whose": "own",
      "zone": "deck",
      "printed": "Activates in deck. At the start of your turn, if allied followers have evolved at least 6 times this match, Invoke this card.",
      "when": {
        "evolvedCountAtLeast": {
          "n": 6
        }
      },
      "effects": [
        {
          "printed": "Invoke this card.",
          "op": "invoke"
        }
      ]
    },
    {
      "on": "invoked",
      "printed": "When this card is Invoked, gain Crest: Sandalphon, Primarch Successor and return this card to hand",
      "effects": [
        {
          "printed": "gain Crest: Sandalphon, Primarch Successor and return this card to hand",
          "op": "seq",
          "effects": [
            {
              "op": "crest",
              "gain": "crest:10404110",
              "player": "self"
            },
            {
              "op": "returnToHand",
              "select": {
                "pick": "self"
              }
            }
          ]
        }
      ]
    },
    {
      "on": "fanfare",
      "printed": "Fanfare: Super Skybound Art - Do this 5 times: \"Deal 2 damage to a random enemy.\"",
      "effects": [
        {
          "printed": "Super Skybound Art - Do this 5 times: \"Deal 2 damage to a random enemy.\"",
          "op": "if",
          "cond": {
            "skyboundArt": {
              "n": 15
            }
          },
          "then": [
            {
              "op": "repeat",
              "times": 5,
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "side": "enemy",
                    "kind": "character",
                    "pick": "random",
                    "includeLeader": true,
                    "zone": "field"
                  },
                  "amount": 2
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### `10904110` — Zerael, Sundered Rebirth

playedBaseCostsThisMatch; invoke; Intimidate.

Printed text:

```
Activates in deck. At the end of your turn, if you've played cards with base costs of 1, 2, 3, 4, 5, 6, 7, and 8 this match, Invoke this card.
Fanfare: Select an enemy follower on the field and deal it 9 damage.
Intimidate
```

File: `cards/10009/10904110.json`

```json
{
  "id": "10904110",
  "name": "Zerael, Sundered Rebirth",
  "kind": "follower",
  "class": "neutral",
  "set": 10009,
  "rarity": "legendary",
  "token": false,
  "cost": 9,
  "text": "Activates in deck. At the end of your turn, if you've played cards with base costs of 1, 2, 3, 4, 5, 6, 7, and 8 this match, Invoke this card.\nFanfare: Select an enemy follower on the field and deal it 9 damage.\nIntimidate",
  "attack": 9,
  "defense": 9,
  "traits": {
    "intimidate": true
  },
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "zone": "deck",
      "printed": "Activates in deck. At the end of your turn, if you've played cards with base costs of 1, 2, 3, 4, 5, 6, 7, and 8 this match, Invoke this card.",
      "when": {
        "playedBaseCostsThisMatch": [
          1,
          2,
          3,
          4,
          5,
          6,
          7,
          8
        ]
      },
      "effects": [
        {
          "printed": "Invoke this card.",
          "op": "invoke"
        }
      ]
    },
    {
      "on": "fanfare",
      "printed": "Fanfare: Select an enemy follower on the field and deal it 9 damage.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field and deal it 9 damage.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "choose"
          },
          "amount": 9
        }
      ]
    }
  ]
}
```

### `10564120` — Kukishiro, Mistbloom

Fanfare crest + returnToDeck 2 random + draw; Rush.

Printed text:

```
Fanfare: Gain Crest: Kukishiro, Mistbloom. Return 2 random cards from your hand to deck. Draw 2 cards.
Rush
```

File: `cards/10005/10564120.json`

```json
{
  "id": "10564120",
  "name": "Kukishiro, Mistbloom",
  "kind": "follower",
  "class": "havencraft",
  "set": 10005,
  "rarity": "legendary",
  "token": false,
  "cost": 7,
  "text": "Fanfare: Gain Crest: Kukishiro, Mistbloom. Return 2 random cards from your hand to deck. Draw 2 cards.\nRush",
  "attack": 5,
  "defense": 5,
  "traits": {
    "rush": true
  },
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Gain Crest: Kukishiro, Mistbloom. Return 2 random cards from your hand to deck. Draw 2 cards.",
      "effects": [
        {
          "printed": "Gain Crest: Kukishiro, Mistbloom.",
          "op": "crest",
          "gain": "crest:10564120",
          "player": "self"
        },
        {
          "printed": "Return 2 random cards from your hand to deck.",
          "op": "returnToDeck",
          "select": {
            "side": "ally",
            "zone": "hand",
            "kind": "card",
            "pick": "randomDistinct",
            "count": 2
          },
          "position": "random"
        },
        {
          "printed": "Draw 2 cards.",
          "op": "draw",
          "count": 2
        }
      ]
    }
  ]
}
```

### `10714110` — Thestae, Anathema of Distortion

buff with neg stat-of-self; counter combo; Evolve crest that buffs deck (persists when drawn).

Printed text:

```
Fanfare: Select an enemy follower on the field and give it -0/-X. X is this follower's attack. Increase your Combo by 1.
Evolve: Gain Crest: Thestae, Anathema of Distortion.
```

File: `cards/10007/10714110.json`

```json
{
  "id": "10714110",
  "name": "Thestae, Anathema of Distortion",
  "kind": "follower",
  "class": "forestcraft",
  "set": 10007,
  "rarity": "legendary",
  "token": false,
  "cost": 4,
  "text": "Fanfare: Select an enemy follower on the field and give it -0/-X. X is this follower's attack. Increase your Combo by 1.\nEvolve: Gain Crest: Thestae, Anathema of Distortion.",
  "attack": 3,
  "defense": 3,
  "tribes": [
    "anathema"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Select an enemy follower on the field and give it -0/-X. X is this follower's attack. Increase your Combo by 1.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field and give it -0/-X. X is this follower's attack.",
          "op": "buff",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "choose"
          },
          "attack": 0,
          "defense": {
            "neg": {
              "stat": {
                "of": {
                  "pick": "self"
                },
                "which": "attack"
              }
            }
          }
        },
        {
          "printed": "Increase your Combo by 1.",
          "op": "counter",
          "key": "combo",
          "how": "add",
          "amount": 1
        }
      ]
    },
    {
      "on": "evolve",
      "printed": "Evolve: Gain Crest: Thestae, Anathema of Distortion.",
      "effects": [
        {
          "printed": "Gain Crest: Thestae, Anathema of Distortion.",
          "op": "crest",
          "gain": "crest:10714110",
          "player": "self"
        }
      ]
    }
  ]
}
```

### `10934110` — Sephie, Maven Convict

fuse any card; on fused + pay pp 2; Fanfare summon 2 sequential; crest oncePerTurn + enter filter.

Printed text:

```
Fuse: Cards
Whenever you Fuse to this card, spend 2 play points to summon an Obsessed Test Subject.
Fanfare: Summon 2 copies of Obsessed Test Subject.
Super-Evolve: Gain Crest: Sephie, Maven Convict.
```

File: `cards/10009/10934110.json`

```json
{
  "id": "10934110",
  "name": "Sephie, Maven Convict",
  "kind": "follower",
  "class": "runecraft",
  "set": 10009,
  "rarity": "legendary",
  "token": false,
  "cost": 7,
  "text": "Fuse: Cards\nWhenever you Fuse to this card, spend 2 play points to summon an Obsessed Test Subject.\nFanfare: Summon 2 copies of Obsessed Test Subject.\nSuper-Evolve: Gain Crest: Sephie, Maven Convict.",
  "attack": 4,
  "defense": 4,
  "abilities": [
    {
      "on": "fused",
      "printed": "Whenever you Fuse to this card, spend 2 play points to summon an Obsessed Test Subject.",
      "effects": [
        {
          "printed": "spend 2 play points to summon an Obsessed Test Subject.",
          "op": "pay",
          "resource": "pp",
          "amount": 2,
          "effects": [
            {
              "op": "summon",
              "card": {
                "named": "10931110"
              },
              "count": 1
            }
          ]
        }
      ]
    },
    {
      "on": "fanfare",
      "printed": "Fanfare: Summon 2 copies of Obsessed Test Subject.",
      "effects": [
        {
          "printed": "Summon 2 copies of Obsessed Test Subject.",
          "op": "summon",
          "card": {
            "named": "10931110"
          },
          "count": 2
        }
      ]
    },
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: Gain Crest: Sephie, Maven Convict.",
      "effects": [
        {
          "printed": "Gain Crest: Sephie, Maven Convict.",
          "op": "crest",
          "gain": "crest:10934110",
          "player": "self"
        }
      ]
    }
  ],
  "fuse": {
    "partners": {},
    "printed": "Fuse: Cards"
  }
}
```

### `10933110` — Ecstatic Scholar

wasFused on superEvolve; pick choose.

Printed text:

```
Fuse: Cards
Fanfare: Draw 2 cards. Summon an Obsessed Test Subject.
Super-Evolve: If you've Fused to this card, select an allied Obsessed Test Subject on the field and give it Drain.
```

File: `cards/10009/10933110.json`

```json
{
  "id": "10933110",
  "name": "Ecstatic Scholar",
  "kind": "follower",
  "class": "runecraft",
  "set": 10009,
  "rarity": "gold",
  "token": false,
  "cost": 5,
  "text": "Fuse: Cards\nFanfare: Draw 2 cards. Summon an Obsessed Test Subject.\nSuper-Evolve: If you've Fused to this card, select an allied Obsessed Test Subject on the field and give it Drain.",
  "attack": 2,
  "defense": 3,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Draw 2 cards. Summon an Obsessed Test Subject.",
      "effects": [
        {
          "printed": "Draw 2 cards.",
          "op": "draw",
          "count": 2
        },
        {
          "printed": "Summon an Obsessed Test Subject.",
          "op": "summon",
          "card": {
            "named": "10931110"
          },
          "count": 1
        }
      ]
    },
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: If you've Fused to this card, select an allied Obsessed Test Subject on the field and give it Drain.",
      "when": {
        "wasFused": true
      },
      "effects": [
        {
          "printed": "select an allied Obsessed Test Subject on the field and give it Drain.",
          "op": "grantTraits",
          "select": {
            "side": "ally",
            "zone": "field",
            "kind": "follower",
            "pick": "choose",
            "filter": {
              "card": "10931110"
            }
          },
          "traits": {
            "drain": true
          }
        }
      ]
    }
  ],
  "fuse": {
    "partners": {},
    "printed": "Fuse: Cards"
  }
}
```

### `10703210` — City of Babelon

Countdown 1; endOfTurn sequence step 3 destroys self; Engage 1 choose discard + countdown +1.

Printed text:

```
Countdown (1)
At the end of your turn, activate an ability in sequence from the following.
1. Deal 2 damage to a random enemy follower.
2. Restore 2 defense to your leader.
3. Deal 2 damage to the enemy leader. Destroy this card.
Engage (1): Select a card in your hand and discard it. Delay the count of this amulet by 1.
```

File: `cards/10007/10703210.json`

```json
{
  "id": "10703210",
  "name": "City of Babelon",
  "kind": "amulet",
  "class": "neutral",
  "set": 10007,
  "rarity": "gold",
  "token": false,
  "cost": 1,
  "text": "Countdown (1)\nAt the end of your turn, activate an ability in sequence from the following.\n1. Deal 2 damage to a random enemy follower.\n2. Restore 2 defense to your leader.\n3. Deal 2 damage to the enemy leader. Destroy this card.\nEngage (1): Select a card in your hand and discard it. Delay the count of this amulet by 1.",
  "countdown": 1,
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, activate an ability in sequence from the following.\n1. Deal 2 damage to a random enemy follower.\n2. Restore 2 defense to your leader.\n3. Deal 2 damage to the enemy leader. Destroy this card.",
      "effects": [
        {
          "printed": "activate an ability in sequence from the following.\n1. Deal 2 damage to a random enemy follower.\n2. Restore 2 defense to your leader.\n3. Deal 2 damage to the enemy leader. Destroy this card.",
          "op": "sequence",
          "steps": [
            {
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "side": "enemy",
                    "zone": "field",
                    "kind": "follower",
                    "pick": "random"
                  },
                  "amount": 2
                }
              ],
              "printed": "1. Deal 2 damage to a random enemy follower."
            },
            {
              "effects": [
                {
                  "op": "restore",
                  "select": {
                    "pick": "all",
                    "side": "ally",
                    "zone": "leader",
                    "kind": "leader"
                  },
                  "amount": 2
                }
              ],
              "printed": "2. Restore 2 defense to your leader."
            },
            {
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "pick": "all",
                    "side": "enemy",
                    "zone": "leader",
                    "kind": "leader"
                  },
                  "amount": 2
                },
                {
                  "op": "destroy",
                  "select": {
                    "pick": "self"
                  }
                }
              ],
              "printed": "3. Deal 2 damage to the enemy leader. Destroy this card."
            }
          ]
        }
      ]
    },
    {
      "on": "engage",
      "cost": 1,
      "sacrifice": false,
      "printed": "Engage (1): Select a card in your hand and discard it. Delay the count of this amulet by 1.",
      "effects": [
        {
          "printed": "Select a card in your hand and discard it.",
          "op": "discard",
          "select": {
            "side": "ally",
            "zone": "hand",
            "kind": "card",
            "pick": "choose"
          }
        },
        {
          "printed": "Delay the count of this amulet by 1.",
          "op": "countdown",
          "select": {
            "pick": "self"
          },
          "delta": 1
        }
      ]
    }
  ]
}
```

### `10604110` — Omegotep, the Dreaded One

choose pick 2 by random; option 4 buff + replicate fanfare (re-entrant); pp recover; superEvolve replicate.

Printed text:

```
Fanfare: Activate 2 random abilities from the following.
1. Destroy a random enemy follower.
2. Deal 2 damage to the enemy leader.
3. Recover 2 play points.
4. Give this follower +4/+4 and activate its Fanfare ability.
Super-Evolve: Replicate the effects of this card's Fanfare ability.
```

File: `cards/10006/10604110.json`

```json
{
  "id": "10604110",
  "name": "Omegotep, the Dreaded One",
  "kind": "follower",
  "class": "neutral",
  "set": 10006,
  "rarity": "legendary",
  "token": false,
  "cost": 9,
  "text": "Fanfare: Activate 2 random abilities from the following.\n1. Destroy a random enemy follower.\n2. Deal 2 damage to the enemy leader.\n3. Recover 2 play points.\n4. Give this follower +4/+4 and activate its Fanfare ability.\nSuper-Evolve: Replicate the effects of this card's Fanfare ability.",
  "attack": 4,
  "defense": 4,
  "tribes": [
    "encroacher"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Activate 2 random abilities from the following.\n1. Destroy a random enemy follower.\n2. Deal 2 damage to the enemy leader.\n3. Recover 2 play points.\n4. Give this follower +4/+4 and activate its Fanfare ability.",
      "effects": [
        {
          "printed": "Activate 2 random abilities from the following.\n1. Destroy a random enemy follower.\n2. Deal 2 damage to the enemy leader.\n3. Recover 2 play points.\n4. Give this follower +4/+4 and activate its Fanfare ability.",
          "op": "choose",
          "pick": 2,
          "by": "random",
          "options": [
            {
              "printed": "1. Destroy a random enemy follower.",
              "effects": [
                {
                  "op": "destroy",
                  "select": {
                    "side": "enemy",
                    "zone": "field",
                    "kind": "follower",
                    "pick": "random"
                  }
                }
              ]
            },
            {
              "printed": "2. Deal 2 damage to the enemy leader.",
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "pick": "all",
                    "side": "enemy",
                    "zone": "leader",
                    "kind": "leader"
                  },
                  "amount": 2
                }
              ]
            },
            {
              "printed": "3. Recover 2 play points.",
              "effects": [
                {
                  "op": "pp",
                  "action": "recover",
                  "amount": 2
                }
              ]
            },
            {
              "printed": "4. Give this follower +4/+4 and activate its Fanfare ability.",
              "effects": [
                {
                  "op": "buff",
                  "select": {
                    "pick": "self"
                  },
                  "attack": 4,
                  "defense": 4
                },
                {
                  "op": "replicate",
                  "ability": "fanfare"
                }
              ]
            }
          ]
        }
      ]
    },
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: Replicate the effects of this card's Fanfare ability.",
      "effects": [
        {
          "printed": "Replicate the effects of this card's Fanfare ability.",
          "op": "replicate",
          "ability": "fanfare"
        }
      ]
    }
  ]
}
```

### `10923110` — Roughwater First Mate

chosen damage + summon token amulet; Evolve replicate; superEvolve addToHand two named + cost set 0.

Printed text:

```
Fanfare: Select an enemy follower on the field and deal it 3 damage. Summon a Dread Pirate's Flag.
Evolve: Replicate the effects of this card's Fanfare ability.
Super-Evolve: Add a Gilded Blade and Gilded Necklace to your hand and set their costs to 0.
```

File: `cards/10009/10923110.json`

```json
{
  "id": "10923110",
  "name": "Roughwater First Mate",
  "kind": "follower",
  "class": "swordcraft",
  "set": 10009,
  "rarity": "gold",
  "token": false,
  "cost": 5,
  "text": "Fanfare: Select an enemy follower on the field and deal it 3 damage. Summon a Dread Pirate's Flag.\nEvolve: Replicate the effects of this card's Fanfare ability.\nSuper-Evolve: Add a Gilded Blade and Gilded Necklace to your hand and set their costs to 0.",
  "attack": 3,
  "defense": 3,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Select an enemy follower on the field and deal it 3 damage. Summon a Dread Pirate's Flag.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field and deal it 3 damage.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "choose"
          },
          "amount": 3
        },
        {
          "printed": "Summon a Dread Pirate's Flag.",
          "op": "summon",
          "card": {
            "named": "90021210"
          },
          "count": 1
        }
      ]
    },
    {
      "on": "evolve",
      "printed": "Evolve: Replicate the effects of this card's Fanfare ability.",
      "effects": [
        {
          "printed": "Replicate the effects of this card's Fanfare ability.",
          "op": "replicate",
          "ability": "fanfare"
        }
      ]
    },
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: Add a Gilded Blade and Gilded Necklace to your hand and set their costs to 0.",
      "effects": [
        {
          "printed": "Add a Gilded Blade and Gilded Necklace to your hand and set their costs to 0.",
          "op": "seq",
          "effects": [
            {
              "op": "addToHand",
              "card": {
                "named": "90021310"
              },
              "count": 1,
              "as": "a"
            },
            {
              "op": "addToHand",
              "card": {
                "named": "90021340"
              },
              "count": 1,
              "as": "b"
            },
            {
              "op": "cost",
              "select": {
                "pick": "bound",
                "ref": "a"
              },
              "set": 0
            },
            {
              "op": "cost",
              "select": {
                "pick": "bound",
                "ref": "b"
              },
              "set": 0
            }
          ]
        }
      ]
    }
  ]
}
```

### `10564110` — Sofina, Inspiring Strength

choose; effect-granted evolve on random filtered ally with other.

Printed text:

```
Fanfare: Select a Mode to activate.
1. Evolve this follower.
2. Evolve another random unevolved allied follower with Ward and give it +1/+1.
Ward
At the end of your turn, if this follower is evolved, give all other followers on the field -1/-1.
```

File: `cards/10005/10564110.json`

```json
{
  "id": "10564110",
  "name": "Sofina, Inspiring Strength",
  "kind": "follower",
  "class": "havencraft",
  "set": 10005,
  "rarity": "legendary",
  "token": false,
  "cost": 4,
  "text": "Fanfare: Select a Mode to activate.\n1. Evolve this follower.\n2. Evolve another random unevolved allied follower with Ward and give it +1/+1.\nWard\nAt the end of your turn, if this follower is evolved, give all other followers on the field -1/-1.",
  "attack": 2,
  "defense": 2,
  "traits": {
    "ward": true
  },
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Select a Mode to activate.\n1. Evolve this follower.\n2. Evolve another random unevolved allied follower with Ward and give it +1/+1.",
      "effects": [
        {
          "printed": "Select a Mode to activate.\n1. Evolve this follower.\n2. Evolve another random unevolved allied follower with Ward and give it +1/+1.",
          "op": "choose",
          "pick": 1,
          "by": "player",
          "options": [
            {
              "printed": "1. Evolve this follower.",
              "effects": [
                {
                  "op": "evolve",
                  "select": {
                    "pick": "self"
                  },
                  "super": false
                }
              ]
            },
            {
              "printed": "2. Evolve another random unevolved allied follower with Ward and give it +1/+1.",
              "effects": [
                {
                  "op": "seq",
                  "effects": [
                    {
                      "op": "evolve",
                      "select": {
                        "side": "ally",
                        "zone": "field",
                        "kind": "follower",
                        "pick": "random",
                        "other": true,
                        "filter": {
                          "unevolved": true,
                          "hasTrait": "ward"
                        }
                      },
                      "as": "t",
                      "super": false
                    },
                    {
                      "op": "buff",
                      "select": {
                        "pick": "bound",
                        "ref": "t"
                      },
                      "attack": 1,
                      "defense": 1
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    },
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, if this follower is evolved, give all other followers on the field -1/-1.",
      "when": {
        "evolved": true
      },
      "effects": [
        {
          "printed": "give all other followers on the field -1/-1.",
          "op": "buff",
          "select": {
            "side": "any",
            "zone": "field",
            "kind": "follower",
            "pick": "all",
            "other": true
          },
          "attack": -1,
          "defense": -1
        }
      ]
    }
  ]
}
```

### `10854110` — Itsurugi & Taketsumi, Brothers

choose on Fanfare and Evolve; two-sentence options; pp recover; ep gain.

Printed text:

```
Fanfare: Select a Mode to activate.
1. Deal 4 damage to the enemy leader. Restore 4 defense to your leader.
2. Deal 5 damage to all enemy followers. Recover 1 evolution point.
Evolve: Select a Mode to activate.
1. Draw 2 cards.
2. Recover 2 play points.
```

File: `cards/10008/10854110.json`

```json
{
  "id": "10854110",
  "name": "Itsurugi & Taketsumi, Brothers",
  "kind": "follower",
  "class": "abysscraft",
  "set": 10008,
  "rarity": "legendary",
  "token": false,
  "cost": 8,
  "text": "Fanfare: Select a Mode to activate.\n1. Deal 4 damage to the enemy leader. Restore 4 defense to your leader.\n2. Deal 5 damage to all enemy followers. Recover 1 evolution point.\nEvolve: Select a Mode to activate.\n1. Draw 2 cards.\n2. Recover 2 play points.",
  "attack": 6,
  "defense": 5,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Select a Mode to activate.\n1. Deal 4 damage to the enemy leader. Restore 4 defense to your leader.\n2. Deal 5 damage to all enemy followers. Recover 1 evolution point.",
      "effects": [
        {
          "printed": "Select a Mode to activate.\n1. Deal 4 damage to the enemy leader. Restore 4 defense to your leader.\n2. Deal 5 damage to all enemy followers. Recover 1 evolution point.",
          "op": "choose",
          "pick": 1,
          "by": "player",
          "options": [
            {
              "printed": "1. Deal 4 damage to the enemy leader. Restore 4 defense to your leader.",
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "pick": "all",
                    "side": "enemy",
                    "zone": "leader",
                    "kind": "leader"
                  },
                  "amount": 4
                },
                {
                  "op": "restore",
                  "select": {
                    "pick": "all",
                    "side": "ally",
                    "zone": "leader",
                    "kind": "leader"
                  },
                  "amount": 4
                }
              ]
            },
            {
              "printed": "2. Deal 5 damage to all enemy followers. Recover 1 evolution point.",
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "side": "enemy",
                    "zone": "field",
                    "kind": "follower",
                    "pick": "all"
                  },
                  "amount": 5
                },
                {
                  "op": "ep",
                  "action": "gain",
                  "super": false,
                  "amount": 1
                }
              ]
            }
          ]
        }
      ]
    },
    {
      "on": "evolve",
      "printed": "Evolve: Select a Mode to activate.\n1. Draw 2 cards.\n2. Recover 2 play points.",
      "effects": [
        {
          "printed": "Select a Mode to activate.\n1. Draw 2 cards.\n2. Recover 2 play points.",
          "op": "choose",
          "pick": 1,
          "by": "player",
          "options": [
            {
              "printed": "1. Draw 2 cards.",
              "effects": [
                {
                  "op": "draw",
                  "count": 2
                }
              ]
            },
            {
              "printed": "2. Recover 2 play points.",
              "effects": [
                {
                  "op": "pp",
                  "action": "recover",
                  "amount": 2
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### `10633310` — Bewitching Eld Crystals

choose; seq binding; Enhance replacesBase pick all.

Printed text:

```
Select a Mode to activate.
1. Summon a Crystalspawn and give it +1/+0 and Storm.
2. Summon 2 copies of Crystalspawn and give them +1/+0.
Enhance (5): Activate all of them instead.
```

File: `cards/10006/10633310.json`

```json
{
  "id": "10633310",
  "name": "Bewitching Eld Crystals",
  "kind": "spell",
  "class": "runecraft",
  "set": 10006,
  "rarity": "gold",
  "token": false,
  "cost": 3,
  "text": "Select a Mode to activate.\n1. Summon a Crystalspawn and give it +1/+0 and Storm.\n2. Summon 2 copies of Crystalspawn and give them +1/+0.\nEnhance (5): Activate all of them instead.",
  "tribes": [
    "encroacher"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select a Mode to activate.\n1. Summon a Crystalspawn and give it +1/+0 and Storm.\n2. Summon 2 copies of Crystalspawn and give them +1/+0.",
      "effects": [
        {
          "printed": "Select a Mode to activate.\n1. Summon a Crystalspawn and give it +1/+0 and Storm.\n2. Summon 2 copies of Crystalspawn and give them +1/+0.",
          "op": "choose",
          "pick": 1,
          "by": "player",
          "options": [
            {
              "printed": "1. Summon a Crystalspawn and give it +1/+0 and Storm.",
              "effects": [
                {
                  "op": "seq",
                  "effects": [
                    {
                      "op": "summon",
                      "card": {
                        "named": "10631110"
                      },
                      "count": 1,
                      "as": "s"
                    },
                    {
                      "op": "buff",
                      "select": {
                        "pick": "bound",
                        "ref": "s"
                      },
                      "attack": 1,
                      "defense": 0
                    },
                    {
                      "op": "grantTraits",
                      "select": {
                        "pick": "bound",
                        "ref": "s"
                      },
                      "traits": {
                        "storm": true
                      }
                    }
                  ]
                }
              ]
            },
            {
              "printed": "2. Summon 2 copies of Crystalspawn and give them +1/+0.",
              "effects": [
                {
                  "op": "seq",
                  "effects": [
                    {
                      "op": "summon",
                      "card": {
                        "named": "10631110"
                      },
                      "count": 2,
                      "as": "s"
                    },
                    {
                      "op": "buff",
                      "select": {
                        "pick": "bound",
                        "ref": "s"
                      },
                      "attack": 1,
                      "defense": 0
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  ],
  "modes": [
    {
      "kind": "enhance",
      "cost": 5,
      "replacesBase": true,
      "printed": "Enhance (5): Activate all of them instead.",
      "effects": [
        {
          "printed": "Activate all of them instead.",
          "op": "choose",
          "pick": "all",
          "by": "player",
          "optionsFrom": "fanfare"
        }
      ]
    }
  ]
}
```

### `10634120` — Calge-Danthla, Eld Crystals

zone hand cost reduction; Faith faith:10634120.

Printed text:

```
Activates in hand. Whenever an allied Crystalspawn enters the field, reduce the cost of this card by 1.
Fanfare: Summon 2 copies of Crystalspawn and give them Storm.
Evolve: Add a Depths of the Eld Crystals to your hand.
```

File: `cards/10006/10634120.json`

```json
{
  "id": "10634120",
  "name": "Calge-Danthla, Eld Crystals",
  "kind": "follower",
  "class": "runecraft",
  "set": 10006,
  "rarity": "legendary",
  "token": false,
  "cost": 10,
  "text": "Activates in hand. Whenever an allied Crystalspawn enters the field, reduce the cost of this card by 1.\nFanfare: Summon 2 copies of Crystalspawn and give them Storm.\nEvolve: Add a Depths of the Eld Crystals to your hand.",
  "attack": 5,
  "defense": 5,
  "tribes": [
    "encroacher"
  ],
  "abilities": [
    {
      "on": "when",
      "event": "ally_follower_enter",
      "zone": "hand",
      "filter": {
        "card": "10631110"
      },
      "printed": "Activates in hand. Whenever an allied Crystalspawn enters the field, reduce the cost of this card by 1.",
      "effects": [
        {
          "printed": "reduce the cost of this card by 1.",
          "op": "cost",
          "select": {
            "pick": "self"
          },
          "delta": -1
        }
      ]
    },
    {
      "on": "fanfare",
      "printed": "Fanfare: Summon 2 copies of Crystalspawn and give them Storm.",
      "effects": [
        {
          "printed": "Summon 2 copies of Crystalspawn and give them Storm.",
          "op": "seq",
          "effects": [
            {
              "op": "summon",
              "card": {
                "named": "10631110"
              },
              "count": 2,
              "as": "s"
            },
            {
              "op": "grantTraits",
              "select": {
                "pick": "bound",
                "ref": "s"
              },
              "traits": {
                "storm": true
              }
            }
          ]
        }
      ]
    },
    {
      "on": "evolve",
      "printed": "Evolve: Add a Depths of the Eld Crystals to your hand.",
      "effects": [
        {
          "printed": "Add a Depths of the Eld Crystals to your hand.",
          "op": "addToHand",
          "card": {
            "named": "90034330"
          },
          "count": 1
        }
      ]
    }
  ]
}
```

### `90034330` — Depths of the Eld Crystals

randomSplit over the faith counter.

Printed text:

```
Summon a Crystalspawn and give it +X/+X. Restore Y defense to your leader. Deal Z damage to the enemy leader. X, Y, and Z are determined randomly and add up to your faith's value.
```

File: `cards/tokens/90034330.json`

```json
{
  "id": "90034330",
  "name": "Depths of the Eld Crystals",
  "kind": "spell",
  "class": "runecraft",
  "set": 90000,
  "rarity": "legendary",
  "token": true,
  "cost": 6,
  "text": "Summon a Crystalspawn and give it +X/+X. Restore Y defense to your leader. Deal Z damage to the enemy leader. X, Y, and Z are determined randomly and add up to your faith's value.",
  "tribes": [
    "encroacher"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Summon a Crystalspawn and give it +X/+X. Restore Y defense to your leader. Deal Z damage to the enemy leader. X, Y, and Z are determined randomly and add up to your faith's value.",
      "effects": [
        {
          "printed": "Summon a Crystalspawn and give it +X/+X. Restore Y defense to your leader. Deal Z damage to the enemy leader. X, Y, and Z are determined randomly and add up to your faith's value.",
          "op": "randomSplit",
          "keys": [
            "X",
            "Y",
            "Z"
          ],
          "times": {
            "counter": "faith"
          },
          "effects": [
            {
              "printed": "Summon a Crystalspawn and give it +X/+X.",
              "op": "seq",
              "effects": [
                {
                  "op": "summon",
                  "card": {
                    "named": "10631110"
                  },
                  "count": 1,
                  "as": "s"
                },
                {
                  "op": "buff",
                  "select": {
                    "pick": "bound",
                    "ref": "s"
                  },
                  "attack": {
                    "var": "X"
                  },
                  "defense": {
                    "var": "X"
                  }
                }
              ]
            },
            {
              "printed": "Restore Y defense to your leader.",
              "op": "restore",
              "select": {
                "pick": "all",
                "side": "ally",
                "zone": "leader",
                "kind": "leader"
              },
              "amount": {
                "var": "Y"
              }
            },
            {
              "printed": "Deal Z damage to the enemy leader.",
              "op": "damage",
              "select": {
                "pick": "all",
                "side": "enemy",
                "zone": "leader",
                "kind": "leader"
              },
              "amount": {
                "var": "Z"
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### `10534120` — Ara, Dawnblossom

on spellboost in hand; Fanfare 10 damage choose; Evolve transform choose + other.

Printed text:

```
On Spellboost: Reduce the cost of this card by 1.
Fanfare: Select an enemy follower on the field and deal it 10 damage.
Evolve: Select another follower on the field and transform it into a Regal Falcon.
```

File: `cards/10005/10534120.json`

```json
{
  "id": "10534120",
  "name": "Ara, Dawnblossom",
  "kind": "follower",
  "class": "runecraft",
  "set": 10005,
  "rarity": "legendary",
  "token": false,
  "cost": 10,
  "text": "On Spellboost: Reduce the cost of this card by 1.\nFanfare: Select an enemy follower on the field and deal it 10 damage.\nEvolve: Select another follower on the field and transform it into a Regal Falcon.",
  "attack": 4,
  "defense": 4,
  "abilities": [
    {
      "on": "spellboost",
      "zone": "hand",
      "printed": "On Spellboost: Reduce the cost of this card by 1.",
      "effects": [
        {
          "printed": "Reduce the cost of this card by 1.",
          "op": "cost",
          "select": {
            "pick": "self"
          },
          "delta": -1
        }
      ]
    },
    {
      "on": "fanfare",
      "printed": "Fanfare: Select an enemy follower on the field and deal it 10 damage.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field and deal it 10 damage.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "choose"
          },
          "amount": 10
        }
      ]
    },
    {
      "on": "evolve",
      "printed": "Evolve: Select another follower on the field and transform it into a Regal Falcon.",
      "effects": [
        {
          "printed": "Select another follower on the field and transform it into a Regal Falcon.",
          "op": "transform",
          "select": {
            "side": "any",
            "zone": "field",
            "kind": "follower",
            "pick": "choose",
            "other": true
          },
          "into": {
            "named": "90061130"
          }
        }
      ]
    }
  ]
}
```

### `10573310` — Sincerity of the Dewdrop

transform; kind card on the field includes amulets.

Printed text:

```
Select a card on the field and transform it into an Imari's Little Buddies.
```

File: `cards/10005/10573310.json`

```json
{
  "id": "10573310",
  "name": "Sincerity of the Dewdrop",
  "kind": "spell",
  "class": "portalcraft",
  "set": 10005,
  "rarity": "gold",
  "token": false,
  "cost": 1,
  "text": "Select a card on the field and transform it into an Imari's Little Buddies.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select a card on the field and transform it into an Imari's Little Buddies.",
      "effects": [
        {
          "printed": "Select a card on the field and transform it into an Imari's Little Buddies.",
          "op": "transform",
          "select": {
            "side": "any",
            "zone": "field",
            "kind": "card",
            "pick": "choose"
          },
          "into": {
            "named": "90074140"
          }
        }
      ]
    }
  ]
}
```

### `10624110` — Noel IV, Ruthless Warlord

multi-tier Enhance (both + Fanfare); seq binding Bane.

Printed text:

```
Fanfare: Summon a Fearless Soldier and give it Bane.
Enhance (7): Summon a Fearless Soldier and give it Drain.
Enhance (8): Summon a Fearless Soldier and give it Storm.
Super-Evolve: Give all other allied followers on the field +1/+1.
```

File: `cards/10006/10624110.json`

```json
{
  "id": "10624110",
  "name": "Noel IV, Ruthless Warlord",
  "kind": "follower",
  "class": "swordcraft",
  "set": 10006,
  "rarity": "legendary",
  "token": false,
  "cost": 6,
  "text": "Fanfare: Summon a Fearless Soldier and give it Bane.\nEnhance (7): Summon a Fearless Soldier and give it Drain.\nEnhance (8): Summon a Fearless Soldier and give it Storm.\nSuper-Evolve: Give all other allied followers on the field +1/+1.",
  "attack": 4,
  "defense": 5,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Summon a Fearless Soldier and give it Bane.",
      "effects": [
        {
          "printed": "Summon a Fearless Soldier and give it Bane.",
          "op": "seq",
          "effects": [
            {
              "op": "summon",
              "card": {
                "named": "10621110"
              },
              "count": 1,
              "as": "s"
            },
            {
              "op": "grantTraits",
              "select": {
                "pick": "bound",
                "ref": "s"
              },
              "traits": {
                "bane": true
              }
            }
          ]
        }
      ]
    },
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: Give all other allied followers on the field +1/+1.",
      "effects": [
        {
          "printed": "Give all other allied followers on the field +1/+1.",
          "op": "buff",
          "select": {
            "side": "ally",
            "zone": "field",
            "kind": "follower",
            "pick": "all",
            "other": true
          },
          "attack": 1,
          "defense": 1
        }
      ]
    }
  ],
  "modes": [
    {
      "kind": "enhance",
      "cost": 7,
      "printed": "Enhance (7): Summon a Fearless Soldier and give it Drain.",
      "effects": [
        {
          "printed": "Summon a Fearless Soldier and give it Drain.",
          "op": "seq",
          "effects": [
            {
              "op": "summon",
              "card": {
                "named": "10621110"
              },
              "count": 1,
              "as": "s"
            },
            {
              "op": "grantTraits",
              "select": {
                "pick": "bound",
                "ref": "s"
              },
              "traits": {
                "drain": true
              }
            }
          ]
        }
      ]
    },
    {
      "kind": "enhance",
      "cost": 8,
      "printed": "Enhance (8): Summon a Fearless Soldier and give it Storm.",
      "effects": [
        {
          "printed": "Summon a Fearless Soldier and give it Storm.",
          "op": "seq",
          "effects": [
            {
              "op": "summon",
              "card": {
                "named": "10621110"
              },
              "count": 1,
              "as": "s"
            },
            {
              "op": "grantTraits",
              "select": {
                "pick": "bound",
                "ref": "s"
              },
              "traits": {
                "storm": true
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### `10001110` — Indomitable Fighter

simplest Enhance.

Printed text:

```
Enhance (4): Give this follower +3/+3.
```

File: `cards/10000/10001110.json`

```json
{
  "id": "10001110",
  "name": "Indomitable Fighter",
  "kind": "follower",
  "class": "neutral",
  "set": 10000,
  "rarity": "bronze",
  "token": false,
  "cost": 2,
  "text": "Enhance (4): Give this follower +3/+3.",
  "attack": 2,
  "defense": 2,
  "modes": [
    {
      "kind": "enhance",
      "cost": 4,
      "printed": "Enhance (4): Give this follower +3/+3.",
      "effects": [
        {
          "printed": "Give this follower +3/+3.",
          "op": "buff",
          "select": {
            "pick": "self"
          },
          "attack": 3,
          "defense": 3
        }
      ]
    }
  ]
}
```

### `10671110` — Shoddy Plaything

Accelerate; Ward; summoned copy is base cost 6 (ruling).

Printed text:

```
Fanfare: Draw 3 cards.
Ward
Accelerate (2): Summon a Shoddy Plaything.
```

File: `cards/10006/10671110.json`

```json
{
  "id": "10671110",
  "name": "Shoddy Plaything",
  "kind": "follower",
  "class": "portalcraft",
  "set": 10006,
  "rarity": "bronze",
  "token": false,
  "cost": 6,
  "text": "Fanfare: Draw 3 cards.\n\nWard",
  "attack": 1,
  "defense": 3,
  "traits": {
    "ward": true
  },
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Draw 3 cards.",
      "effects": [
        {
          "printed": "Draw 3 cards.",
          "op": "draw",
          "count": 3
        }
      ]
    }
  ],
  "modes": [
    {
      "kind": "accelerate",
      "cost": 2,
      "printed": "Accelerate (2): Summon a Shoddy Plaything.",
      "effects": [
        {
          "printed": "Summon a Shoddy Plaything.",
          "op": "summon",
          "card": {
            "named": "10671110"
          },
          "count": 1
        }
      ]
    }
  ]
}
```

### `10662110` — Venerating Dyer

Crystallize Countdown + Last Words summon.

Printed text:

```
Rush
Bane
Crystallize (1): Countdown (3)
Last Words: Summon a Venerating Dyer.
```

File: `cards/10006/10662110.json`

```json
{
  "id": "10662110",
  "name": "Venerating Dyer",
  "kind": "follower",
  "class": "havencraft",
  "set": 10006,
  "rarity": "silver",
  "token": false,
  "cost": 4,
  "text": "Rush\nBane",
  "attack": 5,
  "defense": 3,
  "traits": {
    "rush": true,
    "bane": true
  },
  "modes": [
    {
      "kind": "crystallize",
      "cost": 1,
      "printed": "Crystallize (1): Countdown (3)\nLast Words: Summon a Venerating Dyer.",
      "countdown": 3,
      "abilities": [
        {
          "on": "lastWords",
          "printed": "Last Words: Summon a Venerating Dyer.",
          "effects": [
            {
              "printed": "Summon a Venerating Dyer.",
              "op": "summon",
              "card": {
                "named": "10662110"
              },
              "count": 1
            }
          ]
        }
      ]
    }
  ]
}
```

### `10072210` — Puppet Theater

Countdown amulet; Fanfare; endOfTurn.

Printed text:

```
Fanfare: Add a Puppet to your hand.
Countdown (2)
At the end of your turn, add a Puppet to your hand.
```

File: `cards/10000/10072210.json`

```json
{
  "id": "10072210",
  "name": "Puppet Theater",
  "kind": "amulet",
  "class": "portalcraft",
  "set": 10000,
  "rarity": "silver",
  "token": false,
  "cost": 2,
  "text": "Fanfare: Add a Puppet to your hand.\nCountdown (2)\nAt the end of your turn, add a Puppet to your hand.",
  "countdown": 2,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Add a Puppet to your hand.",
      "effects": [
        {
          "printed": "Add a Puppet to your hand.",
          "op": "addToHand",
          "card": {
            "named": "90071110"
          },
          "count": 1
        }
      ]
    },
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, add a Puppet to your hand.",
      "effects": [
        {
          "printed": "add a Puppet to your hand.",
          "op": "addToHand",
          "card": {
            "named": "90071110"
          },
          "count": 1
        }
      ]
    }
  ]
}
```

### `10963210` — Juratio

Engage with sacrifice; Fanfare destroy all.

Printed text:

```
Fanfare: Destroy all followers.
Engage (1): Destroy this card. Draw a card. Restore 1 defense to your leader.
```

File: `cards/10009/10963210.json`

```json
{
  "id": "10963210",
  "name": "Juratio",
  "kind": "amulet",
  "class": "havencraft",
  "set": 10009,
  "rarity": "gold",
  "token": false,
  "cost": 6,
  "text": "Fanfare: Destroy all followers.\nEngage (1): Destroy this card. Draw a card. Restore 1 defense to your leader.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Destroy all followers.",
      "effects": [
        {
          "printed": "Destroy all followers.",
          "op": "destroy",
          "select": {
            "side": "any",
            "zone": "field",
            "kind": "follower",
            "pick": "all"
          }
        }
      ]
    },
    {
      "on": "engage",
      "cost": 1,
      "sacrifice": true,
      "printed": "Engage (1): Destroy this card. Draw a card. Restore 1 defense to your leader.",
      "effects": [
        {
          "printed": "Draw a card.",
          "op": "draw",
          "count": 1
        },
        {
          "printed": "Restore 1 defense to your leader.",
          "op": "restore",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 1
        }
      ]
    }
  ]
}
```

### `90051140` — Rotting Zombie

Last Words summon + removeAbilities on the bound summon.

Printed text:

```
Last Words: Summon a Rotting Zombie and remove Last Words from it.
```

File: `cards/tokens/90051140.json`

```json
{
  "id": "90051140",
  "name": "Rotting Zombie",
  "kind": "follower",
  "class": "abysscraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 3,
  "text": "Last Words: Summon a Rotting Zombie and remove Last Words from it.",
  "attack": 2,
  "defense": 2,
  "tribes": [
    "departed"
  ],
  "abilities": [
    {
      "on": "lastWords",
      "printed": "Last Words: Summon a Rotting Zombie and remove Last Words from it.",
      "effects": [
        {
          "printed": "Summon a Rotting Zombie and remove Last Words from it.",
          "op": "seq",
          "effects": [
            {
              "op": "summon",
              "card": {
                "named": "90051140"
              },
              "count": 1,
              "as": "s"
            },
            {
              "op": "removeAbilities",
              "select": {
                "pick": "bound",
                "ref": "s"
              },
              "on": [
                "lastWords"
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### `10654110` — Armes, Depletive Demon

Aura; cantBeDestroyedByAbilities; Clash; superEvolve attacksPerTurn 3.

Printed text:

```
Aura
Can't be destroyed by abilities.
Clash: Destroy the opposing follower.
Super-Evolve: Give this follower "Can attack 3 times per turn."
```

File: `cards/10006/10654110.json`

```json
{
  "id": "10654110",
  "name": "Armes, Depletive Demon",
  "kind": "follower",
  "class": "abysscraft",
  "set": 10006,
  "rarity": "legendary",
  "token": false,
  "cost": 9,
  "text": "Aura\nCan't be destroyed by abilities.\nClash: Destroy the opposing follower.\nSuper-Evolve: Give this follower \"Can attack 3 times per turn.\"",
  "attack": 10,
  "defense": 10,
  "traits": {
    "aura": true,
    "cantBeDestroyedByAbilities": true
  },
  "abilities": [
    {
      "on": "clash",
      "printed": "Clash: Destroy the opposing follower.",
      "effects": [
        {
          "printed": "Destroy the opposing follower.",
          "op": "destroy",
          "select": {
            "pick": "opposing"
          }
        }
      ]
    },
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: Give this follower \"Can attack 3 times per turn.\"",
      "effects": [
        {
          "printed": "Give this follower \"Can attack 3 times per turn.\"",
          "op": "grantTraits",
          "select": {
            "pick": "self"
          },
          "traits": {
            "attacksPerTurn": 3
          }
        }
      ]
    }
  ]
}
```

### `10704110` — Illamrita, Designated Target

followerStrike; grantAbility + quoted traits; Last Words crest.

Printed text:

```
Follower Strike: Give this follower Barrier. Give the opposing follower "Can't attack followers or leaders" and "At the end of your turn, banish this card."
Last Words: Gain Crest: Illamrita, Designated Target.
```

File: `cards/10007/10704110.json`

```json
{
  "id": "10704110",
  "name": "Illamrita, Designated Target",
  "kind": "follower",
  "class": "neutral",
  "set": 10007,
  "rarity": "legendary",
  "token": false,
  "cost": 6,
  "text": "Follower Strike: Give this follower Barrier. Give the opposing follower \"Can't attack followers or leaders\" and \"At the end of your turn, banish this card.\"\nLast Words: Gain Crest: Illamrita, Designated Target.",
  "attack": 1,
  "defense": 4,
  "abilities": [
    {
      "on": "followerStrike",
      "printed": "Follower Strike: Give this follower Barrier. Give the opposing follower \"Can't attack followers or leaders\" and \"At the end of your turn, banish this card.\"",
      "effects": [
        {
          "printed": "Give this follower Barrier.",
          "op": "grantTraits",
          "select": {
            "pick": "self"
          },
          "traits": {
            "barrier": true
          }
        },
        {
          "printed": "Give the opposing follower \"Can't attack followers or leaders\" and \"At the end of your turn, banish this card.\"",
          "op": "seq",
          "effects": [
            {
              "op": "grantTraits",
              "select": {
                "pick": "opposing"
              },
              "traits": {
                "cantAttackFollowers": true,
                "cantAttackLeader": true
              }
            },
            {
              "op": "grantAbility",
              "select": {
                "pick": "opposing"
              },
              "ability": {
                "on": "endOfTurn",
                "whose": "own",
                "printed": "At the end of your turn, banish this card.",
                "effects": [
                  {
                    "printed": "banish this card.",
                    "op": "banish",
                    "select": {
                      "pick": "self"
                    }
                  }
                ]
              }
            }
          ]
        }
      ]
    },
    {
      "on": "lastWords",
      "printed": "Last Words: Gain Crest: Illamrita, Designated Target.",
      "effects": [
        {
          "printed": "Gain Crest: Illamrita, Designated Target.",
          "op": "crest",
          "gain": "crest:10704110",
          "player": "self"
        }
      ]
    }
  ]
}
```

### `90044330` — Depths of the Eld Blades

on discarded.

Printed text:

```
When this card is discarded, deal 1 damage to the enemy leader and restore 1 defense to your leader.
Deal 1 damage to the enemy leader. Restore 1 defense to your leader.
```

File: `cards/tokens/90044330.json`

```json
{
  "id": "90044330",
  "name": "Depths of the Eld Blades",
  "kind": "spell",
  "class": "dragoncraft",
  "set": 90000,
  "rarity": "legendary",
  "token": true,
  "cost": 2,
  "text": "When this card is discarded, deal 1 damage to the enemy leader and restore 1 defense to your leader.\nDeal 1 damage to the enemy leader. Restore 1 defense to your leader.",
  "tribes": [
    "encroacher"
  ],
  "abilities": [
    {
      "on": "discarded",
      "printed": "When this card is discarded, deal 1 damage to the enemy leader and restore 1 defense to your leader.",
      "effects": [
        {
          "printed": "deal 1 damage to the enemy leader and restore 1 defense to your leader.",
          "op": "seq",
          "effects": [
            {
              "op": "damage",
              "select": {
                "pick": "all",
                "side": "enemy",
                "zone": "leader",
                "kind": "leader"
              },
              "amount": 1
            },
            {
              "op": "restore",
              "select": {
                "pick": "all",
                "side": "ally",
                "zone": "leader",
                "kind": "leader"
              },
              "amount": 1
            }
          ]
        }
      ]
    },
    {
      "on": "fanfare",
      "printed": "Deal 1 damage to the enemy leader. Restore 1 defense to your leader.",
      "effects": [
        {
          "printed": "Deal 1 damage to the enemy leader.",
          "op": "damage",
          "select": {
            "pick": "all",
            "side": "enemy",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 1
        },
        {
          "printed": "Restore 1 defense to your leader.",
          "op": "restore",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 1
        }
      ]
    }
  ]
}
```

### `10002110` — Arriet, Luxminstrel

superEvolve replaces evolve (instead).

Printed text:

```
Evolve: Restore 2 defense to your leader.
Super-Evolve: Restore 4 defense instead.
```

File: `cards/10000/10002110.json`

```json
{
  "id": "10002110",
  "name": "Arriet, Luxminstrel",
  "kind": "follower",
  "class": "neutral",
  "set": 10000,
  "rarity": "silver",
  "token": false,
  "cost": 3,
  "text": "Evolve: Restore 2 defense to your leader.\nSuper-Evolve: Restore 4 defense instead.",
  "attack": 3,
  "defense": 3,
  "abilities": [
    {
      "on": "evolve",
      "printed": "Evolve: Restore 2 defense to your leader.",
      "effects": [
        {
          "printed": "Restore 2 defense to your leader.",
          "op": "restore",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 2
        }
      ]
    },
    {
      "on": "superEvolve",
      "replaces": "evolve",
      "printed": "Super-Evolve: Restore 4 defense instead.",
      "effects": [
        {
          "printed": "Restore 4 defense instead.",
          "op": "restore",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 4
        }
      ]
    }
  ]
}
```

### `10032120` — Blaze Destroyer

on spellboost in hand.

Printed text:

```
On Spellboost: Reduce the cost of this card by 1.
```

File: `cards/10000/10032120.json`

```json
{
  "id": "10032120",
  "name": "Blaze Destroyer",
  "kind": "follower",
  "class": "runecraft",
  "set": 10000,
  "rarity": "silver",
  "token": false,
  "cost": 10,
  "text": "On Spellboost: Reduce the cost of this card by 1.",
  "attack": 8,
  "defense": 6,
  "abilities": [
    {
      "on": "spellboost",
      "zone": "hand",
      "printed": "On Spellboost: Reduce the cost of this card by 1.",
      "effects": [
        {
          "printed": "Reduce the cost of this card by 1.",
          "op": "cost",
          "select": {
            "pick": "self"
          },
          "delta": -1
        }
      ]
    }
  ]
}
```

### `10131320` — Stormy Blast

vars X=2; spellboost increments; var X amount.

Printed text:

```
X starts at 2.
On Spellboost: Increase X by 1.
Select an enemy follower on the field and deal it X damage.
```

File: `cards/10001/10131320.json`

```json
{
  "id": "10131320",
  "name": "Stormy Blast",
  "kind": "spell",
  "class": "runecraft",
  "set": 10001,
  "rarity": "bronze",
  "token": false,
  "cost": 1,
  "text": "X starts at 2.\nOn Spellboost: Increase X by 1.\nSelect an enemy follower on the field and deal it X damage.",
  "abilities": [
    {
      "on": "spellboost",
      "zone": "hand",
      "printed": "On Spellboost: Increase X by 1.",
      "effects": [
        {
          "printed": "Increase X by 1.",
          "op": "counter",
          "key": {
            "var": "X"
          },
          "how": "add",
          "amount": 1
        }
      ]
    },
    {
      "on": "fanfare",
      "printed": "Select an enemy follower on the field and deal it X damage.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field and deal it X damage.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "choose"
          },
          "amount": {
            "var": "X"
          }
        }
      ]
    }
  ],
  "vars": {
    "X": 2
  }
}
```

### `10031110` — Dazzling Runeknight

choose with spellboostHand and pay earth.

Printed text:

```
Fanfare: Select a Mode to activate.
1. Spellboost your hand 2 times.
2. Earth Rite (1) - Give this follower +2/+2 and Ward
```

File: `cards/10000/10031110.json`

```json
{
  "id": "10031110",
  "name": "Dazzling Runeknight",
  "kind": "follower",
  "class": "runecraft",
  "set": 10000,
  "rarity": "bronze",
  "token": false,
  "cost": 3,
  "text": "Fanfare: Select a Mode to activate.\n1. Spellboost your hand 2 times.\n2. Earth Rite (1) - Give this follower +2/+2 and Ward",
  "attack": 2,
  "defense": 2,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Select a Mode to activate.\n1. Spellboost your hand 2 times.\n2. Earth Rite (1) - Give this follower +2/+2 and Ward",
      "effects": [
        {
          "printed": "Select a Mode to activate.\n1. Spellboost your hand 2 times.\n2. Earth Rite (1) - Give this follower +2/+2 and Ward",
          "op": "choose",
          "pick": 1,
          "by": "player",
          "options": [
            {
              "printed": "1. Spellboost your hand 2 times.",
              "effects": [
                {
                  "op": "spellboostHand",
                  "times": 2
                }
              ]
            },
            {
              "printed": "2. Earth Rite (1) - Give this follower +2/+2 and Ward",
              "effects": [
                {
                  "op": "pay",
                  "resource": "earth",
                  "amount": 1,
                  "effects": [
                    {
                      "op": "buff",
                      "select": {
                        "pick": "self"
                      },
                      "attack": 2,
                      "defense": 2
                    },
                    {
                      "op": "grantTraits",
                      "select": {
                        "pick": "self"
                      },
                      "traits": {
                        "ward": true
                      }
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### `90031210` — Magic Sediment

Earth Sigil token; Engage gains a sigil.

Printed text:

```
Earth Sigil
Engage (1): Gain an earth sigil.
```

File: `cards/tokens/90031210.json`

```json
{
  "id": "90031210",
  "name": "Magic Sediment",
  "kind": "amulet",
  "class": "runecraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 1,
  "text": "Earth Sigil\nEngage (1): Gain an earth sigil.",
  "tribes": [
    "earth sigil"
  ],
  "abilities": [
    {
      "on": "engage",
      "cost": 1,
      "sacrifice": true,
      "printed": "Engage (1): Gain an earth sigil.",
      "effects": [
        {
          "printed": "Gain an earth sigil.",
          "op": "counter",
          "key": "earth",
          "how": "add",
          "amount": 1
        }
      ]
    }
  ]
}
```

### `10754120` — Macmillan, Reaper of Ceremonies

pay shadows 10; when ally_follower_enter + tribe departed.

Printed text:

```
Fanfare: Necromancy (10) - Summon 3 copies of Rotting Zombie.
During your turn, whenever an allied Departed follower enters the field, give it +1/+0, Rush, and Ward and deal 1 damage to the enemy leader.
```

File: `cards/10007/10754120.json`

```json
{
  "id": "10754120",
  "name": "Macmillan, Reaper of Ceremonies",
  "kind": "follower",
  "class": "abysscraft",
  "set": 10007,
  "rarity": "legendary",
  "token": false,
  "cost": 9,
  "text": "Fanfare: Necromancy (10) - Summon 3 copies of Rotting Zombie.\nDuring your turn, whenever an allied Departed follower enters the field, give it +1/+0, Rush, and Ward and deal 1 damage to the enemy leader.",
  "attack": 4,
  "defense": 4,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Necromancy (10) - Summon 3 copies of Rotting Zombie.",
      "effects": [
        {
          "printed": "Necromancy (10) - Summon 3 copies of Rotting Zombie.",
          "op": "pay",
          "resource": "shadows",
          "amount": 10,
          "effects": [
            {
              "op": "summon",
              "card": {
                "named": "90051140"
              },
              "count": 3
            }
          ]
        }
      ]
    },
    {
      "on": "when",
      "event": "ally_follower_enter",
      "filter": {
        "tribe": "departed"
      },
      "printed": "During your turn, whenever an allied Departed follower enters the field, give it +1/+0, Rush, and Ward and deal 1 damage to the enemy leader.",
      "when": {
        "turnOwner": "self"
      },
      "effects": [
        {
          "printed": "give it +1/+0, Rush, and Ward and deal 1 damage to the enemy leader.",
          "op": "seq",
          "effects": [
            {
              "op": "buff",
              "select": {
                "pick": "entering"
              },
              "attack": 1,
              "defense": 0
            },
            {
              "op": "grantTraits",
              "select": {
                "pick": "entering"
              },
              "traits": {
                "rush": true,
                "ward": true
              }
            },
            {
              "op": "damage",
              "select": {
                "pick": "all",
                "side": "enemy",
                "zone": "leader",
                "kind": "leader"
              },
              "amount": 1
            }
          ]
        }
      ]
    }
  ]
}
```

### `10554110` — Milteo & Luzen

two reanimate leaves; anyEvolve destroy 6 other random; anySuperEvolve crest.

Printed text:

```
Fanfare: Reanimate (4) and Reanimate (2).
When this follower evolves, destroy 6 other random followers.
When this follower super-evolves, gain Crest: Milteo & Luzen.
```

File: `cards/10005/10554110.json`

```json
{
  "id": "10554110",
  "name": "Milteo & Luzen",
  "kind": "follower",
  "class": "abysscraft",
  "set": 10005,
  "rarity": "legendary",
  "token": false,
  "cost": 7,
  "text": "Fanfare: Reanimate (4) and Reanimate (2).\nWhen this follower evolves, destroy 6 other random followers.\nWhen this follower super-evolves, gain Crest: Milteo & Luzen.",
  "attack": 3,
  "defense": 3,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Reanimate (4) and Reanimate (2).",
      "effects": [
        {
          "printed": "Reanimate (4) and Reanimate (2).",
          "op": "seq",
          "effects": [
            {
              "op": "reanimate",
              "maxCost": 4
            },
            {
              "op": "reanimate",
              "maxCost": 2
            }
          ]
        }
      ]
    },
    {
      "on": "anyEvolve",
      "printed": "When this follower evolves, destroy 6 other random followers.",
      "effects": [
        {
          "printed": "destroy 6 other random followers.",
          "op": "destroy",
          "select": {
            "side": "any",
            "zone": "field",
            "kind": "follower",
            "pick": "randomDistinct",
            "count": 6,
            "other": true
          }
        }
      ]
    },
    {
      "on": "anySuperEvolve",
      "printed": "When this follower super-evolves, gain Crest: Milteo & Luzen.",
      "effects": [
        {
          "printed": "gain Crest: Milteo & Luzen.",
          "op": "crest",
          "gain": "crest:10554110",
          "player": "self"
        }
      ]
    }
  ]
}
```

### `10012110` — May, Journey Elf

combo 3 (counts itself — Q&A).

Printed text:

```
Fanfare: Combo (3) - Select an enemy follower on the field and deal it 3 damage.
```

File: `cards/10000/10012110.json`

```json
{
  "id": "10012110",
  "name": "May, Journey Elf",
  "kind": "follower",
  "class": "forestcraft",
  "set": 10000,
  "rarity": "silver",
  "token": false,
  "cost": 1,
  "text": "Fanfare: Combo (3) - Select an enemy follower on the field and deal it 3 damage.",
  "attack": 1,
  "defense": 1,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Combo (3) - Select an enemy follower on the field and deal it 3 damage.",
      "effects": [
        {
          "printed": "Combo (3) - Select an enemy follower on the field and deal it 3 damage.",
          "op": "if",
          "cond": {
            "combo": {
              "n": 3
            }
          },
          "then": [
            {
              "op": "damage",
              "select": {
                "side": "enemy",
                "zone": "field",
                "kind": "follower",
                "pick": "choose"
              },
              "amount": 3
            }
          ]
        }
      ]
    }
  ]
}
```

### `10041310` — Strike of the Dragonewt

if overflow with instead as if/else.

Printed text:

```
Select an enemy follower on the field and deal it 2 damage. If you're in Overflow, deal 4 damage instead.
```

File: `cards/10000/10041310.json`

```json
{
  "id": "10041310",
  "name": "Strike of the Dragonewt",
  "kind": "spell",
  "class": "dragoncraft",
  "set": 10000,
  "rarity": "bronze",
  "token": false,
  "cost": 1,
  "text": "Select an enemy follower on the field and deal it 2 damage. If you're in Overflow, deal 4 damage instead.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select an enemy follower on the field and deal it 2 damage. If you're in Overflow, deal 4 damage instead.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field and deal it 2 damage. If you're in Overflow, deal 4 damage instead.",
          "op": "if",
          "cond": {
            "overflow": true
          },
          "then": [
            {
              "op": "damage",
              "select": {
                "side": "enemy",
                "zone": "field",
                "kind": "follower",
                "pick": "choose"
              },
              "amount": 4
            }
          ],
          "else": [
            {
              "op": "damage",
              "select": {
                "side": "enemy",
                "zone": "field",
                "kind": "follower",
                "pick": "choose"
              },
              "amount": 2
            }
          ]
        }
      ]
    }
  ]
}
```

### `10543310` — Sloth of the Crestpetal

repeat + independent if overflow sentence.

Printed text:

```
Do this 2 times: "Deal 2 damage to a random enemy follower." If you're in Overflow, deal 2 damage to the enemy leader.
```

File: `cards/10005/10543310.json`

```json
{
  "id": "10543310",
  "name": "Sloth of the Crestpetal",
  "kind": "spell",
  "class": "dragoncraft",
  "set": 10005,
  "rarity": "gold",
  "token": false,
  "cost": 2,
  "text": "Do this 2 times: \"Deal 2 damage to a random enemy follower.\" If you're in Overflow, deal 2 damage to the enemy leader.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Do this 2 times: \"Deal 2 damage to a random enemy follower.\" If you're in Overflow, deal 2 damage to the enemy leader.",
      "effects": [
        {
          "printed": "Do this 2 times: \"Deal 2 damage to a random enemy follower.\"",
          "op": "repeat",
          "times": 2,
          "effects": [
            {
              "op": "damage",
              "select": {
                "side": "enemy",
                "zone": "field",
                "kind": "follower",
                "pick": "random"
              },
              "amount": 2
            }
          ]
        },
        {
          "printed": "If you're in Overflow, deal 2 damage to the enemy leader.",
          "op": "if",
          "cond": {
            "overflow": true
          },
          "then": [
            {
              "op": "damage",
              "select": {
                "pick": "all",
                "side": "enemy",
                "zone": "leader",
                "kind": "leader"
              },
              "amount": 2
            }
          ]
        }
      ]
    }
  ]
}
```

### `10403110` — Gran & Djeeta, Valiant Skyfarers

Skybound Art + choose.

Printed text:

```
Fanfare: Select a Mode to activate.
Skybound Art - Evolve this follower.
1. Deal 5 damage to a random enemy follower.
2. Draw 2 followers.
```

File: `cards/10004/10403110.json`

```json
{
  "id": "10403110",
  "name": "Gran & Djeeta, Valiant Skyfarers",
  "kind": "follower",
  "class": "neutral",
  "set": 10004,
  "rarity": "gold",
  "token": false,
  "cost": 4,
  "text": "Fanfare: Select a Mode to activate.\nSkybound Art - Evolve this follower.\n1. Deal 5 damage to a random enemy follower.\n2. Draw 2 followers.",
  "attack": 3,
  "defense": 2,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Select a Mode to activate.\nSkybound Art - Evolve this follower.\n1. Deal 5 damage to a random enemy follower.\n2. Draw 2 followers.",
      "effects": [
        {
          "printed": "Select a Mode to activate.",
          "op": "choose",
          "pick": 1,
          "by": "player",
          "options": [
            {
              "printed": "1. Deal 5 damage to a random enemy follower.",
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "side": "enemy",
                    "zone": "field",
                    "kind": "follower",
                    "pick": "random"
                  },
                  "amount": 5
                }
              ]
            },
            {
              "printed": "2. Draw 2 followers.",
              "effects": [
                {
                  "op": "draw",
                  "count": 2,
                  "filter": {
                    "kind": "follower"
                  }
                }
              ]
            }
          ]
        },
        {
          "printed": "Skybound Art - Evolve this follower.",
          "op": "if",
          "cond": {
            "skyboundArt": {
              "n": 10
            }
          },
          "then": [
            {
              "op": "evolve",
              "select": {
                "pick": "self"
              },
              "super": false
            }
          ]
        }
      ]
    }
  ]
}
```

### `10471120` — Tsubasa, Blazing Gearcyclist

counter skyboundHand.

Printed text:

```
Fanfare: Increase the Skybound Art gauges of all cards in your hand by 1.
Rush
```

File: `cards/10004/10471120.json`

```json
{
  "id": "10471120",
  "name": "Tsubasa, Blazing Gearcyclist",
  "kind": "follower",
  "class": "portalcraft",
  "set": 10004,
  "rarity": "bronze",
  "token": false,
  "cost": 2,
  "text": "Fanfare: Increase the Skybound Art gauges of all cards in your hand by 1.\nRush",
  "attack": 3,
  "defense": 1,
  "traits": {
    "rush": true
  },
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Increase the Skybound Art gauges of all cards in your hand by 1.",
      "effects": [
        {
          "printed": "Increase the Skybound Art gauges of all cards in your hand by 1.",
          "op": "counter",
          "key": "skyboundHand",
          "how": "add",
          "amount": 1
        }
      ]
    }
  ]
}
```

### `10944110` — Antemaria, Piercing Convict

attackedLeaderLastTurn; ignoresWard.

Printed text:

```
Fanfare: If an allied follower attacked a leader on your last turn, give this follower Storm.
Rush
Ignores Ward.
```

File: `cards/10009/10944110.json`

```json
{
  "id": "10944110",
  "name": "Antemaria, Piercing Convict",
  "kind": "follower",
  "class": "dragoncraft",
  "set": 10009,
  "rarity": "legendary",
  "token": false,
  "cost": 7,
  "text": "Fanfare: If an allied follower attacked a leader on your last turn, give this follower Storm.\nRush\nIgnores Ward.",
  "attack": 6,
  "defense": 5,
  "traits": {
    "rush": true,
    "ignoresWard": true
  },
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: If an allied follower attacked a leader on your last turn, give this follower Storm.",
      "effects": [
        {
          "printed": "If an allied follower attacked a leader on your last turn, give this follower Storm.",
          "op": "if",
          "cond": {
            "attackedLeaderLastTurn": true
          },
          "then": [
            {
              "op": "grantTraits",
              "select": {
                "pick": "self"
              },
              "traits": {
                "storm": true
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### `10021310` — Way of the Maid

independent sentences (empty hand still draws); draw with filter.

Printed text:

```
Select a card in your hand and return it to deck. Draw 2 Swordcraft followers.
```

File: `cards/10000/10021310.json`

```json
{
  "id": "10021310",
  "name": "Way of the Maid",
  "kind": "spell",
  "class": "swordcraft",
  "set": 10000,
  "rarity": "bronze",
  "token": false,
  "cost": 2,
  "text": "Select a card in your hand and return it to deck. Draw 2 Swordcraft followers.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select a card in your hand and return it to deck. Draw 2 Swordcraft followers.",
      "effects": [
        {
          "printed": "Select a card in your hand and return it to deck.",
          "op": "returnToDeck",
          "select": {
            "side": "ally",
            "zone": "hand",
            "kind": "card",
            "pick": "choose"
          },
          "position": "random"
        },
        {
          "printed": "Draw 2 Swordcraft followers.",
          "op": "draw",
          "count": 2,
          "filter": {
            "kind": "follower",
            "class": "swordcraft"
          }
        }
      ]
    }
  ]
}
```

### `10554120` — Shakdoh, Nightblossom

repeat around seq with count amount and Then-if.

Printed text:

```
Fanfare: Do this 2 times: "Return your hand to deck. Draw X cards. X is the number of cards you returned. Then, if you have at least 4 cards with the same cost in your hand, deal 4 damage to all enemies."
Super-Evolve: Replicate the effects of this card's Fanfare ability.
```

File: `cards/10005/10554120.json`

```json
{
  "id": "10554120",
  "name": "Shakdoh, Nightblossom",
  "kind": "follower",
  "class": "abysscraft",
  "set": 10005,
  "rarity": "legendary",
  "token": false,
  "cost": 10,
  "text": "Fanfare: Do this 2 times: \"Return your hand to deck. Draw X cards. X is the number of cards you returned. Then, if you have at least 4 cards with the same cost in your hand, deal 4 damage to all enemies.\"\nSuper-Evolve: Replicate the effects of this card's Fanfare ability.",
  "attack": 4,
  "defense": 4,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Do this 2 times: \"Return your hand to deck. Draw X cards. X is the number of cards you returned. Then, if you have at least 4 cards with the same cost in your hand, deal 4 damage to all enemies.\"",
      "effects": [
        {
          "printed": "Do this 2 times: \"Return your hand to deck. Draw X cards. X is the number of cards you returned. Then, if you have at least 4 cards with the same cost in your hand, deal 4 damage to all enemies.\"",
          "op": "repeat",
          "times": 2,
          "effects": [
            {
              "op": "seq",
              "as": "r",
              "effects": [
                {
                  "op": "returnToDeck",
                  "select": {
                    "side": "ally",
                    "zone": "hand",
                    "kind": "card",
                    "pick": "all"
                  },
                  "as": "r",
                  "position": "random"
                },
                {
                  "op": "draw",
                  "count": {
                    "count": {
                      "pick": "bound",
                      "ref": "r"
                    }
                  }
                },
                {
                  "op": "if",
                  "cond": {
                    "handSameCostAtLeast": {
                      "n": 4
                    }
                  },
                  "then": [
                    {
                      "op": "damage",
                      "select": {
                        "side": "enemy",
                        "kind": "character",
                        "pick": "all",
                        "includeLeader": true,
                        "zone": "field"
                      },
                      "amount": 4
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    },
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: Replicate the effects of this card's Fanfare ability.",
      "effects": [
        {
          "printed": "Replicate the effects of this card's Fanfare ability.",
          "op": "replicate",
          "ability": "fanfare"
        }
      ]
    }
  ]
}
```

### `10443310` — Primal Beast Absorption

banish + addToHand copyOf bound.

Printed text:

```
Select an enemy card on the field, banish it, and add a copy of it to your hand.
```

File: `cards/10004/10443310.json`

```json
{
  "id": "10443310",
  "name": "Primal Beast Absorption",
  "kind": "spell",
  "class": "dragoncraft",
  "set": 10004,
  "rarity": "gold",
  "token": false,
  "cost": 5,
  "text": "Select an enemy card on the field, banish it, and add a copy of it to your hand.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select an enemy card on the field, banish it, and add a copy of it to your hand.",
      "effects": [
        {
          "printed": "Select an enemy card on the field, banish it, and add a copy of it to your hand.",
          "op": "seq",
          "effects": [
            {
              "op": "banish",
              "select": {
                "side": "enemy",
                "zone": "field",
                "kind": "card",
                "pick": "choose"
              },
              "as": "t"
            },
            {
              "op": "addToHand",
              "card": {
                "copyOf": {
                  "pick": "bound",
                  "ref": "t"
                },
                "exact": true
              },
              "count": 1
            }
          ]
        }
      ]
    }
  ]
}
```

### `10444120` — Zooey, Ally of the World

leaderModifier max defense + damage cap with duration.

Printed text:

```
Fanfare: Gain 1 max play point.
Enhance (10): Give this follower Storm. Set your leader's max defense to 1. Give your leader "Can't take more than 0 damage at a time" until the end of your opponent's turn.
```

File: `cards/10004/10444120.json`

```json
{
  "id": "10444120",
  "name": "Zooey, Ally of the World",
  "kind": "follower",
  "class": "dragoncraft",
  "set": 10004,
  "rarity": "legendary",
  "token": false,
  "cost": 5,
  "text": "Fanfare: Gain 1 max play point.\nEnhance (10): Give this follower Storm. Set your leader's max defense to 1. Give your leader \"Can't take more than 0 damage at a time\" until the end of your opponent's turn.",
  "attack": 5,
  "defense": 5,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Gain 1 max play point.",
      "effects": [
        {
          "printed": "Gain 1 max play point.",
          "op": "pp",
          "action": "gainMax",
          "amount": 1
        }
      ]
    }
  ],
  "modes": [
    {
      "kind": "enhance",
      "cost": 10,
      "printed": "Enhance (10): Give this follower Storm. Set your leader's max defense to 1. Give your leader \"Can't take more than 0 damage at a time\" until the end of your opponent's turn.",
      "effects": [
        {
          "printed": "Give this follower Storm.",
          "op": "grantTraits",
          "select": {
            "pick": "self"
          },
          "traits": {
            "storm": true
          }
        },
        {
          "printed": "Set your leader's max defense to 1.",
          "op": "leaderModifier",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "maxDefense": {
            "set": 1
          }
        },
        {
          "printed": "Give your leader \"Can't take more than 0 damage at a time\" until the end of your opponent's turn.",
          "op": "leaderModifier",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "damageCap": 0,
          "until": "endOfOpponentTurn"
        }
      ]
    }
  ]
}
```

### `10071310` — Bullet from Beyond

addToHand of two named tokens.

Printed text:

```
Select an enemy follower on the field and destroy it. Add a Gear of Ambition and Gear of Remembrance to your hand.
```

File: `cards/10000/10071310.json`

```json
{
  "id": "10071310",
  "name": "Bullet from Beyond",
  "kind": "spell",
  "class": "portalcraft",
  "set": 10000,
  "rarity": "bronze",
  "token": false,
  "cost": 4,
  "text": "Select an enemy follower on the field and destroy it. Add a Gear of Ambition and Gear of Remembrance to your hand.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select an enemy follower on the field and destroy it. Add a Gear of Ambition and Gear of Remembrance to your hand.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field and destroy it.",
          "op": "destroy",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "choose"
          }
        },
        {
          "printed": "Add a Gear of Ambition and Gear of Remembrance to your hand.",
          "op": "seq",
          "effects": [
            {
              "op": "addToHand",
              "card": {
                "named": "90071210"
              },
              "count": 1
            },
            {
              "op": "addToHand",
              "card": {
                "named": "90071220"
              },
              "count": 1
            }
          ]
        }
      ]
    }
  ]
}
```

### `90071210` — Gear of Ambition

cantBePlayed; fuse recipe transform.

Printed text:

```
Fuse: Artifact amulets
When you Fuse to this card, transform it into a Striker Artifact.
Can't be played.
```

File: `cards/tokens/90071210.json`

```json
{
  "id": "90071210",
  "name": "Gear of Ambition",
  "kind": "amulet",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 1,
  "text": "Fuse: Artifact amulets\nWhen you Fuse to this card, transform it into a Striker Artifact.\nCan't be played.",
  "tribes": [
    "artifact"
  ],
  "traits": {
    "cantBePlayed": true
  },
  "fuse": {
    "printed": "Fuse: Artifact amulets",
    "partners": {
      "tribe": "artifact",
      "kind": "amulet"
    },
    "recipes": [
      {
        "result": {
          "transformInto": "90072110"
        }
      }
    ]
  }
}
```

### `90072110` — Striker Artifact

fuse partners Artifact cards; cost-total recipes.

Printed text:

```
Fuse: Artifact cards
When you Fuse to this card, transform it based on the total cost of the cards fused.
1: Ominous Artifact α
2: Ominous Artifact β
3 or more: Ominous Artifact γ
Rush
```

File: `cards/tokens/90072110.json`

```json
{
  "id": "90072110",
  "name": "Striker Artifact",
  "kind": "follower",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "silver",
  "token": true,
  "cost": 3,
  "text": "Fuse: Artifact cards\nWhen you Fuse to this card, transform it based on the total cost of the cards fused.\n1: Ominous Artifact α\n2: Ominous Artifact β\n3 or more: Ominous Artifact γ\nRush",
  "attack": 5,
  "defense": 1,
  "tribes": [
    "artifact"
  ],
  "traits": {
    "rush": true
  },
  "fuse": {
    "partners": {
      "tribe": "artifact"
    },
    "recipes": [
      {
        "costTotal": 1,
        "result": {
          "transformInto": "90073110"
        }
      },
      {
        "costTotal": 2,
        "result": {
          "transformInto": "90073120"
        }
      },
      {
        "costTotalGte": 3,
        "result": {
          "transformInto": "90073130"
        }
      }
    ],
    "printed": "Fuse: Artifact cards"
  }
}
```

### `90073110` — Ominous Artifact α

fuse with memory; when wasFused both.

Printed text:

```
Fuse: Ominous Artifact β and Ominous Artifact γ
When you've Fused both to this card, transform it into a Masterwork Artifact Ω.
At the end of your turn, restore 3 defense to your leader.
```

File: `cards/tokens/90073110.json`

```json
{
  "id": "90073110",
  "name": "Ominous Artifact α",
  "kind": "follower",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "gold",
  "token": true,
  "cost": 5,
  "text": "Fuse: Ominous Artifact β and Ominous Artifact γ\nWhen you've Fused both to this card, transform it into a Masterwork Artifact Ω.\nAt the end of your turn, restore 3 defense to your leader.",
  "attack": 3,
  "defense": 5,
  "tribes": [
    "artifact"
  ],
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, restore 3 defense to your leader.",
      "effects": [
        {
          "printed": "restore 3 defense to your leader.",
          "op": "restore",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 3
        }
      ]
    }
  ],
  "fuse": {
    "printed": "Fuse: Ominous Artifact β and Ominous Artifact γ",
    "partners": {
      "cards": [
        "90073120",
        "90073130"
      ]
    },
    "recipes": [
      {
        "requires": [
          "90073120",
          "90073130"
        ],
        "result": {
          "transformInto": "90074110"
        }
      }
    ]
  }
}
```

### `10812110` — Ruflet, Primeval Fairy

oncePerTurn when self_buffed_up; Last Words addToHand.

Printed text:

```
Once on each of your turns, when this follower is given + attack or defense on the field, summon a Fairy.
Last Words: Add a Fairy to your hand.
```

File: `cards/10008/10812110.json`

```json
{
  "id": "10812110",
  "name": "Ruflet, Primeval Fairy",
  "kind": "follower",
  "class": "forestcraft",
  "set": 10008,
  "rarity": "silver",
  "token": false,
  "cost": 2,
  "text": "Once on each of your turns, when this follower is given + attack or defense on the field, summon a Fairy.\nLast Words: Add a Fairy to your hand.",
  "attack": 2,
  "defense": 2,
  "abilities": [
    {
      "on": "when",
      "event": "self_buffed_up",
      "oncePerTurn": true,
      "printed": "Once on each of your turns, when this follower is given + attack or defense on the field, summon a Fairy.",
      "effects": [
        {
          "printed": "summon a Fairy.",
          "op": "summon",
          "card": {
            "named": "90011110"
          },
          "count": 1
        }
      ]
    },
    {
      "on": "lastWords",
      "printed": "Last Words: Add a Fairy to your hand.",
      "effects": [
        {
          "printed": "Add a Fairy to your hand.",
          "op": "addToHand",
          "card": {
            "named": "90011110"
          },
          "count": 1
        }
      ]
    }
  ]
}
```

### `10822110` — Katze, Magical Thief

oncePerTurn when ally_spell_played.

Printed text:

```
Once on each of your turns, when you play a spell, deal 2 damage to a random enemy follower.
Evolve: Add a Glittering Gold to your hand.
```

File: `cards/10008/10822110.json`

```json
{
  "id": "10822110",
  "name": "Katze, Magical Thief",
  "kind": "follower",
  "class": "swordcraft",
  "set": 10008,
  "rarity": "silver",
  "token": false,
  "cost": 3,
  "text": "Once on each of your turns, when you play a spell, deal 2 damage to a random enemy follower.\nEvolve: Add a Glittering Gold to your hand.",
  "attack": 1,
  "defense": 4,
  "abilities": [
    {
      "on": "when",
      "event": "ally_spell_played",
      "oncePerTurn": true,
      "printed": "Once on each of your turns, when you play a spell, deal 2 damage to a random enemy follower.",
      "effects": [
        {
          "printed": "deal 2 damage to a random enemy follower.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "random"
          },
          "amount": 2
        }
      ]
    },
    {
      "on": "evolve",
      "printed": "Evolve: Add a Glittering Gold to your hand.",
      "effects": [
        {
          "printed": "Add a Glittering Gold to your hand.",
          "op": "addToHand",
          "card": {
            "named": "90021350"
          },
          "count": 1
        }
      ]
    }
  ]
}
```

### `10464120` — Vira, Luminous Primal Knight

damageCap 3; Super Skybound super-evolve self.

Printed text:

```
Fanfare: Select 2 enemy followers on the field and banish them.
Super Skybound Art - Super-evolve this follower.
Ward
Can't take more than 3 damage at a time.
```

File: `cards/10004/10464120.json`

```json
{
  "id": "10464120",
  "name": "Vira, Luminous Primal Knight",
  "kind": "follower",
  "class": "havencraft",
  "set": 10004,
  "rarity": "legendary",
  "token": false,
  "cost": 8,
  "text": "Fanfare: Select 2 enemy followers on the field and banish them.\nSuper Skybound Art - Super-evolve this follower.\nWard\nCan't take more than 3 damage at a time.",
  "attack": 6,
  "defense": 8,
  "traits": {
    "ward": true,
    "damageCap": 3
  },
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Select 2 enemy followers on the field and banish them.\nSuper Skybound Art - Super-evolve this follower.",
      "effects": [
        {
          "printed": "Select 2 enemy followers on the field and banish them.",
          "op": "banish",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "choose",
            "count": 2
          }
        },
        {
          "printed": "Super Skybound Art - Super-evolve this follower.",
          "op": "if",
          "cond": {
            "skyboundArt": {
              "n": 15
            }
          },
          "then": [
            {
              "op": "evolve",
              "select": {
                "pick": "self"
              },
              "super": true
            }
          ]
        }
      ]
    }
  ]
}
```

### `10561120` — Bouquet Believer

Enhance two sentences; when ally_draw + turnOwner.

Printed text:

```
Enhance (4): Draw a card. Give this follower Bane.
During your turn, whenever you draw a card, give this follower Rush.
```

File: `cards/10005/10561120.json`

```json
{
  "id": "10561120",
  "name": "Bouquet Believer",
  "kind": "follower",
  "class": "havencraft",
  "set": 10005,
  "rarity": "bronze",
  "token": false,
  "cost": 1,
  "text": "Enhance (4): Draw a card. Give this follower Bane.\nDuring your turn, whenever you draw a card, give this follower Rush.",
  "attack": 1,
  "defense": 1,
  "abilities": [
    {
      "on": "when",
      "event": "ally_draw",
      "printed": "During your turn, whenever you draw a card, give this follower Rush.",
      "when": {
        "turnOwner": "self"
      },
      "effects": [
        {
          "printed": "give this follower Rush.",
          "op": "grantTraits",
          "select": {
            "pick": "self"
          },
          "traits": {
            "rush": true
          }
        }
      ]
    }
  ],
  "modes": [
    {
      "kind": "enhance",
      "cost": 4,
      "printed": "Enhance (4): Draw a card. Give this follower Bane.",
      "effects": [
        {
          "printed": "Draw a card.",
          "op": "draw",
          "count": 1
        },
        {
          "printed": "Give this follower Bane.",
          "op": "grantTraits",
          "select": {
            "pick": "self"
          },
          "traits": {
            "bane": true
          }
        }
      ]
    }
  ]
}
```

### `10901310` — Initiation of Rebirth

addToDeck copyOf highest-baseCost destroyed, random tie, position random; independent draw.

Printed text:

```
Add a copy of a random allied follower destroyed this match with the highest base cost to your deck without revealing it. Draw a card.
```

File: `cards/10009/10901310.json`

```json
{
  "id": "10901310",
  "name": "Initiation of Rebirth",
  "kind": "spell",
  "class": "neutral",
  "set": 10009,
  "rarity": "bronze",
  "token": false,
  "cost": 2,
  "text": "Add a copy of a random allied follower destroyed this match with the highest base cost to your deck without revealing it. Draw a card.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Add a copy of a random allied follower destroyed this match with the highest base cost to your deck without revealing it. Draw a card.",
      "effects": [
        {
          "printed": "Add a copy of a random allied follower destroyed this match with the highest base cost to your deck without revealing it.",
          "op": "addToDeck",
          "card": {
            "copyOf": {
              "side": "ally",
              "kind": "follower",
              "pick": "highest",
              "orderBy": "baseCost",
              "filter": {
                "destroyedThisMatch": true
              },
              "zone": "field"
            },
            "exact": true
          },
          "count": 1,
          "position": "random"
        },
        {
          "printed": "Draw a card.",
          "op": "draw",
          "count": 1
        }
      ]
    }
  ]
}
```

## Supporting tokens, collectibles, and crests

These exist so every `crest.gain` and `{named}` card-source in the examples resolves to a file.

### `10061120` — Fox of Purity

follower. File: `cards/10000/10061120.json`

```
Ward
```

```json
{
  "id": "10061120",
  "name": "Fox of Purity",
  "kind": "follower",
  "class": "havencraft",
  "set": 10000,
  "rarity": "bronze",
  "token": false,
  "cost": 2,
  "text": "Ward",
  "attack": 1,
  "defense": 3,
  "traits": {
    "ward": true
  }
}
```

### `10621110` — Fearless Soldier

follower. File: `cards/10006/10621110.json`

```
Enhance (3): Give this follower +1/+1.
Rush
```

```json
{
  "id": "10621110",
  "name": "Fearless Soldier",
  "kind": "follower",
  "class": "swordcraft",
  "set": 10006,
  "rarity": "bronze",
  "token": false,
  "cost": 2,
  "text": "Enhance (3): Give this follower +1/+1.\nRush",
  "attack": 2,
  "defense": 2,
  "traits": {
    "rush": true
  },
  "modes": [
    {
      "kind": "enhance",
      "cost": 3,
      "printed": "Enhance (3): Give this follower +1/+1.",
      "effects": [
        {
          "printed": "Give this follower +1/+1.",
          "op": "buff",
          "select": {
            "pick": "self"
          },
          "attack": 1,
          "defense": 1
        }
      ]
    }
  ]
}
```

### `10631110` — Crystalspawn

follower. File: `cards/10006/10631110.json`

```
Rush
```

```json
{
  "id": "10631110",
  "name": "Crystalspawn",
  "kind": "follower",
  "class": "runecraft",
  "set": 10006,
  "rarity": "bronze",
  "token": false,
  "cost": 1,
  "text": "Rush",
  "attack": 1,
  "defense": 1,
  "tribes": [
    "encroacher"
  ],
  "traits": {
    "rush": true
  }
}
```

### `10931110` — Obsessed Test Subject

follower. File: `cards/10009/10931110.json`

```
When this follower enters the field, if at least 5 other allied copies of Obsessed Test Subject have entered the field this match, give it +3/+3.
Rush
```

```json
{
  "id": "10931110",
  "name": "Obsessed Test Subject",
  "kind": "follower",
  "class": "runecraft",
  "set": 10009,
  "rarity": "bronze",
  "token": false,
  "cost": 2,
  "text": "When this follower enters the field, if at least 5 other allied copies of Obsessed Test Subject have entered the field this match, give it +3/+3.\nRush",
  "attack": 2,
  "defense": 2,
  "traits": {
    "rush": true
  },
  "abilities": [
    {
      "on": "enter",
      "printed": "When this follower enters the field, if at least 5 other allied copies of Obsessed Test Subject have entered the field this match, give it +3/+3.",
      "when": {
        "enterCountAtLeast": {
          "n": 5,
          "other": true,
          "side": "ally",
          "card": "10931110"
        }
      },
      "effects": [
        {
          "printed": "give it +3/+3.",
          "op": "buff",
          "select": {
            "pick": "self"
          },
          "attack": 3,
          "defense": 3
        }
      ]
    }
  ]
}
```

### `90011110` — Fairy

follower. File: `cards/tokens/90011110.json`

```
Rush
```

```json
{
  "id": "90011110",
  "name": "Fairy",
  "kind": "follower",
  "class": "forestcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 1,
  "text": "Rush",
  "attack": 1,
  "defense": 1,
  "tribes": [
    "pixie"
  ],
  "traits": {
    "rush": true
  }
}
```

### `90021120` — Steelclad Knight

follower. File: `cards/tokens/90021120.json`

```json
{
  "id": "90021120",
  "name": "Steelclad Knight",
  "kind": "follower",
  "class": "swordcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 1,
  "text": "",
  "attack": 2,
  "defense": 2,
  "tribes": [
    "officer"
  ]
}
```

### `90021210` — Dread Pirate's Flag

amulet. File: `cards/tokens/90021210.json`

```
Countdown (7)
Whenever you play a spell, advance this amulet's count by 1.
Last Words: Deal 2 damage to the enemy leader.
```

```json
{
  "id": "90021210",
  "name": "Dread Pirate's Flag",
  "kind": "amulet",
  "class": "swordcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 1,
  "text": "Countdown (7)\nWhenever you play a spell, advance this amulet's count by 1.\nLast Words: Deal 2 damage to the enemy leader.",
  "countdown": 7,
  "abilities": [
    {
      "on": "when",
      "event": "ally_spell_played",
      "printed": "Whenever you play a spell, advance this amulet's count by 1.",
      "effects": [
        {
          "printed": "advance this amulet's count by 1.",
          "op": "countdown",
          "select": {
            "pick": "self"
          },
          "delta": -1
        }
      ]
    },
    {
      "on": "lastWords",
      "printed": "Last Words: Deal 2 damage to the enemy leader.",
      "effects": [
        {
          "printed": "Deal 2 damage to the enemy leader.",
          "op": "damage",
          "select": {
            "pick": "all",
            "side": "enemy",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 2
        }
      ]
    }
  ]
}
```

### `90021310` — Gilded Blade

spell. File: `cards/tokens/90021310.json`

```
Select an enemy follower on the field or the enemy leader and deal it 1 damage.
```

```json
{
  "id": "90021310",
  "name": "Gilded Blade",
  "kind": "spell",
  "class": "swordcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 1,
  "text": "Select an enemy follower on the field or the enemy leader and deal it 1 damage.",
  "tribes": [
    "loot"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select an enemy follower on the field or the enemy leader and deal it 1 damage.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field or the enemy leader and deal it 1 damage.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "character",
            "pick": "choose",
            "includeLeader": true
          },
          "amount": 1
        }
      ]
    }
  ]
}
```

### `90021340` — Gilded Necklace

spell. File: `cards/tokens/90021340.json`

```
Select an allied follower on the field and give it +0/+1 and Ward.
```

```json
{
  "id": "90021340",
  "name": "Gilded Necklace",
  "kind": "spell",
  "class": "swordcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 1,
  "text": "Select an allied follower on the field and give it +0/+1 and Ward.",
  "tribes": [
    "loot"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select an allied follower on the field and give it +0/+1 and Ward.",
      "effects": [
        {
          "printed": "Select an allied follower on the field and give it +0/+1 and Ward.",
          "op": "seq",
          "effects": [
            {
              "op": "buff",
              "select": {
                "side": "ally",
                "zone": "field",
                "kind": "follower",
                "pick": "choose"
              },
              "as": "t",
              "attack": 0,
              "defense": 1
            },
            {
              "op": "grantTraits",
              "select": {
                "pick": "bound",
                "ref": "t"
              },
              "traits": {
                "ward": true
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### `90021350` — Glittering Gold

spell. File: `cards/tokens/90021350.json`

```
Select a Mode to activate.
1. Draw a card.
2. Deal 2 damage to a random enemy follower.
```

```json
{
  "id": "90021350",
  "name": "Glittering Gold",
  "kind": "spell",
  "class": "swordcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 0,
  "text": "Select a Mode to activate.\n1. Draw a card.\n2. Deal 2 damage to a random enemy follower.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select a Mode to activate.\n1. Draw a card.\n2. Deal 2 damage to a random enemy follower.",
      "effects": [
        {
          "printed": "Select a Mode to activate.\n1. Draw a card.\n2. Deal 2 damage to a random enemy follower.",
          "op": "choose",
          "pick": 1,
          "by": "player",
          "options": [
            {
              "printed": "1. Draw a card.",
              "effects": [
                {
                  "op": "draw",
                  "count": 1
                }
              ]
            },
            {
              "printed": "2. Deal 2 damage to a random enemy follower.",
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "side": "enemy",
                    "zone": "field",
                    "kind": "follower",
                    "pick": "random"
                  },
                  "amount": 2
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### `90034320` — Ars Magna

spell. File: `cards/tokens/90034320.json`

```
Select an enemy and deal it 2 damage. Restore 1 defense to your leader.
```

```json
{
  "id": "90034320",
  "name": "Ars Magna",
  "kind": "spell",
  "class": "runecraft",
  "set": 90000,
  "rarity": "legendary",
  "token": true,
  "cost": 1,
  "text": "Select an enemy and deal it 2 damage. Restore 1 defense to your leader.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select an enemy and deal it 2 damage. Restore 1 defense to your leader.",
      "effects": [
        {
          "printed": "Select an enemy and deal it 2 damage.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "kind": "character",
            "pick": "choose",
            "includeLeader": true,
            "zone": "field"
          },
          "amount": 2
        },
        {
          "printed": "Restore 1 defense to your leader.",
          "op": "restore",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 1
        }
      ]
    }
  ]
}
```

### `90061110` — Holy Falcon

follower. File: `cards/tokens/90061110.json`

```
Storm
```

```json
{
  "id": "90061110",
  "name": "Holy Falcon",
  "kind": "follower",
  "class": "havencraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 3,
  "text": "Storm",
  "attack": 2,
  "defense": 2,
  "traits": {
    "storm": true
  }
}
```

### `90061130` — Regal Falcon

follower. File: `cards/tokens/90061130.json`

```
Storm
```

```json
{
  "id": "90061130",
  "name": "Regal Falcon",
  "kind": "follower",
  "class": "havencraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 6,
  "text": "Storm",
  "attack": 4,
  "defense": 4,
  "traits": {
    "storm": true
  }
}
```

### `90071110` — Puppet

follower. File: `cards/tokens/90071110.json`

```
Rush
At the end of your opponent's turn, destroy this card.
```

```json
{
  "id": "90071110",
  "name": "Puppet",
  "kind": "follower",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 0,
  "text": "Rush\nAt the end of your opponent's turn, destroy this card.",
  "attack": 1,
  "defense": 1,
  "tribes": [
    "puppetry"
  ],
  "traits": {
    "rush": true
  },
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "opponent",
      "printed": "At the end of your opponent's turn, destroy this card.",
      "effects": [
        {
          "printed": "At the end of your opponent's turn, destroy this card.",
          "op": "destroy",
          "select": {
            "pick": "self"
          }
        }
      ]
    }
  ]
}
```

### `90071220` — Gear of Remembrance

amulet. File: `cards/tokens/90071220.json`

```
Fuse: Artifact amulets
When you Fuse to this card, transform it into a Fortifier Artifact.
Can't be played.
```

```json
{
  "id": "90071220",
  "name": "Gear of Remembrance",
  "kind": "amulet",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "bronze",
  "token": true,
  "cost": 1,
  "text": "Fuse: Artifact amulets\nWhen you Fuse to this card, transform it into a Fortifier Artifact.\nCan't be played.",
  "tribes": [
    "artifact"
  ],
  "traits": {
    "cantBePlayed": true
  },
  "fuse": {
    "printed": "Fuse: Artifact amulets",
    "partners": {
      "tribe": "artifact",
      "kind": "amulet"
    },
    "recipes": [
      {
        "result": {
          "transformInto": "90072120"
        }
      }
    ]
  }
}
```

### `90072120` — Fortifier Artifact

follower. File: `cards/tokens/90072120.json`

```
Fuse: Artifact cards
When you Fuse to this card, transform it based on the total cost of the cards fused.
1: Ominous Artifact α
2: Ominous Artifact β
3 or more: Ominous Artifact γ
Ward
```

```json
{
  "id": "90072120",
  "name": "Fortifier Artifact",
  "kind": "follower",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "silver",
  "token": true,
  "cost": 3,
  "text": "Fuse: Artifact cards\nWhen you Fuse to this card, transform it based on the total cost of the cards fused.\n1: Ominous Artifact α\n2: Ominous Artifact β\n3 or more: Ominous Artifact γ\nWard",
  "attack": 1,
  "defense": 5,
  "tribes": [
    "artifact"
  ],
  "traits": {
    "ward": true
  },
  "fuse": {
    "partners": {
      "tribe": "artifact"
    },
    "recipes": [
      {
        "costTotal": 1,
        "result": {
          "transformInto": "90073110"
        }
      },
      {
        "costTotal": 2,
        "result": {
          "transformInto": "90073120"
        }
      },
      {
        "costTotalGte": 3,
        "result": {
          "transformInto": "90073130"
        }
      }
    ],
    "printed": "Fuse: Artifact cards"
  }
}
```

### `90073120` — Ominous Artifact β

follower. File: `cards/tokens/90073120.json`

```
At the end of your turn, deal 3 damage to the enemy leader.
```

```json
{
  "id": "90073120",
  "name": "Ominous Artifact β",
  "kind": "follower",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "gold",
  "token": true,
  "cost": 5,
  "text": "At the end of your turn, deal 3 damage to the enemy leader.",
  "attack": 4,
  "defense": 4,
  "tribes": [
    "artifact"
  ],
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, deal 3 damage to the enemy leader.",
      "effects": [
        {
          "printed": "deal 3 damage to the enemy leader.",
          "op": "damage",
          "select": {
            "pick": "all",
            "side": "enemy",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 3
        }
      ]
    }
  ]
}
```

### `90073130` — Ominous Artifact γ

follower. File: `cards/tokens/90073130.json`

```
At the end of your turn, deal 3 damage to all enemy followers.
```

```json
{
  "id": "90073130",
  "name": "Ominous Artifact γ",
  "kind": "follower",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "gold",
  "token": true,
  "cost": 5,
  "text": "At the end of your turn, deal 3 damage to all enemy followers.",
  "attack": 5,
  "defense": 3,
  "tribes": [
    "artifact"
  ],
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, deal 3 damage to all enemy followers.",
      "effects": [
        {
          "printed": "deal 3 damage to all enemy followers.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "all"
          },
          "amount": 3
        }
      ]
    }
  ]
}
```

### `90074110` — Masterwork Artifact Ω

follower. File: `cards/tokens/90074110.json`

```
Fanfare: Deal 5 damage to all enemy followers. Restore 5 defense to your leader.
Storm
Ward
Aura
```

```json
{
  "id": "90074110",
  "name": "Masterwork Artifact Ω",
  "kind": "follower",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "legendary",
  "token": true,
  "cost": 10,
  "text": "Fanfare: Deal 5 damage to all enemy followers. Restore 5 defense to your leader.\nStorm\nWard\nAura",
  "attack": 10,
  "defense": 10,
  "tribes": [
    "artifact"
  ],
  "traits": {
    "storm": true,
    "ward": true,
    "aura": true
  },
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Deal 5 damage to all enemy followers. Restore 5 defense to your leader.",
      "effects": [
        {
          "printed": "Deal 5 damage to all enemy followers.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "all"
          },
          "amount": 5
        },
        {
          "printed": "Restore 5 defense to your leader.",
          "op": "restore",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 5
        }
      ]
    }
  ]
}
```

### `90074140` — Imari's Little Buddies

follower. File: `cards/tokens/90074140.json`

```
Rush
```

```json
{
  "id": "90074140",
  "name": "Imari's Little Buddies",
  "kind": "follower",
  "class": "portalcraft",
  "set": 90000,
  "rarity": "legendary",
  "token": true,
  "cost": 2,
  "text": "Rush",
  "attack": 3,
  "defense": 3,
  "traits": {
    "rush": true
  }
}
```

### `crest:10404110` — Crest: Sandalphon, Primarch Successor

crest. File: `cards/crests/crest-10404110.json`

```
Countdown (2)
At the end of your turn, restore 1 defense to all allies.
```

```json
{
  "id": "crest:10404110",
  "name": "Crest: Sandalphon, Primarch Successor",
  "grantedBy": [
    "10404110"
  ],
  "faith": false,
  "text": "Countdown (2)\nAt the end of your turn, restore 1 defense to all allies.",
  "countdown": 2,
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, restore 1 defense to all allies.",
      "effects": [
        {
          "printed": "restore 1 defense to all allies.",
          "op": "restore",
          "select": {
            "side": "ally",
            "kind": "character",
            "pick": "all",
            "includeLeader": true,
            "zone": "field"
          },
          "amount": 1
        }
      ]
    }
  ]
}
```

### `crest:10434120` — Crest: Cagliostro, Genius Alchemist

crest. File: `cards/crests/crest-10434120.json`

```
At the start of your turn, Earth Rite (1) - Add an Ars Magna to your hand.
```

```json
{
  "id": "crest:10434120",
  "name": "Crest: Cagliostro, Genius Alchemist",
  "grantedBy": [
    "10434120"
  ],
  "faith": false,
  "text": "At the start of your turn, Earth Rite (1) - Add an Ars Magna to your hand.",
  "abilities": [
    {
      "on": "startOfTurn",
      "whose": "own",
      "printed": "At the start of your turn, Earth Rite (1) - Add an Ars Magna to your hand.",
      "effects": [
        {
          "printed": "Earth Rite (1) - Add an Ars Magna to your hand.",
          "op": "pay",
          "resource": "earth",
          "amount": 1,
          "effects": [
            {
              "op": "addToHand",
              "card": {
                "named": "90034320"
              },
              "count": 1
            }
          ]
        }
      ]
    }
  ]
}
```

### `crest:10554110` — Crest: Milteo & Luzen

crest. File: `cards/crests/crest-10554110.json`

```
Allied followers' Fanfare and Enhance abilities don't activate.
Whenever you play a follower, evolve it.
```

```json
{
  "id": "crest:10554110",
  "name": "Crest: Milteo & Luzen",
  "grantedBy": [
    "10554110"
  ],
  "faith": false,
  "text": "Allied followers' Fanfare and Enhance abilities don't activate.\nWhenever you play a follower, evolve it.",
  "abilities": [
    {
      "on": "static",
      "printed": "Allied followers' Fanfare and Enhance abilities don't activate.",
      "modifier": {
        "suppress": [
          "fanfare",
          "enhance"
        ],
        "select": {
          "pick": "all",
          "side": "ally",
          "zone": "field",
          "kind": "follower"
        }
      }
    },
    {
      "on": "when",
      "event": "ally_card_played",
      "filter": {
        "kind": "follower"
      },
      "printed": "Whenever you play a follower, evolve it.",
      "effects": [
        {
          "printed": "evolve it.",
          "op": "evolve",
          "select": {
            "pick": "entering"
          },
          "super": false
        }
      ]
    }
  ]
}
```

### `crest:10564120` — Crest: Kukishiro, Mistbloom

crest. File: `cards/crests/crest-10564120.json`

```
During your turn, whenever you draw a 1-, 3-, or 5-cost card, summon a Fox of Purity or Holy Falcon at random.
During your turn, whenever you draw a 2-, 4-, or 6-cost card, summon an enemy Fox of Purity or Holy Falcon at random.
```

```json
{
  "id": "crest:10564120",
  "name": "Crest: Kukishiro, Mistbloom",
  "grantedBy": [
    "10564120"
  ],
  "faith": false,
  "text": "During your turn, whenever you draw a 1-, 3-, or 5-cost card, summon a Fox of Purity or Holy Falcon at random.\nDuring your turn, whenever you draw a 2-, 4-, or 6-cost card, summon an enemy Fox of Purity or Holy Falcon at random.",
  "abilities": [
    {
      "on": "when",
      "event": "ally_draw",
      "filter": {
        "costIn": [
          1,
          3,
          5
        ]
      },
      "printed": "During your turn, whenever you draw a 1-, 3-, or 5-cost card, summon a Fox of Purity or Holy Falcon at random.",
      "when": {
        "turnOwner": "self"
      },
      "effects": [
        {
          "printed": "summon a Fox of Purity or Holy Falcon at random.",
          "op": "summon",
          "card": {
            "randomFrom": {
              "cards": [
                "10061120",
                "90061110"
              ]
            }
          },
          "count": 1
        }
      ]
    },
    {
      "on": "when",
      "event": "ally_draw",
      "filter": {
        "costIn": [
          2,
          4,
          6
        ]
      },
      "printed": "During your turn, whenever you draw a 2-, 4-, or 6-cost card, summon an enemy Fox of Purity or Holy Falcon at random.",
      "when": {
        "turnOwner": "self"
      },
      "effects": [
        {
          "printed": "summon an enemy Fox of Purity or Holy Falcon at random.",
          "op": "summon",
          "card": {
            "randomFrom": {
              "cards": [
                "10061120",
                "90061110"
              ]
            }
          },
          "count": 1,
          "controller": "opponent"
        }
      ]
    }
  ]
}
```

### `crest:10574110` — Crest: Slaus, Revolving Wheel of Fortune

crest. File: `cards/crests/crest-10574110.json`

```
Countdown (3)
At the start of your turn, activate a random ability that hasn't been activated yet from the following.
1. Increase the cost of all cards in your hand by 1 until the end of the turn.
2. Give all allied followers on the field -2/-2.
3. Deal 3 damage to your leader.
```

```json
{
  "id": "crest:10574110",
  "name": "Crest: Slaus, Revolving Wheel of Fortune",
  "grantedBy": [
    "10574110"
  ],
  "faith": false,
  "text": "Countdown (3)\nAt the start of your turn, activate a random ability that hasn't been activated yet from the following.\n1. Increase the cost of all cards in your hand by 1 until the end of the turn.\n2. Give all allied followers on the field -2/-2.\n3. Deal 3 damage to your leader.",
  "countdown": 3,
  "abilities": [
    {
      "on": "startOfTurn",
      "whose": "own",
      "printed": "At the start of your turn, activate a random ability that hasn't been activated yet from the following.\n1. Increase the cost of all cards in your hand by 1 until the end of the turn.\n2. Give all allied followers on the field -2/-2.\n3. Deal 3 damage to your leader.",
      "effects": [
        {
          "printed": "activate a random ability that hasn't been activated yet from the following.\n1. Increase the cost of all cards in your hand by 1 until the end of the turn.\n2. Give all allied followers on the field -2/-2.\n3. Deal 3 damage to your leader.",
          "op": "choose",
          "pick": 1,
          "by": "randomUnused",
          "options": [
            {
              "printed": "1. Increase the cost of all cards in your hand by 1 until the end of the turn.",
              "effects": [
                {
                  "op": "cost",
                  "select": {
                    "side": "ally",
                    "zone": "hand",
                    "kind": "card",
                    "pick": "all"
                  },
                  "delta": 1,
                  "untilEndOfTurn": true
                }
              ]
            },
            {
              "printed": "2. Give all allied followers on the field -2/-2.",
              "effects": [
                {
                  "op": "buff",
                  "select": {
                    "side": "ally",
                    "zone": "field",
                    "kind": "follower",
                    "pick": "all"
                  },
                  "attack": -2,
                  "defense": -2
                }
              ]
            },
            {
              "printed": "3. Deal 3 damage to your leader.",
              "effects": [
                {
                  "op": "damage",
                  "select": {
                    "pick": "all",
                    "side": "ally",
                    "zone": "leader",
                    "kind": "leader"
                  },
                  "amount": 3
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### `crest:10704110` — Crest: Illamrita, Designated Target

crest. File: `cards/crests/crest-10704110.json`

```
Countdown (2)
Last Words: Summon an Illamrita, Designated Target and evolve it.
```

```json
{
  "id": "crest:10704110",
  "name": "Crest: Illamrita, Designated Target",
  "grantedBy": [
    "10704110"
  ],
  "faith": false,
  "text": "Countdown (2)\nLast Words: Summon an Illamrita, Designated Target and evolve it. ",
  "countdown": 2,
  "abilities": [
    {
      "on": "lastWords",
      "printed": "Last Words: Summon an Illamrita, Designated Target and evolve it.",
      "effects": [
        {
          "printed": "Summon an Illamrita, Designated Target and evolve it.",
          "op": "seq",
          "effects": [
            {
              "op": "summon",
              "card": {
                "named": "10704110"
              },
              "count": 1,
              "as": "s"
            },
            {
              "op": "evolve",
              "select": {
                "pick": "bound",
                "ref": "s"
              },
              "super": false
            }
          ]
        }
      ]
    }
  ]
}
```

### `crest:10714110` — Crest: Thestae, Anathema of Distortion

crest. File: `cards/crests/crest-10714110.json`

```
Countdown (3)
At the end of your turn, Combo (3) - Give all followers in your deck +1/+1.
```

```json
{
  "id": "crest:10714110",
  "name": "Crest: Thestae, Anathema of Distortion",
  "grantedBy": [
    "10714110"
  ],
  "faith": false,
  "text": "Countdown (3)\nAt the end of your turn, Combo (3) - Give all followers in your deck +1/+1.",
  "countdown": 3,
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, Combo (3) - Give all followers in your deck +1/+1.",
      "effects": [
        {
          "printed": "Combo (3) - Give all followers in your deck +1/+1.",
          "op": "if",
          "cond": {
            "combo": {
              "n": 3
            }
          },
          "then": [
            {
              "op": "buff",
              "select": {
                "side": "ally",
                "zone": "deck",
                "kind": "follower",
                "pick": "all"
              },
              "attack": 1,
              "defense": 1
            }
          ]
        }
      ]
    }
  ]
}
```

### `crest:10724110` — Crest: Gildaria, Anathema of Attunement

crest. File: `cards/crests/crest-10724110.json`

```
Countdown (1)
During your turn, whenever an allied follower enters the field, deal 1 damage to the enemy leader.
```

```json
{
  "id": "crest:10724110",
  "name": "Crest: Gildaria, Anathema of Attunement",
  "grantedBy": [
    "10724110"
  ],
  "faith": false,
  "text": "Countdown (1)\nDuring your turn, whenever an allied follower enters the field, deal 1 damage to the enemy leader.",
  "countdown": 1,
  "abilities": [
    {
      "on": "when",
      "event": "ally_follower_enter",
      "printed": "During your turn, whenever an allied follower enters the field, deal 1 damage to the enemy leader.",
      "when": {
        "turnOwner": "self"
      },
      "effects": [
        {
          "printed": "deal 1 damage to the enemy leader.",
          "op": "damage",
          "select": {
            "pick": "all",
            "side": "enemy",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 1
        }
      ]
    }
  ]
}
```

### `crest:10934110` — Crest: Sephie, Maven Convict

crest. File: `cards/crests/crest-10934110.json`

```
Once on each of your turns, when an allied Obsessed Test Subject enters the field, give it Storm.
```

```json
{
  "id": "crest:10934110",
  "name": "Crest: Sephie, Maven Convict",
  "grantedBy": [
    "10934110"
  ],
  "faith": false,
  "text": "Once on each of your turns, when an allied Obsessed Test Subject enters the field, give it Storm.",
  "abilities": [
    {
      "on": "when",
      "event": "ally_follower_enter",
      "filter": {
        "card": "10931110"
      },
      "oncePerTurn": true,
      "printed": "Once on each of your turns, when an allied Obsessed Test Subject enters the field, give it Storm.",
      "effects": [
        {
          "printed": "give it Storm.",
          "op": "grantTraits",
          "select": {
            "pick": "entering"
          },
          "traits": {
            "storm": true
          }
        }
      ]
    }
  ]
}
```

### `crest:10954110` — Crest: Istyndet vs. Mitilykket

crest. File: `cards/crests/crest-10954110.json`

```
At the end of your turn, if there's an allied card on the field with Last Words, destroy a random allied card with Last Words and a random enemy follower.
```

```json
{
  "id": "crest:10954110",
  "name": "Crest: Istyndet vs. Mitilykket",
  "grantedBy": [
    "10954110"
  ],
  "faith": false,
  "text": "At the end of your turn, if there's an allied card on the field with Last Words, destroy a random allied card with Last Words and a random enemy follower.",
  "abilities": [
    {
      "on": "endOfTurn",
      "whose": "own",
      "printed": "At the end of your turn, if there's an allied card on the field with Last Words, destroy a random allied card with Last Words and a random enemy follower.",
      "when": {
        "fieldHas": {
          "filter": {
            "hasLastWords": true
          },
          "side": "ally",
          "kind": "card"
        }
      },
      "effects": [
        {
          "printed": "destroy a random allied card with Last Words and a random enemy follower.",
          "op": "seq",
          "effects": [
            {
              "op": "destroy",
              "select": {
                "side": "ally",
                "zone": "field",
                "kind": "card",
                "pick": "random",
                "filter": {
                  "hasLastWords": true
                }
              }
            },
            {
              "op": "destroy",
              "select": {
                "side": "enemy",
                "zone": "field",
                "kind": "follower",
                "pick": "random"
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### `faith:10634120` — Faith: Calge-Danthla, Eld Crystals

crest. File: `cards/crests/faith-10634120.json`

```
This faith's value starts at 0.

Whenever an allied Crystalspawn enters the field, increase this faith's value by 1.
```

```json
{
  "id": "faith:10634120",
  "name": "Faith: Calge-Danthla, Eld Crystals",
  "grantedBy": [
    "10634120"
  ],
  "faith": true,
  "text": "This faith's value starts at 0.\n\nWhenever an allied Crystalspawn enters the field, increase this faith's value by 1.",
  "abilities": [
    {
      "on": "when",
      "event": "ally_follower_enter",
      "filter": {
        "card": "10631110"
      },
      "printed": "Whenever an allied Crystalspawn enters the field, increase this faith's value by 1.",
      "effects": [
        {
          "printed": "increase this faith's value by 1.",
          "op": "counter",
          "key": "faith",
          "how": "add",
          "amount": 1
        }
      ]
    }
  ]
}
```


### `10614120` — Sathanid, Eld Lance

File: `cards/10006/10614120.json`

```json
{
  "id": "10614120",
  "name": "Sathanid, Eld Lance",
  "kind": "follower",
  "class": "forestcraft",
  "set": 10006,
  "rarity": "legendary",
  "token": false,
  "cost": 1,
  "text": "Fanfare: Reduce your faith's value by 10 to add a Depths of the Eld Lance to your hand and give your faith \"Whenever an allied follower evolves, deal 1 damage to the enemy leader.\"\nDrain",
  "attack": 1,
  "defense": 1,
  "tribes": [
    "encroacher"
  ],
  "traits": {
    "drain": true
  },
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Reduce your faith's value by 10 to add a Depths of the Eld Lance to your hand and give your faith \"Whenever an allied follower evolves, deal 1 damage to the enemy leader.\"",
      "effects": [
        {
          "printed": "Reduce your faith's value by 10 to add a Depths of the Eld Lance to your hand and give your faith \"Whenever an allied follower evolves, deal 1 damage to the enemy leader.\"",
          "op": "pay",
          "resource": "faith",
          "amount": 10,
          "effects": [
            {
              "op": "addToHand",
              "card": {
                "named": "90014330"
              },
              "count": 1
            },
            {
              "op": "grantAbility",
              "select": {
                "pick": "all",
                "side": "ally",
                "zone": "crests",
                "kind": "faith"
              },
              "ability": {
                "on": "when",
                "event": "ally_evolve",
                "printed": "Whenever an allied follower evolves, deal 1 damage to the enemy leader.",
                "effects": [
                  {
                    "printed": "deal 1 damage to the enemy leader.",
                    "op": "damage",
                    "select": {
                      "pick": "all",
                      "side": "enemy",
                      "zone": "leader",
                      "kind": "leader"
                    },
                    "amount": 1
                  }
                ]
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### `10624120` — Yidmetra, Eld Sword

File: `cards/10006/10624120.json`

```json
{
  "id": "10624120",
  "name": "Yidmetra, Eld Sword",
  "kind": "follower",
  "class": "swordcraft",
  "set": 10006,
  "rarity": "legendary",
  "token": false,
  "cost": 2,
  "text": "Fanfare: Add a Depths of the Eld Sword to your hand.\nEvolve: Reduce your faith's value by 5 to give it \"Whenever you play an Enhanced card, give all allied followers on the field +1/+1.\"",
  "attack": 1,
  "defense": 2,
  "tribes": [
    "encroacher"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Add a Depths of the Eld Sword to your hand.",
      "effects": [
        {
          "printed": "Add a Depths of the Eld Sword to your hand.",
          "op": "addToHand",
          "card": {
            "named": "90024320"
          },
          "count": 1
        }
      ]
    },
    {
      "on": "evolve",
      "printed": "Evolve: Reduce your faith's value by 5 to give it \"Whenever you play an Enhanced card, give all allied followers on the field +1/+1.\"",
      "effects": [
        {
          "printed": "Reduce your faith's value by 5 to give it \"Whenever you play an Enhanced card, give all allied followers on the field +1/+1.\"",
          "op": "pay",
          "resource": "faith",
          "amount": 5,
          "effects": [
            {
              "op": "grantAbility",
              "select": {
                "pick": "all",
                "side": "ally",
                "zone": "crests",
                "kind": "faith"
              },
              "ability": {
                "on": "when",
                "event": "ally_card_played",
                "filter": {
                  "enhanced": true
                },
                "printed": "Whenever you play an Enhanced card, give all allied followers on the field +1/+1.",
                "effects": [
                  {
                    "printed": "give all allied followers on the field +1/+1.",
                    "op": "buff",
                    "select": {
                      "pick": "all",
                      "side": "ally",
                      "zone": "field",
                      "kind": "follower"
                    },
                    "attack": 1,
                    "defense": 1
                  }
                ]
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### `90014330` — Depths of the Eld Lance

File: `cards/tokens/90014330.json`

```json
{
  "id": "90014330",
  "name": "Depths of the Eld Lance",
  "kind": "spell",
  "class": "forestcraft",
  "set": 90000,
  "rarity": "legendary",
  "token": true,
  "cost": 1,
  "text": "Select an unevolved allied follower on the field and evolve it.",
  "tribes": [
    "encroacher"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select an unevolved allied follower on the field and evolve it.",
      "effects": [
        {
          "printed": "Select an unevolved allied follower on the field and evolve it.",
          "op": "evolve",
          "select": {
            "pick": "choose",
            "side": "ally",
            "zone": "field",
            "kind": "follower",
            "filter": {
              "unevolved": true
            }
          },
          "super": false
        }
      ]
    }
  ]
}
```

### `90024320` — Depths of the Eld Sword

File: `cards/tokens/90024320.json`

```json
{
  "id": "90024320",
  "name": "Depths of the Eld Sword",
  "kind": "spell",
  "class": "swordcraft",
  "set": 90000,
  "rarity": "legendary",
  "token": true,
  "cost": 0,
  "text": "Select an enemy follower on the field and deal it 1 damage.\nEnhance (1): Deal 3 damage instead.",
  "tribes": [
    "encroacher"
  ],
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Select an enemy follower on the field and deal it 1 damage.",
      "effects": [
        {
          "printed": "Select an enemy follower on the field and deal it 1 damage.",
          "op": "damage",
          "select": {
            "pick": "choose",
            "side": "enemy",
            "zone": "field",
            "kind": "follower"
          },
          "amount": 1
        }
      ]
    }
  ],
  "modes": [
    {
      "kind": "enhance",
      "cost": 1,
      "replacesBase": true,
      "printed": "Enhance (1): Deal 3 damage instead.",
      "effects": [
        {
          "printed": "Deal 3 damage instead.",
          "op": "damage",
          "select": {
            "pick": "choose",
            "side": "enemy",
            "zone": "field",
            "kind": "follower"
          },
          "amount": 3
        }
      ]
    }
  ]
}
```

### `faith:10614120` — Faith: Sathanid, Eld Lance

File: `cards/crests/faith-10614120.json`

```json
{
  "id": "faith:10614120",
  "name": "Faith: Sathanid, Eld Lance",
  "grantedBy": [
    "10614120"
  ],
  "faith": true,
  "text": "This faith's value starts at 0.\n\nWhenever an allied follower evolves, increase this faith's value by 1.",
  "abilities": [
    {
      "on": "when",
      "event": "ally_evolve",
      "printed": "Whenever an allied follower evolves, increase this faith's value by 1.",
      "effects": [
        {
          "printed": "increase this faith's value by 1.",
          "op": "counter",
          "key": "faith",
          "how": "add",
          "amount": 1
        }
      ]
    }
  ]
}
```

### `faith:10624120` — Faith: Yidmetra, Eld Sword

File: `cards/crests/faith-10624120.json`

```json
{
  "id": "faith:10624120",
  "name": "Faith: Yidmetra, Eld Sword",
  "grantedBy": [
    "10624120"
  ],
  "faith": true,
  "text": "This faith's value starts at 0.\n\nWhenever you play an Enhanced card, increase this faith's value by 1.",
  "abilities": [
    {
      "on": "when",
      "event": "ally_card_played",
      "filter": {
        "enhanced": true
      },
      "printed": "Whenever you play an Enhanced card, increase this faith's value by 1.",
      "effects": [
        {
          "printed": "increase this faith's value by 1.",
          "op": "counter",
          "key": "faith",
          "how": "add",
          "amount": 1
        }
      ]
    }
  ]
}
```

### `10022120` — Rusty, Luxcard Trickster

draw all deck copies by id (`count: {count: Selector}`) then grantTraits on bound draw.

Printed text:

```
Super-Evolve: Draw all copies of Rusty, Luxcard Trickster and give them Storm.
```

File: `cards/10000/10022120.json`

```json
{
  "id": "10022120",
  "name": "Rusty, Luxcard Trickster",
  "kind": "follower",
  "class": "swordcraft",
  "set": 10000,
  "rarity": "silver",
  "token": false,
  "cost": 3,
  "text": "Super-Evolve: Draw all copies of Rusty, Luxcard Trickster and give them Storm.",
  "attack": 3,
  "defense": 3,
  "abilities": [
    {
      "on": "superEvolve",
      "printed": "Super-Evolve: Draw all copies of Rusty, Luxcard Trickster and give them Storm.",
      "effects": [
        {
          "printed": "Draw all copies of Rusty, Luxcard Trickster and give them Storm.",
          "op": "seq",
          "effects": [
            {
              "op": "draw",
              "filter": {
                "card": "10022120"
              },
              "count": {
                "count": {
                  "side": "ally",
                  "zone": "deck",
                  "kind": "card",
                  "pick": "all",
                  "filter": {
                    "card": "10022120"
                  }
                }
              },
              "as": "d"
            },
            {
              "op": "grantTraits",
              "select": {
                "pick": "bound",
                "ref": "d"
              },
              "traits": {
                "storm": true
              }
            }
          ]
        }
      ]
    }
  ],
  "tribes": []
}
```

### `10503310` — Fate of the World

destroy random highest-attack enemy follower (`pick: highest`, `orderBy: attack`).

Printed text:

```
Draw 2 cards. Destroy a random enemy follower with the highest attack.
Enhance (10): Deal 4 damage to all enemies.
```

File: `cards/10005/10503310.json`

```json
{
  "id": "10503310",
  "name": "Fate of the World",
  "kind": "spell",
  "class": "neutral",
  "set": 10005,
  "rarity": "gold",
  "token": false,
  "cost": 5,
  "text": "Draw 2 cards. Destroy a random enemy follower with the highest attack.\nEnhance (10): Deal 4 damage to all enemies.",
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Draw 2 cards. Destroy a random enemy follower with the highest attack.",
      "effects": [
        {
          "printed": "Draw 2 cards.",
          "op": "draw",
          "count": 2
        },
        {
          "printed": "Destroy a random enemy follower with the highest attack.",
          "op": "destroy",
          "select": {
            "side": "enemy",
            "zone": "field",
            "kind": "follower",
            "pick": "highest",
            "orderBy": "attack"
          }
        }
      ]
    }
  ],
  "modes": [
    {
      "kind": "enhance",
      "cost": 10,
      "printed": "Enhance (10): Deal 4 damage to all enemies.",
      "effects": [
        {
          "printed": "Deal 4 damage to all enemies.",
          "op": "damage",
          "select": {
            "side": "enemy",
            "kind": "character",
            "pick": "all",
            "includeLeader": true,
            "zone": "field"
          },
          "amount": 4
        }
      ]
    }
  ],
  "tribes": []
}
```

### `10804110` — Alabaster Bahamut

mode 3 `removeCrests` over `zone: crests` (faiths excluded per official Q&A).

Printed text:

```
Fanfare: Select a Mode to activate.
1. Banish all other followers from the field.
2. Banish all amulets from the field.
3. Banish all crests.
```

File: `cards/10008/10804110.json`

```json
{
  "id": "10804110",
  "name": "Alabaster Bahamut",
  "kind": "follower",
  "class": "neutral",
  "set": 10008,
  "rarity": "legendary",
  "token": false,
  "cost": 9,
  "text": "Fanfare: Select a Mode to activate.\n1. Banish all other followers from the field.\n2. Banish all amulets from the field.\n3. Banish all crests.",
  "attack": 13,
  "defense": 13,
  "abilities": [
    {
      "on": "fanfare",
      "printed": "Fanfare: Select a Mode to activate.\n1. Banish all other followers from the field.\n2. Banish all amulets from the field.\n3. Banish all crests.",
      "effects": [
        {
          "printed": "Select a Mode to activate.\n1. Banish all other followers from the field.\n2. Banish all amulets from the field.\n3. Banish all crests.",
          "op": "choose",
          "pick": 1,
          "by": "player",
          "options": [
            {
              "printed": "1. Banish all other followers from the field.",
              "effects": [
                {
                  "op": "banish",
                  "select": {
                    "side": "any",
                    "zone": "field",
                    "kind": "follower",
                    "pick": "all",
                    "other": true
                  }
                }
              ]
            },
            {
              "printed": "2. Banish all amulets from the field.",
              "effects": [
                {
                  "op": "banish",
                  "select": {
                    "side": "any",
                    "zone": "field",
                    "kind": "amulet",
                    "pick": "all"
                  }
                }
              ]
            },
            {
              "printed": "3. Banish all crests.",
              "effects": [
                {
                  "op": "removeCrests",
                  "select": {
                    "side": "any",
                    "zone": "crests",
                    "kind": "card",
                    "pick": "all"
                  }
                }
              ]
            }
          ]
        }
      ]
    }
  ],
  "tribes": []
}
```

### `crest:10744110` — Crest: Burnite, Anathema of Ash

startOfTurn self-damage; `leader_restored` once per turn.

Printed text:

```
At the start of your turn, deal 2 damage to your leader.
Once on each of your turns, when your leader's defense is restored, deal 1 damage to it.
```

File: `cards/crests/crest-10744110.json`

```json
{
  "id": "crest:10744110",
  "name": "Crest: Burnite, Anathema of Ash",
  "grantedBy": [
    "10744110"
  ],
  "faith": false,
  "text": "At the start of your turn, deal 2 damage to your leader.\nOnce on each of your turns, when your leader's defense is restored, deal 1 damage to it.",
  "abilities": [
    {
      "on": "startOfTurn",
      "whose": "own",
      "printed": "At the start of your turn, deal 2 damage to your leader.",
      "effects": [
        {
          "printed": "deal 2 damage to your leader.",
          "op": "damage",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 2
        }
      ]
    },
    {
      "on": "when",
      "event": "leader_restored",
      "oncePerTurn": true,
      "printed": "Once on each of your turns, when your leader's defense is restored, deal 1 damage to it.",
      "effects": [
        {
          "printed": "deal 1 damage to it.",
          "op": "damage",
          "select": {
            "pick": "all",
            "side": "ally",
            "zone": "leader",
            "kind": "leader"
          },
          "amount": 1
        }
      ]
    }
  ]
}
```
