"""review13, game 1: at the bot's decisions after 27 (turn 5 End Turn vs the Feline attacking the 2/1), 38 (turn 6, the
evolve's target: the 6/4 vs the 2/1) and 40 (the Skeleton: face vs the 2/1), count the choices of the served spec
(deal=indep, the default) and of the same spec with deal=block (the opponent's unknown hand dealt to the k worlds from
one shared shuffle, without replacement across worlds; in both modes every root candidate is searched in the same k
worlds), over the served seed and seeds 1..N.
usage (repo root): python s13_deal.py [N=16]"""
import json, sys
from collections import Counter
import arena

G = 'results/games/4984932781931433298-b4973563.json'
SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
N = int(sys.argv[1]) if len(sys.argv) > 1 else 16
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))


def at(n):
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def bot_index(n):
    return sum(1 for st in d['actions'][:n] for v in [[v for kk, v in st.items() if kk not in ('value', 'bot_value')][0]]
               if isinstance(v, dict) and v.get('player') == 'b')


def short(a):
    if 'end_turn' in a: return 'end_turn'
    if 'attack' in a:
        t = a['attack']['target']
        return f"attack slot{a['attack']['attacker_slot']}->{'leader' if t == 'leader' else 'enemy slot' + str(t['slot'])}"
    if 'choose' in a: return 'choose ' + json.dumps(a['choose']['option'])
    return json.dumps(a)[:60]


for n in (27, 38, 40):
    g = at(n); served = int(d['seed']) + bot_index(n)
    for label, spec in (('deal=indep (served)', SPEC), ('deal=block', SPEC + ',deal=block')):
        c = Counter(); sv = short(g.bot_action_value(spec, served)['action'])
        for seed in range(1, N + 1):
            c[short(g.bot_action_value(spec, seed)['action'])] += 1
        print(f'after {n} actions, {label}: served seed -> {sv}; seeds 1-{N}: {dict(c)}', flush=True)
