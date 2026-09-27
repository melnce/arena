"""Counterfactual: replay GAME_ID up to ACTION_INDEX but SKIP the listed action indices,
then ask the bot at that position (like replay_at.py). Read-only, in memory.
usage: python cf_skip.py GAME_ID ACTION_INDEX SKIP(comma list) [SPEC] [SEED]"""
import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import botview as bv
import arena

gid, idx = sys.argv[1], int(sys.argv[2])
skip = {int(x) for x in sys.argv[3].split(',') if x}
spec = sys.argv[4] if len(sys.argv) > 4 else 'h0:nodes=16000'
seed = int(sys.argv[5]) if len(sys.argv) > 5 else 1
db = arena.load_cards(bv.REPO + r'\cards')
rec = json.load(open(bv.REPO + rf'\results\games\{gid}.json', encoding='utf-8'))
g = arena.Game(db, rec['seed'], rec['deckA'], rec['deckB'], rec.get('first') or 'coin')
for i, a in enumerate(rec['actions'][:idx]):
    if i in skip:
        print(f"skipped [{i}]: {bv.short_action(g, bv.body(a)) if 'reseed' not in a else a}")
        continue
    if 'reseed' in a:
        g.reseed(int(a['reseed']))
    else:
        g.apply(bv.body(a))
print(f"position before [{idx}]: turn {g.turn}, {'BOT' if g.active == 'b' else 'HUMAN'} to act, phase {g.phase}")
print('\n'.join(bv.state_lines(g)))
e = g.bot_action_explain(spec, seed)
if isinstance(e, str):
    e = json.loads(e)
print(f"\n{spec} seed {seed}: chooses {bv.short_action(g, e.get('chosen'))}; path {e.get('path')}, nodes {e.get('nodes')}, k {e.get('k')}")
for c in sorted(e.get('candidates', []), key=lambda c: -(c['root_agg'] if c.get('root_agg') is not None else -1e9))[:8]:
    ws = [w for w in c.get('worlds', []) if not w.get('skipped')]
    ends = sorted({w.get('end') for w in ws if w.get('end')})
    per_world = ' '.join(f"{w['clamped']:+.0f}" for w in ws if w.get('clamped') is not None)
    print(f"  {c['root_agg'] if c.get('root_agg') is not None else float('nan'):+7.1f} (worst {c['worst']:+.1f}; worlds {per_world}; ends {','.join(ends)}): {bv.short_action(g, c['action'])}")
    best = max((w for w in ws if w.get('pv')), key=lambda w: w.get('clamped', -1e9), default=None)
    if best:
        pv = []
        for x in best['pv'][:10]:
            kk = next(iter(x)); vv = x[kk]
            pv.append(f"{'B' if vv.get('player') == 'b' else 'H'}:{kk}" + (f" {bv.nm(vv['card'])}" if kk == 'play' else '')
                      + (f" s{vv.get('attacker_slot')}->{vv.get('target')}" if kk == 'attack' else ''))
        print(f"        line: {' | '.join(pv)}  [end {best.get('end')}]")
