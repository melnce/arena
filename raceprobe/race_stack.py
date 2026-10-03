#!/usr/bin/env python3
"""race stack — given the actual leaf's value, do the race inputs add anything? (pilot companion to race_probe.py)

race_probe.py refits a whole leaf on the data, which overfits on a small pilot (≈ 1 800 columns). This test keeps the
real leaf fixed (`h0-linear-v3`, trained on 4.2 M other rows) and fits only a handful of numbers on top of it:

  L0  the leaf's pre-activation + intercept
  L1  L0 + the 12 race inputs (race_probe.race)
  L2  L0 + 12 label-free pseudo-random columns (control: must not help)

5 folds by game, out-of-fold predictions; logistic regression (Newton) on the 0/1 result (the leaf's tanh is a
logistic in disguise: tanh(x) = 2 sigma(2x) - 1), log-loss and MSE of 2p - 1; paired per game, game-clustered z.

Usage: python race_stack.py LEAF.json DIR [DIR ...] [--out REPORT.json] [--folds 5] [--seed 1] [--lam 1.0]
"""
from __future__ import annotations

import argparse
import json
import math

import numpy as np

import race_probe as rp


def leaf_pre(model, f, ids):
    """The leaf's pre-activation, as net.rs computes it (linear arch)."""
    mu = np.asarray(model["feat_mean"]); sd = np.asarray(model["feat_std"])
    sd = np.where(sd > 0, sd, 1.0)
    lin = model["linear"]
    pre = lin["b"] + ((f - mu) / sd) @ np.asarray(lin["w"])
    vocab = np.asarray(model["vocab"], dtype=np.int64)
    for z, zone in enumerate(model["zones"]):
        off, cnt, hist = zone["id_offset"], zone["count"], zone["hist_offset"]
        blk = ids[:, off:off + cnt].astype(np.int64)
        pos = np.clip(np.searchsorted(vocab, blk), 0, len(vocab) - 1)
        idx = np.where(vocab[pos] == blk, pos, 0)
        table = np.asarray(lin["zone_w"][z])
        cnts = f[:, hist:hist + cnt] if hist is not None else np.ones(blk.shape)
        pre = pre + (table[idx] * cnts).sum(1)
    return pre


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("leaf")
    ap.add_argument("dirs", nargs="+")
    ap.add_argument("--folds", type=int, default=5)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--lam", type=float, default=1.0)
    ap.add_argument("--newton", type=int, default=12)
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    model = json.load(open(a.leaf))
    sh = rp.shards(a.dirs)
    pre, R, Y, Gm, HPme = [], [], [], [], []
    proj = np.random.RandomState(7).normal(size=(rp.FL, 12)) * 1e3
    NZ = []
    for f, ids, y, g in rp.chunks(sh, 20000):
        pre.append(leaf_pre(model, f, ids)); R.append(rp.race(f)); Y.append(y); Gm.append(g)
        HPme.append(f[:, rp.ME_HP]); NZ.append(np.sin(f @ proj) * math.sqrt(2.0))
    pre = np.concatenate(pre); R = np.concatenate(R); Y = np.concatenate(Y); Gm = np.concatenate(Gm)
    HPme = np.concatenate(HPme); NZ = np.concatenate(NZ)
    N = len(Y)
    Rs = (R - R.mean(0)) / np.where(R.std(0) > 0, R.std(0), 1)
    y01 = (Y + 1) / 2
    ug = np.unique(Gm)
    perm = np.random.RandomState(a.seed).permutation(len(ug))
    fold_by_index = np.empty(len(ug), dtype=np.int64); fold_by_index[perm] = np.arange(len(ug)) % a.folds
    fk = fold_by_index[np.searchsorted(ug, Gm)]
    gix = np.searchsorted(ug, Gm)
    # the fixed leaf alone, as the engine values it: 60 tanh(pre) -> as a probability, sigma(2 pre)
    p_leaf = 1 / (1 + np.exp(-2 * pre))
    designs = {"L0": np.column_stack([pre, np.ones(N)]),
               "L1": np.column_stack([pre, Rs, np.ones(N)]),
               "L2": np.column_stack([pre, NZ, np.ones(N)])}

    def fit(X, yy):
        w = np.zeros(X.shape[1])
        for _ in range(a.newton):
            p = 1 / (1 + np.exp(-np.clip(X @ w, -30, 30)))
            H = (X * (p * (1 - p))[:, None]).T @ X
            H[np.diag_indices_from(H)] += a.lam
            H[-1, -1] -= a.lam
            g = X.T @ (yy - p) - a.lam * np.r_[w[:-1], 0.0]
            w = w + np.linalg.solve(H, g)
        return w

    P = {k: np.empty(N) for k in designs}
    for k in range(a.folds):
        tr, te = fk != k, fk == k
        for name, X in designs.items():
            w = fit(X[tr], y01[tr])
            P[name][te] = 1 / (1 + np.exp(-np.clip(X[te] @ w, -30, 30)))
    eps = 1e-6

    def ll(p):
        p = np.clip(p, eps, 1 - eps)
        return -(y01 * np.log(p) + (1 - y01) * np.log(1 - p))

    def paired(d):
        D = np.bincount(gix, weights=d, minlength=len(ug)); n = np.bincount(gix, minlength=len(ug))
        m = d.mean()
        se = math.sqrt(len(ug) / (len(ug) - 1) * ((D - n * m) ** 2).sum()) / N
        return m, se, m / se
    out = {"rows": int(N), "games": int(len(ug))}
    base_ll = ll(P["L0"]); base_se = (2 * P["L0"] - 1 - Y) ** 2
    out["leaf_alone"] = {"logloss": float(ll(p_leaf).mean()), "mse": float(((2 * p_leaf - 1 - Y) ** 2).mean())}
    for name in ("L0", "L1", "L2"):
        out[name] = {"logloss": float(ll(P[name]).mean()), "mse": float(((2 * P[name] - 1 - Y) ** 2).mean())}
    for name in ("L1", "L2"):
        m, se, z = paired(ll(P[name]) - base_ll)
        m2, se2, z2 = paired((2 * P[name] - 1 - Y) ** 2 - base_se)
        out[name + "_vs_L0"] = {"d_logloss": m, "se": se, "z": z, "d_mse": m2, "z_mse": z2}
    subs = {"threat_me": R[:, 2] == 1, "near_me_only": (R[:, 3] == 1) & (R[:, 2] == 0), "no_threat_me": R[:, 3] == 0,
            "threat_opp": R[:, 8] == 1, "hp_me_le5": HPme <= 5, "hp_me_6_10": (HPme > 5) & (HPme <= 10),
            "hp_me_gt10": HPme > 10}
    out["subsets"] = {}
    for sname, msk in subs.items():
        out["subsets"][sname] = {"rows": int(msk.sum()), "logloss_L0": float(base_ll[msk].mean()),
                                 "logloss_L1": float(ll(P["L1"])[msk].mean()),
                                 "mse_L0": float(base_se[msk].mean()),
                                 "mse_L1": float(((2 * P["L1"] - 1 - Y) ** 2)[msk].mean())}
    # coefficients on all data
    wl1 = fit(designs["L1"], y01)
    out["L1_coef"] = {"leaf_pre": float(wl1[0]), **{rp.RACE_NAMES[j]: float(wl1[1 + j] / (R[:, j].std() or 1))
                                                    for j in range(12)}, "intercept": float(wl1[-1])}
    print(json.dumps(out, indent=1))
    if a.out:
        with open(a.out, "w", encoding="utf-8") as fo:
            json.dump(out, fo, indent=1)


if __name__ == "__main__":
    main()
