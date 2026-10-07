"""Exhaustive check: can the side to move win this turn? Depth-first over every legal action sequence of the current
turn (full information, the game's own RNG), with a transposition set on the state hash.
usage (repo root): python lethal_dfs.py GAME.json N [action-json ...]
  replays the capture's first N actions, applies the extra actions (JSON objects), then searches the side to move.
Prints whether a winning line exists, the line, and the number of states visited."""
import json, sys
import arena

db = arena.load_cards('cards')
d = json.load(open(sys.argv[1], encoding='utf-8'))
n = int(sys.argv[2])
g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
for s in d['actions'][:n]:
    g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
for x in sys.argv[3:]:
    g.apply(json.loads(x))
A = lambda v: v() if callable(v) else v
me = A(g.active)
turn0 = A(g.turn)
print(f'searching side {me} on turn {turn0}')
seen = set(); visited = 0; LIMIT = 2_000_000
best = None


def dfs(state, line):
    global visited, best
    if visited >= LIMIT or best is not None:
        return
    visited += 1
    w = A(state.winner)
    if w == me:
        best = list(line); return
    if w is not None or A(state.terminal) or A(state.turn) != turn0 or A(state.active) != me:
        return
    h = state.hash()
    if h in seen:
        return
    seen.add(h)
    for a in state.legal():
        a = json.loads(a) if isinstance(a, str) else a
        if 'end_turn' in a:
            continue
        s2 = state.clone()
        try:
            s2.apply(a)
        except Exception:
            continue
        line.append(a)
        dfs(s2, line)
        line.pop()
        if best is not None:
            return


dfs(g, [])
print(f'visited {visited} states ({len(seen)} distinct){" - LIMIT reached, not exhaustive" if visited >= LIMIT else ""}')
if best:
    print('WIN FOUND:')
    for a in best:
        print('   ', json.dumps(a))
else:
    print('no winning line this turn' + ('' if visited < LIMIT else ' within the limit'))
