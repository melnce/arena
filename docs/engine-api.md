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
fn determinize_with(state: &State, perspective: PlayerId, seed: u64, info: Info) -> State;
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
to `HIST_WIDTH = 96`. `vocab(state)` returns that sorted union (capped at
96). `encode_with_vocab(state, perspective, vocab)` is the same walk with
the vocabulary supplied by the caller and binary-search histogram lookups;
`encode` is `encode_with_vocab(state, p, &vocab(state))`.

## search_key (M5)

`search_key(state) -> u64` is FNV-1a 64 over a hand-rolled little-endian
canonical walk of every `State` field **except the RNG**. It differs when
`choose_used`, `once_used`, hand-zone once-per-turn flags, or the mulligan
actor differ; clones match; RNG-only reseeds match. Not interchangeable with
`hash` (the snapshot hash is pinned by the oracle corpus).

## determinize (M5)

`determinize(state, perspective, seed) -> State` is the default
(`Info::Draws`) path: clone, reseed `rng` from `seed`, and resample the
opponent's hand and deck from their known remaining pool: hand size
preserved, public tokens kept in hand, the rest drawn uniformly,
leftover pool becomes the deck. Own side is untouched.
`encode(determinize(s), p) == encode(s, p)`.

`determinize_with(state, perspective, seed, info)` is the same entry
point with an explicit information regime. `info` governs what the
**search** simulates, not what the **leaf** observes: `encode` is
unchanged and still masks the opponent's hand. That separation is
deliberate.

| `info` | own deck | opponent hand/deck | who has this |
|---|---|---|---|
| `fair` *(default)* | resampled (hand untouched) | resampled | a human with open decklists |
| `draws` | exact order known | resampled | pre-flip path — nobody, really |
| `all` | exact order known | not resampled | a hard-mode sparring bot |

## Policy (M5)

```text
trait Policy {
    fn choose(&mut self, db: &CardDb, state: &State, legal: &[Action], rng: &mut Xoshiro256ss) -> usize;
    fn last_value(&self) -> Option<f32> { None }
}

fn policy::by_name(name: &str, seed: u64) -> Option<Box<dyn Policy>>;
fn policy::names() -> &'static [&'static str];   // "random", "first-legal", "h0"

enum End { Lethal, Deckout, TurnCap, ActionCap, NoLegal, Illegal }
struct Outcome { winner: Option<PlayerId>, first: PlayerId, turns: u32, actions: u32, end: End }
fn play_game(db: &CardDb, state: &mut State, pol_a: &mut dyn Policy, pol_b: &mut dyn Policy, rng: &mut Xoshiro256ss) -> Outcome
```

`play_game` is the shared drive loop (bench, Python matchup, m5a fixtures).
It stops when `winner` is set or the phase is `Terminal`; `state.turn >
MAX_TURNS` → `TurnCap`; `actions >= MAX_ACTIONS` → `ActionCap`; empty
`legal_actions` → `NoLegal`; `apply` error → `Illegal`. The chosen index is
clamped. Both seats draw from the caller rng (typically
`policy_rng(game seed)` — the same stream the bench uses). `Lethal` is the
loser's `leader_defense <= 0`; otherwise a decided game is `Deckout`. No
`Instant`, threads, or `std::fs` in `play/` — it builds for
`wasm32-unknown-unknown` with the rest of `policy/`.

`AnyPolicy::parse_spec` is strict (a typo never silently becomes
`Random`):

```text
"random" | "first-legal" | "h0" | "h0-fast"
| "h0:depth=6,beam=4,k=4,nodes=2000"
```

Any subset of the H0 keys; omitted keys take [`H0::default`].
`k` = `determinizations`, `nodes` = `node_cap`. `"h0"` is
`H0::default()`; `"h0-fast"` is `H0::fast()`. Extra keys: `value=v0|v1|net`
(default `net` = the built-in `h0-linear-v1`; `value=v0` is the
hand-written leaf the bot used before this default), `net=<path>`
(overrides the built-in; only meaningful with `value=net`;
`h0:net=<path>` alone means `h0:value=net,net=<path>`; the path
may not contain commas), `odepth=` / `obeam=` (opponent model; defaults `0` / `3`),
`olethal=0|1` (cheap opponent-lethal sweep on the greedy path; default `1`;
`olethal=0` restores the pre-flip greedy path; ignored when `odepth≥1`),
`oevo=0|1` (extend that sweep with one evolve; default `1` is the
sweep-8b flip; `oevo=0` restores the pre-flip glance path; only
meaningful with `olethal=1` and `odepth=0`; no hard error for other
combinations),
`osteps=<u32>` (greedy forced-`EndTurn` step;
default `6`; hard stop is `osteps+3`),
`wv=<f32>` (saturation bound on every accumulated value; default `80`),
`pess=<f32>` (pessimism weight on the root mean, in `[0, 1]`; default `0`
is today's mean; `1` is the worst determinization),
`tt=0|1` (per-decision transposition table; default `1`),
`alloc=root|fair` (how the node cap is spent across `(root, candidate)`
pairs; default `fair` = per-pair share; `alloc=root` restores the
pre-#46 root-major spend),
`info=fair|draws|all` (what the search is allowed to know; default
`fair` = own deck resampled (hand untouched), opponent resampled —
a human with open decklists; `draws` restores the pre-flip path
(own draw order exact); `all` is the true state — no resampling —
and builds one root regardless of `k`; any other value is a parse
error naming `info` and listing the three),
and `w_shadows=`, `w_earth=`, `w_faith=`, `w_rally=`,
`w_boost=`, `w_need=`, `w_lw=` (f32; only meaningful with `value=v1`;
`0` disables a term). `Err` names the offending token. `AnyPolicy::spec`
is the canonical form (`"h0:depth=…,beam=…,k=…,nodes=…"` plus `value=v0` /
`value=v1` or `value=net,net=<path>` — the built-in net is not printed,
non-default `odepth` / `obeam`, `olethal=0` / non-default `osteps` when set,
`oevo=0` when the evolve branch is off, non-default `wv`, non-default `pess`, `tt=0` when the table is off, `alloc=root` when the
allocator is the pre-#46 root-major spend, `info=draws` / `info=all`
when the information regime is not the default `fair` (same shape as
`alloc=root`), and any
non-default weight, or the short names). `"h0"` still round-trips to `"h0"`. `by_name` is
`parse_spec(name).ok()`; `names()` stays
`["random", "first-legal", "h0"]` so the WASM client's bot list does not
change. The client's `h0` now plays with the learned leaf.

`Policy` is object-safe. `last_value` is the value the policy computed
for the position at its most recent `choose`, from the acting player's
perspective, on the leaf's scale (`[-wv, wv]`); `None` when it did not
search. The trait default is `None` (`Random` / `FirstLegal`); `H0`
stores it in `H0.last_value` (reset every `choose`; ignored by
`h0_fields_eq` / `spec()`); `AnyPolicy` forwards to the inner policy.
Reading it never alters the search. `policy/` (and `encode`, `search_key`,
`determinize`, `play`) compile for `wasm32-unknown-unknown`: no `Instant`
/ `SystemTime`, threads, `std::fs`, or `getrandom`. The node cap is the
only search budget. `seed` is accepted at construction; current policies
do not store it — `choose` uses the caller rng (typically
`policy_rng(seed)`).

`Random` and `FirstLegal` are the arena-bench / arena-trace policies (same
streams and output as before). `H0` is a determinized search bot:

| Parameter | Default | Meaning |
|---|---|---|
| `depth` | 6 | own-turn actions before cutoff |
| `beam` | 4 | top-k by value each ply |
| `determinizations` | 4 | opponent-reply samples |
| `node_cap` | 2000 | `apply` calls per decision (budget ≈ 2 ms) |
| `value` | `net` (built-in `h0-linear-v1`) | leaf evaluator; `v0` is the hand-written leaf; `v1` adds economy terms |
| `net` | — | path to a `net.json` overriding the built-in model |
| `w_shadows` | 0.12 | v1: saturated shadows (cap 10) |
| `w_earth` | 0.35 | v1: saturated earth sigils (cap 6) |
| `w_faith` | 0.15 | v1: saturated faith (cap 10) |
| `w_rally` | 0.05 | v1: saturated rally (cap 15) |
| `w_boost` | 0.10 | v1: spellboost counters on Spellboost cards in hand |
| `w_need` | 0.60 | v1: “is this card live yet” over hand thresholds |
| `w_lw` | 0.80 | v1: Last Words followers on the field |
| `odepth` | 0 | opponent-model action depth; `0` = greedy line |
| `obeam` | 3 | opponent beam (plus `EndTurn` always) |
| `olethal` | 1 | glance-level opponent-lethal sweep before the greedy line; `0` restores the pre-flip greedy path. Ignored when `odepth≥1` |
| `oevo` | 1 | extend that sweep with at most one `Evolve` / `super_evolve` per leaf. Only meaningful with `olethal=1` and `odepth=0`. Sweep 8b pooled 0.527 [0.515, 0.540] / +19.0 Elo vs `h0:olethal=1,osteps=6` on the seven real decks (6 174 games). Owner flipped the default on 2026-09-19 |
| `osteps` | 6 | greedy-line steps before a forced `EndTurn`; hard stop is `osteps+3` (default 9) |
| `wv` | 80 | saturation bound on every accumulated value (`finite` clamps to ±`wv`); a detected opponent lethal returns exactly `-wv` |
| `pess` | 0 | pessimism weight on the root aggregation: `(1-pess)*mean + pess*worst` over the K determinizations. `0` is today's mean (that path is the existing expression, not a blend). No default changed; a flip needs the owner's yardstick |
| `tt` | 1 | per-decision transposition table; `0` restores the pre-#32 search |
| `alloc` | `fair` | budget spend across `(root, candidate)` pairs; `fair` = per-pair share so every candidate is scored on every determinization; `root` = pre-#46 root-major (later pairs skipped when the cap binds) |
| `info` | `fair` | what the search is allowed to know. `fair` (default) = resample the perspective player's own deck (hand untouched) and the opponent's hand/deck — a human with open decklists. `draws` = own draw order exact, opponent resampled (the pre-flip path). `all` = no resampling; the search rolls out against the opponent's real hand. Under `all`, H0 builds **one** root regardless of `k` (every determinization would be identical). `info` is a search-time knob; `encode` still masks the opponent's hand at the leaf. Sweep 8b pooled 0.513 [0.500, 0.525] / +9.0 Elo vs `h0:olethal=1,osteps=6` on the seven real decks (6 174 games). Owner flipped the default on 2026-09-19 |

H0 builds `K = max(1, determinizations)` search roots via
`determinize_with(state, me, seed, info)` (which reseeds the game RNG)
from the policy rng. Under `info=all` every root would be identical,
so K is 1 regardless of `determinizations` — do not silently do
`k` times the work for one tree. Own-turn search, lethal, and the
opponent model all run on those roots — the true hidden hand and live
game RNG are never read, except under `info=all` where the opponent
hand *is* the true hand. `encode` is unchanged and still masks the
opponent's hand; `info` governs the search, not the leaf. A lethal is taken only when every root agrees (a random lethal is
a bet, not a lethal). Candidate values are the mean over the K
determinizations (`acc[j] / n[j]`). `pess` blends that mean with the
worst `finite` sample: `v = (1-pess)*mean + pess*worst`. At `pess=0`
(the default) the blend is not taken — the expression is still
`acc[j] / n[j]`, bit-identical to today. A detected opponent lethal
is pinned at `-wv` (the clamp floor) and can be averaged away: with
`k=4` and `wv=80`, one dead root and three +60 samples score
`(−80+180)/4 = +25`, which beats a safe +10. `pess` is the knob that
weights the dead sample harder. This PR does not flip the default;
a flip would need the owner's yardstick.
The node cap is global. `H0::fast()` uses `K = 1` and a 1-ply value
on that root (no depth-2 consensus-lethal walk). `h0-fast` keeps `tt=0`,
`value=v0`, `olethal=0`, and `osteps=3` so the cheap test baseline stays
byte-identical to the pre-flip search (the TE precedent: `h0-fast` keeps
`tt=0`; unlike `alloc`, `olethal` / `osteps` are reachable at depth 2).

The opponent model is a switch. `odepth=0` (default) is the historical
greedy line: at most `osteps+3` steps (default nine), and **EndTurn after
`osteps`** (default six) whenever `EndTurn` is legal. `odepth≥1` replaces that with a depth-limited beam
over the opponent's actions, ranked by the opponent's leaf value (the
value is antisymmetric, so maximising theirs minimises mine). Each ply
keeps the `obeam` best actions **and always `EndTurn`**, so "do nothing
more" is a considered line rather than a hard 3-step cutoff. If any
explored opponent line reaches `winner == opponent`, the model returns
`-wv` immediately (the opponent is assumed to find its lethal).
A mid-turn choice handed to me is resolved with a 1-ply greedy pick from
my perspective, then the opponent's line continues. The node cap is
shared with the own-turn search and is not raised by this switch.

When `olethal=1` (the default) and `odepth=0`, `opponent_reply` runs a bounded lethal
sweep from the opponent's side *before* the greedy line. The sweep asks
one question — can the opponent kill my leader this turn with lines a
human would see at a glance — and does it with few applies: (1) a **face
line** — clone the state and, while a legal `Attack { target: Leader }`
exists, apply the first one (attacks to the leader commute, so one fixed
order suffices); if my leader dies, that is lethal; (2) **play-then-face**
— for each legal `Play` from the original determinized opponent hand,
apply it, depth-first branch any pending `Choose` in index order, apply
any required `Confirm`, and if my leader died that is lethal; otherwise
run the face line from there only if the play produced a new legal
leader attack or lowered my leader defense. `Evolve`, `Engage`, `Fuse`,
and `BonusPp` lines are skipped when `oevo=0` (rare lethal sources; the
greedy line still uses them). The whole sweep spends at most 40 `apply`s
per leaf when `oevo=0`, all charged to `nodes` through `try_apply`, and
stops early at the cap (cycle guard as today). If it finds lethal the
leaf is `-wv` and the greedy line is skipped; otherwise the greedy line
runs with the remaining budget. `olethal=0` restores the pre-flip greedy
path. `odepth≥1` ignores `olethal` — the beam search is the opponent
model.

When `oevo=1` (the default) and the sweep is running (`olethal=1`,
`odepth=0`), two extra branches sit after those glance lines. At most
one evolve is legal per turn (`evolved_this_turn` / `ep`), so the
branching is small; `super_evolve` variants are legal actions like any
other and are not special-cased. (1) **play-then-evolve** — after a play
that already passed the new-face-attacker / lowered-defence test, if the
face line missed, try each legal `Evolve` in the post-play state,
starting with the slot of the follower just played, then the other face
attackers, and run the face line after each; (2) **standalone evolve** —
with no play, for each legal `Evolve`, apply and run the face line.
This is the owner's missed line: play a Storm, evolve it (+2 and any
`Evolve:` ability), attack with it and an existing body. The apply cap
is 120 when `oevo=1` and stays 40 when `oevo=0`. Sweep 8b pooled
0.527 [0.515, 0.540] / +19.0 Elo; the owner flipped the default on
2026-09-19. `oevo=0` restores the pre-flip glance path.

The owner's standing yardstick (`results` branch, `sweep5/SUMMARY.md`,
`b79421a`, engine `f7b0a61`, data seed 6, 16 oracle decks, wall 3 h 38)
flipped the default to the pair:

| candidate | main (4 096) | reverse (2 048) | verdict |
|---|---|---|---|
| **`h0:olethal=1,osteps=6`** | **0.526 [0.511, 0.541]** | **0.538 [0.516, 0.559]** | **better** |
| `h0:olethal=1` | 0.512 [0.497, 0.527] | 0.512 [0.491, 0.534] | coin flip |
| `h0:osteps=6` | 0.517 [0.486, 0.547] screen only | — | not a finalist |
| `h0:odepth=1` / `odepth=1,obeam=5` / `odepth=2,obeam=3` | 0.366 / 0.357 / 0.373 screen | — | skipped |

`olethal=1` alone is noise and `osteps=6` alone is slow and no better;
the pair clears the bar on both seats. Throughput (mirrors, same policy
on both seats): `sweep5/tp-baseline.json` 2.897 g/s vs
`sweep5/tp-c02.json` 2.401 g/s — a clean **1.21× wall-clock per game**,
paid by every future sweep, training round, and yardstick. Scaling from
sweep 4 (`alloc=fair` 0.563 → `alloc=fair,nodes=6000` 0.583, i.e. ≈ +14
Elo for 3× nodes), an equal-wall-clock baseline would recover only
≈ 3 Elo of the flip's ≈ 18–26 Elo — **that is an estimate** across two
sweeps with different seeds and engines. The desktop `h0 (strong)`
option (`ui/src/main.ts` `STRONG_H0 = "h0:nodes=6000"`) and
`py/serve.py --strong` (`h0:nodes=16000`) inherit the new default
automatically; no client edit.

`wv` is the saturation bound on **every** accumulated value, not only a
root-level terminal stand-in: `finite(v, wv)` is `v.clamp(-wv, wv)`,
and both the pair loop and `one_ply` add that clamped number into
`acc[j]`. A detected opponent lethal returns exactly `-wv` and
propagates unchanged, so "dead" is pinned to the floor of the scale.
The in-search terminal stays ±`INF` (`1e9`); only the number that
reaches the root average is clamped. Default `wv=80` is today's clamp
— it sits *inside* the reachable live range of `value_v0` (`4.5 ×`
leader-defense difference is already ±90, plus `board_score` of every
follower). A live position at −95 (behind on life and board, but
alive) therefore clamps to −80, the same score as a lost root, so "I
survive this turn at 3 life" and "I am dead" are indistinguishable,
and a sure next-turn win (+95) equals a lucky lethal (+80).
Candidates: `wv=300` sits just above the live range; `wv=1000` makes
one lethal root out of four outvote three bad live roots. This PR
does not flip the default.

`choose` spends the node cap root by root and candidate by candidate
(`for root in roots { for candidate in subset { … search_own } }`).
With `alloc=root` a single candidate's subtree can consume
thousands of applies at depth 6 / beam 4, so when the cap binds the
later candidates on that root are never evaluated (`n[j] == 0` → they
cannot be chosen) and later determinizations are skipped. `alloc=fair`
(the default) keeps the same pair order but gives each remaining pair a
share of the leftover budget: `share = max(24, remaining / pairs_left)`,
`pair_cap = min(node_cap, nodes + share)`. A pair that finishes under
its share hands the rest to the following pairs; a pair that would
exceed it is cut inside `search_own` exactly as the global cap cuts
today. Only when the global budget is exhausted is a pair skipped.
`alloc=root` restores the pre-#46 root-major spend.

The owner's standing 4 096-game yardstick (`results` branch,
`sweep4/SUMMARY.md`, seed 4, engine `a17c6c0`, 16 oracle decks, wall
2 h 03) flipped the default:

| candidate | main (4 096 games) | reverse (2 048) | throughput vs `h0` |
|---|---|---|---|
| `h0:alloc=fair` | **0.563 [0.548, 0.579]** | **0.538 [0.516, 0.559]** | **2.89 vs 2.53 g/s** |
| `h0:alloc=fair,nodes=6000` | 0.583 [0.568, 0.598] | 0.552 [0.531, 0.574] | 1.31 g/s |
| `h0:alloc=fair,depth=4` (screen) | 0.560 [0.529, 0.590] | — | 3.05 g/s |
| `h0:alloc=fair,k=8` (screen) | 0.560 [0.529, 0.590] | — | 2.46 g/s |

`alloc=fair` is better by the standing rule on both seats (+6.3 pt)
and 14 % faster than `alloc=root`, because per-pair caps cut subtrees
earlier. The desktop `h0 (strong)` option (`h0:nodes=6000`) and
`py/serve.py --strong` (`h0:nodes=16000`) inherit the new default
automatically; no client edit.

Measured on this box (200 mid-game states, `nodes=2000`): `root`
skipped 3 182 pairs, `fair` skipped 0; node totals 270 854 vs 265 257
(within 5 %). 50-game abyss-p8rfn `--stats`:
`pairs_skipped/decision` 10.53 (`root`) vs 0.04 (`fair`),
`cap_hit_rate` 0.418 vs 0.144. `MIN_SHARE` floors the pair share at 24
when every remaining pair can still receive it; otherwise the leftover
is split evenly so later pairs still get a search. Decisions where
`consensus_lethal` already spent the cap are not counted — neither
allocator had a budget.

When `tt=1`, each `choose` builds an empty
`HashMap<(u64, u8, bool), f32>` keyed by
`(search_key(state), remaining depth, side-to-move-is-me)` and consults
it in `search_own` / `search_opp` before expanding a node. After a
state's value is computed it is stored. The table is dropped at the end
of the decision and is never shared across roots or decisions (the K
determinized roots have different hidden hands and therefore different
keys). A hit does not increment the node cap — that is the point. A
cached value may have been computed under a different cycle-guard
`line`; reusing it is the standard transposition approximation. `tt=1`
is the default. `tt=0` restores the pre-#32 search.

`arena-bench --stats` prints, after the JSON line, one `search-stats`
line per seat with means per decision:

```text
search-stats A h0: decisions=N nodes/decision=… cap_hit_rate=… candidates/decision=… pairs_skipped/decision=… opp_leaves/decision=… opp_cap_hit_rate=… tt_hits/decision=… tt_stores/decision=… opp_lethal_checks/decision=… opp_lethal_found/decision=… opp_lethal_evo_found/decision=… opp_lethal_nodes/decision=… chose_with_lethal_root/decision=… cands_with_lethal_root/decision=…
```

`decisions` is `choose` count; `nodes` are `apply`s; `cap_hit_rate` is
the fraction of decisions that exhausted `node_cap`; `candidates` are
legal actions kept after the Bonus-PP filter; `pairs_skipped` is
`k × |subset| − attempted` (pairs never given a search after
`consensus_lethal` left leftover budget) per decision;
`opp_leaves` /
`opp_cap_hit_rate` describe the opponent model; `tt_hits` / `tt_stores`
are transposition-table lookups that returned a value and writes
(`0` when `tt=0`); `opp_lethal_checks` / `opp_lethal_found` /
`opp_lethal_evo_found` / `opp_lethal_nodes` are sweeps run, lethals
found, lethals found *only* through an evolve branch, and applies spent
by the `olethal` sweep (`0` when `olethal=0`; the default is `olethal=1`;
`opp_lethal_evo_found` stays 0 unless `oevo=1`, which is the default).
`chose_with_lethal_root` is the fraction of searched decisions whose
chosen candidate had at least one determinization at exactly `-wv`
(the clamp floor; compared with a small epsilon).
`cands_with_lethal_root` is the same test summed over every candidate
offered at that decision, so the chosen-vs-available ratio is readable.
Both stay 0 on the `one_ply` / consensus-lethal / no-search paths.
Non-H0 seats print
`decisions=0`.

`BonusPp` is considered only in the **activate** direction
(`!bonus_pp.active`); cancel is never chosen. Cycles are skipped: any
action whose resulting `search_key` is already on the current line is
dropped. Mulligan: swap every card whose cost is ≥ 4 (both seats).

Value: leader-defense difference, board (atk+def with Ward/Storm/evolved
weights), hand size, next-turn PP / EP / SEP, crest / countdown presence;
terminal = ±∞ on a single root, finite-clamped when averaging. That
arithmetic is `value=v0`. The default leaf is the built-in
`h0-linear-v1` net (`value=net`). `value=v1` adds
saturated shadows / earth / faith / rally, spellboost counters on cards
that print Spellboost, a threshold-shaped “live” bonus for hand cards
that pay those resources or check Rally, and a count of Last Words
followers on the field — all from a state-free `NeedsTable` built next
to `when_cards`. Weights are the `w_*` keys.

`arena-bench` accepts `--policy` / `--vs` as `parse_spec` strings (unknown
names exit with the parser message) and drives games through `play_game`.
`--stats` adds the per-seat search-stat lines described above (JSON
fields are unchanged). Seeding is unchanged: `seed.wrapping_add(g)`,
`First::A`, `policy_rng(s)` shared by both seats. Caps:
`engine::limits::{MAX_TURNS, MAX_ACTIONS}` = 60 / 800.

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
| `acting()` | string | `"a"` / `"b"` — whose decision the current phase waits for (`acting_player`) |
| `active()` | string | `"a"` / `"b"` — the player whose turn it is |
| `turn()` | number | round number (`0` during mulligan) |
| `winner()` | string \| null | `"a"` / `"b"` / `null` |
| `botAction(policy, seed)` | string | one `NeutralAction` JSON for the acting player. `policy` is any `parse_spec` string (it always was — `by_name` is `parse_spec`; the `Game` caches a `Box<dyn Policy>` keyed by that string). `seed` is a bigint or number. Unknown specs throw a string. |
| `cardText(id)` | string | JSON `{id, name, text, kind, cost, …}` from the baked bundle. Cards also include `crests` and `forms` (see below). Crest / faith ids omit both (do not recurse). |
| `bundleInfo()` | string | `{cards, crests, bytes}` |
| `version()` | string | git SHA baked at build, or `"dev"` |
| `botPolicies()` | string | JSON array from `engine::policy::names()` — `["random","first-legal","h0"]`. The client lists those plus `h0 (standard)` = `h0` and `h0 (strong)` = `h0:nodes=6000` (desktop default strong, phone default standard, stored as `svwb.botPolicy`). Engine `H0::default()` stays 2 000 nodes: sweep2 measured `h0:nodes=6000` at 0.562 [0.547, 0.577] over 4 096 games (reverse seating 0.553 [0.531, 0.574]; +6.2 pt, both seats) vs current `h0`. |

`cardText` on a collectible / token adds two read-only arrays (no existing field changes):

| Field | Shape | Purpose |
|---|---|---|
| `crests` | `[{ id, name, text, faith, grantedBy }]` | Crests this card grants or is bound to. Authored `{op:"crest", gain}` walk first, then any further crest whose `granted_by` lists this card id (id order). `grantedBy` is that card's 8-digit id (exactly one per crest). Empty `[]` when none. An id that does not resolve in `card_db().crest` is skipped (client hover must not panic); the wasm sweep asserts resolved entries equal walked+granted_by ids. Crest / faith `cardText(id)` also returns `grantedBy` (no nested `crests`). |
| `forms` | `[{ kind: "enhance"\|"accelerate"\|"crystallize", cost, printed }]` | Straight passthrough of `card.modes()` in authored order. The client hides a `printed` line that already appears in `text` (Enhance); Accelerate / Crystallize lines are shown from here. |

Card image hashes are **not** a WASM binding. The M4 client reads `ui/public/catalog-images.json` (generated by `tools/gen-ui-catalog.mjs` from `cards/official/catalog.json`) so a catalog refresh does not require a wasm rebuild. Each entry is `id → {name, cost, class, kind, attack, defense, card, banner, evoCard, evoBanner}`. Image URLs follow `docs/design.md`: `https://shadowverse-wb.com/uploads/card_image/eng/card/<card>.png` (evolved art from `evoCard`), `referrerPolicy="no-referrer"`.

## Local bot server (`py/serve.py`)

A tiny stdlib HTTP server on the owner's PC so the web UI can spend H0's
node budget natively instead of in single-threaded wasm. Phone / laptop
without the server keep today's wasm path. Owner runbook:
`docs/local-bot.md`.

`Game.reseed(seed)` and `Game.bot_action(policy, seed)` on the Python
binding match the wasm methods (`state.reseed`; `by_name` + `policy_rng`
+ `choose` over `legal_actions`, returned as a NeutralAction dict). The
server constructs the policy per call (no cache). Unknown specs raise
`ValueError` naming the policy and `names()`.

Bind `127.0.0.1` only (`--host`, `--port 8765`). `--strong` (default
`h0:nodes=16000`) is the spec used whenever the request's `policy` is an
h0 variant (`h0`, `h0:nodes=6000`, `h0:…`). `random` / `first-legal` and
any non-h0 spec pass through unchanged — the server, not the client,
decides H0 strength. `--origins` is the CORS allow list (default
`https://arena-nu-one.vercel.app,http://localhost:5173,http://127.0.0.1:5173`);
a request whose `Origin` is not on the list is served but gets no
`Access-Control-Allow-Origin`. Allowed origins also get
`Access-Control-Allow-Headers: content-type`. `OPTIONS` is the preflight.
`--cards` defaults to the repo's `cards/` (same walk as `matchup.py`).

| Endpoint | Body / query | Reply |
|---|---|---|
| `GET /health` | — | `{"ok": true, "strong": "<spec>", "cpus": <os.cpu_count()>, "version": "<git short HEAD or 'dev'>"}` |
| `POST /bot` | see below | `200 {"action": NeutralAction, "policy": "<effective spec>", "ms": <decision>, "hash": "<hash before the action>"}` |
| `POST /game` | position log + `winner` | `200 {"ok": true, "game_id": "<seed>-<8 hex>"}` |

`POST /bot` JSON:

```text
{
  "seed": "<u64 string>",
  "deckA": "<deck JSON string or {id: count}>",
  "deckB": "<deck JSON string or {id: count}>",
  "first": "a" | "b" | "coin",
  "actions": [ NeutralAction | {"reseed": "<u64 string>"} ],
  "policy": "<parse_spec>",
  "botSeed": "<u64 string>",
  "hash": "<optional client Game.hash() string>"
}
```

Replay contract: build `arena.Game(db, int(seed), deckA, deckB, first)`
with decks parsed the same way as `matchup.py`'s `parse_deck_json` (the
UI stores deck JSON strings; wasm `GameInner::new` parses the same
`{id: n}` shape), then apply every log step (`apply` or `reseed`). The
position log is exactly `Session.toPositionLog`: seed, decks, first,
actions including F6/F8 `reseed` steps. If `hash` is present it is
compared to `str(game.hash())` before the decision; mismatch →
`409 {"error": "state hash mismatch", "server": "…", "client": "…"}`.
Other client errors → `400 {"error": "…"}`; unexpected → `500`. One
decision at a time (a lock); a second concurrent request waits. One
stdout line per `/bot`: policy, turn, ms, hash-ok, `game=<game_id>`.

`--games-dir` (default `results/games`) writes each vs-bot game as it
is played. `game_id` is `f"{seed}-{sha1(deckA|deckB|first)[:8]}"` over
the canonical decoded decks. On every successful `/bot` the server
writes `<games-dir>/<game_id>.json` with `v, game_id, seed, decks,
first, actions, policy, strong, engine, updated, final=false`, but
only overwrites when the incoming `actions` list is longer than the
stored one. `--no-games` disables this. IO errors are logged and
swallowed so a capture failure never breaks a bot reply.

`POST /game` is the finished-game counterpart (same CORS allow-list
and lock as `/bot`). The client posts `toPositionLog` plus
`{"winner":"a"|"b"|null}`. The same file is written with
`"final": true`, `"winner"`, and `"finished"` when the log is at least
as long as the stored one. Without it, capture would stop at the last
bot decision.

Client (`ui/src/main.ts` + `botStepRemote` in `session.ts`): on load and
when the mode becomes vs-bot, `GET http://127.0.0.1:8765/health` with
`AbortSignal.timeout(400)`. Success stores `{strong, cpus, version}`;
any failure is silent (`console.debug` at most) and the badge reads
`bot: browser`. A settings toggle **"Use local bot server when available"**
(`localStorage` `svwb.localBot`, default on) gates the probe and the
remote step. The badge (`#botBackendBadge`, next to the vs-bot policy
select) is `bot: local server (h0:nodes=16000, 28 cpus)` or
`bot: browser`. While a remote decision is pending the badge reads
`bot: local server — thinking…`.

The vs-bot loop calls `botStepRemote` only when the seat's policy
starts with `h0` (never for `random` / `first-legal` — those stay
instant wasm). `nextBotSeed` is taken **once** per decision. On `200`
the returned action is applied through `applyAction` (same log / undo /
replay as a wasm move). On any failure (network, timeout 90 s, non-200,
JSON, or `applyAction` throwing illegal) the client falls back to wasm
**with that same seed** (`botStepWithSeed`) so rng consumption stays
one-seed-per-decision, logs one `console.warn`, and pins the badge to
`bot: browser (server error: …)` for the rest of the game. Watch mode,
the watch step button, hotseat, undo/redo/reroll are untouched.

When a vs-bot session reaches `phase() === "terminal"` and the local
server is in use (toggle on, not `?localbot=0`, health probe succeeded),
the client POSTs `toPositionLog(session)` plus `winner` to
`http://127.0.0.1:8765/game`, fire-and-forget (`AbortSignal.timeout(2000)`,
failures swallowed with one `console.debug`). Exactly once per session
(`reportedGameFor` guard, reset in `startSession`). With no server or
the toggle off, nothing is sent.

With no server reachable the client is unchanged: the same wasm
`botAction` calls, the same seeds, the same rng consumption. Existing
Playwright tests run without a server.

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
| `Game.bot_action(policy, seed) -> dict` | One NeutralAction for the acting player (`by_name` + `policy_rng` + `choose`). |
| `Game.bot_action_value(policy, seed) -> dict` | Same decision as `bot_action`, plus `{"action": NeutralAction, "value": float \| None}` where `value` is the policy's `last_value()` after `choose` (`None` for policies that do not search, e.g. `random`). |
| `Game.bot_action_explain(policy, seed) -> dict` | Same decision as `bot_action_value` for `h0` specs, plus a per-candidate explain record (see below). Non-`h0` policies return `{"chosen": NeutralAction, "path": "opaque"}`. Recording does not change the chosen action or node count. |
| `arena.play_random(db, seed, deck_a, deck_b, first="coin") -> dict` | `{winner, turns, actions, first}`. Random-legal + `policy_rng`. |
| `arena.matchup(db, decks, games, seed, policy="random", threads=None, policy_a=None, policy_b=None, first="alternate", records=False, export=None, export_epsilon=0.0) -> dict` | Every ordered pair including mirrors. `policy_a` / `policy_b` default to `policy` and accept any `parse_spec` string (a bad spec raises `ValueError` with the parser message). `first`: `"alternate"` (default) — game `g` of every pair is `First::A` when `g` is even and `First::B` when odd, so seats are mirrored at equal counts; `"coin"` — today's `First::Coin` (the game seed decides); `"a"` / `"b"`. Seeds are unchanged: `game_seed(seed, pair_index, game_index)`; both seats share `policy_rng(game seed)` as the bench does. Per pair `{games, a_wins, b_wins, draws, first_player_wins, a_games_as_first, a_wins_as_first, mean_turns, mean_actions, end}` where `draws = games − a_wins − b_wins` and `end` counts `{lethal, deckout, turn_cap, action_cap, no_legal, illegal}`. Top level: `seed, games, policy_a, policy_b, first, threads, matrix`. With `records=True`, a list `records` in job order of `{a, b, g, seed, first, winner, turns, actions, end}` (deck names, `"a"`/`"b"`/`None`, end reason as a string). Same seed + same thread count → identical output including `records`; `threads=1` equals `threads=None`. `export=None` (default) is byte-identical to today — no extra files, no `"export"` key. `export=<dir>` writes training-sample shards (see Training samples (M5b)) and adds `"export": {dir, samples, games}`; `export_epsilon` is ε-greedy exploration on those games (inner policy still asked first). |
| `py/stats.py::wilson(k, n, z=1.96) -> (lo, hi)` | Wilson score interval for `k` successes in `n` trials, clipped to `[0, 1]`. `n == 0` → `(0.0, 1.0)`. |

`py/matchup.py` loads `oracle/decks/*.json` (plus `--deck-file` extras in the
client's `{id: count}` / `[ids]` shapes), prints the two existing tables, a
third table of A's decisive win rate with the Wilson 95 % interval, and a
summary block (overall rate, first-player rate, means, end-reason counts,
games/s, `os.cpu_count()`, per-deck row/column rates). `--policy-a` /
`--policy-b` default to `--policy`; `--first` defaults to `alternate`.
`--export <dir>` / `--export-epsilon <f>` (default 0) forward to `matchup`;
the summary prints `samples: N (k per game)` when exporting.
`matchup.json` also stores `wilson95` per cell and `summary`.

## H0 explain (`bot_action_explain`)

`Game.bot_action_explain(policy, seed)` returns the same `chosen` and `value`
as `bot_action_value` for `h0` specs. The sink is armed only for that call;
when not used, search behaviour and node accounting are unchanged.

For `h0`, the dict also carries:

| key | type | meaning |
|---|---|---|
| `path` | str | Which `H0::choose` branch decided: `single_legal`, `mulligan`, `one_ply`, `consensus_lethal`, or `search`. |
| `k` | int | Determinized roots (`1` under `info=all`). |
| `node_cap` | int | Global node cap for the decision. |
| `alloc` | str | `fair` or `root`. |
| `nodes` | int | Total `apply`s spent. |
| `candidates` | list | One entry per root candidate, in candidate order. |
| `chosen_index` | int | Index in `legal` of the chosen action. |
| `tie_set` | list[int] | Every candidate whose aggregated value equals the maximum; the chosen index is the lowest. |

Each candidate entry:

| key | type | meaning |
|---|---|---|
| `legal_index` | int | Index in `legal`. |
| `action` | NeutralAction | That candidate. |
| `worlds` | list | Per determinized root `r`. |
| `root_agg` | float | Aggregated value over scored worlds. |
| `worst` | float | Worst clamped sample. |
| `n` | int | Worlds that scored this candidate. |

Each world entry:

| key | type | meaning |
|---|---|---|
| `raw` | float | `search_own` return before `finite`. |
| `clamped` | float | After `finite`. |
| `node_cap` | int | Cap for this `(root, candidate)` pair. |
| `nodes` | int | Nodes spent on this pair. |
| `hit_cap` | bool | Pair reached its cap. |
| `skipped` | bool | Pair never searched (global cap already spent). |
| `pv` | list[NeutralAction] | Principal variation (up to 12 actions) of the line that produced the value. |
| `end` | str \| null | `depth`, `cap`, `terminal`, `opp_reply`, or `opp_lethal`. |
| `leaf` | object \| null | `{value, phase, turn, active}` at the line end. |

Non-`h0` policies return only `{"chosen": NeutralAction, "path": "opaque"}`.

## Training samples (M5b)

`Recorder` wraps any `Box<dyn Policy>` and implements `Policy`. On each
`choose` for its seat it encodes the state (`Observation::LEN` = 545,
`IDS_LEN` = 220), records `value_v0` (the hand-written leaf), then asks
the inner policy. With probability `epsilon` it replaces that choice by a
uniform legal index **after** the inner call, so `epsilon = 0` consumes no
extra rng and is byte-identical. After the game the caller labels the
buffer: `+1` if that seat won, `−1` if it lost, `0` on a draw.
`Recorder::take` hands the buffer over. No file IO in the engine (wasm32
builds the module; it just is not used).

`arena.matchup(..., export=<dir>)` wraps both seats, labels both buffers
from the outcome, and appends samples under a mutex as rayon workers
finish. **Sample order is not deterministic across runs — the content
is.** Every sample carries `game_index = pair_index * games + g`
(`pair_index = i * n + j`, the same index `game_seed` uses) and a
per-seat `decision_index`. The multiset of samples for a given seed is
identical run to run. Files in `<export>/` are raw little-endian arrays
with no headers (`numpy.fromfile`; `py/samples.py::load` reshapes them):

| file | dtype | shape |
|---|---|---|
| `features.f32le` | f32 | N × 545 |
| `ids.u32le` | u32 | N × 220 |
| `labels.f32le` | f32 | N (`+1` / `−1` / `0`, acting player's outcome) |
| `aux.f32le` | f32 | N × 11 |
| `meta.json` | — | `{samples, feature_len, ids_len, aux_columns, layout, policy_a, policy_b, first, seed, games, epsilon, decks, engine_version}` |

`aux` columns, in order: `game_index`, `decision_index`, `side` (0 = A,
1 = B), `turn`, `phase` (0 mulligan, 1 main, 2 choice, 3 end, 4
terminal), `v0`, `legal_len`, `chosen`, `random` (0/1), `first_is_me`
(0/1), `search_v` (the inner policy's `last_value` at that decision, or
`NaN` when it did not search — mulligan, forced/`legal_len == 1`,
single-candidate, or a policy with no search). Indices 0–9 are
unchanged; `search_v` is the 11th (last) column. The ε-random
replacement does not change `search_v` (it is the value of the
position, not of the random action). `meta.layout` is
`Observation::LAYOUT` as `{name, offset, width}`.
`py/samples.py::load(dir)` returns numpy arrays `features (N, 545)
float32`, `ids (N, 220) uint32`, `labels (N,)`,
`aux (N, len(aux_columns))`, plus `meta` and `aux_columns`. Existing
10-column data sets (no `search_v`) still load. Numpy is a test/tool
dependency, not of the extension.

## Learned value (M5b-lite)

The default H0 leaf is the built-in `h0-linear-v1` model
(`engine/models/h0-linear-v1.json`), `include_str!`-embedded and parsed
once through `OnceLock` (`builtin_net()` / `BUILTIN_NET_NAME`). Provenance
and the immutability rule live in `engine/models/README.md`: model files
are measured artifacts — a retrained model is a new file with a new name
and becomes the default only after it beats the current default on the
4 096-game yardstick. The search, the determinization, and the opponent
model stay exactly as they are (`value=v1`, `tt`, `wv`, `odepth`
untouched; `olethal` / `osteps` later flipped to `1` / `6`). `value=v0` is the hand-written leaf the
bot used before this default and is byte-identical to that arithmetic.
`h0:value=net` (no path) keeps the built-in — it is the same as `"h0"`.
`net=<path>` overrides the built-in (`h0:net=<path>` alone means
`h0:value=net,net=<path>`). `net=` with `value=v0` or `value=v1` is a
parse error naming both keys (`net= requires value=net`). A missing file
is a parse error naming the path. The path may not contain commas.
`spec()` prints nothing for the built-in net and `value=net,net=<path>`
when a path is set; `h0_fields_eq` compares the path. `names()` is
unchanged. The linear leaf costs ~6 µs against v0's 0.15 µs (−14 %
throughput). The model adds ~56 KB to the engine and to the wasm binary;
`ValueNet::load` is still the only `std::fs` user and is never called
from wasm.

`ValueNet::load(path)` parses `net.json` (`serde_json`; no ONNX runtime).
`ValueNet::value(&obs)` standardizes the 545 features with the stored
mean/std (std floored at `1e-3` at train time), adds per-zone card
terms, applies `tanh`, and returns **`scale × tanh(…)`**. Default
`scale = 60` puts a sure win at +60, below the terminal stand-in `wv`
(80), so `one_ply`'s `+3.0` face-attack bonus stays proportionate on the
v0 scale. Do not change `wv` or the bonus.

Model file (one JSON object, f32 values as JSON numbers):

```text
{
  "arch": "linear" | "mlp",
  "feature_len": 545,
  "feat_mean": [545], "feat_std": [545],
  "vocab": [ids ascending, index 0 = 0],
  "zones": [{"name", "id_offset", "count", "hist_offset" | null} × 5],
  "scale": 60.0,
  "linear": {"w": [545], "zone_w": [5][vocab], "b": f}
    | "mlp": {"emb": [vocab][emb], "w1": [hidden][545 + 5·emb],
              "b1": [hidden], "w2": [hidden], "b2": f},
  "trained_on": {"dirs", "rows", "games", "holdout": {…metrics}}
}
```

Unknown ids at inference map to index 0. Opponent-hand slots are always
0 and are ignored.

Zones (220 id slots, 5 groups):

| name | id slots | count weight |
|---|---|---|
| `own_hand` | `0..9` | 1 per slot |
| `own_deck` | `9..105` | `own_deck_hist` at feature offset 353 |
| `opp_board` | `IDS_OPP_BOARD..+5` | 1 per slot |
| `own_board` | `IDS_OWN_BOARD..+5` | 1 per slot |
| `opp_pool` | `IDS_OPP_POOL..+96` | `opp_known_pool_hist` at offset 449 |

`linear`: `tanh(w·x + Σ_zone Σ_slot count·W_zone[id] + b)` — one scalar
weight table per zone. `mlp`: per-zone weighted sums of a shared `emb`
(vocab × `emb`) concatenated with the 545 standardized features
(`545 + 5·emb`) → `hidden` ReLU → 1 → `tanh`.

The `Evaluator` computes `vocab(root)` **once per `choose`** from the
root state and reuses it for every leaf `encode_with_vocab`. A token
that first appears inside the search tree is simply not in the
histogram (its column stays empty). Training data uses the per-state
vocabulary, which is the same thing whenever no new token appeared.

The learned leaf is trained on one label per position — by default the
final outcome of the game, ±1. That label is the same for every
position of a game, so on a long game the early positions carry almost
no information, and it says nothing about *how good* a position is,
only who eventually won. A net that is good pointwise on outcome can
still rank siblings poorly inside the search. The standard remedy is
**search-improved targets**: at every decision H0 already computes a
value for the position (the K-root average of the chosen action's
search value, on the leaf's own scale), and that number is a far less
noisy teacher than the outcome.

`py/train_value.py --data <dir> [<dir> …] --model linear|mlp --out <net.json>`
loads every directory with `py/samples.py`, concatenates, and splits
**by game** (`holdout` fraction of distinct `game_index` values, offset
per directory so games never collide). Flags: `--holdout` (default 0.1),
`--epochs` (30), `--seed`, `--hidden` (128), `--emb` (16), `--l2`
(1e-4), `--max-samples`, `--target outcome|search|mix` (default
`outcome` — trains exactly as before), `--mix-weight w` (default 0.5;
only meaningful with `mix`), `--search-scale S` (default 60.0, the
built-in net's `scale`), `--eval MODEL [MODEL …]`. Per row
`s = clip(search_v / S, −1, 1)`; the training target is `outcome` →
`label`; `search` → `s`; `mix` → `(1 − w)·label + w·s`. Rows whose
`search_v` is `NaN`, and every row of a data set that has no
`search_v` column at all, use `label` regardless of `--target` (the
summary prints how many rows had a search value). Loss is MSE against
that target; Adam with `--l2` weight decay; early stopping on holdout
MSE (patience 3). Holdout metrics (`sign_acc`, `auc`, `mse`, by turn
band, the `v0` baseline) stay against the **outcome** label. The
report adds a `search_v` block alongside `v0` (the search value's own
sign accuracy / AUC against the outcome on the held-out rows; skipped
with a note when the column is absent) and, when `--eval` is given,
an `eval` map of each model JSON's `metric_block` on those same rows
(printed as `--- eval <basename> ---` after `v0`; unknown ids map to
index 0 as in the engine). `target`, `mix_weight`, `search_scale`, and
the search-row count are written into the report and into
`trained_on`. The model file format does not change.

## One-command iteration (`py/iterate.py`)

```
python py/iterate.py --tag net3 --seed 401 [options]
```

A driver only: it calls `py/matchup.py` and `py/train_value.py` as
subprocesses (the venv interpreter, absolute paths, no `shell=True`) and
changes no engine or training behaviour. Everything lands under
`<root>/<tag>/` (`--root` defaults to `<repo>/results`). Stages run in
order and are resumable — a stage whose outputs already exist is skipped
with a `skip: <what>` line (`--force` reruns everything; `--only
data|train|yard|summary|publish` runs one stage; `--skip-data` and the
other `--skip-*` flags are the obvious negatives). Every subprocess argv
is printed before it runs; stdout/stderr is teed to
`<tag>/<name>.txt`. A non-zero exit stops the run naming the stage and
the log file. `<tag>/RUN.json` records the command line, start/end times
per stage, `git rev-parse HEAD`, `sys.version`, and `os.cpu_count()`.

| stage | what | files |
|---|---|---|
| **data** | `matchup.py --policy <bot>` (default `h0`) `--games G` (default 24) `--seed S --export <tag>/data-e0` and `--seed S+1 --export-epsilon <ε>` (default 0.1) `--export <tag>/data-e10`. `--decks` passed through when given. | `data-e0.txt` / `data-e10.txt`, the export directories, `data-e0.json` / `data-e10.json` |
| **train** | For each `--models` entry (default `linear mlp`): `train_value.py --data <tag>/data-e0 <tag>/data-e10 <extra --data dirs>` (new directories first so `--max-samples` keeps them) `--out <tag>/m.json --target` (default `outcome`) `--mix-weight` / `--search-scale` / `--eval` (default `engine/models/h0-linear-v1.json` when that file exists) and `--epochs` / `--max-samples` when given. If `torch` is not importable and `mlp` is requested, that model is skipped with a clear line; the run does not fail. | `train-m.txt`, `m.json`, `m.report.json` |
| **yard** | For each trained model `C = h0:value=net,net=<absolute path of <tag>/m.json>` (the path may not contain a comma — the spec parser splits on commas) against `--baseline` (default `h0`): main (`--policy-a C --policy-b B --games` `--yard-games`, default 16), reverse seating (`--policy-a B --policy-b C --games` `--reverse-games`, default 8), sanity (`C` vs `random`, `--sanity-games` default 100, `--decks basic-forest`), throughput (`--policy C --games` `--tp-games` default 1), and a craft mirror per `--mirrors` deck (default `royal-nattui`, `--mirror-games` default 200). Once: `--policy B --games <tp-games>` → `tp-h0`. `--threads` passes through everywhere. `--yard-seed` (default 1) is independent of the data `--seed` so evaluation games are not the same shuffles the net trained on. | `main-m.txt/json`, `reverse-m.*`, `sanity-m.*`, `tp-m.*`, `mirror-<deck>-m.*`, `tp-h0.*` |
| **summary** | Written from the JSON files (never by parsing the text). | `SUMMARY.md` |
| **publish** | Off unless `--publish`. | copy into the results worktree (below) |

`--smoke` sets the whole run to a few minutes on 4 cpus for testing:
`--decks basic-forest basic-rune --games 1 --yard-games 1
--reverse-games 1 --sanity-games 4 --mirror-games 2 --tp-games 1
--epochs 2 --models linear`. Explicit flags still override. Everything
else is identical, including publish.

`SUMMARY.md` has a header (tag, seeds, bot, baseline, engine SHA, total
wall time and per-stage times); a **data** table (games, samples,
samples per game, first-player rate, throughput per run); a **holdout**
table per model (net / v0 / search_v / each eval baseline: sign acc,
AUC, MSE overall and by turn band — from `m.report.json`); a
**yardstick** table per model (main rate with the Wilson 95 % interval,
reverse seat as the candidate's rate `1 −` A's rate with the interval
flipped, sanity rate, throughput g/s vs `tp-h0`, each mirror's rate +
interval); and a **verdict** per model by the standing rule: `better` if
the main interval's low end > 0.50 **and** the reverse interval
(candidate's) low end > 0.50; `worse` if the main interval's high end <
0.50; `coin flip` if both intervals contain 0.50; otherwise
`unclear (…)` naming which side disagrees. It ends with the one line
the owner reads first: `verdict: <model> <better|worse|coin flip|unclear>
— main 0.5xx [lo, hi], reverse 0.5xx [lo, hi]` per model.

`--publish` (default off; `--publish-remote` default `origin`,
`--publish-branch` default `results`, `--publish-dir` default
`<repo>/../arena-results-wt`) copies `<tag>/*.txt`, `<tag>/*.json` and
`SUMMARY.md` — never the `data-*` directories — into
`<publish-dir>/<tag>/`. That directory is a `git worktree` of the
orphan branch `results`: if the branch exists on the remote,
`git worktree add <dir> <branch>` (or `git -C <dir> pull --ff-only`
when the worktree already exists); if not, `git worktree add --detach
<dir>`, `git -C <dir> checkout --orphan <branch>`, `git -C <dir> rm -rf
-q .`, an empty first commit, `git -C <dir> push -u <remote> <branch>`.
Then `git -C <dir> add -A`, `git -C <dir> commit -m "<tag> results
<YYYY-MM-DD>"`, `git -C <dir> push`. The main working tree is never
touched. A publish with nothing new to commit is not an error. Do not
push to `origin/results` from a test environment — tests use a
temporary bare remote only.

The script never writes under `engine/models/` — shipping a model stays
a reviewed PR (`engine/models/README.md`).

## Sweeps (`py/sweep.py`)

```
python py/sweep.py --tag sweep1 --candidates "h0:nodes=4000" "h0:depth=4,beam=8" "h0:k=8" --baseline h0 --publish
```

A measurement driver only: it calls `py/matchup.py` as a subprocess (the
venv interpreter, absolute paths, no `shell=True`) and runs no training.
Everything lands under `<root>/<tag>/` (`--root` defaults to
`<repo>/results`). Candidates are numbered in the order given;
**files are named by index, never by spec** (specs contain `:`, `=`, `,`
and paths): `<tag>/candidates.json` is
`[{"index": 1, "spec": "h0:nodes=4000"}, …]`, then `c01-screen.json/.txt`,
`c01-final.json`, `c01-reverse.json`, `tp-c01.json`, plus one
`tp-baseline.json`. Every spec is validated up front by a 0-game
`arena.matchup` call, which exercises `AnyPolicy::parse_spec` (the same
parser `by_name` uses); a bad token such as `h0:depht=2` stops the run
before any matchup file is written.

| stage | what | files |
|---|---|---|
| **screen** | For each candidate `C` vs `--baseline` `B` (default `h0`): `--policy-a C --policy-b B --games` `--screen-games` (default 4 = 1 024 games over the 16 decks), `--seed` (default 1). Once: `--policy B --games` `--tp-games` (default 1) → `tp-baseline`. Per candidate: `--policy C --games <tp-games>` → `tp-cNN`. `--decks` / `--threads` pass through. | `cNN-screen.*`, `tp-cNN.*`, `tp-baseline.*` |
| **finalists** | Rank by screen rate; keep the top `--finalists` (default 2) among those whose screen interval's **high** end is **above** 0.50 (already lost at ±3 % → no final; high end exactly 0.50 is not above). The choice and the reason for every candidate (`finalist`, `skipped: interval high 0.48 < 0.50`, `not in top 2`) go into `RUN.json` and the summary. | `RUN.json` |
| **final** | For each finalist: main `--games` `--final-games` (default 16 = 4 096) and reverse `--policy-a B --policy-b C --games` `--final-reverse` (default 8 = 2 048). Optional `--mirrors <deck …>` at `--mirror-games` (default none). | `cNN-final.*`, `cNN-reverse.*`, `cNN-mirror-<deck>.*` |
| **summary** | Written from the JSON files (never by parsing the text). | `SUMMARY.md` |
| **publish** | Off unless `--publish`. Same flags and worktree / orphan-branch mechanics as `iterate.py` (`--publish-remote`, `--publish-branch`, `--publish-dir`). Copies `<tag>/*.txt`, `*.json`, `SUMMARY.md`; never touches the main working tree; nothing new to commit is not an error. | copy into the results worktree |

Stages are resumable per file exactly like `iterate.py`: a matchup whose
`.json` already exists is skipped (`--force` reruns; `--only
screen|final|summary|publish` runs one stage). Every subprocess argv is
printed and teed to `<tag>/<name>.txt`. `<tag>/RUN.json` records the
command line, engine SHA, per-stage times, and the finalist decisions.

`--smoke` sets `--decks basic-forest basic-rune --screen-games 1
--final-games 1 --final-reverse 1 --tp-games 1 --finalists 1` and, unless
`--baseline` is given, the `h0-fast` spec
`h0:depth=2,beam=2,k=1,nodes=80,value=v0,tt=0`. Explicit flags still
override.

`SUMMARY.md` has a header (tag, baseline, seed, engine SHA, wall time
per stage); a **screen** table sorted by rate (index, spec, rate +
Wilson interval, games, throughput g/s vs baseline, finalist decision);
a **final** table (index, spec, main rate + interval, reverse as the
candidate's rate with the interval flipped, mirrors); a `verdict:` line
per finalist by the standing rule (`better` if both low ends > 0.50;
`worse` if the main high end < 0.50; `coin flip` if both intervals
contain 0.50; else `unclear (…)`); and a last line
`best: <spec> (<verdict>)` naming the finalist with the highest main
rate, or `best: none` when there is no finalist.

`py/runlib.py` is the shared home of the helpers both drivers import
(`verdict`, `format_verdict_line`, `reverse_candidate`, the tee runner,
the matchup caller, Wilson/rate formatting, and publish / worktree).

## Blunder review (`py/review.py`)

```
python py/review.py --games results/games --tag review1
```

Replays captured vs-bot games (`py/serve.py --games-dir`) against a
deep reference and ranks the analysed seat's mistakes. A **blunder**
here is a decision, at a non-mulligan position with more than one
legal action, whose reference value is worse than the reference's own
choice:

```
cost = max(0, v_best − v_played)
```

`v_best` is `Game.bot_action_value(ref, seed_for(game_id, ply))["value"]`
— the K-root search value of the reference's chosen action, from the
acting player's perspective. `v_played` is that same reference's value
after the recorded action (terminal → `±wv`, default 80; otherwise the
reference value on the resulting position, negated when the acting
player changed so the number stays in the analysed player's frame).
Search noise can make `v_best − v_played` slightly negative; those
are clamped to 0 and counted (a large count means the reference is too
shallow to be a reference).

The default `--ref` is `h0:nodes=200000,k=16`. `k=16` is four times
H0's usual determinizations: hidden information is where a shallow
read goes wrong. `--side bot|human|both` (default `bot` = seat B
unless the log sets `humanSide`). `--threads` parallelises across
decisions (each rebuilds from the log). Seeds are
`seed_for(game_id, ply[, extra])` so a rerun is byte-identical.
`--only`, `--force`, `--cards`, `--top` (default 25), `--out`
(default `results/<tag>`).

Outputs under `results/<tag>/` (resumable per game; skip unless
`--force`):

| file | what |
|---|---|
| `<game_id>.json` | full decision list (`ply, turn, phase, acting, played, reference, v_played, v_best, cost, bot_value, position`) |
| `REVIEW.md` | per-game header + value trace (reference value at the start of each analysed turn), then the top-N blunders, then aggregates by turn band (1–3 / 4–6 / 7–9 / 10+), action kind (play / attack / evolve / end-turn / ability), and side, plus the clamped-negative count and the reference's mean decision time |
| `RUN.json` | `runlib` record (argv, engine SHA, stage times) |

Card names in `REVIEW.md` come from the card JSON `name` field under
`cards/` (plus `full()` instance names); ids when a name is missing.

`--publish` pushes the tag directory to the orphan `results` branch
with the same worktree / `--publish-remote` / `--publish-branch` /
`--publish-dir` mechanics as `sweep.py`. `REVIEW.md` is copied in
addition to the `.json` / `.txt` files `copy_tag_artifacts` already
takes. The main working tree is never touched.
