# review5 tools

Throwaway helpers used for `../REPORT.md`. They run on the owner's checkout (`REPO` in `botview.py`) against the
**current** engine build (games played on 2d12e25; same Rust as dc868a4). `botview.py` is review3's helper module.

- `botview3.py` — builds `../botview/`: full state per player-turn; every cheater decision with a
  `bot_action_explain` re-run at `h0:nodes=16000,horizon=3,info=all`; `[SOLVER]` tags from
  `arena.forced_lethal(game.clone(), 50000)` before every action.
- `escape_check3.py` — at the cheater's last two turns before each of its 13 losses: first moves = the recorded one,
  its top-8 candidates and its own choice, each turn completed by the cheater itself, then the solver for the owner.
  Output: `../escape_check.txt`.
- `replay_now.py GAME_ID N [SPEC] [SEED]` / `replay_all.py` (all candidates) — re-ask the bot before action `[N]` and
  print the exact solver verdict (200k) for the side to move.

`[N]` = 0-based index into the capture's `actions` (mulligans `[0]`, `[1]`; these captures have no `reseed` steps).
