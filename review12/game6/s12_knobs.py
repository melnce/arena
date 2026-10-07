"""review12 game 6, action 89: does an existing opponent-model knob make the served bot (v3 leaf pinned) see the owner's
turn-9 lethal and kill Sephie? Seeds 1-3 per variant; prints the chosen action, the Sephie/end_turn root values, the
world ends seen. usage (repo root): python s12_knobs.py [extra-key-set ...]   (default: the five sets in knobs.txt)"""
import json
import arena

G = 'results/games/4280399497766595080-104cf827.json'
BASE = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=engine/models/h0-linear-v3.json'
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))
g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
for s in d['actions'][:89]:
    g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
short = lambda a: ('end_turn' if 'end_turn' in a else
                   f"attack slot{a['attack']['attacker_slot']}->{a['attack']['target'] if a['attack']['target'] == 'leader' else 'slot' + str(a['attack']['target']['slot'])}")
import sys
EXTRAS = sys.argv[1:] or ['okill=7', 'olsolve=4000', 'omacro=1', 'okill=7,olsolve=4000,omacro=1', 'info=all,okill=7,olsolve=4000,omacro=1']
for extra in EXTRAS:
    for seed in (1, 2, 3):
        try:
            e = g.bot_action_explain(BASE + ',' + extra, seed); e = json.loads(e) if isinstance(e, str) else e
        except Exception as x:
            print(f'{extra} seed {seed}: ERROR {x}', flush=True); break
        row = '  '.join(f"{short(c['action'])} {c['root_agg']:.2f}" for c in e['candidates'])
        ends = {}
        for c in e['candidates']:
            if 'end_turn' in c['action']:
                for w in c['worlds']:
                    ends[w.get('end')] = ends.get(w.get('end'), 0) + 1
        print(f"{extra} seed {seed}: path {e.get('path')} chosen {short(e['chosen'])} | {row} | end_turn world ends {ends}", flush=True)
