"""Python `crest_order` switch vs the pre-fix reference fixture.

Replay digests and h0 explain summaries must match
`fixtures/crest_order_reference.json` under `crest_order="as_recorded"`.
Default `"rulebook"` diverges only on Slaus countdown games. Never regenerate
the fixture here — it was made on `bcfb6cd` (pre-fix main).
"""

from __future__ import annotations

import hashlib
import itertools
import json
import random
import time
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = Path(__file__).resolve().parent / "fixtures" / "crest_order_reference.json"
SLAUS_CREST = "crest:10574110"
EXPLAIN_SPEC = "h0:nodes=3000,horizon=3,k=4"
EXPLAIN_STEPS = [39, 40, 41, 42, 43]


def rnd(x, nd):
    if isinstance(x, float):
        return round(x, nd)
    if isinstance(x, list):
        return [rnd(v, nd) for v in x]
    if isinstance(x, dict):
        return {k: rnd(v, nd) for k, v in x.items()}
    return x


def h(x):
    return hashlib.sha256(json.dumps(x, sort_keys=True, default=str).encode()).hexdigest()[:16]


@pytest.fixture(scope="module")
def ref():
    return json.loads(FIXTURE.read_text())


@pytest.fixture(scope="module")
def pools_and_decks():
    pools = json.loads((ROOT / "oracle/decks/POOLS.json").read_text())
    stems = sorted(set(pools["meta"]))
    decks = {s: json.loads((ROOT / "oracle/decks" / f"{s}.json").read_text()) for s in stems}
    pairs = list(itertools.product(stems, stems))
    return decks, pairs


@pytest.fixture(scope="module")
def db():
    import arena

    return arena.load_cards(str(ROOT / "cards"))


def new_game(db, decks, pairs, gi, k, crest_order=None):
    import arena

    a, b = pairs[gi]
    seed = 1000003 * gi + 17 * k + 5
    kw = {"first": ("a" if k % 2 == 0 else "b")}
    if crest_order:
        kw["crest_order"] = crest_order
    return arena.Game(db, seed, decks[a], decks[b], **kw), seed, (a, b)


def replay_digest(db, decks, pairs, gi, k, crest_order=None):
    g, seed, ab = new_game(db, decks, pairs, gi, k, crest_order=crest_order)
    rng = random.Random(seed)
    dig = hashlib.sha256()
    steps = 0
    while not g.terminal and steps < 600:
        legal = g.legal()
        enc0 = rnd(g.encode("a", 3, vocab=g.vocab()), 6)
        enc1 = rnd(g.encode("b", 3, vocab=g.vocab()), 6)
        dig.update(f"{g.hash():016x}|{h(legal)}|{h(enc0)}|{h(enc1)}|{h(g.full())}".encode())
        if not legal:
            break
        g.apply(legal[rng.randrange(len(legal))])
        steps += 1
    return {
        "pair": list(ab),
        "pair_index": gi,
        "k": k,
        "seed": seed,
        "steps": steps,
        "final_hash": f"{g.hash():016x}",
        "digest": dig.hexdigest()[:24],
    }


def explain_summary(r):
    return {
        "chosen_index": r.get("chosen_index"),
        "path": r.get("path"),
        "nodes": r.get("nodes"),
        "horizon_nodes": r.get("horizon_nodes"),
        "nodes_lethal": r.get("nodes_lethal"),
        "tie_set": r.get("tie_set"),
        "value": rnd(r.get("value"), 4),
        "candidates": [
            {
                "legal_index": c.get("legal_index"),
                "n": c.get("n"),
                "root_agg": rnd(c.get("root_agg"), 4),
            }
            for c in r.get("candidates") or []
        ],
    }


def explains(db, decks, pairs, gi, k, crest_order=None):
    g, seed, _ = new_game(db, decks, pairs, gi, k, crest_order=crest_order)
    rng = random.Random(seed)
    rows, step = [], 0
    while not g.terminal and step <= max(EXPLAIN_STEPS):
        legal = g.legal()
        if step in EXPLAIN_STEPS:
            r = g.clone().bot_action_explain(EXPLAIN_SPEC, seed + step)
            rows.append({"step": step, "state_hash": f"{g.hash():016x}", **explain_summary(r)})
        g.apply(legal[rng.randrange(len(legal))])
        step += 1
    return rows


def slaus_crest(full, player):
    for c in full["players"][player]["crests"]:
        if c.get("id") == SLAUS_CREST:
            return c
    return None


def test_as_recorded_matches_fixture(db, pools_and_decks, ref):
    decks, pairs = pools_and_decks
    t0 = time.perf_counter()
    for expected in ref["games"]:
        got = replay_digest(
            db, decks, pairs, expected["pair_index"], expected["k"], crest_order="as_recorded"
        )
        assert got["steps"] == expected["steps"], (expected["pair_index"], expected["k"], got, expected)
        assert got["final_hash"] == expected["final_hash"], (
            expected["pair_index"],
            expected["k"],
            got["final_hash"],
            expected["final_hash"],
        )
        assert got["digest"] == expected["digest"], (
            expected["pair_index"],
            expected["k"],
            got["digest"],
            expected["digest"],
        )

    gi, k = ref["explain"]["game"]
    rows = explains(db, decks, pairs, gi, k, crest_order="as_recorded")
    assert len(rows) == len(ref["explain"]["rows"])
    for got, expected in zip(rows, ref["explain"]["rows"]):
        assert got == expected, (got["step"], got, expected)
    elapsed = time.perf_counter() - t0
    print(f"test_as_recorded_matches_fixture elapsed={elapsed:.1f}s")


def test_rulebook_differs_only_on_slaus_games(db, pools_and_decks, ref):
    decks, pairs = pools_and_decks
    slaus = {tuple(x) for x in ref["slaus_games"]}
    t0 = time.perf_counter()
    for expected in ref["games"]:
        key = (expected["pair_index"], expected["k"])
        got = replay_digest(db, decks, pairs, expected["pair_index"], expected["k"])
        if key in slaus:
            assert got["digest"] != expected["digest"], f"slaus game {key} should differ under rulebook"
        else:
            assert got["steps"] == expected["steps"], key
            assert got["final_hash"] == expected["final_hash"], key
            assert got["digest"] == expected["digest"], key

    gi, k = ref["explain"]["game"]
    rows = explains(db, decks, pairs, gi, k)  # default rulebook
    assert any(got != exp for got, exp in zip(rows, ref["explain"]["rows"])), (
        "expected at least one explain row to differ under rulebook"
    )
    elapsed = time.perf_counter() - t0
    print(f"test_rulebook_differs_only_on_slaus_games elapsed={elapsed:.1f}s")


def test_invisibility_until_slaus_expiry(db, pools_and_decks):
    decks, pairs = pools_and_decks
    g_rec, seed, _ = new_game(db, decks, pairs, 145, 0, crest_order="as_recorded")
    g_live, _, _ = new_game(db, decks, pairs, 145, 0, crest_order="rulebook")
    rng = random.Random(seed)
    step = 0
    first_diff = None
    expiry_step = None
    while not g_rec.terminal and not g_live.terminal and step < 600:
        assert g_rec.hash() == g_live.hash(), f"hash diverge before apply at step {step}"
        assert g_rec.full() == g_live.full(), f"full diverge before apply at step {step}"
        vocab = g_rec.vocab()
        assert g_rec.encode("a", 3, vocab=vocab) == g_live.encode("a", 3, vocab=vocab)
        assert g_rec.encode("b", 3, vocab=vocab) == g_live.encode("b", 3, vocab=vocab)

        full_before = g_rec.full()
        slaus_before = slaus_crest(full_before, "a") or slaus_crest(full_before, "b")
        cd1_before = slaus_before is not None and slaus_before.get("countdown") == 1

        legal = g_rec.legal()
        if not legal:
            break
        action = legal[rng.randrange(len(legal))]
        ev_live = g_live.apply(action)
        g_rec.apply(action)
        step += 1

        slaus_after = slaus_crest(g_rec.full(), "a") or slaus_crest(g_rec.full(), "b")
        if cd1_before and slaus_after is None and expiry_step is None:
            expiry_step = step
            assert any(
                isinstance(e, dict) and "crest_turn_start_countdown_lost" in e for e in (ev_live or [])
            ), "rulebook SoT crest expiry emits CrestTurnStartCountdownLost"

        if g_rec.hash() != g_live.hash() and first_diff is None:
            first_diff = step
            break

    assert first_diff is not None, "expected rulebook and as_recorded to diverge"
    assert expiry_step is not None, "expected to observe Slaus CD1 expiry"
    assert first_diff == expiry_step, (
        f"first differing step {first_diff} must be the Slaus CD1 expiry step {expiry_step}"
    )


def test_clone_and_property_and_invalid(db, pools_and_decks):
    import arena

    decks, pairs = pools_and_decks
    g = new_game(db, decks, pairs, 0, 0, crest_order="as_recorded")[0]
    assert g.crest_order == "as_recorded"
    c = g.clone()
    assert c.crest_order == "as_recorded"
    g_def = new_game(db, decks, pairs, 0, 0)[0]
    assert g_def.crest_order == "rulebook"
    assert g_def.clone().crest_order == "rulebook"
    a, b = pairs[0]
    with pytest.raises(ValueError):
        arena.Game(db, 1, decks[a], decks[b], first="a", crest_order="legacy")
