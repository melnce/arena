#!/usr/bin/env python3
"""Train a linear or small-MLP leaf value from matchup export shards."""

from __future__ import annotations

import argparse
import importlib.util
import json
import time
from pathlib import Path
from typing import Any

FEATURE_LEN_V1 = 545
FEATURE_LEN_V2 = 567
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
LBFGS_CHUNK = 8192
LBFGS_GRAD_TOL = 1e-7
LBFGS_ITERS_DEFAULT = 500

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


def _index_dtype(n_vocab: int):
    import numpy as np

    return np.uint16 if n_vocab <= 65535 else np.uint32


def _chunk_mean_std(features, row_idx, std_floor: float, chunk: int = 2048):
    """Mean/std over training rows without materialising a train-only copy."""
    import numpy as np

    n_feat = features.shape[1]
    floor = float(std_floor)
    if row_idx.size == 0:
        return np.zeros(n_feat, np.float32), np.full(n_feat, floor, np.float32)
    sum_v = np.zeros(n_feat, np.float64)
    sum_sq = np.zeros(n_feat, np.float64)
    n = int(row_idx.size)
    for start in range(0, n, chunk):
        sel = row_idx[start : start + chunk]
        block = features[sel]
        sum_v += block.sum(axis=0, dtype=np.float64)
        sum_sq += np.multiply(block, block, dtype=np.float32).sum(axis=0, dtype=np.float64)
    mean = (sum_v / n).astype(np.float32)
    var = np.maximum(sum_sq / n - mean.astype(np.float64) ** 2, 0.0)
    std = np.maximum(np.sqrt(var).astype(np.float32), floor)
    return mean, std


def _standardize_rows(features, row_idx, mean, std, out=None):
    import numpy as np

    x = (features[row_idx] - mean) / std
    if out is None:
        return x.astype(np.float32, copy=False)
    n = row_idx.size
    out[:n] = x
    return out[:n]


def _read_shard_meta(path: Path) -> dict[str, Any]:
    return json.loads((path / "meta.json").read_text())


def _read_into(path: Path, dest) -> None:
    with open(path, "rb") as f:
        f.readinto(memoryview(dest).cast("B"))


def _mmap_shard(path: Path, meta: dict[str, Any], take: int | None = None):
    import numpy as np

    fl = int(meta["feature_len"])
    ids_len = int(meta["ids_len"])
    n_aux = len(meta["aux_columns"])
    n = take if take is not None else int(meta["samples"])
    features = np.memmap(path / "features.f32le", dtype="<f4", mode="r", shape=(n, fl))
    ids = np.memmap(path / "ids.u32le", dtype="<u4", mode="r", shape=(n, ids_len))
    labels = np.memmap(path / "labels.f32le", dtype="<f4", mode="r", shape=(n,))
    aux = np.memmap(path / "aux.f32le", dtype="<f4", mode="r", shape=(n, n_aux))
    return features, ids, labels, aux


def load_dirs(dirs: list[str], max_samples: int | None = None) -> dict[str, Any]:
    import numpy as np

    metas: list[tuple[Path, dict[str, Any], int]] = []
    encoding: int | None = None
    feature_len: int | None = None
    ids_len = 220
    total = 0
    has_search_v = False
    for d in dirs:
        path = Path(d)
        meta = _read_shard_meta(path)
        enc = int(meta.get("encoding", 1))
        fl = int(meta["feature_len"])
        if encoding is None:
            encoding = enc
            feature_len = fl
            ids_len = int(meta["ids_len"])
        elif enc != encoding or fl != feature_len:
            raise SystemExit(
                f"mixed encoding versions in --data ({encoding} vs {enc} or "
                f"feature_len {feature_len} vs {fl})"
            )
        n = int(meta["samples"])
        metas.append((path, meta, n))
        total += n
        if "search_v" in meta["aux_columns"]:
            has_search_v = True
    if max_samples is not None:
        total = min(total, max_samples)
    fl = feature_len if feature_len is not None else FEATURE_LEN_V1
    enc = encoding if encoding is not None else 1
    aux_width = max(len(m[1]["aux_columns"]) for m in metas) if metas else 10

    if len(metas) == 1:
        path, meta, n = metas[0]
        take = total
        features, id_arr, lab, ax = _mmap_shard(path, meta, take)
        game_index = ax[:, 0]
        cols = list(meta["aux_columns"])
        if "search_v" in cols:
            search_v = ax[:, cols.index("search_v")]
        else:
            search_v = np.full((total,), np.nan, dtype=np.float32)
    else:
        features = np.empty((total, fl), dtype=np.float32)
        id_arr = np.empty((total, ids_len), dtype=np.uint32)
        lab = np.empty((total,), dtype=np.float32)
        ax = np.full((total, aux_width), np.nan, dtype=np.float32)
        game_index = np.empty((total,), dtype=np.int64)
        search_v = np.full((total,), np.nan, dtype=np.float32)
        offset = 0
        pos = 0
        for path, meta, n in metas:
            take = n
            if max_samples is not None and pos + take > total:
                take = total - pos
            if take <= 0:
                break
            n_aux = len(meta["aux_columns"])
            _read_into(path / "features.f32le", features[pos : pos + take])
            _read_into(path / "ids.u32le", id_arr[pos : pos + take])
            _read_into(path / "labels.f32le", lab[pos : pos + take])
            aux_block = ax[pos : pos + take, :n_aux]
            _read_into(path / "aux.f32le", aux_block)
            gi = aux_block[:, 0].astype(np.int64, copy=False)
            if gi.size:
                mapped = gi + offset
                offset = int(mapped.max()) + 1
            else:
                mapped = gi
            game_index[pos : pos + take] = mapped
            cols = list(meta["aux_columns"])
            if "search_v" in cols:
                search_v[pos : pos + take] = aux_block[:, cols.index("search_v")].astype(
                    np.float32, copy=False
                )
            pos += take
    return {
        "features": features,
        "ids": id_arr,
        "labels": lab,
        "aux": ax,
        "game_index": game_index,
        "search_v": search_v,
        "has_search_v": has_search_v,
        "encoding": enc,
        "feature_len": fl,
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
    hold_games = np.asarray(games[:n_hold], dtype=np.float64)
    gi = np.asarray(game_index, dtype=np.float64)
    is_hold = np.isin(gi, hold_games)
    train_idx = np.flatnonzero(~is_hold).astype(np.int32, copy=False)
    hold_idx = np.flatnonzero(is_hold).astype(np.int32, copy=False)
    return train_idx, hold_idx


def build_vocab(ids, row_idx, chunk: int = 8192) -> list[int]:
    import numpy as np

    uniq: set[int] = {0}
    for start in range(0, row_idx.size, chunk):
        sel = row_idx[start : start + chunk]
        uniq.update(int(x) for x in np.unique(ids[sel]).tolist())
    rest = sorted(i for i in uniq if i != 0)
    return [0] + rest


def id_index_table(vocab: list[int], ids, dtype=None):
    import numpy as np

    if dtype is None:
        dtype = _index_dtype(len(vocab))
    v = np.asarray(vocab, dtype=np.int64)
    raw = ids.astype(np.int64, copy=False)
    pos = np.searchsorted(v, raw)
    pos = np.clip(pos, 0, max(len(v) - 1, 0))
    hit = v[pos] == raw
    return np.where(hit, pos, 0).astype(dtype, copy=False)


def zone_counts(features, row_idx, zone: dict[str, Any]):
    import numpy as np

    s = zone["count"]
    if zone["hist_offset"] is None:
        return np.ones((row_idx.size, s), dtype=np.float32)
    off = int(zone["hist_offset"])
    return features[row_idx, off : off + s].astype(np.float32, copy=False)


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
        row_idx = np.arange(features.shape[0], dtype=np.int64)
        for z, zone in enumerate(zones):
            sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
            zidx = idx[:, sl]
            count = zone_counts(features, row_idx, zone)
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
        row_idx = np.arange(features.shape[0], dtype=np.int64)
        for zone in zones:
            sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
            zidx = idx[:, sl]
            count = zone_counts(features, row_idx, zone)[..., None]
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

    def l2_penalty(self, l2: float):
        return 0.5 * l2 * (
            self.w.pow(2).sum() + self.zone_w.pow(2).sum() + self.b.pow(2).sum()
        )


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


def _batch_zone_tensors(torch, features, ids, row_idx, vocab, device):
    idx_np = id_index_table(vocab, ids[row_idx])
    idx = []
    counts = []
    for zone in ZONES:
        sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
        idx.append(torch.from_numpy(idx_np[:, sl].astype("int64", copy=False)).to(device))
        c = zone_counts(features, row_idx, zone)
        counts.append(torch.from_numpy(c).to(device))
    return idx, counts


def train_torch(
    spec_arch: str,
    features,
    ids,
    train_idx,
    hold_idx,
    y_all,
    mean,
    std,
    vocab: list[int],
    hidden: int,
    emb: int,
    epochs: int,
    l2: float,
    seed: int,
):
    import numpy as np

    torch = _try_torch()
    assert torch is not None
    torch.manual_seed(seed)
    device = torch.device("cpu")
    n_feat = features.shape[1]
    n_vocab = len(vocab)
    if spec_arch == "linear":
        model = LinearTorch(torch, n_feat, n_vocab)
    else:
        model = MlpTorch(torch, n_feat, n_vocab, hidden, emb)
    opt = torch.optim.Adam(model.parameters(), lr=1e-3, weight_decay=l2)
    y_tr_np = y_all[train_idx]
    has_hold = hold_idx.size > 0
    y_ho_np = y_all[hold_idx] if has_hold else None

    mean_t = torch.from_numpy(mean).to(device)
    std_t = torch.from_numpy(std).to(device)

    best_state = [p.detach().clone() for p in model.parameters()]
    best_mse = float("inf")
    best_epoch = 0
    patience = 3
    n = train_idx.size
    rng = torch.Generator()
    rng.manual_seed(seed)
    x_buf = np.empty((BATCH, n_feat), dtype=np.float32)
    epochs_run = 0
    for epoch in range(epochs):
        epochs_run = epoch + 1
        perm = torch.randperm(n, generator=rng)
        for start in range(0, n, BATCH):
            b = perm[start : start + BATCH].cpu().numpy()
            row_idx = train_idx[b]
            xb = torch.from_numpy(
                _standardize_rows(features, row_idx, mean, std, x_buf)
            ).to(device)
            idx_b, c_b = _batch_zone_tensors(torch, features, ids, row_idx, vocab, device)
            pred = model.forward(xb, idx_b, c_b)
            yb = torch.from_numpy(y_tr_np[b]).to(device)
            loss = torch.mean((pred - yb) ** 2)
            opt.zero_grad(set_to_none=True)
            loss.backward()
            opt.step()
        if has_hold:
            with torch.no_grad():
                hold_sq = 0.0
                for hstart in range(0, hold_idx.size, BATCH):
                    hrows = hold_idx[hstart : hstart + BATCH]
                    xb = torch.from_numpy(
                        _standardize_rows(features, hrows, mean, std, x_buf)
                    ).to(device)
                    idx_b, c_b = _batch_zone_tensors(torch, features, ids, hrows, vocab, device)
                    pred_h = model.forward(xb, idx_b, c_b)
                    yhb = torch.from_numpy(y_ho_np[hstart : hstart + hrows.size]).to(device)
                    hold_sq += float(torch.sum((pred_h - yhb) ** 2).item())
                hold_mse = hold_sq / hold_idx.size
            if hold_mse < best_mse - 1e-12:
                best_mse = hold_mse
                best_state = [p.detach().clone() for p in model.parameters()]
                best_epoch = epochs_run
                patience = 3
            else:
                patience -= 1
                if patience <= 0:
                    break
        else:
            best_state = [p.detach().clone() for p in model.parameters()]
            best_epoch = epochs_run
    with torch.no_grad():
        for p, b in zip(model.parameters(), best_state):
            p.copy_(b)
    return model, {"epochs_run": epochs_run, "best_epoch": best_epoch}


def train_lbfgs(
    features,
    ids,
    train_idx,
    y_all,
    mean,
    std,
    vocab: list[int],
    l2: float,
    max_iter: int,
    grad_tol: float,
):
    import numpy as np

    torch = _try_torch()
    assert torch is not None
    device = torch.device("cpu")
    n_feat = features.shape[1]
    n_vocab = len(vocab)
    model = LinearTorch(torch, n_feat, n_vocab)
    y_tr_np = y_all[train_idx].astype(np.float32, copy=False)
    n = train_idx.size
    function_evals = 0
    final_loss = float("inf")
    grad_max = float("inf")
    stopped = "iterations"

    def objective_and_grad():
        nonlocal function_evals, final_loss, grad_max
        function_evals += 1
        model.w.grad = None
        model.zone_w.grad = None
        model.b.grad = None
        total = 0.0
        for start in range(0, n, LBFGS_CHUNK):
            end = min(start + LBFGS_CHUNK, n)
            row_idx = train_idx[start:end]
            xb = torch.from_numpy(_standardize_rows(features, row_idx, mean, std)).to(device)
            idx_b, c_b = _batch_zone_tensors(torch, features, ids, row_idx, vocab, device)
            yb = torch.from_numpy(y_tr_np[start:end]).to(device)
            pred = model.forward(xb, idx_b, c_b)
            chunk_loss = torch.sum((pred - yb) ** 2) / n
            total += float(chunk_loss.item())
            chunk_loss.backward()
        if l2 > 0:
            pen = model.l2_penalty(l2)
            total += float(pen.item())
            pen.backward()
        final_loss = total
        gmax = 0.0
        for p in model.parameters():
            if p.grad is not None:
                gmax = max(gmax, float(p.grad.abs().max().item()))
        grad_max = gmax
        return torch.tensor(final_loss, device=device)

    opt = torch.optim.LBFGS(
        model.parameters(),
        max_iter=1,
        line_search_fn="strong_wolfe",
        tolerance_grad=0.0,
        tolerance_change=0.0,
    )
    iterations = 0
    for it in range(max_iter):
        iterations = it + 1

        def closure():
            opt.zero_grad()
            return objective_and_grad()

        opt.step(closure)
        if grad_max <= grad_tol:
            stopped = "tolerance"
            break

    return model, {
        "optimizer": "lbfgs",
        "iterations": iterations,
        "function_evals": function_evals,
        "final_loss": final_loss,
        "grad_max": grad_max,
        "stopped": stopped,
    }


def dump_torch(
    arch: str,
    model,
    mean,
    std,
    vocab: list[int],
    trained_on: dict[str, Any],
    feature_len: int,
    encoding: int,
) -> dict[str, Any]:
    import numpy as np

    def to_list(t):
        return t.detach().cpu().numpy().astype(np.float32).tolist()

    out: dict[str, Any] = {
        "arch": arch,
        "feature_len": feature_len,
        "encoding": encoding,
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
    features,
    ids,
    train_idx,
    hold_idx,
    y_all,
    mean,
    std,
    vocab: list[int],
    epochs: int,
    l2: float,
    seed: int,
):
    """Manual-gradient Adam for the linear model when torch is missing."""
    import numpy as np

    rng = np.random.RandomState(seed)
    n_feat = features.shape[1]
    n_vocab = len(vocab)
    w = np.zeros(n_feat, dtype=np.float64)
    zone_w = np.zeros((5, n_vocab), dtype=np.float64)
    b = 0.0
    lr = 1e-3
    b1, b2, eps = 0.9, 0.999, 1e-8
    mw = np.zeros_like(w)
    vw = np.zeros_like(w)
    mz = np.zeros_like(zone_w)
    vz = np.zeros_like(zone_w)
    mb = vb = 0.0
    idx_dtype = _index_dtype(n_vocab)

    def forward_batch(row_idx, w, zone_w, b):
        xb = _standardize_rows(features, row_idx, mean, std).astype(np.float64)
        idx_np = id_index_table(vocab, ids[row_idx], dtype=idx_dtype)
        extra = np.zeros(row_idx.size, dtype=np.float64)
        for z, zone in enumerate(ZONES):
            sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
            zidx = idx_np[:, sl]
            cnt = zone_counts(features, row_idx, zone).astype(np.float64)
            extra += (zone_w[z][zidx] * cnt).sum(axis=1)
        pre = xb @ w + extra + b
        return np.tanh(pre), xb, idx_np

    best = (w.copy(), zone_w.copy(), b)
    best_mse = float("inf")
    best_epoch = 0
    patience = 3
    t = 0
    n = train_idx.size
    y_tr = y_all[train_idx]
    epochs_run = 0
    for epoch in range(epochs):
        epochs_run = epoch + 1
        perm = rng.permutation(n)
        for start in range(0, n, BATCH):
            sel = perm[start : start + BATCH]
            row_idx = train_idx[sel]
            yb = y_tr[sel].astype(np.float64)
            pred, xb, idx_np = forward_batch(row_idx, w, zone_w, b)
            dpred = 2.0 * (pred - yb) / sel.size
            dpre = dpred * (1.0 - pred * pred)
            gw = xb.T @ dpre + l2 * w
            gb = float(dpre.sum() + l2 * b)
            gz = np.zeros_like(zone_w)
            for z, zone in enumerate(ZONES):
                sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
                zidx = idx_np[:, sl]
                cnt = zone_counts(features, row_idx, zone).astype(np.float64)
                np.add.at(gz[z], zidx.reshape(-1), (dpre[:, None] * cnt).reshape(-1))
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
        if hold_idx.size:
            pred_h, _, _ = forward_batch(hold_idx, w, zone_w, b)
            hold_mse = float(np.mean((pred_h - y_all[hold_idx].astype(np.float64)) ** 2))
            if hold_mse < best_mse - 1e-12:
                best_mse = hold_mse
                best = (w.copy(), zone_w.copy(), b)
                best_epoch = epochs_run
                patience = 3
            else:
                patience -= 1
                if patience <= 0:
                    break
        else:
            best = (w.copy(), zone_w.copy(), b)
            best_epoch = epochs_run
    w, zone_w, b = best
    return (
        w.astype(np.float32),
        zone_w.astype(np.float32),
        float(b),
        {"epochs_run": epochs_run, "best_epoch": best_epoch},
    )


def _json_safe(obj: Any) -> Any:
    """Replace NaN/Inf with None so the model file is strict JSON."""
    if isinstance(obj, float) and (obj != obj or obj in (float("inf"), float("-inf"))):
        return None
    if isinstance(obj, dict):
        return {k: _json_safe(v) for k, v in obj.items()}
    if isinstance(obj, list):
        return [_json_safe(v) for v in obj]
    return obj


def _format_metric_block(lines: list[str], who: str, block: dict[str, Any]) -> None:
    lines.append(f"--- {who} ---")
    if block.get("skipped"):
        lines.append(f"  skipped: {block['skipped']}")
        return
    for key in ("overall", "<=3", "4-6", ">=7"):
        m = block[key]
        lines.append(
            f"  {key:8} rows={m['rows']:6}  mse={m['mse']:.5f}  "
            f"sign_acc={m['sign_acc']:.4f}  auc={m['auc']:.4f}"
        )


def _eval_holdout_plan(
    ev_path: Path,
    ev_spec: dict[str, Any],
    data_encoding: int,
    data_feature_len: int,
) -> tuple[str | None, bool, str | None]:
    """Return report key, whether to use a v1 column slice, and skip note."""
    ev_enc = int(ev_spec.get("encoding", 1))
    ev_fl = int(ev_spec["feature_len"])
    if ev_enc == data_encoding and ev_fl == data_feature_len:
        return ev_path.name, False, None
    if (
        ev_enc == 1
        and data_encoding == 2
        and ev_fl == FEATURE_LEN_V1
        and data_feature_len == FEATURE_LEN_V2
    ):
        note = (
            f"eval {ev_path.name}: scoring v1 model on leading "
            f"{FEATURE_LEN_V1} columns of v2 rows"
        )
        return f"{ev_path.name} (v1 block of v2 rows)", True, note
    return None, False, "encoding mismatch"


def format_report(report: dict[str, Any]) -> str:
    lines = [
        f"arch={report['arch']}  train_rows={report['rows_train']} holdout_rows={report['rows_holdout']}",
        f"games train/holdout={report['games_train']}/{report['games_holdout']}",
        f"label balance train={report['label_balance_train']} holdout={report['label_balance_holdout']}",
        f"training time {report['train_seconds']:.2f}s",
    ]
    opt = report.get("optimizer", "adam")
    lines.append(f"optimizer={opt}")
    if opt == "adam":
        lines.append(
            f"epochs_run={report.get('epochs_run')} best_epoch={report.get('best_epoch')}"
        )
    else:
        lines.append(
            f"iterations={report.get('iterations')} stopped={report.get('stopped')} "
            f"final_loss={report.get('final_loss'):.6f} grad_max={report.get('grad_max'):.2e}"
        )
    if "target" in report:
        lines.append(
            f"target={report['target']}  mix_weight={report['mix_weight']}  "
            f"search_scale={report['search_scale']}  "
            f"search_rows={report['search_rows']}/{report['rows_train'] + report['rows_holdout']}"
        )
    for who in ("net", "v0"):
        _format_metric_block(lines, who, report[who])
    if report.get("search_v") is None:
        lines.append("--- search_v ---")
        lines.append("  (no search_v column; rows fell back to the outcome label)")
    elif "search_v" in report:
        _format_metric_block(lines, "search_v", report["search_v"])
    for name, block in (report.get("eval") or {}).items():
        _format_metric_block(lines, f"eval {name}", block)
    return "\n".join(lines)


def train(args: argparse.Namespace) -> dict[str, Any]:
    import numpy as np

    if args.optimizer == "lbfgs" and args.model != "linear":
        raise SystemExit("--optimizer lbfgs is only supported with --model linear")

    data = load_dirs(args.data, args.max_samples)

    torch = _try_torch()
    if args.model == "mlp" and torch is None:
        raise SystemExit(
            "torch is required for --model mlp (linear can train with numpy gradients)"
        )
    if args.optimizer == "lbfgs" and torch is None:
        raise SystemExit("torch is required for --optimizer lbfgs")
    feature_len = int(data["feature_len"])
    encoding = int(data["encoding"])
    features = data["features"]
    ids = data["ids"]
    labels = data["labels"]
    aux = data["aux"]
    game_index = data["game_index"]
    search_v = data["search_v"]
    has_search_v = bool(data["has_search_v"])
    if features.shape[0] == 0:
        raise SystemExit("no samples in --data")
    train_idx, hold_idx = split_by_game(game_index, args.holdout, args.seed)
    if train_idx.size == 0:
        train_idx = np.arange(features.shape[0], dtype=np.int64)
        hold_idx = np.zeros((0,), dtype=np.int64)

    scale_s = float(args.search_scale)
    mix_w = float(args.mix_weight)
    has_sv = np.isfinite(search_v)
    search_rows = int(np.sum(has_sv))
    s_unit = np.clip(search_v / scale_s, -1.0, 1.0)
    if args.target == "search":
        y_all = np.where(has_sv, s_unit, labels).astype(np.float32)
    elif args.target == "mix":
        y_all = np.where(has_sv, (1.0 - mix_w) * labels + mix_w * s_unit, labels).astype(
            np.float32
        )
    else:
        y_all = labels
    lab_tr = y_all[train_idx]
    lab_ho = y_all[hold_idx]
    turns_ho = aux[hold_idx, 3] if hold_idx.size else np.zeros((0,), dtype=np.float32)
    v0_ho = aux[hold_idx, 5] if hold_idx.size else np.zeros((0,), dtype=np.float32)
    sv_ho = search_v[hold_idx] if hold_idx.size else np.zeros((0,), dtype=np.float32)
    games_tr = int(np.unique(game_index[train_idx]).size)
    games_ho = int(np.unique(game_index[hold_idx]).size) if hold_idx.size else 0

    std_floor = float(args.std_floor)
    mean, std = _chunk_mean_std(features, train_idx, std_floor)
    vocab = build_vocab(ids, train_idx)
    import gc

    gc.collect()

    t0 = time.perf_counter()
    train_meta: dict[str, Any] = {
        "optimizer": "lbfgs" if args.optimizer == "lbfgs" else "adam"
    }
    if args.optimizer == "lbfgs":
        model, lbfgs_meta = train_lbfgs(
            features,
            ids,
            train_idx,
            y_all,
            mean,
            std,
            vocab,
            args.l2,
            args.lbfgs_iters,
            LBFGS_GRAD_TOL,
        )
        train_meta.update(lbfgs_meta)
        trained_on_stub: dict[str, Any] = {}
        spec = dump_torch(
            "linear", model, mean, std, vocab, trained_on_stub, feature_len, encoding
        )
    elif torch is not None:
        model, adam_meta = train_torch(
            args.model,
            features,
            ids,
            train_idx,
            hold_idx,
            y_all,
            mean,
            std,
            vocab,
            args.hidden,
            args.emb,
            args.epochs,
            args.l2,
            args.seed,
        )
        train_meta.update(adam_meta)
        trained_on_stub: dict[str, Any] = {}
        spec = dump_torch(
            args.model, model, mean, std, vocab, trained_on_stub, feature_len, encoding
        )
    else:
        w, zone_w, b, adam_meta = train_linear_numpy(
            features,
            ids,
            train_idx,
            hold_idx,
            y_all,
            mean,
            std,
            vocab,
            args.epochs,
            args.l2,
            args.seed,
        )
        train_meta.update(adam_meta)
        spec = {
            "arch": "linear",
            "feature_len": feature_len,
            "encoding": encoding,
            "feat_mean": mean.tolist(),
            "feat_std": std.tolist(),
            "vocab": vocab,
            "zones": _zones_json(),
            "scale": SCALE,
            "linear": {"w": w.tolist(), "zone_w": zone_w.tolist(), "b": b},
            "trained_on": {},
        }
    train_s = time.perf_counter() - t0

    if hold_idx.size:
        pred_parts = []
        for hstart in range(0, hold_idx.size, BATCH):
            hrows = hold_idx[hstart : hstart + BATCH]
            pred_parts.append(predict(spec, features[hrows], ids[hrows]))
        pred_ho = np.concatenate(pred_parts)
    else:
        pred_ho = np.zeros((0,), dtype=np.float32)
    scale = float(spec.get("scale", SCALE))
    pred_unit = pred_ho / scale if pred_ho.size else pred_ho
    net_metrics = metric_block(pred_unit, lab_ho, turns_ho)
    v0_metrics = metric_block(v0_ho, lab_ho, turns_ho)
    if has_search_v:
        sv_ok = np.isfinite(sv_ho)
        search_metrics = metric_block(sv_ho[sv_ok], lab_ho[sv_ok], turns_ho[sv_ok])
    else:
        search_metrics = None
    eval_blocks: dict[str, Any] = {}
    for path in args.eval or []:
        ev_path = Path(path)
        ev_spec = json.loads(ev_path.read_text())
        report_key, v1_slice, note = _eval_holdout_plan(
            ev_path, ev_spec, encoding, feature_len
        )
        if report_key is None:
            print(f"eval {ev_path.name}: skipped ({note})", flush=True)
            eval_blocks[ev_path.name] = {"skipped": note}
            continue
        if note:
            print(note, flush=True)
        if hold_idx.size:
            ev_parts = []
            for hstart in range(0, hold_idx.size, BATCH):
                hrows = hold_idx[hstart : hstart + BATCH]
                chunk_feat = features[hrows]
                if v1_slice:
                    chunk_feat = chunk_feat[:, :FEATURE_LEN_V1]
                ev_parts.append(predict(ev_spec, chunk_feat, ids[hrows]))
            ev_pred = np.concatenate(ev_parts)
        else:
            ev_pred = np.zeros((0,), dtype=np.float32)
        ev_scale = float(ev_spec.get("scale", SCALE))
        ev_unit = ev_pred / ev_scale if ev_pred.size else ev_pred
        eval_blocks[report_key] = metric_block(ev_unit, lab_ho, turns_ho)
    report = {
        "arch": spec["arch"],
        "rows_train": int(lab_tr.size),
        "rows_holdout": int(lab_ho.size),
        "games_train": games_tr,
        "games_holdout": games_ho,
        "label_balance_train": label_balance(lab_tr),
        "label_balance_holdout": label_balance(lab_ho),
        "train_seconds": train_s,
        "net": net_metrics,
        "v0": v0_metrics,
        "search_v": search_metrics,
        "eval": eval_blocks,
        "dirs": [str(Path(d)) for d in args.data],
        "holdout": args.holdout,
        "seed": args.seed,
        "epochs": args.epochs,
        "target": args.target,
        "mix_weight": mix_w,
        "search_scale": scale_s,
        "search_rows": search_rows,
        "std_floor": std_floor,
        **train_meta,
    }
    spec["trained_on"] = {
        "dirs": report["dirs"],
        "rows": int(features.shape[0]),
        "games": int(np.unique(game_index).size),
        "target": args.target,
        "mix_weight": mix_w,
        "search_scale": scale_s,
        "search_rows": search_rows,
        "std_floor": std_floor,
        "holdout": {
            "rows": report["rows_holdout"],
            "games": games_ho,
            "net": net_metrics,
            "v0": v0_metrics,
            "search_v": search_metrics,
        },
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(_json_safe(spec)) + "\n")
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
    p.add_argument("--target", choices=("outcome", "search", "mix"), default="outcome")
    p.add_argument("--mix-weight", type=float, default=0.5)
    p.add_argument("--search-scale", type=float, default=SCALE)
    p.add_argument("--eval", nargs="+", default=None)
    p.add_argument("--optimizer", choices=("adam", "lbfgs"), default="adam")
    p.add_argument("--lbfgs-iters", type=int, default=LBFGS_ITERS_DEFAULT)
    p.add_argument("--std-floor", type=float, default=STD_FLOOR)
    args = p.parse_args(argv)
    train(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
