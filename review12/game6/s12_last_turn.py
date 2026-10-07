"""review12, last game (4280399497766595080-104cf827): the bot's final turn (turn 8).
1. explain: the served spec's root values at the moment it ended the turn (after 89 actions), for a few bot seeds.
2. draws: with Sephie killed (both rush Test Subjects attack her, then end_turn), reseed the game RNG before end_turn so
   the owner's turn-9 draw varies, and search the owner's turn exhaustively for a win (as lethal_dfs.py).
3. cover: as draws, but keeps reseeding (16, 17, ...) until each distinct card left in the owner's deck that draws did
   not reach has been drawn once, and searches those.
usage (repo root): python s12_last_turn.py explain|draws|cover"""
import json, sys
import arena

G = 'results/games/4280399497766595080-104cf827.json'
SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=engine/models/h0-linear-v3.json'  # v3 = 3742a1b's default leaf
A = lambda v: v() if callable(v) else v
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))


def at(n):
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def owner_can_win(g, limit=1_500_000):
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
        for a in s.legal():
            a = json.loads(a) if isinstance(a, str) else a
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
    r = dfs(g)
    return r, cnt[0], cnt[0] >= limit


mode = sys.argv[1] if len(sys.argv) > 1 else 'explain'
if mode == 'explain':
    g = at(89)
    for seed in (1, 2, 3):
        e = g.bot_action_explain(SPEC, seed)
        e = json.loads(e) if isinstance(e, str) else e
        print(f"seed {seed}: path {e.get('path')}, chosen {json.dumps(e.get('chosen'))}, value {e.get('value')}, nodes {e.get('nodes')}")
        for c in e.get('candidates', []):
            ends = {}
            for w in c.get('worlds', []):
                ends[w.get('end')] = ends.get(w.get('end'), 0) + 1
            print(f"   {json.dumps(c['action'])[:90]:92s} root_agg {c.get('root_agg'):8.2f} worst {c.get('worst'):8.2f} n {c.get('n')} ends {ends}")
elif mode == 'cover':
    base = at(89)
    for a in ({'attack': {'attacker_slot': 3, 'player': 'b', 'target': {'slot': 1}}},
              {'attack': {'attacker_slot': 3, 'player': 'b', 'target': {'slot': 1}}}):
        base.apply(a)
    f = base.full(); f = json.loads(f) if isinstance(f, str) else f
    deck = sorted({c['name'] for c in f['players']['a']['deck']})
    print('distinct cards left in the owner deck:', len(deck), deck)
    done = {'Alchemic Flare', 'Tico, Mysterian Spellcrafter', "Witch's New Brew", 'Ecstatic Scholar', 'Obsidian Raven',
            'Humane Love', 'Lyria, Skydestined', 'Wills United', 'Obsessed Test Subject', 'Sweet Abomination'}
    todo = [n for n in deck if n not in done]
    print('not reached by draws (reseeds None, 1-15):', todo)
    rs = 15
    while todo and rs < 5000:
        rs += 1
        g = base.clone(); g.reseed(rs); g.apply({'end_turn': {'player': 'b'}})
        f = g.full(); f = json.loads(f) if isinstance(f, str) else f
        drawn = f['players']['a']['hand'][-1]['name']
        if drawn in todo:
            todo.remove(drawn)
            r, cnt, cap = owner_can_win(g, limit=3_000_000)
            print(f"reseed {rs}: owner draws {drawn!r} -> owner win this turn: {r} ({cnt} states{', CAP' if cap else ''})", flush=True)
    print('left uncovered:', todo)
else:
    base = at(89)
    for a in ({'attack': {'attacker_slot': 3, 'player': 'b', 'target': {'slot': 1}}},
              {'attack': {'attacker_slot': 3, 'player': 'b', 'target': {'slot': 1}}}):
        base.apply(a)
    wins = 0; tot = 0
    for rs in [None] + list(range(1, 16)):
        g = base.clone()
        if rs is not None:
            g.reseed(rs)
        g.apply({'end_turn': {'player': 'b'}})
        f = g.full(); f = json.loads(f) if isinstance(f, str) else f
        hand = [c['name'] for c in f['players']['a']['hand']]
        r, cnt, cap = owner_can_win(g)
        tot += 1; wins += int(r)
        print(f"reseed {rs}: owner hand {hand[-1]!r} (last card = the draw) -> owner win this turn: {r} ({cnt} states{', CAP' if cap else ''})")
    print(f'owner has a turn-9 win in {wins} of {tot} draws')
