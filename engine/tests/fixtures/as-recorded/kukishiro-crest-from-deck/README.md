# Kukishiro crest as-recorded overlay

Recordings taken before the `randomNamed` crest fix used `randomFrom` on Crest: Kukishiro,
Mistbloom, which pulled Fox of Purity from the deck when present and otherwise summoned
nothing. Replays that load `with_kukishiro_crest_as_recorded` (or `load_recorded_db`, which
chains it) keep that recording-time behaviour for the crest only.

**Fixture:** `hbcheck/neg-play-100087-ply0065.json` (B plays Kukishiro at action 50).

Only replays of recordings made before this PR should load this overlay. Never use it for
new recordings or generated games (`load_db()` without this overlay is current rules).

Introduced in the Kukishiro crest / Sofina / burned-draw PR.
