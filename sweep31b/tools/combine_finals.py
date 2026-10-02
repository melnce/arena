"""Combine one candidate's finals from several sweeps, per arm (the confirm-run rule):
    python combine_finals.py results/sweep25:c01 results/sweep25b:c01 [...] [--strip KEY=VALUE ...]
Main arms are summed (candidate = side A), reverse arms are summed (candidate = side B); decisive games only (draws
excluded, as sweep.py does). Prints each part and the combined main / reverse / pooled with Wilson intervals and Elo,
and `better` = both combined arm lows > 0.50. The parts must share the same candidate spec, baseline and deck list;
`--strip net=engine/models/h0-linear-v2.json` drops that key before comparing (a part that pinned the leaf the other
part had as its default)."""
import json, math, os, sys
import numpy as np

REPO = os.getcwd()  # run from the repo root (the folders below are relative to it)


def wilson(k, n, z=1.96):
    p = k / n; dd = 1 + z * z / n; c = (p + z * z / (2 * n)) / dd
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / dd
    return c - h, c + h


elo = lambda p: 400 * math.log10(p / (1 - p))


def load(path):
    d = json.load(open(path, encoding='utf-8'))
    dk = d['decks']
    a = np.array([[d['matrix'][r][c]['a_wins'] for c in dk] for r in dk], float)
    b = np.array([[d['matrix'][r][c]['b_wins'] for c in dk] for r in dk], float)
    return dk, a, b, d['policy_a'], d['policy_b']


def line(lab, k, n):
    lo, hi = wilson(k, n)
    return f"  {lab:8s} {k / n:.4f} ({int(k)}/{int(n)}) [{lo:.4f}, {hi:.4f}]  Elo {elo(k / n):+.1f} [{elo(lo):+.1f}, {elo(hi):+.1f}]", lo


argv = sys.argv[1:]
strip = [argv[i + 1] for i, a in enumerate(argv) if a == '--strip']
parts = [a for i, a in enumerate(argv) if a != '--strip' and (i == 0 or argv[i - 1] != '--strip')]
norm = lambda s: ','.join(x for x in s.replace(':', ',', 1).split(',') if x not in strip).replace(',', ':', 1) if ':' in s else s
tot = dict(km=0.0, nm=0.0, kr=0.0, nr=0.0)
ref = None
for arg in parts:
    folder, c = arg.rsplit(':', 1)
    folder = os.path.join(REPO, folder)
    dk, am, bm, pa, pb = load(os.path.join(folder, f'{c}-final.json'))
    dk2, ar, br, rpa, rpb = load(os.path.join(folder, f'{c}-reverse.json'))
    assert dk == dk2 and rpa == pb and rpb == pa, (arg, pa, pb, rpa, rpb)
    key = (norm(pa), norm(pb), tuple(dk))
    assert ref is None or key == ref, f'{arg}: different candidate, baseline or decks: {key[:2]} vs {ref[:2]}'
    ref = key
    km, nm, kr, nr = am.sum(), (am + bm).sum(), br.sum(), (ar + br).sum()
    print(f"{arg}: candidate {pa} vs {pb}")
    for lab, k, n in (('main', km, nm), ('reverse', kr, nr), ('pooled', km + kr, nm + nr)):
        print(line(lab, k, n)[0])
    for k_, v in (('km', km), ('nm', nm), ('kr', kr), ('nr', nr)):
        tot[k_] += v
print(f"\nCOMBINED ({' + '.join(parts)}){' with ' + ', '.join(strip) + ' stripped' if strip else ''}:")
lows = {}
for lab, k, n in (('main', tot['km'], tot['nm']), ('reverse', tot['kr'], tot['nr']), ('pooled', tot['km'] + tot['kr'], tot['nm'] + tot['nr'])):
    s, lows[lab] = line(lab, k, n)
    print(s)
print(f"  better (combined main low > 0.50 and reverse low > 0.50): {lows['main'] > 0.5 and lows['reverse'] > 0.5}")
