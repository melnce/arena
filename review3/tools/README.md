# review3 tools

Throwaway helpers used for the bot-side analysis in `../BOTVIEW.md`. They run on the owner's machine
(`REPO` in `botview.py` is `C:\Users\agban\projects\arena`) and read `results/review3/<id>.json` and
`results/games/<id>.json` there (copies are in `../` and `../games/`).

The games were played on engine build **a45e2bb**. Replays must use bindings built from that commit: the
helpers put `tools/frozen/` first on `sys.path`, so copy an `arena` package built at a45e2bb into
`tools/frozen/arena/` (the binary is not published). Later builds change the Bonus PP rule (PR #84), so
positions after a burned charge diverge.

- `botview.py`: writes the timelines in `../botview/` (full state per player-turn, every bot decision with the
  reference's view and a `bot_action_explain` re-run).
- `replay_at.py GAME_ID N [SPEC] [SEED]`: re-ask the bot at the position before action `[N]`; prints candidates,
  per-world values, how lines ended, and the expected line.
- `cf_skip.py GAME_ID N SKIP [SPEC] [SEED]`: the same, after replaying with the listed action indices skipped.
