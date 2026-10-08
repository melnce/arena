"""review13, game 1 (4984932781931433298-b4973563), the bot's turn 6: Highwire Feline's fanfare kills the owner's evolved
7/1 Test Subject; the evolve's 3 damage goes to the 6/4 Test Subject (-> 6/1), the 6/7 Feline then attacks that 6/1, and
the Skeleton goes face, leaving the owner's 2/1 Test Subject alive.
Root values at the decisions after 27 actions (turn 5: End Turn with the evolved Feline able to attack), 38 (the evolve target), 39 (the Feline's attack) and 40 (the Skeleton's
attack), served spec (ed574a4 defaults = v4 leaf), at the EXACT served seed first (the UI sends botSeed = the game's seed + the
index of the bot request, ui/src/session.ts nextBotSeed; this reproduces every recorded bot_value of game 1), then seeds
1-3 and info=all.
usage (repo root): python s13_g1t6.py [N ...]   (default: 27 38 39 40)"""
import json
import arena

G = 'results/games/4984932781931433298-b4973563.json'
SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json'
A = lambda v: v() if callable(v) else v
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))


def at(n):
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def short(a):
    if 'end_turn' in a: return 'end_turn'
    if 'attack' in a:
        t = a['attack']['target']
        return f"attack slot{a['attack']['attacker_slot']}->{'leader' if t == 'leader' else 'enemy slot' + str(t['slot'])}"
    if 'choose' in a: return 'choose ' + json.dumps(a['choose']['option'])
    if 'play' in a: return f"play {a['play']['card']}"
    if 'evolve' in a: return f"evolve slot{a['evolve']['slot']}{' super' if a['evolve'].get('super') else ''}"
    return json.dumps(a)[:60]


import sys
NS = [int(x) for x in sys.argv[1:]] or [27, 38, 39, 40]
def bot_index(n):
    k = 0
    for st in d['actions'][:n]:
        v = [v for kk, v in st.items() if kk not in ('value', 'bot_value')][0]
        k += isinstance(v, dict) and v.get('player') == 'b'
    return k


for n in NS:
    g = at(n)
    served_seed = int(d['seed']) + bot_index(n)
    rec = d['actions'][n]
    print(f"== decision after {n} actions; the bot played: {short({k: v for k, v in rec.items() if k not in ('value', 'bot_value')})} "
          f"(recorded bot_value {rec.get('bot_value')})", flush=True)
    for label, spec in (('served', SPEC), ('info=all', SPEC + ',info=all')):
        for seed in ((served_seed, 1, 2, 3) if label == 'served' else (served_seed, 1)):
            e = g.bot_action_explain(spec, seed); e = json.loads(e) if isinstance(e, str) else e
            rows = sorted(e['candidates'], key=lambda c: -c['root_agg'])
            sl = f'{seed} (= the served seed)' if seed == served_seed else str(seed)
            print(f"  {label} seed {sl}: path {e.get('path')} chosen {short(e['chosen'])} | " +
                  '  '.join(f"{short(c['action'])} {c['root_agg']:.2f}/{c['worst']:.2f}" for c in rows), flush=True)
