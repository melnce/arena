"""missed_lethal hits (decisions where the auditee had a kill - verdict 'lethal' - and the action taken gave it up:
hit true, converted false) split by rng_dependent, for two audits on the same deals; relative and paired change
(cluster bootstrap on seed). Also per kill turn: kill turns with a deterministic kill that were not converted.
Usage: python missed_split.py NEW_PREFIX OLD_PREFIX [NEW_LABEL OLD_LABEL]. Run from the repo root."""
import json, os, sys
import numpy as np

R = os.path.join(os.getcwd(), 'results', 'lethal')
NEW, OLD = sys.argv[1], sys.argv[2]
LN = sys.argv[3] if len(sys.argv) > 3 else NEW
LO = sys.argv[4] if len(sys.argv) > 4 else OLD
rng = np.random.default_rng(20260922)
W = np.stack([np.bincount(rng.integers(0, 512, 512), minlength=512) for _ in range(2000)]).astype(float)


def load(prefix):
    # per seed: [hits det, hits rng, decisions with a lethal verdict, det kill turns missed, det kill turns]
    M = np.zeros((512, 5))
    for t in ('fa', 'fb'):
        d = json.load(open(os.path.join(R, f'{prefix}-{t}.json'), encoding='utf-8'))
        for r in d['records']:
            si = int(r['game_id'].split('-')[1]) - 1
            turns = {}
            for x in r['decisions']:
                if x['kind'] != 'missed_lethal' or x['verdict'] != 'lethal':
                    continue
                M[si, 2] += 1
                if x.get('hit') and not x.get('converted'):
                    M[si, 0 if x.get('rng_dependent') is False else 1] += 1
                k = turns.setdefault((x['auditee'], x['turn']), [False, x.get('converted')])
                if x.get('rng_dependent') is False:
                    k[0] = True
            for det, conv in turns.values():
                if det:
                    M[si, 4] += 1
                    M[si, 3] += (not conv)
    return M


A = {LN: load(NEW), LO: load(OLD)}
B = {k: W @ v for k, v in A.items()}
print(f"missed_lethal hits (decision level; the auditee had a kill and the action gave it up), {LN} vs {LO}, same deals:")
for j, name in ((0, 'deterministic (rng_dependent false)'), (1, 'rng-dependent (true)')):
    a, o = A[LN][:, j].sum(), A[LO][:, j].sum()
    rel = B[LN][:, j] / B[LO][:, j] - 1
    lo, hi = np.percentile(rel, [2.5, 97.5])
    dlo, dhi = np.percentile(B[LN][:, j] - B[LO][:, j], [2.5, 97.5])
    print(f"  {name:38} {LN} {a:5.0f}   {LO} {o:5.0f}   change {a - o:+5.0f} [{dlo:+.0f}, {dhi:+.0f}]   relative {a / o - 1:+.0%} [{lo:+.0%}, {hi:+.0%}]")
print(f"  decisions with a 'lethal' verdict: {LN} {A[LN][:, 2].sum():.0f}, {LO} {A[LO][:, 2].sum():.0f}")
a, o = A[LN][:, 3].sum(), A[LO][:, 3].sum()
na, no = A[LN][:, 4].sum(), A[LO][:, 4].sum()
rel = B[LN][:, 3] / B[LO][:, 3] - 1
lo, hi = np.percentile(rel, [2.5, 97.5])
print(f"kill turns with a deterministic kill, not converted: {LN} {a:.0f}/{na:.0f} = {a / na:.3f}   {LO} {o:.0f}/{no:.0f} = {o / no:.3f}   relative {a / o - 1:+.0%} [{lo:+.0%}, {hi:+.0%}]")
