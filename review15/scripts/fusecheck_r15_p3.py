"""fusecheck_r15.py parts 3 and 4, in parallel (7 processes), for the games named on the command line: every bot End
Turn the lethal audit called 'none' or 'unknown' (handed_lethal), re-checked with the fuse-aware search (dfs2.can_win,
cap 2 000 000 nodes, as in fusecheck_r15.py); then part 4 (Fuse cards in the bot's deck).
Used to finish game 1 after fusecheck_r15.py's single-process part 3 had done games 4, 2 and 3.
usage (repo root): python fusecheck_r15_p3.py REVIEW15_DIR GAME_ID_PREFIX..."""
import json, sys, glob, os
from multiprocessing import Pool
import arena
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dfs2 import can_win


def at(gid, n):
    db = arena.load_cards('cards')
    d = json.load(open(f'results/games/{gid}.json', encoding='utf-8'))
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def work(job):
    gid, ply, verdict = job
    w, n, wl = can_win(at(gid, ply + 1))
    return gid, ply, verdict, w, n, [json.dumps(x)[:90] for x in wl]


if __name__ == '__main__':
    R = sys.argv[1]
    want = sys.argv[2:]
    au = json.load(open(os.path.join(R, 'lethal_audit.json'), encoding='utf-8'))
    jobs = []
    for rec in au['records']:
        gid = [f for f in os.listdir(os.path.join(R, 'games')) if f.startswith(str(rec['seed']))][0][:-5]
        if not any(gid.startswith(p) for p in want):
            continue
        for dec in rec.get('decisions', []):
            if dec.get('kind') == 'handed_lethal' and dec.get('verdict') != 'lethal':
                jobs.append((gid, dec['ply'], dec.get('verdict')))
    print(f'== 3 (continued, {len(jobs)} End Turns, parallel)', flush=True)
    with Pool(7) as p:
        res = p.map(work, jobs)
    for gid, ply, verdict, w, n, wl in res:
        print(f"  {gid} ply {ply} (engine {verdict}): owner can win {w} ({n} nodes){' via ' + str(wl) if w else ''}", flush=True)
    print(f'  re-checked {len(res)}; fuse-aware kills found {sum(bool(r[3]) for r in res)}; undecided {sum(r[3] is None for r in res)}', flush=True)

    print('== 4. Fuse cards in the bot deck', flush=True)
    d = json.load(open('results/games/6065548863054576576-b43e6214.json', encoding='utf-8'))
    fz = []
    for cid in d['deckB']:
        for q in glob.glob(f'cards/*/{cid}.json'):
            c = json.load(open(q, encoding='utf-8'))
            if 'Fuse' in (c.get('text') or ''):
                fz.append(c.get('name'))
    print(f'  Sword Rally cards with Fuse: {fz or "none"}', flush=True)
