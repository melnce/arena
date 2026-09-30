"""Bot's-eye timelines for the 2026-09-29 games vs h0:nodes=16000,horizon=3 (engine 71127bf, CURRENT venv build;
no deep reference). Every bot decision: bot_action_explain re-run at the served spec. Every decision point (both
sides): the exact forced-lethal solver on a clone. Throwaway."""
import glob, json, os, sys
from multiprocessing import Pool

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import botview as bv  # helpers only (names, state lines, action text)
sys.path[:] = [p for p in sys.path if not p.rstrip('\\/').endswith('frozen')]  # use the CURRENT engine, not a45e2bb

REPO = bv.REPO
OUT = os.path.join(HERE, 'botview3')
BOT = 'h0:nodes=16000,horizon=3,info=all'
LETHAL_BUDGET = 50000
T0 = '2026-09-30T08:40'  # games finished after the server start (UTC 00:04 = 02:04 local)
_db = None


def init():
    global _db
    import arena
    _db = arena.load_cards(REPO + r'\cards')


def lethal(g):
    import arena
    v = arena.forced_lethal(g.clone(), LETHAL_BUDGET)
    return v if isinstance(v, dict) else json.loads(v)


def pv_text(pv):
    out = []
    for x in pv[:10]:
        kk = next(iter(x)); vv = x[kk]
        out.append(f"{'B' if vv.get('player') == 'b' else 'H'}:{kk}" + (f" {bv.nm(vv['card'])}" if kk == 'play' else '')
                   + (f" s{vv.get('attacker_slot')}->{vv.get('target')}" if kk == 'attack' else '')
                   + (f" {json.dumps(vv.get('option'))}" if kk == 'choose' else ''))
    return ' | '.join(out)


def one(gid):
    import arena
    rec = json.load(open(REPO + rf'\results\games\{gid}.json', encoding='utf-8'))
    g = arena.Game(_db, rec['seed'], rec['deckA'], rec['deckB'], rec.get('first') or 'coin')
    f0 = g.full()
    first = 'b' if f0['players']['a'].get('is_second') else 'a'
    L = [f"GAME {gid}  (engine {rec.get('engine')}, bot {rec.get('policy')} = the CHEATER: sees the human's hand and deck contents, not the draw order (draws stay random); leaf h0-linear-v2)",
         f"human (seat a) deck: {bv.deckname(rec['deckA'])};  bot (seat b) deck: {bv.deckname(rec['deckB'])}",
         f"went first: {'BOT' if first == 'b' else 'HUMAN'};  winner: {'BOT' if rec.get('winner') == 'b' else 'HUMAN'};  actions {len(rec['actions'])}",
         "Values: +80 = forced win for the side to move, -80 = forced loss; bot values from the bot's side.",
         "Per bot decision: bot_value = the live bot's own value; explain = the served spec re-run on this position (seed differs",
         "from the live game, so 're-run would choose' can differ); candidates show aggregate over worlds, worst world, per-world",
         "values and how lines ended. [SOLVER] = the exact forced-lethal solver (sees everything) on the position BEFORE the action.",
         ""]
    events = []
    cur = None
    for i, a in enumerate(rec['actions']):
        if 'reseed' in a:
            g.reseed(int(a['reseed']))
            continue
        b = bv.body(a)
        k = next(iter(b))
        player = b[k].get('player')
        if g.phase in ('main', 'choice') and (g.turn, g.active) != cur:
            cur = (g.turn, g.active)
            L.append(f"=== turn {g.turn}, {'BOT' if g.active == 'b' else 'HUMAN'} to act ===")
            L.extend(bv.state_lines(g))
        line = f"  [{i}] {bv.short_action(g, b)}"
        extra = []
        if k != 'mulligan' and g.phase in ('main', 'choice'):
            v = lethal(g)
            if str(v.get('verdict')) == 'lethal':
                who = 'BOT' if player == 'b' else 'HUMAN'
                tag = f"   [SOLVER: {who} has a forced kill here{' (rng-dependent)' if v.get('rng_dependent') else ''}; line {pv_text(v.get('line') or [])}]"
                line += tag
                events.append((i, g.turn, who))
        if player == 'b' and k != 'mulligan':
            if 'bot_value' in a:
                line += f"   bot_value {a['bot_value']:+.1f}"
            try:
                e = g.bot_action_explain(BOT, rec['seed'] + i)
                e = e if isinstance(e, dict) else json.loads(e)
            except Exception as ex:
                e = None
                extra.append(f"      explain failed: {ex!r}"[:200])
            if e and e.get('path') not in ('single_legal',):
                cands = sorted(e.get('candidates', []), key=lambda c: -(c['root_agg'] if c.get('root_agg') is not None else -1e9))
                same = json.dumps(e.get('chosen'), sort_keys=True) == json.dumps(b, sort_keys=True)
                margin = (cands[0]['root_agg'] - cands[1]['root_agg']) if len(cands) > 1 and cands[1].get('root_agg') is not None else None
                extra.append(f"      explain: path {e.get('path')}, {len(cands)} candidates, nodes {e.get('nodes')}"
                             + (f", top-2 margin {margin:.1f}" if margin is not None else '')
                             + ('' if same else f"; re-run would choose: {bv.short_action(g, e.get('chosen'))}"))
                for c in cands[:3]:
                    ws = [w for w in c.get('worlds', []) if not w.get('skipped')]
                    ends = sorted({w.get('end') for w in ws if w.get('end')})
                    pw = ' '.join(f"{w['clamped']:+.0f}" for w in ws if w.get('clamped') is not None)
                    extra.append(f"        cand {c['root_agg'] if c.get('root_agg') is not None else float('nan'):+7.1f} (worst {c['worst']:+.1f}; worlds {pw}; ends {','.join(ends)}): {bv.short_action(g, c['action'])}")
                if cands:
                    best = max((w for w in cands[0].get('worlds', []) if not w.get('skipped') and w.get('pv')), key=lambda w: w.get('clamped', -1e9), default=None)
                    if best:
                        extra.append(f"        expected line (top candidate, best world): {pv_text(best['pv'])}  [end {best.get('end')}]")
        L.append(line)
        L.extend(extra)
        g.apply(b)
    fp = g.full()['players']
    L.append(f"\nFINAL: winner {'BOT' if g.winner == 'b' else 'HUMAN'}; bot HP {fp['b']['leader_defense']}, human HP {fp['a']['leader_defense']}")
    # summarise solver events per player-turn
    L.append("\nSOLVER SUMMARY (first action index per player-turn where the side to move had a forced kill):")
    seen = {}
    for i, t, who in events:
        seen.setdefault((t, who), i)
    for (t, who), i in sorted(seen.items(), key=lambda kv: kv[1]):
        L.append(f"  turn {t} {who}: forced kill available from [{i}]")
    os.makedirs(OUT, exist_ok=True)
    open(os.path.join(OUT, gid + '.txt'), 'w', encoding='utf-8').write('\n'.join(L) + '\n')
    return gid, first, rec.get('winner'), len(rec['actions']), sorted(seen.items(), key=lambda kv: kv[1])


if __name__ == '__main__':
    import datetime as dt
    gids = []
    for f in sorted(glob.glob(REPO + r'\results\games\*.json'), key=os.path.getmtime):
        r = json.load(open(f, encoding='utf-8'))
        if r.get('final') and r.get('policy') == BOT and str(r.get('finished', '')) >= T0:
            gids.append(r['game_id'])
    print('games:', gids, flush=True)
    os.makedirs(OUT, exist_ok=True)
    import shutil
    shutil.copy2(os.path.join(bv.OUT, 'CARDS.txt'), os.path.join(OUT, 'CARDS.txt'))
    with Pool(min(8, len(gids)), initializer=init) as p:
        for gid, first, winner, n, ev in p.imap_unordered(one, gids):
            print(gid, 'first', first, 'winner', winner, 'actions', n, 'solver kills:', ev, flush=True)

