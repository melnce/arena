# Engine API (M1 contract)

Not compiled. M1 implements these signatures. Types are Rust-flavoured documentation.

## Crate layout

- `arena-engine` (this workspace member): headless, no `web-sys` / `js-sys` / `wasm-bindgen`. CI greps `cargo tree -p arena-engine -e normal`.
- `wasm/` (M4): wraps the engine for the thin client.
- `py/` (M5): PyO3 wrapper for the bot.

M1 obligations (not done here):

- Serde types for the card schema with `#[serde(deny_unknown_fields)]`, tagged enums matching the discriminators (`kind`, `on`, `op`, `pick`, …).
- A test that loads every `cards/**/*.json` and accepts it under both the JSON Schema and the Rust types.
- The same test in TypeScript if TS types are hand-mirrored.
- A benchmark harness: games/second for both engines on the same seeded games. **No multiplier is quoted before that measurement.**

## State

State is flat and index-addressed.

```text
struct Slot(u8);          // 0..4, entry order, compacted after leaves
struct HandIndex(u8);     // draw order
struct PlayerId { A, B }

struct CardInstance {
    card: CardId,                 // Cygames id
    cost: i32,                    // effective
    base_cost: i32,
    attack: i32,
    defense: i32,
    max_defense: i32,
    evolved: bool,
    super_evolved: bool,
    traits: Traits,
    granted: Vec<Ability>,        // runtime grants
    flags: InstanceFlags,         // was_fused, fused_kinds (α), ambush remaining, …
    vars: BTreeMap<VarKey, i32>,  // X, Y, Z
    skybound: i32,                // per card in hand; 0 elsewhere
    countdown: Option<i32>,
    attacks_left: i32,
    can_attack: bool,
}

struct PlayerState {
    leader_defense: i32,
    leader_max: i32,
    pp: i32,
    pp_max: i32,
    bonus_pp: i32,
    ep: i32,
    sep: i32,
    evolves_used: i32,
    shadows: i32,
    combo: i32,
    earth: i32,
    faith: i32,
    rally: i32,
    leader_mods: Vec<LeaderMod>,
    crests: Vec<CrestInstance>,   // id + countdown; cap 5 shared with Faith
    hand: Vec<CardInstance>,
    field: [Option<CardInstance>; 5],
    deck: Multiset<CardInstance>, // unordered
    cemetery: Multiset<CardInstance>,
    banished: Multiset<CardInstance>,
}

struct State {
    players: [PlayerState; 2],
    turn: u32,
    active: PlayerId,
    phase: Phase,
    winner: Option<PlayerId>,
    rng: Rng,                     // inside the state
}
```

No object identity is load-bearing. `clone()` is a deep copy of a few `Vec`s. Per-instance data lives on `CardInstance`.

**Deck.** A draw is a uniform random pick from the multiset, never "the top card". Cards returned to deck keep modifiers (official Q&A, Way of the Maid `10021310`).

## RNG

```text
trait Rng {
    fn gen_range(&mut self, n: u32) -> u32;
}

fn reseed(state: &mut State, seed: u64);
```

**Algorithm (M1):** [xoshiro256**](https://prng.di.unimi.it/) (Blackman & Vigna), stored inside `State`. A `u64` seed is expanded with SplitMix64 into the four 64-bit words:

```text
splitmix64(z):
  z += 0x9E3779B97F4A7C15
  z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
  z = (z ^ (z >> 27)) * 0x94D049BB133111EB
  return z ^ (z >> 31)

seed(s):
  s0,s1,s2,s3 = splitmix64 four times starting from s
  if all-zero, set s0 = 1

next():
  result = rotl(s1 * 5, 7) * 9
  t = s1 << 17
  s2 ^= s0; s3 ^= s1; s1 ^= s2; s0 ^= s3
  s2 ^= t
  s3 = rotl(s3, 45)
  return result
```

`gen_range(n)` (n > 0) uses rejection sampling on the high 32 bits of `next()` so the distribution is uniform. A clone is a complete fork; a replay from a clone is bit-identical.

`ScriptedRng` consumes the trace's `rng` array **by outcome**. Each game-level decision (`draw`, `random_target`, `random_card`, `random_unused`, `random_split`, `coin`, `reanimate`, `multiset_pick`) matches `chose` against the current candidate list; a miss is `OraclePickNotLegal`. The live generator is not advanced in scripted mode. Entries with `"what":"raw"` (old-engine shuffles) are ignored before the array is fed to `ScriptedRng`.

Opening-hand draws are **not** `rng` picks. When `GameConfig.opening_hands` is set (replay of a header that carries `opening_hands`), `new_game` removes those card ids from the deck multisets in the listed draw order and does not roll. Live `arena-trace` still rolls the opening four, then writes them onto the header.

The random-legal *policy* (`arena-trace`, soak) uses a **separate** xoshiro256** stream (`policy_rng(seed)` = xoshiro256** seeded with `seed + 0xA5A5_A5A5_A5A5_A5A5`) so the trace's `rng` array contains only the game's rolls.

No global RNG.

## Construction

```text
struct GameConfig {
    seed: u64,
    deck_a: Multiset<CardId>,   // 40
    deck_b: Multiset<CardId>,
    first: First,               // Coin | A | B
    opening_hands: Option<OpeningHands>,  // pre-mulligan draw-order ids; skip RNG draws
}

fn new_game(db: &CardDb, cfg: GameConfig) -> Result<State, LoadError>;
fn legal_actions(db: &CardDb, state: &State) -> Vec<Action>;
fn apply(db: &CardDb, state: &mut State, action: Action) -> Result<Vec<Event>, Illegal>;
```

M1 takes `&CardDb` on the three verbs so `State` has no lifetime. Missing / unsupported cards fail at `new_game` / `CardDb::require_supported` with a typed error that names the file or `Unsupported { card, construct }`.

**Faith at match start.** A Faith crest is granted to every player whose starting deck (or opening hand) contains a card that carries that Faith — the rulebook's Sham-Nacha rule ("active while … is in your deck"). No card effect ever `crest {gain}`s a `faith:` id. Sathanid / Yidmetra (`10614120`, `10624120`) `grantAbility` onto the player's own Faith (`zone: crests`, `kind: faith`).

**Skybound Art** is per card in hand (turns + evolves while that copy is in hand + Tsubasa boosts), stored on the hand `CardInstance`, not as a player-level gauge.

## Actions

```text
enum Action {
    MulliganConfirm { swap: [bool; 4] },
    Play { hand: u8 },
    Attack { attacker: Slot, target: AttackTarget },  // Slot | Leader
    Evolve { slot: Slot, super_evolve: bool }, // NeutralAction JSON key remains "super"
    Engage { slot: Slot },
    Fuse { host: u8 },            // then Phase::Choice for partners
    BonusPp,
    Choose(u8),
    Confirm,
    EndTurn,
}

fn legal_actions(state: &State) -> Vec<Action>;
// deterministic, stable order, never executes effects
// empty iff terminal

fn apply(state: &mut State, action: Action) -> Result<Vec<Event>, Illegal>;
```

Mulligan is an action like any other. Undo/redo is not an engine concept (a client keeps clones).

While an effect waits for a player choice:

```text
enum Phase {
    Mulligan { player: PlayerId },
    Main,
    Combat,
    Choice { player: PlayerId, node: ChoiceNode },
    End,
    Terminal,
}

enum ChoiceNode {
    Targets { options: Vec<TargetOpt> },   // already computed
    Modes { options: Vec<u8> },
    Cards { options: Vec<CardId> },
    FusePartners { host: u8, options: Vec<u8> },
    MultiPick { options: Vec<ChoiceOpt>, picked: Vec<u8> },
}
```

Legal actions in `Choice` are exactly `Choose(i)` over that list, plus `Confirm` on multi-pick nodes.

## Events

`apply` returns the event log for the trace and the UI. Closed list (M1 may add variants that a pool card forces; each addition is a schema/docs change):

```text
enum Event {
    Draw { player: PlayerId, card: CardId },
    Mulligan { player: PlayerId, swapped: Vec<CardId> },
    Play { player: PlayerId, card: CardId, form: PlayForm },
    Summon { player: PlayerId, card: CardId, slot: Slot },
    Enter { slot: Slot },
    Damage { target: Target, amount: i32, lethal: bool },
    Restore { target: Target, amount: i32 },
    Destroy { slot: Slot, card: CardId },
    Banish { card: CardId, from: Zone },
    Transform { slot: Slot, into: CardId },
    Evolve { slot: Slot, super: bool, granted: bool },
    TriggerFired { source: Source, on: TriggerTag },
    ChoiceOffered { player: PlayerId, node: ChoiceNode },
    RandomPick { what: PickWhat, chose: PickOut },
    Counter { key: CounterKey, value: i32 },
    CrestGain { player: PlayerId, id: CrestId },
    CrestRemove { player: PlayerId, id: CrestId },
    Fuse { host: CardId, partners: Vec<CardId> },
    TurnStart { player: PlayerId, turn: u32 },
    TurnEnd { player: PlayerId },
    Win { player: PlayerId },
}
```

## Hash, snapshot, encode

```text
fn hash(state: &State) -> u64;

fn snapshot(state: &State) -> CanonicalState;   // docs/trace-format.md

fn encode(state: &State, perspective: PlayerId) -> Features;
// signature and intent only — M5. The engine keeps perfect information;
// the encoder masks. The bot is given the opponent's decklist, not the hand.
```

## Throughput

M1's benchmark reports games/second for both engines on the same seeded games. No speedup factor is claimed in M0.
