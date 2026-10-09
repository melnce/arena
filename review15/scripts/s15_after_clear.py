"""review15: after the bot's Mars board clear (from safe_lines.json), the owner's next turn: resources, hand, the forced
kill check (budget 500 000), the most face damage the owner can deal that turn (exhaustive search over the owner's turn,
transposition on the state hash), and whether Alchemic Flare is castable with no enemy follower.
usage (repo root): python s15_after_clear.py SAFE_LINES.json GAME_ID:FIRST_PLY [...]"""
import json, sys
import arena

A = lambda v: v() if callable(v) else v
J = lambda a: json.loads(a) if isinstance(a, str) else a
db = arena.load_cards('cards')
lines = json.load(open(sys.argv[1], encoding='utf-8'))
for arg in sys.argv[2:]:
    gid, ply = arg.split(':'); ply = int(ply)
    d = json.load(open(f'results/games/{gid}.json', encoding='utf-8'))
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:ply]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    for a in lines[gid]:
        g.apply(a)
    f = J(g.full()); pa, pb = f['players']['a'], f['players']['b']
    print(f'== {gid}: after the Mars clear, the owner to move (turn {A(g.turn)})')
    print(f"  owner: {pa['leader_defense']} HP, PP {pa['pp']}/{pa['pp_max']}, evolve points {pa.get('ep')}, super-evolve points {pa.get('sep')}, field {[c['name'] for c in pa['field'] if c]}")
    print(f"  owner's hand: {[c['name'] for c in pa['hand']]}")
    print(f"  bot: {pb['leader_defense']} HP, field {[c['name'] for c in pb['field'] if c]}")
    fl = arena.forced_lethal(g, 500_000)
    print(f"  owner's forced kill: {fl['verdict']} (nodes {fl['nodes']})")
    me, t0, hp0 = A(g.active), A(g.turn), pb['leader_defense']
    best = [0, []]; seen = set(); n = [0]

    def dfs(s, line):
        n[0] += 1
        if n[0] > 1_000_000:
            return
        dmg = hp0 - J(s.full())['players']['b']['leader_defense']
        if dmg > best[0]:
            best[0], best[1] = dmg, list(line)
        if A(s.winner) is not None or A(s.turn) != t0 or A(s.active) != me:
            return
        h = s.hash()
        if h in seen:
            return
        seen.add(h)
        for a in map(J, s.legal()):
            if 'end_turn' in a:
                continue
            s2 = s.clone(); s2.apply(a); dfs(s2, line + [a])
    dfs(g, [])
    print(f"  most face damage the owner can deal this turn: {best[0]} of {hp0} ({n[0]} states, {'capped' if n[0] > 1_000_000 else 'exhaustive'}); "
          f"a line: {[json.dumps(a) for a in best[1]]}")
    print(f"  Alchemic Flare castable: {any('play' in a and a['play']['card'] == '10433310' for a in map(J, g.legal()))}")
