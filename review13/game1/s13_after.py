"""review13, game 1, turn 6: the bot's next decision in the actual line (after 39 actions: the 6/4 was hit, now 6/1) and
in the alternative (the evolve's 3 damage on the 2/1 instead, which dies; the 6/4 is now enemy slot 0), at the seed the
served bot used for that decision (the game's seed + 20) and seeds 1-3, plus info=all. If the search valued boards
monotonically, the alternative's best continuation (Feline into the 6/4) should be worth at least the actual line's.
usage (repo root): python s13_after.py"""
import json
import arena

G = 'results/games/4984932781931433298-b4973563.json'
SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
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
    return json.dumps(a)[:60]


served = int(d['seed']) + 20
act = at(39)
alt = at(38); alt.apply({'choose': {'option': {'slot': 0}, 'player': 'b'}})
for label, g in (('actual (6/4 hit, now 6/1 in enemy slot 1; the 2/1 in slot 0)', act), ('alternative (2/1 killed; the 6/4 in enemy slot 0)', alt)):
    print('==', label, flush=True)
    for name, spec, seeds in (('served', SPEC, (served, 1, 2, 3)), ('info=all', SPEC + ',info=all', (served, 1))):
        for seed in seeds:
            e = g.bot_action_explain(spec, seed); e = json.loads(e) if isinstance(e, str) else e
            rows = sorted(e['candidates'], key=lambda c: -c['root_agg'])
            sl = f'{seed} (served)' if seed == served else str(seed)
            print(f"  {name} seed {sl}: chosen {short(e['chosen'])} value {e['value']:.2f} | " +
                  '  '.join(f"{short(c['action'])} {c['root_agg']:.2f}" for c in rows), flush=True)
