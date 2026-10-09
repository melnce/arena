"""review14: the held-back moments the scan flags (the bot's End Turn while a follower could still make a free kill).
For each: the root values at the exact served seed (game seed + the bot's earlier actions; ui/src/session.ts nextBotSeed),
the same seed with info=all, the served hbcheck's own End' vs kill-attack A' aggregates (explain 'holdback'), and the
choice over seeds 1..N.
usage (repo root): python s14_hb.py HB.jsonl [N=8]"""
import json, sys
from collections import Counter
import arena

SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
N = int(sys.argv[2]) if len(sys.argv) > 2 else 8
db = arena.load_cards('cards')


def short(a):
    if 'end_turn' in a: return 'end_turn'
    if 'attack' in a:
        t = a['attack']['target']
        return f"attack slot{a['attack']['attacker_slot']}->{'leader' if t == 'leader' else 'enemy slot' + str(t['slot'])}"
    return json.dumps(a)[:60]


for line in open(sys.argv[1], encoding='utf-8'):
    r = json.loads(line)
    if 'ply' not in r:
        continue
    d = json.load(open(f"results/games/{r['game']}.json", encoding='utf-8'))
    n = r['ply']
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    k = 0
    for s in d['actions'][:n]:
        a = {kk: v for kk, v in s.items() if kk not in ('value', 'bot_value')}
        v = list(a.values())[0]
        k += isinstance(v, dict) and v.get('player') == 'b'
        g.apply(a)
    served = int(d['seed']) + k
    rec = {kk: v for kk, v in d['actions'][n].items() if kk not in ('value', 'bot_value')}
    print(f"== {r['game']} ply {n} turn {r['turn']}: X {r['primary']['x']} {r['primary']['x_attack']}/{r['primary']['x_defense']} -> Y {r['primary']['y']}; "
          f"played {short(rec)} (bot_value {d['actions'][n].get('bot_value')})", flush=True)
    for name, spec in (('served', SPEC), ('info=all', SPEC + ',info=all')):
        e = g.bot_action_explain(spec, served); e = json.loads(e) if isinstance(e, str) else e
        rows = sorted(e['candidates'], key=lambda c: -c['root_agg'])
        print(f"  {name} at the served seed: path {e.get('path')} chosen {short(e['chosen'])} | " +
              '  '.join(f"{short(c['action'])} {c['root_agg']:.2f}" for c in rows[:6]), flush=True)
        hb = e.get('holdback')
        if name == 'served' and hb:
            legal = [json.loads(x) if isinstance(x, str) else x for x in g.legal()]
            print(f"  hbcheck ran (docs/engine-api.md: the kill is played when its A' aggregate beats End'): End' {hb['end_prime']:.2f} vs " +
                  '  '.join(f"{short(legal[a['legal_index']])} A' {a['aggregate']:.2f}" for a in hb['attacks']), flush=True)
        elif name == 'served':
            print('  hbcheck: no holdback record in the explain', flush=True)
    c = Counter(short(g.bot_action_value(SPEC, s)['action']) for s in range(1, N + 1))
    print(f'  served spec over seeds 1-{N}: {dict(c)}', flush=True)
