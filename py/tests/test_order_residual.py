"""Residual order net: numpy reference vs ``Game.evaluate``."""

from __future__ import annotations

import bisect
import json
from pathlib import Path

import pytest

np = pytest.importorskip("numpy")
pytest.importorskip("arena")

ROOT = Path(__file__).resolve().parents[2]
TINY = (
    Path(__file__).resolve().parent / "fixtures" / "order_residual" / "tiny-residual.json"
)
REVIEW_DIRS = [
    Path(__file__).resolve().parent / "fixtures" / "review13" / "games",
    Path(__file__).resolve().parent / "fixtures" / "review14" / "games",
]


def _id_index(vocab: list[int], card_id: int) -> int:
    i = bisect.bisect_left(vocab, card_id)
    if i < len(vocab) and vocab[i] == card_id:
        return i
    return 0


def _slot_count(features: np.ndarray, zone: dict, slot: int) -> float:
    h = zone.get("hist_offset")
    if h is None:
        return 1.0
    return float(features[h + slot])


def residual_value(
    model: dict,
    features: np.ndarray,
    ids: np.ndarray,
    *,
    clip: float,
) -> float:
    mean = np.asarray(model["feat_mean"], dtype=np.float32)
    std = np.asarray(model["feat_std"], dtype=np.float32)
    f = np.asarray(features, dtype=np.float32)
    x = (f - mean) / std
    if clip > 0:
        x = np.clip(x, -clip, clip)
    x = x.astype(np.float32)

    vocab = model["vocab"]
    zones = model["zones"]
    lin = model["linear"]
    pre = np.float32(lin["b"])
    w = np.asarray(lin["w"], dtype=np.float32)
    for i in range(len(x)):
        pre = np.float32(pre + np.float32(w[i] * x[i]))
    for z, zone in enumerate(zones):
        table = np.asarray(lin["zone_w"][z], dtype=np.float32)
        off = zone["id_offset"]
        for slot in range(zone["count"]):
            count = _slot_count(f, zone, slot)
            if zone.get("hist_offset") is not None and count == 0.0:
                continue
            idx = _id_index(vocab, int(ids[off + slot]))
            pre = np.float32(pre + np.float32(count * table[idx]))

    mlp = model["mlp"]
    emb_dim = len(mlp["emb"][0][0])
    emb_parts: list[np.ndarray] = []
    for z, zone in enumerate(zones):
        ez = np.zeros(emb_dim, dtype=np.float32)
        table = mlp["emb"][z]
        off = zone["id_offset"]
        for slot in range(zone["count"]):
            count = _slot_count(f, zone, slot)
            if zone.get("hist_offset") is not None and count == 0.0:
                continue
            idx = _id_index(vocab, int(ids[off + slot]))
            ez += count * np.asarray(table[idx], dtype=np.float32)
        emb_parts.append(ez)
    e = np.concatenate(emb_parts)
    w1 = np.asarray(mlp["w1"], dtype=np.float32)
    b1 = np.asarray(mlp["b1"], dtype=np.float32)
    w2 = np.asarray(mlp["w2"], dtype=np.float32)
    hidden = w1.shape[0]
    feat_len = len(x)
    if clip in (0.0, 5.0):
        x0 = (-mean / std).astype(np.float32)
        if clip > 0:
            x0 = np.clip(x0, -clip, clip).astype(np.float32)
        h = np.zeros(hidden, dtype=np.float32)
        for k in range(hidden):
            acc = np.float32(b1[k])
            for i in range(feat_len):
                acc = np.float32(acc + np.float32(w1[k, i] * x0[i]))
            for i in range(feat_len):
                if f[i] != 0:
                    acc = np.float32(acc + np.float32(w1[k, i] * (x[i] - x0[i])))
            for j in range(len(e)):
                acc = np.float32(acc + np.float32(w1[k, feat_len + j] * e[j]))
            h[k] = max(np.float32(0), acc)
    else:
        inp = np.concatenate([x, e])
        h = np.maximum(np.float32(0), w1 @ inp + b1)
    mlp_out = np.float32(mlp["b2"])
    for k in range(len(w2)):
        mlp_out = np.float32(mlp_out + np.float32(w2[k] * h[k]))
    pre = np.float32(pre + mlp_out)
    return float(np.float32(model["scale"]) * np.tanh(pre))


def _replay(db, cap: dict, n: int):
    import arena

    game = arena.Game(
        db,
        cap["seed"],
        cap["deckA"],
        cap["deckB"],
        cap.get("first") or "coin",
    )
    for step in cap["actions"][:n]:
        body = {k: v for k, v in step.items() if k not in ("value", "bot_value")}
        game.apply(body)
    return game


def _collect_games(db, min_positions: int = 200) -> list[tuple]:
    out: list[tuple] = []
    for games_dir in REVIEW_DIRS:
        if not games_dir.is_dir():
            continue
        for path in sorted(games_dir.glob("*.json")):
            cap = json.loads(path.read_text())
            n_act = len(cap["actions"])
            for ply in range(4, n_act + 1):
                try:
                    g = _replay(db, cap, ply)
                except Exception:
                    continue
                if g.terminal:
                    continue
                player = g.active
                out.append((g, player))
                if len(out) >= min_positions:
                    return out
    assert len(out) >= min_positions, f"only {len(out)} positions"
    return out


@pytest.fixture(scope="module")
def tiny_model() -> dict:
    return json.loads(TINY.read_text())


def test_residual_parity_on_review_positions(db, tiny_model: dict) -> None:
    net_path = str(TINY.resolve())
    specs = [
        (f"h0:net={net_path}", 5.0),
        (f"h0:net={net_path},clip=0", 0.0),
    ]
    for game, player in _collect_games(db, 200):
        enc = game.encode(player, 3)
        f_arr = np.asarray(enc["features"], dtype=np.float32)
        id_arr = np.asarray(enc["ids"], dtype=np.uint32)
        for spec, clip in specs:
            ref = residual_value(tiny_model, f_arr, id_arr, clip=clip)
            got = float(game.evaluate(spec, player))
            assert abs(got - ref) <= 1e-4, (got, ref, spec)
