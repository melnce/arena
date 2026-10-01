"""Checkpoint lethal audit: cp2's h0 (checkpoint-f{a,b}) vs 2026-09-22 (matched-f{a,b}); matched.py method, steps 1-5.
Usage: python checkpoint_report.py [NEW_PREFIX] [OLD_PREFIX]   (defaults checkpoint / matched). Throwaway."""
import json, math, os, sys, collections
import numpy as np

R = os.path.join(os.getcwd(), 'results', 'lethal')  # run from the repo root
NEW = sys.argv[1] if len(sys.argv) > 1 else 'checkpoint'
OLD = sys.argv[2] if len(sys.argv) > 2 else 'matched'
# sweep 9 per-deck win rates: the external "win" axis of the 2026-09-22 analysis; fixes the top/bottom six
S9 = {'portal-af': .725, 'portal-evo': .703, 'abyss-midrange': .692, 'sword-rally': .665, 'haven-evo': .603,
      'dragon-aggro': .603, 'abyss-aggro': .598, 'sword-loot': .582, 'forest-combo': .523, 'dragon-ramp': .501,
      'rune-spell': .395, 'portal-cutthroat': .394, 'rune-test-subject': .353, 'haven-amulet': .306,
      'rune-crystal': .219, 'haven-kukishiro': .138}
S9 = {'meta-' + k: v for k, v in S9.items()}
DECKS = sorted(S9, key=lambda d: -S9[d])
DI = {d: i for i, d in enumerate(DECKS)}
TOP = [DI[d] for d in DECKS if S9[d] >= .60]
BOT = [DI[d] for d in DECKS if S9[d] < .40]
assert len(TOP) == 6 and len(BOT) == 6
WV9 = np.array([S9[d] for d in DECKS])
FILES = {'fa': 'a', 'fb': 'b'}
short = lambda d: d.removeprefix('meta-')
bucket = lambda x: 0 if x <= -5 else 1 if x <= -1 else 2 if x == 0 else 3 if x <= 4 else 4


def corr(x, y):
    x, y = np.asarray(x, float), np.asarray(y, float)
    x, y = x - x.mean(), y - y.mean()
    return float((x * y).sum() / math.sqrt((x * x).sum() * (y * y).sum()))


def ci(b, f='+.3f'):
    lo, hi = np.nanpercentile(np.asarray(b, float), [2.5, 97.5])
    return f"[{lo:{f}}, {hi:{f}}]"


def fail(msg):
    print("\nSTOP:", msg)
    sys.exit(1)


def load(prefix):
    data = {t: json.load(open(fr'{R}\{prefix}-{t}.json', encoding='utf-8')) for t in FILES}
    for t, d in data.items():
        hdr = (d['bot_seat'], d['games'], d['policy_a'], d['policy_b'], d['pool'], d['budget'])
        if hdr[0:2] != ('both', 512) or hdr[2] != hdr[3] or hdr[4:] != ('meta', 50000):
            fail(f"{prefix}-{t}: unexpected header {hdr}")
    return data


new, old = load(NEW), load(OLD)
# gate: same seeds, same pairs
for t in FILES:
    a = {r['game_id']: r['bot_deck'] for r in new[t]['records']}
    b = {r['game_id']: r['bot_deck'] for r in old[t]['records']}
    if a != b:
        fail(f"{t}: game_id -> bot_deck differs between {NEW} and {OLD}")
seeds = sorted({int(r['game_id'].split('-')[1]) for t in FILES for r in new[t]['records']})
if len(seeds) != 512:
    fail("expected 512 seeds")
SI = {s: i for i, s in enumerate(seeds)}
print(f"gate: {NEW} and {OLD} share every game_id -> deck pair in both files (512 seeds): OK")
rng = np.random.default_rng(20260922)
REPS = 2000
W = np.stack([np.bincount(rng.integers(0, 512, 512), minlength=512) for _ in range(REPS)]).astype(float)


def flatten(data):
    ends, kturns, agames = [], {}, []
    for t, first in FILES.items():
        for r in data[t]['records']:
            si = SI[int(r['game_id'].split('-')[1])]
            da, db = r['bot_deck'].split('/')
            deck_of = {'a': da, 'b': db}
            for s in 'ab':
                other = 'b' if s == 'a' else 'a'
                agames.append(dict(t=t, si=si, gid=r['game_id'], seat=s, deck=DI[deck_of[s]], first=(s == first),
                                   lost=(r['winner'] == other), won=(r['winner'] == s)))
            for x in r['decisions']:
                s = x['auditee']
                if x['kind'] == 'handed_lethal':
                    if x['verdict'] in ('lethal', 'none'):
                        ends.append((si, DI[deck_of[s]], bucket(x['defense_deficit']), s == first, bool(x['hit']), x['turn']))
                elif x['verdict'] == 'lethal':
                    key = (t, r['game_id'], s, x['turn'])
                    k = kturns.setdefault(key, dict(si=si, deck=DI[deck_of[s]], first=(s == first), ply=x['ply'],
                                                    nodes=x['nodes'], converted=x['converted'], det=False))
                    if x['ply'] < k['ply']:
                        k['ply'], k['nodes'] = x['ply'], x['nodes']
                    if x['rng_dependent'] is False:
                        k['det'] = True
                    if x['converted'] != k['converted']:
                        fail(f"converted differs within a turn {key}")
    return ends, kturns, agames


def std_handed(Esum):
    cell = Esum.sum(axis=0)
    pooled = np.nan_to_num(cell[..., 0] / cell[..., 1])
    obs = Esum[..., 0].sum(axis=(1, 2)) / Esum[..., 1].sum(axis=(1, 2))
    share = Esum[..., 1] / Esum[..., 1].sum(axis=(1, 2), keepdims=True)
    return obs, (share * pooled).sum(axis=(1, 2))


def std2(Es2):
    cell = Es2.sum(axis=0)
    pooled = np.nan_to_num(cell[..., 0] / cell[..., 1])
    obs = Es2[..., 0].sum(axis=(1, 2, 3)) / Es2[..., 1].sum(axis=(1, 2, 3))
    share = Es2[..., 1] / Es2[..., 1].sum(axis=(1, 2, 3), keepdims=True)
    return obs, (share * pooled).sum(axis=(1, 2, 3))


def covsplit(e, o, wv):
    cv = lambda v: float(((v - v.mean()) * (wv - wv.mean())).sum())
    return cv(e) / cv(o)


def grp(idx, a):
    sub = np.take(a, idx, axis=-3)
    return sub[..., 0].sum(axis=(-1, -2)) / sub[..., 1].sum(axis=(-1, -2))


def analyse(data):
    ends, kturns, agames = flatten(data)
    out = dict(n_ends=len(ends), n_kt=len(kturns), n_ag=len(agames))
    # win rate per deck, auditee games, both seats
    G = np.zeros((512, 16, 3))
    for g in agames:
        G[g['si'], g['deck']] += (g['won'], g['lost'], 1)
    out['G'] = G
    out['win'] = G.sum(0)[:, 0] / G.sum(0)[:, 2]
    out['win_b'] = np.einsum('rs,sd->rd', W, G[..., 0]) / np.einsum('rs,sd->rd', W, G[..., 2])
    # handed_lethal cells
    E = np.zeros((512, 16, 5, 2, 2))
    E2 = np.zeros((512, 16, 5, 2, 2, 2))
    for si, dk, b, f, h, turn in ends:
        E[si, dk, b, int(f)] += (h, 1)
        E2[si, dk, b, int(f), int(turn > 3)] += (h, 1)
    out['E'], out['E2'] = E, E2
    out['bootE'] = np.einsum('rs,sdbfk->rdbfk', W, E)
    out['bootE2'] = np.einsum('rs,sdbftk->rdbftk', W, E2)
    out['kturns'] = kturns
    out['agames'] = agames
    return out


A = {'cp2': analyse(new), 'cp1': analyse(old)}
LAB = ('cp2', 'cp1')
print(f"end-turns with a verdict: cp2 {A['cp2']['n_ends']}, cp1 {A['cp1']['n_ends']};  "
      f"kill turns: {A['cp2']['n_kt']} / {A['cp1']['n_kt']};  auditee-games {A['cp2']['n_ag']} / {A['cp1']['n_ag']}")

# ---------------- 1. balance ----------------
print("\n=== 1. balance: auditee_first share of end-turns per deck ===")
print(f"  {'deck':20} {'cp2':>16} {'cp1':>16}")
for dk, d in enumerate(DECKS):
    cells = []
    for l in LAB:
        v = A[l]['E'][..., 1].sum(0)[dk]
        cells.append(f"{v[:, 1].sum() / v.sum():.3f} (n={v.sum():4.0f})")
    print(f"  {short(d):20} {cells[0]:>16} {cells[1]:>16}")

# ---------------- 2. win rate per deck ----------------
print("\n=== 2. win rate per deck (auditee games, both seats; 128 per deck), sorted by cp2 ===")
wt, wo = A['cp2']['win'], A['cp1']['win']
dwb = A['cp2']['win_b'] - A['cp1']['win_b']
print(f"  {'deck':20} {'cp2':>6} {'cp1':>6} {'change':>7}  {'95% CI (paired)':>17}  sweep9")
for dk in np.argsort(-wt):
    print(f"  {short(DECKS[dk]):20} {wt[dk]:6.3f} {wo[dk]:6.3f} {wt[dk] - wo[dk]:+7.3f}  {ci(dwb[:, dk]):>17}  {WV9[dk]:.3f}")
sp_t, sp_o = wt.max() - wt.min(), wo.max() - wo.min()
spb = A['cp2']['win_b'].max(1) - A['cp2']['win_b'].min(1) - (A['cp1']['win_b'].max(1) - A['cp1']['win_b'].min(1))
print(f"  spread max-min: cp2 {sp_t:.3f} ({wt.min():.3f}-{wt.max():.3f}), cp1 {sp_o:.3f} ({wo.min():.3f}-{wo.max():.3f}); "
      f"change {sp_t - sp_o:+.3f} {ci(spb)}")
print(f"  SD across decks: cp2 {wt.std():.3f}, cp1 {wo.std():.3f};  corr(cp2, cp1) {corr(wt, wo):+.3f};  corr(cp2, sweep9) {corr(wt, WV9):+.3f}")

# ---------------- 3. handed_lethal, standardised ----------------
print("\n=== 3. handed_lethal, standardised on deficit bucket x first player ===")
for axis_name in ('sweep9', 'own'):
    print(f"  -- win axis: {'sweep 9 per-deck win rate (as on 2026-09-22)' if axis_name == 'sweep9' else 'each audit own per-deck win rate (step 2; shares games with handed_lethal)'}")
    for l in LAB:
        a = A[l]
        wv = WV9 if axis_name == 'sweep9' else a['win']
        Es = a['E'].sum(0)
        obs, exp = std_handed(Es)
        bo = [std_handed(a['bootE'][i]) for i in range(REPS)]
        wvb = (lambda i: WV9) if axis_name == 'sweep9' else (lambda i: a['win_b'][i])
        ce = [corr(e, wvb(i)) for i, (o, e) in enumerate(bo)]
        cr = [corr(o - e, wvb(i)) for i, (o, e) in enumerate(bo)]
        o2, e2 = std2(a['E2'].sum(0))
        bb2 = [std2(a['bootE2'][i]) for i in range(REPS)]
        rate = Es[..., 0].sum() / Es[..., 1].sum()
        print(f"    {l:6} rate {rate:.4f}  corr(expected,win) {corr(exp, wv):+.3f} {ci(ce)}  corr(residual,win) {corr(obs - exp, wv):+.3f} {ci(cr)}  "
              f"split expected {covsplit(exp, obs, wv):.0%} / residual {1 - covsplit(exp, obs, wv):.0%}")
        print(f"    {'':6} + early/late stratum:  corr(expected,win) {corr(e2, wv):+.3f} {ci([corr(e, wvb(i)) for i, (o, e) in enumerate(bb2)])}  "
              f"corr(residual,win) {corr(o2 - e2, wv):+.3f} {ci([corr(o - e, wvb(i)) for i, (o, e) in enumerate(bb2)])}  "
              f"split expected {covsplit(e2, o2, wv):.0%} / residual {1 - covsplit(e2, o2, wv):.0%}")
Es_t, Es_o = A['cp2']['E'].sum(0), A['cp1']['E'].sum(0)
print("  pooled handed rate by deficit bucket (cp2 | cp1):")
for b, name in enumerate(("<= -5", "-4..-1", "0", "1..4", ">= 5")):
    t_, o_ = Es_t[:, b].sum(0), Es_o[:, b].sum(0)
    print(f"    {name:7} {t_[:, 0].sum() / t_[:, 1].sum():.3f} (n={t_[:, 1].sum():.0f})  |  {o_[:, 0].sum() / o_[:, 1].sum():.3f} (n={o_[:, 1].sum():.0f})")

# ---------------- 4. missed_lethal per kill turn ----------------
print("\n=== 4. missed_lethal per kill turn (top/bottom six fixed by sweep 9, as on 2026-09-22) ===")
allnodes = np.array([k['nodes'] for l in LAB for k in A[l]['kturns'].values()], float)
q1, q2 = np.quantile(allnodes, [1 / 3, 2 / 3])
for l in LAB:
    nn = np.array([k['nodes'] for k in A[l]['kturns'].values()], float)
    o1, o2_ = np.quantile(nn, [1 / 3, 2 / 3])
    print(f"  {l}: own tercile cuts {o1:.0f} / {o2_:.0f}")
print(f"  common tercile cuts (both audits pooled): {q1:.0f} / {q2:.0f}")
for l in LAB:
    KT = np.zeros((512, 16, 3, 2))
    for k in A[l]['kturns'].values():
        tt = 0 if k['nodes'] <= q1 else (1 if k['nodes'] <= q2 else 2)
        KT[k['si'], k['deck'], tt] += (not k['converted'], 1)
    A[l]['KT'] = KT
    A[l]['kboot'] = np.einsum('rs,sdtk->rdtk', W, KT)
print(f"  {'':12} {'cp2':>34} {'cp1':>34}")
rows = []
for name, fn in (('all', lambda a: a[..., 0].sum(axis=(-1, -2)) / a[..., 1].sum(axis=(-1, -2))),
                 ('top six', lambda a: grp(TOP, a)), ('bottom six', lambda a: grp(BOT, a)),
                 ('bottom-top', lambda a: grp(BOT, a) - grp(TOP, a))):
    cells = []
    for l in LAB:
        KTs, kb = A[l]['KT'].sum(0), A[l]['kboot']
        cells.append(f"{float(fn(KTs)):+.4f} {ci(fn(kb), '+.4f')}" if name == 'bottom-top' else f"{float(fn(KTs)):.4f} {ci(fn(kb), '.4f')}")
    diff = fn(A['cp2']['kboot']) - fn(A['cp1']['kboot'])
    print(f"  {name:12} {cells[0]:>34} {cells[1]:>34}   change {float(fn(A['cp2']['KT'].sum(0)) - fn(A['cp1']['KT'].sum(0))):+.4f} {ci(diff, '+.4f')}")
for tt in range(3):
    cells = []
    for l in LAB:
        KTs, kb = A[l]['KT'].sum(0), A[l]['kboot']
        m, n = KTs[:, tt, 0].sum(), KTs[:, tt, 1].sum()
        cells.append(f"{m:.0f}/{n:.0f} = {m / n:.4f} {ci(kb[:, :, tt, 0].sum(1) / kb[:, :, tt, 1].sum(1), '.4f')}")
    print(f"  tercile {tt + 1}   {cells[0]:>34} {cells[1]:>34}")
print("  counts: top six " + " | ".join(f"{l} {A[l]['KT'].sum(0)[TOP, :, 0].sum():.0f}/{A[l]['KT'].sum(0)[TOP, :, 1].sum():.0f}" for l in LAB)
      + ";  bottom six " + " | ".join(f"{l} {A[l]['KT'].sum(0)[BOT, :, 0].sum():.0f}/{A[l]['KT'].sum(0)[BOT, :, 1].sum():.0f}" for l in LAB))

# ---------------- 5. thrown games ----------------
print("\n=== 5. thrown games: auditee-games with a deterministic missed kill turn that the auditee lost ===")
for l in LAB:
    thrown = {(k2[0], k2[1], k2[2]) for k2, k in A[l]['kturns'].items() if (not k['converted']) and k['det']}
    TG = np.zeros((512, 16, 2))
    for g in A[l]['agames']:
        TG[g['si'], g['deck']] += (((g['t'], g['gid'], g['seat']) in thrown) and g['lost'], 1)
    A[l]['TG'] = TG
    A[l]['tgb'] = np.einsum('rs,sdk->rdk', W, TG)
pooled = lambda a: a[..., 0].sum(-1) / a[..., 1].sum(-1)
for l in LAB:
    TGs = A[l]['TG'].sum(0)
    print(f"  {l:6} pooled {TGs[:, 0].sum():.0f}/{TGs[:, 1].sum():.0f} = {TGs[:, 0].sum() / TGs[:, 1].sum():.2%}  "
          f"{ci(pooled(A[l]['tgb']) * 100, '.2f')} %")
d = pooled(A['cp2']['tgb']) - pooled(A['cp1']['tgb'])
tt_, to_ = A['cp2']['TG'].sum(0), A['cp1']['TG'].sum(0)
print(f"  change (paired by seed) {(tt_[:, 0].sum() - to_[:, 0].sum()) / 2048:+.2%}  {ci(d * 100, '+.2f')} points")
print(f"  {'deck':20} {'cp2':>6} {'cp1':>6}   (of 128 auditee-games each)")
for dk in np.argsort(-(tt_[:, 0])):
    print(f"  {short(DECKS[dk]):20} {tt_[dk, 0]:6.0f} {to_[dk, 0]:6.0f}")
