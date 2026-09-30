"""Re-ask the bot at one position of a recorded game.

usage: python replay_at.py GAME_ID ACTION_INDEX [SPEC] [SEED]
  GAME_ID       e.g. 5146842333902506308-5ce21003
  ACTION_INDEX  the [N] shown in the botview timeline; the position is BEFORE action N is applied
  SPEC          policy spec, default h0:nodes=16000,horizon=3,info=all (the cheater) (try h0:nodes=200000,k=16 for the deep reference,
                or knobs like k=8, depth=8, odepth=2, obeam=4, pess=1)
  SEED          explain seed (default 1); different seeds sample different hidden-information worlds
Prints the bot's candidates (aggregate value, worst world, how lines ended) and its expected line.
Read-only: replays in memory, writes nothing."""
import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import botview as bv
sys.path[:] = [p for p in sys.path if not p.rstrip('\\/').endswith('frozen')]  # current engine
import arena

gid, idx = sys.argv[1], int(sys.argv[2])
spec = sys.argv[3] if len(sys.argv) > 3 else 'h0:nodes=16000,horizon=3,info=all'
seed = int(sys.argv[4]) if len(sys.argv) > 4 else 1
db = arena.load_cards(bv.REPO + r'\cards')
rec = json.load(open(bv.REPO + rf'\results\games\{gid}.json', encoding='utf-8'))
g = arena.Game(db, rec['seed'], rec['deckA'], rec['deckB'], rec.get('first') or 'coin')
for i, a in enumerate(rec['actions'][:idx]):
    if 'reseed' in a:
        g.reseed(int(a['reseed']))
    else:
        g.apply(bv.body(a))
print(f"position before [{idx}]: turn {g.turn}, {'BOT' if g.active == 'b' else 'HUMAN'} to act, phase {g.phase}")
print('\n'.join(bv.state_lines(g)))
if idx < len(rec['actions']):
    print(f"recorded action: {bv.short_action(g, bv.body(rec['actions'][idx]))}")
e = g.bot_action_explain(spec, seed)
if isinstance(e, str):
    e = json.loads(e)
print(f"\n{spec} seed {seed}: chooses {bv.short_action(g, e.get('chosen'))}; path {e.get('path')}, nodes {e.get('nodes')}, k {e.get('k')}")
for c in sorted(e.get('candidates', []), key=lambda c: -(c['root_agg'] if c.get('root_agg') is not None else -1e9))[:40]:
    ws = [w for w in c.get('worlds', []) if not w.get('skipped')]
    ends = sorted({w.get('end') for w in ws if w.get('end')})
    per_world = ' '.join(f"{w['clamped']:+.0f}" for w in ws if w.get('clamped') is not None)
    print(f"  {c['root_agg'] if c.get('root_agg') is not None else float('nan'):+7.1f} (worst {c['worst']:+.1f}; worlds {per_world}; ends {','.join(ends)}): {bv.short_action(g, c['action'])}")
    best = max((w for w in ws if w.get('pv')), key=lambda w: w.get('clamped', -1e9), default=None)
    if best:
        pv = []
        for x in best["pv"][:6]:
            kk = next(iter(x)); vv = x[kk]
            pv.append(f"{'B' if vv.get('player') == 'b' else 'H'}:{kk}" + (f" {bv.nm(vv['card'])}" if kk == 'play' else '')
                      + (f" s{vv.get('attacker_slot')}->{vv.get('target')}" if kk == 'attack' else ''))
        print(f"        line: {' | '.join(pv)}  [end {best.get('end')}]")

v = arena.forced_lethal(g.clone(), 200000)
v = v if isinstance(v, dict) else json.loads(v)
print(f"\nexact solver (200k nodes) for the side to move: {v.get('verdict')}" + (' (rng-dependent)' if v.get('rng_dependent') else ''))
