"""Kill turns by type, for several lethal audits on the same deals (checkpoint 1, checkpoint 2, tkill, ...).
A kill turn = an (auditee, turn) in which some decision got a 'lethal' verdict. It is deterministic if any of those
verdicts has rng_dependent false (a kill that needs no luck existed at some point in the turn), chance-dependent if
all of them are rng_dependent. Missed = the turn was not converted. Also decision-level missed_lethal hits (verdict
lethal, hit, not converted) by rng_dependent. Intervals: cluster bootstrap on seed (2 000 resamples, the same
resamples for every audit, so differences are paired).
Usage: python killturns.py LABEL=PREFIX [LABEL=PREFIX ...]   (e.g. cp1=checkpoint cp2=checkpoint2 tkill=tkill)
Run from the repo root."""
import json, os, sys
import numpy as np

R = os.path.join(os.getcwd(), 'results', 'lethal')
AUD = [a.split('=', 1) for a in sys.argv[1:]]
rng = np.random.default_rng(20260922)
W = np.stack([np.bincount(rng.integers(0, 512, 512), minlength=512) for _ in range(2000)]).astype(float)
COLS = ('kt', 'kt_det', 'kt_det_missed', 'kt_chance', 'kt_chance_missed', 'hit_det', 'hit_rng', 'lethal_dec')


def load(prefix):
    M = np.zeros((512, len(COLS)))
    for t in ('fa', 'fb'):
        d = json.load(open(os.path.join(R, f'{prefix}-{t}.json'), encoding='utf-8'))
        for r in d['records']:
            si = int(r['game_id'].split('-')[1]) - 1
            turns = {}
            for x in r['decisions']:
                if x['kind'] != 'missed_lethal' or x['verdict'] != 'lethal':
                    continue
                M[si, 7] += 1
                if x.get('hit') and not x.get('converted'):
                    M[si, 5 if x.get('rng_dependent') is False else 6] += 1
                k = turns.setdefault((x['auditee'], x['turn']), [False, bool(x.get('converted'))])
                k[0] |= x.get('rng_dependent') is False
            for det, conv in turns.values():
                M[si, 0] += 1
                M[si, 1 if det else 3] += 1
                M[si, 2 if det else 4] += (not conv)
    return M


A = {lab: load(pre) for lab, pre in AUD}
B = {lab: W @ M for lab, M in A.items()}
labs = [lab for lab, _ in AUD]
ci = lambda b, f: '[' + ', '.join(format(v, f) for v in np.percentile(b, [2.5, 97.5])) + ']'
print('kill turns (an auditee turn with at least one lethal verdict), same 512 deals x 2 files each:')
print(f"  {'':34}" + ''.join(f"{lab:>26}" for lab in labs))
rows = (('kill turns', lambda M: M[..., 0], None),
        ('  deterministic (a luck-free kill)', lambda M: M[..., 1], None),
        ('  chance-dependent only', lambda M: M[..., 3], None),
        ('  chance-dependent share', lambda M: M[..., 3] / M[..., 0], '.3f'),
        ('missed: all kill turns', lambda M: (M[..., 2] + M[..., 4]) / M[..., 0], '.3f'),
        ('missed: deterministic turns', lambda M: M[..., 2] / M[..., 1], '.3f'),
        ('missed: chance-dependent turns', lambda M: M[..., 4] / M[..., 3], '.3f'))
for name, fn, fmt in rows:
    cells = []
    for lab in labs:
        v = fn(A[lab].sum(0))
        if fmt is None:
            cells.append(f"{v:.0f}")
        else:
            cells.append(f"{v:{fmt}} {ci(fn(B[lab]), fmt)}")
    print(f"  {name:34}" + ''.join(f"{c:>26}" for c in cells))
print('  counts missed: ' + '; '.join(f"{lab} det {A[lab][:, 2].sum():.0f}/{A[lab][:, 1].sum():.0f}, chance {A[lab][:, 4].sum():.0f}/{A[lab][:, 3].sum():.0f}" for lab in labs))
print('decision-level missed_lethal hits (the action taken gave up a kill): ' + '; '.join(
    f"{lab} deterministic {A[lab][:, 5].sum():.0f}, rng-dependent {A[lab][:, 6].sum():.0f} (of {A[lab][:, 7].sum():.0f} lethal-verdict decisions)" for lab in labs))
print('paired changes (later minus earlier, same resamples):')
for i in range(1, len(labs)):
    a, o = labs[i], labs[i - 1]
    for name, fn in (('chance-dependent kill turns', lambda M: M[..., 3]), ('deterministic kill turns', lambda M: M[..., 1]),
                     ('miss rate, deterministic turns', lambda M: M[..., 2] / M[..., 1]),
                     ('miss rate, chance-dependent turns', lambda M: M[..., 4] / M[..., 3]),
                     ('miss rate, all kill turns', lambda M: (M[..., 2] + M[..., 4]) / M[..., 0])):
        v = fn(A[a].sum(0)) - fn(A[o].sum(0))
        f = '+.0f' if 'turns' in name and 'rate' not in name else '+.3f'
        print(f"  {a} - {o}: {name:34} {v:{f}} {ci(fn(B[a]) - fn(B[o]), f)}")
    for j, name in ((5, 'deterministic hits'), (6, 'rng-dependent hits')):
        rel = B[a][:, j] / B[o][:, j] - 1
        print(f"  {a} - {o}: {name:34} {A[a][:, j].sum() - A[o][:, j].sum():+.0f}  relative {A[a][:, j].sum() / A[o][:, j].sum() - 1:+.0%} {ci(rel * 100, '+.0f')} %")
