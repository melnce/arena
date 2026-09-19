"""Blunder analyser (`py/review.py`)."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import pytest

pytest.importorskip("arena")

_REVIEW = Path(__file__).resolve().parents[1] / "review.py"
_spec = importlib.util.spec_from_file_location("arena_review", _REVIEW)
assert _spec and _spec.loader
review = importlib.util.module_from_spec(_spec)
sys.modules["arena_review"] = review
_spec.loader.exec_module(review)

TINY_REF = "h0:depth=3,beam=2,k=1,nodes=400"


def _decks(root: Path) -> tuple[dict[str, int], dict[str, int]]:
    forest = json.loads((root / "oracle" / "decks" / "basic-forest.json").read_text())
    rune = json.loads((root / "oracle" / "decks" / "basic-rune.json").read_text())
    return {str(k): int(v) for k, v in forest.items()}, {str(k): int(v) for k, v in rune.items()}


def _play_first_legal(db, deck_a, deck_b, seed: int = 1, plies: int = 16):
    import arena

    game = arena.Game(db, seed, deck_a, deck_b, "a")
    actions = []
    for i in range(plies):
        if game.terminal:
            break
        act = game.bot_action("first-legal", seed + i)
        actions.append(act)
        game.apply(act)
    return game, actions


def _capture(game_id: str, seed: int, deck_a, deck_b, actions, **extra) -> dict:
    rec = {
        "v": 1,
        "game_id": game_id,
        "seed": seed,
        "deckA": deck_a,
        "deckB": deck_b,
        "first": "a",
        "actions": actions,
        "policy": "first-legal",
        "strong": "h0:nodes=16000",
        "engine": "test",
        "updated": "2026-01-01T00:00:00Z",
        "final": True,
        "winner": extra.pop("winner", None),
    }
    rec.update(extra)
    return rec


def _assert_decision_shape(d: dict) -> None:
    for key in (
        "ply",
        "turn",
        "phase",
        "acting",
        "played",
        "reference",
        "v_played",
        "v_best",
        "bot_value",
        "position",
    ):
        assert key in d, key
    assert d["v_played"] is not None
    assert d["v_best"] is not None
    if review.passes_turn(d.get("played")):
        assert "optimism" in d
        assert "cost" not in d
    else:
        assert "cost" in d
        assert "delta" in d
        assert float(d["cost"]) >= 0


def test_seed_for_is_deterministic() -> None:
    a = review.seed_for("1-deadbeef", 7)
    b = review.seed_for("1-deadbeef", 7)
    c = review.seed_for("1-deadbeef", 7, 1)
    assert a == b
    assert a != c
    assert 0 <= a < 2**64


def test_review_synthetic_game(db, root: Path, tmp_path: Path) -> None:
    deck_a, deck_b = _decks(root)
    game, actions = _play_first_legal(db, deck_a, deck_b, seed=3, plies=20)
    gid = "3-synth001"
    log = _capture(gid, 3, deck_a, deck_b, actions, winner=game.winner)
    games_dir = tmp_path / "games"
    games_dir.mkdir()
    (games_dir / f"{gid}.json").write_text(json.dumps(log, indent=2) + "\n")
    out = tmp_path / "out"
    rc = review.main(
        [
            "--games",
            str(games_dir),
            "--ref",
            TINY_REF,
            "--side",
            "bot",
            "--tag",
            "t1",
            "--out",
            str(out),
            "--threads",
            "2",
            "--cards",
            str(root / "cards"),
        ]
    )
    assert rc == 0
    dest = out / f"{gid}.json"
    assert dest.is_file()
    first = dest.read_text(encoding="utf-8")
    payload = json.loads(first)
    assert payload["game_id"] == gid
    assert payload["ref"] == TINY_REF
    assert isinstance(payload["decisions"], list)
    assert payload["decisions"], "expected at least one analysed bot decision"
    for d in payload["decisions"]:
        _assert_decision_shape(d)
        assert d["acting"] == "b"
        if "cost" in d:
            assert float(d["cost"]) >= 0
        else:
            assert "optimism" in d
    assert payload["clamped_negative"] >= 0
    run = json.loads((out / "RUN.json").read_text())
    assert run["human_side_assumed"] is True

    rc = review.main(
        [
            "--games",
            str(games_dir),
            "--ref",
            TINY_REF,
            "--side",
            "bot",
            "--tag",
            "t1",
            "--out",
            str(out),
            "--threads",
            "2",
            "--cards",
            str(root / "cards"),
            "--force",
        ]
    )
    assert rc == 0
    second = dest.read_text(encoding="utf-8")
    assert first == second

    rc = review.main(
        [
            "--games",
            str(games_dir),
            "--ref",
            TINY_REF,
            "--side",
            "bot",
            "--tag",
            "t1",
            "--out",
            str(out),
            "--threads",
            "2",
            "--cards",
            str(root / "cards"),
        ]
    )
    assert rc == 0
    assert dest.read_text(encoding="utf-8") == first
    assert (out / "REVIEW.md").is_file()
    assert (out / "RUN.json").is_file()


def test_worst_legal_has_positive_cost(db, root: Path) -> None:
    deck_a, deck_b = _decks(root)
    found = None
    for seed in range(1, 12):
        game, actions = _play_first_legal(db, deck_a, deck_b, seed=seed, plies=24)
        log = _capture(f"{seed}-worst", seed, deck_a, deck_b, actions, winner=game.winner)
        for ply, step in enumerate(actions):
            if not isinstance(step, dict) or "reseed" in step:
                continue
            if review.action_player(step) != "b":
                continue
            pos = review.rebuild_to(db, log, ply)
            if pos.phase == "mulligan" or pos.terminal or len(pos.legal()) <= 1:
                continue
            legal = pos.legal()
            ref = pos.bot_action_value(TINY_REF, review.seed_for(log["game_id"], ply))
            if ref["value"] is None:
                continue
            worst = None
            worst_v = float("inf")
            for act in legal:
                clone = pos.clone()
                clone.apply(act)
                if clone.terminal:
                    v = 80.0 if clone.winner == "b" else -80.0
                else:
                    after = clone.bot_action_value(
                        TINY_REF, review.seed_for(log["game_id"], ply, 1)
                    )
                    if after["value"] is None:
                        continue
                    v = float(after["value"])
                    if review.acting_of(clone) != "b":
                        v = -v
                if v < worst_v:
                    worst_v = v
                    worst = act
            if worst is None or worst == ref["action"]:
                continue
            if review.passes_turn(worst):
                continue
            if float(ref["value"]) - worst_v <= 0:
                continue
            new_actions = actions[:ply] + [worst]
            cont = pos.clone()
            cont.apply(worst)
            n = 0
            while not cont.terminal and n < 8:
                nxt = cont.bot_action("first-legal", seed + 100 + n)
                new_actions.append(nxt)
                cont.apply(nxt)
                n += 1
            found = _capture(
                f"{seed}-worst",
                seed,
                deck_a,
                deck_b,
                new_actions,
                winner=cont.winner,
            )
            found["_ply"] = ply
            break
        if found:
            break
    assert found is not None, "could not construct a worst-legal decision"
    ply = found.pop("_ply")
    result = review.analyse_game(
        db, found, ref=TINY_REF, side="bot", threads=1, names={}
    )
    match = [d for d in result["decisions"] if d["ply"] == ply]
    assert match, result["decisions"]
    assert float(match[0]["cost"]) > 0


def test_side_human_analyses_other_seat(db, root: Path) -> None:
    deck_a, deck_b = _decks(root)
    game, actions = _play_first_legal(db, deck_a, deck_b, seed=5, plies=18)
    log = _capture("5-human", 5, deck_a, deck_b, actions, winner=game.winner)
    bot = review.analyse_game(db, log, ref=TINY_REF, side="bot", threads=1, names={})
    human = review.analyse_game(db, log, ref=TINY_REF, side="human", threads=1, names={})
    assert bot["decisions"]
    assert human["decisions"]
    assert {d["acting"] for d in bot["decisions"]} == {"b"}
    assert {d["acting"] for d in human["decisions"]} == {"a"}


def test_rerun_skips_unless_force(db, root: Path, tmp_path: Path) -> None:
    deck_a, deck_b = _decks(root)
    game, actions = _play_first_legal(db, deck_a, deck_b, seed=2, plies=14)
    gid = "2-skipme"
    log = _capture(gid, 2, deck_a, deck_b, actions, winner=game.winner)
    games_dir = tmp_path / "games"
    games_dir.mkdir()
    (games_dir / f"{gid}.json").write_text(json.dumps(log) + "\n")
    out = tmp_path / "out"
    argv = [
        "--games",
        str(games_dir),
        "--ref",
        TINY_REF,
        "--tag",
        "skip",
        "--out",
        str(out),
        "--threads",
        "1",
        "--cards",
        str(root / "cards"),
    ]
    assert review.main(argv) == 0
    dest = out / f"{gid}.json"
    first = dest.read_text(encoding="utf-8")
    stamp = dest.stat().st_mtime_ns
    assert review.main(argv) == 0
    assert dest.stat().st_mtime_ns == stamp
    assert dest.read_text(encoding="utf-8") == first
    assert review.main([*argv, "--force"]) == 0
    assert dest.stat().st_mtime_ns >= stamp
    assert json.loads(dest.read_text())["decisions"]


def test_turn_passing_emits_optimism_and_no_cost() -> None:
    out = review.decorate_decision(
        {
            "ply": 3,
            "turn": 2,
            "phase": "main",
            "acting": "b",
            "played": {"end_turn": {"player": "b"}},
            "reference": {"play": {"card": "x", "player": "b", "hand_pos": 0}},
            "v_best": 10.0,
            "v_played": -27.2,
        }
    )
    assert "cost" not in out
    assert out["optimism"] == pytest.approx(37.2)


def test_same_action_contributes_to_noise_floor() -> None:
    same = review.decorate_decision(
        {
            "ply": 1,
            "turn": 1,
            "played": {"play": {"card": "x", "player": "b", "hand_pos": 0}},
            "reference": {"play": {"card": "x", "player": "b", "hand_pos": 0}},
            "v_best": 5.0,
            "v_played": 3.0,
        }
    )
    end = review.decorate_decision(
        {
            "ply": 2,
            "turn": 1,
            "played": {"end_turn": {"player": "b"}},
            "reference": {"end_turn": {"player": "b"}},
            "v_best": 40.0,
            "v_played": 0.0,
        }
    )
    other = review.decorate_decision(
        {
            "ply": 3,
            "turn": 1,
            "played": {"play": {"card": "y", "player": "b", "hand_pos": 1}},
            "reference": {"play": {"card": "z", "player": "b", "hand_pos": 1}},
            "v_best": 8.0,
            "v_played": 1.0,
        }
    )
    stats = review.review_stats(
        [{"decisions": [same, end, other]}],
        wv=80.0,
        mean_ms=0.0,
        clamped_total=0,
    )
    assert stats["noise_n"] == 1
    assert stats["noise_mean"] == pytest.approx(2.0)
    assert stats["analysed_end_turns"] == 1
    assert "cost" not in end
    md = review.build_review_md(
        [{"game_id": "g", "decisions": [same, end, other], "ref": "h0:wv=80"}],
        ref="h0:wv=80",
        top=5,
        names={},
        name_source="ids",
        clamped_total=0,
        mean_ms=0.0,
    )
    assert "mean end-of-turn optimism" in md
    assert "noise floor" in md
    for ln in md.splitlines():
        if ln[:2].rstrip(".").isdigit() or (len(ln) > 2 and ln[0].isdigit() and ln[1] == "."):
            assert "played end-turn" not in ln
            if "cost=" in ln:
                assert "end-turn" not in ln


def test_human_side_b_scores_seat_a_as_bot(db, root: Path) -> None:
    deck_a, deck_b = _decks(root)
    game, actions = _play_first_legal(db, deck_a, deck_b, seed=5, plies=18)
    log = _capture(
        "5-side-b", 5, deck_a, deck_b, actions, winner=game.winner, humanSide="b"
    )
    bot = review.analyse_game(db, log, ref=TINY_REF, side="bot", threads=1, names={})
    assert bot["decisions"]
    assert {d["acting"] for d in bot["decisions"]} == {"a"}


def test_missing_human_side_warns_and_sets_assumed(
    db, root: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    deck_a, deck_b = _decks(root)
    game, actions = _play_first_legal(db, deck_a, deck_b, seed=4, plies=14)
    gid = "4-assume"
    log = _capture(gid, 4, deck_a, deck_b, actions, winner=game.winner)
    games_dir = tmp_path / "games"
    games_dir.mkdir()
    (games_dir / f"{gid}.json").write_text(json.dumps(log) + "\n")
    out = tmp_path / "out"
    rc = review.main(
        [
            "--games",
            str(games_dir),
            "--ref",
            TINY_REF,
            "--tag",
            "assume",
            "--out",
            str(out),
            "--threads",
            "1",
            "--cards",
            str(root / "cards"),
        ]
    )
    assert rc == 0
    captured = capsys.readouterr()
    assert review.HUMAN_SIDE_WARN in captured.out
    run = json.loads((out / "RUN.json").read_text())
    assert run["human_side_assumed"] is True
    assert run["review"]["human_side_assumed"] is True
