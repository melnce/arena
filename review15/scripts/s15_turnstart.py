"""review15: at the first decision of a bot turn that ended handing the owner a forced kill, the bot's view at the exact
served seed: root candidates with their values and how many of each candidate's sampled worlds ended in an opponent
kill (opp_lethal / opp_solver); then a safe line (from s15_safe.py) replayed with the owner's forced_lethal at a larger
budget (200 000) to confirm 'none'.
usage (repo root): python s15_turnstart.py SAFE_LINES.json GAME_ID:FIRST_PLY [...]   (SAFE_LINES.json: {game_id: [actions]})"""
import json, sys, os
import arena
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dfs2 import can_win

SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
J = lambda a: json.loads(a) if isinstance(a, str) else a
A = lambda v: v() if callable(v) else v
db = arena.load_cards('cards')
names = {}


def name(cid):
    if cid not in names:
        import glob
        p = glob.glob(f'cards/*/{cid}.json')
        names[cid] = json.load(open(p[0], encoding='utf-8')).get('name') if p else cid
    return names[cid]


def short(a):
    if 'end_turn' in a: return 'end_turn'
    if 'attack' in a:
        t = a['attack']['target']
        return f"attack slot{a['attack']['attacker_slot']}->{'leader' if t == 'leader' else 'enemy slot' + str(t['slot'])}"
    if 'play' in a: return f"play {name(a['play']['card'])}"
    if 'evolve' in a: return f"{'super-' if a['evolve'].get('super') else ''}evolve slot{a['evolve']['slot']}"
    return json.dumps(a)[:50]


safe_lines = json.load(open(sys.argv[1], encoding='utf-8'))
for arg in sys.argv[2:]:
    gid, ply = arg.split(':'); ply = int(ply); line = safe_lines[gid]
    d = json.load(open(f'results/games/{gid}.json', encoding='utf-8'))
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    k = 0
    for s in d['actions'][:ply]:
        a = {kk: v for kk, v in s.items() if kk not in ('value', 'bot_value')}
        v = list(a.values())[0]
        k += isinstance(v, dict) and v.get('player') == 'b'
        g.apply(a)
    e = J(g.bot_action_explain(SPEC, int(d['seed']) + k))
    print(f"== {gid} first decision of the turn (after {ply} actions), served seed: path {e.get('path')}, chosen {short(e['chosen'])}", flush=True)
    for c in sorted(e['candidates'], key=lambda c: -c['root_agg']):
        ends = {}
        for w in c['worlds']:
            ends[w.get('end')] = ends.get(w.get('end'), 0) + 1
        kill = ends.get('opp_lethal', 0) + ends.get('opp_solver', 0)
        print(f"   {short(c['action']):40s} {c['root_agg']:8.2f}  worlds ending in an opponent kill: {kill}/{len(c['worlds'])}", flush=True)
    g2 = g.clone()
    for a in line:
        g2.apply(a)
    f = J(g2.full())
    fl = arena.forced_lethal(g2, 200_000)
    w, n, wl = can_win(g2, limit=5_000_000)
    print(f"   safe line {[short(a) for a in line]} -> engine forced_lethal: {fl['verdict']} (nodes {fl['nodes']}); "
          f"fuse-aware owner win: {w} ({n} nodes){' via ' + str([short(x) if 'fuse' not in x else 'fuse onto hand card ' + str(x['fuse']['host_pos']) for x in wl]) if w else ''}; "
          f"after it: bot {f['players']['b']['leader_defense']} HP, field {[c['name'] + ' ' + str(c['attack']) + '/' + str(c['defense']) for c in f['players']['b']['field'] if c]}; "
          f"owner field {[c['name'] + ' ' + str(c['attack']) + '/' + str(c['defense']) for c in f['players']['a']['field'] if c]}", flush=True)
