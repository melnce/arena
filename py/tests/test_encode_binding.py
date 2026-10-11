"""`Game.vocab`, `Game.encode`, and `Game.evaluate` vs export and train_value."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

import pytest

np = pytest.importorskip("numpy")

ROOT = Path(__file__).resolve().parents[2]
V3_NET = ROOT / "engine" / "models" / "h0-linear-v4.json"
V2_NET = ROOT / "engine" / "models" / "h0-linear-v3.json"

_SAMPLES = Path(__file__).resolve().parents[1] / "samples.py"
_spec = importlib.util.spec_from_file_location("arena_samples", _SAMPLES)
assert _spec and _spec.loader
samples = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(samples)

_TRAIN = Path(__file__).resolve().parents[1] / "train_value.py"
_tspec = importlib.util.spec_from_file_location("arena_train_value", _TRAIN)
assert _tspec and _tspec.loader
train_value = importlib.util.module_from_spec(_tspec)
_tspec.loader.exec_module(train_value)

# Version-3 histogram blocks in `features` (see `engine/src/encode.rs` layout).
HIST_V3 = (
    (353, 96),  # own_deck_hist
    (449, 96),  # opp_known_pool_hist
    (577, 96),  # own_entered_hist
    (673, 96),  # opp_entered_hist
    (769, 96),  # own_cemetery_hist
    (865, 96),  # opp_cemetery_hist
)


def _acting_player(g) -> str:
    """Seat whose policy runs `choose` (differs from `active` during mulligan/choice)."""
    leg = g.legal()[0]
    if g.phase == "mulligan":
        return leg["mulligan"]["player"]
    if g.phase == "choice" and "choose" in leg:
        return leg["choose"]["player"]
    return g.active


def _load_deck(root: Path, name: str) -> dict[str, int]:
    path = root / "oracle" / "decks" / f"{name}.json"
    return {str(k): int(v) for k, v in json.loads(path.read_text()).items()}


def _export_index(data: dict, game_index: int, side: int, decision_index: int) -> int:
    aux = data["aux"]
    cols = data["aux_columns"]
    gi = aux[:, cols.index("game_index")].astype(int)
    sd = aux[:, cols.index("side")].astype(int)
    di = aux[:, cols.index("decision_index")].astype(int)
    hits = np.where((gi == game_index) & (sd == side) & (di == decision_index))[0]
    assert hits.size == 1, (game_index, side, decision_index, hits.size)
    return int(hits[0])


def _replay_export_parity(
    db,
    root: Path,
    tmp_path: Path,
    *,
    encoding: int,
    games: int,
) -> None:
    import arena

    deck_a = _load_deck(root, "basic-forest")
    deck_b = _load_deck(root, "basic-rune")
    decks = {"basic-forest": deck_a, "basic-rune": deck_b}
    out = tmp_path / f"enc{encoding}"
    result = arena.matchup(
        db,
        decks,
        games,
        91,
        policy="first-legal",
        records=True,
        threads=1,
        export=str(out),
        encoding=encoding,
    )
    data = samples.load(out)
    feat_len = int(data["meta"]["feature_len"])
    ids_len = int(data["meta"]["ids_len"])
    names = sorted(decks.keys())
    n_decks = len(names)

    for rec in result["records"]:
        g = arena.Game(
            db,
            rec["seed"],
            decks[rec["a"]],
            decks[rec["b"]],
            first=rec["first"],
        )
        i, j = names.index(rec["a"]), names.index(rec["b"])
        game_index = (i * n_decks + j) * games + rec["g"]
        decisions = {"a": 0, "b": 0}
        for _ in range(int(rec["actions"])):
            actor = _acting_player(g)
            enc = g.encode(actor, encoding)
            side = 0 if actor == "a" else 1
            row = _export_index(data, game_index, side, decisions[actor])
            exp_f = data["features"][row]
            exp_i = data["ids"][row]
            assert len(enc["features"]) == feat_len
            assert len(enc["ids"]) == ids_len
            np.testing.assert_array_equal(np.asarray(enc["features"], dtype=np.float32), exp_f)
            np.testing.assert_array_equal(np.asarray(enc["ids"], dtype=np.uint32), exp_i)
            legal = g.legal()
            assert legal, f"no legal with actions remaining at g={rec['g']}"
            g.apply(legal[0])
            decisions[actor] += 1


def test_export_encoding_v3_parity(db, root: Path, tmp_path: Path) -> None:
    _replay_export_parity(db, root, tmp_path, encoding=3, games=2)


def test_export_encoding_v2_parity(db, root: Path, tmp_path: Path) -> None:
    _replay_export_parity(db, root, tmp_path, encoding=2, games=1)


def test_vocab_encode_and_histogram_blocks(db, root: Path) -> None:
    import arena

    deck = _load_deck(root, "basic-forest")
    g = arena.Game(db, 7, deck, deck, first="a")
    # Advance past mulligan if needed.
    while g.phase == "mulligan":
        g.apply(g.legal()[0])
    player = _acting_player(g)
    full = g.encode(player, 3)
    with_vocab = g.encode(player, 3, vocab=g.vocab())
    assert full["features"] == with_vocab["features"]
    assert full["ids"] == with_vocab["ids"]

    voc = list(g.vocab())
    assert len(voc) >= 2
    trimmed = voc[1:]
    reduced = g.encode(player, 3, vocab=trimmed)
    fa = np.asarray(full["features"], dtype=np.float32)
    fb = np.asarray(reduced["features"], dtype=np.float32)
    hist_mask = np.zeros(fa.shape[0], dtype=bool)
    for off, width in HIST_V3:
        hist_mask[off : off + width] = True
    np.testing.assert_array_equal(fa[~hist_mask], fb[~hist_mask])
    assert not np.array_equal(fa[hist_mask], fb[hist_mask])

    bad = list(reversed(voc))
    with pytest.raises(ValueError, match="strictly ascending"):
        g.encode(player, 3, vocab=bad)


def _collect_positions(db, root: Path, n: int = 12) -> list[tuple]:
    import arena

    deck_a = _load_deck(root, "basic-forest")
    deck_b = _load_deck(root, "basic-rune")
    g = arena.Game(db, 44, deck_a, deck_b, first="coin")
    out: list[tuple] = []
    while not g.terminal and len(out) < n:
        out.append((g.clone(), _acting_player(g)))
        g.apply(g.legal()[0])
    return out


def test_evaluate_matches_train_value_predict(db, root: Path) -> None:
    import arena

    spec = json.loads(V2_NET.read_text())
    policy = f"h0:net={V2_NET},clip=0"
    positions = _collect_positions(db, root, n=12)
    assert len(positions) >= 10
    for g, player in positions:
        enc = g.encode(player, 2)
        feats = np.asarray([enc["features"]], dtype=np.float32)
        ids = np.asarray([enc["ids"]], dtype=np.uint32)
        py_v = float(train_value.predict(spec, feats, ids)[0])
        eng_v = g.evaluate(policy, player)
        assert abs(eng_v - py_v) <= 1e-4, (eng_v, py_v)


def test_evaluate_builtin_matches_v4_net(db, root: Path) -> None:
    import arena

    explicit = f"h0:net={V3_NET}"
    positions = _collect_positions(db, root, n=12)
    for g, player in positions:
        v_builtin = g.evaluate("h0", player)
        v_v4 = g.evaluate(explicit, player)
        assert v_builtin == v_v4


def test_encode_evaluate_errors(db, root: Path) -> None:
    import arena

    deck = _load_deck(root, "basic-forest")
    g = arena.Game(db, 1, deck, deck, first="a")
    while g.phase == "mulligan":
        g.apply(g.legal()[0])
    p = g.active
    with pytest.raises(ValueError, match='player must be "a" or "b"'):
        g.encode("x", 3)
    with pytest.raises(ValueError, match="version must be"):
        g.encode(p, 9)
    with pytest.raises(ValueError, match="h0"):
        g.evaluate("random", p)
    with pytest.raises(ValueError, match='player must be "a" or "b"'):
        g.evaluate("h0", "z")
