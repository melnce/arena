# Trace format

Shared JSONL both engines emit. A second implementer should need no questions. Version `v: 1`.

One object per line. Keys sorted on `CanonicalState`. No floats, no uids, no engine-private fields.

## Line 1 — header

```json
{
  "v": 1,
  "engine": "<name>",
  "seed": 0,
  "first": "a",
  "deck_a": ["10001110", "…"],
  "deck_b": ["…"],
  "opening_hands": {
    "a": ["10001110", "10001120", "10001130", "10001210"],
    "b": ["10011110", "10011120", "10011130", "10011210"]
  }
}
```

- `engine`: `"arena"` or `"practice-tool"` (or any stable name).
- `seed`: `u64`.
- `first`: `"a"` or `"b"`.
- `deck_a` / `deck_b`: multisets of 40 card ids, **sorted** (so `["x","x","y"]` not insertion order). Deck **order is not part of the format**.
- `opening_hands` (**required**): the four-card opening hands **before** the mulligan, each a list of card ids **in draw order**. The old engine draws those hands inside game setup, before any action line exists, so they cannot be per-line `draw` picks. A replayer builds the pre-mulligan state by removing those ids from the decklist multisets (A's four from `deck_a`, B's four from `deck_b`). Every later draw (mulligan replacements, the first player's turn-1 draw, turn draws, effect draws) stays a per-line `{"what":"draw","chose":"<card id>"}` pick.

The old engine also emits `{"what":"raw","kind":"shuffle",…}` entries for its deck shuffles — ignore every `raw` pick.

If `first` was decided by a coin, the header's `first` is the outcome and the first `rng` pick of the first action line may repeat `{"what":"coin","chose":"a"}` when the engine actually rolled.

## Action lines

One line per applied action:

```json
{
  "i": 0,
  "action": { "mulligan": { "player": "a", "swap": [true, false, false, true] } },
  "rng": [ { "what": "draw", "chose": "10131320" } ],
  "state": { },
  "legal": [ ]
}
```

- `i`: 0-based index of applied actions after the header.
- `action`: `NeutralAction` (below).
- `rng`: every random decision made while applying the action, in the order the engine asked, **recorded by outcome, not index**.
- `state`: `CanonicalState` after the action.
- `legal` (optional): sorted list of `NeutralAction` legal in the resulting state. A run that includes `legal` is green only if both the state and this set match.

## NeutralAction

Closed. Discriminator is the single key.

| NeutralAction | Shape |
|---|---|
| mulligan | `{ "mulligan": { "player": "a"\|"b", "swap": [bool, bool, bool, bool] } }` |
| play | `{ "play": { "player", "hand_pos": 0, "card": "<id>" } }` |
| attack | `{ "attack": { "player", "attacker_slot": 0, "target": { "slot": 1 } \| "leader" } }` |
| evolve | `{ "evolve": { "player", "slot": 0, "super": false } }` |
| engage | `{ "engage": { "player", "slot": 0 } }` |
| fuse | `{ "fuse": { "player", "host_pos": 0, "partner_pos": [1, 2] } }` |
| bonus_pp | `{ "bonus_pp": { "player" } }` |
| choose | `{ "choose": { "player", "option": { "card": "<id>" } \| { "slot": 0 } \| "leader" \| { "mode": 0 } } }` |
| confirm | `{ "confirm": { "player" } }` |
| end_turn | `{ "end_turn": { "player" } }` |

`choose.option` names the chosen thing **by content**, not by the old engine's uid.

### Map from the old engine (`src/core/types/actions.ts`)

| Old `ActionType` | NeutralAction |
|---|---|
| `PLAY_CARD` | `play` (`hand_pos` from the hand order at the time; `card` is the Cygames id) |
| `ATTACK` | `attack` (`attackerUid` → `attacker_slot`; `defender` card uid → `{slot}`; leader → `"leader"`) |
| `CHOOSE_TARGET` | `choose` with `{card}`, `{slot}`, or `"leader"` |
| `EVOLVE` | `evolve` (`mode: "super"` → `super: true`) |
| `ENGAGE` | `engage` |
| `BONUS_PP` | `bonus_pp` |
| `CHOOSE_MODE` | `choose` with `{mode: n}` (0-based into the offered list) |
| `TOGGLE_MULLIGAN` | **not emitted**. The old two-step toggle+confirm collapses to one `mulligan` when `CONFIRM_MULLIGAN` is applied. The `swap` bitmask is the toggled set at confirm time. |
| `CONFIRM_MULLIGAN` | `mulligan` |
| `CONFIRM_TARGETS` | `confirm` |
| `FUSE` | `fuse` (partners filled from the following `CHOOSE_TARGET`s / confirm; the emitted line is the completed fuse) |
| `END_TURN` | `end_turn` |
| `UNDO` / `REDO` / `RESET_HISTORY` | **not in the format**. Undo is a client concept. |

`FORCE_COMPLETE_PENDING` (soak-only) is not a NeutralAction; a fizzle is recorded as the action that was already in flight plus whatever the engine did.

## Pick

Every random decision, by outcome.

```jsonc
{"what":"draw","chose":"10131320"}
{"what":"random_target","among":"enemy_followers","chose":{"slot":2}}
{"what":"random_card","chose":"10131320"}
{"what":"random_split","chose":[2,0,1]}   // counts for keys in schema order, e.g. X,Y,Z
{"what":"coin","chose":"a"}
{"what":"random_unused","chose":{"mode":1}}
{"what":"reanimate","chose":"90051140"}
{"what":"multiset_pick","among":"deck","chose":"10001110"}
```

`ScriptedRng` matches `chose` against its candidate list. If `chose` is not a candidate, the run fails with `oracle picked X; not legal here`.

Do not record RNG as an index into an unstable list.

## CanonicalState

Projection both engines can produce. Keys sorted.

```jsonc
{
  "active": "a",
  "phase": "main",          // "mulligan" | "main" | "choice" | "end" | "terminal"
  "players": {
    "a": { /* PlayerState */ },
    "b": { /* PlayerState */ }
  },
  "turn": 1,
  "winner": null            // "a" | "b" | null
}
```

### PlayerState

```jsonc
{
  "banished": { "90011110": 1 },     // sorted multiset {card_id: count}
  "cemetery": { "10001110": 2 },
  "combo": 0,
  "crests": [ { "countdown": 3, "id": "crest:10574110" } ],
  "deck": { "10001110": 3, "10002110": 2 },
  "earth": 0,
  "ep": 2,
  "evolves_used": 0,
  "faith": 0,
  "field": [ /* 0..4, omitted empties are null */ ],
  "hand": [ { "card": "10131320", "cost": 1, "vars": { "X": 2 }, "skybound": 0 } ],
  "leader_defense": 20,
  "leader_max": 20,
  "pp": 1,
  "pp_bonus": 0,
  "pp_max": 1,
  "rally": 0,
  "sep": 0,
  "shadows": 0
}
```

`field` is an array of five slots (null if empty), each:

```jsonc
{
  "attack": 3,
  "attacks_left": 1,
  "can_attack": true,
  "card": "10574110",
  "countdown": null,
  "defense": 3,
  "evolved": false,
  "max_defense": 3,
  "super": false,
  "traits": ["ambush"],     // sorted
  "vars": { "X": 1 },       // optional; Stormy Blast-style X on a field copy if it exists
  "granted": ["lastWords"]  // optional; sorted trigger tags, not printed text
}
```

`hand` is in draw order: `[{card, cost, vars?, skybound?}, …]`. `vars` is Stormy Blast's X (and any other `{X,Y,Z}`). `skybound` is the Skybound Art gauge **per card in hand** (turn + evolves while in hand + Tsubasa boosts) — not a per-player field.

`deck`, `cemetery`, `banished` are **sorted multisets** `{card_id: count}` (JSON object keys sorted).

### Deliberately not in CanonicalState

| Omitted | Why |
|---|---|
| uids / object identity | not load-bearing; slots and multisets replace them |
| per-instance granted-ability **text** | grants are gameplay; the snapshot carries a sorted `granted` list of **trigger tags** (`lastWords`, `fanfare`, …), not the quoted string |
| undo / history / checkpoints | client-only |
| pending-target click buffers | `phase: choice` plus `legal` is enough |
| RNG internal counters | the next line's `rng` picks are the contract |
| hidden-hand flags | the engine is perfect-information |
| animation / UI | out of scope |

## Diff procedure

1. Parse line 1. Header `seed`, `first`, sorted `deck_a` / `deck_b` must match. `engine` may differ.
2. For each subsequent line with the same `i`:
   - Diff `action` (must be identical NeutralAction).
   - Diff `state` with a sorted-key deep equal. Print the JSON path and both values at the first differing leaf.
   - If both sides have `legal`, sort and compare.
   - `rng` is **not** required to match as a byte string (the old engine may roll more times internally). The **outcomes that affect state** must be consistent with `state`. A harness that drives both engines from one side's `rng` treats a non-candidate outcome as a finding.
3. First divergence wins. Extra/missing lines are a divergence at that `i`.
4. Green: every line's `state` matches, and `legal` matches when present, through terminal or the agreed action cap.

## Conventions

Pinned 2026-09-10 from the first differential run. Both emitters follow these; a reader may rely on them.

- **`turn`** is the round number: both players' first turns are turn 1, both second turns are turn 2, and so on (the number the rules text means by "your Nth turn"). It is `0` while `phase` is `mulligan`.
- **PP before a player's first turn.** A player has `pp: 0, pp_max: 0, pp_bonus: 0` until their first turn has started; during the mulligan that is both players, during the first player's turn 1 it is the second player.
- **`legal` during the mulligan.** While `phase` is `mulligan`, `legal` is the 16 `mulligan` actions (every `swap` bitmask) of the player whose mulligan is pending, sorted like every other `legal` list. The first player's mulligan is decided first.
- **Actor.** The `player` of every action is the player who performs it: `end_turn.player` is the player whose turn ends, `mulligan.player` the player deciding, `choose`/`confirm` the player who owns the pending choice.
- **`play.hand_pos` / `play.card`**, `fuse.host_pos` / `partner_pos` and `choose.option.card` are resolved against the state **before** the action is applied. `play` carries no form: whether the card resolves as Enhance, printed, Accelerate or Crystallize is the engine's decision from the PP available (Enhance when affordable; the printed form when affordable; otherwise the highest payable alternate form).
- **`legal` is a set** — identical NeutralActions appear once.
- **A card chosen from hand, deck or cemetery** is `choose {card: "<id>"}`; copies of one id collapse to one option; the engine picks any copy.
- **When a `choose {card}` names an id with several copies in the zone, the copy at the lowest position is taken.**
- **`evolves_used` counts every allied follower evolution this match — EP, SEP or effect.**
- **`leader_defense` is clamped at 0.**
- **A crest entry carries `countdown` only when it has one.**
- **At a line whose `phase` is `terminal`, only `phase` and `winner` are compared; the rest of the state is post-mortem and engine-private.**
- **`bonus_pp`** is a toggle. It activates the second player's current-tier charge, and cancels an activated one while the bonus orb is still unspent (regular orbs are spent first; the orb is spent last). Activate → cancel → activate in one turn is legal; once the orb is spent, `bonus_pp` is not legal again that turn. `legal` lists `bonus_pp` in both the activatable and the cancellable state.
- **`traits`** is the sorted list of the schema's boolean `Traits` flags that are true on the instance: `ambush, aura, bane, barrier, cantAttackFollowers, cantAttackLeader, cantBeDestroyedByAbilities, cantBePlayed, drain, ignoresWard, intimidate, rush, storm, ward`. Keyword and mechanic names (`lastWords`, `enhance`, `engage`, `countdown`, `spellboost`, `accelerate`, `counter`, …) are never traits.
- **`granted`** is the sorted **set** of trigger tags present on the instance that the printed card does not carry — runtime grants only — and is omitted when empty. Tags are the schema's trigger names: `fanfare, lastWords, evolve, superEvolve, anyEvolve, anySuperEvolve, strike, followerStrike, clash, enter, leave, discarded, invoked, fused, spellboost, engage, startOfTurn, endOfTurn, when, enhance`. A grant of an ability the card already prints (a second Last Words on a printed Last Words follower) is invisible under this definition; accepted for M1.
- **Picks for every card that leaves the deck.** A card that leaves the deck by a draw is a `draw` pick; one that leaves by any other effect (a search, a summon from the deck) is `{"what":"multiset_pick","among":"deck","chose":"<card id>"}`, one per card in engine order. Invoke names its card and records nothing. Filtered draws ("draw a follower") are `draw` picks whose `chose` must be among the matching candidates.
- **`raw` picks** are engine-private (the old engine's shuffles) and are ignored by every reader.
- **Header extensions.** Header keys prefixed `x_` are engine-private and ignored by readers (`x_final_hash`); every other header key is the format.
- **Cemetery.** `cemetery` holds every card that went there — destroyed followers and amulets and played spells alike; `shadows` is the separate counter the rules spend.
