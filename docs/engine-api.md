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

`ScriptedRng` consumes the trace's `rng` array **by outcome**. Each game-level decision (`draw`, `random_target`, `random_card`, `random_unused`, `random_split`, `coin`, `reanimate`, `multiset_pick`) matches `chose` against the current candidate list; a miss is `OraclePickNotLegal`. For `random_target`, candidates are labeled by surviving-board index (E36); a raw field slot is accepted as an alias when that label is not already taken. The live generator is not advanced in scripted mode. Entries with `"what":"raw"` (old-engine shuffles) are ignored before the array is fed to `ScriptedRng`.

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
    BonusPp,                  // toggle: activate / cancel while the bonus orb is unspent (pp_bonus > 0)
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

fn encode(state: &State, perspective: PlayerId) -> Observation;
fn search_key(state: &State) -> u64;
fn determinize(state: &State, perspective: PlayerId, seed: u64) -> State;
```

The engine stays perfect-information. `encode` masks. The bot is given the
opponent's decklist, not the hand (`rules/owner-rulings.md` Hidden information
— 2026-08-13). `snapshot` / `hash` are unchanged (FNV-1a 64 of the canonical
snapshot; they omit hidden state).

## ActionId (M5)

A total, fixed table of every `Action` the engine can return, independent of
the state. `ActionId::COUNT == 114`. `from_action` / `to_action` are inverses
on every legal action. `legal_mask(db, state) -> [bool; 114]` is the bitset of
`legal_actions`; `legal_ids` preserves `legal_actions` order.

| Range | Kind | Count | Encoding |
|---|---|---|---|
| 0–15 | `MulliganConfirm { swap }` | 16 | `swap` as a 4-bit mask (`bit i` = card `i`) |
| 16–24 | `Play { hand }` | 9 | `hand` in `0..HAND_LIMIT` |
| 25–54 | `Attack` | 30 | 5 attackers × 6 targets (enemy slots 0–4, then leader) |
| 55–64 | `Evolve` | 10 | 5 slots × {evolve, super} |
| 65–69 | `Engage` | 5 | slot 0–4 |
| 70–78 | `Fuse { host }` | 9 | `host` in `0..HAND_LIMIT` |
| 79 | `BonusPp` | 1 | |
| 80–111 | `Choose(i)` | 32 | `i < 32` (asserted: no `ChoiceNode` in a 20k-state sample exceeded 32) |
| 112 | `Confirm` | 1 | |
| 113 | `EndTurn` | 1 | |

```text
fn ActionId::from_action(action: &Action) -> ActionId;
fn ActionId::to_action(self) -> Action;
fn legal_mask(db: &CardDb, state: &State) -> [bool; ActionId::COUNT];
fn legal_ids(db: &CardDb, state: &State) -> Vec<ActionId>;
```

## Observation (M5)

`encode(state, perspective) -> Observation` with `features: Vec<f32>` of length
`Observation::LEN` (545) and a parallel `ids: Vec<u32>` of length
`Observation::IDS_LEN` (220). Card ids stay ids (embedding inputs for M5b);
they are never one-hot over the pool.

What `perspective` may see: own hand (ids, current cost, spellboost, fused /
can't-play / once-used flags, skybound), own deck as a multiset of ids (order
is never information), own crests / counters / PP / EP / SEP / leader; the
opponent's board, crests, counters, PP / EP / SEP, leader; the opponent's
**hand size** and **deck size**; the opponent's **known remaining pool** =
starting decklist − public-zone cards + tokens still in hand/deck. The
opponent-hand id region is always `0`.

`PlayerState` stores snapshot-neutral `starting_deck`, `public_removals`, and
`public_hand_additions` (not in `CanonicalState`).

`Observation::LAYOUT` — name, offset, width:

| Name | Offset | Width |
|---|---|---|
| `phase` | 0 | 5 (mulligan, main, choice, end, terminal) |
| `turn` | 5 | 1 |
| `i_am_active` | 6 | 1 |
| `winner` | 7 | 1 (+1 me / −1 opp / 0) |
| `first_is_me` | 8 | 1 |
| `choice_ids` | 9 | 32 (`Choose(i)` offered) |
| `me_scalars` | 41 | 29 |
| `opp_scalars` | 70 | 29 |
| `own_hand_cost` | 99 | 9 |
| `own_hand_spellboost` | 108 | 9 |
| `own_hand_skybound` | 117 | 9 |
| `own_hand_fused` | 126 | 9 |
| `own_hand_cant_play` | 135 | 9 |
| `own_hand_once_used` | 144 | 9 |
| `own_board` | 153 | 100 (5 × 20: atk/def/max/evo/super/traits/cap/attacks/once/seq/bound/kind) |
| `opp_board` | 253 | 100 |
| `own_deck_hist` | 353 | 96 |
| `opp_known_pool_hist` | 449 | 96 |

Ids: own hand 9, own-deck vocab 96, opponent board 5, own board 5, opponent
known-pool vocab 96, opponent hand 9 (always 0). Histograms are counts over
the sorted union of both starting decklists plus visible token ids, padded
to `HIST_WIDTH = 96`.

## search_key (M5)

`search_key(state) -> u64` is FNV-1a 64 over a hand-rolled little-endian
canonical walk of every `State` field **except the RNG**. It differs when
`choose_used`, `once_used`, hand-zone once-per-turn flags, or the mulligan
actor differ; clones match; RNG-only reseeds match. Not interchangeable with
`hash` (the snapshot hash is pinned by the oracle corpus).

## determinize (M5)

`determinize(state, perspective, seed) -> State` clones, reseeds `rng` from
`seed`, and resamples the opponent's hand and deck from their known remaining
pool: hand size preserved, public tokens kept in hand, the rest drawn
uniformly, leftover pool becomes the deck. Own side is untouched.
`encode(determinize(s), p) == encode(s, p)`.

## Policy (M5)

```text
trait Policy {
    fn choose(&mut self, db: &CardDb, state: &State, legal: &[Action], rng: &mut Xoshiro256ss) -> usize;
}

fn policy::by_name(name: &str, seed: u64) -> Option<Box<dyn Policy>>;
fn policy::names() -> &'static [&'static str];   // "random", "first-legal", "h0"
```

`Policy` is object-safe. `policy/` (and `encode`, `search_key`, `determinize`)
compile for `wasm32-unknown-unknown`: no `Instant` / `SystemTime`, threads,
`std::fs`, or `getrandom`. The node cap is the only search budget. `seed` is
accepted at construction; current policies do not store it — `choose` uses
the caller rng (typically `policy_rng(seed)`).

`Random` and `FirstLegal` are the arena-bench / arena-trace policies (same
streams and output as before). `H0` is a determinized search bot:

| Parameter | Default | Meaning |
|---|---|---|
| `depth` | 6 | own-turn actions before cutoff |
| `beam` | 4 | top-k by value each ply |
| `determinizations` | 4 | opponent-reply samples |
| `node_cap` | 2000 | `apply` calls per decision (budget ≈ 2 ms) |

H0 evaluates every legal action on clones, searches its own turn to
`EndTurn` (beam + depth), then the opponent's reply via a **greedy value
maximiser** on each determinization (not a nested H0 — a depth-2 H0 × 4
samples would spend the node cap on the opponent and miss own-turn lethal).
Value: leader-defense difference, board (atk+def with Ward/Storm/evolved
weights), hand size, next-turn PP / EP / SEP, crest / countdown presence;
terminal = ±∞. Lethal lines on the current turn are taken first.

`arena-bench` accepts `--policy random|first-legal|h0` and `--vs` for
asymmetric seats. Caps: `engine::limits::{MAX_TURNS, MAX_ACTIONS}` = 60 / 800.

## Throughput

M1's benchmark reports games/second for both engines on the same seeded games. No speedup factor is claimed in M0.

## Bindings (`wasm/`, M4)

JSON-neutral JS API over `arena-engine` compiled to WASM. The client never depends on Rust types — every method takes or returns a JSON string (or a thrown string). `apply` accepts a `NeutralAction` exactly as `arena-replay` reads it (`from_neutral` / `apply_neutral`). Undo/redo is not an engine concept; the client keeps `clone()` snapshots.

JSON strings (not `serde-wasm-bindgen` JS objects) so the wire format is the same as `docs/trace-format.md` and a JS client can replay traces without a type mapping.

The WASM binary `include_str!`s every authored `cards/**/*.json` except `cards/official/`, produced by `wasm/build.rs`. The engine crate itself gains no wasm dependency.

| Method | Returns | Notes |
|---|---|---|
| `new Game(seed, deckA, deckB, first)` | `Game` | `seed` is a bigint or number; `deckA` / `deckB` are JSON multisets `{id: n}`; `first` is `"coin"` \| `"a"` \| `"b"` |
| `legal()` | string | JSON array of `NeutralAction`, same order as `legal_actions` |
| `apply(action)` | string | JSON array of `Event`; throws a string naming the action and `legal` length on `Illegal` |
| `snapshot()` | string | `CanonicalState` JSON (`snapshot_json`) |
| `full()` | string | the full `State` as JSON (perfect information; the client masks nothing yet) |
| `hash()` | string | `u64` as a decimal string |
| `phase()` | string | `"mulligan"` \| `"main"` \| `"choice"` \| `"end"` \| `"terminal"` |
| `clone()` | `Game` | deep copy for undo/redo |
| `free()` | void | wasm-bindgen drop |
| `cardText(id)` | string | JSON `{id, name, text, kind, cost}` from the baked bundle |
| `bundleInfo()` | string | `{cards, crests, bytes}` |
| `version()` | string | git SHA baked at build, or `"dev"` |

## Python (M5 foundation)

`py/` is a PyO3 `cdylib` (`arena-py`, imported as `arena`). JSON at the
boundary; dicts on the Python side. `matchup` and `play_random` run whole
games in Rust — the Python boundary is not on the per-action path.

`CardDb` is loaded from the filesystem (`CardDb::load`; `root` may be the
repo root or the `cards/` directory). `new_game` still `require_supported`s
every deck id. `State: Send` and `CardDb: Sync` so rayon can share one db.

| API | Notes |
|---|---|
| `arena.load_cards(root: str = "cards") -> CardDb` | Filesystem load, serde-validated. |
| `Game(db, seed, deck_a, deck_b, first="coin", opening_hands=None)` | `deck_*` are `{id: count}`; `opening_hands` is the trace header shape. |
| `Game.legal() -> list[dict]` | `NeutralAction` dicts, engine order. |
| `Game.apply(action, rng=None) -> list[dict]` | Event dicts. `rng` is the trace line's pick array (replay). Raises `arena.Illegal(str)`. |
| `Game.snapshot() -> dict` | `CanonicalState` (`docs/trace-format.md`). |
| `Game.full() -> dict` | Perfect-information dump of public `State` fields (instance ids, flags, ordered zones). |
| `Game.hash() -> int` | FNV-1a 64 of the canonical snapshot JSON. |
| `Game.phase` / `active` / `turn` / `winner` / `terminal` | `"mulligan"\|"main"\|"choice"\|"end"\|"terminal"`; `"a"/"b"`; `winner` is `str \| None`. |
| `Game.clone() -> Game` | Deep copy of `State` (search). |
| `arena.play_random(db, seed, deck_a, deck_b, first="coin") -> dict` | `{winner, turns, actions, first}`. Random-legal + `policy_rng`. |
| `arena.matchup(db, decks, games, seed, policy="random", threads=None) -> dict` | Every ordered pair including mirrors. Per pair `{games, a_wins, b_wins, first_player_wins, mean_turns, mean_actions}`. Seed per game = FNV-1a64 of `(seed, pair_index, game_index)`. `policy` is `"random"` (arena-bench random-legal) or `"first-legal"`. Deterministic for a given seed and thread count. |

`py/matchup.py` loads `oracle/decks/*.json` and prints the win-rate matrix.
