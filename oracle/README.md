# Oracle

The old practice tool (`melnce/Practice-Tool`, TypeScript) stays running and is the **differential-test oracle**.

Seeded games are replayed through both engines. Both emit the JSONL in [`docs/trace-format.md`](../docs/trace-format.md). The harness diffs line-by-line; the first divergence wins. Every divergence is a bug in one of the two engines and is adjudicated by printed card text plus [`rules/owner-rulings.md`](../rules/owner-rulings.md) (Q&A sits between them).

The old engine already has the pieces the later emitter brief will wrap:

- `getLegalSoakActions` — legal action list
- `canonicalJson` / `captureFullSnapshot` — state projection (today includes uids and engine-private fields the new `CanonicalState` deliberately drops)
- `replaySoakTrace` — replay a recorded action list

M0 does not emit traces and does not change the old repo. The shared format is specified here so both emitters can be written without further design questions.

`ScriptedRng` on the new engine consumes each line's `rng` picks by matching the recorded **outcome** against its own candidate list. An outcome that is not a candidate is itself a finding: the oracle picked something illegal here.
