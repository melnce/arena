"""review15: the bot's End Turns that handed the owner a forced kill (py/lethal_audit.py handed_lethal hits). For each:
- the owner's kill line (forced_lethal after the End Turn, budget 50 000) and whether the owner took a kill that turn;
- the bot's own view at the exact served seed (seed + the bot's earlier actions; ui/src/session.ts nextBotSeed): the
  root values, and how many of End Turn's sampled worlds ended in an opponent kill (opp_lethal / opp_solver);
- the same decision with olsolve=4000 (the opponent forced-lethal solver, off in the served spec);
- every one-action alternative to End Turn (pending choices expanded), then End Turn: does the owner still have a kill?
usage (repo root): python s15_handed.py GAME_ID:PLY [GAME_ID:PLY ...]"""
import json, sys
import arena

SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1'
BUDGET = 50_000
A = lambda v: v() if callable(v) else v
J = lambda a: json.loads(a) if isinstance(a, str) else a
db = arena.load_cards('cards')


def short(a):
    if 'end_turn' in a: return 'end_turn'
    if 'attack' in a:
        t = a['attack']['target']
        return f"attack slot{a['attack']['attacker_slot']}->{'leader' if t == 'leader' else 'enemy slot' + str(t['slot'])}"
    if 'play' in a: return f"play {a['play']['card']}"
    if 'evolve' in a: return f"{'super-' if a['evolve'].get('super') else ''}evolve slot{a['evolve']['slot']}"
    if 'choose' in a: return 'choose ' + json.dumps(a['choose']['option'])
    return json.dumps(a)[:60]


def finish_and_end(g, depth=0):
    """Expand pending choices; return the list of games after End Turn (one per choice path)."""
    legal = [J(x) for x in g.legal()]
    if any('end_turn' in a for a in legal):
        g2 = g.clone(); g2.apply({'end_turn': {'player': 'b'}}); return [g2]
    if depth > 3:
        return []
    out = []
    for a in legal:
        if 'choose' in a or 'confirm' in a:
            g2 = g.clone(); g2.apply(a); out += finish_and_end(g2, depth + 1)
    return out


for spec_arg in sys.argv[1:]:
    gid, ply = spec_arg.split(':'); ply = int(ply)
    d = json.load(open(f'results/games/{gid}.json', encoding='utf-8'))
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    k = 0
    for s in d['actions'][:ply]:
        a = {kk: v for kk, v in s.items() if kk not in ('value', 'bot_value')}
        v = list(a.values())[0]
        k += isinstance(v, dict) and v.get('player') == 'b'
        g.apply(a)
    rec = {kk: v for kk, v in d['actions'][ply].items() if kk not in ('value', 'bot_value')}
    f = J(g.full())
    hp = {s: f['players'][s]['leader_defense'] for s in 'ab'}
    print(f"== {gid} ply {ply} (turn {A(g.turn)}): recorded {short(rec)}; leaders owner {hp['a']}, bot {hp['b']}", flush=True)
    after = g.clone(); after.apply(rec)
    fl = arena.forced_lethal(after, BUDGET)
    print(f"  owner's forced kill after End Turn: {fl['verdict']} (nodes {fl['nodes']}, rng_dependent {fl.get('rng_dependent')}): "
          f"{[short(x) for x in fl.get('line', [])]}", flush=True)
    # did the owner win on that turn?
    t_owner = A(after.turn); won = False; gg = after
    for s in d['actions'][ply + 1:]:
        a = {kk: v for kk, v in s.items() if kk not in ('value', 'bot_value')}
        if A(gg.turn) != t_owner or A(gg.active) != 'a':
            break
        gg.apply(a)
        if A(gg.winner) == 'a':
            won = True; break
    print(f"  the owner {'took a kill that turn' if won else 'did NOT win that turn'} (game winner {d['winner']})", flush=True)
    served = int(d['seed']) + k
    for name, spec in (('served', SPEC), ('+olsolve=4000', SPEC + ',olsolve=4000')):
        e = J(g.bot_action_explain(spec, served))
        rows = sorted(e['candidates'], key=lambda c: -c['root_agg'])
        et = [c for c in e['candidates'] if 'end_turn' in c['action']]
        ends = {}
        for w in (et[0]['worlds'] if et else []):
            ends[w.get('end')] = ends.get(w.get('end'), 0) + 1
        print(f"  {name} at the served seed: path {e.get('path')} chosen {short(e['chosen'])} | End Turn worlds {ends} | " +
              '  '.join(f"{short(c['action'])} {c['root_agg']:.2f}" for c in rows[:6]), flush=True)
    alts = [J(x) for x in g.legal() if 'end_turn' not in J(x)]
    res = []
    for a in alts:
        g2 = g.clone(); g2.apply(a)
        verdicts = []
        for g3 in finish_and_end(g2):
            verdicts.append(arena.forced_lethal(g3, BUDGET)['verdict'])
        res.append((short(a), verdicts))
    safe = [r for r in res if r[1] and all(v == 'none' for v in r[1])]
    print(f"  one-action alternatives ({len(alts)}): " + '; '.join(f"{n} -> {','.join(v) or 'n/a'}" for n, v in res), flush=True)
    print(f"  alternatives that leave the owner no kill: {[n for n, _ in safe] or 'none'}", flush=True)
