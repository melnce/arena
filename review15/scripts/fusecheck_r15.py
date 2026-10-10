"""review15 re-check with the fuse-aware search (dfs2.py), after the engine's forced_lethal was found to miss kills that
fuse with partners (pos_key collision during the partner Choice):
1. after each Mars clear (safe_lines.json): can the owner win, and the most face damage (exhaustive);
2. every end-of-turn position of the two 'avoidable' turns that forced_lethal called 'none' (s15_safe.py's DFS, same
   order and cap): can the owner win after it?
3. every bot End Turn the lethal audit called 'none' or 'unknown' (handed_lethal): can the owner win after it?
4. whether the bot's deck (Sword Rally) has any card with Fuse (if none, the bot-side missed-kill verdicts stand).
usage (repo root): python fusecheck_r15.py REVIEW15_DIR"""
import json, sys, glob, os
import arena
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dfs2 import can_win, max_face, A, J

R = sys.argv[1]
db = arena.load_cards('cards')
lines = json.load(open(os.path.join(R, 'safe_lines.json'), encoding='utf-8'))
GAMES = {}


def game(gid):
    if gid not in GAMES:
        GAMES[gid] = json.load(open(f'results/games/{gid}.json', encoding='utf-8'))
    return GAMES[gid]


def at(gid, n):
    d = game(gid)
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def sh(a):
    return json.dumps(a)[:90]


print('== 1. after each Mars clear', flush=True)
for gid, start in (('1068176807631359990-b43e6214', 86), ('2374435823510868837-b43e6214', 129)):
    g = at(gid, start)
    for a in lines[gid]:
        g.apply(a)
    w, n, ln = can_win(g)
    mf, n2, ln2, ex = max_face(g)
    print(f'  {gid}: owner can win: {w} ({n} nodes){" via " + str([sh(x) for x in ln]) if w else ""}; most face damage {mf} ({"exhaustive" if ex else "capped"})', flush=True)

print('== 2. the none-leaves of the two avoidable turns', flush=True)
for gid, start, t_end in (('1068176807631359990-b43e6214', 86, 9), ('2374435823510868837-b43e6214', 129, 11)):
    g0 = at(gid, start)
    seen = set(); leaves = []; capped = [False]

    def dfs(s, line):
        if len(leaves) >= 4000:
            capped[0] = True; return
        if A(s.winner) is not None or A(s.turn) != t_end or A(s.active) != 'b':
            return
        h = s.hash()
        if h in seen:
            return
        seen.add(h)
        for a in map(J, s.legal()):
            s2 = s.clone(); s2.apply(a)
            if 'end_turn' in a:
                leaves.append((s2, line + [a]))
                if len(leaves) >= 4000:
                    capped[0] = True; return
            else:
                dfs(s2, line + [a])
    dfs(g0, [])
    nones = [(s2, ln) for s2, ln in leaves if A(s2.winner) is None and arena.forced_lethal(s2, 20_000)['verdict'] == 'none']
    print(f'  {gid}: {len(leaves)} leaves, forced_lethal none at {len(nones)}', flush=True)
    for s2, ln in nones:
        w, n, wl = can_win(s2)
        print(f'    fuse-aware: owner can win {w} ({n} nodes){" via " + str([sh(x) for x in wl]) if w else ""}', flush=True)

print('== 3. lethal-audit End Turns with verdict none/unknown', flush=True)
au = json.load(open(os.path.join(R, 'lethal_audit.json'), encoding='utf-8'))
found = 0; checked = 0
for rec in au['records']:
    gid = [f for f in os.listdir(os.path.join(R, 'games')) if f.startswith(str(rec['seed']))][0][:-5]
    for dec in rec.get('decisions', []):
        if dec.get('kind') != 'handed_lethal' or dec.get('verdict') == 'lethal':
            continue
        g = at(gid, dec['ply'] + 1)
        w, n, wl = can_win(g)
        checked += 1; found += bool(w)
        print(f"  {gid} ply {dec['ply']} (engine {dec.get('verdict')}): owner can win {w} ({n} nodes){' via ' + str([sh(x) for x in wl]) if w else ''}", flush=True)
print(f'  re-checked {checked}; fuse-aware kills found {found}', flush=True)

print('== 4. Fuse cards in the bot deck', flush=True)
d = game('6065548863054576576-b43e6214')
fz = []
for cid in d['deckB']:
    for p in glob.glob(f'cards/*/{cid}.json'):
        c = json.load(open(p, encoding='utf-8'))
        if 'Fuse' in (c.get('text') or ''):
            fz.append(c.get('name'))
print(f'  Sword Rally cards with Fuse: {fz or "none"}', flush=True)
