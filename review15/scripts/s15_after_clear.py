"""review15: after the bot's Mars board clear (from safe_lines.json), the owner's next turn: resources, hand, the forced
kill check (budget 500 000; NOTE: it misses kills that fuse with partners), the fuse-aware win search and most face damage
(dfs2.py: exhaustive, one partner per fuse), and whether Alchemic Flare is castable with no enemy follower.
usage (repo root): python s15_after_clear.py SAFE_LINES.json GAME_ID:FIRST_PLY [...]"""
import json, sys, os
import arena
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dfs2 import can_win, max_face

A = lambda v: v() if callable(v) else v
J = lambda a: json.loads(a) if isinstance(a, str) else a
db = arena.load_cards('cards')
lines = json.load(open(sys.argv[1], encoding='utf-8'))
for arg in sys.argv[2:]:
    gid, ply = arg.split(':'); ply = int(ply)
    d = json.load(open(f'results/games/{gid}.json', encoding='utf-8'))
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:ply]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    for a in lines[gid]:
        g.apply(a)
    f = J(g.full()); pa, pb = f['players']['a'], f['players']['b']
    print(f'== {gid}: after the Mars clear, the owner to move (turn {A(g.turn)})')
    print(f"  owner: {pa['leader_defense']} HP, PP {pa['pp']}/{pa['pp_max']}, evolve points {pa.get('ep')}, super-evolve points {pa.get('sep')}, field {[c['name'] for c in pa['field'] if c]}")
    print(f"  owner's hand: {[c['name'] for c in pa['hand']]}")
    print(f"  bot: {pb['leader_defense']} HP, field {[c['name'] for c in pb['field'] if c]}")
    fl = arena.forced_lethal(g, 500_000)
    print(f"  engine forced_lethal: {fl['verdict']} (nodes {fl['nodes']})")
    w, n, ln = can_win(g, limit=5_000_000)
    print(f"  owner's win (fuse-aware): {w} ({n} nodes){' via ' + str([json.dumps(x) for x in ln]) if w else ''}")
    mf, n2, ln2, ex = max_face(g, limit=5_000_000)
    print(f"  most face damage the owner can deal this turn (fuse-aware): {mf} of {pb['leader_defense']} ({n2} states, {'exhaustive' if ex else 'capped'})")
    print(f"  Alchemic Flare castable: {any('play' in a and a['play']['card'] == '10433310' for a in map(J, g.legal()))}")
