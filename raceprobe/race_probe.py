#!/usr/bin/env python3
"""race probe — do "race" inputs (HP against the board's damage, low-HP bands) predict game results better than the
leaf's current inputs? (the owner's question, 2026-10-03)

The leaf (`h0-linear-v3`) sees each leader's HP as one number and the board slot by slot. A linear model over those
can value HP only at a constant rate, whether the opponent's board threatens lethal or not. The race inputs below are
non-linear functions of the same features (so nothing new is observed, only expressible):

    for me (the sample's player) and, mirrored, for the opponent:
      lo5, lo10      max(0, 5 - hp), max(0, 10 - hp)            HP bands (HP worth more when low)
      threat         1[board attack against me >= hp]           the other side's board alone is lethal-sized
      near           1[board attack against me >= hp - 3]
      margin         clip(hp - board attack against me, -5, 10)
      inter          hp * board attack against me / 20

"board attack against me" = the sum of attack of the other side's followers (encoding-2 board slots, kind 1). Wards,
my blockers and the hands are ignored.

Models (both see exactly the same rows):
  M0  the leaf's information: 567 encoding-2 features (standardised) + card identities per zone (the leaf's five zones:
      own hand, own deck histogram, opp board, own board, opp known pool histogram; counts as in `net.rs`) + intercept
  M1  M0 + the 12 race inputs (standardised)

  M2  control: M0 + 12 label-free pseudo-random columns (must not help)

Primary test: ridge regression on the +1 / -1 result, K folds split by game; every game gets out-of-fold predictions
from both models (clipped to [-1, 1]). The paired per-game difference of squared error M1 - M0 gives the mean and a
game-clustered standard error. Secondary (`--logistic`): logistic regression (Newton) on one split, log-loss and MSE.

Usage (numpy only; the export shards of `arena.matchup(..., export=dir, encoding=2)` / `iterate.py --only data`):
  python race_probe.py --out REPORT.json DIR [DIR ...] [--folds 5] [--seed 1] [--lams 10 100 ...] [--chunk 20000] [--logistic]
  The ridge penalty is chosen as the one with the lowest out-of-fold MSE for M0 (the baseline) and used for both.
"""
from __future__ import annotations

import argparse
import json
import math
import time
from pathlib import Path

import numpy as np

FL = 567
IDS_LEN = 220
ME_HP, OPP_HP = 41, 70
OWN_BOARD, OPP_BOARD, SLOT_W, SLOTS = 153, 253, 20, 5
# the leaf's zones (h0-linear-v3.json "zones"): (name, id_offset, count, hist_offset)
ZONES = [("own_hand", 0, 9, None), ("own_deck", 9, 96, 353), ("opp_board", 105, 5, None),
         ("own_board", 110, 5, None), ("opp_pool", 115, 96, 449)]
RACE_NAMES = [f"{n}_{s}" for s in ("me", "opp") for n in ("lo5", "lo10", "threat", "near", "margin", "inter")]


def shards(dirs):
    out = []
    offset = 0
    for d in dirs:
        d = Path(d)
        meta = json.loads((d / "meta.json").read_text())
        assert int(meta["encoding"]) == 2 and int(meta["feature_len"]) == FL, f"{d}: encoding-2 shards only"
        n = int(meta["samples"])
        ids_len = int(meta["ids_len"])
        n_aux = len(meta["aux_columns"])
        f = np.memmap(d / "features.f32le", dtype="<f4", mode="r", shape=(n, FL))
        ids = np.memmap(d / "ids.u32le", dtype="<u4", mode="r", shape=(n, ids_len))
        lab = np.memmap(d / "labels.f32le", dtype="<f4", mode="r", shape=(n,))
        aux = np.memmap(d / "aux.f32le", dtype="<f4", mode="r", shape=(n, n_aux))
        gi = np.asarray(aux[:, meta["aux_columns"].index("game_index")], dtype=np.int64)
        games = gi + offset
        offset = int(games.max()) + 1 if n else offset
        out.append({"dir": str(d), "n": n, "f": f, "ids": ids, "lab": lab, "games": games})
    return out


def chunks(sh, size):
    for s in sh:
        for a in range(0, s["n"], size):
            b = min(a + size, s["n"])
            yield (np.asarray(s["f"][a:b], dtype=np.float64), np.asarray(s["ids"][a:b]),
                   np.asarray(s["lab"][a:b], dtype=np.float64), s["games"][a:b])


def board_attack(f, base):
    atk = f[:, base:base + SLOTS * SLOT_W:SLOT_W]
    kind = f[:, base + 19:base + SLOTS * SLOT_W:SLOT_W]
    return (atk * (kind == 1)).sum(1)


def race(f):
    me_hp, opp_hp = f[:, ME_HP], f[:, OPP_HP]
    vs_me = board_attack(f, OPP_BOARD)     # the opponent's board, against my HP
    vs_opp = board_attack(f, OWN_BOARD)    # my board, against the opponent's HP
    cols = []
    for hp, atk in ((me_hp, vs_me), (opp_hp, vs_opp)):
        cols += [np.maximum(0, 5 - hp), np.maximum(0, 10 - hp), (atk >= hp).astype(np.float64),
                 (atk >= hp - 3).astype(np.float64), np.clip(hp - atk, -5, 10), hp * atk / 20.0]
    return np.stack(cols, 1)


def identity(f, ids, vocab):
    n, V = f.shape[0], len(vocab)
    out = np.zeros((n, len(ZONES) * V), dtype=np.float64)
    rows = np.arange(n)
    for z, (_, off, cnt, hist) in enumerate(ZONES):
        blk = ids[:, off:off + cnt].astype(np.int64)
        pos = np.searchsorted(vocab, blk)
        pos = np.clip(pos, 0, V - 1)
        idx = np.where(vocab[pos] == blk, pos, 0)
        w = f[:, hist:hist + cnt] if hist is not None else np.ones((n, cnt))
        flat = (rows[:, None] * (len(ZONES) * V) + z * V + idx).ravel()
        out.ravel()[:] += np.bincount(flat, weights=w.ravel(), minlength=n * len(ZONES) * V)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dirs", nargs="+")
    ap.add_argument("--out", required=True)
    ap.add_argument("--folds", type=int, default=5)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--lams", type=float, nargs="+", default=[10.0, 100.0, 1000.0, 10000.0, 100000.0])
    ap.add_argument("--chunk", type=int, default=20000)
    ap.add_argument("--logistic", action="store_true")
    ap.add_argument("--newton", type=int, default=6)
    a = ap.parse_args()
    t0 = time.time()
    sh = shards(a.dirs)
    N = sum(s["n"] for s in sh)

    # pass 1: vocab, folds, standardisation
    vocab = set([0])
    s1 = np.zeros(FL); s2 = np.zeros(FL); r1 = np.zeros(12); r2 = np.zeros(12)
    all_games = np.concatenate([s["games"] for s in sh])
    for f, ids, y, g in chunks(sh, a.chunk):
        vocab.update(np.unique(ids).tolist())
        s1 += f.sum(0); s2 += (f * f).sum(0)
        r = race(f); r1 += r.sum(0); r2 += (r * r).sum(0)
    vocab = np.array(sorted(vocab), dtype=np.int64)
    V = len(vocab)
    mu = s1 / N; sd = np.sqrt(np.maximum(s2 / N - mu * mu, 0)); sd[sd < 1e-9] = 1.0
    rmu = r1 / N; rsd = np.sqrt(np.maximum(r2 / N - rmu * rmu, 0)); rsd[rsd < 1e-9] = 1.0
    ug = np.unique(all_games)
    perm = np.random.RandomState(a.seed).permutation(len(ug))
    fold_by_index = np.empty(len(ug), dtype=np.int64)
    fold_by_index[perm] = np.arange(len(ug)) % a.folds

    def folds_of(g):
        return fold_by_index[np.searchsorted(ug, g)]
    P_dense, P_id, P_race, P_noise = FL, len(ZONES) * V, 12, 12
    P = P_dense + P_id + P_race + P_noise + 1
    i_dense = np.arange(0, P_dense)
    i_id = np.arange(P_dense, P_dense + P_id)
    i_race = np.arange(P_dense + P_id, P_dense + P_id + P_race)
    i_noise = np.arange(P_dense + P_id + P_race, P_dense + P_id + P_race + P_noise)
    i_icpt = P - 1
    S0 = np.concatenate([i_dense, i_id, [i_icpt]])
    S1 = np.concatenate([i_dense, i_id, i_race, [i_icpt]])
    S2 = np.concatenate([i_dense, i_id, i_noise, [i_icpt]])   # control: 12 label-free pseudo-random columns
    proj = np.random.RandomState(7).normal(size=(FL, P_noise)) * 1e3
    print(f"rows {N}, games {len(ug)}, vocab {V}, columns {P} (M0 {len(S0)}, M1 {len(S1)}), pass 1 {time.time() - t0:.0f}s",
          flush=True)

    def design(f, ids):
        X = np.empty((f.shape[0], P))
        X[:, i_dense] = (f - mu) / sd
        X[:, i_id] = identity(f, ids, vocab)
        X[:, i_race] = (race(f) - rmu) / rsd
        X[:, i_noise] = np.sin(f @ proj) * math.sqrt(2.0)
        X[:, i_icpt] = 1.0
        return X

    # pass 2: per-fold Gram
    K = a.folds
    G = np.zeros((K, P, P)); b = np.zeros((K, P)); nk = np.zeros(K)
    for f, ids, y, g in chunks(sh, a.chunk):
        X = design(f, ids)
        fk = folds_of(g)
        for k in range(K):
            m = fk == k
            if m.any():
                Xm = X[m]
                G[k] += Xm.T @ Xm; b[k] += Xm.T @ y[m]; nk[k] += m.sum()
    print(f"pass 2 (Gram) done {time.time() - t0:.0f}s", flush=True)
    Gall, ball = G.sum(0), b.sum(0)

    def solve(Gm, bm, S, lam):
        A = Gm[np.ix_(S, S)].copy()
        pen = np.full(len(S), lam); pen[S == i_icpt] = 0.0
        A[np.diag_indices_from(A)] += pen
        return np.linalg.solve(A, bm[S])

    def full(S, w):
        out = np.zeros(P); out[S] = w
        return out

    lams = a.lams
    # full-length weight vectors (zeros outside a model's columns), so predictions need no column copies
    W0 = {l: [full(S0, solve(Gall - G[k], ball - b[k], S0, l)) for k in range(K)] for l in lams}
    W1 = {l: [full(S1, solve(Gall - G[k], ball - b[k], S1, l)) for k in range(K)] for l in lams}
    W2 = {l: [full(S2, solve(Gall - G[k], ball - b[k], S2, l)) for k in range(K)] for l in lams}

    # pass 3: out-of-fold predictions for every penalty, paired by game
    acc = {l: {"gD": np.zeros(len(ug)), "gD2": np.zeros(len(ug)), "tot": np.zeros(3), "sub": {}} for l in lams}
    gN = np.zeros(len(ug)); cnt = 0
    for f, ids, y, g in chunks(sh, a.chunk):
        X = design(f, ids)
        fk = folds_of(g)
        gix = np.searchsorted(ug, g)
        gN += np.bincount(gix, minlength=len(ug)); cnt += len(y)
        r = race(f)
        groups = {"threat_me": r[:, 2] == 1, "near_me_only": (r[:, 3] == 1) & (r[:, 2] == 0),
                  "no_threat_me": r[:, 3] == 0, "threat_opp": r[:, 8] == 1, "hp_me_le5": f[:, ME_HP] <= 5,
                  "hp_me_6_10": (f[:, ME_HP] > 5) & (f[:, ME_HP] <= 10), "hp_me_gt10": f[:, ME_HP] > 10}
        for l in lams:
            p0 = np.empty(len(y)); p1 = np.empty(len(y)); p2 = np.empty(len(y))
            for k in range(K):
                m = fk == k
                if m.any():
                    Xm = X[m]
                    p0[m] = Xm @ W0[l][k]; p1[m] = Xm @ W1[l][k]; p2[m] = Xm @ W2[l][k]
            e0 = (np.clip(p0, -1, 1) - y) ** 2; e1 = (np.clip(p1, -1, 1) - y) ** 2; e2 = (np.clip(p2, -1, 1) - y) ** 2
            A_ = acc[l]
            A_["tot"] += [e0.sum(), e1.sum(), e2.sum()]
            A_["gD"] += np.bincount(gix, weights=e1 - e0, minlength=len(ug))
            A_["gD2"] += np.bincount(gix, weights=e2 - e0, minlength=len(ug))
            for name, m in groups.items():
                s_ = A_["sub"].setdefault(name, [0.0, 0.0, 0])
                s_[0] += e0[m].sum(); s_[1] += e1[m].sum(); s_[2] += int(m.sum())
    keep = gN > 0
    rep = {"rows": int(N), "games": int(len(ug)), "vocab": int(V), "folds": K, "by_lam": {}}
    for l in lams:
        A_ = acc[l]
        D, ng = A_["gD"][keep], gN[keep]
        mean_d = D.sum() / cnt
        se = math.sqrt(len(D) / (len(D) - 1) * ((D - ng * mean_d) ** 2).sum()) / cnt
        D2 = A_["gD2"][keep]
        mean_d2 = D2.sum() / cnt
        se2 = math.sqrt(len(D2) / (len(D2) - 1) * ((D2 - ng * mean_d2) ** 2).sum()) / cnt
        rep["by_lam"][str(l)] = {"mse_M0": A_["tot"][0] / cnt, "mse_M1": A_["tot"][1] / cnt, "diff_M1_minus_M0": mean_d,
                                 "se_game_clustered": se, "z": mean_d / se if se > 0 else float("nan"),
                                 "mse_M2_noise": A_["tot"][2] / cnt, "diff_M2_minus_M0": mean_d2,
                                 "z_noise": mean_d2 / se2 if se2 > 0 else float("nan"),
                                 "subsets": {k: {"rows": v[2], "mse_M0": v[0] / max(v[2], 1), "mse_M1": v[1] / max(v[2], 1)}
                                             for k, v in A_["sub"].items()}}
    best = min(lams, key=lambda l: rep["by_lam"][str(l)]["mse_M0"])
    rep["lam_best_for_M0"] = best
    base = float(np.mean([1.0]))  # MSE of always predicting 0 is 1.0 for +1 / -1 results
    print(f"MSE of predicting 0: {base:.3f}")
    for l in lams:
        r_ = rep["by_lam"][str(l)]
        print(f"lam {l:>8}: out-of-fold MSE M0 {r_['mse_M0']:.5f}  M1 {r_['mse_M1']:.5f}  "
              f"M1-M0 {r_['diff_M1_minus_M0']:+.5f} (se {r_['se_game_clustered']:.5f}, z {r_['z']:+.2f})  "
              f"control M2-M0 {r_['diff_M2_minus_M0']:+.5f} (z {r_['z_noise']:+.2f})"
              f"{'   <- best M0' if l == best else ''}", flush=True)
    for k, v in rep["by_lam"][str(best)]["subsets"].items():
        print(f"  [lam {best}] {k:14s} rows {v['rows']:8d}  mse M0 {v['mse_M0']:.5f}  M1 {v['mse_M1']:.5f}  "
              f"diff {v['mse_M1'] - v['mse_M0']:+.5f}")
    a.lam = best

    # full-data M1 fit: race coefficients (per raw unit) and the marginal value of 1 HP
    w1 = solve(Gall, ball, S1, a.lam)
    pos = {c: j for j, c in enumerate(S1)}
    race_w = {RACE_NAMES[j]: w1[pos[i_race[j]]] / rsd[j] for j in range(12)}
    hp_w = w1[pos[i_dense[ME_HP]]] / sd[ME_HP]
    w0 = solve(Gall, ball, S0, a.lam); pos0 = {c: j for j, c in enumerate(S0)}
    hp_w0 = w0[pos0[i_dense[ME_HP]]] / sd[ME_HP]
    rep["race_coef_raw"] = race_w
    rep["hp_coef_raw_M1"] = hp_w
    rep["hp_coef_raw_M0"] = hp_w0
    print(f"value of +1 own HP in M0 (linear, everywhere): {hp_w0:+.4f} (result scale, -1..+1)")
    print("race coefficients (per raw unit):", {k: round(v, 4) for k, v in race_w.items()})

    def val(hp, atk):
        r = np.array([max(0, 5 - hp), max(0, 10 - hp), float(atk >= hp), float(atk >= hp - 3),
                      min(max(hp - atk, -5), 10), hp * atk / 20.0])
        return hp_w * hp + sum(race_w[RACE_NAMES[j]] * r[j] for j in range(6))
    table = {}
    for atk in (0, 3, 6, 9):
        table[atk] = {hp: round(val(hp + 1, atk) - val(hp, atk), 4) for hp in (2, 4, 6, 8, 10, 14, 18)}
        print(f"  M1: value of +1 own HP when the opponent's board has {atk} attack:", table[atk])
    rep["marginal_hp_M1"] = table

    if a.logistic:
        test_k = 0
        Ws = {}
        for name, S in (("M0", S0), ("M1", S1)):
            w = np.zeros(len(S))
            for it in range(a.newton):
                H = np.zeros((len(S), len(S))); gr = np.zeros(len(S))
                for f, ids, y, g in chunks(sh, a.chunk):
                    fk = folds_of(g); m = fk != test_k
                    if not m.any():
                        continue
                    X = design(f[m], ids[m])[:, S]; yy = (y[m] + 1) / 2
                    p = 1 / (1 + np.exp(-np.clip(X @ w, -30, 30)))
                    H += (X * (p * (1 - p))[:, None]).T @ X; gr += X.T @ (yy - p)
                pen = np.full(len(S), a.lam); pen[S == i_icpt] = 0.0
                H[np.diag_indices_from(H)] += pen; gr -= pen * w
                w += np.linalg.solve(H, gr)
            Ws[name] = (S, w)
        ll = {"M0": 0.0, "M1": 0.0}; ms = {"M0": 0.0, "M1": 0.0}; n_te = 0; gl = {}
        for f, ids, y, g in chunks(sh, a.chunk):
            fk = folds_of(g); m = fk == test_k
            if not m.any():
                continue
            X = design(f[m], ids[m]); yy = (y[m] + 1) / 2; n_te += int(m.sum())
            per = {}
            for name, (S, w) in Ws.items():
                p = np.clip(1 / (1 + np.exp(-np.clip(X[:, S] @ w, -30, 30))), 1e-6, 1 - 1e-6)
                l = -(yy * np.log(p) + (1 - yy) * np.log(1 - p))
                ll[name] += l.sum(); ms[name] += ((2 * p - 1 - y[m]) ** 2).sum(); per[name] = l
            for gg, dd in zip(g[m].tolist(), (per["M1"] - per["M0"]).tolist()):
                s = gl.setdefault(gg, [0.0, 0]); s[0] += dd; s[1] += 1
        D = np.array([v[0] for v in gl.values()]); ng = np.array([v[1] for v in gl.values()])
        md = D.sum() / n_te
        se_l = math.sqrt(len(D) / (len(D) - 1) * ((D - ng * md) ** 2).sum()) / n_te
        rep["logistic"] = {"test_rows": n_te, "test_games": len(D), "logloss_M0": ll["M0"] / n_te,
                           "logloss_M1": ll["M1"] / n_te, "mse_M0": ms["M0"] / n_te, "mse_M1": ms["M1"] / n_te,
                           "diff_logloss": md, "se": se_l, "z": md / se_l if se_l > 0 else float("nan")}
        print("logistic:", json.dumps(rep["logistic"], indent=1))
    rep["seconds"] = time.time() - t0
    Path(a.out).write_text(json.dumps(rep, indent=1))
    print(f"wrote {a.out} ({rep['seconds']:.0f}s)")


if __name__ == "__main__":
    main()
