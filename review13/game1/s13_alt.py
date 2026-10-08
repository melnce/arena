"""review13, game 1 (4984932781931433298-b4973563), the bot's turn 6: compare the board after the bot's actual line with
the line that sends the evolve's 3 damage to the owner's 2/1 Test Subject instead (it dies), then attacks the 6/4 Test
Subject with the evolved 6/7 Highwire Feline and sends the Skeleton face as the bot did. Then the bot leader's HP through
the owner's turns 6 and 7 of the actual game. usage (repo root): python s13_alt.py"""
import json
import arena

G = 'results/games/4984932781931433298-b4973563.json'
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))


def at(n):
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def board(g):
    f = g.full(); f = json.loads(f) if isinstance(f, str) else f
    return {side: dict(hp=f['players'][side]['leader_defense'],
                       field=[f"{c['name']} {c['attack']}/{c['defense']}{' (evo)' if c.get('evolved') else ''}" for c in f['players'][side]['field'] if c])
            for side in 'ab'}


act = at(42)  # through the bot's end_turn (index 41)
alt = at(38)  # the evolve's target choice is next
alt.apply({'choose': {'option': {'slot': 0}, 'player': 'b'}})                   # 3 damage to the 2/1 Test Subject
alt.apply({'attack': {'attacker_slot': 1, 'player': 'b', 'target': {'slot': 0}}})  # Feline 6/7 into the 6/4 (now slot 0)
alt.apply({'attack': {'attacker_slot': 0, 'player': 'b', 'target': 'leader'}})     # Skeleton face, as played
alt.apply({'end_turn': {'player': 'b'}})
print('actual line (choose slot 1, Feline -> slot 1, Skeleton face), board at the owner\'s turn 6:', board(act))
print('alternative (choose slot 0, Feline -> the 6/4, Skeleton face), board at the owner\'s turn 6:', board(alt))
g = at(42)
for i, s in enumerate(d['actions'][42:], 42):
    a = {k: v for k, v in s.items() if k not in ('value', 'bot_value')}
    hb = board(g)['b']['hp']; g.apply(a); ha = board(g)['b']['hp']
    if 'attack' in a and a['attack']['player'] == 'a' and a['attack']['target'] == 'leader':
        print(f"action {i}: owner attacker slot {a['attack']['attacker_slot']} -> the bot's leader: {hb} -> {ha}")
