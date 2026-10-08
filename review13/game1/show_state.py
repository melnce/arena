"""Print a compact board state of a captured vs-bot game after N actions (full information).
usage (repo root): python show_state.py GAME.json N [N2 ...]"""
import json, sys
import arena


def fol(c):
    fl = c.get('flags', {})
    tags = []
    if c.get('super_evolved'): tags.append('SEVO')
    elif c.get('evolved'): tags.append('evo')
    if c.get('traits'): tags.append('/'.join(c['traits']))
    can = fl.get('attacks_left', 0) > 0 and not (fl.get('summoning_sick') and not ({'rush', 'storm'} & set(c.get('traits') or [])))
    if c.get('kind') == 'follower':
        tags.append(f"atk_left={fl.get('attacks_left')}{' sick' if fl.get('summoning_sick') else ''}")
        return f"{c['name']} [{c['card']}] {c['attack']}/{c['defense']} ({', '.join(tags)})"
    cd = c.get('countdown')
    return f"{c['name']} [{c['card']}] {c.get('kind')}{' cd=' + str(cd) if cd is not None else ''}"


def show(g, label):
    f = g.full(); f = json.loads(f) if isinstance(f, str) else f
    print(f"== {label}: turn {f['turn']}, active {f['active']}, phase {f['phase']}")
    for side in ('a', 'b'):
        p = f['players'][side]
        print(f"  [{side}] leader {p['leader_defense']}/{p['leader_max']}  pp {p['pp']}/{p['pp_max']}  ep {p.get('ep')} sep {p.get('sep')}"
              f"  evolves_used {p.get('evolves_used')}  crests {[c.get('name', c) if isinstance(c, dict) else c for c in p.get('crests', [])]}"
              f"  enter_counts {p.get('enter_counts')}")
        for i, c in [(i, c) for i, c in enumerate(p["field"]) if c]:
            print(f"      field[{i}] {fol(c)}")
        print(f"      hand: {[c['name'] + ' [' + c['card'] + '] c' + str(c.get('cost')) for c in p['hand']]}")


db = arena.load_cards('cards')
d = json.load(open(sys.argv[1], encoding='utf-8'))
g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
marks = sorted(int(x) for x in sys.argv[2:])
i = 0
for s in d['actions']:
    while marks and marks[0] == i:
        show(g, f'after {i} actions'); marks.pop(0)
    g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    i += 1
for m in marks:
    show(g, f'after {i} actions (end)')
