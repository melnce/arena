"""Calibration of the fixed v3 leaf by race situation: mean predicted win chance vs the actual win rate."""
import json, sys, numpy as np
import race_probe as rp, race_stack as rs
model = json.load(open(sys.argv[1])); sh = rp.shards(sys.argv[2:])
pre, R, Y, HP, OA, G = [], [], [], [], [], []
for f, ids, y, g in rp.chunks(sh, 20000):
    pre.append(rs.leaf_pre(model, f, ids)); R.append(rp.race(f)); Y.append(y); HP.append(f[:, rp.ME_HP])
    OA.append(rp.board_attack(f, rp.OPP_BOARD)); G.append(g)
pre = np.concatenate(pre); R = np.concatenate(R); Y = np.concatenate(Y); HP = np.concatenate(HP); OA = np.concatenate(OA)
G = np.concatenate(G)
p = 1 / (1 + np.exp(-2 * pre)); win = (Y + 1) / 2
def row(name, m):
    if m.sum() == 0: return
    gg = np.unique(G[m])
    print(f"{name:34s} rows {int(m.sum()):7d}  games {len(gg):5d}  leaf says {p[m].mean():.3f}  actual {win[m].mean():.3f}  "
          f"gap {win[m].mean() - p[m].mean():+.3f}")
print("situation (the sample's player)          rows     games   leaf's mean win chance vs actual win rate")
row("all", np.ones(len(Y), bool))
row("opp board attack >= my HP (threat)", R[:, 2] == 1)
row("opp board attack in [HP-3, HP-1]", (R[:, 3] == 1) & (R[:, 2] == 0))
row("opp board attack < HP-3", R[:, 3] == 0)
row("my HP <= 5", HP <= 5)
row("my HP 6-10", (HP > 5) & (HP <= 10))
row("my HP > 10", HP > 10)
row("my board attack >= opp HP", R[:, 8] == 1)
for lo, hi in ((-20, -3), (-3, 0), (0, 3), (3, 6), (6, 10), (10, 30)):
    m = (HP - OA >= lo) & (HP - OA < hi)
    row(f"my HP - opp board attack in [{lo},{hi})", m)
# leaf-confident bins within threat
for lo, hi in ((0.0, 0.3), (0.3, 0.5), (0.5, 0.7), (0.7, 1.01)):
    m = (R[:, 2] == 1) & (p >= lo) & (p < hi)
    row(f"threat, leaf says [{lo:.1f},{hi:.1f})", m)
    m = (R[:, 3] == 0) & (p >= lo) & (p < hi)
    row(f"no threat, leaf says [{lo:.1f},{hi:.1f})", m)
