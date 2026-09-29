"""Follow-ups: resolve two 'unknown' solver verdicts with a 10x budget; show the recorded vs the safe line in 9420 t7."""
import json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import botview as bv
sys.path[:] = [p for p in sys.path if not p.rstrip('\\/').endswith('frozen')]
import arena
import escape_check as ec
from botview2 import pv_text

db = arena.load_cards(bv.REPO + r'\cards')
ec._db = db
J = ec.J


def rec_of(gid):
    return json.load(open(bv.REPO + rf'\results\games\{gid}.json', encoding='utf-8'))


# 1) the two unknowns, 10x budget: the human's position right after the bot's recorded turn
for gid, idx in (('14155189002142913913-5ce21003', 51), ('11296488206978013884-5ce21003', 47)):
    rec = rec_of(gid)
    g = ec.replay_to(rec, idx)
    who = 'HUMAN' if g.active == 'a' else 'BOT'
    v = J(arena.forced_lethal(g.clone(), 500000))
    print(f"{gid} before [{idx}] ({who} to act, turn {g.turn}): verdict {v.get('verdict')} (nodes {v.get('nodes')}, rng {v.get('rng_dependent')})"
          + (f"; line: {pv_text(v.get('line') or [])}" if v.get('verdict') == 'lethal' else ''))

# 2) 9420 turn 7: recorded bot turn vs the perfect-info continuation after the same first move (Hark)
gid = '9420046197828951589-5ce21003'
rec = rec_of(gid)
g = ec.replay_to(rec, 64)
print(f"\n{gid} bot turn 7 from [64]:")
print('  recorded:', ' ; '.join(f"[{i}] {bv.short_action(ec.replay_to(rec, i), bv.body(rec['actions'][i]))}" for i in range(64, 71)))
g2 = ec.replay_to(rec, 64)
first = bv.body(rec['actions'][64])
g2.apply(first)
rest = ec.finish_turn(g2, ec.PERFECT, rec['seed'] + 64 + 1000)
print('  safe (same first move, perfect-info rest):', bv.short_action(ec.replay_to(rec, 64), first), ';', ' ; '.join(rest))
print('  human kill after safe line:', ec.human_kill(g2))
# where do they diverge, and what did the served bot see there?
g3 = ec.replay_to(rec, 65)
for j, i in enumerate(range(65, 71)):
    ra = bv.body(rec['actions'][i])
    pa = J(g3.bot_action(ec.PERFECT, rec['seed'] + 64 + 1000 + j))
    same = json.dumps(ra, sort_keys=True) == json.dumps(pa, sort_keys=True)
    if not same:
        print(f"  first divergence at [{i}]: recorded {bv.short_action(g3, ra)}  vs  perfect-info {bv.short_action(g3, pa)}")
        e = J(g3.bot_action_explain(ec.SERVED, rec['seed'] + i))
        for c in sorted(e.get('candidates', []), key=lambda c: -(c['root_agg'] if c.get('root_agg') is not None else -1e9))[:5]:
            ws = [w for w in c.get('worlds', []) if not w.get('skipped')]
            per_world = ' '.join('%+.0f' % w['clamped'] for w in ws if w.get('clamped') is not None)
            ends = ','.join(sorted({w.get('end') for w in ws if w.get('end')}))
            print(f"    served cand {c['root_agg']:+7.1f} worst {c['worst']:+6.1f} worlds {per_world} ends {ends}: {bv.short_action(g3, c['action'])}")
        # and the solver right after each of the two choices, with the rest of the turn played by the perfect-info bot
        for lab, act in (('recorded', ra), ('perfect-info', pa)):
            g4 = ec.replay_to(rec, i)
            g4.apply(act)
            ec.finish_turn(g4, ec.PERFECT, rec['seed'] + 7000)
            print(f"    after {lab} choice + perfect-info rest -> human kill: {ec.human_kill(g4)}")
        break
    g3.apply(ra)
