# review7 — 8 owner games vs the eight-world served bot (2026-10-02)

Abyss Midrange mirror only, played through `py/serve.py` on engine **`d5f4310`** (WL + WM: the **v3 leaf** is the
default; `tkill` / `okill` / `omacro` off), with the served spec the owner chose after sweep 31 + 31b (combined
`better`, +18.1 Elo bot vs bot):

- served: `py/serve.py --strong h0:nodes=32000,horizon=3,k=8 --cheat h0:nodes=32000,horizon=3,k=8,info=all`
- all 8 games vs the fair spec `h0:nodes=32000,horizon=3,k=8` (no cheater games)

**Bot won 2 of 8.**

| # | game | owner side | owner deck (= bot deck) | first | winner | actions | finished (UTC) |
|---|---|---|---|---|---|---|---|
| 1 | `17060962360713023452-5ce21003` | a | meta-abyss-midrange | bot | owner | 58 | 2026-10-02T11:02:35Z |
| 2 | `8752271578818342984-5ce21003` | a | meta-abyss-midrange | owner | owner | 77 | 2026-10-02T11:10:08Z |
| 3 | `11568809992822859365-5ce21003` | a | meta-abyss-midrange | owner | owner | 66 | 2026-10-02T11:13:02Z |
| 4 | `2441626901096349772-5ce21003` | a | meta-abyss-midrange | bot | **bot** | 92 | 2026-10-02T11:44:10Z |
| 5 | `15985272443408746742-5ce21003` | a | meta-abyss-midrange | owner | **bot** | 100 | 2026-10-02T11:52:20Z |
| 6 | `9016811404352548057-5ce21003` | a | meta-abyss-midrange | owner | owner | 72 | 2026-10-02T11:58:51Z |
| 7 | `15586532504811393507-5ce21003` | a | meta-abyss-midrange | owner | owner | 53 | 2026-10-02T12:04:22Z |
| 8 | `9660812344611175472-5ce21003` | a | meta-abyss-midrange | bot | owner | 62 | 2026-10-02T12:09:12Z |

`games/<game_id>.json` are the raw serve.py captures, byte for byte (`actions` index 0 and 1 are the mulligans; no
`reseed` steps). Context, mirror games vs fair bots: review3/4 1/12 (v2, k=4, 16 000 nodes), review6 3/7 (v2, k=4,
16 000 nodes, a69b248). Sweep 33 was running on the same box during these games (node-limited search: the bot's
choices are unaffected, only its thinking time).
