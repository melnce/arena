"""review12 game 6 re-check with the fuse-aware search (dfs2.py): with Sephie killed (both rush Test Subjects attack her)
the bot ends its turn 8; can the owner win on turn 9? The same draws as s12_last_turn.py (actual, reseeds 1-15) and its
'cover' reseeds (18-20): 13 distinct cards, each searched once, under the first reseed that draws it (as review12 did).
The earlier search keyed its transposition set on the state hash alone and so never explored a fuse with partners; the
owner held Sephie and Ecstatic Scholar (both Fuse hosts) and has Sephie's crest.
dfs2.can_win: fuse-aware, one partner per fuse (see dfs2.py), engine-like move order, cap 5 000 000 nodes per draw.
The draws are searched in parallel (7 processes).
usage (repo root): python fusecheck.py (dfs2.py beside it)"""
import json, sys, os, time
from multiprocessing import Pool
import arena
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dfs2 import can_win, A, J

G = 'results/games/4280399497766595080-104cf827.json'
RESEEDS = [None] + list(range(1, 16)) + [18, 19, 20]
LIMIT = 5_000_000


def base_state(db):
    d = json.load(open(G, encoding='utf-8'))
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:89]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def after(db, rs, kill_sephie=True):
    g = base_state(db)
    if kill_sephie:
        for _ in range(2):
            g.apply({'attack': {'attacker_slot': 3, 'player': 'b', 'target': {'slot': 1}}})
    if rs is not None:
        g.reseed(rs)
    g.apply({'end_turn': {'player': 'b'}})
    return g


def drawn(g):
    return J(g.full())['players']['a']['hand'][-1]['name']


def work(rs):
    db = arena.load_cards('cards')
    g = after(db, rs)
    t = time.time()
    w, n, ln = can_win(g, limit=LIMIT)
    return rs, drawn(g), w, n, time.time() - t, [json.dumps(x) for x in ln]


if __name__ == '__main__':
    db = arena.load_cards('cards')
    w, n, _ = can_win(after(db, None, kill_sephie=False))
    print(f'control (Sephie alive, actual draw): owner can win {w} ({n} nodes)', flush=True)
    first = {}
    for rs in RESEEDS:
        first.setdefault(drawn(after(db, rs)), rs)
    print(f'{len(first)} distinct draws: ' + ', '.join(f'{c} (reseed {rs})' for c, rs in first.items()), flush=True)
    wins = unk = 0
    with Pool(7) as p:
        for rs, card, w, n, sec, ln in p.imap_unordered(work, list(first.values())):
            wins += bool(w); unk += (w is None)
            print(f"reseed {rs}: draws {card!r} -> owner win: {w} ({n} nodes, {sec:.0f} s{', CAP: undecided' if w is None else ''})"
                  f"{' via ' + str(ln) if w else ''}", flush=True)
    print(f'owner has a turn-9 win in {wins} of {len(first)} distinct draws ({unk} undecided)', flush=True)
