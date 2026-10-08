"""review13, game 1, turn 6: the bot's root value of 'Skeleton face' (then End Turn, its only move) in two positions whose
results are identical except that the owner keeps the 2/1 Test Subject: the actual line (after 40 actions: the 6/4 hit
and killed by the Feline) and the alternative (the evolve's 3 damage on the 2/1, the Feline into the 6/4). End Turn
alone is a forced move without a search value, so the comparison is one step earlier. Same spec, same seeds,
info=fair (served) and info=all.
usage (repo root): python s13_endturn.py [N=8]"""
import json, sys
import arena

G = 'results/games/4984932781931433298-b4973563.json'
SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
N = int(sys.argv[1]) if len(sys.argv) > 1 else 8
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))


def at(n):
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def board(g):
    f = g.full(); f = json.loads(f) if isinstance(f, str) else f
    return {s: (f['players'][s]['leader_defense'], [f"{c['name']} {c['attack']}/{c['defense']}" for c in f['players'][s]['field'] if c]) for s in 'ab'}


def et(e):
    """root value of the Skeleton (attacker slot 0) attacking the leader"""
    return next(c['root_agg'] for c in e['candidates'] if c['action'] == {'attack': {'attacker_slot': 0, 'player': 'b', 'target': 'leader'}})


act = at(40)
alt = at(38)
for a in ({'choose': {'option': {'slot': 0}, 'player': 'b'}}, {'attack': {'attacker_slot': 1, 'player': 'b', 'target': {'slot': 0}}}):
    alt.apply(a)
print('actual     :', board(act)); print('alternative:', board(alt))
for name, spec in (('served (info=fair)', SPEC), ('info=all', SPEC + ',info=all')):
    diffs = []
    for seed in range(1, N + 1):
        va = {json.dumps(c['action']): c['root_agg'] for c in json.loads(json.dumps(act.bot_action_explain(spec, seed)))['candidates']} \
            if not isinstance(act.bot_action_explain(spec, seed), str) else None
        ea = act.bot_action_explain(spec, seed); ea = json.loads(ea) if isinstance(ea, str) else ea
        eb = alt.bot_action_explain(spec, seed); eb = json.loads(eb) if isinstance(eb, str) else eb
        ta, tb = et(ea), et(eb)
        diffs.append(tb - ta)
        print(f'  {name} seed {seed}: Skeleton face with the owner\'s 2/1 alive {ta:.2f} | without it {tb:.2f} | difference {tb - ta:+.2f}', flush=True)
    print(f'  {name}: mean difference (without - with) {sum(diffs) / len(diffs):+.2f}; without-it better in {sum(x > 0 for x in diffs)}/{len(diffs)} seeds', flush=True)
