"""handed_lethal at equal deficit: NEW's rate standardised to OLD's mix of end-turn situations, paired change.
End-turn = a handed_lethal decision with a verdict ('lethal' or 'none'); hit = the opponent then had a kill.
Cells: (a) deficit bucket (<=-5, -4..-1, 0, 1..4, >=5; as table 3) x auditee first; (b) exact defense_deficit clipped
to [-10, +10] x auditee first. Standardised rate = sum over cells of OLD's share x NEW's cell rate (cells NEW lacks
fall back to OLD's rate). Paired change and relative change by cluster bootstrap on seed (the same 2 000 resamples
for both audits). Usage: python handed_eq.py NEW_PREFIX OLD_PREFIX [NEW_LABEL OLD_LABEL]. Run from the repo root."""
import json, os, sys
import numpy as np

R = os.path.join(os.getcwd(), 'results', 'lethal')
NEW, OLD = sys.argv[1], sys.argv[2]
LN = sys.argv[3] if len(sys.argv) > 3 else NEW
LO = sys.argv[4] if len(sys.argv) > 4 else OLD
rng = np.random.default_rng(20260922)
W = np.stack([np.bincount(rng.integers(0, 512, 512), minlength=512) for _ in range(2000)]).astype(float)
bucket = lambda x: 0 if x <= -5 else 1 if x <= -1 else 2 if x == 0 else 3 if x <= 4 else 4
fine = lambda x: max(-10, min(10, x)) + 10


def load(prefix, cell, ncell):
    E = np.zeros((512, ncell, 2, 2))  # seed, deficit cell, auditee first, (hits, n)
    for t, first in (('fa', 'a'), ('fb', 'b')):
        d = json.load(open(os.path.join(R, f'{prefix}-{t}.json'), encoding='utf-8'))
        for r in d['records']:
            si = int(r['game_id'].split('-')[1]) - 1
            for x in r['decisions']:
                if x['kind'] == 'handed_lethal' and x['verdict'] in ('lethal', 'none'):
                    E[si, cell(x['defense_deficit']), int(x['auditee'] == first)] += (bool(x['hit']), 1)
    return E


def std(new, old):
    """new, old: (..., cells..., 2) summed over seeds. Returns (raw new, raw old, new standardised to old's mix)."""
    ax = tuple(range(new.ndim - 3, new.ndim - 1))
    w = old[..., 1] / old[..., 1].sum(axis=ax, keepdims=True)
    r_old = np.where(old[..., 1] > 0, old[..., 0] / np.maximum(old[..., 1], 1), 0)
    r_new = np.where(new[..., 1] > 0, new[..., 0] / np.maximum(new[..., 1], 1), r_old)
    raw = lambda a: a[..., 0].sum(axis=ax) / a[..., 1].sum(axis=ax)
    return raw(new), raw(old), (w * r_new).sum(axis=ax)


for name, cell, ncell in (('deficit bucket x first (as table 3)', bucket, 5), ('exact deficit [-10, +10] x first', fine, 21)):
    En, Eo = load(NEW, cell, ncell), load(OLD, cell, ncell)
    rn, ro, sn = std(En.sum(0), Eo.sum(0))
    Bn, Bo = np.einsum('rs,sdfk->rdfk', W, En), np.einsum('rs,sdfk->rdfk', W, Eo)
    brn, bro, bsn = std(Bn, Bo)
    ci = lambda b, f: '[' + ', '.join(format(v, f) for v in np.percentile(b, [2.5, 97.5])) + ']'
    print(f"handed_lethal, cells: {name}")
    print(f"  end-turns with a verdict: {LN} {En[..., 1].sum():.0f}, {LO} {Eo[..., 1].sum():.0f}; hits {LN} {En[..., 0].sum():.0f}, {LO} {Eo[..., 0].sum():.0f}")
    print(f"  raw rate        {LN} {rn:.4f} {ci(brn, '.4f')}   {LO} {ro:.4f} {ci(bro, '.4f')}")
    print(f"  {LN} at {LO}'s deficit mix: {sn:.4f} {ci(bsn, '.4f')}")
    print(f"  paired change at equal deficit: {sn - ro:+.4f} {ci(bsn - bro, '+.4f')}   relative {sn / ro - 1:+.1%} {ci((bsn / bro - 1) * 100, '+.1f')} %")
    print(f"  paired change, raw:             {rn - ro:+.4f} {ci(brn - bro, '+.4f')}")
