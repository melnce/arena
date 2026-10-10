"""Repro: arena.forced_lethal misses kills that fuse with partners (engine/src/lethal.rs).

pos_key(state) = (hash(state), rng fingerprint). hash() is the public snapshot hash, and it does not change when a fuse
partner is chosen in the Choice phase. So after Fuse, the `choose` child has the same pos_key as its parent, which is
on path_keys, and the search skips it as a cycle: no fuse with a partner is ever completed.

Position: review15 game 4 (1068176807631359990-b43e6214), the real 86 actions, then the bot's Mars board clear
(review15/safe_lines.json). The owner (a) is to move with 9 PP; the bot is at 9 with an empty board.
Owner's kill: Tetra & Ladica, super-evolve (8/8 Storm), face 8; fuse a card onto Sephie in hand (2 PP: an Obsessed Test
Subject, given Storm by the owner's Sephie crest), face 5. 13 into 9.
usage (repo root): python fuse_lethal_repro.py <review15 dir>"""
import json, sys, os
import arena

R = sys.argv[1]
GID = '1068176807631359990-b43e6214'
J = lambda x: json.loads(x) if isinstance(x, str) else x
A = lambda v: v() if callable(v) else v
db = arena.load_cards('cards')
d = json.load(open(os.path.join(R, 'games', GID + '.json'), encoding='utf-8'))
g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
for a in d['actions'][:86]:
    g.apply({k: v for k, v in a.items() if k not in ('value', 'bot_value')})
for a in json.load(open(os.path.join(R, 'safe_lines.json'), encoding='utf-8'))[GID]:
    g.apply(a)
print('owner to move:', A(g.active), '| forced_lethal:', {k: v for k, v in arena.forced_lethal(g, 200_000).items() if k != 'line'})

line = [{"play": {"card": "10834110", "hand_pos": 3, "player": "a"}},
        {"evolve": {"player": "a", "slot": 0, "super": True}},
        {"attack": {"attacker_slot": 0, "player": "a", "target": "leader"}}]
for a in line:
    g.apply(a)
print('after Tetra & Ladica, super-evolve, face (bot at %d):' % J(g.snapshot())['players']['b']['leader_defense'],
      {k: v for k, v in arena.forced_lethal(g, 200_000).items() if k != 'line'})
g.apply({"fuse": {"host_pos": 0, "partner_pos": [], "player": "a"}})
h0 = g.hash(); ph0 = A(g.phase)
g.apply({"choose": {"option": {"card": "10932110"}, "player": "a"}})
print(f'fuse onto Sephie, then choose a partner: hash {h0} -> {g.hash()} (phase {ph0} -> {A(g.phase)})')
print('after the choose, Confirm pending:', {k: v for k, v in arena.forced_lethal(g, 200_000).items() if k != 'line'})
