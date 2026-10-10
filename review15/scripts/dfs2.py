"""Fuse-aware within-turn searches for the side to move (full information, the game's own RNG).

Why: during a multi-select Choice (fuse partner selection), the public state hash does not change when a partner is
chosen. A transposition / cycle check keyed on hash() alone therefore prunes every fuse-with-partners line. That is the
bug in engine/src/lethal.rs (pos_key = (hash, rng fingerprint) + path_keys cycle check) and in the earlier analysis DFS
scripts. Here the key also carries the phase and, in a Choice phase, the sorted legal-action list, which changes with
every pick.

  can_win(g, limit)  -> (True/False/None, nodes, line)   None = node limit reached before a verdict
  max_face(g, limit) -> (best damage to the enemy leader this turn, nodes, line, exhaustive?)

one_partner=True (default): in a Choice phase where Confirm is already legal (a fuse with a partner picked), only
Confirm is explored, so every fuse uses exactly one partner. Fusing more partners only removes cards from hand; the
Rune Test Subject fuse hosts (Sephie, Maven Convict: one Test Subject per fuse; Ecstatic Scholar: 'if you've Fused to
this card') do not depend on the partner count. Exhaustive within that restriction.
"""
import json

A = lambda v: v() if callable(v) else v
J = lambda a: json.loads(a) if isinstance(a, str) else a


def key(s, legal=None):
    ph = A(s.phase)
    if 'choice' in str(ph):
        legal = legal if legal is not None else [J(a) for a in s.legal()]
        return (s.hash(), 'choice', tuple(sorted(json.dumps(a, sort_keys=True) for a in legal)))
    return (s.hash(), str(ph))


def leader_hp(s, side):
    return J(s.snapshot())['players'][side]['leader_defense']


def rank(a):
    if 'attack' in a:
        return 0 if a['attack']['target'] == 'leader' else 1
    if 'evolve' in a: return 2
    if 'play' in a: return 3
    if 'fuse' in a: return 5
    return 4


def moves(legal, one_partner):
    """the moves explored (ordered like the engine solver: face attacks, attacks, evolves, plays, other, fuse)"""
    if one_partner and any('confirm' in a for a in legal):
        return [a for a in legal if 'confirm' in a]
    return sorted(legal, key=rank)


def can_win(g, limit=2_000_000, one_partner=True):
    me, t0 = A(g.active), A(g.turn)
    seen = set(); n = [0]; line = []; capped = [False]

    def dfs(s):
        if n[0] >= limit:
            capped[0] = True
            return False
        n[0] += 1
        if A(s.winner) == me:
            return True
        if A(s.winner) is not None or A(s.turn) != t0 or A(s.active) != me:
            return False
        legal = [J(a) for a in s.legal()]
        k = key(s, legal)
        if k in seen:
            return False
        seen.add(k)
        for a in moves(legal, one_partner):
            if 'end_turn' in a:
                continue
            s2 = s.clone()
            try:
                s2.apply(a)
            except Exception:
                continue
            line.append(a)
            if dfs(s2):
                return True
            line.pop()
        return False
    r = dfs(g)
    if r:
        return True, n[0], list(line)
    return (None if capped[0] else False), n[0], []


def max_face(g, limit=2_000_000, one_partner=True):
    me, t0 = A(g.active), A(g.turn)
    opp = 'b' if me == 'a' else 'a'
    hp0 = leader_hp(g, opp)
    seen = set(); n = [0]; best = [0, []]; line = []; capped = [False]

    def dfs(s):
        if n[0] >= limit:
            capped[0] = True
            return
        n[0] += 1
        dmg = hp0 - leader_hp(s, opp)
        if dmg > best[0]:
            best[0], best[1] = dmg, list(line)
        if A(s.winner) is not None or A(s.turn) != t0 or A(s.active) != me:
            return
        legal = [J(a) for a in s.legal()]
        k = key(s, legal)
        if k in seen:
            return
        seen.add(k)
        for a in moves(legal, one_partner):
            if 'end_turn' in a:
                continue
            s2 = s.clone()
            try:
                s2.apply(a)
            except Exception:
                continue
            line.append(a)
            dfs(s2)
            line.pop()
    dfs(g)
    return best[0], n[0], best[1], not capped[0]
