"""Table 6 (as inferred; the checkpoint 2 runbook's own definition was not available): handed kills by the shape of the
opponent's winning line. A handed kill = a handed_lethal decision with verdict 'lethal' and hit. Shapes: any follower
attack in the line ('attack' actions are follower attacks: attacker_slot); attacks without any play; play(s) + attack;
no attack (plays / abilities only); with an evolve. Shares of handed kills, cluster bootstrap on seed; paired change.
Usage: python table6.py NEW_PREFIX OLD_PREFIX [NEW_LABEL OLD_LABEL]. Throwaway."""
import json, os, sys
import numpy as np

R = os.path.join(os.getcwd(), 'results', 'lethal')  # run from the repo root
NEW, OLD = sys.argv[1], sys.argv[2]
LN = sys.argv[3] if len(sys.argv) > 3 else NEW
LO = sys.argv[4] if len(sys.argv) > 4 else OLD
SHAPES = ('with a follower attack', 'attacks, no play', 'play + attack', 'no attack', 'with an evolve')


def shape_flags(line):
    k = [next(iter(a)) for a in line or []]
    att, play, evo = 'attack' in k, 'play' in k, 'evolve' in k
    return (att, att and not play, att and play, not att, evo)


def load(prefix):
    rows = []  # (seed index, flags..., deficit)
    for t in ('fa', 'fb'):
        d = json.load(open(fr'{R}\{prefix}-{t}.json', encoding='utf-8'))
        for r in d['records']:
            s = int(r['game_id'].split('-')[1])
            for x in r['decisions']:
                if x['kind'] == 'handed_lethal' and x['verdict'] == 'lethal' and x.get('hit'):
                    rows.append((s, *shape_flags(x.get('line')), x.get('line_len') or len(x.get('line') or [])))
    return rows


seeds = list(range(1, 513))
SI = {s: i for i, s in enumerate(seeds)}
rng = np.random.default_rng(20260922)
W = np.stack([np.bincount(rng.integers(0, 512, 512), minlength=512) for _ in range(2000)]).astype(float)
A = {}
for lab, pre in ((LN, NEW), (LO, OLD)):
    rows = load(pre)
    M = np.zeros((512, len(SHAPES) + 1))
    for row in rows:
        M[SI[row[0]], :len(SHAPES)] += row[1:1 + len(SHAPES)]
        M[SI[row[0]], -1] += 1
    A[lab] = (M, W @ M, rows)
print(f"handed kills: {LN} {int(A[LN][0][:, -1].sum())}, {LO} {int(A[LO][0][:, -1].sum())}")
print(f"  {'shape (share of handed kills)':32} {LN:>24} {LO:>24}   change (paired by seed)")
for j, name in enumerate(SHAPES):
    cells, bs = [], []
    for lab in (LN, LO):
        M, B, _ = A[lab]
        p = M[:, j].sum() / M[:, -1].sum()
        b = B[:, j] / B[:, -1]
        bs.append(b)
        lo, hi = np.percentile(b, [2.5, 97.5])
        cells.append(f"{int(M[:, j].sum()):4d} = {p:.3f} [{lo:.3f}, {hi:.3f}]")
    d = bs[0] - bs[1]
    p0 = A[LN][0][:, j].sum() / A[LN][0][:, -1].sum() - A[LO][0][:, j].sum() / A[LO][0][:, -1].sum()
    lo, hi = np.percentile(d, [2.5, 97.5])
    print(f"  {name:32} {cells[0]:>24} {cells[1]:>24}   {p0:+.3f} [{lo:+.3f}, {hi:+.3f}]")
for lab in (LN, LO):
    ll = np.array([r[-1] for r in A[lab][2]])
    print(f"  line length {lab}: mean {ll.mean():.2f}, median {np.median(ll):.0f}")
