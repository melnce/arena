# Earrings Engage as-recorded overlay

Before the sacrifice-Engage source fix, **Earrings of Sunlight** (`10761210`) Engage
destroyed the amulet and silently skipped replicating its Fanfare. Bot recordings and
review fixtures taken under that behaviour pin decisions at positions that only exist
when Engage does nothing after the destroy.

This directory replaces only the Engage ability’s `effects` with `[]` on an otherwise
identical card. Replays that load `with_earrings_engage_as_recorded` (or `load_recorded_db`,
which chains it) match the hashes recorded on `main` before the fix.

**Fixtures:** `hbcheck/neg-play-100087-ply0065`, `hbcheck/pos-play-100085-ply0058`,
`tkroll/fa-play-103-ply0100`, `tkroll/fa-play-150-ply0086`, `tkroll/fb-play-122-ply0064`,
and review14 game `6598261483642061665-d90f8eb2` (plies 27, 52, 104).

Only replays of recordings made before the Engage fix should load this overlay. Never use
it for new recordings or generated games (`load_db()` without this overlay is current rules).

Introduced in PR #132.
