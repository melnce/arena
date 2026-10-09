"""review15: for each bot turn that ended handing the owner a forced kill, search the bot's whole turn: every distinct
end-of-turn position the bot could reach from its turn's first decision (depth-first over its legal actions, transposition
set on the state hash, capped), then forced_lethal for the owner from each (budget 20 000). Does any line leave the
owner without a kill? Prints the counts and one safe line if found, plus the bot's actual turn.
usage (repo root): python s15_safe.py GAME_ID:PLY [...]   (PLY = the index of the bot's End Turn in the capture)"""
import json, sys
import arena

BUDGET = 20_000
CAP = 4_000
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
    return json.dumps(a)[:50]


for arg in sys.argv[1:]:
    gid, ply = arg.split(':'); ply = int(ply)
    d = json.load(open(f'results/games/{gid}.json', encoding='utf-8'))
    acts = [{kk: v for kk, v in s.items() if kk not in ('value', 'bot_value')} for s in d['actions']]
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    # find the first bot action of the turn that ends at ply
    states = []
    for i, a in enumerate(acts[:ply]):
        states.append((i, A(g.turn), A(g.active)))
        g.apply(a)
    t_end = A(g.turn)
    start = min(i for i, t, act in states if t == t_end and act == 'b')
    g0 = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for a in acts[:start]:
        g0.apply(a)
    print(f"== {gid} turn {t_end}: the bot's turn is actions {start}-{ply}: {[short(a) for a in acts[start:ply + 1]]}", flush=True)
    seen = set(); leaves = []; capped = [False]

    def dfs(s, line):
        if len(leaves) >= CAP:
            capped[0] = True; return
        if A(s.winner) is not None or A(s.turn) != t_end or A(s.active) != 'b':
            return
        h = s.hash()
        if h in seen:
            return
        seen.add(h)
        for a in map(J, s.legal()):
            s2 = s.clone(); s2.apply(a)
            if 'end_turn' in a:
                leaves.append((s2, line + [a]))
                if len(leaves) >= CAP:
                    capped[0] = True; return
            else:
                dfs(s2, line + [a])

    dfs(g0, [])
    n_l = n_none = n_unk = 0; safe = None
    for s2, line in leaves:
        if A(s2.winner) is not None:
            continue
        v = arena.forced_lethal(s2, BUDGET)['verdict']
        if v == 'lethal': n_l += 1
        elif v == 'none':
            n_none += 1
            if safe is None: safe = line
        else: n_unk += 1
    print(f"  distinct bot turns searched: {len(leaves)}{' (CAP reached: not exhaustive)' if capped[0] else ' (exhaustive)'}; "
          f"owner kill after: lethal {n_l}, none {n_none}, unknown {n_unk}", flush=True)
    if safe:
        print(f"  a safe line: {[short(a) for a in safe]}", flush=True)
