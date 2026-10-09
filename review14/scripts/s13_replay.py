"""Replay every bot decision of review13's captures with the served spec at the served seed (the UI sends botSeed = the
game's seed + the index of the bot request: ui/src/session.ts nextBotSeed) and compare the action and its bot_value with
the capture. Decisions without a recorded bot_value are compared on the action only.
usage (repo root): python s13_replay.py GAME.json [GAME.json ...]"""
import json, sys
import arena

SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
db = arena.load_cards('cards')
for path in sys.argv[1:]:
    d = json.load(open(path, encoding='utf-8'))
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    k = 0; same_v = same_a = n_v = n_a = 0; bad = []
    for i, s in enumerate(d['actions']):
        a = {kk: v for kk, v in s.items() if kk not in ('value', 'bot_value')}
        v = list(a.values())[0]
        if isinstance(v, dict) and v.get('player') == 'b':
            r = g.bot_action_value(SPEC, int(d['seed']) + k)
            if s.get('bot_value') is not None:
                n_v += 1
                ok = r['action'] == a and abs(r['value'] - s['bot_value']) < 1e-6
                same_v += ok
            else:
                n_a += 1
                ok = r['action'] == a
                same_a += ok
            if not ok:
                bad.append((i, json.dumps(a)[:60], json.dumps(r['action'])[:60]))
            k += 1
        g.apply(a)
    print(f"{path.split('/')[-1].split(chr(92))[-1]}: with a recorded value {same_v}/{n_v} same action and value; "
          f"without {same_a}/{n_a} same action; differing: {bad}", flush=True)
