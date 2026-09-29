# review4 tools

Throwaway helpers used for `../ANALYSIS.md`. Unlike review3's, these run on the **current** engine build (the games
were played on **71127bf**; `botview2.py` strips review3's `frozen/` path so `import arena` loads the venv build).
`REPO` in `botview.py` is the owner's checkout (`C:\Users\agban\projects\arena`); the captures are in `../games/`
here and in `results/games/` there.

- `botview.py` — review3's helpers (card names, state lines, action text), imported by the others.
- `botview2.py` — writes the timelines in `../botview/`: full state per player-turn, every bot decision with a
  `bot_action_explain` re-run at the served spec `h0:nodes=16000,horizon=3`, and `[SOLVER]` tags from
  `arena.forced_lethal(game.clone(), 50000)` on the position before every action.
- `escape_check.py` — at the bot's last two turns before the owner's winning turn: first moves = the recorded one, the
  served top-8 candidates and a perfect-information bot's choice, each completed by `h0:nodes=16000,horizon=3,info=all`,
  then the solver for the owner's reply. Output: `../escape_check.txt`.
- `escape_detail.py` — re-runs two `unknown` verdicts at a 500 000-node budget and finds where the recorded 9420 turn 7
  diverged from the safe line. Output: `../escape_detail.txt`.

Positions: `[N]` is the 0-based index into the capture's `actions` (the two mulligans are `[0]` and `[1]`; these
captures have no `reseed` steps). "Before `[N]`" = `Game(db, seed, deckA, deckB, first)` with `actions[0..N-1]` applied.
