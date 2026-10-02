# review9 — holdback2 Part 2: 12 owner games vs the served bot with the held-back check (2026-10-02)

(Runbook holdback2 names this `review8`; that name was already taken by the owner's Portal AF games, results
`d31937b`, published before the runbook arrived — so Part 2 is `review9`.)

- Engine **`d920fe1`** (WP: `hbcheck`; v3 default leaf; the balance patch not merged). Rebuilt 2026-10-02 22:05;
  `pytest py/tests -q` on this build: 154 passed, 6 skipped, 1 xfailed.
- Served with `py\serve.py --strong "h0:nodes=32000,horizon=3,k=8,hbcheck=2000"`; serve.py printed
  `strong:   h0:nodes=32000,horizon=3,k=8,hbcheck=2000` (health check: version d920fe1).
- Abyss Midrange mirror, the owner on side `a`, the fair bot only (no cheater games). Sweep 35 (Part 1) ran on the box
  from 23:09 (node-limited: thinking time only).

**Bot won 5 of 12.**

| # | game | first | winner | actions | finished (UTC) |
|---|---|---|---|---|---|
| 1 | `14138316141107943154-5ce21003` | owner | owner | 110 | 2026-10-02T20:17:45Z |
| 2 | `9070295640122167966-5ce21003` | owner | **bot** | 74 | 2026-10-02T20:22:41Z |
| 3 | `9019578278799017103-5ce21003` | bot | **bot** | 103 | 2026-10-02T20:29:25Z |
| 4 | `2389810489872439281-5ce21003` | owner | owner | 89 | 2026-10-02T20:41:24Z |
| 5 | `6404131829274723179-5ce21003` | owner | **bot** | 88 | 2026-10-02T20:49:01Z |
| 6 | `15924744114754175862-5ce21003` | bot | **bot** | 60 | 2026-10-02T20:52:15Z |
| 7 | `2549306584232634047-5ce21003` | owner | owner | 101 | 2026-10-02T20:59:41Z |
| 8 | `4956050980750562774-5ce21003` | bot | **bot** | 102 | 2026-10-02T21:05:10Z |
| 9 | `1498700160175729134-5ce21003` | owner | owner | 80 | 2026-10-02T21:08:42Z |
| 10 | `7351330708175418227-5ce21003` | bot | owner | 74 | 2026-10-02T21:13:41Z |
| 11 | `8009973016970256968-5ce21003` | owner | owner | 82 | 2026-10-02T21:19:50Z |
| 12 | `14697323178199318731-5ce21003` | bot | owner | 111 | 2026-10-02T21:26:25Z |

`games/<game_id>.json`: the raw serve.py captures, byte for byte.

**Owner's notes on clearly bad trades:** none given while playing (to be added if he names any; the bots thread's list
of `holdback_trade` attacks goes to him for verdicts).

**Preview, not the deciding measurement:** `holdback_audit.py scan` (holdback1's script, unchanged) on these 12 captures,
on this box's d920fe1 bindings, bot seat only — `holdback_scan_preview.jsonl`. It finds **5 HB moments**
(0.42 per game); the runbook's Part 2 rows are applied by the bots thread on its own scan. (The script's printed
`VERDICT` line is holdback1's rule and does not apply here.)

| game # | ply | turn | primary trade X -> Y (card ids) | X next turn | can the owner kill X |
|---|---|---|---|---|---|
| 4 | 82 | 9 | 10854110 -> 90051140 | game over | kill (sure 4/4) |
| 5 | 12 | 2 | 10951120 -> 10751120 | died | kill (sure 4/4) |
| 7 | 96 | 10 | 90051140 -> 90051140 | game over | kill (sure 4/4) |
| 8 | 76 | 9 | 90051140 -> 10952110 | survived | kill (sure 1/4) |
| 11 | 32 | 5 | 10751120 -> 90051110 | survived | kill (sure 4/4) |
