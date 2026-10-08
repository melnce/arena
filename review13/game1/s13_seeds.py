"""review13, game 1: how often does the served spec make the recorded choice at the bot's decisions after 27 actions
(turn 5: End Turn vs the evolved Feline attacking the owner's 2/1), 38 (turn 6: the evolve's target), 40 (turn 6: the
Skeleton's attack)? serve.py takes the bot seed from the browser per request and does not store it, so sample seeds.
Prints, per decision, the choice counts over seeds 1..N and any seed whose value equals the capture's bot_value.
usage (repo root): python s13_seeds.py [N=32]"""
import json, sys
from collections import Counter
import arena

G = 'results/games/4984932781931433298-b4973563.json'
SPEC = 'h0:nodes=32000,horizon=3,k=8,tkill=10000,tkroll=8,hbcheck=2000,fuseguard=1,net=results/sweep43/leaf-v3-e3.json'
N = int(sys.argv[1]) if len(sys.argv) > 1 else 32
db = arena.load_cards('cards')
d = json.load(open(G, encoding='utf-8'))


def at(n):
    g = arena.Game(db, d['seed'], d['deckA'], d['deckB'], d['first'])
    for s in d['actions'][:n]:
        g.apply({k: v for k, v in s.items() if k not in ('value', 'bot_value')})
    return g


def short(a):
    if 'end_turn' in a: return 'end_turn'
    if 'attack' in a:
        t = a['attack']['target']
        return f"attack slot{a['attack']['attacker_slot']}->{'leader' if t == 'leader' else 'enemy slot' + str(t['slot'])}"
    if 'choose' in a: return 'choose ' + json.dumps(a['choose']['option'])
    return json.dumps(a)[:60]


for n in (27, 38, 40):
    g = at(n)
    rec = d['actions'][n]
    played = short({k: v for k, v in rec.items() if k not in ('value', 'bot_value')})
    cnt = Counter(); same = []
    for seed in range(1, N + 1):
        r = g.bot_action_value(SPEC, seed)
        r = json.loads(r) if isinstance(r, str) else r
        a, v = (r['action'], r['value']) if isinstance(r, dict) else (r[0], r[1])
        a = json.loads(a) if isinstance(a, str) else a
        cnt[short(a)] += 1
        if rec.get('bot_value') is not None and abs(v - rec['bot_value']) < 1e-4:
            same.append(seed)
    print(f"after {n} actions, played {played} (bot_value {rec.get('bot_value')}): over seeds 1-{N}: {dict(cnt)}; "
          f"seeds reproducing the recorded value: {same}", flush=True)
