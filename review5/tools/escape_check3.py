"""Was the loss avoidable on the bot's last turn? For each game: at the start of the bot's last turn (before the
human's winning turn), try first moves = the recorded one, the served bot's top candidates and a perfect-information
bot's choice; complete the bot's turn with a perfect-information bot (info=all); then ask the exact forced-lethal solver
whether the human still has a forced kill. Current engine (71127bf venv build). Throwaway."""
import json, os, sys
from multiprocessing import Pool

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import botview as bv
sys.path[:] = [p for p in sys.path if not p.rstrip('\\/').endswith('frozen')]

REPO = bv.REPO
SERVED = 'h0:nodes=16000,horizon=3,info=all'
PERFECT = 'h0:nodes=16000,horizon=3,info=all'
BUDGET = 50000
GAMES = ['15092800735689879391-5ce21003', '5366400070542567175-5ce21003', '10436817046835498199-5ce21003', '17202454263948740065-5ce21003', '13991699318320519384-5ce21003', '15374915787985923294-5ce21003', '672758849267719898-5ce21003', '6889336734586405565-5ce21003', '15502683015898007302-5ce21003', '4611759015070762461-5ce21003', '2790993798946146787-5ce21003', '5951640394299249771-5ce21003', '4040560043826215616-5ce21003']
_db = None


def init():
    global _db
    import arena
    _db = arena.load_cards(REPO + r'\cards')


def J(x):
    return x if isinstance(x, (dict, list)) else json.loads(x)


def replay_to(rec, n):
    import arena
    g = arena.Game(_db, rec['seed'], rec['deckA'], rec['deckB'], rec.get('first') or 'coin')
    for a in rec['actions'][:n]:
        if 'reseed' in a:
            g.reseed(int(a['reseed']))
        else:
            g.apply(bv.body(a))
    return g


def finish_turn(g, spec, seed, cap=60):
    """Let `spec` play the bot's (seat b) turn to its end. Returns the short text of the actions played."""
    done = []
    n = 0
    while not g.terminal and g.phase in ('main', 'choice') and g.active == 'b' and n < cap:
        act = J(g.bot_action(spec, seed + n))
        k = next(iter(act))
        if act[k].get('player') != 'b':
            break
        done.append(bv.short_action(g, act))
        g.apply(act)
        n += 1
    return done


def human_kill(g):
    import arena
    if g.terminal:
        return 'game over: ' + str(g.winner)
    v = J(arena.forced_lethal(g.clone(), BUDGET))
    return f"{v.get('verdict')}" + (' (rng-dependent)' if v.get('rng_dependent') else '')


def one(gid):
    rec = json.load(open(REPO + rf'\results\games\{gid}.json', encoding='utf-8'))
    acts = rec['actions']
    # find player-turn boundaries by replaying
    g = replay_to(rec, 0)
    starts = []   # (index, turn, active) at the first main/choice action of each player-turn
    cur = None
    for i, a in enumerate(acts):
        if 'reseed' in a:
            g.reseed(int(a['reseed']))
            continue
        if g.phase in ('main', 'choice') and (g.turn, g.active) != cur:
            cur = (g.turn, g.active)
            starts.append((i, g.turn, g.active))
        g.apply(bv.body(a))
    last_h = max(s for s in starts if s[2] == 'a')
    bot_turns = [s for s in starts if s[2] == 'b' and s[0] < last_h[0]]
    out = [f"GAME {gid}: human's winning turn starts at [{last_h[0]}] (turn {last_h[1]})"]
    for bt in bot_turns[-2:][::-1]:          # the bot's last turn, then the one before it
        idx, turn, _ = bt
        g0 = replay_to(rec, idx)
        out.append(f"  bot turn {turn} from [{idx}]  (bot HP {g0.full()['players']['b']['leader_defense']}, human HP {g0.full()['players']['a']['leader_defense']})")
        # what actually happened: the recorded bot turn -> human kill?
        rec_end = next(s[0] for s in starts if s[0] > idx)
        g1 = replay_to(rec, rec_end)
        out.append(f"    recorded turn ({rec_end - idx} actions) -> human forced kill after it: {human_kill(g1)}")
        # first-move options
        e = J(g0.bot_action_explain(SERVED, rec['seed'] + idx))
        cands = sorted(e.get('candidates', []), key=lambda c: -(c['root_agg'] if c.get('root_agg') is not None else -1e9))
        firsts = [('recorded', bv.body(acts[idx]))] + [(f"served cand {c['root_agg']:+.1f} (worst {c['worst']:+.1f})", c['action']) for c in cands[:8]]
        pe = J(g0.bot_action_explain(PERFECT, rec['seed'] + idx))
        firsts.append((f"perfect-info choice", pe.get('chosen')))
        seen = set()
        for label, a0 in firsts:
            key = json.dumps(a0, sort_keys=True)
            if key in seen:
                continue
            seen.add(key)
            g2 = replay_to(rec, idx)
            try:
                first_txt = bv.short_action(g2, a0)
                g2.apply(a0)
                rest = finish_turn(g2, PERFECT, rec['seed'] + idx + 1000)
                verdict = human_kill(g2)
                fp = g2.full()['players']
                out.append(f"    {label:34s} first: {first_txt}\n        + perfect-info rest ({len(rest)} actions) -> human forced kill: {verdict};  "
                           f"after: bot HP {fp['b']['leader_defense']}, human HP {fp['a']['leader_defense']}, bot board {sum(1 for c in fp['b']['field'] if c)}, human board {sum(1 for c in fp['a']['field'] if c)}"
                           + (f"\n        rest: {' ; '.join(rest)[:400]}" if rest else ''))
            except Exception as ex:
                out.append(f"    {label}: failed {ex!r}"[:200])
    return '\n'.join(out)


if __name__ == '__main__':
    with Pool(min(8, len(GAMES)), initializer=init) as p:
        for text in p.imap(one, GAMES):
            print(text, flush=True)

