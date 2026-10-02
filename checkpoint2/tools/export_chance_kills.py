"""Export the checkpoint 2 games that contain a chance-dependent-only kill turn (an auditee turn whose 'lethal' verdicts
are all rng_dependent), with their full action logs, for an offline dice-replay analysis.
The audit JSONs keep no action logs, so each game is replayed with the audit's own play_one (py/lethal_audit.py):
checkpoint 2 ran plain h0 on 8e66bcf, where v2 was the default leaf; this build's default is v3, so the replay pins
v2 by path (h0:net=engine/models/h0-linear-v2.json; the keys added since - okill/omacro/tkill - are off by default).
Each replay is checked: the winner must match the record, and at every decision of the chance-only kill turns the
solver (arena.forced_lethal, the audit's budget) must give the recorded verdict and rng_dependent flag.
Usage: python export_chance_kills.py OUT.json [MAX_GAMES]. Run from the repo root."""
import json, os, sys, time
sys.path.insert(0, os.path.join(os.getcwd(), 'py'))
import arena
import lethal_audit as la

OUT = sys.argv[1]
MAXG = int(sys.argv[2]) if len(sys.argv) > 2 else None
SPEC = 'h0:net=engine/models/h0-linear-v2.json'
R = os.path.join('results', 'lethal')
db = arena.load_cards('cards')
_, decks = la.resolve_selected_decks(la.Path('oracle') / 'decks', 'meta', None)
J = lambda x: json.loads(x) if isinstance(x, str) else x

targets = []
for half in ('fa', 'fb'):
    d = json.load(open(os.path.join(R, f'checkpoint2-{half}.json'), encoding='utf-8'))
    assert d['policy_a'] == d['policy_b'] == 'h0' and d['bot_seat'] == 'both', (d['policy_a'], d['bot_seat'])
    budget = d['budget']
    for r in d['records']:
        turns = {}
        for x in r['decisions']:
            if x['kind'] == 'missed_lethal' and x['verdict'] == 'lethal':
                turns.setdefault((x['auditee'], x['turn']), []).append(x)
        chance = {k: v for k, v in turns.items() if all(x.get('rng_dependent') is not False for x in v)}
        if chance:
            targets.append((half, budget, r, chance))
print(f"games with a chance-only kill turn: {len(targets)}; turns: {sum(len(c) for *_, c in targets)}", flush=True)
if MAXG:
    targets = targets[:MAXG]

games, problems = [], []
t0 = time.time()
for i, (half, budget, r, chance) in enumerate(targets, 1):
    rec = la.play_one(db, r['seed'], decks[r['deck_a']], decks[r['deck_b']], r['first'], SPEC, SPEC)
    acts = rec['actions']
    ok = rec['winner'] == r['winner']
    checks = []
    # re-solve every decision of the chance-only turns at its ply
    want = {x['ply']: x for v in chance.values() for x in v}
    g = arena.Game(db, r['seed'], decks[r['deck_a']], decks[r['deck_b']], r['first'])
    for ply in range(len(acts) + 1):
        if ply in want:
            x = want[ply]
            v = J(arena.forced_lethal(g.clone(), budget))
            got = (v.get('verdict'), v.get('rng_dependent'))
            checks.append(got == ('lethal', True) and la.acting_of(g) == x['acting'])
        if ply < len(acts):
            g.apply(acts[ply])
    ok = ok and all(checks) and len(checks) == len(want)
    if not ok:
        problems.append({'half': half, 'game_id': r['game_id'], 'winner_match': rec['winner'] == r['winner'],
                         'checks': checks, 'n_want': len(want)})
    games.append({
        'audit_file': f'checkpoint2-{half}.json', 'game_id': r['game_id'], 'seed': r['seed'],
        'deck_a': r['deck_a'], 'deck_b': r['deck_b'], 'deck_a_cards': decks[r['deck_a']], 'deck_b_cards': decks[r['deck_b']],
        'first': r['first'], 'winner': r['winner'], 'replay_verified': ok,
        'actions': acts,
        'chance_kill_turns': [{'auditee': a, 'turn': t, 'converted': bool(v[0].get('converted')),
                               'decisions': [{k: y for k, y in x.items()} for x in v]} for (a, t), v in sorted(chance.items())],
    })
    if i % 10 == 0 or i == len(targets):
        print(f"  {i}/{len(targets)} games, {len(problems)} problems, {time.time() - t0:.0f} s", flush=True)

out = {
    'about': ('checkpoint 2 lethal audit (results/lethal/checkpoint2-f{a,b}.json; h0 on 8e66bcf, v2 default leaf, '
              'solver budget %d): the games that contain a chance-dependent-only kill turn, with full action logs '
              'replayed on %s using the audit\'s play_one and policy %s for both seats (bot_action(spec, seed + n) '
              'for the n-th action; actions[0..1] are the mulligans). A decision record\'s ply = the number of actions '
              'applied before it. replay_verified: the winner matches the audit record and the solver re-finds a '
              'dice-dependent kill (verdict lethal, rng_dependent true) at every decision of those turns.'
              % (budget, 'the build of d5f4310', SPEC)),
    'definitions': {'chance_kill_turn': "an (auditee, turn) whose missed_lethal decisions with verdict 'lethal' all have "
                                        'rng_dependent true (no luck-free kill at any point of the turn)',
                    'converted': 'the auditee won during that turn (as recorded by the audit)'},
    'policy_replayed': SPEC, 'decks_source': 'oracle/decks (pool meta)',
    'n_games': len(games), 'n_turns': sum(len(g['chance_kill_turns']) for g in games),
    'n_turns_missed': sum(1 for g in games for t in g['chance_kill_turns'] if not t['converted']),
    'replay_problems': problems,
    'games': games,
}
json.dump(out, open(OUT, 'w', encoding='utf-8', newline='\n'), separators=(',', ':'))
print(f"wrote {OUT}: {out['n_games']} games, {out['n_turns']} turns ({out['n_turns_missed']} missed), "
      f"{len(problems)} replay problems, {os.path.getsize(OUT) / 1e6:.2f} MB", flush=True)
