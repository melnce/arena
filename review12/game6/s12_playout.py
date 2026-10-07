"""review12 game 6: from the bot's End Turn moment (after 89 actions, bot to move on turn 8), let the bot finish its turn
with a given spec and seed (one explain per decision, chosen action applied), then search the owner's turn 9
for a win: first the owner's actual line by card (Tetra & Ladica, Humane Love, an evolve, then every face attack),
then a full search (as lethal_dfs.py) that tries face attacks first, then plays, then evolves; exhaustive when it
finds no win (no cap reached). Served spec = v3 leaf pinned (3742a1b's default); variant = + olsolve=4000.
usage (repo root): python s12_playout.py [seeds]"""
import json, sys
import arena

G = 'results/games/4280399497766595080-104cf827.json'
SERVED = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=engine/models/h0-linear-v3.json'
A = lambda v: v() if callable(v) else v
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))


def at(n):
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def J(a):
    return json.loads(a) if isinstance(a, str) else a


def face_all(s):
    while A(s.winner) is None:
        att = [a for a in map(J, s.legal()) if 'attack' in a and a['attack']['target'] == 'leader']
        if not att:
            break
        s.apply(att[0])
    return A(s.winner) == 'a'


def actual_line(g):
    s = g.clone()
    for card in ('10834110', '10932310'):
        pl = [a for a in map(J, s.legal()) if 'play' in a and a['play']['card'] == card]
        if not pl:
            return False
        s.apply(pl[0])
    evos = [a for a in map(J, s.legal()) if 'evolve' in a]
    for ev in evos + [None]:
        s2 = s.clone()
        if ev is not None:
            s2.apply(ev)
        if face_all(s2):
            return True
    return False


def rank(a):
    if 'attack' in a:
        return 0 if a['attack']['target'] == 'leader' else 3
    return 1 if 'play' in a else 2 if 'evolve' in a else 4


def owner_can_win(g, limit=3_000_000):
    me, t0 = A(g.active), A(g.turn)
    seen = set(); cnt = [0]

    def dfs(s):
        if cnt[0] >= limit:
            return False
        cnt[0] += 1
        if A(s.winner) == me:
            return True
        if A(s.winner) is not None or A(s.terminal) or A(s.turn) != t0 or A(s.active) != me:
            return False
        h = s.hash()
        if h in seen:
            return False
        seen.add(h)
        for a in sorted(map(J, s.legal()), key=rank):
            if 'end_turn' in a:
                continue
            s2 = s.clone()
            try:
                s2.apply(a)
            except Exception:
                continue
            if dfs(s2):
                return True
        return False
    return dfs(g), cnt[0], cnt[0] >= limit


short = lambda a: ('end_turn' if 'end_turn' in a else
                   f"attack slot{a['attack']['attacker_slot']}->{a['attack']['target'] if a['attack']['target'] == 'leader' else 'slot' + str(a['attack']['target']['slot'])}"
                   if 'attack' in a else json.dumps(a))
seeds = [int(x) for x in sys.argv[1:]] or [1, 2, 3, 4, 5, 6]
for label, spec in (('served', SERVED), ('olsolve=4000', SERVED + ',olsolve=4000')):
    lost = 0
    for seed in seeds:
        g = at(89); line = []
        for _ in range(8):
            if A(g.active) != 'b' or A(g.turn) != 8 or A(g.winner) is not None:
                break
            e = g.bot_action_explain(spec, seed); e = json.loads(e) if isinstance(e, str) else e
            line.append(short(e['chosen'])); g.apply(e['chosen'])
        f = g.full(); f = json.loads(f) if isinstance(f, str) else f
        sephie = any(c and c.get('card') == '10934110' for c in f['players']['a']['field'])
        if actual_line(g):
            r, cnt, cap = True, 0, False
        else:
            r, cnt, cap = owner_can_win(g)
        lost += int(r)
        print(f"{label} seed {seed}: bot line {line} | Sephie alive {sephie} | owner win on turn 9: {r} ({'the actual line by card' if r and cnt == 0 else str(cnt) + ' states'}{', CAP' if cap else ''})", flush=True)
    print(f'{label}: owner wins on turn 9 after {lost} of {len(seeds)} bot seeds', flush=True)
