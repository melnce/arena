"""review12 game 6, action 89 (the bot's end_turn on turn 8): served spec on the current build with the v3 leaf pinned
(reproduces the served bot of 3742a1b), the v4 default leaf (the bot served from ed574a4 on), and full information.
Prints the root values per candidate and the explain 'deal' (the sampled hidden cards) for v3 seed 1.
usage (repo root): python s12_worlds.py"""
import json
import arena

G = 'results/games/4280399497766595080-104cf827.json'
BASE = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))
g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
for s in d['actions'][:89]:
    g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
short = lambda a: ('end_turn' if 'end_turn' in a else
                   f"attack slot{a['attack']['attacker_slot']}->{a['attack']['target'] if a['attack']['target'] == 'leader' else 'slot' + str(a['attack']['target']['slot'])}")
for label, spec in (('v3 (served on 3742a1b)', BASE + ',net=engine/models/h0-linear-v3.json'),
                    ('v4 (default from ed574a4)', BASE),
                    ('v3, info=all', BASE + ',info=all,net=engine/models/h0-linear-v3.json')):
    for seed in (1, 2, 3):
        e = g.bot_action_explain(spec, seed); e = json.loads(e) if isinstance(e, str) else e
        row = '  '.join(f"{short(c['action'])} {c['root_agg']:.2f}/{c['worst']:.2f}" for c in e['candidates'])
        ends = sorted({w.get('end') for c in e['candidates'] for w in c['worlds']})
        print(f"{label} seed {seed}: path {e.get('path')} chosen {short(e['chosen'])} | {row} | ends {ends}", flush=True)
        if label.startswith('v3 (') and seed == 1:
            print('   deal:', json.dumps(e.get('deal'))[:3000], flush=True)
            print('   hread:', json.dumps(e.get('hread'))[:600], '| wbase:', json.dumps(e.get('wbase'))[:300], flush=True)
