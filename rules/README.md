# Rules

Precedence, highest first:

1. **Owner ruling** — [`owner-rulings.md`](owner-rulings.md)
2. **Printed card text**, as clarified by the official Cygames Q&A — [`official-qa.md`](official-qa.md) and each card's `text` (from `cards/official/catalog.json`, not the old repo)
3. **Rulebook synthesis** — [`rulebook.md`](rulebook.md)

The rulebook is derived from the other two sources and is never authoritative on its own. A ruling that contradicts a Q&A is deliberate and stays.

A new ruling is written into `owner-rulings.md` first, before any card file or engine change that depends on it.

## Sources

Card facts, texts, token links, rotation flags, crest / Faith / Crystallize / Accelerate texts, and Q&A come from Cygames (`cards/official/catalog.json`). See `docs/design.md` § Sources.

The only material taken from `melnce/Practice-Tool` is this folder's ruling knowledge (`owner-rulings.md`), the rulebook (`rulebook.md`), and the old engine's use as a differential oracle (interaction knowledge — never binding). Nothing else from that repo is a source.
