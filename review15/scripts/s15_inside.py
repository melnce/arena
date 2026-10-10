"""review15: how the bot values the Mars board clear once it is inside the line, and how safe the clear is beyond the
realized RNG. For each (game, turn start, safe line from safe_lines.json):
- after Mars + the first Steelclad Knight, and at the clear's last decision (before its final attack): explain at the
  seed the browser would send for that decision (game seed + the bot's request count on that line) and at seeds 1-2;
  prints the chosen action, its value, and how many of the 8 sampled worlds end in an owner kill for the chosen
  candidate and for the line's next action;
- the clear replayed after reseeding the game RNG at the turn start (seeds 1, 2, 3, 12345): the owner's turn-start draw
  and whether the owner then has a kill: the engine's forced_lethal (budget 200 000; misses kills that fuse with
  partners) and the fuse-aware search (dfs2.py).
usage (repo root): python s15_inside.py SAFE_LINES.json GAME_ID:FIRST_PLY [...]"""
import json, sys, glob, os
import arena
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dfs2 import can_win

SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
A = lambda v: v() if callable(v) else v
J = lambda a: json.loads(a) if isinstance(a, str) else a
db = arena.load_cards('cards')
names = {}


def name(cid):
    if cid not in names:
        p = glob.glob(f'cards/*/{cid}.json')
        names[cid] = json.load(open(p[0], encoding='utf-8')).get('name') if p else cid
    return names[cid]


def short(a):
    if 'end_turn' in a: return 'end_turn'
    if 'attack' in a:
        t = a['attack']['target']
        return f"attack slot{a['attack']['attacker_slot']}->{'leader' if t == 'leader' else 'enemy slot' + str(t['slot'])}"
    if 'play' in a: return f"play {name(a['play']['card'])}"
    return json.dumps(a)[:50]


def kills(c):
    return sum(1 for w in c['worlds'] if w.get('end') in ('opp_lethal', 'opp_solver'))


lines = json.load(open(sys.argv[1], encoding='utf-8'))
for arg in sys.argv[2:]:
    gid, ply = arg.split(':'); ply = int(ply); line = lines[gid]
    d = json.load(open(f'results/games/{gid}.json', encoding='utf-8'))
    acts = [{k: v for k, v in s.items() if k not in ('value', 'bot_value')} for s in d['actions']]

    def at_start():
        g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
        for a in acts[:ply]:
            g.apply(a)
        return g
    k0 = sum(1 for a in acts[:ply] if isinstance(list(a.values())[0], dict) and list(a.values())[0].get('player') == 'b')
    print(f'== {gid}, bot turn starting after {ply} actions; the clear: {[short(a) for a in line]}', flush=True)
    for label, n in (('after Mars + Knight', 2), ("at the clear's last decision", len(line) - 2)):
        g = at_start()
        for a in line[:n]:
            g.apply(a)
        nxt = line[n]
        for seed in (int(d['seed']) + k0 + n, 1, 2):
            e = J(g.bot_action_explain(SPEC, seed))
            ch = next(c for c in e['candidates'] if c['action'] == e['chosen']) if e.get('candidates') else None
            nc = next((c for c in e.get('candidates') or [] if c['action'] == nxt), None)
            sl = 'line seed' if seed > 1000 else f'seed {seed}'
            chs = f' ({kills(ch)}/8 worlds end in an owner kill)' if ch else ''
            ncs = '%.2f (%d/8)' % (nc['root_agg'], kills(nc)) if nc else 'n/a'
            print(f"  {label}, {sl}: chosen {short(e['chosen'])} {e['value']:.2f}{chs}; the line's next move {short(nxt)}: {ncs}", flush=True)
    for rs in (1, 2, 3, 12345):
        g = at_start(); g.reseed(rs)
        ok = True
        for a in line:
            try:
                g.apply(a)
            except Exception as x:
                ok = False; break
        if not ok:
            print(f'  reseed {rs}: the clear is not legal under this RNG', flush=True); continue
        f = J(g.full())
        drawn = f['players']['a']['hand'][-1]['name'] if f['players']['a']['hand'] else None
        v = arena.forced_lethal(g, 200_000)
        w, n, _ = can_win(g, limit=5_000_000)
        print(f"  reseed {rs} at the turn start: owner draws {drawn!r}; after the clear: engine forced_lethal {v['verdict']}"
              f"{' (rng-dependent)' if v.get('rng_dependent') else ''}; fuse-aware owner win: {w} ({n} nodes)", flush=True)
