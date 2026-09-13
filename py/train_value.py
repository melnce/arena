#!/usr/bin/env python3
"""Train a linear or small-MLP leaf value from matchup export shards."""

from __future__ import annotations

import argparse
import importlib.util
import json
import time
from pathlib import Path
from typing import Any

FEATURE_LEN = 545
HIST_WIDTH = 96
IDS_OWN_HAND = 0
IDS_OWN_DECK = 9
IDS_OPP_BOARD = 9 + HIST_WIDTH
IDS_OWN_BOARD = 9 + HIST_WIDTH + 5
IDS_OPP_POOL = 9 + HIST_WIDTH + 2 * 5
OWN_DECK_HIST = 353
OPP_POOL_HIST = 353 + HIST_WIDTH
STD_FLOOR = 1e-3
SCALE = 60.0
BATCH = 1024

ZONES: list[dict[str, Any]] = [
    {"name": "own_hand", "id_offset": IDS_OWN_HAND, "count": 9, "hist_offset": None},
    {
        "name": "own_deck",
        "id_offset": IDS_OWN_DECK,
        "count": HIST_WIDTH,
        "hist_offset": OWN_DECK_HIST,
    },
    {"name": "opp_board", "id_offset": IDS_OPP_BOARD, "count": 5, "hist_offset": None},
    {"name": "own_board", "id_offset": IDS_OWN_BOARD, "count": 5, "hist_offset": None},
    {
        "name": "opp_pool",
        "id_offset": IDS_OPP_POOL,
        "count": HIST_WIDTH,
        "hist_offset": OPP_POOL_HIST,
    },
]


def _load_samples():
    path = Path(__file__).resolve().parent / "samples.py"
    spec = importlib.util.spec_from_file_location("arena_samples", path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _try_torch():
    try:
        import torch

        return torch
    except ImportError:
        return None


def load_dirs(dirs: list[str], max_samples: int | None = None) -> dict[str, Any]:
    import numpy as np

    samples = _load_samples()
    feats: list[Any] = []
    ids: list[Any] = []
    labels: list[Any] = []
    aux: list[Any] = []
    games: list[Any] = []
    offset = 0
    for d in dirs:
        data = samples.load(d)
        gi = data["aux"][:, 0].astype(np.int64)
        if gi.size:
            mapped = gi + offset
            offset = int(mapped.max()) + 1
        else:
            mapped = gi
        feats.append(data["features"])
        ids.append(data["ids"])
        labels.append(data["labels"])
        aux.append(data["aux"])
        games.append(mapped)
    features = np.concatenate(feats, axis=0) if feats else np.zeros((0, FEATURE_LEN), np.float32)
    id_arr = np.concatenate(ids, axis=0) if ids else np.zeros((0, 220), np.uint32)
    lab = np.concatenate(labels, axis=0) if labels else np.zeros((0,), np.float32)
    ax = np.concatenate(aux, axis=0) if aux else np.zeros((0, 10), np.float32)
    game_index = np.concatenate(games, axis=0) if games else np.zeros((0,), np.int64)
    if max_samples is not None and features.shape[0] > max_samples:
        features = features[:max_samples]
        id_arr = id_arr[:max_samples]
        lab = lab[:max_samples]
        ax = ax[:max_samples]
        game_index = game_index[:max_samples]
    return {
        "features": features.astype(np.float32, copy=False),
        "ids": id_arr.astype(np.uint32, copy=False),
        "labels": lab.astype(np.float32, copy=False),
        "aux": ax.astype(np.float32, copy=False),
        "game_index": game_index.astype(np.int64, copy=False),
    }


def split_by_game(
    game_index,
    holdout: float,
    seed: int,
) -> tuple[Any, Any]:
    import numpy as np

    games = np.unique(game_index)
    rng = np.random.RandomState(seed)
    rng.shuffle(games)
    n_hold = int(round(len(games) * holdout))
    hold = set(int(g) for g in games[:n_hold])
    train_m = np.array([int(g) not in hold for g in game_index], dtype=bool)
    hold_m = ~train_m
    return train_m, hold_m


def build_vocab(ids) -> list[int]:
    import numpy as np

    uniq = set(int(x) for x in np.unique(ids).tolist())
    uniq.add(0)
    rest = sorted(i for i in uniq if i != 0)
    return [0] + rest


def id_index_table(vocab: list[int], ids):
    import numpy as np

    v = np.asarray(vocab, dtype=np.int64)
    raw = ids.astype(np.int64, copy=False)
    pos = np.searchsorted(v, raw)
    pos = np.clip(pos, 0, max(len(v) - 1, 0))
    hit = v[pos] == raw
    return np.where(hit, pos, 0).astype(np.int64)


def zone_counts(features, zone: dict[str, Any]):
    import numpy as np

    s = zone["count"]
    if zone["hist_offset"] is None:
        return np.ones((features.shape[0], s), dtype=np.float32)
    off = int(zone["hist_offset"])
    return features[:, off : off + s].astype(np.float32, copy=False)


def predict(spec: dict[str, Any], features, ids):
    """Script-side forward (numpy). Returns `scale * tanh(pre)`."""
    import numpy as np

    mean = np.asarray(spec["feat_mean"], dtype=np.float32)
    std = np.asarray(spec["feat_std"], dtype=np.float32)
    x = (features.astype(np.float32, copy=False) - mean) / std
    vocab = [int(v) for v in spec["vocab"]]
    idx = id_index_table(vocab, ids)
    scale = float(spec.get("scale", SCALE))
    zones = spec["zones"]
    if spec["arch"] == "linear":
        lin = spec["linear"]
        w = np.asarray(lin["w"], dtype=np.float32)
        zone_w = np.asarray(lin["zone_w"], dtype=np.float32)
        b = float(lin["b"])
        extra = np.zeros(x.shape[0], dtype=np.float32)
        for z, zone in enumerate(zones):
            sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
            zidx = idx[:, sl]
            count = zone_counts(features, zone)
            extra += (zone_w[z][zidx] * count).sum(axis=1)
        pre = x @ w + extra + b
    else:
        mlp = spec["mlp"]
        emb = np.asarray(mlp["emb"], dtype=np.float32)
        w1 = np.asarray(mlp["w1"], dtype=np.float32)
        b1 = np.asarray(mlp["b1"], dtype=np.float32)
        w2 = np.asarray(mlp["w2"], dtype=np.float32)
        b2 = float(mlp["b2"])
        parts = [x]
        for zone in zones:
            sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
            zidx = idx[:, sl]
            count = zone_counts(features, zone)[..., None]
            parts.append((emb[zidx] * count).sum(axis=1))
        inp = np.concatenate(parts, axis=1)
        h = np.maximum(inp @ w1.T + b1, 0.0)
        pre = h @ w2 + b2
    return (scale * np.tanh(pre)).astype(np.float32)


def auc_score(scores, labels) -> float:
    import numpy as np

    s = np.asarray(scores, dtype=np.float64)
    y = np.asarray(labels, dtype=np.float64)
    m = y != 0
    s, y = s[m], y[m]
    pos = s[y > 0]
    neg = s[y < 0]
    if pos.size == 0 or neg.size == 0:
        return float("nan")
    if pos.size * neg.size <= 4_000_000:
        diff = pos[:, None] - neg[None, :]
        return float((np.sum(diff > 0) + 0.5 * np.sum(diff == 0)) / (pos.size * neg.size))
    order = np.argsort(s, kind="mergesort")
    ss = s[order]
    ys = y[order]
    ranks = np.arange(1, len(s) + 1, dtype=np.float64)
    i = 0
    n = len(ss)
    while i < n:
        j = i + 1
        while j < n and ss[j] == ss[i]:
            j += 1
        if j - i > 1:
            ranks[i:j] = ranks[i:j].mean()
        i = j
    n_pos = float(pos.size)
    n_neg = float(neg.size)
    u = ranks[ys > 0].sum() - n_pos * (n_pos + 1) / 2.0
    return float(u / (n_pos * n_neg))


def sign_acc(pred, labels) -> float:
    import numpy as np

    m = labels != 0
    if not np.any(m):
        return float("nan")
    return float(np.mean(np.sign(pred[m]) == np.sign(labels[m])))


def mse(pred, labels) -> float:
    import numpy as np

    if pred.size == 0:
        return float("nan")
    return float(np.mean((pred - labels) ** 2))


def label_balance(labels) -> dict[str, int]:
    import numpy as np

    return {
        "+1": int(np.sum(labels > 0)),
        "-1": int(np.sum(labels < 0)),
        "0": int(np.sum(labels == 0)),
    }


TURN_BUCKETS = (("<=3", lambda t: t <= 3), ("4-6", lambda t: (t >= 4) & (t <= 6)), (">=7", lambda t: t >= 7))


def metric_block(pred, labels, turns) -> dict[str, Any]:
    import numpy as np

    out: dict[str, Any] = {
        "overall": {
            "mse": mse(pred, labels),
            "sign_acc": sign_acc(pred, labels),
            "auc": auc_score(pred, labels),
            "rows": int(labels.size),
        }
    }
    for name, pred_fn in TURN_BUCKETS:
        m = pred_fn(turns) if turns.size else np.zeros(0, dtype=bool)
        out[name] = {
            "mse": mse(pred[m], labels[m]),
            "sign_acc": sign_acc(pred[m], labels[m]),
            "auc": auc_score(pred[m], labels[m]),
            "rows": int(np.sum(m)),
        }
    return out


def _zones_json() -> list[dict[str, Any]]:
    return [
        {
            "name": z["name"],
            "id_offset": z["id_offset"],
            "count": z["count"],
            "hist_offset": z["hist_offset"],
        }
        for z in ZONES
    ]


class LinearTorch:
    def __init__(self, torch, n_feat: int, n_vocab: int):
        self.torch = torch
        self.w = torch.nn.Parameter(torch.zeros(n_feat))
        self.zone_w = torch.nn.Parameter(torch.zeros(5, n_vocab))
        self.b = torch.nn.Parameter(torch.zeros(1))

    def parameters(self):
        return [self.w, self.zone_w, self.b]

    def forward(self, x, idx, counts):
        extra = (self.zone_w[0][idx[0]] * counts[0]).sum(dim=1)
        extra = extra + (self.zone_w[1][idx[1]] * counts[1]).sum(dim=1)
        extra = extra + (self.zone_w[2][idx[2]] * counts[2]).sum(dim=1)
        extra = extra + (self.zone_w[3][idx[3]] * counts[3]).sum(dim=1)
        extra = extra + (self.zone_w[4][idx[4]] * counts[4]).sum(dim=1)
        return (x @ self.w + extra + self.b[0]).tanh()


class MlpTorch:
    def __init__(self, torch, n_feat: int, n_vocab: int, hidden: int, emb: int):
        self.torch = torch
        self.emb = torch.nn.Parameter(torch.randn(n_vocab, emb) * 0.02)
        self.w1 = torch.nn.Parameter(torch.randn(hidden, n_feat + 5 * emb) * (1.0 / (n_feat + 5 * emb) ** 0.5))
        self.b1 = torch.nn.Parameter(torch.zeros(hidden))
        self.w2 = torch.nn.Parameter(torch.randn(hidden) * (1.0 / hidden**0.5))
        self.b2 = torch.nn.Parameter(torch.zeros(1))

    def parameters(self):
        return [self.emb, self.w1, self.b1, self.w2, self.b2]

    def forward(self, x, idx, counts):
        parts = [x]
        for z in range(5):
            parts.append((self.emb[idx[z]] * counts[z].unsqueeze(-1)).sum(dim=1))
        inp = self.torch.cat(parts, dim=1)
        h = (inp @ self.w1.t() + self.b1).relu()
        return (h @ self.w2 + self.b2[0]).tanh()


def _pack_zone_tensors(torch, features, idx_all, device):
    idx = []
    counts = []
    feat_t = torch.from_numpy(features).to(device)
    for zone in ZONES:
        sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
        idx.append(torch.from_numpy(idx_all[:, sl]).to(device))
        c = zone_counts(features, zone)
        counts.append(torch.from_numpy(c).to(device))
    return feat_t, idx, counts


def train_torch(
    spec_arch: str,
    x_train,
    idx_train,
    y_train,
    x_hold,
    idx_hold,
    y_hold,
    features_train,
    features_hold,
    n_vocab: int,
    hidden: int,
    emb: int,
    epochs: int,
    l2: float,
    seed: int,
):
    torch = _try_torch()
    assert torch is not None
    torch.manual_seed(seed)
    device = torch.device("cpu")
    n_feat = x_train.shape[1]
    if spec_arch == "linear":
        model = LinearTorch(torch, n_feat, n_vocab)
    else:
        model = MlpTorch(torch, n_feat, n_vocab, hidden, emb)
    opt = torch.optim.Adam(model.parameters(), lr=1e-3, weight_decay=l2)
    y_tr = torch.from_numpy(y_train).to(device)
    _, idx_tr, c_tr = _pack_zone_tensors(torch, features_train, idx_train, device)
    x_tr = torch.from_numpy(x_train).to(device)
    has_hold = y_hold.size > 0
    if has_hold:
        y_ho = torch.from_numpy(y_hold).to(device)
        _, idx_ho, c_ho = _pack_zone_tensors(torch, features_hold, idx_hold, device)
        x_ho = torch.from_numpy(x_hold).to(device)

    best_state = [p.detach().clone() for p in model.parameters()]
    best_mse = float("inf")
    patience = 3
    n = x_train.shape[0]
    rng = torch.Generator()
    rng.manual_seed(seed)
    for _epoch in range(epochs):
        perm = torch.randperm(n, generator=rng)
        for start in range(0, n, BATCH):
            b = perm[start : start + BATCH]
            pred = model.forward(
                x_tr[b],
                [t[b] for t in idx_tr],
                [t[b] for t in c_tr],
            )
            loss = torch.mean((pred - y_tr[b]) ** 2)
            opt.zero_grad()
            loss.backward()
            opt.step()
        if has_hold:
            with torch.no_grad():
                pred_h = model.forward(x_ho, idx_ho, c_ho)
                hold_mse = float(torch.mean((pred_h - y_ho) ** 2).item())
            if hold_mse < best_mse - 1e-12:
                best_mse = hold_mse
                best_state = [p.detach().clone() for p in model.parameters()]
                patience = 3
            else:
                patience -= 1
                if patience <= 0:
                    break
        else:
            best_state = [p.detach().clone() for p in model.parameters()]
    with torch.no_grad():
        for p, b in zip(model.parameters(), best_state):
            p.copy_(b)
    return model


def dump_torch(arch: str, model, mean, std, vocab: list[int], trained_on: dict[str, Any]) -> dict[str, Any]:
    import numpy as np

    def to_list(t):
        return t.detach().cpu().numpy().astype(np.float32).tolist()

    out: dict[str, Any] = {
        "arch": arch,
        "feature_len": FEATURE_LEN,
        "feat_mean": np.asarray(mean, dtype=np.float32).tolist(),
        "feat_std": np.asarray(std, dtype=np.float32).tolist(),
        "vocab": vocab,
        "zones": _zones_json(),
        "scale": SCALE,
        "trained_on": trained_on,
    }
    if arch == "linear":
        out["linear"] = {
            "w": to_list(model.w),
            "zone_w": to_list(model.zone_w),
            "b": float(model.b.detach().cpu().item()),
        }
    else:
        out["mlp"] = {
            "emb": to_list(model.emb),
            "w1": to_list(model.w1),
            "b1": to_list(model.b1),
            "w2": to_list(model.w2),
            "b2": float(model.b2.detach().cpu().item()),
        }
    return out


def train_linear_numpy(
    x_train,
    idx_train,
    y_train,
    x_hold,
    idx_hold,
    y_hold,
    features_train,
    features_hold,
    n_vocab: int,
    epochs: int,
    l2: float,
    seed: int,
):
    """Manual-gradient Adam for the linear model when torch is missing."""
    import numpy as np

    rng = np.random.RandomState(seed)
    w = np.zeros(FEATURE_LEN, dtype=np.float64)
    zone_w = np.zeros((5, n_vocab), dtype=np.float64)
    b = 0.0
    lr = 1e-3
    b1, b2, eps = 0.9, 0.999, 1e-8
    mw = np.zeros_like(w)
    vw = np.zeros_like(w)
    mz = np.zeros_like(zone_w)
    vz = np.zeros_like(zone_w)
    mb = vb = 0.0
    counts_tr = [zone_counts(features_train, z) for z in ZONES]
    counts_ho = [zone_counts(features_hold, z) for z in ZONES] if y_hold.size else []
    idx_tr = [idx_train[:, z["id_offset"] : z["id_offset"] + z["count"]] for z in ZONES]
    idx_ho = (
        [idx_hold[:, z["id_offset"] : z["id_offset"] + z["count"]] for z in ZONES]
        if y_hold.size
        else []
    )

    def forward(x, idxs, counts, w, zone_w, b):
        extra = np.zeros(x.shape[0], dtype=np.float64)
        for z in range(5):
            extra += (zone_w[z][idxs[z]] * counts[z]).sum(axis=1)
        return np.tanh(x @ w + extra + b)

    best = (w.copy(), zone_w.copy(), b)
    best_mse = float("inf")
    patience = 3
    t = 0
    n = x_train.shape[0]
    for _epoch in range(epochs):
        perm = rng.permutation(n)
        for start in range(0, n, BATCH):
            sel = perm[start : start + BATCH]
            xb = x_train[sel].astype(np.float64)
            yb = y_train[sel].astype(np.float64)
            idxs = [i[sel] for i in idx_tr]
            cnts = [c[sel].astype(np.float64) for c in counts_tr]
            pre = xb @ w
            extra = np.zeros(sel.size, dtype=np.float64)
            for z in range(5):
                extra += (zone_w[z][idxs[z]] * cnts[z]).sum(axis=1)
            pre = pre + extra + b
            pred = np.tanh(pre)
            dpred = 2.0 * (pred - yb) / sel.size
            dpre = dpred * (1.0 - pred * pred)
            gw = xb.T @ dpre + l2 * w
            gb = float(dpre.sum() + l2 * b)
            gz = np.zeros_like(zone_w)
            for z in range(5):
                np.add.at(gz[z], idxs[z].reshape(-1), (dpre[:, None] * cnts[z]).reshape(-1))
                gz[z] += l2 * zone_w[z]
            t += 1
            mw[:] = b1 * mw + (1 - b1) * gw
            vw[:] = b2 * vw + (1 - b2) * (gw * gw)
            w -= lr * (mw / (1 - b1**t)) / (np.sqrt(vw / (1 - b2**t)) + eps)
            mz[:] = b1 * mz + (1 - b1) * gz
            vz[:] = b2 * vz + (1 - b2) * (gz * gz)
            zone_w -= lr * (mz / (1 - b1**t)) / (np.sqrt(vz / (1 - b2**t)) + eps)
            mb = b1 * mb + (1 - b1) * gb
            vb = b2 * vb + (1 - b2) * (gb * gb)
            b -= lr * (mb / (1 - b1**t)) / ((vb / (1 - b2**t)) ** 0.5 + eps)
        if y_hold.size:
            pred_h = forward(
                x_hold.astype(np.float64),
                idx_ho,
                [c.astype(np.float64) for c in counts_ho],
                w,
                zone_w,
                b,
            )
            hold_mse = float(np.mean((pred_h - y_hold.astype(np.float64)) ** 2))
            if hold_mse < best_mse - 1e-12:
                best_mse = hold_mse
                best = (w.copy(), zone_w.copy(), b)
                patience = 3
            else:
                patience -= 1
                if patience <= 0:
                    break
        else:
            best = (w.copy(), zone_w.copy(), b)
    w, zone_w, b = best
    return w.astype(np.float32), zone_w.astype(np.float32), float(b)


def format_report(report: dict[str, Any]) -> str:
    lines = [
        f"arch={report['arch']}  train_rows={report['rows_train']} holdout_rows={report['rows_holdout']}",
        f"games train/holdout={report['games_train']}/{report['games_holdout']}",
        f"label balance train={report['label_balance_train']} holdout={report['label_balance_holdout']}",
        f"training time {report['train_seconds']:.2f}s",
    ]
    for who in ("net", "v0"):
        block = report[who]
        lines.append(f"--- {who} ---")
        for key in ("overall", "<=3", "4-6", ">=7"):
            m = block[key]
            lines.append(
                f"  {key:8} rows={m['rows']:6}  mse={m['mse']:.5f}  "
                f"sign_acc={m['sign_acc']:.4f}  auc={m['auc']:.4f}"
            )
    return "\n".join(lines)


def train(args: argparse.Namespace) -> dict[str, Any]:
    import numpy as np

    torch = _try_torch()
    if args.model == "mlp" and torch is None:
        raise SystemExit(
            "torch is required for --model mlp (linear can train with numpy gradients)"
        )
    data = load_dirs(args.data, args.max_samples)
    features = data["features"]
    ids = data["ids"]
    labels = data["labels"]
    aux = data["aux"]
    game_index = data["game_index"]
    if features.shape[0] == 0:
        raise SystemExit("no samples in --data")
    train_m, hold_m = split_by_game(game_index, args.holdout, args.seed)
    if not np.any(train_m):
        train_m = np.ones(features.shape[0], dtype=bool)
        hold_m = np.zeros(features.shape[0], dtype=bool)

    feat_tr, feat_ho = features[train_m], features[hold_m]
    ids_tr, ids_ho = ids[train_m], ids[hold_m]
    y_tr, y_ho = labels[train_m], labels[hold_m]
    turns_tr = aux[train_m, 3]
    turns_ho = aux[hold_m, 3]
    v0_ho = aux[hold_m, 5]
    games_tr = int(np.unique(game_index[train_m]).size)
    games_ho = int(np.unique(game_index[hold_m]).size) if np.any(hold_m) else 0

    mean = feat_tr.mean(axis=0).astype(np.float32)
    std = np.maximum(feat_tr.std(axis=0).astype(np.float32), STD_FLOOR)
    x_tr = (feat_tr - mean) / std
    x_ho = (feat_ho - mean) / std if feat_ho.size else feat_ho.reshape(0, FEATURE_LEN)

    vocab = build_vocab(ids_tr)
    idx_tr = id_index_table(vocab, ids_tr)
    idx_ho = (
        id_index_table(vocab, ids_ho)
        if ids_ho.size
        else np.zeros((0, ids.shape[1]), dtype=np.int64)
    )

    t0 = time.perf_counter()
    if torch is not None:
        model = train_torch(
            args.model,
            x_tr,
            idx_tr,
            y_tr,
            x_ho,
            idx_ho,
            y_ho,
            feat_tr,
            feat_ho if feat_ho.size else feat_tr[:0],
            len(vocab),
            args.hidden,
            args.emb,
            args.epochs,
            args.l2,
            args.seed,
        )
        trained_on_stub: dict[str, Any] = {}
        spec = dump_torch(args.model, model, mean, std, vocab, trained_on_stub)
    else:
        w, zone_w, b = train_linear_numpy(
            x_tr,
            idx_tr,
            y_tr,
            x_ho,
            idx_ho,
            y_ho,
            feat_tr,
            feat_ho if feat_ho.size else feat_tr[:0],
            len(vocab),
            args.epochs,
            args.l2,
            args.seed,
        )
        spec = {
            "arch": "linear",
            "feature_len": FEATURE_LEN,
            "feat_mean": mean.tolist(),
            "feat_std": std.tolist(),
            "vocab": vocab,
            "zones": _zones_json(),
            "scale": SCALE,
            "linear": {"w": w.tolist(), "zone_w": zone_w.tolist(), "b": b},
            "trained_on": {},
        }
    train_s = time.perf_counter() - t0

    pred_ho = (
        predict(spec, feat_ho, ids_ho)
        if feat_ho.shape[0]
        else np.zeros((0,), dtype=np.float32)
    )
    # Training is MSE(tanh, label). `predict` returns scale×tanh so the
    # engine leaf lives on the v0/`wv` scale; report MSE on the tanh.
    scale = float(spec.get("scale", SCALE))
    pred_unit = pred_ho / scale if pred_ho.size else pred_ho
    net_metrics = metric_block(pred_unit, y_ho, turns_ho)
    v0_metrics = metric_block(v0_ho, y_ho, turns_ho)
    report = {
        "arch": spec["arch"],
        "rows_train": int(y_tr.size),
        "rows_holdout": int(y_ho.size),
        "games_train": games_tr,
        "games_holdout": games_ho,
        "label_balance_train": label_balance(y_tr),
        "label_balance_holdout": label_balance(y_ho),
        "train_seconds": train_s,
        "net": net_metrics,
        "v0": v0_metrics,
        "dirs": [str(Path(d)) for d in args.data],
        "holdout": args.holdout,
        "seed": args.seed,
        "epochs": args.epochs,
    }
    spec["trained_on"] = {
        "dirs": report["dirs"],
        "rows": int(features.shape[0]),
        "games": int(np.unique(game_index).size),
        "holdout": {
            "rows": report["rows_holdout"],
            "games": games_ho,
            "net": net_metrics,
            "v0": v0_metrics,
        },
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(spec) + "\n")
    report_path = out.with_suffix(out.suffix + ".report.json")
    if out.suffix == ".json":
        report_path = out.with_name(out.stem + ".report.json")
    report_path.write_text(json.dumps(report, indent=2) + "\n")
    text = format_report(report)
    print(text)
    print(f"wrote {out} and {report_path}")
    return report


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--data", nargs="+", required=True)
    p.add_argument("--model", choices=("linear", "mlp"), required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--holdout", type=float, default=0.1)
    p.add_argument("--epochs", type=int, default=30)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--hidden", type=int, default=128)
    p.add_argument("--emb", type=int, default=16)
    p.add_argument("--l2", type=float, default=1e-4)
    p.add_argument("--max-samples", type=int, default=None)
    args = p.parse_args(argv)
    train(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
