# Oracle gate

The old TypeScript engine (`melnce/Practice-Tool`) is the **oracle for rules**, not for card data. It emits JSONL in [`docs/trace-format.md`](trace-format.md). `arena-replay` / `arena_engine::oracle::replay_trace` replay each line with `ScriptedRng` and diff `state` (and `legal` when present). A committed gzipped set lives under `oracle/traces/`; `engine/tests/oracle.rs` replays every file and fails CI on an unexpected first divergence.

A divergence caused by the old repo's card JSON is `old-data`: allowlist it with the card named. **Never** "fix" it by copying the old behaviour into this engine.

## Producing the traces

Repo: `melnce/Practice-Tool`, branch `cursor/trace-emitter` / [PR #390](https://github.com/melnce/Practice-Tool/pull/390).

**Emitter commit for the committed set:** `c9c7aad226ffd1b5cd0067da12e43e6a24c4b1a5` (use exactly this SHA; head of PR #390).

```
git clone https://github.com/melnce/Practice-Tool.git
cd Practice-Tool
git checkout c9c7aad226ffd1b5cd0067da12e43e6a24c4b1a5
npm ci
```

Decklists (this repo, `{"<card id>": count}`, 40 cards) are under `oracle/decks/`. Copy them next to the old engine or pass absolute paths. Then, from the old repo root:

```
npm run trace -- --seed=20260910 --games=30 --deck-a=oracle/decks/ramp-37772.json --deck-b=oracle/decks/ramp-37772.json --out=<tmp>/ramp-37772-mirror
npm run trace -- --seed=20260910 --games=20 --deck-a=oracle/decks/basic-forest.json --deck-b=oracle/decks/basic-forest.json --out=<tmp>/basic-forest-mirror
npm run trace -- --seed=20260910 --games=20 --deck-a=oracle/decks/basic-rune.json --deck-b=oracle/decks/basic-rune.json --out=<tmp>/basic-rune-mirror
npm run trace -- --seed=20260910 --games=20 --deck-a=oracle/decks/basic-portal.json --deck-b=oracle/decks/basic-portal.json --out=<tmp>/basic-portal-mirror
npm run trace -- --seed=20260910 --games=30 --deck-a=oracle/decks/abyss-p8rfn.json --deck-b=oracle/decks/abyss-p8rfn.json --out=<tmp>/abyss-p8rfn-mirror
```

`basic-rune.json` is the original 14-card Basic Rune list, including Witch's New Brew `10031210` ×3. Owner ruling 2026-09-10: Brew (and Magic Sediment) have Aura even though the catalog text does not print it, matching the old engine.

### Re-gzip

Never commit plain JSONL under `oracle/`. From each set directory:

```
gzip -n -k -c trace-20260910-0.jsonl > oracle/traces/<set>/trace-20260910-0.jsonl.gz
```

`-n` drops the timestamp so the bytes are stable. Check before committing:

- every header carries `opening_hands`
- every `rng` entry that is `raw` carries `kind: "shuffle"`
- no `play.card` starts with `uid`
- every game's last line has `phase: "terminal"`

If any of those fail, the emitter commit is wrong — do not patch the traces.

The committed set at `c9c7aad2` passes all four checks: every game's last line is `phase: "terminal"`.

## Allowlist

`oracle/known-divergences.json` is the reasoned skip list for the **first** divergence of a trace. The test passes a red line only when an entry matches the same `trace` (`<set>/<file>.jsonl`, no `.gz`), same `i`, and same `path`. Values are not part of the match. An entry that matches nothing is stale and fails the test — remove it; do not accumulate.

A valid entry:

```json
{
  "trace": "ramp-37772-mirror/trace-20260910-2.jsonl",
  "i": 33,
  "path": "players.a.field[0].max_defense",
  "class": "old-rule",
  "reason": "owner ruling 2026-09-10: a −0/−4 lowers max_defense by 4; the old engine set it to the current defense",
  "since": "2026-09-10"
}
```

`class` is one of `old-data` | `old-emitter` | `engine` | `convention` | `old-rule`. `old-rule` means the old engine disagrees with an owner ruling or the rulebook; arena is right — never used for an emitter representation defect, that is `old-emitter`. `reason` is one sentence a stranger can check and must name the card or rule. `path` is a JSON path, or `legal` / `illegal` / `error` (`error` is an `OraclePickNotLegal` failure at that `i`).

`ARENA_ORACLE_STRICT=1` ignores the allowlist (the true red set). CI does not set it.

Do not fix an `engine` divergence in the same change that records it. One brief per root cause.

## Running the gate

```
cargo test --release --test oracle
```

CI shares the release build with the soak step (`cargo test --release --test oracle`). Debug replay of the 120 traces is ~8 s, so the test stays in the default `cargo test`.
