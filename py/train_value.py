#!/usr/bin/env python3
"""Train a linear or small-MLP leaf value from matchup export shards."""

from __future__ import annotations

import argparse
import importlib.util
import json
import time
from pathlib import Path
from typing import Any

_RACE_MOD = Path(__file__).resolve().parent / "race.py"
_race_spec = importlib.util.spec_from_file_location("arena_race", _RACE_MOD)
assert _race_spec and _race_spec.loader
_race = importlib.util.module_from_spec(_race_spec)
_race_spec.loader.exec_module(_race)
RACE_NAMES = _race.RACE_NAMES
race_features = _race.race_features

FEATURE_LEN_V1 = 545
FEATURE_LEN_V2 = 567
FEATURE_LEN_V3 = 961
IDS_LEN_V3 = 230
HIST_WIDTH = 96
IDS_OWN_CRESTS = 220
IDS_OPP_CRESTS = 225
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
STACK_SEARCH_ROWS_DEFAULT = 300_000

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

ZONES_V3: list[dict[str, Any]] = ZONES + [
    {"name": "own_crests", "id_offset": IDS_OWN_CRESTS, "count": 5, "hist_offset": None},
    {"name": "opp_crests", "id_offset": IDS_OPP_CRESTS, "count": 5, "hist_offset": None},
    {"name": "own_amulet_soon", "id_offset": IDS_OWN_BOARD, "count": 5, "hist_offset": 567},
    {"name": "opp_amulet_soon", "id_offset": IDS_OPP_BOARD, "count": 5, "hist_offset": 572},
    {"name": "own_entered", "id_offset": IDS_OWN_DECK, "count": HIST_WIDTH, "hist_offset": 577},
    {"name": "opp_entered", "id_offset": IDS_OPP_POOL, "count": HIST_WIDTH, "hist_offset": 673},
    {"name": "own_cemetery", "id_offset": IDS_OWN_DECK, "count": HIST_WIDTH, "hist_offset": 769},
    {"name": "opp_cemetery", "id_offset": IDS_OPP_POOL, "count": HIST_WIDTH, "hist_offset": 865},
]

STACK_ZONE_GROUPS = ("crests", "amulets", "entered", "cemetery")
STACK_ZONE_ROWS = {
    "crests": (5, 7),
    "amulets": (7, 9),
    "entered": (9, 11),
    "cemetery": (11, 13),
}


def zones_for_encoding(encoding: int) -> list[dict[str, Any]]:
    return ZONES_V3 if encoding == 3 else ZONES


def _load_samples():
    path = Path(__file__).resolve().parent / "samples.py"
    spec = importlib.util.spec_from_file_location("arena_samples", path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _try_torch():
    import os

    if os.environ.get("TORCH_DISABLE"):
        return None
    try:
        import torch

        return torch
    except ImportError:
        return None


def _index_dtype(n_vocab: int):
    import numpy as np

    return np.uint16 if n_vocab <= 65535 else np.uint32


def _chunk_race_mean_std(features, row_idx, std_floor: float, chunk: int = 2048):
    """Mean/std of race inputs over training rows."""
    import numpy as np

    floor = float(std_floor)
    if row_idx.size == 0:
        return np.zeros(12, np.float32), np.full(12, floor, np.float32)
    sum_v = np.zeros(12, np.float64)
    sum_sq = np.zeros(12, np.float64)
    n = int(row_idx.size)
    for start in range(0, n, chunk):
        sel = row_idx[start : start + chunk]
        block = race_features(features[sel])
        sum_v += block.sum(axis=0, dtype=np.float64)
        sum_sq += np.multiply(block, block, dtype=np.float32).sum(axis=0, dtype=np.float64)
    mean = (sum_v / n).astype(np.float32)
    var = np.maximum(sum_sq / n - mean.astype(np.float64) ** 2, 0.0)
    std = np.maximum(np.sqrt(var).astype(np.float32), floor)
    return mean, std


def _standardize_race_rows(features, row_idx, mean, std, out=None):
    import numpy as np

    x = (race_features(features[row_idx]) - mean) / std
    if out is None:
        return x.astype(np.float32, copy=False)
    n = row_idx.size
    out[:n] = x
    return out[:n]


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
        if "race" in spec:
            rb = spec["race"]
            r = race_features(features)
            r_x = (r - np.asarray(rb["mean"], dtype=np.float32)) / np.asarray(
                rb["std"], dtype=np.float32
            )
            pre = pre + r_x @ np.asarray(rb["w"], dtype=np.float32)
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


def _zones_json(encoding: int = 1) -> list[dict[str, Any]]:
    return [
        {
            "name": z["name"],
            "id_offset": z["id_offset"],
            "count": z["count"],
            "hist_offset": z["hist_offset"],
        }
        for z in zones_for_encoding(encoding)
    ]


class LinearTorch:
    def __init__(
        self,
        torch,
        n_feat: int,
        n_vocab: int,
        n_zones: int = 5,
        use_race: bool = False,
        encoding: int = 1,
    ):
        self.torch = torch
        self.n_zones = n_zones
        self.encoding = encoding
        self.w = torch.nn.Parameter(torch.zeros(n_feat))
        self.zone_w = torch.nn.Parameter(torch.zeros(n_zones, n_vocab))
        self.b = torch.nn.Parameter(torch.zeros(1))
        self.use_race = use_race
        self.race_w = torch.nn.Parameter(torch.zeros(12)) if use_race else None
        if encoding == 3 and n_zones >= 13:
            self.zone_w.data[5:13, 0] = 0.0

            def _zero_new_zone_grad(grad):
                grad[5:13, 0] = 0
                return grad

            self.zone_w.register_hook(_zero_new_zone_grad)

    def parameters(self):
        out = [self.w, self.zone_w, self.b]
        if self.race_w is not None:
            out.append(self.race_w)
        return out

    def forward(self, x, idx, counts, race_x=None):
        extra = self.torch.zeros(x.shape[0], device=x.device)
        for z in range(self.n_zones):
            extra = extra + (self.zone_w[z][idx[z]] * counts[z]).sum(dim=1)
        pre = x @ self.w + extra + self.b[0]
        if self.race_w is not None and race_x is not None:
            pre = pre + race_x @ self.race_w
        return pre.tanh()

    def l2_penalty(self, l2: float):
        pen = self.w.pow(2).sum() + self.zone_w.pow(2).sum() + self.b.pow(2).sum()
        if self.race_w is not None:
            pen = pen + self.race_w.pow(2).sum()
        return 0.5 * l2 * pen


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

    def forward(self, x, idx, counts, race_x=None):
        parts = [x]
        for z in range(5):
            parts.append((self.emb[idx[z]] * counts[z].unsqueeze(-1)).sum(dim=1))
        inp = self.torch.cat(parts, dim=1)
        h = (inp @ self.w1.t() + self.b1).relu()
        return (h @ self.w2 + self.b2[0]).tanh()


def _batch_zone_tensors(torch, features, ids, row_idx, vocab, zones, device):
    idx_np = id_index_table(vocab, ids[row_idx])
    idx = []
    counts = []
    for zone in zones:
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
    use_race: bool = False,
    race_mean=None,
    race_std=None,
    encoding: int = 1,
):
    import numpy as np

    torch = _try_torch()
    assert torch is not None
    torch.manual_seed(seed)
    device = torch.device("cpu")
    n_feat = features.shape[1]
    n_vocab = len(vocab)
    zones = zones_for_encoding(encoding)
    if spec_arch == "linear":
        model = LinearTorch(
            torch, n_feat, n_vocab, len(zones), use_race=use_race, encoding=encoding
        )
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
    race_buf = np.empty((BATCH, 12), dtype=np.float32) if use_race else None
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
            idx_b, c_b = _batch_zone_tensors(torch, features, ids, row_idx, vocab, zones, device)
            race_b = None
            if use_race:
                race_b = torch.from_numpy(
                    _standardize_race_rows(features, row_idx, race_mean, race_std, race_buf)
                ).to(device)
            pred = model.forward(xb, idx_b, c_b, race_b)
            yb = torch.from_numpy(y_tr_np[b]).to(device)
            loss = torch.mean((pred - yb) ** 2)
            opt.zero_grad(set_to_none=True)
            loss.backward()
            opt.step()
            if encoding == 3:
                model.zone_w.data[5:13, 0] = 0.0
        if has_hold:
            with torch.no_grad():
                hold_sq = 0.0
                for hstart in range(0, hold_idx.size, BATCH):
                    hrows = hold_idx[hstart : hstart + BATCH]
                    xb = torch.from_numpy(
                        _standardize_rows(features, hrows, mean, std, x_buf)
                    ).to(device)
                    idx_b, c_b = _batch_zone_tensors(torch, features, ids, hrows, vocab, zones, device)
                    race_b = None
                    if use_race:
                        race_b = torch.from_numpy(
                            _standardize_race_rows(features, hrows, race_mean, race_std, race_buf)
                        ).to(device)
                    pred_h = model.forward(xb, idx_b, c_b, race_b)
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
    encoding: int = 1,
):
    import numpy as np

    torch = _try_torch()
    assert torch is not None
    device = torch.device("cpu")
    n_feat = features.shape[1]
    n_vocab = len(vocab)
    zones = zones_for_encoding(encoding)
    model = LinearTorch(torch, n_feat, n_vocab, len(zones), encoding=encoding)
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
            idx_b, c_b = _batch_zone_tensors(torch, features, ids, row_idx, vocab, zones, device)
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
        if encoding == 3:
            model.zone_w.data[5:13, 0] = 0.0
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
    race_mean=None,
    race_std=None,
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
        "zones": _zones_json(encoding),
        "scale": SCALE,
        "trained_on": trained_on,
    }
    if arch == "linear":
        out["linear"] = {
            "w": to_list(model.w),
            "zone_w": to_list(model.zone_w),
            "b": float(model.b.detach().cpu().item()),
        }
        if getattr(model, "race_w", None) is not None:
            out["race"] = {
                "version": 1,
                "names": list(RACE_NAMES),
                "mean": np.asarray(race_mean, dtype=np.float32).tolist(),
                "std": np.asarray(race_std, dtype=np.float32).tolist(),
                "w": to_list(model.race_w),
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
    use_race: bool = False,
    race_mean=None,
    race_std=None,
    encoding: int = 1,
):
    """Manual-gradient Adam for the linear model when torch is missing."""
    import numpy as np

    rng = np.random.RandomState(seed)
    n_feat = features.shape[1]
    n_vocab = len(vocab)
    zones = zones_for_encoding(encoding)
    n_zones = len(zones)
    w = np.zeros(n_feat, dtype=np.float64)
    zone_w = np.zeros((n_zones, n_vocab), dtype=np.float64)
    b = 0.0
    race_w = np.zeros(12, dtype=np.float64) if use_race else None
    lr = 1e-3
    b1, b2, eps = 0.9, 0.999, 1e-8
    mw = np.zeros_like(w)
    vw = np.zeros_like(w)
    mz = np.zeros_like(zone_w)
    vz = np.zeros_like(zone_w)
    mb = vb = 0.0
    mrw = vrw = None
    if race_w is not None:
        mrw = np.zeros_like(race_w)
        vrw = np.zeros_like(race_w)
    idx_dtype = _index_dtype(n_vocab)

    def forward_batch(row_idx, w, zone_w, b, rw=None):
        xb = _standardize_rows(features, row_idx, mean, std).astype(np.float64)
        idx_np = id_index_table(vocab, ids[row_idx], dtype=idx_dtype)
        extra = np.zeros(row_idx.size, dtype=np.float64)
        for z, zone in enumerate(zones):
            sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
            zidx = idx_np[:, sl]
            cnt = zone_counts(features, row_idx, zone).astype(np.float64)
            extra += (zone_w[z][zidx] * cnt).sum(axis=1)
        pre = xb @ w + extra + b
        if rw is not None:
            rx = _standardize_race_rows(features, row_idx, race_mean, race_std).astype(np.float64)
            pre = pre + rx @ rw
        return np.tanh(pre), xb, idx_np

    best = (w.copy(), zone_w.copy(), b, race_w.copy() if race_w is not None else None)
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
            pred, xb, idx_np = forward_batch(row_idx, w, zone_w, b, race_w)
            dpred = 2.0 * (pred - yb) / sel.size
            dpre = dpred * (1.0 - pred * pred)
            gw = xb.T @ dpre + l2 * w
            gb = float(dpre.sum() + l2 * b)
            grw = None
            if race_w is not None:
                rx = _standardize_race_rows(features, row_idx, race_mean, race_std).astype(
                    np.float64
                )
                grw = rx.T @ dpre + l2 * race_w
            gz = np.zeros_like(zone_w)
            for z, zone in enumerate(zones):
                sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
                zidx = idx_np[:, sl]
                cnt = zone_counts(features, row_idx, zone).astype(np.float64)
                np.add.at(gz[z], zidx.reshape(-1), (dpre[:, None] * cnt).reshape(-1))
                gz[z] += l2 * zone_w[z]
            if encoding == 3:
                gz[5:13, 0] = 0.0
            t += 1
            mw[:] = b1 * mw + (1 - b1) * gw
            vw[:] = b2 * vw + (1 - b2) * (gw * gw)
            w -= lr * (mw / (1 - b1**t)) / (np.sqrt(vw / (1 - b2**t)) + eps)
            mz[:] = b1 * mz + (1 - b1) * gz
            vz[:] = b2 * vz + (1 - b2) * (gz * gz)
            zone_w -= lr * (mz / (1 - b1**t)) / (np.sqrt(vz / (1 - b2**t)) + eps)
            _zero_v3_new_zone_index0(zone_w, encoding)
            mb = b1 * mb + (1 - b1) * gb
            vb = b2 * vb + (1 - b2) * (gb * gb)
            b -= lr * (mb / (1 - b1**t)) / ((vb / (1 - b2**t)) ** 0.5 + eps)
            if race_w is not None and grw is not None:
                mrw[:] = b1 * mrw + (1 - b1) * grw
                vrw[:] = b2 * vrw + (1 - b2) * (grw * grw)
                race_w -= lr * (mrw / (1 - b1**t)) / (np.sqrt(vrw / (1 - b2**t)) + eps)
        if hold_idx.size:
            pred_h, _, _ = forward_batch(hold_idx, w, zone_w, b, race_w)
            hold_mse = float(np.mean((pred_h - y_all[hold_idx].astype(np.float64)) ** 2))
            if hold_mse < best_mse - 1e-12:
                best_mse = hold_mse
                best = (w.copy(), zone_w.copy(), b, race_w.copy() if race_w is not None else None)
                best_epoch = epochs_run
                patience = 3
            else:
                patience -= 1
                if patience <= 0:
                    break
        else:
            best = (w.copy(), zone_w.copy(), b, race_w.copy() if race_w is not None else None)
            best_epoch = epochs_run
    w, zone_w, b, race_w = best
    _zero_v3_new_zone_index0(zone_w, encoding)
    return (
        w.astype(np.float32),
        zone_w.astype(np.float32),
        float(b),
        race_w.astype(np.float32) if race_w is not None else None,
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
) -> tuple[str | None, int | None, str | None]:
    """Return report key, feature prefix length to slice (if any), and skip note."""
    ev_enc = int(ev_spec.get("encoding", 1))
    ev_fl = int(ev_spec["feature_len"])
    if ev_enc == data_encoding and ev_fl == data_feature_len:
        return ev_path.name, None, None
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
        return f"{ev_path.name} (v1 block of v2 rows)", FEATURE_LEN_V1, note
    if (
        ev_enc == 2
        and data_encoding == 3
        and ev_fl == FEATURE_LEN_V2
        and data_feature_len == FEATURE_LEN_V3
    ):
        note = (
            f"eval {ev_path.name}: scoring v2 model on leading "
            f"{FEATURE_LEN_V2} columns of v3 rows"
        )
        return f"{ev_path.name} (v2 block of v3 rows)", FEATURE_LEN_V2, note
    if (
        ev_enc == 1
        and data_encoding == 3
        and ev_fl == FEATURE_LEN_V1
        and data_feature_len == FEATURE_LEN_V3
    ):
        note = (
            f"eval {ev_path.name}: scoring v1 model on leading "
            f"{FEATURE_LEN_V1} columns of v3 rows"
        )
        return f"{ev_path.name} (v1 block of v3 rows)", FEATURE_LEN_V1, note
    return None, None, "encoding mismatch"


def compute_pre_activation(spec: dict[str, Any], features, ids):
    """Linear pre-activation (no tanh, no scale)."""
    import numpy as np

    fl = int(spec["feature_len"])
    mean = np.asarray(spec["feat_mean"], dtype=np.float32)
    std = np.asarray(spec["feat_std"], dtype=np.float32)
    x = (features[:, :fl].astype(np.float32, copy=False) - mean) / std
    vocab = [int(v) for v in spec["vocab"]]
    idx = id_index_table(vocab, ids)
    zones = spec["zones"]
    lin = spec["linear"]
    w = np.asarray(lin["w"], dtype=np.float32)
    zone_w = np.asarray(lin["zone_w"], dtype=np.float32)
    b = float(lin["b"])
    row_idx = np.arange(features.shape[0], dtype=np.int64)
    extra = np.zeros(features.shape[0], dtype=np.float32)
    for z, zone in enumerate(zones):
        sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
        zidx = idx[:, sl]
        count = zone_counts(features, row_idx, zone)
        extra += (zone_w[z][zidx] * count).sum(axis=1)
    pre = x @ w + extra + b
    if "race" in spec:
        rb = spec["race"]
        r = race_features(features[:, :FEATURE_LEN_V1])
        r_x = (r - np.asarray(rb["mean"], dtype=np.float32)) / np.asarray(
            rb["std"], dtype=np.float32
        )
        pre = pre + r_x @ np.asarray(rb["w"], dtype=np.float32)
    return pre.astype(np.float32, copy=False)


def _validate_base_model(base: dict[str, Any], path: Path) -> None:
    if int(base.get("encoding", 1)) != 2:
        raise SystemExit(f"--stack-on {path}: base must be encoding 2")
    if base.get("arch") != "linear":
        raise SystemExit(f"--stack-on {path}: base must be arch linear")
    if int(base["feature_len"]) != FEATURE_LEN_V2:
        raise SystemExit(f"--stack-on {path}: base feature_len must be {FEATURE_LEN_V2}")
    zones = base.get("zones") or []
    if len(zones) != 5:
        raise SystemExit(f"--stack-on {path}: base zones must be the standard five")
    for got, want in zip(zones, _zones_json(2)):
        if (
            got.get("name") != want["name"]
            or int(got.get("id_offset")) != want["id_offset"]
            or int(got.get("count")) != want["count"]
            or got.get("hist_offset") != want["hist_offset"]
        ):
            raise SystemExit(f"--stack-on {path}: base zones must be the standard five")


def _shard_deck_layout(meta: dict[str, Any]) -> tuple[list[str], int]:
    decks = list(meta.get("decks") or [])
    games = int(meta.get("games", 0))
    if not decks or games <= 0:
        raise ValueError("shard missing decks or games")
    return decks, games


def _row_deck_indices(
    game_index,
    side,
    first_is_me,
    decks: list[str],
    games_per_pair: int,
):
    import numpy as np

    n = len(decks)
    gi = np.asarray(game_index, dtype=np.int64)
    pair = gi // games_per_pair
    i = pair // n
    j = pair % n
    s = np.asarray(side, dtype=np.int64)
    own = np.where(s == 0, i, j)
    opp = np.where(s == 0, j, i)
    return own.astype(np.int32), opp.astype(np.int32), np.asarray(first_is_me, dtype=np.float32)


def _stack_deck_controls(
    aux,
    cols: list[str],
    decks: list[str],
    games_per_pair: int,
):
    import numpy as np

    n_decks = len(decks)
    gi = aux[:, cols.index("game_index")].astype(np.int64, copy=False)
    side = aux[:, cols.index("side")]
    first_is_me = aux[:, cols.index("first_is_me")]
    own, opp, first = _row_deck_indices(gi, side, first_is_me, decks, games_per_pair)
    max_game = n_decks * n_decks * games_per_pair
    if gi.size and (int(gi.min()) < 0 or int(gi.max()) >= max_game):
        raise SystemExit(
            f"--stack-on deck controls: game_index out of range [0, {max_game}) "
            f"(got min={int(gi.min())}, max={int(gi.max())})"
        )
    if own.size and (
        int(own.min()) < 0
        or int(own.max()) >= n_decks
        or int(opp.min()) < 0
        or int(opp.max()) >= n_decks
    ):
        raise SystemExit(
            f"--stack-on deck controls: deck index out of range [0, {n_decks}) "
            f"(own min={int(own.min())}, max={int(own.max())}; "
            f"opp min={int(opp.min())}, max={int(opp.max())})"
        )
    return own, opp, first


def _stack_vocab(base_vocab: list[int], ids, row_idx) -> list[int]:
    seen = set(int(v) for v in base_vocab)
    import numpy as np

    for start in range(0, row_idx.size, 8192):
        sel = row_idx[start : start + 8192]
        seen.update(int(x) for x in np.unique(ids[sel]).tolist())
    rest = sorted(i for i in seen if i != 0)
    return [0] + rest


def _reindex_zone_row(row: list[float], old_vocab: list[int], new_vocab: list[int]) -> list[float]:
    import numpy as np

    old = np.asarray(row, dtype=np.float64)
    empty = float(old[0])
    out = np.full(len(new_vocab), empty, dtype=np.float64)
    new_set = set(new_vocab)
    for oid, card in enumerate(old_vocab):
        if card not in new_set:
            continue
        out[new_vocab.index(card)] = old[oid]
    return out.astype(np.float32).tolist()


def _zero_v3_new_zone_index0(zone_w, encoding: int) -> None:
    if encoding != 3:
        return
    import numpy as np

    zw = np.asarray(zone_w)
    zw[5:13, 0] = 0.0


def _subsample_row_idx_by_games(
    row_idx,
    game_index,
    max_rows: int | None,
    seed: int,
):
    import numpy as np

    row_idx = np.asarray(row_idx, dtype=np.int64)
    if max_rows is None or row_idx.size <= max_rows:
        return row_idx
    games = np.unique(game_index[row_idx])
    rng = np.random.RandomState(seed)
    perm = rng.permutation(games)
    picked: list[np.ndarray] = []
    count = 0
    for g in perm:
        rows = row_idx[game_index[row_idx] == g]
        if count + rows.size > max_rows and count > 0:
            break
        picked.append(rows)
        count += rows.size
        if count >= max_rows:
            break
    if not picked:
        return row_idx[:max_rows]
    return np.concatenate(picked).astype(np.int64, copy=False)


def _stack_sparse_zones(
    features,
    ids,
    row_idx,
    vocab: list[int],
    zones: list[dict[str, Any]],
    z_lo: int,
    z_hi: int,
):
    """Per new zone: (row, vocab_index, value) with id>0 and value>0 where required."""
    import numpy as np

    n = row_idx.size
    idx = id_index_table(vocab, ids[row_idx])
    out: list[tuple[np.ndarray, np.ndarray, np.ndarray]] = []
    for z in range(z_lo, z_hi):
        zone = zones[z]
        sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
        zidx = idx[:, sl]
        count = zone_counts(features, row_idx, zone)
        rows_all: list[np.ndarray] = []
        cols_all: list[np.ndarray] = []
        vals_all: list[np.ndarray] = []
        for slot in range(int(zone["count"])):
            c = count[:, slot]
            ids_s = zidx[:, slot]
            if zone["hist_offset"] is not None:
                mask = (c > 0) & (ids_s > 0)
            else:
                mask = ids_s > 0
            if not np.any(mask):
                continue
            local = np.flatnonzero(mask)
            rows_all.append(local)
            cols_all.append(ids_s[mask].astype(np.int64, copy=False))
            vals_all.append(c[mask].astype(np.float64, copy=False))
        if rows_all:
            out.append(
                (
                    np.concatenate(rows_all),
                    np.concatenate(cols_all),
                    np.concatenate(vals_all),
                )
            )
        else:
            out.append(
                (
                    np.zeros(0, dtype=np.int64),
                    np.zeros(0, dtype=np.int64),
                    np.zeros(0, dtype=np.float64),
                )
            )
    return out


def _stack_zone_extra_sparse(zw, sparse):
    import numpy as np

    extra = np.zeros(sparse["n"], dtype=np.float64)
    for zi, (rows, cols, vals) in enumerate(sparse["zones"]):
        if rows.size:
            np.add.at(extra, rows, vals * zw[zi, cols])
    return extra


def _stack_coordinate_search(
    grid: list[float],
    groups: tuple[str, ...],
    fit_rows,
    score_mse,
    lams_init: dict[str, float],
):
    lams = dict(lams_init)
    step_mses: list[tuple[str, float]] = []
    for group in groups:
        best_lam = lams[group]
        best_mse = float("inf")
        for lam in grid:
            trial = dict(lams)
            trial[group] = lam
            theta = fit_rows(trial)
            mse = score_mse(theta)
            if mse < best_mse - 1e-9 or (abs(mse - best_mse) <= 1e-9 and lam > best_lam):
                best_mse = mse
                best_lam = lam
        lams[group] = best_lam
        step_mses.append((group, best_mse))
    return lams, step_mses


def _new_zone_extra(
    zone_w,
    ids,
    features,
    row_idx,
    vocab: list[int],
    zones: list[dict[str, Any]],
    z_lo: int,
    z_hi: int,
):
    import numpy as np

    idx = id_index_table(vocab, ids[row_idx])
    extra = np.zeros(row_idx.size, dtype=np.float64)
    for z in range(z_lo, z_hi):
        zone = zones[z]
        sl = slice(int(zone["id_offset"]), int(zone["id_offset"]) + int(zone["count"]))
        zidx = idx[:, sl]
        count = zone_counts(features, row_idx, zone)
        row_w = zone_w[z - z_lo]
        for slot in range(int(zone["count"])):
            c = count[:, slot]
            ids_s = zidx[:, slot]
            if zone["hist_offset"] is not None:
                mask = c > 0
                if not np.any(mask):
                    continue
                extra[mask] += c[mask] * row_w[ids_s[mask]]
            else:
                extra += c * row_w[ids_s]
    return extra


def _mse_tanh(pre, y):
    import numpy as np

    return float(np.mean((np.tanh(pre) - y) ** 2))


def _lbfgs_numpy(
    grad_fn,
    x0,
    max_iter: int = 200,
    grad_tol: float = 1e-6,
):
    import numpy as np

    x = x0.astype(np.float64, copy=True)
    m = 10
    s_hist: list[np.ndarray] = []
    y_hist: list[np.ndarray] = []
    rho_hist: list[float] = []
    _, g = grad_fn(x)
    for _ in range(max_iter):
        if np.linalg.norm(g, ord=np.inf) <= grad_tol:
            break
        q = g.copy()
        for s, y, rho in zip(reversed(s_hist), reversed(y_hist), reversed(rho_hist)):
            a = rho * float(s @ q)
            q = q - a * y
        if s_hist:
            ys = float(y_hist[-1] @ s_hist[-1])
            yy = float(y_hist[-1] @ y_hist[-1])
            h0 = ys / yy if yy > 0 else 1.0
        else:
            h0 = 1.0
        r = h0 * q
        for s, y, rho in zip(s_hist, y_hist, rho_hist):
            b = rho * float(y @ r)
            r = r + (float(s @ q) - b) * s
        p = -r
        loss0, g0 = grad_fn(x)
        step = 1.0
        for _ls in range(20):
            x_new = x + step * p
            loss1, g1 = grad_fn(x_new)
            if loss1 <= loss0 + 1e-4 * step * float(g0 @ p):
                s = x_new - x
                y = g1 - g0
                x = x_new
                g = g1
                if float(s @ y) > 1e-12:
                    rho = 1.0 / float(s @ y)
                    s_hist.append(s)
                    y_hist.append(y)
                    rho_hist.append(rho)
                    if len(s_hist) > m:
                        s_hist.pop(0)
                        y_hist.pop(0)
                        rho_hist.pop(0)
                break
            step *= 0.5
        else:
            break
    return x


def train_stack_on(args: argparse.Namespace) -> dict[str, Any]:
    import hashlib
    import numpy as np

    base_path = Path(args.stack_on)
    base = json.loads(base_path.read_text())
    _validate_base_model(base, base_path)
    data = load_dirs(args.data, args.max_samples)
    encoding = int(data["encoding"])
    if encoding != 3:
        raise SystemExit("--stack-on requires encoding-3 data")
    features = data["features"]
    ids = data["ids"]
    labels = data["labels"]
    aux = data["aux"]
    game_index = data["game_index"]
    if features.shape[0] == 0:
        raise SystemExit("no samples in --data")
    train_idx, hold_idx = split_by_game(game_index, args.holdout, args.seed)
    if hold_idx.size == 0:
        raise SystemExit("--stack-on requires a non-empty holdout split")

    metas = [_read_shard_meta(Path(d)) for d in args.data]
    aux_cols = [list(m["aux_columns"]) for m in metas]
    if len(set(tuple(c) for c in aux_cols)) > 1:
        raise SystemExit(
            "--stack-on requires identical aux_columns across --data folders "
            + ", ".join(args.data)
        )
    try:
        decks, games_per_pair = _shard_deck_layout(metas[0])
        for m in metas[1:]:
            d2, g2 = _shard_deck_layout(m)
            if d2 != decks or g2 != games_per_pair:
                raise ValueError("mixed deck layout")
    except ValueError as e:
        if not getattr(args, "no_deck_controls", False):
            raise SystemExit(f"--stack-on deck controls: {e} (use --no-deck-controls to skip)")
        decks, games_per_pair = [], 0

    cols = list(metas[0]["aux_columns"])
    if decks:
        own_deck, opp_deck, first_col = _stack_deck_controls(
            aux, cols, decks, games_per_pair
        )
        n_decks = len(decks)
        use_deck = True
    else:
        own_deck = opp_deck = np.zeros(features.shape[0], dtype=np.int32)
        first_col = np.zeros(features.shape[0], dtype=np.float32)
        n_decks = 0
        use_deck = False

    y_all = labels.astype(np.float32, copy=False)
    base_pre = compute_pre_activation(base, features, ids)
    vocab = _stack_vocab([int(v) for v in base["vocab"]], ids, np.arange(features.shape[0]))
    n_vocab = len(vocab)
    zones = zones_for_encoding(3)
    z_lo, z_hi = 5, 13
    n_zone_w = (z_hi - z_lo) * (n_vocab - 1)
    n_deck_w = (2 * n_decks + 1) if use_deck else 0

    grid = [float(x) for x in getattr(args, "l2_grid", "1e-6,1e-5,1e-4,1e-3,1e-2,1e-1,1").split(",")]
    stack_search_rows = int(
        getattr(args, "stack_search_rows", STACK_SEARCH_ROWS_DEFAULT)
    )
    tr = train_idx
    ho = hold_idx
    search_tr = _subsample_row_idx_by_games(tr, game_index, stack_search_rows, args.seed)

    def _prepare(rows):
        return {
            "n": int(rows.size),
            "base_pre": base_pre[rows].astype(np.float64, copy=False),
            "y": y_all[rows],
            "own_deck": own_deck[rows],
            "opp_deck": opp_deck[rows],
            "first_col": first_col[rows],
            "zones": _stack_sparse_zones(features, ids, rows, vocab, zones, z_lo, z_hi),
        }

    def unpack(theta):
        zw = theta[:n_zone_w].reshape(z_hi - z_lo, n_vocab - 1)
        full = np.zeros((z_hi - z_lo, n_vocab), dtype=np.float64)
        full[:, 1:] = zw
        if use_deck:
            d0 = n_zone_w
            own_w = theta[d0 : d0 + n_decks]
            opp_w = theta[d0 + n_decks : d0 + 2 * n_decks]
            first_w = theta[d0 + 2 * n_decks]
            return full, own_w, opp_w, first_w
        return full, None, None, None

    def objective(theta, prep, lams, return_grad=False, return_loss=False):
        zw, own_w, opp_w, first_w = unpack(theta)
        pre = prep["base_pre"].astype(np.float64, copy=True)
        if use_deck:
            pre += own_w[prep["own_deck"]]
            pre += opp_w[prep["opp_deck"]]
            pre += first_w * prep["first_col"]
        pre += _stack_zone_extra_sparse(zw, prep)
        y = prep["y"]
        n = prep["n"]
        pen = 0.0
        for name, (a, b) in STACK_ZONE_ROWS.items():
            lam = lams[name]
            pen += 0.5 * lam * float(np.sum(zw[a - z_lo : b - z_lo, 1:] ** 2))
        loss = float(np.mean((np.tanh(pre) - y) ** 2)) + pen
        if not return_grad and not return_loss:
            return loss
        if return_loss and not return_grad:
            return loss, np.zeros_like(theta)
        grad = np.zeros_like(theta)
        resid = (2.0 / n) * (np.tanh(pre) - y) * (1.0 - np.tanh(pre) ** 2)
        if use_deck:
            d0 = n_zone_w
            od, opd, fc = prep["own_deck"], prep["opp_deck"], prep["first_col"]
            for d in range(n_decks):
                grad[d0 + d] = float(np.sum(resid[od == d]))
                grad[d0 + n_decks + d] = float(np.sum(resid[opd == d]))
            grad[d0 + 2 * n_decks] = float(np.sum(resid * fc))
        off = 0
        for zi, (rows, cols, vals) in enumerate(prep["zones"]):
            if rows.size:
                np.add.at(grad[off : off + n_vocab - 1], cols - 1, resid[rows] * vals)
            off += n_vocab - 1
        for name, (a, b) in STACK_ZONE_ROWS.items():
            lam = lams[name]
            for z in range(a, b):
                g0 = (z - z_lo) * (n_vocab - 1)
                g1 = g0 + (n_vocab - 1)
                grad[g0:g1] += lam * zw[z - z_lo, 1:].reshape(-1)
        if return_loss:
            return loss, grad
        return grad

    def fit_rows(row_idx, lams, deck_start=None):
        prep = _prepare(row_idx)
        theta0 = np.zeros(n_zone_w + n_deck_w, dtype=np.float64)
        if use_deck and deck_start is not None:
            theta0[n_zone_w:] = deck_start

        def fn(x):
            return objective(x, prep, lams, return_grad=True, return_loss=True)

        return _lbfgs_numpy(fn, theta0)

    lams = {g: grid[-1] for g in STACK_ZONE_GROUPS}
    deck_theta = np.zeros(n_deck_w, dtype=np.float64)
    if use_deck:
        tr_prep = _prepare(tr)

        def deck_obj(dw, prep_fit, return_loss=False):
            pre = prep_fit["base_pre"].astype(np.float64, copy=True)
            pre += dw[0:n_decks][prep_fit["own_deck"]]
            pre += dw[n_decks : 2 * n_decks][prep_fit["opp_deck"]]
            pre += dw[2 * n_decks] * prep_fit["first_col"]
            loss = float(np.mean((np.tanh(pre) - prep_fit["y"]) ** 2))
            if not return_loss:
                return loss
            grad = np.zeros_like(dw)
            resid = (2.0 / prep_fit["n"]) * (np.tanh(pre) - prep_fit["y"]) * (
                1.0 - np.tanh(pre) ** 2
            )
            for d in range(n_decks):
                grad[d] = float(np.sum(resid[prep_fit["own_deck"] == d]))
                grad[n_decks + d] = float(np.sum(resid[prep_fit["opp_deck"] == d]))
            grad[2 * n_decks] = float(np.sum(resid * prep_fit["first_col"]))
            return loss, grad

        deck_theta = _lbfgs_numpy(
            lambda x: deck_obj(x, tr_prep, return_loss=True),
            deck_theta,
        )

    def with_deck(pre, rows):
        if not use_deck:
            return pre
        out = pre.astype(np.float64, copy=True)
        out += deck_theta[0:n_decks][own_deck[rows]]
        out += deck_theta[n_decks : 2 * n_decks][opp_deck[rows]]
        out += deck_theta[2 * n_decks] * first_col[rows]
        return out

    def pre_from_fit(theta, rows):
        zw, own_w, opp_w, first_w = unpack(theta)
        prep = _prepare(rows)
        pre = prep["base_pre"].astype(np.float64, copy=True)
        if use_deck and own_w is not None:
            pre += own_w[prep["own_deck"]]
            pre += opp_w[prep["opp_deck"]]
            pre += first_w * prep["first_col"]
        pre += _stack_zone_extra_sparse(zw, prep)
        return pre

    mse_base = _mse_tanh(with_deck(base_pre[ho], ho), y_all[ho])
    step_mses = [("base+deck", mse_base)]
    def fit_search(lams):
        return fit_rows(search_tr, lams, deck_start=deck_theta)

    def score_theta(theta):
        return _mse_tanh(pre_from_fit(theta, ho), y_all[ho])

    t_search = time.perf_counter()
    lams, group_mses = _stack_coordinate_search(
        grid, STACK_ZONE_GROUPS, fit_search, score_theta, lams
    )
    search_s = time.perf_counter() - t_search
    step_mses.extend(group_mses)

    warnings: list[str] = []
    for g, lam in lams.items():
        if lam == grid[0]:
            warnings.append(f"{g} penalty at grid minimum ({lam})")
        if lam == grid[-1]:
            warnings.append(f"{g} penalty at grid maximum ({lam})")

    t_train_refit = time.perf_counter()
    theta_train = fit_rows(tr, lams, deck_start=deck_theta)
    train_refit_s = time.perf_counter() - t_train_refit
    final_holdout_mse = _mse_tanh(pre_from_fit(theta_train, ho), y_all[ho])
    step_mses.append(("final (training rows)", final_holdout_mse))

    all_idx = np.arange(features.shape[0], dtype=np.int32)
    t_refit = time.perf_counter()
    theta = fit_rows(all_idx, lams, deck_start=deck_theta)
    refit_s = time.perf_counter() - t_refit
    zw, own_w, opp_w, first_w = unpack(theta)

    base_w = list(base["linear"]["w"])
    base_mean = list(base["feat_mean"])
    base_std = list(base["feat_std"])
    out_w = base_w + [0.0] * (FEATURE_LEN_V3 - len(base_w))
    out_mean = base_mean + [0.0] * (FEATURE_LEN_V3 - len(base_mean))
    out_std = base_std + [1.0] * (FEATURE_LEN_V3 - len(base_std))
    old_vocab = [int(v) for v in base["vocab"]]
    zone_rows: list[list[float]] = []
    for z in range(5):
        zone_rows.append(_reindex_zone_row(base["linear"]["zone_w"][z], old_vocab, vocab))
    full_zw = np.zeros((13, n_vocab), dtype=np.float32)
    for z in range(5):
        full_zw[z] = np.asarray(zone_rows[z], dtype=np.float32)
    full_zw[z_lo:z_hi] = zw.astype(np.float32)

    spec: dict[str, Any] = {
        "arch": "linear",
        "feature_len": FEATURE_LEN_V3,
        "encoding": 3,
        "feat_mean": out_mean,
        "feat_std": out_std,
        "vocab": vocab,
        "zones": _zones_json(3),
        "scale": float(base.get("scale", SCALE)),
        "linear": {
            "w": out_w,
            "zone_w": full_zw.tolist(),
            "b": float(base["linear"]["b"]),
        },
        "trained_on": {
            "stack_on": str(base_path),
            "stack_on_sha256": hashlib.sha256(base_path.read_bytes()).hexdigest(),
            "dirs": [str(Path(d)) for d in args.data],
            "l2_grid": grid,
            "l2_choices": lams,
            "deck_controls": use_deck,
            "stack_search_rows": int(search_tr.size),
            "stack_timing": {
                "search_seconds": search_s,
                "train_refit_seconds": train_refit_s,
                "refit_seconds": refit_s,
            },
            "rows": [5, 6, 7, 8, 9, 10, 11, 12],
        },
    }
    if "race" in base:
        spec["race"] = base["race"]

    top_weights: dict[str, list[tuple[float, int]]] = {}
    for z in range(z_lo, z_hi):
        name = zones[z]["name"]
        pairs = [(float(full_zw[z, i]), vocab[i]) for i in range(1, n_vocab)]
        pairs.sort(key=lambda p: (-abs(p[0]), p[1]))
        top_weights[name] = pairs[:15]

    report: dict[str, Any] = {
        "arch": "linear",
        "stack_on": str(base_path),
        "holdout_mse_steps": step_mses,
        "holdout_mse_final": final_holdout_mse,
        "l2_choices": lams,
        "top_weights": top_weights,
        "warnings": warnings,
        "deck_controls": use_deck,
        "stack_timing": {
            "search_seconds": search_s,
            "train_refit_seconds": train_refit_s,
            "refit_seconds": refit_s,
        },
        "stack_search_rows": int(search_tr.size),
    }
    if use_deck and own_w is not None and opp_w is not None and first_w is not None:
        report["deck_theta"] = {
            "own": own_w.astype(np.float64).tolist(),
            "opp": opp_w.astype(np.float64).tolist(),
            "first": float(first_w),
        }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(_json_safe(spec)) + "\n")
    report_path = out.with_suffix(out.suffix + ".report.json")
    if out.suffix == ".json":
        report_path = out.with_name(out.stem + ".report.json")
    report_path.write_text(json.dumps(_json_safe(report), indent=2) + "\n")
    lines = [
        f"stack-on {base_path.name}",
        f"holdout MSE base={mse_base:.6f}",
    ]
    for name, mse in step_mses[1:]:
        lines.append(f"  after {name}: {mse:.6f}")
    lines.append(
        "  (written leaf refit uses all rows; its holdout score is in-sample)"
    )
    lines.append(f"l2 choices: {lams}")
    for wname, pairs in top_weights.items():
        lines.append(f"  {wname}: " + ", ".join(f"{cid}:{w:.4f}" for w, cid in pairs[:5]))
    for w in warnings:
        lines.append(f"warning: {w}")
    text = "\n".join(lines)
    print(text)
    print(f"wrote {out} and {report_path}")
    return report


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

    if getattr(args, "stack_on", None):
        return train_stack_on(args)

    use_race = bool(getattr(args, "race", False))
    if use_race and args.model == "mlp":
        raise SystemExit("--race is only supported with --model linear")
    if use_race and args.optimizer == "lbfgs":
        raise SystemExit("--optimizer lbfgs with --race is not supported")

    if args.optimizer == "lbfgs" and args.model != "linear":
        raise SystemExit("--optimizer lbfgs is only supported with --model linear")

    data = load_dirs(args.data, args.max_samples)
    encoding = int(data["encoding"])
    if encoding == 3 and args.model == "mlp":
        raise SystemExit("--model mlp is not supported with encoding 3")

    torch = _try_torch()
    if args.model == "mlp" and torch is None:
        raise SystemExit(
            "torch is required for --model mlp (linear can train with numpy gradients)"
        )
    if args.optimizer == "lbfgs" and torch is None:
        raise SystemExit("torch is required for --optimizer lbfgs")
    feature_len = int(data["feature_len"])
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
    race_mean = race_std = None
    if use_race:
        race_mean, race_std = _chunk_race_mean_std(features, train_idx, std_floor)
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
            encoding=encoding,
        )
        train_meta.update(lbfgs_meta)
        trained_on_stub: dict[str, Any] = {}
        spec = dump_torch(
            "linear",
            model,
            mean,
            std,
            vocab,
            trained_on_stub,
            feature_len,
            encoding,
            race_mean,
            race_std,
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
            use_race=use_race,
            race_mean=race_mean,
            race_std=race_std,
            encoding=encoding,
        )
        train_meta.update(adam_meta)
        trained_on_stub: dict[str, Any] = {}
        spec = dump_torch(
            args.model,
            model,
            mean,
            std,
            vocab,
            trained_on_stub,
            feature_len,
            encoding,
            race_mean,
            race_std,
        )
    else:
        w, zone_w, b, race_w, adam_meta = train_linear_numpy(
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
            use_race=use_race,
            race_mean=race_mean,
            race_std=race_std,
            encoding=encoding,
        )
        train_meta.update(adam_meta)
        spec = {
            "arch": "linear",
            "feature_len": feature_len,
            "encoding": encoding,
            "feat_mean": mean.tolist(),
            "feat_std": std.tolist(),
            "vocab": vocab,
            "zones": _zones_json(encoding),
            "scale": SCALE,
            "linear": {"w": w.tolist(), "zone_w": zone_w.tolist(), "b": b},
            "trained_on": {},
        }
        if use_race and race_w is not None:
            spec["race"] = {
                "version": 1,
                "names": list(RACE_NAMES),
                "mean": race_mean.tolist(),
                "std": race_std.tolist(),
                "w": race_w.tolist(),
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
        report_key, feat_prefix, note = _eval_holdout_plan(
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
                if feat_prefix is not None:
                    chunk_feat = chunk_feat[:, :feat_prefix]
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
        **({"race": True} if use_race else {}),
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
    p.add_argument(
        "--race",
        action="store_true",
        help="train optional race block (linear only; not with --optimizer lbfgs)",
    )
    p.add_argument(
        "--stack-on",
        default=None,
        help="encoding-2 linear base model for stacked encoding-3 fit",
    )
    p.add_argument(
        "--no-deck-controls",
        action="store_true",
        help="omit per-deck controls during --stack-on (requires shard decks metadata)",
    )
    p.add_argument(
        "--l2-grid",
        default="1e-6,1e-5,1e-4,1e-3,1e-2,1e-1,1",
        help="comma-separated L2 grid for --stack-on groups",
    )
    p.add_argument(
        "--stack-search-rows",
        type=int,
        default=STACK_SEARCH_ROWS_DEFAULT,
        help="max training rows for --stack-on penalty search (whole games)",
    )
    args = p.parse_args(argv)
    if args.stack_on:
        if args.model != "linear":
            raise SystemExit("--stack-on requires --model linear")
        if args.race:
            raise SystemExit("--stack-on cannot be used with --race")
        if args.optimizer == "lbfgs":
            raise SystemExit("--stack-on cannot be used with --optimizer lbfgs")
        if args.l2 != 1e-4:
            raise SystemExit("--stack-on cannot be used with --l2 (use --l2-grid)")
    train(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
