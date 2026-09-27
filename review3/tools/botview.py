"""Bot's-eye timelines of the review3 games (human = seat a, bot h0:nodes=16000 = seat b).

Writes scratchpad/botview/<game_id>.txt and scratchpad/botview/CARDS.txt.
For every bot decision it re-runs Game.bot_action_explain(BOT, seed) on the replayed position
(the live botSeed was not recorded, so the explained choice can differ from the recorded one;
that is reported). Throwaway."""
import glob, json, os, sys
from multiprocessing import Pool

REPO = r'C:\Users\agban\projects\arena'
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'botview')
BOT = 'h0:nodes=16000'
NOISE = 11.5
sys.path.insert(0, REPO + r'\py')
# frozen copy of the engine build the games were played on (a45e2bb), so a rebuild of the venv cannot change replays
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), 'frozen'))

_db = None
CARDS = {}
for f in glob.glob(REPO + r'\cards\**\*.json', recursive=True):
    try:
        c = json.load(open(f, encoding='utf-8'))
    except Exception:
        continue
    if isinstance(c, dict) and 'id' in c and 'name' in c:
        CARDS[str(c['id'])] = c
DECKNAMES = {}
for f in glob.glob(REPO + r'\oracle\decks\meta-*.json'):
    d = json.load(open(f, encoding='utf-8'))
    cards = d.get('cards', d) if isinstance(d, dict) else {}
    if isinstance(cards, dict):
        DECKNAMES[tuple(sorted((str(k), int(v)) for k, v in cards.items() if str(k).isdigit()))] = os.path.basename(f)[:-5]


def deckname(dk):
    return DECKNAMES.get(tuple(sorted((str(k), int(v)) for k, v in dk.items())), 'custom')


def nm(cid):
    return CARDS.get(str(cid), {}).get('name', str(cid))


def unit(c):
    if not c:
        return '(empty slot)'
    if c.get('kind') == 'follower':
        tr = [t for t in c.get('traits', []) if t in ('ward', 'storm', 'rush', 'bane', 'drain', 'ambush', 'intimidate', 'barrier')]
        ev = '*SE' if c.get('super_evolved') else ('*E' if c.get('evolved') else '')
        fl = c.get('flags', {})
        extra = []
        if fl.get('summoning_sick'):
            extra.append('sick')
        if fl.get('ambush_active'):
            extra.append('ambush')
        return f"{c['name']} {c['attack']}/{c['defense']}{ev}" + (f" [{','.join(tr + extra)}]" if tr or extra else '')
    if c.get('kind') == 'amulet':
        cd = c.get('countdown')
        return f"{c['name']} (amulet{', cd ' + str(cd) if cd is not None else ''})"
    return c.get('name', '?')


def state_lines(g):
    f = g.full()
    out = []
    for s, who in (('b', 'BOT'), ('a', 'HUMAN')):
        p = f['players'][s]
        bonus = ''
        if p.get('is_second'):
            bonus = f" bonusPP(early {'used' if not p.get('pp_bonus_early') else 'avail'}, late {'used' if not p.get('pp_bonus_late') else 'avail'}{', ACTIVE' if p.get('pp_bonus_active') else ''})"
        out.append(f"    {who:5} HP {p['leader_defense']}/{p['leader_max']}  PP {p['pp']}/{p['pp_max']}  EP {p['ep']} SEP {p['sep']}  "
                   f"deck {len(p['deck'])}  hand {len(p['hand'])}  cemetery {len(p['cemetery'])}{bonus}")
        out.append(f"          field: " + ('; '.join(f"[{i}] {unit(c)}" for i, c in enumerate(p['field']) if c) or '(empty)'))
        hand = ', '.join(f"{c['name']}({c['cost']})" for c in p['hand'] if c)
        out.append(f"          hand{' (HIDDEN from the bot)' if s == 'a' else ''}: {hand or '(empty)'}")
    return out


def describe(g, a):
    """Human-readable action on the current (pre-apply) state."""
    if not isinstance(a, dict) or not a:
        return str(a)
    k, v = next(iter(a.items()))
    f = g.full()
    who = 'BOT' if v.get('player') == 'b' else 'HUMAN'
    me = f['players'].get(v.get('player', 'a'), {})
    opp = f['players']['b' if v.get('player') == 'a' else 'a'] if v.get('player') in ('a', 'b') else {}
    if k == 'play':
        tgt = ''
        for key in ('target', 'targets'):
            if key in v:
                tgt = f" -> {v[key]}"
        return f"{who} plays {nm(v['card'])} ({CARDS.get(str(v['card']), {}).get('cost', '?')}pp){tgt}"
    if k == 'attack':
        att = me['field'][v['attacker_slot']] if v.get('attacker_slot', 99) < len(me.get('field', [])) else {}
        t = v.get('target')
        if t == 'leader':
            tt = 'LEADER'
        elif isinstance(t, dict) and 'slot' in t and t['slot'] < len(opp.get('field', [])):
            tt = unit(opp['field'][t['slot']])
        else:
            tt = str(t)
        return f"{who} attacks: {unit(att) if att else v} -> {tt}"
    if k == 'evolve':
        u = me['field'][v['slot']] if v['slot'] < len(me.get('field', [])) else {}
        return f"{who} {'SUPER-evolves' if v.get('super') else 'evolves'} {unit(u) if u else v['slot']}"
    if k == 'end_turn':
        return f"{who} ends turn"
    if k == 'bonus_pp':
        return f"{who} toggles bonus PP"
    if k == 'choose':
        return f"{who} chooses {json.dumps(v.get('option'))}"
    if k == 'mulligan':
        return f"{who} mulligan swap {v.get('swap')}"
    return f"{who} {k} {json.dumps(v)}"


def short_action(g, a):
    try:
        return describe(g, a)
    except Exception:
        return json.dumps(a)[:120]


def body(a):
    return {k: v for k, v in a.items() if k not in ('bot_value',)}


def init():
    global _db
    import arena
    _db = arena.load_cards(REPO + r'\cards')


def explain(g, seed):
    try:
        e = g.bot_action_explain(BOT, seed)
    except Exception as ex:
        return None, f"explain failed: {ex!r}"[:200]
    if isinstance(e, str):
        e = json.loads(e)
    return e, None


def one(gid):
    import arena
    rev = json.load(open(REPO + rf'\results\review3\{gid}.json', encoding='utf-8'))
    rec = json.load(open(REPO + rf'\results\games\{gid}.json', encoding='utf-8'))
    decs = {d['ply']: d for d in rev['decisions']}
    g = arena.Game(_db, rec['seed'], rec['deckA'], rec['deckB'], rec.get('first') or 'coin')
    first_sec = g.full()
    second = [p for p in 'ab' if first_sec['players'][p].get('is_second')]
    first = 'b' if second == ['a'] else 'a'
    L = []
    L.append(f"GAME {gid}")
    L.append(f"human (seat a) deck: {deckname(rec['deckA'])};  bot (seat b, {BOT}, info=open) deck: {deckname(rec['deckB'])}")
    L.append(f"went first: {'BOT' if first == 'b' else 'HUMAN'};  winner: {'BOT' if rev['winner'] == 'b' else 'HUMAN'};  turns: {rev['turns']}")
    L.append("Values: +80 = forced win for the side named, -80 = forced loss. 'ref' = h0:nodes=200000,k=16 (deeper search).")
    L.append("Per decision: bot_value = the bot's own search value at the time; v_best/v_played = reference value of its best move / the move played;")
    L.append("cost = v_best - v_played (>11.5 is beyond measurement noise). All values from the ACTING side's point of view.")
    L.append("explain = the bot's own candidate list re-run on this position (seed differs from the live game, so 'explained choice' may differ).")
    L.append("")
    # bot-perspective reference trace
    trace = []
    seen_turn = set()
    for d in rev['decisions']:
        key = (d['turn'], d['acting'])
        if key in seen_turn:
            continue
        seen_turn.add(key)
        vb = d['v_best'] if d['acting'] == 'b' else -d['v_best']
        trace.append(f"t{d['turn']}{'B' if d['acting'] == 'b' else 'H'}:{vb:+.0f}")
    L.append("Reference value from the BOT's side at the start of each analysed player-turn (tN B = bot's turn, H = human's):")
    L.append("  " + '  '.join(trace))
    L.append("")
    cur_turn_key = None
    flagged = 0
    for i, a in enumerate(rec['actions']):
        if 'reseed' in a:
            g.reseed(int(a['reseed']))
            continue
        b = body(a)
        k = next(iter(b))
        player = b[k].get('player')
        if g.phase in ('main', 'choice') and (g.turn, g.active) != cur_turn_key:
            cur_turn_key = (g.turn, g.active)
            L.append(f"=== turn {g.turn}, {'BOT' if g.active == 'b' else 'HUMAN'} to act ===")
            L.extend(state_lines(g))
        line = f"  [{i}] {short_action(g, b)}"
        d = decs.get(i)
        ex_lines = []
        if player == 'b' and k != 'mulligan':
            if 'bot_value' in a:
                line += f"   bot_value {a['bot_value']:+.1f}"
            if d:
                if 'cost' in d:
                    line += f"   ref: played {d['v_played']:+.1f}, best {d['v_best']:+.1f}, cost {d['cost']:.1f}"
                    if d['cost'] > NOISE:
                        line += f"  <<< MISTAKE; ref prefers: {short_action(g, d['reference'])}"
                elif 'optimism' in d:
                    line += f"   ref after end-turn (bot's view): {d['v_played']:+.1f}; optimism {d['optimism']:+.1f}"
                    if d['v_played'] <= -79.99:
                        line += "  <<< ENDED TURN WITH HUMAN HOLDING A FORCED WIN"
            flag = d is not None and ((d.get('cost', 0) > NOISE) or (d.get('optimism') is not None and d['v_played'] <= -40))
            e, err = explain(g, rec['seed'] + i)
            if err:
                ex_lines.append('      ' + err)
            elif e:
                cands = sorted(e.get('candidates', []), key=lambda c: -(c.get('root_agg') if c.get('root_agg') is not None else -1e9))
                chosen = e.get('chosen')
                same = json.dumps(chosen, sort_keys=True) == json.dumps(b, sort_keys=True)
                margin = (cands[0]['root_agg'] - cands[1]['root_agg']) if len(cands) > 1 and cands[1].get('root_agg') is not None else None
                ex_lines.append(f"      explain: path {e.get('path')}, {len(cands)} candidates, nodes {e.get('nodes')}"
                                + (f", top-2 margin {margin:.1f}" if margin is not None else '')
                                + ('' if same else f"; re-run would choose: {short_action(g, chosen)}"))
                if flag:
                    flagged += 1
                    for c in cands[:4]:
                        ws = [w for w in c.get('worlds', []) if not w.get('skipped')]
                        ends = sorted({w.get('end') for w in ws if w.get('end')})
                        ex_lines.append(f"        cand {c['root_agg']:+7.1f} (worst {c['worst']:+.1f}, n {c['n']}, ends {','.join(ends)}): {short_action(g, c['action'])}")
                    best = max((w for w in cands[0].get('worlds', []) if not w.get('skipped') and w.get('pv')), key=lambda w: w.get('clamped', -1e9), default=None)
                    if best:
                        pv = []
                        for x in best['pv'][:8]:
                            kk = next(iter(x))
                            vv = x[kk]
                            pv.append(f"{'B' if vv.get('player') == 'b' else 'H'}:{kk}" + (f" {nm(vv['card'])}" if kk == 'play' else '')
                                      + (f" s{vv.get('attacker_slot')}->{vv.get('target')}" if kk == 'attack' else ''))
                        ex_lines.append(f"        bot's expected line for its top candidate (best world): {' | '.join(pv)}  [end {best.get('end')}, leaf {best.get('leaf', {}) and round(best['leaf'].get('value', 0), 1)}]")
        elif player == 'a' and d and d.get('cost', 0) > NOISE:
            line += f"   (human move; ref cost {d['cost']:.1f})"
        if player == 'a' and d and 'optimism' not in d and d.get('v_best', 0) >= 79.99 and d.get('v_played', 0) < 79.99:
            line += "   (human missed a forced win here)"
        L.append(line)
        L.extend(ex_lines)
        g.apply(b)
    L.append(f"\nFINAL: winner {'BOT' if g.winner == 'b' else 'HUMAN'}; bot HP {g.full()['players']['b']['leader_defense']}, human HP {g.full()['players']['a']['leader_defense']}")
    os.makedirs(OUT, exist_ok=True)
    open(os.path.join(OUT, gid + '.txt'), 'w', encoding='utf-8').write('\n'.join(L) + '\n')
    return gid, len(L), flagged, deckname(rec['deckA']), rev['winner'], first


def cards_file(gids):
    ids = set()
    for gid in gids:
        rec = json.load(open(REPO + rf'\results\games\{gid}.json', encoding='utf-8'))
        ids |= set(map(str, rec['deckA'])) | set(map(str, rec['deckB']))
    # tokens named by these cards' effects
    def walk(o):
        if isinstance(o, dict):
            if 'named' in o:
                yield str(o['named'])
            for v in o.values():
                yield from walk(v)
        elif isinstance(o, list):
            for v in o:
                yield from walk(v)
    for cid in list(ids):
        ids |= set(walk(CARDS.get(cid, {}).get('abilities', [])))
    L = []
    for cid in sorted(ids, key=lambda x: (CARDS.get(x, {}).get('class', ''), CARDS.get(x, {}).get('cost', 99), x)):
        c = CARDS.get(cid)
        if not c:
            continue
        st = f" {c.get('attack')}/{c.get('defense')}" if c.get('kind') == 'follower' else ''
        L.append(f"{cid}  {c['name']}  [{c.get('class')} {c.get('kind')}{' token' if c.get('token') else ''}, cost {c.get('cost')}{st}]")
        L.append('    ' + c.get('text', '').replace('\n', '\n    '))
    open(os.path.join(OUT, 'CARDS.txt'), 'w', encoding='utf-8').write('\n'.join(L) + '\n')


if __name__ == '__main__':
    gids = sorted(os.path.basename(f)[:-5] for f in glob.glob(REPO + r'\results\review3\*-*.json'))
    os.makedirs(OUT, exist_ok=True)
    cards_file(gids)
    with Pool(min(18, os.cpu_count()), initializer=init) as p:
        for r in p.imap_unordered(one, gids):
            print(*r, flush=True)
